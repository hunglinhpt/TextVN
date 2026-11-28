// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-3 pha 3b · parity: `ApplyReplace` mới (kế hoạch từ `textvn_session::imk` qua `ime_imk_plan`)
//! phải gọi `TextTarget` ĐÚNG như bản Swift cũ (`LegacyApplyReplace`) — cùng thứ tự, cùng tham số,
//! cùng lỗi, cùng `onReset` — trên ma trận outcome × chiến lược × trạng thái marked/selection/app.

import Foundation
import XCTest

@testable import CoreBridge
@testable import IMKLib

/// Ghi mọi lời gọi tới target (chuỗi so sánh được).
final class RecordingTarget: TextTarget {
    var log: [String] = []
    var appMarked: String = ""
    var appHasMarkedRange: Bool
    var selection: NSRange
    var deleteOK: Bool

    init(appHasMarkedRange: Bool, selection: NSRange, deleteOK: Bool, appMarked: String) {
        self.appHasMarkedRange = appHasMarkedRange
        self.selection = selection
        self.deleteOK = deleteOK
        self.appMarked = appMarked
    }

    func insert(_ text: String, replacementRange: NSRange) {
        log.append("insert(\(text),\(replacementRange.location),\(replacementRange.length))")
    }
    func setMarked(_ text: String, selectionRange: NSRange) {
        log.append("setMarked(\(text),\(selectionRange.location))")
        appMarked = text
        appHasMarkedRange = !text.isEmpty
    }
    func unmark() { log.append("unmark") }
    func markedRange() -> NSRange {
        appHasMarkedRange ? NSRange(location: 0, length: (appMarked as NSString).length) : .notFound
    }
    func selectionRange() -> NSRange { selection }
    func deleteBackward(count: Int) -> Bool {
        log.append("deleteBackward(\(count))")
        return deleteOK
    }
}

final class ApplyReplaceParityTests: XCTestCase {
    private func run(_ legacy: Bool, _ outcome: KeyOutcome, _ strategy: OutputStrategy,
                     marked: String, appRange: Bool, selection: NSRange, deleteOK: Bool)
        -> (log: [String], marked: String, resets: Int, error: String) {
        let target = RecordingTarget(appHasMarkedRange: appRange, selection: selection,
                                     deleteOK: deleteOK, appMarked: marked)
        let state = MarkedState()
        if !marked.isEmpty { state.set(marked) }
        var resets = 0
        var err = "-"
        do {
            if legacy {
                try LegacyApplyReplace.apply(outcome, strategy: strategy, target: target,
                                             marked: state, onReset: { resets += 1 })
            } else {
                try ApplyReplace.apply(outcome, strategy: strategy, target: target,
                                       marked: state, onReset: { resets += 1 })
            }
        } catch {
            err = "\(error)"
        }
        return (target.log, state.text, resets, err)
    }

    func testApplyReplaceMatchesLegacyOnMatrix() {
        let strategies: [OutputStrategy] = [.preedit, .backspaceType, .selectionReplace,
                                            .forwardAsCommit, .passthrough]
        let inserts = ["", "a", "việt ", "Ko", "😀"]
        let preedits = ["", "tiê", "nghiêngggg", "e\u{0301}"]
        var actions: [KeyOutcome.Action] = [.pass]
        for d in [0, 1, 3, 9] {
            for ins in inserts {
                actions.append(.restore(deleteCount: d, insert: ins))
                for pre in preedits {
                    actions.append(.replace(deleteCount: d, insert: ins, preedit: pre))
                }
            }
        }
        for ins in inserts { actions.append(.commit(insert: ins)) }
        let markeds = ["", "e", "sỏ", "nghiêng"]
        let selections = [NSRange.notFound, NSRange(location: 2, length: 0),
                          NSRange(location: 1, length: 3)]
        var n = 0
        var bad: [String] = []
        for action in actions {
            let outcome = KeyOutcome(action: action, flags: 0)
            for strategy in strategies {
                for m in markeds {
                    for appRange in [true, false] {
                        for sel in selections {
                            for deleteOK in [true, false] {
                                let a = run(true, outcome, strategy, marked: m, appRange: appRange,
                                            selection: sel, deleteOK: deleteOK)
                                let b = run(false, outcome, strategy, marked: m, appRange: appRange,
                                            selection: sel, deleteOK: deleteOK)
                                n += 1
                                if a.log != b.log || a.marked != b.marked || a.resets != b.resets
                                    || (a.error == "-") != (b.error == "-") {
                                    if bad.count < 5 {
                                        bad.append("\(action) \(strategy) m=\(m) range=\(appRange) "
                                            + "sel=\(sel) del=\(deleteOK): cũ \(a) | mới \(b)")
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        XCTAssertGreaterThan(n, 10_000)
        XCTAssertEqual(bad, [], "SF-3b lệch \(bad.count) ca")
    }
}
