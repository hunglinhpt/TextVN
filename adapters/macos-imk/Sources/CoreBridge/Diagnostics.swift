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
        // F25: append thay vì ghi đè từ byte 0 (relaunch giữ log cũ).
        try? fileHandle?.seekToEndOfFile()
        log("TextVN-IM log opened (pid \(ProcessInfo.processInfo.processIdentifier))")
    }

    /// Đóng log lúc terminate.
    public static func stop() {
        lock.lock()
        defer { lock.unlock() }
        try? fileHandle?.close()
        fileHandle = nil
    }

    // MARK: - Heartbeat (P2-4 §6)

    private static let heartbeatLock = NSLock()
    private static var heartbeatTimer: DispatchSourceTimer?

    /// Ghi `~/Library/Application Support/TextVN/im-heartbeat.json` mỗi `interval`
    /// giây (pid + timestamp). TextVN.app đọc file này ở mục "Sức khỏe hệ thống"
    /// — thiếu nó, health không biết IMK còn sống hay không.
    public static func startHeartbeat(interval: TimeInterval = 5) {
        heartbeatLock.lock()
        defer { heartbeatLock.unlock() }
        guard heartbeatTimer == nil else { return }
        let queue = DispatchQueue(label: "vn.textvn.im.heartbeat", qos: .utility)
        let timer = DispatchSource.makeTimerSource(queue: queue)
        timer.schedule(deadline: .now(), repeating: interval, leeway: .seconds(1))
        timer.setEventHandler { writeHeartbeat() }
        heartbeatTimer = timer
        timer.resume()
    }

    /// Một lần ghi heartbeat (tách để test/đọc trực tiếp).
    public static func writeHeartbeat(now: Date = Date()) {
        let fm = FileManager.default
        let dir = fm.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Application Support/TextVN", isDirectory: true)
        try? fm.createDirectory(at: dir, withIntermediateDirectories: true, attributes: [
            .posixPermissions: 0o700
        ])
        let payload: [String: Any] = [
            "pid": Int(ProcessInfo.processInfo.processIdentifier),
            "timestamp_ms": now.timeIntervalSince1970 * 1000,
            "version": Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "",
        ]
        guard let data = try? JSONSerialization.data(withJSONObject: payload) else { return }
        let url = dir.appendingPathComponent("im-heartbeat.json")
        try? data.write(to: url, options: .atomic)
    }
}
