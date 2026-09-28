// SPDX-License-Identifier: GPL-3.0-or-later
//! IpcClient — unix socket client của TextVN-IM (P2-4 §2, schema `ipc.v1.md`).
//!
//! Transport: `~/Library/Application Support/TextVN/ipc.sock` (0600, dir 0700).
//! Codec: frame = `u32 LE length` + JSON UTF-8; length tối đa 65_536 — frame sai
//! = protocol violation → đóng kết nối, **không retry frame đó** (ipc.v1.md).
//!
//! Offline-first (P0-3 §4): server chết → đọc config/state file lúc activate,
//! **không bao giờ block gõ**. Kết nối lại theo nhịp retry bất đồng bộ.

import CoreBridge
import Foundation

/// Message set v1 **đóng** (ipc.v1.md) — không bịa thêm loại message.
public enum IpcMessage: Equatable {
    case hello(pid: Int32, abi: UInt32, version: String)
    case getSnapshot
    case subscribe(pid: Int32)
    case ping
    case snapshot(configVersion: UInt64, state: [String: Bool], appdbVersion: String, channel: String)
    case ack
    case configReload(version: UInt64)
    case stateUpdate(appID: String, enabled: Bool, version: UInt64)
    case pong(uptimeMs: UInt64)
    case crashReport(code: UInt32, count: UInt32)

    // MARK: Codable

    private enum Kind: String, Codable {
        case hello, getSnapshot, subscribe, ping
        case snapshot, ack, configReload, stateUpdate, pong, crashReport
    }

    private struct Wire: Codable {
        var type: Kind
        var pid: Int32?
        var abi: UInt32?
        var version: String?
        var config_version: UInt64?
        var state: [String: Bool]?
        var appdb_version: String?
        var channel: String?
        var app_id: String?
        var enabled: Bool?
        var uptime_ms: UInt64?
        var code: UInt32?
        var count: UInt32?
    }

    public var jsonData: Data {
        let wire: Wire
        switch self {
        case let .hello(pid, abi, version):
            wire = Wire(type: .hello, pid: pid, abi: abi, version: version,
                        config_version: nil, state: nil, appdb_version: nil,
                        channel: nil, app_id: nil, enabled: nil, uptime_ms: nil,
                        code: nil, count: nil)
        case .getSnapshot:
            wire = Wire(type: .getSnapshot, pid: nil, abi: nil, version: nil,
                        config_version: nil, state: nil, appdb_version: nil,
                        channel: nil, app_id: nil, enabled: nil, uptime_ms: nil,
                        code: nil, count: nil)
        case let .subscribe(pid):
            wire = Wire(type: .subscribe, pid: pid, abi: nil, version: nil,
                        config_version: nil, state: nil, appdb_version: nil,
                        channel: nil, app_id: nil, enabled: nil, uptime_ms: nil,
                        code: nil, count: nil)
        case .ping:
            wire = Wire(type: .ping, pid: nil, abi: nil, version: nil,
                        config_version: nil, state: nil, appdb_version: nil,
                        channel: nil, app_id: nil, enabled: nil, uptime_ms: nil,
                        code: nil, count: nil)
        case let .snapshot(configVersion, state, appdbVersion, channel):
            wire = Wire(type: .snapshot, pid: nil, abi: nil, version: nil,
                        config_version: configVersion, state: state,
                        appdb_version: appdbVersion, channel: channel,
                        app_id: nil, enabled: nil, uptime_ms: nil, code: nil, count: nil)
        case .ack:
            wire = Wire(type: .ack, pid: nil, abi: nil, version: nil,
                        config_version: nil, state: nil, appdb_version: nil,
                        channel: nil, app_id: nil, enabled: nil, uptime_ms: nil,
                        code: nil, count: nil)
        case let .configReload(version):
            wire = Wire(type: .configReload, pid: nil, abi: nil, version: nil,
                        config_version: version, state: nil, appdb_version: nil,
                        channel: nil, app_id: nil, enabled: nil, uptime_ms: nil,
                        code: nil, count: nil)
        case let .stateUpdate(appID, enabled, version):
            wire = Wire(type: .stateUpdate, pid: nil, abi: nil, version: nil,
                        config_version: nil, state: nil, appdb_version: nil,
                        channel: nil, app_id: appID, enabled: enabled,
                        uptime_ms: nil, code: nil, count: nil)
        case let .pong(uptimeMs):
            wire = Wire(type: .pong, pid: nil, abi: nil, version: nil,
                        config_version: nil, state: nil, appdb_version: nil,
                        channel: nil, app_id: nil, enabled: nil,
                        uptime_ms: uptimeMs, code: nil, count: nil)
        case let .crashReport(code, count):
            wire = Wire(type: .crashReport, pid: nil, abi: nil, version: nil,
                        config_version: nil, state: nil, appdb_version: nil,
                        channel: nil, app_id: nil, enabled: nil,
                        uptime_ms: nil, code: code, count: count)
        }
        return (try? JSONEncoder().encode(wire)) ?? Data()
    }

    public static func decode(_ data: Data) -> IpcMessage? {
        guard let wire = try? JSONDecoder().decode(Wire.self, from: data) else { return nil }
        switch wire.type {
        case .hello:
            guard let p = wire.pid, let a = wire.abi, let v = wire.version else { return nil }
            return .hello(pid: p, abi: a, version: v)
        case .getSnapshot: return .getSnapshot
        case .subscribe: return wire.pid.map { .subscribe(pid: $0) }
        case .ping: return .ping
        case .snapshot:
            return .snapshot(
                configVersion: wire.config_version ?? 0,
                state: wire.state ?? [:],
                appdbVersion: wire.appdb_version ?? "",
                channel: wire.channel ?? "stable"
            )
        case .ack: return .ack
        case .configReload: return .configReload(version: wire.config_version ?? 0)
        case .stateUpdate:
            guard let app = wire.app_id, let en = wire.enabled else { return nil }
            return .stateUpdate(appID: app, enabled: en, version: wire.config_version ?? 0)
        case .pong: return .pong(uptimeMs: wire.uptime_ms ?? 0)
        case .crashReport:
            return .crashReport(code: wire.code ?? 0, count: wire.count ?? 0)
        }
    }
}

