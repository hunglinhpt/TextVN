# env-mac — Môi trường build macOS (MAC-001)

> Trạng thái (2026-10-09): **✅ đã kiểm trên CI `macos-latest`** (`ci-macos` + `release.yml`,
> từ 2026-10-07); chủ repo báo phát triển macOS xong 2026-10-08. Các version dưới đây
> là yêu cầu tối thiểu theo `P2-0 §2`; version cụ thể của máy dev chưa được ghi lại.

## Yêu cầu

| Thành phần | Tối thiểu | Đã kiểm |
|---|---|---|
| macOS | 13.0 (Ventura) | ✅ CI `macos-latest` |
| Xcode CLT | 15.x (Swift 5.9) | ✅ CI `macos-latest` |
| Rust | stable ≥ 1.78 (`rust-toolchain.toml`) | ✅ CI `macos-latest` |
| Rust targets | `aarch64-apple-darwin`, `x86_64-apple-darwin` | ✅ CI `macos-latest` |
| SwiftPM | kèm CLT | ✅ CI `macos-latest` |
| `swift-format` | tuỳ chọn (lint Swift) | không dùng (CI lint bằng `bash -n`/`plutil`) |

## Lệnh kiểm nhanh

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin -p textvn-ffi   # engine build
cd adapters/macos-imk && ./build-rust.sh --lib-only && swift build  # template build pass
cd ../macos-tap && swift build                                      # tap module build
cd ../macos-app && swift build                                      # menu bar app + Settings
```

## Acceptance MAC-001

- [x] `cargo build --release --target aarch64-apple-darwin -p textvn-ffi` pass
- [x] `swift build` (adapters/macos-imk, sau `build-rust.sh --lib-only`) pass
- [x] `swift build` (adapters/macos-tap) pass
- [x] `swift test` cả **3** package (`macos-imk`, `macos-tap`, `macos-app`) xanh — CI `ci-macos`
      (vd. run 37569093894), arm64
- [ ] Ghi version thật của môi trường vào bảng trên (máy dev — chưa ghi)
