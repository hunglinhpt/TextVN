// swift-tools-version:5.9
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Package.swift — TextVN macOS Menu Bar App & Settings Panel (P2-4, PLAN §3.6, PLAN §4.1)

import PackageDescription

let package = Package(
    name: "textvn-app",
    platforms: [.macOS(.v13)],
    products: [
        .executable(name: "TextVN", targets: ["TextVNApp"]),
        .library(name: "TextVNAppLib", targets: ["TextVNAppLib"]),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "TextVNAppLib",
            dependencies: [],
            path: "Sources/TextVNAppLib"
        ),
        .executableTarget(
            name: "TextVNApp",
            dependencies: ["TextVNAppLib"],
            path: "Sources/TextVNApp"
        ),
        .testTarget(
            name: "TextVNAppTests",
            dependencies: ["TextVNAppLib"],
            path: "Tests/TextVNAppTests"
        ),
    ]
)
