# Audit tiếp diễn — TextVN, 2026-09-30

## Vòng 10 — delta từ `v0.2.0` đến `v0.2.1`

Đối chiếu HEAD `30d5273`, tag `v0.2.0`, các thay đổi Graphify còn pending và
release GitHub: `v0.2.0` đã có 7 asset (Windows portable/setup, Linux tar,
macOS zip/tar/pkg, SHA256SUMS), nhưng mô tả tự nhận candidate chưa ký số trong
khi GitHub đánh dấu latest. Đã sửa metadata thành **pre-release**; không ghi đè
binary cũ. `v0.2.1` đã được build/test lại từ tag sạch qua
[release-candidate #36719431897](https://github.com/hunglinhpt/TextVN/actions/runs/36719431897)
và có [GitHub Release](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.1)
dạng pre-release với SHA256SUMS. Xem
[báo cáo build](build-release-report.md) để phân biệt rõ CI và local ZIP dirty.

| What / Where | Why (nguyên nhân) | Who chịu ảnh hưởng | When | How xử lý / xác minh |
|---|---|---|---|---|
| F3-13 · `AppDelegate.swift` | `SMAppService.mainApp` không truyền `--autostart`, nên cờ mở dialog bị hiểu thành khởi động thủ công | Người dùng macOS 13+ bật login item | Mỗi lần login; cả khi instance trùng | Giữ app yên khi autostart đã đăng ký, `--settings` vẫn mở rõ ràng; thêm unit test, chờ CI Swift + smoke Mac GUI |
| F4-01 · `AutostartManager.swift` | Đường dẫn app ghép thẳng vào XML plist | Người đặt app trong thư mục có `&`, `<`, `>` | Khi bật fallback LaunchAgent | Escape XML và test đường dẫn đặc biệt; CI Swift |
| F4-02 · `SettingsView.swift` | Nhãn “kiểm tra chính tả” gắn nhầm `free_marking` | Người chỉnh tuỳ chọn macOS | Khi mở rộng Settings | Đổi đúng nhãn “Đặt dấu tự do” khớp schema và UI Linux/Windows |
| F4-03 · `ConfigModel.swift`, `install_macos.sh` | Default `autostart=true` nhưng installer không đăng ký login item, trái spec opt-in | Người cài Mac lần đầu | Sau cài | Default false; không tự sửa cấu hình cũ, giữ quyền lựa chọn người dùng |
| F4-04 · `build-release.ps1` | `-BuildInstaller` không có ISCC vẫn báo build hoàn tất; ZIP cùng tên có thể bị ghi đè | Người phát hành Windows | Khi build | Thiếu ISCC là lỗi cứng, tên ZIP đã tồn tại là lỗi cứng; kiểm cú pháp và CI package |

Ranh giới xác minh: Rust test trên Windows chỉ chứng nhận engine/chung; Swift
chỉ chạy trên CI macOS. Không thể khẳng định typing GUI Mac, Authenticode,
Developer ID/notarization hay AV thực tế từ host Windows. Chữ ký cần chứng chỉ
của chính chủ sở hữu; không dùng chứng chỉ giả, không né AV bằng kỹ thuật ẩn mã.

Graphify đã chạy `graphify update .` sau sửa code: 4.520 node, 8.359 cạnh,
311 community (94% extracted). Báo cáo tại
[`graphify-out/GRAPH_REPORT.md`](../../graphify-out/GRAPH_REPORT.md). Công cụ
không phân loại 267 file (đa số `.keys`/TOML/plist) và parser báo 7 header C
có thể trích thiếu; đây là **giới hạn phân tích**, không tự suy ra compile fail.
Graph được lập từ cây có pending changes, nên dòng `Built from commit` chỉ
thể hiện base Git, không đủ để chứng nhận artifact đã phát hành.

Build Windows cục bộ sau sửa (`-SkipTests`, nhưng `cargo test --workspace` chạy
riêng đã pass): `dist/TextVN-portable-0.2.1-windows-x64-20260930125208.zip`,
SHA-256 `F54B2F34681543873B9E5F384E63843F08E434CAACEAA13194F8EC71C63C949D`.
Trong ZIP, `RELEASE_REPORT.json` ghi `source_tree_clean=false`,
`status=release-candidate`, `authenticode=not-signed`, runtime IPC smoke pass.
**Không đưa ZIP local này lên GitHub**; release workflow phải tạo ZIP mới từ
tag sạch và chạy thêm typing thực tế trên runner Windows; workflow đã pass.

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
