// SPDX-License-Identifier: GPL-3.0-or-later
// ConfigWatcher.swift — Hot-reload config.json (MAC-053, P2-4 §2; review R3 F3-5)
//
// Sửa config.json ngoài UI (sync tool, script, sửa tay lúc app đóng rồi mở lại…)
// phải tới được IMK < 1s. Theo dõi 2 nguồn:
//   • thư mục chứa file (`.write`) — bắt lưu kiểu atomic (ghi file tạm rồi rename),
//     vì fd cũ khi đó vẫn trỏ inode đã bị thay;
//   • chính file (`.write/.extend/.delete/.rename`) — bắt ghi đè tại chỗ.
// Debounce 300ms rồi gọi `onChange` trên main queue; sau mỗi lần fire mở lại fd
// của file (inode có thể đã đổi). Việc đọc/kiểm schema là của `ConfigStore`:
// file sai schema → giữ bản đang dùng.

import Foundation

public final class ConfigWatcher {
    public static let debounce: TimeInterval = 0.3

    private let fileURL: URL
    private let onChange: () -> Void
    private let queue = DispatchQueue(label: "vn.textvn.config.watcher", qos: .utility)
    private var dirSource: DispatchSourceFileSystemObject?
    private var fileSource: DispatchSourceFileSystemObject?
    private var pending: DispatchWorkItem?

    /// `onChange` luôn chạy trên **main queue**.
    public init(fileURL: URL, onChange: @escaping () -> Void) {
        self.fileURL = fileURL
        self.onChange = onChange
    }

    deinit {
        // Không `queue.sync` ở đây: ref cuối có thể rơi ngay trên `queue` (work item
        // giữ self tạm thời) → sync lên chính queue đang chạy = deadlock.
        pending?.cancel()
        dirSource?.cancel()
        fileSource?.cancel()
    }

    public func start() {
        queue.sync {
            guard dirSource == nil else { return }
            let dir = fileURL.deletingLastPathComponent()
            try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true, attributes: [
                .posixPermissions: 0o700
            ])
            dirSource = makeSource(path: dir.path, mask: [.write])
            armFile()
        }
    }

    public func stop() {
        queue.sync {
            pending?.cancel()
            pending = nil
            dirSource?.cancel()
            dirSource = nil
            fileSource?.cancel()
            fileSource = nil
        }
    }

    // Chạy trên `queue`.
    private func armFile() {
        fileSource?.cancel()
        // Chưa có file → nil; watcher thư mục sẽ bắt lúc file được tạo.
        fileSource = makeSource(path: fileURL.path, mask: [.write, .extend, .delete, .rename])
    }

    // Chạy trên `queue`.
    private func makeSource(path: String, mask: DispatchSource.FileSystemEvent)
        -> DispatchSourceFileSystemObject?
    {
        let fd = open(path, O_EVTONLY)
        guard fd >= 0 else { return nil }
        let source = DispatchSource.makeFileSystemObjectSource(fileDescriptor: fd, eventMask: mask, queue: queue)
        source.setEventHandler { [weak self] in self?.schedule() }
        source.setCancelHandler { close(fd) }
        source.resume()
        return source
    }

    // Chạy trên `queue`.
    private func schedule() {
        pending?.cancel()
        let item = DispatchWorkItem { [weak self] in
            guard let self = self, self.dirSource != nil else { return }
            self.armFile()
            let callback = self.onChange
            DispatchQueue.main.async(execute: callback)
        }
        pending = item
        queue.asyncAfter(deadline: .now() + Self.debounce, execute: item)
    }
}
