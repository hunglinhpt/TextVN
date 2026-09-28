# env-mac — Môi trường build macOS (MAC-001)

> Trạng thái: **code-complete, chờ ghi version thật từ máy Mac** — các version
> dưới đây là yêu cầu tối thiểu theo `P2-0 §2`; điền cột "Đã kiểm" khi chạy
> `MAC-001` trên máy thật (acceptance task MAC-001).

## Yêu cầu

| Thành phần | Tối thiểu | Đã kiểm |
|---|---|---|
| macOS | 13.0 (Ventura) | ⏳ |
| Xcode CLT | 15.x (Swift 5.9) | ⏳ |
| Rust | stable ≥ 1.78 (`rust-toolchain.toml`) | ⏳ |
| Rust targets | `aarch64-apple-darwin`, `x86_64-apple-darwin` | ⏳ |
| SwiftPM | kèm CLT | ⏳ |
| `swift-format` | tuỳ chọn (lint Swift) | ⏳ |

## Lệnh kiểm nhanh

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin -p textvn-ffi   # engine build
cd adapters/macos-imk && ./build-rust.sh --lib-only && swift build  # template build pass
cd ../macos-tap && swift build                                      # tap module build
```

## Acceptance MAC-001

- [ ] `cargo build --release --target aarch64-apple-darwin -p textvn-ffi` pass
- [ ] `swift build` (adapters/macos-imk, sau `build-rust.sh --lib-only`) pass
- [ ] `swift build` (adapters/macos-tap) pass
- [ ] `swift test` cả 2 package xanh trên máy thật
- [ ] Ghi version thật của môi trường vào bảng trên
