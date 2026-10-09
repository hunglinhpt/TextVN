// SPDX-License-Identifier: GPL-3.0-or-later
//! StrategyResolver — strategy cho từng phím qua `ime_strategy_resolve` (FFI dùng
//! chung, P2-3 §5 — Swift KHÔNG viết lại thuật toán) + preset appdb đi kèm bundle.
//!
//! R2-43: bản cũ truyền `Data()` rỗng qua `withUnsafeBytes` — Foundation cho
//! `baseAddress` KHÁC nil với count 0, FFI coi con trỏ khác NULL là có appdb và
//! parse chuỗi "" → `IME_ERR_CONFIG` ở MỌI phím → adapter luôn ép BackspaceType
//! (Preedit, SelectionReplace và toggle non-preedit không bao giờ chạy). Không có
//! appdb thì PHẢI truyền NULL/0.
//!
//! `ime_strategy_resolve` parse lại JSON appdb mỗi lần gọi (~58 µs/phím với
//! `appdb.default.json` 8.6 KB) → kết quả được cache theo mọi trường context đổi
//! được giữa các phím (app, field role, enabled, secure); caps/hint cố định.
//! appdb hỏng → bỏ hẳn, resolve theo field role mặc định (fail-open, S4).
//!
//! Chỉ dùng trên main thread (IMK gọi `handle` trên main — P0-2 §3).

import CoreBridge
import CTextVNFFI
import Foundation

public final class StrategyResolver {
    struct Key: Hashable {
        let appID: String
        let role: UInt32
        let enabled: Bool
        let secure: Bool
    }

    /// Đủ cho vài trăm cặp app × field; vượt thì xoá hết (resolve lại rẻ khi cache nguội).
    static let cacheLimit = 512

    private var appdb: Data
    private let caps: UInt32
    private var cache: [Key: Int64] = [:]

    /// `appdb == nil` hoặc rỗng = không preset (strategy theo field role).
    public init(appdb: Data?, caps: UInt32) {
        self.appdb = appdb ?? Data()
        self.caps = caps
    }

    /// Số mục đang cache (test).
    var cacheCount: Int { cache.count }

    /// appdb còn được dùng không (false sau khi FFI từ chối nó).
    var hasAppdb: Bool { !appdb.isEmpty }

    /// Preset appdb build-macos.sh chép vào `TextVN-IM.app/Contents/Resources`.
    public static func loadBundledAppdb(bundle: Bundle = .main) -> Data? {
        guard let url = bundle.url(forResource: "appdb.default", withExtension: "json") else {
            return nil
        }
        return try? Data(contentsOf: url)
    }

    /// Strategy (`IME_STRATEGY_*`) cho context hiện tại; `-1` khi FFI lỗi — engine
    /// tự resolve (fail-open). Cache hit không gọi FFI.
    public func strategy(appID: String, role: UInt32, enabled: Bool, secure: Bool) -> Int64 {
        let key = Key(appID: appID, role: role, enabled: enabled, secure: secure)
        if let hit = cache[key] {
            return hit
        }
        var result = Self.resolve(
            appdb: appdb, appID: appID, role: role, enabled: enabled, secure: secure, caps: caps
        )
        if result.rc != FFI.ok && !appdb.isEmpty {
            Diagnostics.log("appdb rejected rc=\(result.rc) — strategy theo field role")
            appdb = Data()
            cache.removeAll()
            result = Self.resolve(
                appdb: appdb, appID: appID, role: role, enabled: enabled, secure: secure,
                caps: caps
            )
        }
        let value: Int64
        if result.rc == FFI.ok {
            value = result.strategy
        } else {
            Diagnostics.log("strategy_resolve rc=\(result.rc) — dùng hint=-1")
            value = -1
        }
        if cache.count >= Self.cacheLimit {
            cache.removeAll(keepingCapacity: true)
        }
        cache[key] = value
        return value
    }

    /// Một lần gọi FFI, không cache. appdb rỗng → NULL/0 (R2-43).
    public static func resolve(
        appdb: Data, appID: String, role: UInt32, enabled: Bool, secure: Bool, caps: UInt32
    ) -> (rc: Int32, strategy: Int64) {
        var ctx = ime_context_v1()
        ctx.abi_version = FFI.abiVersion
        ctx.enabled = enabled ? 1 : 0
        ctx.secure = secure ? 1 : 0
        ctx.field_role = role
        ctx.caps = caps
        ctx.hint = -1
        var out = Int64(IME_STRATEGY_PASSTHROUGH)
        let appIdC = Array(appID.utf8CString)
        // Closure nhiều lệnh phải `return` tường minh — thiếu thì rc là `()` (MAC-032).
        let rc: Int32 = appIdC.withUnsafeBufferPointer { buf -> Int32 in
            ctx.app_id = buf.baseAddress
            if appdb.isEmpty {
                return ime_strategy_resolve(&ctx, nil, 0, &out)
            }
            return appdb.withUnsafeBytes { raw -> Int32 in
                ime_strategy_resolve(
                    &ctx,
                    raw.baseAddress.map { $0.assumingMemoryBound(to: UInt8.self) },
                    raw.count, &out
                )
            }
        }
        return (rc, out)
    }
}
