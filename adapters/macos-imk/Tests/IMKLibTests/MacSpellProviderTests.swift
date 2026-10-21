// SPDX-License-Identifier: GPL-3.0-or-later
//! MacSpellProviderTests — unit tests cho macOS spellcheck provider (AS-042/AS-060, P4-10 §3).

import AppKit
import CoreBridge
import CTextVNFFI
import Foundation
import XCTest

@testable import IMKLib

final class MacSpellProviderTests: XCTestCase {

    func testMacSpellProviderInit() {
        let provider = MacSpellProvider()
        XCTAssertEqual(provider.language, "vi")
        // isAvailable phụ thuộc vào máy có bật từ điển tiếng Việt hay không (M1)
        _ = provider.isAvailable

        let cStruct = provider.makeCStruct()
        XCTAssertEqual(cStruct.abi_version, FFI.abiVersion)
        XCTAssertNotNil(cStruct.contains)
        XCTAssertNotNil(cStruct.suggest)
        XCTAssertNotNil(cStruct.ctx)
    }

    func testMacSpellProviderFailOpenWhenUnavailable() {
        let provider = MacSpellProvider(language: "vi", isAvailableOverride: false)
        XCTAssertFalse(provider.isAvailable)

        // Fail-open: mọi từ đều được xem là đúng khi không có từ điển
        XCTAssertTrue(provider.contains("từ_lạ_không_rõ"))
        XCTAssertTrue(provider.contains(""))
        XCTAssertTrue(provider.contains("sai_chinh_ta"))

        // Fail-open: không gợi ý gì cả
        XCTAssertEqual(provider.suggest("sai_chinh_ta"), [])
        XCTAssertEqual(provider.suggest(""), [])
    }

    func testMacSpellProviderCAbiCallbacksFailOpen() {
        let provider = MacSpellProvider(language: "vi", isAvailableOverride: false)
        let cStruct = provider.makeCStruct()

        guard let containsFn = cStruct.contains, let suggestFn = cStruct.suggest else {
            XCTFail("C callbacks must not be nil")
            return
        }

        // Test contains callback với provider unavailable -> fail-open trả 1
        "testword".withCString { cstr in
            let rc = containsFn(cStruct.ctx, cstr, UInt32(strlen(cstr)))
            XCTAssertEqual(rc, 1, "contains fail-open must return 1")
        }

        // Test contains callback với null arguments -> fail-open trả 1
        XCTAssertEqual(containsFn(nil, nil, 0), 1)
        XCTAssertEqual(containsFn(cStruct.ctx, nil, 0), 1)

        // Test suggest callback với provider unavailable -> fail-open trả 0
        var buf = [CChar](repeating: 0, count: 256)
        "testword".withCString { cstr in
            let count = suggestFn(cStruct.ctx, cstr, UInt32(strlen(cstr)), &buf, UInt32(buf.count), 3)
            XCTAssertEqual(count, 0, "suggest fail-open must return 0")
        }

        // Test suggest callback với null arguments -> fail-open trả 0
        XCTAssertEqual(suggestFn(nil, nil, 0, nil, 0, 0), 0)
        XCTAssertEqual(suggestFn(cStruct.ctx, nil, 0, &buf, UInt32(buf.count), 3), 0)
        "testword".withCString { cstr in
            XCTAssertEqual(suggestFn(cStruct.ctx, cstr, UInt32(strlen(cstr)), nil, 0, 3), 0)
        }
    }

    func testMacSpellProviderSuggestBufferFormatting() {
        // Test kiểm tra ghi vào buffer C: các từ phân cách bởi \0
        let provider = MacSpellProvider(language: "en")
        let cStruct = provider.makeCStruct()

        guard let suggestFn = cStruct.suggest else {
            XCTFail("suggest callback missing")
            return
        }

        var buf = [CChar](repeating: 0, count: 512)
        let count = "helo".withCString { cstr in
            suggestFn(cStruct.ctx, cstr, UInt32(strlen(cstr)), &buf, UInt32(buf.count), 3)
        }

        if provider.isAvailable && count > 0 {
            // Đọc lại các từ null-separated từ buffer
            var parsedWords: [String] = []
            buf.withUnsafeBufferPointer { ptr in
                guard let base = ptr.baseAddress else { return }
                var offset = 0
                for _ in 0..<count {
                    if let s = String(validatingUTF8: base.advanced(by: offset)) {
                        parsedWords.append(s)
                        offset += s.utf8.count + 1
                    }
                }
            }
            XCTAssertEqual(parsedWords.count, Int(count))
            XCTAssertFalse(parsedWords.isEmpty)
        }
    }

    func testMacSpellProviderBufferCapacityExhaustion() {
        // Buffer quá bé (ví dụ chỉ 2 byte) -> không thể ghi đủ từ gợi ý
        let provider = MacSpellProvider(language: "en")
        let cStruct = provider.makeCStruct()

        guard let suggestFn = cStruct.suggest else {
            XCTFail("suggest callback missing")
            return
        }

        var tinyBuf = [CChar](repeating: 0, count: 2)
        let count = "helo".withCString { cstr in
            suggestFn(cStruct.ctx, cstr, UInt32(strlen(cstr)), &tinyBuf, UInt32(tinyBuf.count), 3)
        }
        // Với buffer 2 byte, gợi ý từ tiếng Anh dài hơn 1 ký tự sẽ không vừa -> count = 0
        XCTAssertEqual(count, 0)
    }

    func testEngineSetSpellProviderLifecycle() throws {
        let engine = try ImeEngine(configJSON: nil)
        let provider = MacSpellProvider(language: "vi", isAvailableOverride: false)

        // 1. Gắn struct trực tiếp
        let rc1 = engine.setSpellProvider(provider.makeCStruct())
        XCTAssertEqual(rc1, FFI.ok)

        // 2. Huỷ provider -> trở về NoDictionary
        let rc2 = engine.setSpellProvider(nil)
        XCTAssertEqual(rc2, FFI.ok)

        // 3. Đăng ký qua helper register
        let registered = provider.register(with: engine, force: true)
        XCTAssertTrue(registered)

        // 4. Huỷ qua helper unregister
        let unregistered = provider.unregister(from: engine)
        XCTAssertTrue(unregistered)
    }

    func testCustomLanguageIfAvailable() {
        // Trên macOS, từ điển "en" gần như luôn khả dụng
        let enProvider = MacSpellProvider(language: "en")
        guard enProvider.isAvailable else { return }

        // Từ đúng tiếng Anh
        XCTAssertTrue(enProvider.contains("hello"))
        XCTAssertTrue(enProvider.contains("world"))

        // Từ sai chính tả
        XCTAssertFalse(enProvider.contains("asdzxcqweqwe123"))

        // Gợi ý
        let suggestions = enProvider.suggest("helo", limit: 3)
        XCTAssertFalse(suggestions.isEmpty)
        XCTAssertLessThanOrEqual(suggestions.count, 3)
        XCTAssertTrue(suggestions.contains { $0.lowercased() == "hello" || $0.lowercased() == "help" })
    }
}
