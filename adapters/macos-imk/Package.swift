// swift-tools-version:5.9
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Package.swift — TextVN IMK adapter macOS (P2-1 §2, ADR-006: IMK primary).
//
// Targets:
//  - CTextVNFFI : header C-ABI `textvn_ffi.h` (P0-2 — nguồn sự thật `ffi/include/`).
//  - CoreBridge : wrapper Swift an toàn quanh FFI + bảng keycode mac (generated).
//  - IMKLib     : logic adapter (controller, translator, field detect, IPC, marked).
//  - IMKApp     : TextVN-IM.app executable (IMKServer — main.swift mỏng).
//
// Link engine (MAC-003/RM1): `build-rust.sh` đặt `libtextvn_ffi.a` vào `<pkg>/lib/`
// trước khi `swift build`. linkerSettings đặt trên **CoreBridge** (nơi symbol
// `ime_*` được tham chiếu) để SwiftPM propagate flag xuống MỌI link point —
// gồm cả test bundle (review R1 F2: đặt trên executable làm swift test thiếu flag).
// Đường dẫn **tuyệt đối** qua #filePath — `swift test --package-path …` chạy từ
// CWD nào cũng đúng.

import PackageDescription

let packageRoot = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let libDir = packageRoot.appendingPathComponent("lib").path

let package = Package(
    name: "textvn-imk",
    platforms: [.macOS(.v13)],
    products: [
        .executable(name: "TextVN-IM", targets: ["IMKApp"]),
    ],
    targets: [
        .target(
            name: "CTextVNFFI",
            path: "Sources/CTextVNFFI",
            publicHeadersPath: "include"
        ),
        .target(
            name: "CoreBridge",
            dependencies: ["CTextVNFFI"],
            path: "Sources/CoreBridge",
            linkerSettings: [
                .unsafeFlags(
                    ["-L\(libDir)", "-ltextvn_ffi"],
                    .when(platforms: [.macOS])
                ),
            ]
        ),
        .target(
            name: "IMKLib",
            dependencies: ["CoreBridge"],
            path: "Sources/IMKLib"
        ),
        .executableTarget(
            name: "IMKApp",
            dependencies: ["IMKLib"],
            path: "Sources/IMKApp"
        ),
        .testTarget(
            name: "IMKLibTests",
            dependencies: ["IMKLib"],
            path: "Tests/IMKLibTests"
        ),
    ]
)
