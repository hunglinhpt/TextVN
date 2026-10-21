// SPDX-License-Identifier: GPL-3.0-or-later
//! MacSpellProvider — OS spell-check provider macOS qua NSSpellChecker (AS-042, P4-10 §3).
//!
//! Thiết kế theo D32/D33 (P4-10 §3.3):
//! - Swift sở hữu NSSpellChecker (AppKit), Rust nhận con trỏ hàm C-ABI `ime_os_spell_v1`.
//! - Fail-open: không có từ điển "vi" / lỗi / timeout -> contains = 1 (đúng), suggest = 0 gợi ý.
//! - Quyền riêng tư (S2): KHÔNG bao giờ log từ / câu / text người dùng.
//! - Document tag: cấp `uniqueSpellDocumentTag()` riêng cho TextVN, giải phóng khi deinit.

import AppKit
import CoreBridge
import CTextVNFFI
import Foundation

/// C-ABI callback cho `ime_os_spell_v1.contains`.
///
/// Trả về:
/// - `1`: từ đúng, HOẶC fail-open khi lỗi / không có từ điển
/// - `0`: từ sai / lạ (cần báo spelling issue)
/// - `-1`: không rõ (Rust coi là fail-open)
private func macSpellContainsCallback(
    ctx: UnsafeMutableRawPointer?,
    wordUtf8: UnsafePointer<CChar>?,
    wordLen: UInt32
) -> Int32 {
    guard let ctx = ctx, let wordUtf8 = wordUtf8, wordLen > 0 else {
        return 1 // fail-open
    }
    let provider = Unmanaged<MacSpellProvider>.fromOpaque(ctx).takeUnretainedValue()
    return provider.cContains(wordUtf8: wordUtf8, wordLen: wordLen)
}

/// C-ABI callback cho `ime_os_spell_v1.suggest`.
///
/// Ghi các từ gợi ý cách nhau bởi `\0` vào `outBuf` (tối đa `outCap` bytes).
/// Trả về số lượng từ gợi ý đã ghi (0 nếu không có gợi ý hoặc fail-open).
private func macSpellSuggestCallback(
    ctx: UnsafeMutableRawPointer?,
    wordUtf8: UnsafePointer<CChar>?,
    wordLen: UInt32,
    outBuf: UnsafeMutablePointer<CChar>?,
    outCap: UInt32,
    limit: UInt32
) -> UInt32 {
    guard let ctx = ctx,
          let wordUtf8 = wordUtf8,
          wordLen > 0,
          let outBuf = outBuf,
          outCap > 0,
          limit > 0 else {
        return 0 // fail-open
    }
    let provider = Unmanaged<MacSpellProvider>.fromOpaque(ctx).takeUnretainedValue()
    return provider.cSuggest(
        wordUtf8: wordUtf8,
        wordLen: wordLen,
        outBuf: outBuf,
        outCap: outCap,
        limit: limit
    )
}

public final class MacSpellProvider {
    public static let shared = MacSpellProvider()

    public let language: String
    public let isAvailable: Bool

    private let checker: NSSpellChecker
    private let tag: Int

    /// Khởi tạo provider cho ngôn ngữ chỉ định (mặc định "vi").
    ///
    /// - Parameters:
    ///   - language: mã ngôn ngữ BCP-47 / ISO (ví dụ: "vi", "en").
    ///   - checker: instance NSSpellChecker (mặc định .shared).
    ///   - isAvailableOverride: ghi đè capability probe (dành cho test fail-open).
    public init(
        language: String = "vi",
        checker: NSSpellChecker = .shared,
        isAvailableOverride: Bool? = nil
    ) {
        self.language = language
        self.checker = checker
        self.tag = NSSpellChecker.uniqueSpellDocumentTag()

        if let overrideVal = isAvailableOverride {
            self.isAvailable = overrideVal
        } else {
            let prefix = language.lowercased()
            self.isAvailable = checker.availableLanguages.contains { code in
                let lower = code.lowercased()
                return lower == prefix || lower.hasPrefix(prefix + "-") || lower.hasPrefix(prefix + "_") || lower.hasPrefix(prefix)
            }
        }
        Diagnostics.log("MacSpellProvider init — lang=\(language) available=\(self.isAvailable)")
    }

    deinit {
        checker.closeSpellDocument(withTag: tag)
    }

    // MARK: - Swift API

