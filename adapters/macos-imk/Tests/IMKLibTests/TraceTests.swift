// SPDX-License-Identifier: GPL-3.0-or-later
// SF-1e — ghi VẾT VÀNG của adapter macOS (P0-6 §6.1): chạy engine THẬT (CoreBridge) +
// `ApplyReplace`/`MarkedState` của IMKLib trên luồng phím do Rust xuất
// (`textvn-cli replay … --emit-keys <dir>`, SF-1c) — KHÔNG viết lại bộ đọc `.keys` bằng Swift (G22).
// Mỗi thao tác lên `TextTarget` được chuẩn hoá thành đúng 5 lệnh của SF-1c/SF-1d:
// `delete_before` · `commit` · `set_preedit` · `forward_key` · `eat_key`, một dòng JSONL
// `{"case","step","cmd","text","n"}` ⇒ `corpus/traces/mac/<rel>.jsonl` để SF-1f so 3 adapter.
//
// Phạm vi (P0-6 §6.1 "chạy ApplyReplace/Marked trên corpus"): lớp áp outcome của controller.
// Phần cổng phím của `handle(NSEvent)` (phím bơm, modifier, ô secure, trợ lý) cần NSEvent thật —
// chuyển vào session ở SF-2; phần rẽ nhánh `.pass`/ranh giới Enter-Tab dưới đây phản chiếu
// `TextVNInputController.handle` (test parity TẠM theo G22 cho tới SF-2).
//
// Chạy: TEXTVN_EMIT_KEYS=<dir> [TEXTVN_TRACE_OUT=<dir>] swift test --filter TraceTests
// G8: chỉ ghi chữ của corpus cố định, không nội dung người dùng.

import CoreBridge
import Foundation
import XCTest

@testable import IMKLib

/// `TextTarget` ghi lại thao tác thành lệnh session chuẩn hoá.
final class TraceRecordingTarget: TextTarget {
    private(set) var cmds: [(cmd: String, text: String, n: Int)] = []
    private var markedNow = ""

    func emit(_ cmd: String, _ text: String = "", _ n: Int = 0) {
        cmds.append((cmd, text, n))
    }

    func take() -> [(cmd: String, text: String, n: Int)] {
        defer { cmds.removeAll() }
        return cmds
    }

    /// NSTextInput: `insertText` thay marked range nếu có ⇒ commit rồi ẩn preedit (P0-6 §3).
    func insert(_ text: String, replacementRange: NSRange) {
        if !text.isEmpty {
            emit("commit", text)
        }
        if !markedNow.isEmpty {
            markedNow = ""
            emit("set_preedit")
        }
    }

    func setMarked(_ text: String, selectionRange: NSRange) {
        if text.isEmpty {
            if !markedNow.isEmpty {
                emit("set_preedit")
            }
        } else {
            emit("set_preedit", text, text.unicodeScalars.count)
        }
        markedNow = text
    }

    /// Nhả marked giữ nguyên chữ ⇒ chữ marked thành text thật.
    func unmark() {
        guard !markedNow.isEmpty else { return }
        emit("commit", markedNow)
        markedNow = ""
        emit("set_preedit")
    }

    func markedRange() -> NSRange {
        markedNow.isEmpty
            ? NSRange(location: NSNotFound, length: 0)
            : NSRange(location: 0, length: (markedNow as NSString).length)
    }

    func selectionRange() -> NSRange { NSRange(location: 0, length: 0) }

    func deleteBackward(count: Int) -> Bool {
        if count > 0 {
            emit("delete_before", "", count)
        }
        return true
    }

    func deleteSurroundingText(before: Int, after: Int) -> Bool {
        if before > 0 {
            emit("delete_before", "", before)
        }
        return true
    }
}

/// Một ca do `--emit-keys` xuất (schema trong `coordination/changes/SF-1c.md`).
struct EmittedCase: Decodable {
    struct App: Decodable {
        let bundle: String
        let field_role: UInt32
        let caps: UInt32
        let secure: Bool
        let enabled: Bool
    }

