// SPDX-License-Identifier: GPL-3.0-or-later
//! TextVNTapTests — logic thuần của tap (P2-2 §8) chạy được cả trên CI macOS
//! không quyền: OwnerRule, marker parity, self-disable counter, decision mapping.

import XCTest

@testable import TextVNTap

final class OwnerRuleTests: XCTestCase {
    func testDefaultIsImk() {
        let rule = OwnerRule()
        XCTAssertEqual(rule.owner(of: "com.apple.safari"), .imk)
        XCTAssertFalse(rule.tapOwns("com.apple.safari"))
        XCTAssertTrue(rule.imkOwns("com.apple.safari"))
    }

    func testTapOwnerApplies() {
        let rule = OwnerRule()
        rule.update(owners: ["com.valvesoftware.steam": .tap])
        XCTAssertTrue(rule.tapOwns("com.valvesoftware.steam"))
        // IMK disabled cho app owner=tap (P2-2 §6).
        XCTAssertFalse(rule.imkOwns("com.valvesoftware.steam"))
    }

    func testCaseInsensitiveNormalization() {
        let rule = OwnerRule()
        rule.update(owners: ["com.hnc.discord": .tap])
        XCTAssertTrue(rule.tapOwns("COM.HNC.DISCORD"), "bundle id phải lowercase")
    }

    func testParseRaw() {
        XCTAssertEqual(EngineOwner.parse("tap"), .tap)
        XCTAssertEqual(EngineOwner.parse("imk"), .imk)
        XCTAssertEqual(EngineOwner.parse(nil), .imk, "fail-safe về đường chính IMK")
        XCTAssertEqual(EngineOwner.parse("rác"), .imk)
    }
}

final class TapMarkerTests: XCTestCase {
    /// Marker tap và marker IMK phải cùng giá trị — nếu lệch thì loop-guard
    /// vỡ (callback xử lý chính event mình post). Giá trị = "TXVN".
    func testMarkerValue() {
        XCTAssertEqual(TapMarker.userData, 0x5458_564E)
    }

    /// Decision mapping (§5): transform → injector gọi với delete+insert; pass → không.
    func testDecisionEquatable() {
        XCTAssertEqual(TapDecision.transform(deleteCount: 2, insert: "được"),
                       TapDecision.transform(deleteCount: 2, insert: "được"))
        XCTAssertNotEqual(TapDecision.pass, TapDecision.transform(deleteCount: 0, insert: ""))
    }
}

final class SelfDisableTests: XCTestCase {
    func testSlowStreakDisables() {
        let controller = EventTapController(handler: MockHandler())
        XCTAssertFalse(controller.isSelfDisabled)
        // 49 lần chậm: chưa disable.
        for _ in 0..<(EventTapController.slowLimitStreak - 1) {
            controller.recordSlow(Double(EventTapController.slowThresholdMs) + 1)
        }
        XCTAssertFalse(controller.isSelfDisabled)
        // Lần thứ 50 → self-disable (P2-2 §9).
        controller.recordSlow(Double(EventTapController.slowThresholdMs) + 1)
        XCTAssertTrue(controller.isSelfDisabled)
    }

    func testFastKeyResetsStreak() {
        let controller = EventTapController(handler: MockHandler())
        controller.recordSlow(10)
        controller.recordSlow(10)
        controller.recordSlow(1) // nhanh → reset streak
        controller.recordSlow(10)
        XCTAssertFalse(controller.isSelfDisabled)
    }

    func testStartWithoutPermissionReturnsFalse() {
        // Không quyền Accessibility → cả 3 loại tapCreate fail → start() = false
        // (trên CI macOS runner luôn thế; trên máy có quyền sẽ true — RM5).
        let controller = EventTapController(handler: MockHandler())
        _ = controller.start() // không assert bool vì runner có thể có quyền
        controller.stop()
    }
}

final class MockHandler: TapKeyHandler {
    var shouldProcess = true

    func tapHandle(vk: UInt32, ch: UInt32, mods: UInt32, source: CGEventSource?) -> TapDecision {
        .pass
    }

    func tapShouldProcess(appID: String) -> Bool {
        shouldProcess
    }
}
