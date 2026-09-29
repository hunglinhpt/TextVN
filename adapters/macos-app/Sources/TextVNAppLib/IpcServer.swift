// SPDX-License-Identifier: GPL-3.0-or-later
// IpcServer.swift — Unix domain socket IPC server for TextVN macOS (P2-4 §2, schemas/ipc.v1.md)

import Foundation

public protocol IpcServerDelegate: AnyObject {
    func ipcServer(_ server: IpcServer, didToggleViEn appID: String, enabled: Bool)
    func ipcServer(_ server: IpcServer, didReceiveCrashReport code: UInt32, count: UInt32)
}

public final class IpcServer {
    public static let shared = IpcServer()
    public static let maxFrameLength: Int = 65_536
    public static let serverVersion = "0.1.0"

    public weak var delegate: IpcServerDelegate?

    public private(set) var isRunning: Bool = false
    public private(set) var connectedClientsCount: Int = 0

    private let socketURL: URL
    private var listenFd: Int32 = -1
    private let queue = DispatchQueue(label: "vn.textvn.ipc.server", qos: .userInitiated)
    private var listenSource: DispatchSourceRead?
    private var clientSources: [Int32: DispatchSourceRead] = [:]
    private var clientBuffers: [Int32: Data] = [:]
    private var subscribers: Set<Int32> = []

    private let startTime: Date = Date()
    private var configVersion: UInt64 = 1
    private var appStates: [String: Bool] = [:]

    public init(socketURL: URL? = nil) {
        if let custom = socketURL {
            self.socketURL = custom
        } else {
            let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
            let textvnDir = appSupport.appendingPathComponent("TextVN", isDirectory: true)
            self.socketURL = textvnDir.appendingPathComponent("ipc.sock")
        }
    }

    public static func defaultSocketURL() -> URL {
        let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        return appSupport.appendingPathComponent("TextVN", isDirectory: true).appendingPathComponent("ipc.sock")
    }

    // MARK: - Frame Encoding & Decoding Helpers

    public static func encodeFrame(_ json: [String: Any]) -> Data? {
        guard let data = try? JSONSerialization.data(withJSONObject: json) else { return nil }
        guard data.count <= maxFrameLength else { return nil }

        var frame = Data()
        var len = UInt32(data.count).littleEndian
        frame.append(Data(bytes: &len, count: MemoryLayout<UInt32>.size))
        frame.append(data)
        return frame
    }

    public static func parseFrames(from buffer: inout Data) -> [[String: Any]] {
        var messages: [[String: Any]] = []

        while buffer.count >= 4 {
            let length = buffer.prefix(4).withUnsafeBytes { $0.loadUnaligned(fromByteOffset: 0, as: UInt32.self) }.littleEndian
            let totalLength = 4 + Int(length)

            if length == 0 || length > UInt32(maxFrameLength) {
                // Protocol violation: clear buffer
                buffer.removeAll()
                break
            }

            if buffer.count >= totalLength {
                let payload = buffer.subdata(in: 4..<totalLength)
                buffer.removeSubrange(0..<totalLength)

                if let json = (try? JSONSerialization.jsonObject(with: payload)) as? [String: Any] {
                    messages.append(json)
                }
            } else {
                break
            }
        }

        return messages
    }

    // MARK: - Server Lifecycle

    public func start() throws {
        try queue.sync {
            guard !isRunning else { return }

            let dir = socketURL.deletingLastPathComponent()
            try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true, attributes: [
                .posixPermissions: 0o700
            ])

            // Remove existing stale socket if present
            if FileManager.default.fileExists(atPath: socketURL.path) {
                try? FileManager.default.removeItem(at: socketURL)
            }

            let fd = socket(AF_UNIX, SOCK_STREAM, 0)
            guard fd >= 0 else {
                throw NSError(domain: NSPOSIXErrorDomain, code: Int(errno), userInfo: [NSLocalizedDescriptionKey: "Failed to create socket"])
            }

            // Set non-blocking
            var flags = fcntl(fd, F_GETFL, 0)
            _ = fcntl(fd, F_SETFL, flags | O_NONBLOCK)

            var addr = sockaddr_un()
            addr.sun_family = sa_family_t(AF_UNIX)
            let pathBytes = Array(socketURL.path.utf8)
            guard pathBytes.count < MemoryLayout.size(ofValue: addr.sun_path) else {
                close(fd)
                throw NSError(domain: "TextVN", code: -1, userInfo: [NSLocalizedDescriptionKey: "Socket path too long"])
            }
            withUnsafeMutableBytes(of: &addr.sun_path) { dst in
                dst.baseAddress?.copyBytes(from: pathBytes)
            }

            let len = socklen_t(MemoryLayout<sockaddr_un>.size)
            let bindResult = withUnsafePointer(to: &addr) { ptr in
                ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sa in
                    bind(fd, sa, len)
                }
            }

            guard bindResult == 0 else {
                close(fd)
                throw NSError(domain: NSPOSIXErrorDomain, code: Int(errno), userInfo: [NSLocalizedDescriptionKey: "Failed to bind socket"])
            }

            chmod(socketURL.path, 0o600)

            guard listen(fd, 16) == 0 else {
                close(fd)
                throw NSError(domain: NSPOSIXErrorDomain, code: Int(errno), userInfo: [NSLocalizedDescriptionKey: "Failed to listen on socket"])
            }

            listenFd = fd
            isRunning = true

