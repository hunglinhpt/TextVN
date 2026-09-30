// SPDX-License-Identifier: GPL-3.0-or-later
//! CoreBridge — wrapper Swift an toàn quanh C-ABI `textvn_ffi.h` (P0-2).
//!
//! Bất biến giữ nguyên (P0-2 §0):
//! - Không alloc chéo ranh giới: mọi struct `*_v1` nằm trên **stack Swift**,
//!   Rust không trả pointer nào chứa dữ liệu người dùng.
//! - Fail-open: `ime_key` lỗi → PASS + cờ ERROR (engine tự đảm bảo qua
//!   `catch_unwind`; bridge chỉ cần đọc đúng).
//! - 1 instance = 1 thread (P0-2 §3): `ImeEngine` **không** thread-safe —
//!   IMK gọi từ main thread; tap dùng instance riêng trên thread tap (P2-2 §2).
//! - Text người dùng **không bao giờ** vào log/`ime_last_error` (S2, S5).

import CTextVNFFI
import Foundation

/// Hằng số FFI dùng trong Swift (khớp `#define` trong textvn_ffi.h).
public enum FFI {
    public static let abiVersion: UInt32 = UInt32(IME_ABI_VERSION)
    public static let maxText: Int = Int(IME_MAX_TEXT)

    public static let ok: Int32 = IME_OK
    public static let errConfig: Int32 = IME_ERR_CONFIG

    // action
    public static let actionPass: UInt32 = UInt32(IME_ACTION_PASS)
    public static let actionReplace: UInt32 = UInt32(IME_ACTION_REPLACE)
    public static let actionCommit: UInt32 = UInt32(IME_ACTION_COMMIT)
    public static let actionRestore: UInt32 = UInt32(IME_ACTION_RESTORE)

    // flags
    public static let flagConsumed: UInt32 = UInt32(IME_FLAG_CONSUMED)
    public static let flagWordEnd: UInt32 = UInt32(IME_FLAG_WORD_END)
    public static let flagError: UInt32 = UInt32(IME_FLAG_ERROR)

    // mods
    public static let modShift: UInt32 = UInt32(IME_MOD_SHIFT)
    public static let modCtrl: UInt32 = UInt32(IME_MOD_CTRL)
    public static let modAlt: UInt32 = UInt32(IME_MOD_ALT)
    public static let modMeta: UInt32 = UInt32(IME_MOD_META) // macOS Cmd
    public static let modCaps: UInt32 = UInt32(IME_MOD_CAPS)
    public static let modFn: UInt32 = UInt32(IME_MOD_FN)

    // caps
    public static let capPreedit: UInt32 = UInt32(IME_CAP_PREEDIT)
    public static let capSelection: UInt32 = UInt32(IME_CAP_SELECTION)
    public static let capFieldDetect: UInt32 = UInt32(IME_CAP_FIELD_DETECT)
    public static let capInjectVK: UInt32 = UInt32(IME_CAP_INJECT_VK)
}

/// Kết quả 1 phím đã decode sang Swift (P0-2 §2 — adapter phải làm đúng bảng action).
public struct KeyOutcome: Equatable {
    public enum Action: Equatable {
        case pass
        case replace(deleteCount: Int, insert: String, preedit: String)
        case commit(insert: String)
        case restore(deleteCount: Int, insert: String)
    }

    public let action: Action
    public let flags: UInt32
    public var isError: Bool { flags & FFI.flagError != 0 }
    public var isWordEnd: Bool { flags & FFI.flagWordEnd != 0 }
}

/// Lỗi non-fatal khi tạo instance (P0-2 §5 — adapter log, không crash).
public enum ImeError: Error, Equatable {
    case invalidArg(rc: Int32)
}

/// Sự kiện phím đã normalize (trước khi vào engine) — mirror `ime_key_v1`.
public struct KeyEvent {
    public var vk: UInt32
    public var ch: UInt32
    public var mods: UInt32
    public var keyDown: Bool
    public var isRepeat: Bool
    public var isInjected: Bool

    public init(
        vk: UInt32, ch: UInt32, mods: UInt32,
        keyDown: Bool, isRepeat: Bool = false, isInjected: Bool = false
    ) {
        self.vk = vk
        self.ch = ch
        self.mods = mods
        self.keyDown = keyDown
        self.isRepeat = isRepeat
        self.isInjected = isInjected
    }
}

/// Một instance engine. **Không thread-safe** — chủ thread sở hữu gọi toàn bộ API.
public final class ImeEngine {
    private var instance: OpaquePointer?

    /// `configJSON == nil` → engine chạy config mặc định (P0-2 §1: config sai
    /// schema vẫn tạo instance + trả `IME_ERR_CONFIG` — non-fatal).
    public init(configJSON: Data?) throws {
        var ptr: OpaquePointer?
        let rc: Int32
        if let cfg = configJSON {
            rc = cfg.withUnsafeBytes { raw in
                ime_instance_new(
                    raw.baseAddress.map { $0.assumingMemoryBound(to: UInt8.self) },
                    raw.count, &ptr
                )
            }
        } else {
            rc = ime_instance_new(nil, 0, &ptr)
        }
        guard rc == IME_OK || rc == IME_ERR_CONFIG else {
            throw ImeError.invalidArg(rc: rc)
        }
        instance = ptr
        if rc == IME_ERR_CONFIG {
            Diagnostics.log("config rejected by engine — running defaults (non-fatal)")
        }
    }

    deinit {
        ime_instance_free(instance)
        instance = nil
    }

