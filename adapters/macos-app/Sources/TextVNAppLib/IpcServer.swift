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
    /// `app_id` toàn cục — cùng quy ước với IMK (`IpcMessage.globalAppID`) và
    /// Windows TSF (`GLOBAL_KEY`) (review R3 F3-1).
    public static let globalAppID = "*"
    /// Version gửi trong `Hello`/`Snapshot` — đọc từ Info.plist của TextVN.app,
    /// fallback hằng khi chạy trong swift test (R2 finding 2).
    public static let serverVersion: String =
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
        ?? "0.2.18"

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
        parseFramesChecked(from: &buffer).messages
    }

    /// Tách frame + báo protocol violation (length sai, JSON không phải object,
    /// thiếu `type`). `violation == true` → caller PHẢI đóng kết nối, mirror
    /// `IpcClient` (ipc.v1.md; review R3 F3-3). Các message hợp lệ đứng trước
    /// frame lỗi vẫn được trả về theo thứ tự.
    public static func parseFramesChecked(from buffer: inout Data)
        -> (messages: [[String: Any]], violation: Bool)
    {
        var messages: [[String: Any]] = []

        while buffer.count >= 4 {
            let length = buffer.prefix(4).withUnsafeBytes { $0.loadUnaligned(fromByteOffset: 0, as: UInt32.self) }.littleEndian
            let totalLength = 4 + Int(length)

            if length == 0 || length > UInt32(maxFrameLength) {
                buffer.removeAll()
                return (messages, true)
            }

            guard buffer.count >= totalLength else { break }
            let payload = buffer.subdata(in: 4..<totalLength)
            buffer.removeSubrange(0..<totalLength)

            guard let json = (try? JSONSerialization.jsonObject(with: payload)) as? [String: Any],
                  json["type"] is String else {
                buffer.removeAll()
                return (messages, true)
            }
            messages.append(json)
        }

        return (messages, false)
    }

    // MARK: - Server Lifecycle

    public func start() throws {
        try queue.sync {
            guard !isRunning else { return }

            let dir = socketURL.deletingLastPathComponent()
            try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true, attributes: [
                .posixPermissions: 0o700
            ])
            // `attributes` chỉ áp khi TẠO MỚI — dir có sẵn với quyền rộng hơn cũng
            // phải về 0700 (review R3 F3-17).
            chmod(dir.path, 0o700)
            let pathBytes = Array(socketURL.path.utf8)
            guard pathBytes.count < MemoryLayout.size(ofValue: sockaddr_un().sun_path) else {
                throw NSError(domain: "TextVN", code: -1,
                              userInfo: [NSLocalizedDescriptionKey: "Socket path too long"])
            }

            // A second app instance must not unlink the live server's socket.
            // Never remove a regular file or symlink at this path either.
            if FileManager.default.fileExists(atPath: socketURL.path) {
                var info = stat()
                guard lstat(socketURL.path, &info) == 0,
                      (info.st_mode & mode_t(S_IFMT)) == mode_t(S_IFSOCK) else {
                    throw NSError(domain: "TextVN", code: -2,
                                  userInfo: [NSLocalizedDescriptionKey: "IPC path is not a socket"])
                }
                let probe = socket(AF_UNIX, SOCK_STREAM, 0)
                guard probe >= 0 else {
                    throw NSError(domain: NSPOSIXErrorDomain, code: Int(errno),
                                  userInfo: [NSLocalizedDescriptionKey: "Cannot inspect IPC socket"])
                }
                do {
                    var probeAddr = sockaddr_un()
                    probeAddr.sun_family = sa_family_t(AF_UNIX)
                    withUnsafeMutableBytes(of: &probeAddr.sun_path) { dst in
                        dst.copyBytes(from: pathBytes)
                    }
                    let connected = withUnsafePointer(to: &probeAddr) { ptr in
                        ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sa in
                            connect(probe, sa, socklen_t(MemoryLayout<sockaddr_un>.size)) == 0
                        }
                    }
                    let probeError = errno
                    close(probe)
                    if connected {
                        throw NSError(domain: "TextVN", code: -3,
                                      userInfo: [NSLocalizedDescriptionKey: "IPC server already running"])
                    }
                    guard probeError == ECONNREFUSED || probeError == ENOENT else {
                        throw NSError(domain: NSPOSIXErrorDomain, code: Int(probeError),
                                      userInfo: [NSLocalizedDescriptionKey: "Cannot verify stale IPC socket"])
                    }
                }
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
            withUnsafeMutableBytes(of: &addr.sun_path) { dst in
                dst.copyBytes(from: pathBytes)
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

            // Transport auth (ipc.v1.md, P2-4 §2): chỉ nhận peer cùng uid. Socket
            // 0600 đã chặn cross-user; getpeereid là lớp thứ hai (review R3 F3-9).
            var peerUID: uid_t = 0
            var peerGID: gid_t = 0
            guard getpeereid(clientFd, &peerUID, &peerGID) == 0, peerUID == getuid() else {
                NSLog("[TextVN] IPC: từ chối peer khác uid user hiện tại")
                close(clientFd)
                continue
            }

            var noSigPipe: Int32 = 1
            guard setsockopt(clientFd, SOL_SOCKET, SO_NOSIGPIPE, &noSigPipe,
                             socklen_t(MemoryLayout<Int32>.size)) == 0 else {
                close(clientFd)
                continue
            }

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
                let (msgs, violation) = Self.parseFramesChecked(from: &clientBuffer)
                clientBuffers[cfd] = clientBuffer
                for msg in msgs {
                    // Message trước có thể đã làm đóng client (violation/short write).
                    guard clientSources[cfd] != nil else { return }
                    handleClientMessage(msg, from: cfd)
                }
                if violation, clientSources[cfd] != nil {
                    protocolViolation(cfd, "frame")
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

    /// Chuẩn hoá `app_id` toàn cục: IMK bản cũ (còn chạy tới khi logout sau khi
    /// nâng cấp) gửi `""` — quy về `"*"` để chỉ còn MỘT quy ước (review R3 F3-1).
    static func normalizedAppID(_ appID: String) -> String {
        appID.isEmpty ? globalAppID : appID
    }

    /// Trạng thái per-app đã biết (menu bar đọc để hiển thị check — F3-8).
    public func appState(for appID: String) -> Bool? {
        queue.sync { appStates[Self.normalizedAppID(appID)] }
    }

    /// Ảnh chụp toàn bộ state per-app (test + chẩn đoán).
    public func appStatesSnapshot() -> [String: Bool] {
        queue.sync { appStates }
    }

    /// Payload `Snapshot` đúng bảng v1 đóng — không field thừa (review R3 F3-12).
    static func snapshotMessage(configVersion: UInt64, state: [String: Bool]) -> [String: Any] {
        [
            "type": "Snapshot",
            "config_version": configVersion,
            "state": state.filter { !$0.key.isEmpty },
            "appdb_version": "1.0",
            "channel": "stable",
        ]
    }

    /// ipc.v1.md: violation → đóng kết nối, không retry (review R3 F3-3).
    private func protocolViolation(_ cfd: Int32, _ reason: String) {
        NSLog("[TextVN] IPC protocol violation (%@) — đóng kết nối", reason)
        closeClient(cfd)
    }

    private func handleClientMessage(_ msg: [String: Any], from cfd: Int32) {
        guard let type = msg["type"] as? String else {
            protocolViolation(cfd, "thiếu type")
            return
        }

        switch type {
        case "Hello":
            sendFrame(["type": "Ack"], to: cfd)
        case "GetSnapshot":
            sendFrame(Self.snapshotMessage(configVersion: configVersion, state: appStates), to: cfd)
        case "Subscribe":
            subscribers.insert(cfd)
            sendFrame(["type": "Ack"], to: cfd)
        case "ToggleViEn":
            guard let rawAppID = msg["app_id"] as? String, let enabled = msg["enabled"] as? Bool else {
                protocolViolation(cfd, "ToggleViEn sai field")
                return
            }
            let appID = Self.normalizedAppID(rawAppID)
            appStates[appID] = enabled
            DispatchQueue.main.async { [weak self] in
                guard let self = self else { return }
                self.delegate?.ipcServer(self, didToggleViEn: appID, enabled: enabled)
            }
            broadcastStateUpdate(appID: appID, enabled: enabled, version: configVersion)
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
            // Danh sách v1 đóng: type lạ = violation (review R3 F3-3).
            protocolViolation(cfd, "type lạ \(type)")
        }
    }

    private func sendFrame(_ json: [String: Any], to cfd: Int32) {
        guard let frame = Self.encodeFrame(json) else { return }
        let written = frame.withUnsafeBytes { ptr -> Int in
            guard let base = ptr.baseAddress else { return -1 }
            return write(cfd, base, ptr.count)
        }
        // A short write leaves a truncated frame. Drop this client instead of
        // sending another frame on a corrupted stream; it will reconnect.
        if written != frame.count {
            closeClient(cfd)
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
            for subFd in Array(self.subscribers) {
                self.sendFrame(msg, to: subFd)
            }
        }
    }

    public func broadcastStateUpdate(appID: String, enabled: Bool, version: UInt64) {
        queue.async { [weak self] in
            guard let self = self else { return }
            let key = Self.normalizedAppID(appID)
            self.appStates[key] = enabled
            let msg: [String: Any] = [
                "type": "StateUpdate",
                "app_id": key,
                "enabled": enabled,
                "version": version
            ]
            for subFd in Array(self.subscribers) {
                self.sendFrame(msg, to: subFd)
            }
        }
    }
}