            let source = DispatchSource.makeReadSource(fileDescriptor: fd, queue: queue)
            source.setEventHandler { [weak self] in
                self?.acceptIncomingConnections()
            }
            source.setCancelHandler {
                close(fd)
            }
            source.resume()
            listenSource = source
        }
    }

    public func stop() {
        queue.sync {
            guard isRunning else { return }
            isRunning = false

            listenSource?.cancel()
            listenSource = nil
            listenFd = -1

            for (cfd, source) in clientSources {
                source.cancel()
                close(cfd)
            }
            clientSources.removeAll()
            clientBuffers.removeAll()
            subscribers.removeAll()
            connectedClientsCount = 0

            try? FileManager.default.removeItem(at: socketURL)
        }
    }

    // MARK: - Connection Handling

    private func acceptIncomingConnections() {
        while true {
            var clientAddr = sockaddr_un()
            var clientLen = socklen_t(MemoryLayout<sockaddr_un>.size)
            let clientFd = withUnsafeMutablePointer(to: &clientAddr) { ptr in
                ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sa in
                    accept(listenFd, sa, &clientLen)
                }
            }

            guard clientFd >= 0 else { break }

            var flags = fcntl(clientFd, F_GETFL, 0)
            _ = fcntl(clientFd, F_SETFL, flags | O_NONBLOCK)

            let clientSource = DispatchSource.makeReadSource(fileDescriptor: clientFd, queue: queue)
            clientBuffers[clientFd] = Data()

            clientSource.setEventHandler { [weak self] in
                self?.readClientData(clientFd)
            }
            clientSource.setCancelHandler { [weak self] in
                close(clientFd)
                self?.clientBuffers.removeValue(forKey: clientFd)
                self?.subscribers.remove(clientFd)
                self?.connectedClientsCount = self?.clientSources.count ?? 0
            }

            clientSources[clientFd] = clientSource
            connectedClientsCount = clientSources.count
            clientSource.resume()
        }
    }

    private func readClientData(_ cfd: Int32) {
        var buf = [UInt8](repeating: 0, count: 4096)
        let bytesRead = read(cfd, &buf, buf.count)

        if bytesRead > 0 {
            clientBuffers[cfd]?.append(buf, count: bytesRead)
            if var clientBuffer = clientBuffers[cfd] {
                let msgs = Self.parseFrames(from: &clientBuffer)
                clientBuffers[cfd] = clientBuffer
                for msg in msgs {
                    handleClientMessage(msg, from: cfd)
                }
            }
        } else if bytesRead == 0 || (bytesRead < 0 && errno != EAGAIN && errno != EWOULDBLOCK) {
            closeClient(cfd)
        }
    }

    private func closeClient(_ cfd: Int32) {
        if let source = clientSources.removeValue(forKey: cfd) {
            source.cancel()
        }
        clientBuffers.removeValue(forKey: cfd)
        subscribers.remove(cfd)
        connectedClientsCount = clientSources.count
    }

    // MARK: - Protocol Message Handling

    private func handleClientMessage(_ msg: [String: Any], from cfd: Int32) {
        guard let type = msg["type"] as? String else { return }

        switch type {
        case "Hello":
            sendFrame(["type": "Ack"], to: cfd)
        case "GetSnapshot":
            let uptime = UInt64(Date().timeIntervalSince(startTime) * 1000)
            let snapshot: [String: Any] = [
                "type": "Snapshot",
                "config_version": configVersion,
                "state": appStates,
                "appdb_version": "1.0",
                "channel": "stable",
                "uptime_ms": uptime
            ]
            sendFrame(snapshot, to: cfd)
        case "Subscribe":
            subscribers.insert(cfd)
            sendFrame(["type": "Ack"], to: cfd)
        case "ToggleViEn":
            if let appID = msg["app_id"] as? String, let enabled = msg["enabled"] as? Bool {
                appStates[appID] = enabled
                DispatchQueue.main.async { [weak self] in
                    guard let self = self else { return }
                    self.delegate?.ipcServer(self, didToggleViEn: appID, enabled: enabled)
                }
                broadcastStateUpdate(appID: appID, enabled: enabled, version: configVersion)
            }
        case "Ping":
            let uptime = UInt64(Date().timeIntervalSince(startTime) * 1000)
            sendFrame(["type": "Pong", "uptime_ms": uptime], to: cfd)
        case "CrashReport":
            let code = (msg["code"] as? UInt32) ?? 0
            let count = (msg["count"] as? UInt32) ?? 0
            DispatchQueue.main.async { [weak self] in
                guard let self = self else { return }
                self.delegate?.ipcServer(self, didReceiveCrashReport: code, count: count)
            }
            sendFrame(["type": "Ack"], to: cfd)
        default:
            break
        }
    }

    private func sendFrame(_ json: [String: Any], to cfd: Int32) {
        guard let frame = Self.encodeFrame(json) else { return }
        frame.withUnsafeBytes { ptr in
            _ = write(cfd, ptr.baseAddress, frame.count)
        }
    }

    // MARK: - Outgoing Broadcasts

    public func broadcastConfigReload(version: UInt64) {
        queue.async { [weak self] in
            guard let self = self else { return }
            self.configVersion = version
            let msg: [String: Any] = [
                "type": "ConfigReload",
                "version": version
            ]
            for subFd in self.subscribers {
                self.sendFrame(msg, to: subFd)
            }
        }
    }

    public func broadcastStateUpdate(appID: String, enabled: Bool, version: UInt64) {
        queue.async { [weak self] in
            guard let self = self else { return }
            self.appStates[appID] = enabled
            let msg: [String: Any] = [
                "type": "StateUpdate",
                "app_id": appID,
                "enabled": enabled,
                "version": version
            ]
            for subFd in self.subscribers {
                self.sendFrame(msg, to: subFd)
            }
        }
    }
}
