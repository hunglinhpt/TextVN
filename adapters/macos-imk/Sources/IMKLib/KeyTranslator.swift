// SPDX-License-Identifier: GPL-3.0-or-later
//! KeyTranslator — NSEvent (macOS) → `KeyEvent` đã normalize (P2-1 §5 bước 3).
//!
//! Nhiệm vụ (MAC-013):
//! 1. `UCKeyTranslate` theo **layout hiện tại** của user (không hardcode ABC —
//!    gõ đúng cả khi user đổi sang layout khác).
//! 2. keycode mac → VK canonical Windows qua `KeyMapMacGenerated` (P0-2 §1:
//!    "Win VK là canonical — adapter macOS chuẩn hóa về đây").
//! 3. ModifierFlags → `IME_MOD_*` (Cmd = IME_MOD_META).
//!
//! Không state nào ngoài dead-key state nhỏ (UCKeyTranslate cần).

import AppKit
import Carbon.HIToolbox
import CoreBridge

public enum KeyTranslatorError: Error {
    case noLayout
    case noChar
}

public final class KeyTranslator {
    /// Dead-key state của UCKeyTranslate (giữ giữa các phím — chuẩn Carbon).
    private var deadKeyState: UInt32 = 0

    public init() {}

    // ---------------------------------------------------------------- API

    /// Translate keyDown/keyUp của `NSEvent` → `KeyEvent`, hoặc `nil` nếu phím
    /// không sinh ký tự (F-key, modifier… — vẫn normalize VK để engine PASS đúng).
    public func translate(_ event: NSEvent) -> KeyEvent? {
        guard event.type == .keyDown || event.type == .keyUp else { return nil }

        let mods = Self.modifiers(event.modifierFlags)
        let vk = KeyMapMacGenerated.canonicalVK(UInt32(event.keyCode))

        guard let ch = character(for: event.keyCode, mods: mods) else {
            // Phím không sinh ký tự: chỉ normalize VK (engine coi ch=0 + vk lạ → PASS).
            guard let vk else { return nil }
            return KeyEvent(
                vk: vk, ch: 0, mods: mods,
                keyDown: event.type == .keyDown, isRepeat: event.isARepeat
            )
        }

        let canonical = vk ?? 0
        return KeyEvent(
            vk: canonical, ch: ch, mods: mods,
            keyDown: event.type == .keyDown, isRepeat: event.isARepeat
        )
    }

    /// `flagsChanged` — chỉ cập nhật modifier state (P2-1 §5 bước 2).
    public func noteFlagsChanged() {
        // Dead-key state giữ nguyên (không reset — giữ đúng ngữ nghĩa Carbon).
    }

    // ---------------------------------------------------------------- internals

    /// UCKeyTranslate theo layout hiện tại + dead-key state.
    /// Trả Unicode scalar đầu tiên của chuỗi translate (IME chỉ cần 1 ký tự).
    func character(for keyCode: UInt16, mods: UInt32) -> UInt32? {
        guard let layout = Self.currentLayoutData() else {
            Diagnostics.log("translate: no layout data")
            return nil
        }

        var chars = [UniChar](repeating: 0, count: 4)
        var len = 0
        let shift = (mods & FFI.modShift != 0) ? (UInt32(shiftKey) >> 8) : 0
        let option = (mods & FFI.modAlt != 0) ? (UInt32(optionKey) >> 8) : 0
        let control = (mods & FFI.modCtrl != 0) ? (UInt32(controlKey) >> 8) : 0

        let status = withUnsafeMutablePointer(to: &deadKeyState) { dead in
            UCKeyTranslate(
                layout, keyCode, UInt16(kUCKeyActionDown),
                shift | option | control, UInt32(LMGetKbdType()),
                UInt32(kUCKeyTranslateNoDeadKeysMask), dead,
                4, &len, &chars
            )
        }
        guard status == noErr, len > 0 else { return nil }

        // UniChar[] = UTF-16 code units → Swift String → scalar đầu tiên.
        let string = String(utf16CodeUnits: chars, count: len)
        guard let scalar = string.unicodeScalars.first else { return nil }
        return scalar.value
    }

    /// Layout hiện tại của input source đang chọn (theo dõi thay đổi layout).
    private static var cachedLayout: Data?
    private static var cachedSourceID: String?

    static func currentLayoutData() -> Data? {
        let source = TISCopyCurrentKeyboardLayoutInputSource().takeRetainedValue()
        let sourceID = Self.sourceID(of: source)
        if sourceID == cachedSourceID, let data = cachedLayout {
            return data
        }
        guard let ptr = TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData) else {
            return nil
        }
        let data = Unmanaged<CFData>.fromOpaque(ptr).takeUnretainedValue() as Data
        cachedLayout = data
        cachedSourceID = sourceID
        return data
    }

    private static func sourceID(of source: TISInputSource) -> String? {
        guard let ptr = TISGetInputSourceProperty(source, kTISPropertyLocalizedName) else {
            return nil
        }
        let name = Unmanaged<CFString>.fromOpaque(ptr).takeUnretainedValue() as String
        return name
    }

    /// NSEvent.ModifierFlags → IME_MOD_* (P0-2 §1). Cmd = IME_MOD_META.
    static func modifiers(_ flags: NSEvent.ModifierFlags) -> UInt32 {
        var mods: UInt32 = 0
        if flags.contains(.shift) { mods |= FFI.modShift }
        if flags.contains(.control) { mods |= FFI.modCtrl }
        if flags.contains(.option) { mods |= FFI.modAlt }
        if flags.contains(.command) { mods |= FFI.modMeta }
        if flags.contains(.capsLock) { mods |= FFI.modCaps }
        if flags.contains(.function) { mods |= FFI.modFn }
        return mods
    }
}
