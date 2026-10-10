# Đóng góp cho TextVN

Cảm ơn bạn đã quan tâm đến TextVN! Mọi đóng góp đều được hoan nghênh — từ báo cáo lỗi, đề xuất tính năng, cải thiện tài liệu, đến code.

---

## Quy tắc ứng xử

Dự án này tuân thủ [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/vi/version/2/1/code_of_conduct/). Khi tham gia, bạn đồng ý tuân thủ các quy tắc này.

---

## Cách báo cáo lỗi (Bug Report)

1. **Kiểm tra Issues** — Xem lỗi đã được báo cáo chưa
2. **Chạy `textvn-cli doctor`** và đính kèm output (hoặc file `.zip` từ `--export`)
3. **Mô tả chi tiết**:
   - Hệ điều hành và phiên bản (Windows: Settings → About; macOS: About This Mac; Linux: distro + IBus/Fcitx5)
   - Phiên bản TextVN (Bảng điều khiển → **Thông tin**; Windows: `version.json` trong file `--export`)
   - Các bước tái hiện lỗi
   - Hành vi mong đợi vs thực tế
4. **Không đính kèm text/nội dung bạn đã gõ** — Chúng tôi nghiêm túc bảo vệ quyền riêng tư

---

## Đề xuất tính năng (Feature Request)

- Mở Issue với label `enhancement`
- Mô tả use case cụ thể (không chỉ tính năng trừu tượng)
- Nếu có thể, mô tả cách các bộ gõ khác (UniKey, EVKey) giải quyết vấn đề tương tự

---

## Quy trình phát triển

### 1. Fork & Clone

```bash
git clone https://github.com/YOUR_USERNAME/TextVN.git
cd TextVN
```

### 2. Tạo branch

```bash
git checkout -b feat/ten-tinh-nang
# hoặc
git checkout -b fix/ten-bug
```

Quy ước đặt tên:
- `feat/`: tính năng mới
- `fix/`: sửa lỗi
- `docs/`: tài liệu
- `chore/`: bảo trì (CI, toolchain, dependencies)
- `test/`: tests

### 3. Phát triển

```powershell
# Build
cargo build --workspace

# Test TRƯỚC KHI commit
cargo test --workspace
cargo clippy --workspace -- -D warnings

# REUSE compliance
reuse lint
```

### 4. Commit message

Tuân thủ **Conventional Commits**:

```
<type>(<scope>): <mô tả ngắn> (<ticket>)

[body tùy chọn]

[footer: BREAKING CHANGE, Refs, Fixes]
```

Ví dụ:
```
feat(windows-tray): add per-app disable via context menu (WIN-052)
fix(windows-tsf): prevent heap corruption in AddLanguageProfile (WIN-016)
docs(readme): add build instructions for Windows
```

**Không commit:**
- Mã nguồn chứa thông tin nhạy cảm (key, password, PII)
- Nhật ký gõ phím của người dùng (rule S2)
- File binary lớn (> 5MB) — dùng Git LFS hoặc Releases

### 5. Pull Request

- Tạo PR vào branch `main`
- Mô tả ngắn gọn những gì đã thay đổi và lý do
- Đảm bảo tất cả CI checks xanh
- Reviewer sẽ response trong vòng 7 ngày làm việc

### 6. Phát hành version mới

Bắt buộc theo [docs/release/release-process.md](docs/release/release-process.md)
(quy tắc G16 trong `docs/00-WORKFLOW.md`): Checklist **A** — gates local (fmt,
clippy, test, version-sync, replay, verify) trước khi commit; Checklist **B** —
CI xanh rồi mới tag, và bản phát hành phải đủ CHANGELOG + link compare, README,
metainfo, build-release-report, common-errors (nếu có lỗi mới).

---

## Tiêu chuẩn code

### Rust

- Tuân thủ `rustfmt` (chạy `cargo fmt`)
- Không có `clippy` warnings (`cargo clippy -- -D warnings`)
- `unsafe` code phải có comment `// SAFETY:` giải thích invariant
- Log/debug **không được chứa text người dùng** (rule S2)

### Windows API

- Ưu tiên per-user registry (HKCU) — không đòi admin khi không cần (rule S5)
- Xử lý lỗi COM qua `Result<T, Error>`, không `panic!`
- Named Pipe và Win32 resource phải có cleanup khi drop

### Tests

- Mỗi module có `#[cfg(test)] mod tests { ... }`
- Unit test cho logic core (engine, strategy, config parsing)
- Integration test cho CLI commands
- Không test chức năng cần UI hoặc registry thật trong CI

---

## Cấu trúc dự án & ownership

| Crate / thư mục | Mô tả | Windows | macOS | Linux |
|---|---|---|---|---|
| `core/` (`textvn-core`) | Engine Telex/VNI/VIQR, đặt dấu, bảng mã, gõ tắt | ✅ | ✅ | ✅ |
| `ffi/` (`textvn-ffi`) | C ABI cho adapter (`ffi/include/textvn_ffi.h`) | ✅ | ✅ | ✅ |
| `strategy/` · `appdb/` · `field-detect/` | Chọn cách chèn chữ theo app/ô nhập | ✅ | ✅ | ✅ |
| `config/` (`textvn-config`) | Schema + parser cấu hình `config.v1` | ✅ | ✅ | ✅ |
| `cli/` (`textvn-cli`) | register/doctor/replay/verify | ✅ | replay/verify | replay/verify (cài thành `textvn`) |
| `ipc/` (`textvn-ipc`) | Codec IPC (named pipe); macOS/Linux hiện thực cùng giao thức `schemas/ipc.v1.md` bằng Swift/C | ✅ | — | — |
| `tray/` (`textvn-tray` → `TextVN.exe`) | Khay + bảng điều khiển Win32 | ✅ | — | — |
| `adapters/windows-tsf`, `windows-hook` | TSF TIP · hook (chỉ gói Compatibility) | ✅ | — | — |
| `adapters/macos-imk`, `macos-app` | Bộ gõ IMK + app menu bar/Cài đặt (Swift) | — | ✅ | — |
| `adapters/macos-tap` | CGEventTap opt-in — có test, **chưa** có trong bản phát hành | — | 🔄 | — |
| `adapters/linux-ibus`, `linux-fcitx5`, `linux-common`, `linux-settings` | IBus/Fcitx5 + bảng điều khiển GTK4 (C/C++, CMake) | — | — | ✅ |

---

## License

Khi đóng góp, bạn đồng ý rằng code của bạn được cấp phép theo [GPL-3.0-or-later](LICENSE).

Mọi file mới phải có SPDX header:
```rust
// SPDX-License-Identifier: GPL-3.0-or-later
```

---

## Hỏi & Hỗ trợ

- **Bugs & Features**: Mở Issue trên GitHub
- **Thảo luận**: GitHub Discussions
- **Bảo mật**: Xem [SECURITY.md](SECURITY.md) để báo cáo lỗ hổng bảo mật
