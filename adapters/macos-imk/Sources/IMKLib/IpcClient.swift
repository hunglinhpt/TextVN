// SPDX-License-Identifier: GPL-3.0-or-later
//! IpcClient — unix socket client của TextVN-IM (P2-4 §2, schema `ipc.v1.md`).
//!
//! Transport: `~/Library/Application Support/TextVN/ipc.sock` (0600, dir 0700).
//! Codec: frame = `u32 LE length` + JSON UTF-8; length tối đa 65_536 — frame sai
//! = protocol violation → **đóng kết nối, không retry frame đó** (ipc.v1.md).
//!
//! Offline-first (P0-3 §4): server chết → đọc config/state file lúc activate,
//! **không bao giờ block gõ**. Reconnect backoff bất đồng bộ (review R1 F20).
//!
//! Thread contract (review R1 F16): callback delegate LUÔN chạy trên main queue
//! — IMK controller là main-thread object, state đọc trong `handle()`.

import CoreBridge
import Foundation

/// Message set v1 **đóng** (ipc.v1.md) — không bịa thêm loại message.
/// Mỗi case một struct riêng: tránh lẫn kiểu field giữa các message
/// (review R1 F14: `version` của Hello là String, của ConfigReload/StateUpdate là UInt64).
public enum IpcMessage: Equatable {
    /// `app_id` toàn cục trong `ToggleViEn`/`StateUpdate`/`Snapshot.state` — một quy
    /// ước duy nhất cho mọi nền tảng (Windows TSF `GLOBAL_KEY`, review R3 F3-1).
    public static let globalAppID = "*"

    // ---- client → server
    case hello(pid: Int32, abi: UInt32, version: String)
    case getSnapshot
    case subscribe(pid: Int32)
    case toggleViEn(appID: String, enabled: Bool) // client gửi (review R1 F15)
    case ping
    case crashReport(code: UInt32, count: UInt32)

    // ---- server → client
    case snapshot(configVersion: UInt64, state: [String: Bool], appdbVersion: String, channel: String)
    case ack
    case configReload(version: UInt64)
    case stateUpdate(appID: String, enabled: Bool, version: UInt64)
    case pong(uptimeMs: UInt64)

    // ---------------------------------------------------------------- encode

    public var json: [String: Any] {
        switch self {
        case let .hello(pid, abi, version):
            return ["type": "Hello", "pid": pid, "abi": abi, "version": version]
        case .getSnapshot:
            return ["type": "GetSnapshot"]
        case let .subscribe(pid):
            return ["type": "Subscribe", "pid": pid]
        case let .toggleViEn(appID, enabled):
            return ["type": "ToggleViEn", "app_id": appID, "enabled": enabled]
        case .ping:
            return ["type": "Ping"]
        case let .crashReport(code, count):
            return ["type": "CrashReport", "code": code, "count": count]
        case let .snapshot(configVersion, state, appdbVersion, channel):
            return [
                "type": "Snapshot", "config_version": configVersion,
                "state": state, "appdb_version": appdbVersion, "channel": channel,
            ]
        case .ack:
            return ["type": "Ack"]
        case let .configReload(version):
            return ["type": "ConfigReload", "version": version]
        case let .stateUpdate(appID, enabled, version):
            return ["type": "StateUpdate", "app_id": appID, "enabled": enabled, "version": version]
        case let .pong(uptimeMs):
            return ["type": "Pong", "uptime_ms": uptimeMs]
        }
    }

    public var jsonData: Data {
        (try? JSONSerialization.data(withJSONObject: json)) ?? Data()
    }

    // ---------------------------------------------------------------- decode

