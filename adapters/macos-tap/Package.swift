// swift-tools-version:5.9
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Package.swift — TextVNTap: CGEventTap fallback opt-in (P2-2, ADR-006).
//
// **Opt-in tuyệt đối**: chỉ chạy cho app có `engine_owner: "tap"` trong
// preset/override (P2-2 §1); IMK vẫn là đường chính (P2-1).
// Tách module riêng: crash tap không kéo theo IMK (P2-1 §12).
//
// Package KHÔNG link engine Rust — callback nhận quyết định qua protocol
// `TapKeyHandler` (IMK inject implementation dùng ime_instance B trên thread
// tap — P0-2 §3: 1 instance = 1 thread). Điều này giữ TextVNTap thuần logic
// + CG API, test được không cần engine.

import PackageDescription

let package = Package(
    name: "textvn-tap",
    platforms: [.macOS(.v13)],
    products: [
        .library(name: "TextVNTap", targets: ["TextVNTap"]),
    ],
    targets: [
        .target(name: "TextVNTap", path: "Sources/TextVNTap"),
        .testTarget(name: "TextVNTapTests", dependencies: ["TextVNTap"], path: "Tests/TextVNTapTests"),
    ]
)
