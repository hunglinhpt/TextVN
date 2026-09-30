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
//
// NOTE (MAC-031/MAC-032): PackageDescription trên Xcode 26.6+ (Swift 6) chạy trong
// môi trường restricted — Foundation KHÔNG được import nên `URL`, `NSString` và cả
// `String.components(separatedBy:)` (API Foundation, CI 36520642917 báo "no member
// 'components'") đều không khả dụng. Chỉ dùng stdlib: `lastIndex(of:)` + slice.

import PackageDescription

// Tính absolute path tới thư mục lib/ của package từ #filePath (= Package.swift).
// Chỉ Swift stdlib — hoạt động trong PackageDescription scope.
let libDir: String = {
    let filePath = "\(#filePath)"  // StaticString → String
    // Bỏ component cuối (tên file Package.swift): cắt tại dấu "/" cuối cùng.
    guard let slash = filePath.lastIndex(of: "/") else { return "lib" }
    return String(filePath[..<slash]) + "/lib"
}()

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
            dependencies: ["IMKLib", "CoreBridge"],
            path: "Sources/IMKApp"
        ),
        .testTarget(
            name: "IMKLibTests",
            dependencies: ["IMKLib"],
            path: "Tests/IMKLibTests"
        ),
    ]
)