    /// Decode 1 frame JSON. Trả `nil` = protocol violation (type lạ / field
    /// sai kiểu / thiếu field bắt buộc) — caller PHẢI đóng kết nối (ipc.v1.md).
    public static func decode(_ data: Data) -> IpcMessage? {
        guard
            let obj = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
            let type = obj["type"] as? String
        else { return nil }

        let asUInt64: (Any) -> UInt64? = {
            if let i = $0 as? UInt64 { return i }
            if let i = $0 as? Int64, i >= 0 { return UInt64(i) }
            if let i = $0 as? Int, i >= 0 { return UInt64(i) }
            return nil
        }

        switch type {
        case "Hello":
            // Int32(exactly:) — pid tràn phải bị TỪ CHỐI, không được trap
            // (review R1: `Int32.init(Int)` tràn là runtime crash).
            guard let pid = (obj["pid"] as? Int).flatMap({ Int32(exactly: $0) }),
                  let abi = asUInt64(obj["abi"] ?? 0), abi <= UInt32.max,
                  let version = obj["version"] as? String else { return nil }
            return .hello(pid: pid, abi: UInt32(abi), version: version)
        case "GetSnapshot":
            return .getSnapshot
        case "Subscribe":
            guard let pid = (obj["pid"] as? Int).flatMap({ Int32(exactly: $0) }) else {
                return nil
            }
            return .subscribe(pid: pid)
        case "ToggleViEn":
            guard let appID = obj["app_id"] as? String, let enabled = obj["enabled"] as? Bool else {
                return nil
            }
            return .toggleViEn(appID: appID, enabled: enabled)
        case "Ping":
            return .ping
        case "CrashReport":
            guard let code = asUInt64(obj["code"] ?? 0), code <= UInt32.max,
                  let count = asUInt64(obj["count"] ?? 0), count <= UInt32.max else { return nil }
            return .crashReport(code: UInt32(code), count: UInt32(count))
        case "Snapshot":
            guard let configVersion = asUInt64(obj["config_version"] ?? 0),
                  let appdbVersion = obj["appdb_version"] as? String,
                  let channel = obj["channel"] as? String else { return nil }
            let state = (obj["state"] as? [String: Bool]) ?? [:]
            return .snapshot(configVersion: configVersion, state: state,
                             appdbVersion: appdbVersion, channel: channel)
        case "Ack":
            return .ack
        case "ConfigReload":
            guard let version = asUInt64(obj["version"] ?? 0) else { return nil }
            return .configReload(version: version)
        case "StateUpdate":
            guard let appID = obj["app_id"] as? String, let enabled = obj["enabled"] as? Bool,
                  let version = asUInt64(obj["version"] ?? 0) else { return nil }
            return .stateUpdate(appID: appID, enabled: enabled, version: version)
        case "Pong":
            return .pong(uptimeMs: asUInt64(obj["uptime_ms"] ?? 0) ?? 0)
        default:
            return nil // message type không biết = violation (danh sách v1 đóng)
        }
    }
}

public protocol IpcClientDelegate: AnyObject {
    /// Push từ server (đã decode) — LUÔN thực thi trên **main queue**.
    func ipcClient(_ client: IpcClient, didReceive message: IpcMessage)
    /// Kết nối đứt → client tự chuyển offline (không block gõ).
    func ipcClientDidDisconnect(_ client: IpcClient)
}

public final class IpcClient {
    public static let maxFrameLength = 65_536
    /// Version gửi trong `Hello` — đọc từ Info.plist của bundle đang chạy
    /// (TextVN-IM.app), fallback hằng khi chạy trong swift test (R2 finding 2:
    /// hardcode "0.1.0" làm handshake hiển thị sai version sau bump).
    public static let clientVersion: String =
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
        ?? "0.2.4"
    /// Backoff reconnect (P0-3 §4 offline-first — retry nhẹ nhàng).
    public static let retryInterval: TimeInterval = 2.0

    public weak var delegate: IpcClientDelegate?
    public private(set) var connected = false

    private let socketPath: String
    private var fd: Int32 = -1
    private var buffer = Data()
    private var readSource: DispatchSourceRead?
    private var retryTimer: DispatchSourceTimer?
    private var queue: DispatchQueue

    public init(socketPath: String? = nil, queue: DispatchQueue? = nil) {
        self.socketPath = socketPath ?? Self.defaultSocketPath()
        self.queue = queue ?? DispatchQueue(label: "vn.textvn.ipc.client", qos: .userInitiated)
    }