    /// Kiểm tra từ có trong từ điển hay không.
    /// Fail-open: trả về `true` (coi như đúng) nếu không có từ điển hoặc gặp lỗi.
    public func contains(_ word: String) -> Bool {
        guard isAvailable else { return true }
        guard !word.isEmpty else { return true }

        let ns = word as NSString
        let r = checker.checkSpelling(
            of: word,
            startingAt: 0,
            language: language,
            wrap: false,
            inSpellDocumentWithTag: tag,
            wordCount: nil
        )
        // length == 0 nghĩa là NSSpellChecker không tìm thấy lỗi chính tả trong range
        return r.length == 0
    }

    /// Lấy danh sách gợi ý thay thế cho một từ (tối đa `limit` từ).
    /// Fail-open: trả về `[]` nếu không có từ điển hoặc gặp lỗi.
    public func suggest(_ word: String, limit: Int = 3) -> [String] {
        guard isAvailable else { return [] }
        guard !word.isEmpty, limit > 0 else { return [] }

        let ns = word as NSString
        let all = NSRange(location: 0, length: ns.length)
        var guesses = checker.guesses(
            forWordRange: all,
            in: word,
            language: language,
            inSpellDocumentWithTag: tag
        ) ?? []

        // Bổ sung single proposed correction nếu guesses rỗng
        if guesses.isEmpty, let corr = checker.correction(
            forWordRange: all,
            in: word,
            language: language,
            inSpellDocumentWithTag: tag
        ), !corr.isEmpty {
            guesses = [corr]
        }

        if guesses.count > limit {
            guesses = Array(guesses.prefix(limit))
        }
        return guesses
    }

    // MARK: - C-ABI Bridge

    /// Tạo struct `ime_os_spell_v1` để truyền vào engine qua FFI.
    public func makeCStruct() -> ime_os_spell_v1 {
        ime_os_spell_v1(
            abi_version: FFI.abiVersion,
            contains: macSpellContainsCallback,
            suggest: macSpellSuggestCallback,
            ctx: Unmanaged.passUnretained(self).toOpaque()
        )
    }

    /// Đăng ký provider này với một `ImeEngine`.
    /// `ImeEngine` sẽ giữ tham chiếu tới provider này để bảo vệ vòng đời.
    ///
    /// - Parameter force: đăng ký ngay cả khi `isAvailable == false` (dùng cho test C-ABI callback).
    @discardableResult
    public func register(with engine: ImeEngine, force: Bool = false) -> Bool {
        guard isAvailable || force else {
            engine.setSpellProvider(nil, retainRef: nil)
            return false
        }
        let cStruct = makeCStruct()
        let rc = engine.setSpellProvider(cStruct, retainRef: self)
        let ok = rc == FFI.ok
        Diagnostics.log("MacSpellProvider register — rc=\(rc)")
        return ok
    }

    /// Huỷ đăng ký provider khỏi `ImeEngine`.
    @discardableResult
    public func unregister(from engine: ImeEngine) -> Bool {
        let rc = engine.setSpellProvider(nil, retainRef: nil)
        let ok = rc == FFI.ok
        Diagnostics.log("MacSpellProvider unregister — rc=\(rc)")
        return ok
    }

    // MARK: - Internal C Callbacks Logic

    fileprivate func cContains(wordUtf8: UnsafePointer<CChar>, wordLen: UInt32) -> Int32 {
        guard isAvailable else { return 1 } // fail-open

        let data = Data(bytes: wordUtf8, count: Int(wordLen))
        guard let word = String(data: data, encoding: .utf8) else {
            return 1 // fail-open nếu không parse được UTF-8
        }

        return contains(word) ? 1 : 0
    }

    fileprivate func cSuggest(
        wordUtf8: UnsafePointer<CChar>,
        wordLen: UInt32,
        outBuf: UnsafeMutablePointer<CChar>,
        outCap: UInt32,
        limit: UInt32
    ) -> UInt32 {
        guard isAvailable else { return 0 } // fail-open

        let data = Data(bytes: wordUtf8, count: Int(wordLen))
        guard let word = String(data: data, encoding: .utf8) else {
            return 0 // fail-open
        }

        let suggestions = suggest(word, limit: Int(limit))
        guard !suggestions.isEmpty else { return 0 }

        var writtenCount: UInt32 = 0
        var currentOffset = 0
        let maxCap = Int(outCap)

        for item in suggestions {
            let cleanItem = item.replacingOccurrences(of: "\0", with: "")
            let utf8Count = cleanItem.utf8.count
            // Cần utf8Count + 1 byte cho ký tự '\0'
            if currentOffset + utf8Count + 1 > maxCap {
                break // Không đủ chỗ chứa tiếp
            }

            _ = cleanItem.withCString { cstr in
                memcpy(outBuf.advanced(by: currentOffset), cstr, utf8Count + 1)
            }
            currentOffset += utf8Count + 1
            writtenCount += 1
        }

        return writtenCount
    }
}
