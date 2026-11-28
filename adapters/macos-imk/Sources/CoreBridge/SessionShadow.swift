// SPDX-License-Identifier: GPL-3.0-or-later
//! SessionShadow — SF-2 (P0-6 §5): chạy máy toggle của `textvn-session` (qua C ABI
//! `ime_session_*`, SF-1b) song song logic Ctrl+Shift cũ của IMK.
//!
//! Chỉ bật khi `TEXTVN_SESSION_SHADOW=1` (CI/dev); mặc định `fromEnvironment()` trả nil.
//! Mã cũ luôn là thứ được áp; shadow chỉ ĐẾM lệch "có bật/tắt Việt–Anh hay không"
//! (`IME_SCMD_MODE_INDICATOR`). Lỗi ABI ⇒ shadow tự tắt.
//! G8: log chỉ ghi loại phím và số đếm, không bao giờ ký tự (shadow không nhận `ch`).

import CTextVNFFI
import Foundation

public final class SessionShadow {
    // VK canonical lấy từ bảng sinh mã (G22) — keycode Carbon kVK_Shift/Control/Option/Space.
    public static let vkShift: UInt32 = KeyMapMacGenerated.canonicalVK(0x38) ?? 0x10
    public static let vkControl: UInt32 = KeyMapMacGenerated.canonicalVK(0x3b) ?? 0x11
    public static let vkMenu: UInt32 = KeyMapMacGenerated.canonicalVK(0x3a) ?? 0x12
    public static let vkSpace: UInt32 = KeyMapMacGenerated.canonicalVK(0x31) ?? 0x20

    private var session: OpaquePointer?
    private var ctrl = false
    private var shift = false
    private var altOrCmd = false
    public private(set) var mismatches: UInt32 = 0
    /// Số lần mã cũ bật/tắt khi shadow bật (mẫu số).
    public private(set) var toggles: UInt32 = 0
    /// Số lần session bật/tắt.
    public private(set) var sessionToggles: UInt32 = 0

    /// Bật nếu env `TEXTVN_SESSION_SHADOW=1`, ngược lại nil.
    public static func fromEnvironment() -> SessionShadow? {
        guard ProcessInfo.processInfo.environment["TEXTVN_SESSION_SHADOW"] == "1" else { return nil }
        return SessionShadow()
    }

    /// Bật bất kể env (test). nil nếu không tạo được session.
    public init?() {
        var cfg = ime_session_config_v1()
        cfg.abi_version = UInt32(IME_SESSION_ABI_VERSION)
        cfg.caps = UInt32(IME_SCAP_PREEDIT) | UInt32(IME_SCAP_KEY_UP_EVENTS)
        cfg.config_json = nil
        cfg.config_len = 0
        var ptr: OpaquePointer?
        guard ime_session_new(&cfg, &ptr) == IME_OK, let ptr else { return nil }
        session = ptr
    }

    deinit {
        if let session { ime_session_free(session) }
    }

    public var isActive: Bool { session != nil }

    /// Mất focus / commit / đóng client (cùng chỗ mã cũ xoá trạng thái modifier).
    public func reset(nowMs: UInt64) {
        ctrl = false
        shift = false
        altOrCmd = false
        _ = feed(kind: UInt32(IME_SEV_RESET), vk: 0, mods: 0, arg: UInt32(IME_SRESET_FOCUS), nowMs: nowMs)
    }

    /// `flagsChanged`: dựng lại key-down/key-up từ chênh lệch cờ (IMK không báo phím nào
    /// đổi), rồi so với `legacyToggled` của mã cũ ở cùng sự kiện.
    public func flagsChanged(ctrl newCtrl: Bool, shift newShift: Bool, altOrCmd newAlt: Bool,
                             legacyToggled: Bool, nowMs: UInt64) {
        guard session != nil else { return }
        var fired = false
        var failed = false
        func step(_ r: Bool?) {
            if let r { fired = fired || r } else { failed = true }
        }
        if newAlt != altOrCmd {
            altOrCmd = newAlt
            step(newAlt ? down(Self.vkMenu, nowMs) : up(Self.vkMenu, nowMs))
        }
        if newCtrl != ctrl {
            ctrl = newCtrl
            step(newCtrl ? down(Self.vkControl, nowMs) : up(Self.vkControl, nowMs))
        }
        if newShift != shift {
            shift = newShift
            step(newShift ? down(Self.vkShift, nowMs) : up(Self.vkShift, nowMs))
        }
        if !failed { compare(phase: "flags", legacy: legacyToggled, session: fired) }
    }

    /// `keyDown` phím thường (`isSpace` = Space); `legacyToggled` = mã cũ bật/tắt ở phím này
    /// (Ctrl+Shift+Space, ADR-011).
    public func keyDown(isSpace: Bool, legacyToggled: Bool, nowMs: UInt64) {
        guard session != nil else { return }
        if let r = down(isSpace ? Self.vkSpace : 0, nowMs) {
            compare(phase: isSpace ? "keydown-space" : "keydown", legacy: legacyToggled, session: r)
        }
    }

    // MARK: - private

    private var mods: UInt32 {
        var m: UInt32 = 0
        if ctrl { m |= UInt32(IME_MOD_CTRL) }
        if shift { m |= UInt32(IME_MOD_SHIFT) }
        if altOrCmd { m |= UInt32(IME_MOD_ALT) }
        return m
    }

    private func down(_ vk: UInt32, _ nowMs: UInt64) -> Bool? {
        feed(kind: UInt32(IME_SEV_KEYDOWN), vk: vk, mods: mods, arg: 0, nowMs: nowMs)
    }

    private func up(_ vk: UInt32, _ nowMs: UInt64) -> Bool? {
        feed(kind: UInt32(IME_SEV_KEYUP), vk: vk, mods: mods, arg: 0, nowMs: nowMs)
    }

    /// true = session phát MODE_INDICATOR; nil = lỗi ABI (shadow tắt).
    private func feed(kind: UInt32, vk: UInt32, mods: UInt32, arg: UInt32, nowMs: UInt64) -> Bool? {
        guard let session else { return nil }
        var ev = ime_session_event_v1()
        ev.abi_version = UInt32(IME_SESSION_ABI_VERSION)
        ev.kind = kind
        ev.vk = vk
        ev.ch = 0
        ev.mods = mods
        ev.now_ms = nowMs
        ev.text = nil
        ev.text_len = 0
        ev.arg = arg
        var out = ime_session_out_v1()
        out.abi_version = UInt32(IME_SESSION_ABI_VERSION)
        guard ime_session_event(session, &ev, &out) == IME_OK else {
            ime_session_free(session)
            self.session = nil
            return nil
        }
        let count = min(Int(out.count), Int(IME_SESSION_MAX_CMDS))
        let modeIndicator = UInt32(IME_SCMD_MODE_INDICATOR)
        return withUnsafeBytes(of: &out.cmds) { raw in
            let cmds = raw.bindMemory(to: ime_session_cmd_v1.self)
            return cmds.prefix(count).contains { $0.kind == modeIndicator }
        }
    }

    private func compare(phase: String, legacy: Bool, session: Bool) {
        if legacy { toggles += 1 }
        if session { sessionToggles += 1 }
        guard legacy != session else { return }
        mismatches += 1
        Diagnostics.log("SF2 shadow mismatch \(phase) legacy=\(legacy) session=\(session) (total \(mismatches))")
    }
}