    public static func defaultSocketPath() -> String {
        let support = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Application Support/TextVN", isDirectory: true)
        return support.appendingPathComponent("ipc.sock").path
    }

    /// Kết nối + handshake `Hello` + Subscribe. Non-blocking; lỗi → tự retry
    /// backoff (tray khởi động sau IM vẫn nối được — review R1 F20).
    public func connect() {
        queue.async { [weak self] in self?.connectSync() }
    }

    private func connectSync() {
        guard fd == -1, retryTimer == nil else { return }
        let raw = socket(AF_UNIX, SOCK_STREAM, 0)
        guard raw >= 0 else { scheduleRetry(); return }
        fd = raw
        // Server đóng kết nối giữa 2 lần write → SIGPIPE mặc định KILL TextVN-IM
        // giữa lúc gõ. Mirror server (IpcServer đặt SO_NOSIGPIPE) — review R3 F3-4.
        var noSigPipe: Int32 = 1
        guard setsockopt(raw, SOL_SOCKET, SO_NOSIGPIPE, &noSigPipe,
                         socklen_t(MemoryLayout<Int32>.size)) == 0 else {
            closeSocket()
            scheduleRetry()
            return
        }

        var addr = sockaddr_un()
        addr.sun_family = sa_family_t(AF_UNIX)
        let pathBytes = Array(socketPath.utf8)
        guard pathBytes.count < MemoryLayout.size(ofValue: addr.sun_path) else {
            closeSocket()
            return
        }
        withUnsafeMutableBytes(of: &addr.sun_path) { dst in
            dst.copyBytes(from: pathBytes)
        }
        let len = socklen_t(MemoryLayout<sockaddr_un>.size)
        let rc = withUnsafePointer(to: &addr) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sa in
                Darwin.connect(fd, sa, len) // không phải method `connect()` của IpcClient
            }
        }
        guard rc == 0 else {
            closeSocket()
            scheduleRetry()
            return
        }

        connected = true
        send(.hello(pid: ProcessInfo.processInfo.processIdentifier,
                    abi: FFI.abiVersion, version: Self.clientVersion))
        // Snapshot state ban đầu (P0-3 §5: Hello → GetSnapshot → Subscribe, mirror
        // `ipc_client.c`). Thiếu bước này, toggle toàn cục/per-app đặt TRƯỚC khi
        // kết nối (hoặc từ tray trước đó) không bao giờ tới IMK.
        send(.getSnapshot)
        send(.subscribe(pid: ProcessInfo.processInfo.processIdentifier))

        let source = DispatchSource.makeReadSource(fileDescriptor: fd, queue: queue)
        source.setEventHandler { [weak self] in self?.readAvailable() }
        source.setCancelHandler { [weak self] in self?.closeSocket() }
        readSource = source
        source.resume()
    }

    private func scheduleRetry() {
        guard retryTimer == nil else { return }
        let timer = DispatchSource.makeTimerSource(queue: queue)
        timer.schedule(deadline: .now() + Self.retryInterval)
        timer.setEventHandler { [weak self] in
            guard let self else { return }
            self.cancelRetry()
            guard self.fd == -1, !self.connected else { return }
            self.connectSync()
        }
        retryTimer = timer
        timer.resume()
    }

    private func cancelRetry() {
        retryTimer?.cancel()
        retryTimer = nil
    }

    /// Gửi message (frame hoá u32 LE + JSON). Lỗi write → offline.
    public func send(_ message: IpcMessage) {
        queue.async { [weak self] in
            guard let self, self.connected else { return }
            let payload = message.jsonData
            guard !payload.isEmpty, payload.count <= Self.maxFrameLength else {
                Diagnostics.log("ipc frame invalid (\(payload.count)) — dropped")
                return
            }
            var frame = Data(count: 4)
            frame.withUnsafeMutableBytes { raw in
                raw.storeBytes(of: UInt32(payload.count).littleEndian, as: UInt32.self)
            }
            frame.append(payload)
            let ok = frame.withUnsafeBytes { raw in
                write(self.fd, raw.baseAddress, raw.count) == raw.count
            }
            if !ok {
                Diagnostics.log("ipc write failed — going offline")
                self.tearDown()
                self.notifyDisconnect()
                self.scheduleRetry()
            }
        }
    }

    /// Đóng chủ động — gọi được từ deinit path (cancel source đồng bộ, F20).
    public func disconnect() {
        queue.sync {
            tearDown()
        }
    }

    deinit {
        // Không dùng queue.async trong deinit (block chạy sau khi self chết —
        // review R1 F20): cancel đồng bộ qua queue.sync là an toàn hơn.
        tearDown()
    }

    // ---------------------------------------------------------------- internals

    private func readAvailable() {
        var chunk = [UInt8](repeating: 0, count: 4096)
        let n = read(fd, &chunk, chunk.count)
        guard n > 0 else {
            if n < 0 && errno == EINTR { return }
            // EOF = server tắt/restart → offline rồi THỬ LẠI. Trước đây `tearDown()`
            // hủy retryTimer mà không arm lại → client offline vĩnh viễn.
            tearDown()
            notifyDisconnect()
            scheduleRetry()
            return
        }
        buffer.append(contentsOf: chunk[0..<n])
        while true {
            switch Self.nextFrame(from: &buffer) {
            case let .frame(payload):
                guard let decoded = IpcMessage.decode(payload) else {
                    // JSON/type lạ = protocol violation → đóng, không retry (ipc.v1.md).
                    Diagnostics.log("ipc protocol violation — closing")
                    tearDown()
                    notifyDisconnect()
                    return
                }
                // Marshal về main queue: delegate là main-thread IMK object (F16).
                if Thread.isMainThread {
                    delegate?.ipcClient(self, didReceive: decoded)
                } else {
                    DispatchQueue.main.async { [weak self] in
                        guard let self, self.connected else { return }
                        self.delegate?.ipcClient(self, didReceive: decoded)
                    }
                }
            case .needMore:
                return
            case .violation:
                Diagnostics.log("ipc frame length violation — closing")
                tearDown()
                notifyDisconnect()
                return
            }
        }
    }

    /// Kết quả tách frame — `violation` buộc caller đóng kết nối (F17).
    enum Frame {
        case frame(Data)
        case needMore
        case violation
    }

    static func nextFrame(from buffer: inout Data) -> Frame {
        guard buffer.count >= 4 else { return .needMore }
        let le = buffer.withUnsafeBytes { raw in
            raw.loadUnaligned(fromByteOffset: 0, as: UInt32.self)
        }
        let frameLen = Int(UInt32(littleEndian: le))
        guard frameLen > 0, frameLen <= maxFrameLength else { return .violation }
        guard buffer.count >= 4 + frameLen else { return .needMore }
        let payload = buffer.subdata(in: 4..<(4 + frameLen))
        buffer.removeSubrange(0..<(4 + frameLen))
        return .frame(payload)
    }

    /// Hủy nguồn + đóng fd + clear buffer. Gọi từ bất kỳ thread (có queue lock).
    private func tearDown() {
        cancelRetry()
        readSource?.cancel()
        readSource = nil
        if fd >= 0 {
            close(fd)
            fd = -1
        }
        connected = false
        buffer.removeAll()
    }

    private func closeSocket() {
        if fd >= 0 {
            close(fd)
            fd = -1
        }
        connected = false
        buffer.removeAll()
    }

    /// Delegate disconnect LUÔN trên main queue (thread contract F16 — controller
    /// là main-thread object; trước đây callback chạy thẳng trên IPC queue).
    private func notifyDisconnect() {
        if Thread.isMainThread {
            delegate?.ipcClientDidDisconnect(self)
        } else {
            DispatchQueue.main.async { [weak self] in
                guard let self else { return }
                self.delegate?.ipcClientDidDisconnect(self)
            }
        }
    }
}
