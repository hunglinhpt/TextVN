// SPDX-License-Identifier: GPL-3.0-or-later
//! MacModifierTap — trạng thái phím chuyển Ctrl+Shift nhấn-thả và standalone Ctrl tap
//! của IMK (tách nguyên văn khỏi `TextVNInputController.handle` để test được và để
//! shadow SF-2 so với `textvn-session`; hành vi KHÔNG đổi).
//!
//! IMK chỉ phát `flagsChanged` (bộ cờ modifier sau thay đổi), không báo key-up riêng.

import Foundation

struct MacModifierTap {
    enum Outcome: Equatable {
        case none
        /// Đảo Việt/Anh (đã qua debounce 250 ms).
        case toggle
        /// Nhấn-thả riêng Ctrl ≤ 400 ms (R2-143).
        case ctrlTap
    }

    /// Cửa sổ chống bật/tắt kép (khớp Windows/session `TOGGLE_DEBOUNCE_MS`).
    static let debounceSeconds: TimeInterval = 0.25

    private(set) var ctrlDown = false
    private(set) var shiftDown = false
    private(set) var otherKeyPressed = false
    private var ctrlDownTime: TimeInterval = 0
    private var lastToggleTime: TimeInterval = 0

    /// Mất focus / commit / đóng client: huỷ lần nhấn đang chờ (giữ mốc debounce).
    mutating func reset() {
        ctrlDown = false
        shiftDown = false
        otherKeyPressed = false
    }

    /// Phím thường (keyDown) hoặc chord: lần nhấn Ctrl/Shift đang chờ không còn "trần".
    mutating func otherKey() {
        otherKeyPressed = true
    }

    mutating func flagsChanged(isCtrl: Bool, isShift: Bool, hasAltOrCmd: Bool,
                               now: TimeInterval) -> Outcome {
        // Lần bấm mới bắt đầu từ trạng thái không giữ modifier: phím chữ vừa gõ trước đó
        // không thuộc lần này — thiếu dòng này Ctrl tap không bao giờ chạy sau khi gõ.
        if (isCtrl || isShift) && !ctrlDown && !shiftDown {
            otherKeyPressed = false
        }
        if hasAltOrCmd {
            otherKeyPressed = true
        }

        var outcome = Outcome.none
        if TextVNInputController.isCtrlShiftTap(
            wasCtrl: ctrlDown,
            wasShift: shiftDown,
            currentCtrl: isCtrl,
            currentShift: isShift,
            otherKeyPressed: otherKeyPressed
        ) {
            if now - lastToggleTime >= Self.debounceSeconds {
                lastToggleTime = now
                outcome = .toggle
            }
            otherKeyPressed = true
        } else if TextVNInputController.isCtrlTap(
            wasCtrl: ctrlDown,
            wasShift: shiftDown,
            currentCtrl: isCtrl,
            currentShift: isShift,
            hasAltOrCmd: hasAltOrCmd,
            otherKeyPressed: otherKeyPressed,
            heldSeconds: now - ctrlDownTime
        ) {
            outcome = .ctrlTap
            otherKeyPressed = true
        }

        if isCtrl && !ctrlDown {
            ctrlDownTime = now
        }
        ctrlDown = isCtrl
        shiftDown = isShift

        if !isCtrl && !isShift {
            otherKeyPressed = false
        }
        return outcome
    }
}
