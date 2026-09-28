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
// Link engine: `build-rust.sh` (cargo 2 arch + lipo) đặt `libtextvn_ffi.a`
// vào `lib/` trước khi `swift build` (MAC-003, RM1).

import PackageDescription

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
            path: "Sources/CoreBridge"
        ),
        .target(
            name: "IMKLib",
            dependencies: ["CoreBridge"],
            path: "Sources/IMKLib"
        ),
        .executableTarget(
            name: "IMKApp",
            dependencies: ["IMKLib"],
            path: "Sources/IMKApp",
            linkerSettings: [
                .unsafeFlags(
                    ["-L", "lib", "-ltextvn_ffi"],
                    .when(platforms: [.macOS])
                ),
            ]
        ),
        .testTarget(
            name: "IMKLibTests",
            dependencies: ["IMKLib"],
            path: "Tests/IMKLibTests"
        ),
    ]
)
