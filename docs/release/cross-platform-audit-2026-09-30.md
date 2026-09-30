# Audit tiếp diễn — TextVN, 2026-09-30

Đây là bằng chứng cho **checkout hiện tại**, không thay thế `RELEASE_REPORT.json`
trong từng ZIP và không tự nâng một bản candidate thành production. Các thay đổi
chưa được xác nhận bởi CI trên commit cuối cho đến khi workflow chạy lại.

Windows release build trên host hiện tại đã tạo
`dist/TextVN-portable-0.1.0-windows-x64-20260930020615.zip` (artifact cục bộ, không commit)
(SHA-256 `8055E29000A0278F597A015EF560E58F1988C2E79B76BBE0D2C4391032F8150F`).
`RELEASE_REPORT.json` trong ZIP ghi `release-candidate`, TSF-only, unit/corpus,
format, clippy, ABI và smoke start/IPC/stop pass; cây nguồn còn dirty, chưa ký
Authenticode, chưa có smoke gõ native của **chính ZIP này**. File
`dist/SHA256SUMS-20260930020615.txt` ghi mã băm tương ứng.

Lần thử gõ native qua `test-portable.ps1` trên host Codex **không đạt bước
đăng ký TSF**: CLI trả `RegCreateKeyExW HKCU\Software\Classes\CLSID →
ERROR_ACCESS_DENIED (5)`. Phiên chạy là tài khoản sandbox
`lb\codexsandboxoffline`, khác tài khoản desktop `hungl`; đây là giới hạn
quyền của môi trường test, **không phải bằng chứng ZIP gõ được hay không gõ
được trên phiên người dùng thật**. Instance và thư mục giải nén tạm của lần
thử đã được dừng/xóa sau khi xác nhận registry không trỏ vào đó. Harness được
sửa để tự cleanup cả khi lỗi và không xóa DLL còn được registry sử dụng.

| Nền tảng / What | Where · Why | How đã xử lý | When / bằng chứng | Còn cần ai xác nhận |
|---|---|---|---|---|
| Windows — TSF, config, cài đặt | `adapters/windows-tsf`, `tray`, `cli`, `installer/windows`: replay trễ có thể nhắm sai ô, reload version có thể đi trước dữ liệu, thay đổi UI có thể không được ghi, đăng ký TIP lỗi bị bỏ qua | Ràng buộc target/focus trước replay, reset khi desync, chỉ cập nhật version và broadcast sau khi lưu/đọc thành công, trả lỗi đăng ký TSF; test gõ không đụng editor đang mở | `cargo test --workspace --exclude textvn-win-hook` pass trên Windows; clippy pass; native typing bị chặn ở HKCU ACL của sandbox | Người phát hành chạy smoke GUI Notepad/Office/Chrome dưới phiên Windows thật và kiểm chữ ký/DACL trước production |
| Linux — IBus/Fcitx5 và package | `adapters/linux-*`, `packaging/linux`: Ctrl+Shift+Space có thể toggle hai lần; portable/upgrade có thể làm mất cấu hình hoặc trạng thái daemon | Loại double-toggle, giới hạn cleanup theo manifest TextVN, snapshot/restore môi trường và trạng thái daemon, marker phục hồi khi dở dang | Bash syntax pass trên Windows; corpus Linux headless pass; không chạy được IBus/Fcitx5 native trong host Windows | CI Linux chạy package install/upgrade/uninstall và gõ thật X11/Wayland |
| macOS — IMK, IPC, app, package | `adapters/macos-*`, `scripts/*macos*`, `tools/mac`: toggle gửi sai message, snapshot không áp dụng, frame có thể ghi thiếu/SIGPIPE, engine xử lý trước self-heal, AppDB không được đóng gói, smoke có thể xóa tài liệu TextEdit | Dùng ToggleViEn/Snapshot, retry IPC, chống SIGPIPE, self-heal trước engine, copy AppDB vào bundle, giới hạn smoke test vào tài liệu mới, gate notarization và tên arch đúng | `bash -n` pass; Swift build/test và GUI chưa chạy được trên Windows; CI macOS nay build/test thêm menu bar app | CI macOS + máy Mac GUI xác nhận TextEdit/Safari, secure field, package install/uninstall, ký Developer ID/notarize |

## Giới hạn và rủi ro còn mở

- Windows `LNK4104` ở export COM của TSF vẫn xuất hiện khi link test; chưa chứng
  minh là lỗi chạy, cần kiểm `dumpbin /exports` và load TSF trên máy sạch.
- TSF replay trễ và IMK secure-field detection cần smoke native với focus đổi
  nhanh, ô mật khẩu, app Office/Chromium; corpus mô phỏng không chứng minh điều này.
- Mac FieldDetect hiện cache theo PID và AX bất đồng bộ, có khoảng trễ khi đổi ô
  trong cùng ứng dụng. Chưa thể đánh dấu pass secure input trước test thực tế.
- Không có chứng chỉ Authenticode/Developer ID hay kết quả AV/notarization trong
  checkout. Script Windows cố ý ghi `release-candidate` và chặn `-Channel
  production` khi các điều kiện production chưa được chứng minh. Ký số giúp
  xác thực nhà phát hành; không thể bảo đảm mọi AV không báo nhầm.
- `dist/` chứa ZIP lịch sử đã tạo trước audit. Không xóa/ghi đè chúng trong đợt
  dọn tạm; chỉ `.audit-target/` do audit này sinh ra là mục dọn sau khi test xong.

## Lệnh tái lập

```powershell
cargo fmt --all -- --check
cargo test --workspace --exclude textvn-win-hook
cargo clippy --workspace --all-targets --exclude textvn-win-hook -- -D warnings
powershell -NoProfile -ExecutionPolicy Bypass -File .\build-release.ps1
```

Trên Mac: `swift build` và `swift test` cho cả `macos-imk`, `macos-tap`,
`macos-app`, sau đó chạy `tools/mac/smoke-imk.sh` (tạo tài liệu tạm riêng,
không sửa các tài liệu TextEdit khác). Trên Linux: chạy `scripts/e2e-linux.sh` và `scripts/test-linux-package.sh`
với tarball mới trên session IBus/Fcitx5 thực.
