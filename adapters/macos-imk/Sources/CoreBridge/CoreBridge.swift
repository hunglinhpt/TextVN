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

    // assist kinds (IME_ASSIST_KIND_*)
    public static let assistKindDuplicateWord: UInt32 = UInt32(IME_ASSIST_KIND_DUPLICATE_WORD)
    public static let assistKindSpaceBeforePunct: UInt32 = UInt32(IME_ASSIST_KIND_SPACE_BEFORE_PUNCT)
    public static let assistKindSpaceAfterPunct: UInt32 = UInt32(IME_ASSIST_KIND_SPACE_AFTER_PUNCT)
    public static let assistKindDoubleSpace: UInt32 = UInt32(IME_ASSIST_KIND_DOUBLE_SPACE)
    public static let assistKindLowercaseSentence: UInt32 = UInt32(IME_ASSIST_KIND_LOWERCASE_SENTENCE)
    public static let assistKindDiacriticTypo: UInt32 = UInt32(IME_ASSIST_KIND_DIACRITIC_TYPO)
    public static let assistKindSentenceRepeat: UInt32 = UInt32(IME_ASSIST_KIND_SENTENCE_REPEAT)
    public static let assistKindDoubleNegative: UInt32 = UInt32(IME_ASSIST_KIND_DOUBLE_NEGATIVE)
    public static let assistKindCommaAtEnd: UInt32 = UInt32(IME_ASSIST_KIND_COMMA_AT_END)
    public static let assistKindSpelling: UInt32 = UInt32(IME_ASSIST_KIND_SPELLING)

    // assist triggers (IME_ASSIST_TRIGGER_*)
    public static let assistTriggerNone: UInt32 = UInt32(IME_ASSIST_TRIGGER_NONE)
    public static let assistTriggerWord: UInt32 = UInt32(IME_ASSIST_TRIGGER_WORD)
    public static let assistTriggerSentence: UInt32 = UInt32(IME_ASSIST_TRIGGER_SENTENCE)

    // sentence fix limit
    public static let maxSentenceFixChars: Int = Int(IME_MAX_SENTENCE_FIX_CHARS)

    public static func assistTriggerKind(ch: UInt32) -> UInt32 {
        ime_assist_trigger_kind(ch)
    }
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

    /// Public: adapter (IMKLib) dựng lại outcome đã chỉnh (bỏ ranh giới Enter/Tab
    /// — B2); memberwise init tự sinh chỉ là `internal` của module CoreBridge.
    public init(action: Action, flags: UInt32) {
        self.action = action
        self.flags = flags
    }
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

/// struct AssistIssue đại diện cho một vấn đề ngữ pháp/chính tả (P4-0).
public struct AssistIssue: Equatable {
    public let kind: UInt32
    public let start: UInt32
    public let end: UInt32
    public let autoFixable: Bool
    public let confidence: UInt32
    public let message: String
    public let suggest: String

    public init(
        kind: UInt32,
        start: UInt32,
        end: UInt32,
        autoFixable: Bool,
        confidence: UInt32,
        message: String,
        suggest: String
    ) {
        self.kind = kind
        self.start = start
        self.end = end
        self.autoFixable = autoFixable
        self.confidence = confidence
        self.message = message
        self.suggest = suggest
    }
}

/// Kế hoạch tự sửa cả câu (AS-050 / Mode 2 / A3) — tương đồng SentenceFixPlan của Windows TSF.
public struct SentenceFixPlan: Equatable {
    public let fixedCount: Int
    public let deleteFrom: Int
    public let deleteChars: Int
    public let deleteGlyphs: Int
    public let replacement: String
    public let originalTail: String

    public init(
        fixedCount: Int,
        deleteFrom: Int,
        deleteChars: Int,
        deleteGlyphs: Int,
        replacement: String,
        originalTail: String
    ) {
        self.fixedCount = fixedCount
        self.deleteFrom = deleteFrom
        self.deleteChars = deleteChars
        self.deleteGlyphs = deleteGlyphs
        self.replacement = replacement
        self.originalTail = originalTail
    }
}