public protocol IpcClientDelegate: AnyObject {
    /// Push từ server (đã decode) — thực thi trên queue của caller.
    func ipcClient(_ client: IpcClient, didReceive message: IpcMessage)
    /// Kết nối đứt → client tự chuyển offline (không block gõ).
    func ipcClientDidDisconnect(_ client: IpcClient)
}

public final class IpcClient {
    public static let maxFrameLength = 65_536

    public weak var delegate: IpcClientDelegate?
    public private(set) var connected = false

    private let socketPath: String
    private var fd: Int32 = -1
    private var buffer = Data()
    private var readSource: DispatchSourceRead?
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

    /// Kết nối + handshake `Hello`. Non-blocking: lỗi → offline, trả `false`.
    @discardableResult
    public func connect() -> Bool {
        queue.async { [weak self] in self?.connectSync() }
        return true // caller không chờ — trạng thái thật đọc qua `connected`
    }

    private func connectSync() {
        guard fd == -1 else { return }
        let result = socket(AF_UNIX, SOCK_STREAM, 0)
        guard result >= 0 else { return }
        fd = result

        var addr = sockaddr_un()
        addr.sun_family = sa_family_t(AF_UNIX)
        let pathBytes = Array(socketPath.utf8)
        guard pathBytes.count < MemoryLayout.size(ofValue: addr.sun_path) else {
            closeSocket()
            return
        }
        withUnsafeMutableBytes(of: &addr.sun_path) { raw in
            raw.baseAddress?.copyBytes(from: pathBytes)
        }
        let len = socklen_t(MemoryLayout<sockaddr_un>.size)
        let connectResult = withUnsafePointer(to: &addr) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sa in
                connect(fd, sa, len)
            }
        }
        guard connectResult == 0 else {
            closeSocket()
            return
        }

        connected = true
        send(.hello(pid: ProcessInfo.processInfo.processIdentifier,
                    abi: FFI.abiVersion, version: Self.clientVersion))
        send(.subscribe(pid: ProcessInfo.processInfo.processIdentifier))

        let source = DispatchSource.makeReadSource(fileDescriptor: fd, queue: queue)
        source.setEventHandler { [weak self] in self?.readAvailable() }
        source.setCancelHandler { [weak self] in self?.closeSocket() }
        readSource = source
        source.resume()
    }

    /// Gửi message (frame hoá). Lỗi write → đóng + offline.
    public func send(_ message: IpcMessage) {
        queue.async { [weak self] in
            guard let self, self.connected else { return }
            let payload = message.jsonData
            guard payload.count <= Self.maxFrameLength else {
                Diagnostics.log("ipc frame too large (\(payload.count)) — dropped")
                return
            }
            var frame = Data(count: 4)
            withUnsafeMutableBytes(of: &frame) { raw in
                raw.storeBytes(of: UInt32(payload.count).littleEndian, as: UInt32.self)
            }
            frame.append(payload)
            let ok = frame.withUnsafeBytes { raw in
                write(self.fd, raw.baseAddress, raw.count) == raw.count
            }
            if !ok {
                Diagnostics.log("ipc write failed — going offline")
                self.disconnect()
            }
        }
    }

    /// Đóng chủ động (app terminate).
    public func disconnect() {
        queue.async { [weak self] in
            guard let self else { return }
            self.readSource?.cancel()
            self.closeSocket()
            self.delegate?.ipcClientDidDisconnect(self)
        }
    }

    // ---------------------------------------------------------------- internals

    private func readAvailable() {
        var chunk = [UInt8](repeating: 0, count: 4096)
        let n = read(fd, &chunk, chunk.count)
        guard n > 0 else {
            disconnect()
            return
        }
        buffer.append(contentsOf: chunk[0..<n])
        while let message = Self.nextFrame(from: &buffer) {
            if let decoded = IpcMessage.decode(message) {
                delegate?.ipcClient(self, didReceive: decoded)
            } else {
                // JSON/type lạ = protocol violation → đóng (ipc.v1.md).
                Diagnostics.log("ipc protocol violation — closing")
                disconnect()
                return
            }
        }
    }

    /// Tách frame đầu khỏi buffer (u32 LE length prefix — ipc.v1.md).
    /// Trả `nil` khi chưa đủ dữ liệu hoặc frame vi phạm giới hạn.
    static func nextFrame(from buffer: inout Data) -> Data? {
        guard buffer.count >= 4 else { return nil }
        let le = buffer.withUnsafeBytes { raw in
            raw.loadUnaligned(fromByteOffset: 0, as: UInt32.self)
        }
        let frameLen = Int(le)
        guard frameLen > 0, frameLen <= maxFrameLength else { return nil }
        guard buffer.count >= 4 + frameLen else { return nil }
        let payload = buffer.subdata(in: 4..<(4 + frameLen))
        buffer.removeSubrange(0..<(4 + frameLen))
        return payload
    }

    private func closeSocket() {
        if fd >= 0 {
            close(fd)
            fd = -1
        }
        connected = false
        buffer.removeAll()
    }

    public static let clientVersion = "0.1.0"
}