    struct Event: Decodable {
        let kind: String
        let step: Int?
        let vk: UInt32?
        let ch: UInt32?
        let mods: UInt32?
        let down: Int?
        let injected: Bool?
        let config_json: String?
    }

    let `case`: String
    let config_json: String
    let app: App
    let events: [Event]
}

enum MacTrace {
    /// Chạy một ca, trả các dòng JSONL (tất định).
    static func run(_ c: EmittedCase) throws -> [String] {
        let engine = try ImeEngine(
            configJSON: TextVNInputController.engineConfig(Data(c.config_json.utf8)))
        let caps = c.app.caps != 0 ? c.app.caps : TextVNInputController.imkCaps
        let appID = c.app.bundle.isEmpty ? "com.apple.textedit" : c.app.bundle
        let hint = StrategyResolver(appdb: nil, caps: caps).strategy(
            appID: appID, role: c.app.field_role, enabled: c.app.enabled, secure: c.app.secure)
        engine.setContext(
            enabled: c.app.enabled, secure: c.app.secure, fieldRole: c.app.field_role,
            caps: caps, appId: appID, elementName: nil, hint: hint)
        var nonPreedit = TextVNInputController.configNonPreedit(in: Data(c.config_json.utf8))
        let target = TraceRecordingTarget()
        let marked = MarkedState()
        var lines: [String] = []

        func flush(step: Int) {
            for cmd in target.take() {
                lines.append(line(c.case, step, cmd.cmd, cmd.text, cmd.n))
            }
        }

        for ev in c.events {
            let step = ev.step ?? 0
            switch ev.kind {
            case "config":
                if let json = ev.config_json {
                    _ = engine.reloadConfig(TextVNInputController.engineConfig(Data(json.utf8)))
                    nonPreedit = TextVNInputController.configNonPreedit(in: Data(json.utf8))
                }
            case "reset":
                engine.reset()
                marked.clear()
            case "key":
                let down = (ev.down ?? 1) == 1
                let key = KeyEvent(
                    vk: ev.vk ?? 0, ch: ev.ch ?? 0, mods: ev.mods ?? 0,
                    keyDown: down, isInjected: ev.injected ?? false)
                let outcome = engine.key(key)
                let eaten = try applyOutcome(
                    outcome, key: key, hint: hint, nonPreedit: nonPreedit,
                    engine: engine, target: target, marked: marked)
                if down {
                    target.emit(eaten ? "eat_key" : "forward_key")
                }
                flush(step: step)
            default:
                continue // expect / expect_preedit / mods / context: không đổi thao tác áp
            }
        }
        return lines
    }

    /// Phản chiếu nhánh áp outcome của `TextVNInputController.handle` (trợ lý TẮT). Trả `true` = nuốt.
    static func applyOutcome(
        _ outcome: KeyOutcome, key: KeyEvent, hint: Int64, nonPreedit: Bool,
        engine: ImeEngine, target: TraceRecordingTarget, marked: MarkedState
    ) throws -> Bool {
        if outcome.isError {
            return false
        }
        let strategy = TextVNInputController.outputStrategy(hint: hint, nonPreedit: nonPreedit)
        var toApply = outcome
        var forwardKey = false
        if outcome.isWordEnd,
           key.vk == TextVNInputController.vkReturn || key.vk == TextVNInputController.vkTab,
           let trimmed = TextVNInputController.withoutNativeBoundary(outcome) {
            toApply = trimmed
            forwardKey = true
        }
        switch toApply.action {
        case .pass:
            if !marked.text.isEmpty {
                target.insert(marked.text, replacementRange: .notFound)
                marked.clear()
                if !toApply.isConsumed {
                    engine.reset()
                }
            }
            return toApply.isConsumed
        case .replace, .commit, .restore:
            try ApplyReplace.apply(
                toApply, strategy: strategy, target: target, marked: marked,
                onReset: { engine.reset() })
            return !forwardKey
        }
    }

    static func line(_ c: String, _ step: Int, _ cmd: String, _ text: String, _ n: Int) -> String {
        "{\"case\":\(jsonString(c)),\"step\":\(step),\"cmd\":\"\(cmd)\",\"text\":\(jsonString(text)),\"n\":\(n)}"
    }

