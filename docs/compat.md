# `docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)

> **Lớp test L7 = thủ công**, khác L1–L6: cần máy thật + app thật + người test.
> File này là **bảng ghi kết quả**, không phải tài liệu mô tả. Mỗi lần release candidate điền 1 dòng.
>
> Nguồn danh sách app: `P1-5 §3` (Windows) · `P2-5 §3` (macOS) · `P3-6 §3` (Linux).
> Ngưỡng pass: **12 app CI ≥ 95% case**, đủ **20 app ≥ 95%** (mỗi OS) — release gate `P1-5 §6`.
>
> Quy tắc điền: ✅ = pass ≥95% · ⚠ = 80–95% (phải ghi lý do) · ❌ = <80% hoặc crash/hang
> · ⬜ = chưa chạy (**không** được để ⬜ ở release candidate).

## 0. Cách chạy 1 app (không bỏ qua bước này)

| Bước | Thao tác | Kỳ vọng |
|---|---|---|
| 1 | Bật TextVN, mở app, gõ `dduocj` | `được` (Telex mặc định) |
| 2 | Gõ tiếng Anh: `text` + Space | giữ nguyên `text` (B5 auto-restore) |
| 3 | Ô có autocomplete (address bar / search / cell) | không xoá, không nhân đôi text |
| 4 | Enter giữa văn bản (chat) | gửi tin nhắn đúng, không mất dấu |
| 5 | Ctrl+Z / ⌘Z sau khi gõ tiếng Việt | undo đúng 1 bước |
| 6 | Ô mật khẩu (nếu app có) | **không** hiển thị/ghi ký tự gõ (S2/S3) |
| 7 | Chuyển app qua lại (Alt+Tab) | gõ tiếp vẫn đúng, không mất dấu (B2) |
| 8 | Bật/tắt TextVN giữa lúc gõ | không kẹt phím, không crash |

> Bước 3 và 7 là hai chỗ **B1/B2** hỏng nhiều nhất — đừng bỏ qua dù app "chạy được".
> Mỗi lần fail: ghi **chuỗi phím gõ lại được** (Handbook §9) + tạo issue theo mẫu.

## 1. Windows (20 app — `P1-5 §3`)

| # | App | Phường cần kiểm | Bug | CI | Kết quả | Ngày | Người test |
|---|---|---|---|---|---|---|---|
| 1 | Notepad (Win11) | body, undo, tắt IME | smoke | ✓ | ⬜ | | |
| 2 | WordPad/Word 2019 | body + suggestions | B2 | ✓ | ⬜ | | |
| 3 | VS Code | editor, multi-cursor | B7 | ✓ | ⬜ | | |
| 4 | Chrome | address bar, search, web form, contenteditable | B1 | ✓ | ⬜ | | |
| 5 | Edge | address bar (Chromium khác UA) | B1 | ✓ | ⬜ | | |
| 6 | Firefox | address bar (Gecko) | B1 | ✓ | ⬜ | | |
| 7 | Excel | cell autocomplete, formula bar | B1 | ✓ | ⬜ | | |
| 8 | Windows Terminal | terminal role | B8 | ✓ | ⬜ | | |
| 9 | Explorer | address + search | B1 | ✓ | ⬜ | | |
| 10 | Slack (Electron) | body + Enter | B2, B7 | ✓ | ⬜ | | |
| 11 | Discord (Electron) | body + emoji popup | B7 | ✓ | ⬜ | | |
| 12 | JetBrains IDE | editor + completion popup | B3 | ✓ | ⬜ | | |
| 13 | Outlook/Thunderbird | compose body | B2 | nightly | ⬜ | | |
| 14 | Zalo Desktop | chat input | B2 | nightly | ⬜ | | |
| 15 | LibreOffice Writer | body (không phải MS) | B4 | nightly | ⬜ | | |
| 16 | Figma/Canva | web canvas text | — | nightly | ⬜ | | |
| 17 | KeePassXC | secure → PASSTHROUGH | S3 | nightly | ⬜ | | |
| 18 | RDP (mstsc) | unknown role | — | nightly | ⬜ | | |
| 19 | PowerShell 7 + conhost | owner=hook, ForwardAsCommit | B8 | nightly | ⬜ | | |

## 2. macOS (20 app — `P2-5 §3`)