/// Một instance engine. **Không thread-safe** — chủ thread sở hữu gọi toàn bộ API.
public final class ImeEngine {
    private var instance: OpaquePointer?
    private var spellProviderRef: AnyObject?

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

        let appIdC = Self.cString(from: appId)
        let elC: [CChar]? = elementName.map(Self.cString(from:))

        // `const char*` trong C = `UnsafePointer<CChar>` → buffer pointer KHÔNG
        // mutable (MAC-032: Xcode 26.6 từ chối gán UnsafeMutablePointer).
        appIdC.withUnsafeBufferPointer { appBuf in
            ctx.app_id = appBuf.baseAddress
            if let elC = elC {
                elC.withUnsafeBufferPointer { elBuf in
                    ctx.element_name = elBuf.baseAddress
                    ime_set_context(instance, &ctx)
                }
            } else {
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

    /// Gắn hoặc huỷ spell provider của hệ điều hành (AS-042 · P4-10 §5.2).
    ///
    /// - Parameters:
    ///   - provider: struct `ime_os_spell_v1` do adapter cung cấp, hoặc `nil` để huỷ (quay về NoDictionary).
    ///   - retainRef: đối tượng Swift giữ provider sống (ví dụ `MacSpellProvider`), hoặc `nil`.
    /// - Returns: `FFI.ok` nếu thành công, hoặc mã lỗi `IME_ERR_*`.
    @discardableResult
    public func setSpellProvider(_ provider: ime_os_spell_v1?, retainRef: AnyObject? = nil) -> Int32 {
        spellProviderRef = retainRef
        if var p = provider {
            return ime_assist_set_spell_provider(instance, &p)
        } else {
            return ime_assist_set_spell_provider(instance, nil)
        }
    }

    /// Gắn hoặc huỷ spell provider qua con trỏ thô `const ime_os_spell_v1*`.
    @discardableResult
    public func setSpellProvider(_ provider: UnsafePointer<ime_os_spell_v1>?) -> Int32 {
        ime_assist_set_spell_provider(instance, provider)
    }

    /// Lấy phần đuôi buffer câu của engine dưới dạng chuỗi UTF-8 (Fassist-76).
    ///
    /// - Parameter maxChars: Số ký tự Unicode tối đa (mặc định 256 — D31).
    /// - Returns: Chuỗi đuôi câu, hoặc chuỗi rỗng nếu lỗi/không có.
    public func sentenceTail(maxChars: Int = 256) -> String {
        var neededLen: Int = 0
        let rcProbe = ime_assist_sentence_tail(instance, maxChars, nil, 0, &neededLen)
        guard rcProbe == FFI.ok, neededLen > 0 else { return "" }
        var buf = [CChar](repeating: 0, count: neededLen + 1)
        var actualLen: Int = 0
        let rc = ime_assist_sentence_tail(instance, maxChars, &buf, buf.count, &actualLen)
        guard rc == FFI.ok else { return "" }
        return String(cString: buf)
    }

    /// Tính số ký tự glyph theo bảng mã xuất hiện tại của engine (D27).
    ///
    /// Dùng để adapter biết cần xoá bao nhiêu ký tự trong document khi áp dụng gợi ý.
    ///
    /// R-11 (fail-safe): lỗi FFI ⇒ trả **0** (không phải `text.count`): số char ≠
    /// số glyph ở VNI/TCVN3 ⇒ đo sai ⇒ xoá lệch vị trí (mất dữ liệu). 0 ⇒ kế hoạch
    /// sửa bị từ chối ⇒ **không sửa**.
    public func glyphLen(_ text: String) -> Int {
        var count: Int = 0
        let utf8 = Array(text.utf8)
        let rc = utf8.withUnsafeBufferPointer { buf in
            ime_assist_glyph_len(
                instance,
                buf.baseAddress.map { UnsafeRawPointer($0).assumingMemoryBound(to: CChar.self) },
                buf.count,
                &count
            )
        }
        guard rc == FFI.ok else { return 0 }
        return count
    }

    /// Phân loại phím trigger: NONE(0), WORD(1), SENTENCE(2).
    public static func assistTriggerKind(ch: UInt32) -> UInt32 {
        ime_assist_trigger_kind(ch)
    }

    /// Kiểm tra assist trên buffer câu hiện tại của engine (P4-2, AS-042).
    public func assistCheck() -> [AssistIssue] {
        var out = ime_assist_result_v1()
        out.abi_version = FFI.abiVersion
        let rc = ime_assist_check_instance(instance, &out)
        guard rc == FFI.ok else { return [] }
        return Self.decodeAssistResult(out)
    }

    /// Kiểm tra ngữ pháp chuỗi UTF-8 bất kỳ (D23).
    public static func assistCheck(text: String) -> [AssistIssue] {
        var out = ime_assist_result_v1()
        out.abi_version = FFI.abiVersion
        let utf8 = Array(text.utf8)
        let rc = utf8.withUnsafeBufferPointer { buf in
            ime_assist_check(
                buf.baseAddress.map { UnsafeRawPointer($0).assumingMemoryBound(to: CChar.self) },
                buf.count,
                &out
            )
        }
        guard rc == FFI.ok else { return [] }
        return decodeAssistResult(out)
    }

    private static func decodeAssistResult(_ out: ime_assist_result_v1) -> [AssistIssue] {
        var mutableOut = out
        var result: [AssistIssue] = []
        let count = min(Int(mutableOut.count), Int(IME_MAX_ASSIST_ISSUES))
        withUnsafePointer(to: &mutableOut.issues) { ptr in
            let issuesPtr = UnsafeRawPointer(ptr).assumingMemoryBound(to: ime_assist_issue_v1.self)
            for i in 0..<count {
                let issue = issuesPtr[i]
                let msg = withUnsafeBytes(of: issue.message) { raw in
                    String(cString: raw.baseAddress!.assumingMemoryBound(to: CChar.self))
                }
                let sugg = withUnsafeBytes(of: issue.suggest) { raw in
                    String(cString: raw.baseAddress!.assumingMemoryBound(to: CChar.self))
                }
                result.append(AssistIssue(
                    kind: issue.kind,
                    start: issue.start,
                    end: issue.end,
                    autoFixable: issue.auto_fixable != 0,
                    confidence: issue.confidence,
                    message: msg,
                    suggest: sugg
                ))
            }
        }
        return result
    }

    /// Mã hoá mảng `[AssistIssue]` thành C-struct `ime_assist_result_v1` trên stack (P0-2 §0).
    public static func encodeAssistResult(_ issues: [AssistIssue]) -> ime_assist_result_v1 {
        var res = ime_assist_result_v1()
        res.abi_version = FFI.abiVersion
        let count = min(issues.count, Int(IME_MAX_ASSIST_ISSUES))
        res.count = UInt32(count)
        withUnsafeMutablePointer(to: &res.issues) { ptr in
            let issuesPtr = UnsafeMutableRawPointer(ptr).assumingMemoryBound(to: ime_assist_issue_v1.self)
            for i in 0..<count {
                let issue = issues[i]
                var rawIssue = ime_assist_issue_v1()
                rawIssue.kind = issue.kind
                rawIssue.start = issue.start
                rawIssue.end = issue.end
                rawIssue.auto_fixable = issue.autoFixable ? 1 : 0
                rawIssue.confidence = issue.confidence

                let msgBytes = Array(issue.message.utf8)
                let msgLen = min(msgBytes.count, Int(IME_MAX_ASSIST_MSG) - 1)
                if msgLen > 0 {
                    withUnsafeMutableBytes(of: &rawIssue.message) { raw in
                        msgBytes.withUnsafeBytes { src in
                            if let srcBase = src.baseAddress, let dstBase = raw.baseAddress {
                                dstBase.copyMemory(from: srcBase, byteCount: msgLen)
                            }
                        }
                    }
                }

                let suggBytes = Array(issue.suggest.utf8)
                let suggLen = min(suggBytes.count, Int(IME_MAX_ASSIST_SUGGEST) - 1)
                rawIssue.suggest_len = UInt32(suggLen)
                if suggLen > 0 {
                    withUnsafeMutableBytes(of: &rawIssue.suggest) { raw in
                        suggBytes.withUnsafeBytes { src in
                            if let srcBase = src.baseAddress, let dstBase = raw.baseAddress {
                                dstBase.copyMemory(from: srcBase, byteCount: suggLen)
                            }
                        }
                    }
                }

                issuesPtr[i] = rawIssue
            }
        }
        return res
    }

    /// Tính kế hoạch tự sửa câu (Mode 2 / AS-050) qua C-ABI `ime_assist_plan_sentence_fix`.
    ///
    /// Gọi FFI của engine để tính toán kế hoạch sửa câu an toàn (objective fixes).
    /// Fail-open: trả `nil` nếu lỗi dữ liệu hoặc không có gì cần sửa. Không log câu (S2).
    public func planSentenceFix(
        sentence: String,
        result: ime_assist_result_v1
    ) -> SentenceFixPlan? {
        Self.planSentenceFix(instance: instance, sentence: sentence, result: result)
    }

    /// Tính kế hoạch tự sửa câu từ danh sách `[AssistIssue]` trên instance hiện tại.
    public func planSentenceFix(
        sentence: String,
        issues: [AssistIssue]
    ) -> SentenceFixPlan? {
        planSentenceFix(sentence: sentence, result: Self.encodeAssistResult(issues))
    }

    /// Static wrapper gọi `ime_assist_plan_sentence_fix` với instance tuỳ chọn (nil = không instance).
    public static func planSentenceFix(
        sentence: String,
        result: ime_assist_result_v1
    ) -> SentenceFixPlan? {
        planSentenceFix(instance: nil, sentence: sentence, result: result)
    }

    /// Static wrapper gọi `ime_assist_plan_sentence_fix` với instance cụ thể hoặc nil.
    public static func planSentenceFix(
        instance: OpaquePointer?,
        sentence: String,
        result: ime_assist_result_v1
    ) -> SentenceFixPlan? {
        var outPlan = ime_assist_plan_v1()
        outPlan.abi_version = FFI.abiVersion
        var mutableResult = result
        mutableResult.abi_version = FFI.abiVersion
        let cSentence = cString(from: sentence)
        let rc = cSentence.withUnsafeBufferPointer { buf in
            ime_assist_plan_sentence_fix(
                instance,
                buf.baseAddress,
                &mutableResult,
                &outPlan
            )
        }
        guard rc != 0 else { return nil }
        let repl = withUnsafeBytes(of: outPlan.replacement) { raw in
            String(cString: raw.baseAddress!.assumingMemoryBound(to: CChar.self))
        }
        let origTail = withUnsafeBytes(of: outPlan.original_tail) { raw in
            String(cString: raw.baseAddress!.assumingMemoryBound(to: CChar.self))
        }
        return SentenceFixPlan(
            fixedCount: Int(outPlan.fixed_count),
            deleteFrom: Int(outPlan.delete_from),
            deleteChars: Int(outPlan.delete_chars),
            deleteGlyphs: Int(outPlan.delete_glyphs),
            replacement: repl,
            originalTail: origTail
        )
    }

    /// Static wrapper gọi `ime_assist_plan_sentence_fix` từ `[AssistIssue]`.
    public static func planSentenceFix(
        instance: OpaquePointer? = nil,
        sentence: String,
        issues: [AssistIssue]
    ) -> SentenceFixPlan? {
        planSentenceFix(instance: instance, sentence: sentence, result: encodeAssistResult(issues))
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