    /// JSON string tối giản, khớp `json_str` của Rust (escape `"` `\` và ký tự điều khiển).
    static func jsonString(_ s: String) -> String {
        var out = "\""
        for scalar in s.unicodeScalars {
            switch scalar {
            case "\"": out += "\\\""
            case "\\": out += "\\\\"
            case "\n": out += "\\n"
            case "\r": out += "\\r"
            case "\t": out += "\\t"
            default:
                if scalar.value < 0x20 {
                    out += String(format: "\\u%04x", scalar.value)
                } else {
                    out.unicodeScalars.append(scalar)
                }
            }
        }
        return out + "\""
    }
}

final class TraceTests: XCTestCase {
    private func repoRoot() -> URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent() // IMKLibTests
            .deletingLastPathComponent() // Tests
            .deletingLastPathComponent() // macos-imk
            .deletingLastPathComponent() // adapters
            .deletingLastPathComponent() // gốc repo
    }

    /// SF-1e: sinh `corpus/traces/mac/<rel>.jsonl` từ JSON phím của `--emit-keys`; chạy 2 lần
    /// phải giống hệt (tất định). Thiếu `TEXTVN_EMIT_KEYS` ⇒ SKIP có lý do (máy dev); CI luôn đặt.
    func testGenerateMacTracesFromEmittedKeys() throws {
        guard let input = ProcessInfo.processInfo.environment["TEXTVN_EMIT_KEYS"], !input.isEmpty else {
            throw XCTSkip("TEXTVN_EMIT_KEYS chưa đặt — chạy textvn-cli replay … --emit-keys <dir> trước")
        }
        let outRoot = ProcessInfo.processInfo.environment["TEXTVN_TRACE_OUT"].map {
            URL(fileURLWithPath: $0)
        } ?? repoRoot().appendingPathComponent("corpus/traces/mac")
        let fm = FileManager.default
        let inputURL = URL(fileURLWithPath: input)
        guard let walker = fm.enumerator(at: inputURL, includingPropertiesForKeys: nil) else {
            return XCTFail("không đọc được \(input)")
        }
        var files: [URL] = []
        for case let url as URL in walker where url.pathExtension == "json" {
            files.append(url)
        }
        files.sort { $0.path < $1.path }
        XCTAssertFalse(files.isEmpty, "TEXTVN_EMIT_KEYS=\(input) không có tệp .json nào")

        var written = 0
        for url in files {
            let c = try JSONDecoder().decode(EmittedCase.self, from: Data(contentsOf: url))
            let first = try MacTrace.run(c)
            let second = try MacTrace.run(c)
            XCTAssertEqual(first, second, "vết \(c.case) không tất định")
            XCTAssertFalse(first.isEmpty, "vết \(c.case) rỗng")
            let dest = outRoot.appendingPathComponent(c.case + ".jsonl")
            try fm.createDirectory(
                at: dest.deletingLastPathComponent(), withIntermediateDirectories: true)
            try (first.joined(separator: "\n") + "\n").write(to: dest, atomically: true, encoding: .utf8)
            written += 1
        }
        print("[SF-1e] đã ghi \(written) vết macOS vào \(outRoot.path)")
    }

    /// Bộ ghi: insert khi đang marked ⇒ commit TRƯỚC rồi ẩn preedit (P0-6 §3, R2-165/167).
    func testRecordingTargetOrdersCommitBeforeHidingPreedit() {
        let t = TraceRecordingTarget()
        t.setMarked("việ", selectionRange: NSRange(location: 3, length: 0))
        t.insert("việt", replacementRange: NSRange(location: NSNotFound, length: 0))
        let cmds = t.take().map { "\($0.cmd):\($0.text):\($0.n)" }
        XCTAssertEqual(cmds, ["set_preedit:việ:3", "commit:việt:0", "set_preedit::0"])
        XCTAssertEqual(MacTrace.jsonString("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"")
    }
}
