// SPDX-License-Identifier: GPL-3.0-or-later
//! Diagnostics — log **không chứa text người dùng** (S2, P2-1 §11).
//!
//! Ghi duy nhất: state transition, action id, **độ dài** chuỗi, error code.
//! Không bao giờ: ký tự người gõ, nội dung preedit/insert, tên file.
//! Bật bằng env `TEXTVN_LOG=1` → `~/Library/Logs/TextVN/im-<pid>.log`.
//! Log lỗi/stale không bao giờ được làm crash hay block typing.

import Foundation

public enum Diagnostics {
    private static let lock = NSLock()
    private static var fileHandle: FileHandle?
    /// Crash counter (P2-1 §12) — status item đọc qua `crashCount`.
    private static var crashes = 0

    public static var crashCount: Int { lock.lock(); defer { lock.unlock() }; return crashes }

    public static func recordCrash() {
        lock.lock()
        crashes += 1
        lock.unlock()
        log("engine crash recorded (counter=\(crashes))")
    }

    /// Ghi 1 dòng log (best-effort, không block gõ — FileHandle buffer ở OS level).
    public static func log(_ message: String) {
        lock.lock()
        defer { lock.unlock() }
        guard let handle = fileHandle else { return }
        let ts = Date().timeIntervalSince1970
        let line = "\(String(format: "%.3f", ts)) im: \(message)\n"
        handle.write(line.data(using: .utf8) ?? Data())
    }

    /// Mở log file nếu `TEXTVN_LOG=1`. Gọi 1 lần lúc launch.
    public static func start() {
        lock.lock()
        defer { lock.unlock() }
        guard ProcessInfo.processInfo.environment["TEXTVN_LOG"] == "1" else { return }
        let fm = FileManager.default
        let dir = fm.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Logs/TextVN", isDirectory: true)
        try? fm.createDirectory(at: dir, withIntermediateDirectories: true)
        let url = dir.appendingPathComponent("im-\(ProcessInfo.processInfo.processIdentifier).log")
        if !fm.fileExists(atPath: url.path) {
            fm.createFile(atPath: url.path, contents: nil)
        }
        fileHandle = FileHandle(forWritingAtPath: url.path)
        log("TextVN-IM log opened (pid \(ProcessInfo.processInfo.processIdentifier))")
    }

    /// Đóng log lúc terminate.
    public static func stop() {
        lock.lock()
        defer { lock.unlock() }
        try? fileHandle?.close()
        fileHandle = nil
    }
}