| # | App | Phường cần kiểm | Bug | CI | Kết quả | Ngày | Người test |
|---|---|---|---|---|---|---|---|
| 1 | TextEdit | body, undo | smoke | ✓ | ⬜ | | |
| 2 | Safari | address bar + web form | B1 | ✓ | ⬜ | | |
| 3 | Chrome | address bar + web + contenteditable | B1 | ✓ | ⬜ | | |
| 4 | Firefox | address bar (Gecko) | B1 | ✓ | ⬜ | | |
| 5 | Spotlight (⌘Space) | search | B1 | ✓ | ⬜ | | |
| 6 | Notes | textarea + marked | B11 | ✓ | ⬜ | | |
| 7 | Terminal | terminal role | B8 | ✓ | ⬜ | | |
| 8 | VS Code | editor, multi-cursor | B7/B11 | ✓ | ⬜ | | |
| 9 | Xcode | editor + completion | B3 | ✓ | ⬜ | | |
| 10 | Slack | body + Enter | B2 | ✓ | ⬜ | | |
| 11 | Discord (Electron) | body | B7 | ✓ | ⬜ | | |
| 12 | Finder | rename + Go to folder | B1 | ✓ | ⬜ | | |
| 13 | iTerm2 | terminal + paste | B8/B11 | nightly | ⬜ | | |
| 14 | Messages | chat input + Enter | B2 | nightly | ⬜ | | |
| 15 | Mail | compose | B2 | nightly | ⬜ | | |
| 16 | Microsoft Word | body + suggestions | B2 | nightly | ⬜ | | |
| 17 | Microsoft Excel | cell autocomplete | B1 | nightly | ⬜ | | |
| 18 | System Settings | search field | B1 | nightly | ⬜ | | |
| 19 | IntelliJ IDEA | editor + completion | B3 | nightly | ⬜ | | |
| 20 | 1 game (tap opt-in) | owner=tap, ngoài chat passthrough | RM8 | manual | ⬜ | | |

## 3. Linux (20 app — `P3-6 §3`)

| # | App | Phường cần kiểm | Engine | Bug | CI | Kết quả | Ngày | Người test |
|---|---|---|---|---|---|---|---|---|
| 1 | gedit / GNOME Text Editor | body, undo | ibus | smoke | ✓ | ⬜ | | |
| 2 | Firefox | address bar + web form | ibus | B1 | ✓ | ⬜ | | |
| 3 | Chrome/Chromium | address bar + contenteditable | ibus | B1 | ✓ | ⬜ | | |
| 4 | GNOME Settings | search field | ibus | B1 | ✓ | ⬜ | | |
| 5 | Nautilus | rename + location bar | ibus | B1 | ✓ | ⬜ | | |
| 6 | GNOME Terminal | terminal role | ibus | B8 | ✓ | ⬜ | | |
| 7 | Text Editor (2 window xen kẽ) | multi-context | ibus | B13 | ✓ | ⬜ | | |
| 8 | VS Code | editor, multi-cursor | ibus | B7/B11 | ✓ | ⬜ | | |
| 9 | Slack | body + Enter | ibus | B2 | ✓ | ⬜ | | |
| 10 | Discord (Electron) | body | ibus | B7 | ✓ | ⬜ | | |
| 11 | LibreOffice Writer | body + D-Bus 6 key | ibus | B2 | ✓ | ⬜ | | |
| 12 | LibreOffice Calc | cell | ibus | B1 | ✓ | ⬜ | | |
| 13 | Konsole | terminal (KDE) | fcitx5 | B8 | nightly | ⬜ | | |
| 14 | Kate | body | fcitx5 | — | nightly | ⬜ | | |
| 15 | KDE System Settings | search | fcitx5 | B1 | nightly | ⬜ | | |
| 16 | Thunderbird | compose | ibus | B2 | nightly | ⬜ | | |
| 17 | Telegram Desktop | body + Enter | ibus | B2 | nightly | ⬜ | | |
| 18 | IntelliJ IDEA | editor + completion | ibus | B3 | nightly | ⬜ | | |
| 19 | Obsidian (Electron) | body | ibus | B7 | nightly | ⬜ | | |
| 20 | 1 game X11 (blocklist) | owner=x11 opt-in | x11 | B10/RM8 | manual | ⬜ | | |

## 4. Ghi chú bắt buộc khi điền bảng

| Mục | Quy tắc |
|---|---|
| Chuỗi phím lỗi | Ghi **chính xác** chuỗi đã gõ (vd `dduocj` → sai thành `dduoc`), không mô tả chung chung — Handbook §9 |
| OS + phiên bản app | Bắt buộc (vd `Windows 11 23H2 · Chrome 128.0.6613`) |
| Có bật TextVN không | Ghi rõ; lỗi khi **tắt** IME cũng là bug (fail-open) |
| Nhiều máy | Ghi cả mac + Windows + Linux nếu app chạy đa nền tảng |
| Bug mới | Tạo issue theo mẫu + **corpus `.keys`** tái hiện được (`P0-4 §2`) — không có corpus thì chưa fix |
| Ảnh/video | Chỉ khi lỗi hiển thị (preedit, nhân đôi ký tự) |

## 5. Tóm tắt release candidate

| OS | Số app đã chạy | ≥95% | 80–95% | <80% | Ngày | Ký nhận |
|---|---|---|---|---|---|---|
| Windows | 0/20 | 0 | 0 | 0 | | |
| macOS | 0/20 | 0 | 0 | 0 | | |
| Linux | 0/20 | 0 | 0 | 0 | | |

> **Chưa đạt cổng release** (`P1-5 §6`): cần ≥12 app CI và ≥20 app tổng ≥95% mỗi OS.
> Bảng trên **không** tự đánh dấu ✅ — việc đó là của người test, không tự động hoá được.

| 20 | 1 game có chat (blocklist) | ngoài chat passthrough | B9 | manual | ⬜ | | |