    /// `ime_set_context` — P0-2 §1 `ime_context_v1`. `appId`/`elementName` được
    /// copy sang CChar có NUL terminator — pointer chỉ hợp lệ trong lần gọi này.
    public func setContext(
        enabled: Bool, secure: Bool, fieldRole: UInt32, caps: UInt32,
        appId: String, elementName: String?, hint: Int64
    ) {
        var ctx = ime_context_v1()
        ctx.abi_version = FFI.abiVersion
        ctx.enabled = enabled ? 1 : 0
        ctx.secure = secure ? 1 : 0
        ctx.field_role = fieldRole
        ctx.caps = caps
        ctx.hint = hint

        var appIdC = Self.cString(from: appId)
        var elC: [CChar]? = elementName.map(Self.cString(from:))

        appIdC.withUnsafeMutableBufferPointer { appBuf in
            ctx.app_id = appBuf.baseAddress
            elC?.withUnsafeMutableBufferPointer { elBuf in
                ctx.element_name = elBuf.baseAddress
                ime_set_context(instance, &ctx)
            }
            if elC == nil {
                ctx.element_name = nil
                ime_set_context(instance, &ctx)
            }
        }
    }

    /// `ime_key` — normalize đã xong ở KeyTranslator; luôn trả Outcome (fail-open).
    public func key(_ event: KeyEvent) -> KeyOutcome {
        var k = ime_key_v1()
        k.abi_version = FFI.abiVersion
        k.vk = event.vk
        k.ch = event.ch
        k.mods = event.mods
        k.key_down = event.keyDown ? 1 : 0
        k.is_repeat = event.isRepeat ? 1 : 0
        k.is_injected = event.isInjected ? 1 : 0
        k._reserved = 0

        var out = ime_result_v1() // zero-init chuẩn cho struct C import
        let rc = ime_key(instance, &k, &out)
        guard rc == IME_OK else {
            // P0-2 §5: ERR_INTERNAL → gọi reset (engine fail-open) rồi PASS+flag
            // cho adapter (review R1 F24).
            ime_reset(instance)
            return KeyOutcome(action: .pass, flags: FFI.flagError)
        }
        return Self.decode(out)
    }

    /// `ime_reset` — focus change / word boundary (không đụng buffer app).
    public func reset() {
        ime_reset(instance)
    }

    /// `ime_reload_config` — hot-reload; giữ đúng luồng sở hữu (main thread).
    public func reloadConfig(_ json: Data) -> Bool {
        let rc = json.withUnsafeBytes { raw in
            ime_reload_config(
                instance,
                raw.baseAddress.map { $0.assumingMemoryBound(to: UInt8.self) },
                raw.count
            )
        }
        return rc == IME_OK || rc == IME_ERR_CONFIG
    }

    // ---------------------------------------------------------------- decode

    private static func decode(_ out: ime_result_v1) -> KeyOutcome {
        let insert = readUTF32(out.insert, len: Int(out.insert_len))
        let preedit = readUTF32(out.preedit, len: Int(out.preedit_len))
        let delete = Int(out.delete_count)
        let flags = out.flags

        switch out.action {
        case FFI.actionPass:
            return KeyOutcome(action: .pass, flags: flags)
        case FFI.actionReplace:
            return KeyOutcome(
                action: .replace(deleteCount: delete, insert: insert, preedit: preedit),
                flags: flags
            )
        case FFI.actionCommit:
            return KeyOutcome(action: .commit(insert: insert), flags: flags)
        case FFI.actionRestore:
            return KeyOutcome(action: .restore(deleteCount: delete, insert: insert), flags: flags)
        default:
            Diagnostics.log("unknown action \(out.action) — fail-open PASS")
            return KeyOutcome(action: .pass, flags: flags | FFI.flagError)
        }
    }

    /// C fixed-size array import sang Swift là **tuple** — đọc qua raw bytes của
    /// tuple (layout = mảng C, kích thước 64×4 byte, CI enforce sizeof 532).
    /// `len` > `IME_MAX_TEXT` không thể xảy ra (engine tự giữ ≤ 64 — P0-2 §2)
    /// nhưng vẫn chặn phòng hộ.
    static func readUTF32(_ tuple: (UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                                    UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                                    UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                                    UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                                    UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                                    UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                                    UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                                    UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32),
                          len: Int) -> String {
        withUnsafeBytes(of: tuple) { raw in
            var scalars: [Unicode.Scalar] = []
            scalars.reserveCapacity(min(max(0, len), FFI.maxText))
            for i in 0..<min(max(0, len), FFI.maxText) {
                let v = raw.loadUnaligned(fromByteOffset: i * 4, as: UInt32.self)
                if let s = Unicode.Scalar(v) {
                    scalars.append(s)
                }
            }
            return String(String.UnicodeScalarView(scalars))
        }
    }

    /// String → UTF-32 code units (khi adapter phải dựng `ime_result_v1` cho test).
    public static func utf32(_ text: String) -> [UInt32] {
        Array(text.unicodeScalars).map { $0.value }
    }

    /// [UInt32] → String (dùng cho test roundtrip UTF-32 và mock engine).
    public static func string(fromUTF32 points: [UInt32], len: Int) -> String {
        var scalars: [Unicode.Scalar] = []
        scalars.reserveCapacity(min(max(0, len), points.count))
        for i in 0..<min(max(0, len), points.count) {
            if let s = Unicode.Scalar(points[i]) {
                scalars.append(s)
            }
        }
        return String(String.UnicodeScalarView(scalars))
    }

    /// String → mảng CChar NUL-terminated (app_id / element_name của context).
    static func cString(from text: String) -> [CChar] {
        var out = Array(text.utf8CString)
        if out.last != 0 {
            out.append(0)
        }
        return out
    }
}
