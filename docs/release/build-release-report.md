# Báo cáo dựng & kiểm thử — TextVN

## Bản 0.2.48 — phím tắt trợ lý chỉ Ctrl+Space (R2-136, chủ repo chốt)

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | **R2-136** (`docs/50-assist/P6-TEXT-PREDICTION.md` §2b): bỏ Alt+Space của 0.2.47 trên Windows/Linux — đụng menu cửa sổ + PowerToys Run/Copilot/ChatGPT; đo trên máy chủ repo: Ctrl+Alt+Space bị Claude desktop giữ, Alt+/ bị TSF chặn như Alt+Space, Ctrl+Space/Ctrl+. tới thẳng key sink. Chủ repo chọn chỉ Ctrl+Space. Windows gỡ preserved key Alt+Space + đường trả menu hệ thống; Linux `lc_is_assist_hotkey` chỉ Ctrl+Space; macOS giữ Option+Space; Alt+1..3 giữ |
| Đo | win-tsf 97 · clippy -D warnings 0 · preflight 23/23 · C `test_compose` (Alt+Space = 0) trên ci-linux · **E2E thật** `tools/win/e2e-assist-apps.ps1` **44/44** (Alt+Space khi câu có lỗi vẫn mở menu cửa sổ, chữ không đổi — 4 app) + `tools/win/e2e-assist-p6.ps1` **28/28** |
| CI commit code | `55510fc1` 6/6 xanh (ci-linux C `test_compose` + IBus/Fcitx5 e2e, ci-windows, ci-shared, repo-hygiene, ci-deb, ci-aur — không đụng macOS) |
| Phát hành | ⏳ điền sau tag (B6–B8) |

## Bản 0.2.47 — phím tắt sửa hết + dự đoán, Alt+1..3 Windows, Notepad++, chữ mờ gợi ý (phản hồi chủ repo)

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | Phản hồi chủ repo sau 0.2.46 (`docs/50-assist/P6-TEXT-PREDICTION.md` §2b): **R2-131** kế hoạch sửa chủ động dùng chung 3 OS (`explicit_fix_text`, FFI `ime_assist_plan_sentence_fix_ex`) — nút "Sửa tất cả"/chọn gợi ý sửa được lỗi chính tả; phím tắt trợ lý sửa hết TRƯỚC rồi mới dự đoán, engine nạp câu đã sửa (`ime_reset_with_tail`) · **R2-132** app IMM32/CUAS (Notepad++) sửa qua `IMR_DOCUMENTFEED` + `VK_BACK`/`WM_CHAR`, chờ nhả phím bổ trợ · **R2-133** Alt+Space / Alt+1..3 qua TSF preserved key, phím rảnh trả lại app (`SC_KEYMENU` / `WM_SYSKEYDOWN`); macOS Option+Space, Linux Ctrl/Alt+Space · **R2-134** chữ mờ gợi ý (`Engine::peek_completion`, FFI `ime_peek_completion`, Windows `ghost.rs`; macOS ô bảng ứng viên, Linux dòng phụ) · **R2-135** VS Code/Electron: phím tắt khi đang gõ dở từ hẹn sửa 120 ms (Monaco vẽ lại textarea bất đồng bộ), Alt+Space rảnh ở cửa sổ không `WS_SYSMENU` tự mở menu hệ thống · Ctrl+Shift+G mở popup chọn-sửa |
| Đo | `cargo test` win-tsf 97 · ffi 40 · core (text_prediction 18) · cli · clippy -D warnings 0 · C `test_compose` (`test_assist_hotkey_explicit_preview`) + Swift (Option+Space, nhánh explicit) trên CI · preflight **23/23** · **E2E thật**: `tools/win/e2e-assist-apps.ps1` **44/44** (Notepad 11, Edge 11, Notepad++ 11, VS Code/Electron 11) + `tools/win/e2e-assist-p6.ps1` **28/28** (Notepad/Word/Excel) — trước sửa 12/21 |
| CI commit code | `a82a4902` 7/7 · `db1c0f3b` 7/7 · `5e8092d9` 7/7 (ci-linux C/IBus/Fcitx5 e2e + ci-macos Swift xanh cho phần parity) · `7f9d5034` xanh |
| CI trên commit gắn tag | `d0777f69` (Verified) 7/7 xanh — ci-shared [40274611854](https://github.com/hunglinhpt/TextVN/actions/runs/40274611854), ci-windows [40274611748](https://github.com/hunglinhpt/TextVN/actions/runs/40274611748), ci-linux [40274611824](https://github.com/hunglinhpt/TextVN/actions/runs/40274611824), ci-macos [40274611749](https://github.com/hunglinhpt/TextVN/actions/runs/40274611749), ci-deb, ci-aur, repo-hygiene |
| Ghi chú giới hạn | Windows chưa nối bộ kiểm tra chính tả của hệ điều hành (D39) ⇒ lỗi chính tả từ điển ("nghành") chưa được báo trên Windows · macOS/Linux xem trước dự đoán trong bảng ứng viên/dòng phụ (không chữ mờ tại chỗ — marked text/preedit có thể bị app tự chốt) · phần mềm giữ Alt+Space toàn cục (PowerToys Run, Copilot, ChatGPT) nhận phím trước TextVN |
| Release | v0.2.47 tag ký GPG (Good signature, object `d0777f69`) — release-candidate [#40275600400](https://github.com/hunglinhpt/TextVN/actions/runs/40275600400) 4/4 job (windows/macos/linux/publish); **34 asset**, pre-release |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`); hash machine exe/MSIX/gpg key khớp file tải về; chữ ký `.asc` artifact hợp lệ (Sigstore do job publish tự verify) |
| Checksums | machine exe `b6c26fbb…551d` · MSIX `630383b1…7203` · mac zip `cfd78946…a8a5` · Linux `7ccd30be…aaeb` · PKGBUILD archive `c42c2b0f…baa` |
| Homebrew (B7b/B7d) | cask `version "0.2.47"` + `sha256` = mac zip `cfd78946…a8a5` — sync-cask [#40277140935](https://github.com/hunglinhpt/homebrew-tap/actions/runs/40277140935) success, raw cask trên tap khớp |
| AUR (B7e) | PKGBUILD/.SRCINFO = archive v0.2.47 `c42c2b0f…baa` — `publish-aur.sh` cần khoá SSH AUR (bước của chủ repo) |
| PPA (B7f) | debian/changelog `0.2.47-0ppa1` — `prepare-ppa.sh --upload` cần DEBSIGN key (bước của chủ repo) |
| approved (B7c) | `approved/v0.2.47/` — 34 file byte-identical (SHA256SUMS raw hash trùng khớp), raw URL 200 (machine exe, MSIX, SHA256SUMS) · Store: `STORE_AUTO_PUBLISH` chưa đặt → nộp MSIX qua `store-publish` / Partner Center là bước của chủ repo |

## Bản 0.2.46 — dự đoán văn bản P6 + trợ lý ngữ pháp sống theo câu (audit sâu + E2E thật)

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | **P6 dự đoán văn bản** (`docs/50-assist/P6-TEXT-PREDICTION.md`, opt-in `suggest.enabled`): Tab/Ctrl+Space hoàn tất + cụm nối tiếp, duyệt vòng; audit R2-120 (ngữ cảnh nhiều từ trước tiền tố, dự đoán ngay sau từ, chữ hoa/tên riêng/tiêu ngữ, xuống dòng, kiểu dấu mới) · R2-121 Ctrl+Space nhả từ 3 OS · R2-122 Tab ở overlay · R2-123 replay CUAS · **trợ lý ngữ pháp**: R2-125 panel Windows luôn neo được (Notepad Win11 trước đây KHÔNG BAO GIỜ hiện) · R2-126 bỏ toast "phát hiện lỗi" (cả tray) · R2-127 panel to + vẽ đúng font tiếng Việt · R2-128 gõ tắt/số tiền vượt composition (TF_E_RANGE_NOT_COVERED) · R2-129 **gợi ý sống theo câu** Windows/macOS/Linux, không chiếm phím gõ, Linux Alt+1..3 · **bộ cài**: R2-124 gỡ bản per-user khi cài bản máy · R2-130 nâng cấp tại chỗ dọn sạch bản cũ + gỡ bản máy bằng UAC · hygiene E31 (CRLF → `.gitattributes`) |
| Đo | **745 test workspace** (41 binary; core text_prediction 16, win-tsf 97, tray 78) · clippy -D warnings 0 · preflight 23/23 · **E2E trên ứng dụng thật 28/28** (`tools/win/e2e-assist-p6.ps1`: Notepad Win11 19, Word 6, Excel 3) · test mới FAIL trên mã cũ 9/16 (chứng minh bắt đúng lỗi) |
| CI commit code | `2d57bda6` 7/7 · `7a79220d` 7/7 (ci-windows log `PASS R2-124 …`) · parity `97e7b187` 7/7 (ci-linux e2e IBus thật: `PASS assist live list shows at word boundary (R2-129)`, `PASS live list: digit is typed`; ci-macos swift test arm64 0 failures) |
| CI trên commit gắn tag | `c5c2dea8` (Verified) 7/7 xanh — ci-shared [40247843442](https://github.com/hunglinhpt/TextVN/actions/runs/40247843442), ci-windows [40247843410](https://github.com/hunglinhpt/TextVN/actions/runs/40247843410), ci-linux [40247843453](https://github.com/hunglinhpt/TextVN/actions/runs/40247843453), ci-macos [40247843323](https://github.com/hunglinhpt/TextVN/actions/runs/40247843323), ci-deb, ci-aur, repo-hygiene |
| Ghi chú giới hạn | Ghost text (xem trước ứng viên mờ) chưa có · Linux chưa có trợ lý on-demand qua Ctrl+Space · macOS: Ctrl+Space mặc định là phím tắt đổi nguồn nhập của hệ thống · Word AutoCorrect (văn bản đặt ngôn ngữ tiếng Anh) tự đổi "hàng"→"hang" — không phải TextVN |
| Release | v0.2.46 tag ký GPG (Good signature, object `c5c2dea8`) — release-candidate [#40248402360](https://github.com/hunglinhpt/TextVN/actions/runs/40248402360) 4/4 job (windows/macos/linux/publish); **34 asset**, pre-release |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`); 9/9 hash khớp file tải về; 8/8 chữ ký `.asc` từng artifact hợp lệ (Sigstore do job publish tự verify — máy local không có cosign) |
| Checksums | machine exe `d447055c…2b9c` · MSIX `f18c7e9d…b641` · mac zip `ce336103…64bd` · Linux `f5c5006d…0503` · PKGBUILD archive `f2a61f6f…f46b` |
| Homebrew (B7b/B7d) | cask `version "0.2.46"` + `sha256` = mac zip `ce336103…64bd` — sync-cask [#40249870824](https://github.com/hunglinhpt/homebrew-tap/actions/runs/40249870824) success, raw cask trên tap khớp |
| AUR (B7e) | PKGBUILD/.SRCINFO = archive v0.2.46 `f2a61f6f…f46b` — `publish-aur.sh` cần khoá SSH AUR (bước của chủ repo) |
| PPA (B7f) | debian/changelog `0.2.46-0ppa1` — `prepare-ppa.sh --upload` cần DEBSIGN key (bước của chủ repo) |
| approved (B7c) | `approved/v0.2.46/` — 34 file byte-identical (SHA256SUMS raw hash trùng khớp `c6bd6a21…5175`), raw URL 200 (machine exe, MSIX, SHA256SUMS) · Store: `STORE_AUTO_PUBLISH` chưa đặt → nộp MSIX qua `store-publish` / Partner Center là bước của chủ repo |

## Bản 0.2.45 — gói năng suất văn phòng P5 (OP-001…OP-007, mỗi task 2 lượt review)

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | **P5** (`docs/50-assist/P5-OFFICE-PRODUCTIVITY-PLAN.md` → trạng thái IMPLEMENTED + bảng delta): OP-001 gõ tắt thích ứng hoa/thường (giữ expand có chữ hoa chủ đích); OP-002 guard công thức Excel (`=sum/if/vlookup` đầu ô; KHÔNG chặn sau chữ số — VNI); OP-003 số tiền `=…d/vnd/bc` + Tab (đọc chuẩn kế toán, fail-open MAX_TEXT); OP-004 8 bigram + 3 trigram hành chính (guard "kiểm sát nhập" + "xoay sở"); OP-005 `100 %` → `100%` (bỏ điều kiện punct.len==1 vì tokenizer gộp run); OP-006 `Word.identifier`/`auto_capped` + TECH_BUSINESS_TERMS (camel giữ raw trọn từ, không tham gia should_restore); OP-007 core `detect/transcode/Charset/TextCase/decode_tcvn3/decode_vni/strip_diacritics` + `TranscodeSession` Windows + chord Ctrl+Shift+F9 |
| Đo | **718 test workspace** (core 160 lib + 15 office, charset 11, win-tsf 95) · clippy -D warnings 0 · preflight 23/23 · VI corpus 0/981 FP recall 83,7% (không đổi) · EN 0/295 |
| CI trên commit gắn tag | 7/7 xanh (ci-shared, ci-linux, ci-windows, ci-macos, ci-deb, ci-aur, repo-hygiene) — 2 lỗi đã bắt và sửa trong phiên: `is_multiple_of` chưa ổn định trên rustc Debian trixie (đổi `% 3` + allow MSRV có lý do) và literal SPDX trong bảng plan làm reuse parse hỏng |
| Ghi chú giới hạn | OP-007 v1: VIQR/TelexRaw pass-through (có tài liệu); chord chuyển mã mới nối **Windows** — macOS/Linux dùng core sẵn, cần UI/chord riêng ở vòng sau |
| Release | v0.2.45 tag ký GPG — release-candidate [#40178841488](https://github.com/hunglinhpt/TextVN/actions/runs/40178841488) 4/4 job (windows/macos/linux/publish); **34 asset** |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`); 9/9 hash khớp file tải về |
| Checksums | machine exe `10a9761c…3a04` · MSIX `1221b617…f646` · mac zip `7e1dc111…3ad3` · Linux `733f7310…d461` · PKGBUILD archive `f4e135cd…f76a` |
| Homebrew (B7b) | cask `version "0.2.45"` + `sha256` = mac zip `7e1dc111…3ad3` — sync-cask [#40179919575](https://github.com/hunglinhpt/homebrew-tap/actions/runs/40179919575) success |
| AUR (B7e) | PKGBUILD/.SRCINFO = archive v0.2.45 `f4e135cd…f76a` |
| PPA (B7f) | debian/changelog `0.2.45-0ppa1` |
| approved (B7c) | `approved/v0.2.45/` — 34 file byte-identical (SHA256SUMS raw hash trùng khớp), raw URL 200 |

## Bản 0.2.44 — audit sâu 3 lượt trợ lý ngữ pháp (khoá FP cuối)

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | 3 vòng audit dòng-từng-dòng (pass-1 core+wiring 2 agent song song: 1 CRITICAL + 2 HIGH + 4 MEDIUM + 3 LOW core / 2 MAJOR + 8 MINOR + 1 INFO wiring; pass-2 đối kháng trên diff; pass-3 spot-check + docs). Khá nổi: **L9 sentence_repeat TẮT** (bêu văn phạm đúng "cho anh. Anh xem" — recall 0 row), **1.061 âm tiết Việt ASCII** nhận diện tiếng Việt thiếu dấu (a/an hết đụng "an toan"), a/an exception theo GỐC (hour\*/honor\*/heir\*/honest\*/honour\* + /juː/: unicode/euphemism/ukulele), số gộp **11xx/18xx**, 31 extension file mask, ngoặc cong “ ”, ĐH./TP., double-space trước `#`, Fcitx5 fresh-install ON, R2-28 global-AND-override đồng bộ 3 OS, gate <3 từ parity macOS/Linux, DPI panel assist (GetDpiForWindow + WM_DPICHANGED), check chỉ ở sentence-end (bỏ waste Word-trigger), E29 (FFI cut UTF-8 boundary + DOUBLE_SPACE/SPACE_AFTER_PUNCT plan) + E30 (recheck popup bấm tiếp được + FixAll đủ 8 lỗi) đăng ký `project-common-errors` |
| Đo | VI **0/981 FP** recall 83,7 % · EN **0/295 FP** recall 31/31 · 43 probe adversarial sạch · **701 test** (tăng từ 674) · clippy -D warnings 0 · preflight 23/23 · ime_key p50 4,2 µs / p99 10,2 µs (budget 500 µs) |
| CI trên commit gắn tag b7d110c4 | 7/7 xanh (ci-shared, ci-linux, ci-windows, ci-macos, ci-deb, ci-aur, repo-hygiene) |
| Release | v0.2.44 tag ký GPG — release-candidate [#40162813369](https://github.com/hunglinhpt/TextVN/actions/runs/40162813369) 4/4 job; **34 asset** |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`), 9/9 hash khớp file tải về |
| Checksums | machine exe `302c0832…d748` · setup exe `46bd16cf…698a` · MSIX `c8239148…723d` · portable `dd64c976…6c75` · Linux `e1bb9850…1e0ef` · mac zip `a59fc327…65fb` · mac tar.gz `6cbad4b7…0685` · pkg `84037296…a6c6` · PKGBUILD archive `77335ab0…4f8fb` |
| Homebrew (B7b) | cask `version "0.2.44"` + `sha256` = mac zip `a59fc327…65fb` — sync-cask [#40163734087](https://github.com/hunglinhpt/homebrew-tap/actions/runs/40163734087) success |
| AUR (B7e) | PKGBUILD/.SRCINFO = archive v0.2.44 `77335ab0…4f8fb` |
| PPA (B7f) | debian/changelog `0.2.44-0ppa1` |
| approved (B7c) | `approved/v0.2.44/` — 34 file byte-identical (SHA256SUMS raw hash trùng khớp), raw URL 200 |

## Bản 0.2.43 — Enter trên popup vùng bôi đen áp Sửa-tất-cả (audit v30 F6 — finding LOW cuối)

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | F6: nhánh Apply của `handle_overlay_key` phát hiện popup từ bôi đen (`ActiveAssist.selection.is_some()`) → `selection_fix_all_from_keyboard` = cùng đường nút chuột "Sửa tất cả" (`ApplySelectionFixSession`: SetText đúng range, precondition đọc-lại trong edit lock, `reset_engine`, undo thuộc app). Trước đây Enter bị nuốt no-op vì `apply_assist_suggestion` cần selection rỗng (A2 đơn-từ). Thêm parity v0.2.42: sau SetText `recheck_after_fix` trên câu đã sửa — sạch panel tắt, còn lỗi gợi ý popup hiện lại tại caret |
| Đo | 674 test (0 fail), clippy -D warnings 0, preflight 23/23 (full clean rebuild — rustc ICE 0xc0000409 do target hỏng, không phải code) |
| CI trên commit gắn tag 68db7b0 | 7/7 xanh: ci-shared, ci-linux, ci-windows, ci-macos, ci-deb, ci-aur, repo-hygiene |
| Release | v0.2.43 tag ký GPG — release-candidate [#40153578257](https://github.com/hunglinhpt/TextVN/actions/runs/40153578257) 4/4 job (linux/windows/macos/publish); **34 asset** |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`), 9/9 dòng hash khớp file tải về |
| Checksums | machine exe `af2c8c8f…10c8` · setup exe `5e94b94a…2c9c` · MSIX `1ad5806e…9770` · portable `86d6c597…1802` · Linux `1955e903…64bb` · mac zip `3c9a4c96…8f8e` · mac tar.gz `e1669b35…100d` · pkg `d57700e7…2626` · PKGBUILD archive `51c32410…1aaa` |
| Homebrew (B7b) | cask `version "0.2.43"` + `sha256` = mac zip `3c9a4c96…8f8e` — sync-cask [#40154963633](https://github.com/hunglinhpt/homebrew-tap/actions/runs/40154963633) success |
| AUR (B7e) | PKGBUILD/.SRCINFO = archive v0.2.43 `51c32410…1aaa` |
| PPA (B7f) | debian/changelog `0.2.43-0ppa1` |
| approved (B7c) | `approved/v0.2.43/` — 34 file byte-identical (SHA256SUMS raw hash trùng khớp), raw URL 200 |

## Bản 0.2.42 — check ngữ pháp vùng bôi đen (audit 2 lượt: 8 finding → SHIP)

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | **Selection-check** (vòng 30): Shift+←→↑↓/Home/End (không Ctrl/Alt) → debounce 500 ms → đọc vùng chọn (3..256 ký tự, cap-detection +2 slot) → check → popup neo GetTextExt vùng chọn + nút Sửa-tất-cả; bấm → `ApplySelectionFixSession` (TF_ES_READWRITE): precondition `IsEmpty`+`GetText==expect` **trong edit lock** (không TOCTOU — audit pass-1 xác nhận) → SetText đúng range → `reset_engine()`. Đọc/copy không bao giờ kích hoạt: chuột không qua key sink, Ctrl/Alt lọc, F-key bỏ (F5); hook ở CẢ OnTestKeyUp + OnKeyUp (Notepad/Chrome/VS Code bỏ pha TEST — F2 HIGH); IsWindow guard hwnd chết (F4) |
| Audit 2 lượt | Pass-1: 8 finding (F1 HIGH nút chết 257-500 vì check_text cắt đuôi 256 → MAX hạ 256; F2 HIGH hook thiếu pha UP; F3 reset_engine; F4 timer chết; F5 F-key FP; F7 const). Pass-2: **SHIP** — per-fix verdict bảng + kịch bản Notepad (scenario 2 verify bằng test `acceptance_dod_scenarios_1_to_7`), Word (CUAS/lock reasoning), Chrome address bar (mouse+Ctrl+C = NO popup), **upgrade-mix no-reboot PASS** (DLL 0.2.40 chạy trong app + tray mới: reconnect ≤10 s → Snapshot assist:true — wire ipc byte-identical qua git diff v0.2.40..HEAD) |
| Đo | Corpus giữ nguyên (engine không đổi): VI 0/981 + EN 0/295 FP; 674 test, clippy 0 |
| Release | v0.2.42 tag ký GPG — release-candidate [#40130546478](https://github.com/hunglinhpt/TextVN/actions/runs/40130546478) 4/4 job; **34 asset** |
| CI trên commit gắn tag | 7/7 xanh (3 vòng commit: feature + guard + audit fixes) |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`) |
| Checksums | machine exe `b93443ed…5977` · setup exe `85201745…d329` · MSIX `4216fdbe…9107` · portable `a4bf6959…e76f` · Linux `b726bb20…bb67` · pkg `b862cba9…0dd2a` · **mac zip `23babf71…3ae7`** · mac tar.gz `2ce6fc9f…14d8` · PKGBUILD archive `084018f8…d2ff` |
| Homebrew (B7b) | cask `sha256` = mac zip `23babf71…3ae7` |
| AUR (B7e) | PKGBUILD/.SRCINFO = archive v0.2.42 `084018f8…d2ff` |
| PPA (B7f) | debian/changelog `0.2.42-0ppa1` |
| approved (B7c) | `approved/v0.2.42/` — 34 file, byte-identical |

## Bản 0.2.41 — sửa trợ lý chết khi nâng cấp (3 root cause, audit agent xác nhận)

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | (1) `IpcState::default()` assist_enabled **false** — DLL tắt toàn bộ trợ lý khi tray chưa snapshot → default **true**; (2) state.json ≤0.2.37 giữ `assist_enabled:false` mãi kể cả 0.2.38–0.2.40 (so-version KHÔNG bắt được — audit F1 CRITICAL) → migration persisted-flag `assist_default_on` bật lại đúng 1 lần (test kịch bản thật: false-cũ → bật; tắt tay sau → giữ tắt); (3) toast phát hiện khôi phục làm **fallback popup dưới-phải 8s** khi overlay không neo được (audit F2: nhầm 60s) — `notify_detected` CAS nguyên tử + slot riêng `LAST_DETECT_MS` |
| Audit xác nhận | Agent end-to-end: gate-table 11 cổng Mode 1 + Mode 2/A2/A3 mouse; Win11 balloon OK; macOS/Linux không pattern chết tương tự; upgrade-mix old-tray bounded (installer stop-tray); VI-off = im theo thiết kế |
| Đo | VI **0/981 FP** · EN **0/295 FP** — giữ nguyên (engine crates không đổi); 673 test, clippy 0 |
| Release | v0.2.41 tag ký GPG — release-candidate [#40125231623](https://github.com/hunglinhpt/TextVN/actions/runs/40125231623) 4/4 job; **34 asset** |
| CI trên commit gắn tag | 7/7 xanh (2 vòng: clippy-linux import Ordering) |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`) |
| Checksums | machine exe `0f66b531…980b` · setup exe `e01233f4…25cb` · MSIX `3f2afbe1…7456` · portable `fb60e672…b7a1` · Linux `d28c1c51…a81a` · pkg `8f712a04…596d` · **mac zip `a8616815…61ab`** · mac tar.gz `1b480002…a02b` · PKGBUILD archive `ad16b88e…cb05` |
| Homebrew (B7b) | cask `sha256` = mac zip `a8616815…61ab` |
| AUR (B7e) | PKGBUILD/.SRCINFO = archive v0.2.41 `ad16b88e…cb05` |
| PPA (B7f) | debian/changelog `0.2.41-0ppa1` |
| approved (B7c) | `approved/v0.2.41/` — 34 file, byte-identical |

## Bản 0.2.40 — audit 3 nền tảng (2 agent chuyên gia song song, 24 finding) + panel re-check

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | **CRITICAL article_an phá chữ Việt** ("an khang"→"a khang" khi auto-fix): câu Vi-dominant chỉ mở khoá a/an khi có từ-điểm EN khác; "a"/"an" tự thân KHÔNG là tín hiệu — probe 12/12 đúng. **HIGH UNESCO-class** (/juː/: "a UNESCO" đúng, "an UNESCO" sai bị tha). **HIGH macOS IMK + Linux fresh-install default** đồng bộ highlight (trước off — người dùng không nhận được trợ lý dù UI nói bật). Linux kind 11 wired (replace trượt vòng 24 — bẫy lặp). Click panel: vùng không-tương-tác đóng panel; first-click Select; toast khay 60s mọi nền tảng. **Panel re-check sau fix** (câu đã sửa — còn lỗi hiển thị, sạch tự tắt) + `hide_on_auto_fix`. Docs: popup Windows-only, quick-switch chỉ Windows |
| Perf (probe release, engine thật) | Mọi đường IME 14–52 µs/check — 200× dưới ngân sách 10 ms (URL/TLD-heavy 32, 256-char VI/EN 31/32, mix 15) |
| Đo (CLI release rebuild trước đo) | VI **0/981 FP** recall **28/28** · EN **0/295 FP** recall **31/31** — giữ nguyên qua mọi fix |
| Release | v0.2.40 publish từ tag ký GPG — release-candidate [#40114314189](https://github.com/hunglinhpt/TextVN/actions/runs/40114314189) 4/4 job; **34 asset** |
| CI trên commit gắn tag | 7/7 xanh (2 vòng: unused-fn + IMK Swift tests); 671 test, clippy 0, fmt sạch, preflight 23/23 |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`) |
| Checksums | machine exe `7025e3a7…dbd2` · setup exe `a4a5c589…b63c` · MSIX `dec7041f…9465` · portable `74574ef2…c11d` · Linux `ff549aca…dea4` · pkg `135e4061…0b9` → `135e4061…0b9` · **mac zip `3b7923b3…4105` → `3b7923b3…105`** · mac tar.gz `05a579b9…8240` · PKGBUILD archive `3fe74013…8169` |
| Homebrew (B7b) | cask `sha256` = mac zip `3b7923b3…105` |
| AUR (B7e) | PKGBUILD/.SRCINFO = archive v0.2.40 `3fe74013…8169` |
| PPA (B7f) | debian/changelog `0.2.40-0ppa1`; ci-deb xanh |
| approved (B7c) | `approved/v0.2.40/` — 34 file, byte-identical |

## Bản 0.2.39 — panel trợ lý bấm được bằng chuột (Sửa tất cả + click từ gạch chân)

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | **Overlay NHẬN chuột** (vòng 27): `WM_NCHITTEST` HTCLIENT trên vùng tương tác (nút "Sửa tất cả" / dòng lỗi / từ gạch chân) còn lại HTTRANSPARENT; `WM_LBUTTONDOWN` → `mouse_hit_test` thuần (HitGeometry) → Select/Apply/FixAll dispatch qua `ACTION_SINK` (Weak<TsfShared>, đăng ký ở tip.rs); `should_dismiss` chỉ đóng khi click NGOÀI panel; `ActiveAssist` giữ `ctx` + `AssistCheck` — FixAll tính kế hoạch A3 rồi `request_sync_or_async`, precondition + Undo 5 giây do session tự kiểm. Auto-correct (Mode 2) giữ nguyên |
| Đo | Corpora không đổi: VI 0/981 FP, EN 0/295 FP, recall 100% |
| Release | v0.2.39 publish từ tag ký GPG — release-candidate [#40091488102](https://github.com/hunglinhpt/TextVN/actions/runs/40091488102) 4/4 job; **34 asset** |
| CI trên commit gắn tag | 7/7 xanh; local 669 test (thêm 5 case mouse_hit_test), clippy 0, preflight 23/23 |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`) |
| Checksums | machine exe `07a3c5be…8c90` · setup exe `79e9e129…0eb4` · MSIX `87283aad…9801` · portable `9bc1fcff…01d9a` → `9bc1fcff…d9a` · Linux `6b7aae8e…6b3` · pkg `3654af5e…8dc04` → `3654af5e…c04` · **mac zip `44335175…508de` → `44335175…08de`** · mac tar.gz `e4503f9d…9b684` → `e4503f9d…b684` · PKGBUILD archive `bedce41f…2b98` |
| Homebrew (B7b) | cask `sha256` = mac zip `44335175…08de` |
| AUR (B7e) | PKGBUILD/.SRCINFO = archive v0.2.39 `bedce41f…2b98` |
| PPA (B7f) | debian/changelog `0.2.39-0ppa1`; ci-deb xanh |
| approved (B7c) | `approved/v0.2.39/` — 34 file, byte-identical |

## Bản 0.2.38 — trợ lý ngữ pháp bật mặc định + panel tổng lỗi kiểu Grammarly

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | **Default ON**: `assist.mode` = highlight (config serde + schema + Swift + Windows tray state serde-default + Linux thiếu-khoá → 1); kill-switch default theo config (Windows `assist_enabled` serde-default true, Linux `lc_state_read_assist_enabled(…, 1)`, macOS ViState đã default true). **Panel Grammarly tại caret**: gạch chân đỏ đúng span (có sẵn) + dòng tổng kết mới "N lỗi trong câu vừa gõ (+M khác)" (overlay `summary_text` + `HEADER_H`). **Toast policy**: loại toast "Phát hiện N lỗi"; toast duy nhất = auto-fix vừa thay text, cooldown 2.5 s → 60 s, kèm Ctrl+Z (`notify_auto_fixed`). **Mixed vượt trội**: whitelist lặp từ theo TỪNG TỪ (ASCII → EN list), guard vocative cả-hai-viết-hoa + ngoặc kép MỌI ngôn ngữ ("Lê Lê ơi", nói "không không"), article_an chạy trong câu Việt mix |
| Đo (CLI release rebuild trước đo) | VI: **0/981 FP**, recall **28/28** · EN: **0/295 FP**, recall **31/31** · Mixed probe 10/10 đúng |
| Release | v0.2.38 publish từ tag ký GPG — release-candidate [#40087813329](https://github.com/hunglinhpt/TextVN/actions/runs/40087813329) 4/4 job; **34 asset** |
| CI trên commit gắn tag | 7/7 xanh; vòng land bắt: Swift test D20 (`isOn`), C `test_defaults_when_missing` + `test_assist` + `test_assist_partial_block` (default highlight), Linux cross-check cfg (`summary_text`/`paint_overlay`/`notify_auto_fixed`/`HEADER_H`), fmt `verify.rs` |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`) |
| Checksums | machine exe `f6a064d4…f6ac` · setup exe `2cfa8130…0b62` · MSIX `690eac64…f1f2` · portable `1d3c48e6…1305` · Linux `726141a6…2192` · pkg `912c4e3e…2aac` · **mac zip `2c8f33e3…4f763` → `2c8f33e3…b4f763`** · mac tar.gz `4c0ceb8d…294a45` · PKGBUILD archive `78f131ec…f47a1` |
| Homebrew (B7b) | cask `sha256` = mac zip `2c8f33e3…b4f763` |
| AUR (B7e) | PKGBUILD/.SRCINFO = archive v0.2.38 `78f131ec…f47a1` |
| PPA (B7f) | debian/changelog `0.2.38-0ppa1`; ci-deb xanh |
| approved (B7c) | `approved/v0.2.38/` — 34 file, byte-identical |

## Bản 0.2.37 — Trợ lý ngữ pháp tiếng Anh v1 + Space hoàn tất từ + câu mix VI/EN

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | **EN assistant v1**: routing ngôn ngữ (Vi/En/im-lặng fail-open) + rule `article_an` theo ÂM (danh sách đóng: a university/an hour/an MBA/a NATO/an 8-bit; mơ hồ historic/FAQ không bao giờ soi) — kind FFI 11 wired đủ Rust + 2 header C + C + Swift + ABI verify. `duplicate_word` EN: whitelist lặp hợp lệ (that that/had had/very very/ha ha…), guard vocative cả-hai-viết-hoa + ngoặc kép. **Câu mix văn phòng** ("vui lòng check lại file report nhé", "Anh gửi report cho team ASAP") trước đây bị im lặng — giờ được kiểm tra. **4 bug SỐNG phía VI**: chèn space vào viết tắt/tên miền (e.g./a.m./U.S./example.com), ellipsis rời, viết hoa code/danh sách đề mục. **REQ-B**: `suggest.trigger = tab_and_space` (mặc định tab) — Space hoàn tất từ + kèm dấu cách, không bao giờ viết lại từ đã đúng; completion nạp đủ 3000 từ tần suất |
| Đo (CLI release rebuild trước đo) | VI: **0/981 FP**, recall **28/28 = 100 %** · EN: **0/295 FP** (corpus Tatoeba thật, CC BY 2.0 FR — REUSE.toml + attribution đầy đủ), recall **31/31 = 100 %** (sentence + fragment) |
| Release | v0.2.37 publish từ tag ký GPG — release-candidate [#40070891727](https://github.com/hunglinhpt/TextVN/actions/runs/40070891727) 4/4 job; **34 asset** |
| CI trên commit gắn tag | 7/7 xanh (ci-shared/ci-windows/ci-linux/ci-macos/ci-aur/ci-deb/repo-hygiene); vòng trước bắt: header C macOS thiếu `IME_ASSIST_KIND_ARTICLE_AN` (2 bản textvn_ffi.h), Swift test suggest round-trip thiếu `trigger`, fmt verify.rs |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`) |
| Checksums | machine exe `91d18a71…0c26` · setup exe `f268349c…d555` · MSIX `7f333528…6204` · portable `fe6ac711…e4fd` · Linux `0cb04dcb…7935` · pkg `a8e0defc…e7f45` → `a8e0defc…7f45` · **mac zip `9b1fb0f1…123d`** · mac tar.gz `1e32fc83…f640` · PKGBUILD archive `6d3663e9…a69e` |
| Homebrew (B7b) | cask `sha256` = mac zip `9b1fb0f1…123d` |
| AUR (B7e) | PKGBUILD/.SRCINFO `sha256sums` = archive v0.2.37 `6d3663e9…a69e` |
| PPA (B7f) | debian/changelog `0.2.37-0ppa1`; ci-deb xanh; upload chờ owner |
| approved (B7c) | `approved/v0.2.37/` — 34 file, byte-identical |

## Bản 0.2.36 — trợ lý ngữ pháp FPR 0 % (audit 3 lượt "như Grammarly")

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | Audit 3 lượt từng dòng toàn bộ trợ lý ngữ pháp (assist crate + FFI + 3 adapters + config/corpus/docs). **A1 CRITICAL**: bỏ 5 cặp bảng TYPOS mà từ "sai" là từ hợp lệ (quyến/thuở/đả/áy/điếm — vi phạm Fassist-96, chứng minh 5/5 câu đúng bị autocorrect sửa sai). 4 từ lặp hợp lệ vào whitelist. A2: gate L0 nhận chữ hoa có dấu. Đo lại trung thực (`tools/assist/score_real.py`, CLI rebuild trước đo): **FPR 0,5 % → 0,0 % (0/981, Wilson 0,0–0,4)**, recall câu 87,8 % → 83,7 % (đổi 2 ca mơ hồ lấy FPR 0 — đánh đổi một chiều), diacritic 66,7 %. Chi tiết: `docs/50-assist/P4-12-RECALL-REAL.md` §11 |
| Release | v0.2.36 publish từ tag ký GPG trỏ commit `938bbd3` — release-candidate [#39983783445](https://github.com/hunglinhpt/TextVN/actions/runs/39983783445) 4/4 job; **34 asset** |
| CI trên commit gắn tag | push `main` `938bbd3`: [ci-shared #39980147717](https://github.com/hunglinhpt/TextVN/actions/runs/39980147717), [ci-windows #39980147661](https://github.com/hunglinhpt/TextVN/actions/runs/39980147661), [ci-linux #39980147800](https://github.com/hunglinhpt/TextVN/actions/runs/39980147800), [ci-macos #39980147744](https://github.com/hunglinhpt/TextVN/actions/runs/39980147744), [ci-aur #39980147646](https://github.com/hunglinhpt/TextVN/actions/runs/39980147646), [ci-deb #39980147733](https://github.com/hunglinhpt/TextVN/actions/runs/39980147733), [repo-hygiene #39980147742](https://github.com/hunglinhpt/TextVN/actions/runs/39980147742) — xanh cả 7. Local: 658 test pass, clippy 0, fmt sạch, preflight 23/23 |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`) |
| Checksums (pin từ `SHA256SUMS.txt` đã ký) | machine exe `66050e2e…8ad2` · setup exe `7bb80bb8…f98d` · MSIX (nộp Store) `fbda3c52…ba16` · portable `4ec25f51…3a43` · Linux `3ec5d51a…8bc7` · pkg `c8848521…41e9` · **mac zip `054716ef…5543`** · mac tar.gz `cfea7a86…c6a4` · PKGBUILD source archive v0.2.36 `ad1b8055…3bf5` |
| Homebrew (B7b) | cask `sha256` = mac zip `054716ef…5543` |
| AUR (B7e) | PKGBUILD/.SRCINFO `sha256sums` = archive v0.2.36 `ad1b8055…3bf5`; ci-aur xanh; publish lên AUR còn chờ owner (account + SSH key) |
| PPA (B7f) | debian/changelog `0.2.36-0ppa1`; ci-deb xanh; upload còn chờ owner (Ubuntu SSO + GPG key) |
| approved (B7c) | `approved/v0.2.36/` — 34 file release + chữ ký, byte-identical |

## Bản 0.2.35 — trợ lý ngữ pháp: recall 87,8% + audit sâu (W1–W5, R-7) + kênh Debian/PPA

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | **Trợ lý ngữ pháp (vòng D19 + audit 3 vòng):** recall corpus thật 57,1% → **87,8%** / diacritic_typo 12,5% → **75%**, FPR **0,5%** (chuẩn D19: ≥80% / ≤2%) — bảng cặp từ-sai-không-phải-từ-hợp-lệ (Fassist-96) + đường biến thể E1/E2/E3 ngân sách ≤8 từ lạ/câu (`a557d3a`). Audit sâu từng dòng: **W1** macOS guard glyph==grapheme trước khi xoá (apply+undo); **W3** undo Windows nhánh async không mất state; **W4** `assist.per_app` có hiệu lực trên Windows (gate `check_sentence` + test); **R-7** logic sửa gộp **một nguồn** `textvn_ffi::plan_issues_from_raw` cho cả 3 OS; guard FFI `glyphs == số char Unicode` fail-safe cho VNI/TCVN3 (bảo vệ IBus/Fcitx5/Linux). Chi tiết `P4-REVIEW-LOG.md` §"VÒNG AUDIT SÂU". Kênh Debian/Ubuntu (PPA `ppa:hunglinhpt/textvn` + lộ trình kho chính thức, R2-119) |
| Gate local (máy Windows dev) | `cargo xtask preflight` **23/23 PASS** (homebrew SKIP — thiếu ruby cục bộ, CI chặn thật; store-validate SKIP — không artifact dist/) · replay shared+win **134/134** · replay adapter Linux **56/56** · repo replay tổng không fail |
| Test trực tiếp các mảng đổi trong bản này | assist 60+4+8 · ffi 36+3 (thêm test `plan_issues_from_raw_loc_va_ngu_nghia_sua`) · win-tsf 89 (thêm test `per_app_tat_assist_rieng_theo_config`) — **199 pass, 0 fail, clippy 0 warning** |
| CI trên commit audit `d4d1565` | 7/7 workflow xanh (ci-aur, ci-deb, ci-shared, ci-windows, ci-linux, ci-macos, repo-hygiene) — link run bổ sung ở mục Phát hành sau tag |
| Lỗi/sự cố mới trong phiên (B4⑤) | CI đỏ `a557d3a` do `cargo fmt` bảng TYPOS mới → sửa `1e52a99` (bài học: luôn `cargo fmt --all` trước commit test mới) |
| Trạng thái | release candidate: chưa Authenticode (chờ SignPath Foundation), macOS chưa Developer ID/notarization |

## Bản 0.2.34 — kênh Linux distro (AUR + ci-aur) + fcitx5 C++20 (R2-117)

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | Gói AUR (`packaging/arch/PKGBUILD` + `.SRCINFO` build từ source tag) + QA gate `ci-aur.yml` (makepkg thật trong `archlinux:base-devel`: build cây hiện tại, `.SRCINFO` diff, đủ 10 file, version khớp Cargo.toml, upload `.pkg`). Phân tích kênh store Linux (`docs/release/linux-distribution.md` — Flathub/Snap/AppImage loại có căn cứ). Fixed R2-117: fcitx5 adapter C++17 → C++20 (header Fcitx5 6.x dùng std::ranges — tag 0.2.33 KHÔNG build được trên Arch, bắt được bởi chính ci-aur). `tools/release/publish-aur.sh` chặn push khi sha256 lệch archive tag (R2-118). Checklist thêm B7e; AGENTS.md mới |
| Release | v0.2.34 publish từ tag ký GPG trỏ commit `7e680e1` — release-candidate [#39111983240](https://github.com/hunglinhpt/TextVN/actions/runs/39111983240) 4/4 job; **34 asset** |
| CI trên commit gắn tag | push `main` `7e680e1` (Verified): [ci-aur #39111618816](https://github.com/hunglinhpt/TextVN/actions/runs/39111618816) (makepkg thật xanh), [ci-shared #39111618762](https://github.com/hunglinhpt/TextVN/actions/runs/39111618762), [ci-linux #39111618941](https://github.com/hunglinhpt/TextVN/actions/runs/39111618941) (C++20 trên Ubuntu), [ci-windows #39111618793](https://github.com/hunglinhpt/TextVN/actions/runs/39111618793), [ci-macos #39111618788](https://github.com/hunglinhpt/TextVN/actions/runs/39111618788), [repo-hygiene #39111618720](https://github.com/hunglinhpt/TextVN/actions/runs/39111618720) — xanh cả 6 |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`) |
| Checksums (pin từ `SHA256SUMS.txt` đã ký) | machine exe `6b22a476…f602` · setup exe `8076f908…ec6ea` · MSIX (nộp Store) `7868c8ed…894c` · portable `8fb6dfb0…269ad` · Linux `0b5b8ad3…1aed` · pkg `38f2f16b…5da2` · **mac zip `5609baf4…a7a7`** · mac tar.gz `73baa9f3…f23d` · PKGBUILD source archive v0.2.34 `7d148e6c…7034` |
| Homebrew (B7b) | cask `sha256` = mac zip `5609baf4…a7a7` |
| AUR (B7e) | PKGBUILD/.SRCINFO `sha256sums` = archive v0.2.34 `7d148e6c…7034`; ci-aur xanh; publish lên aur.archlinux.org còn chờ owner tạo account + SSH key (`tools/release/publish-aur.sh`) |
| approved (B7c) | `approved/v0.2.34/` — 34 file release + chữ ký, byte-identical (hash kiểm từ raw URL) |

## Bản 0.2.33 — mặc định dấu cũ sau cài + ma trận 16 combo + từ điển EN top-3000 + Tab hoàn tất từ tiếng Việt

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | Mặc định sau cài = bật gõ tiếng Việt + dấu cũ trên cả 3 nền tảng (người cũ giữ lựa chọn). Ma trận hồi quy 16 combo bảng mã × kiểu gõ (`core/tests/charset_matrix.rs`, anchor TCVN3/VNI cứng). Từ điển EN mở rộng top-3000 tần suất (en_common 31 → 104 mục, fixture `data/en_freq3000.txt`) với luật R2-56: 12 âm tiết Việt không-cách-gõ-khác thắng (`vn_common.txt`). Tab hoàn tất từ tiếng Việt (`suggest.enabled`, tắt mặc định — `core/src/post/suggest_vn.rs` + `data/vn_freq3000.txt`). Docs gạch chân mô tả đúng từng nền tảng |
| Release | v0.2.33 publish từ tag ký GPG trỏ commit `2ab0a6d` — [release-candidate #39099475633](https://github.com/hunglinhpt/TextVN/actions/runs/39099475633) 4/4 job; **34 asset**; gate chữ ký git (commit + tag Verified) PASS |
| CI trên commit gắn tag | push `main` `2ab0a6d` (Verified): [ci-shared #39099157511](https://github.com/hunglinhpt/TextVN/actions/runs/39099157511), [ci-windows #39099157553](https://github.com/hunglinhpt/TextVN/actions/runs/39099157553), [ci-linux #39099157596](https://github.com/hunglinhpt/TextVN/actions/runs/39099157596), [ci-macos #39099157540](https://github.com/hunglinhpt/TextVN/actions/runs/39099157540), [repo-hygiene #39099157445](https://github.com/hunglinhpt/TextVN/actions/runs/39099157445) — xanh cả 5. Lần đỏ đầu của vòng: `testDefaultConfig` macOS còn kỳ vọng dấu mới → sửa (commit `2d95e41`) |
| Ký số | `gpg --verify` asset → Good signature (FPR `3921595A…A822`); hash tải về khớp `SHA256SUMS.txt` clearsign; hash kiểm lại từ raw `approved/v0.2.33/` byte-identical |
| Checksums (pin từ `SHA256SUMS.txt` đã ký) | machine exe `ed9736ca…ec0f` · setup exe `0a72df47…f69a` · MSIX (nộp Store) `ee41ee1f…6162` · portable `7df8bcbd…2ede` · Linux `f40e7cc5…1721` · pkg `d4179bf2…98ed` · **mac zip `e1be0c53…8aac`** · mac tar.gz `ff5026ac…e5de` |
| Homebrew (B7b) | cask `sha256` = mac zip `e1be0c53…8aac` |
| approved (B7c) | `approved/v0.2.33/` — 34 file release + chữ ký, byte-identical với GitHub Release (hash kiểm từ raw URL) |

## Bản 0.2.32 — rà soát toàn hệ thống sau 0.2.31 (R2-101…R2-115) + CI từng nền tảng

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | 15 finding R2-101…R2-115 (`code-review-2026-10-09-round2.md` §3.8; 9 bản sửa vòng 16 được rà lại — `AUDIT-3PLATFORM-2026-10-07.md` mục F). Popup "Thông tin" thống nhất 3 nền tảng (R2-113). `ci-shared` tách `ci-windows` + `ci-linux` → README có badge Windows / Linux / macOS / engine |
| Thay đổi dở của worktree Codex (`8ec9b15`, 20 file, 2026-09-29) | Đối chiếu từng dòng: ~95 % đã có trên `main`, phần còn lại đã được làm lại tốt hơn — không port |
| Probe thật trên máy dev (Windows 11 26300) | `TextVN.exe --status` không có tray: `main` panic exit 101 → bản sửa `OFFLINE (os error 2)` exit 0 (R2-101). Tray chạy với APPDATA cô lập: `WM_QUERYENDSESSION` → TRUE, tray vẫn sống; `WM_ENDSESSION(TRUE)` → thoát (R2-102); phục vụ đồng thời `textvn-ipc-v1-s1` + `textvn-ipc-v1` (R2-109). Registry API qua ctypes: `RegOpenKeyExW(HKU, "\<sid>")` = 161, không `\` = 0; `RegDeleteTreeW` qua `<sid>\Software\Classes` và `<sid>_Classes` xoá được (R2-104). ISCC biên dịch `.iss` bản máy OK |
| Đỏ trước — xanh sau | R2-107: nhánh `test/r2-107-red` (chỉ e2e mới, code `main`) → [ci-shared #38154207224](https://github.com/hunglinhpt/TextVN/actions/runs/38154207224) job Linux đỏ `FAIL EN giữ sau focus-out/in want="as " got="á "`. R2-106/R2-113: test parity đỏ trên `SettingsView.swift`/`AppInfo.swift` cũ. R2-108: profile Fcitx5 2 nhóm — `main` thêm vào `[Groups/0]`, bản sửa vào `[Groups/1]`. R2-112: file chứa header private key chưa track → gate đỏ. R2-114: trả lại `.read().unwrap()` → gate đỏ |
| Gate local | `cargo xtask preflight` **23/23 PASS** (homebrew SKIP — thiếu ruby, CI chạy thật); `swiftc -parse` 6 file Swift đã sửa OK (typecheck thật chỉ ở `ci-macos` — E16) |
| Lỗi/sự cố mới trong phiên (B4⑤) | R2-101…R2-115; **B22**/A16 (PowerShell `$null` → `""` khi gọi P/Invoke); **E17** (rustc `0xc0000409` / os error 1453 khi build song song trên máy dev → `-j 2`); **E18** (preflight `store-validate` cài/gỡ thật artifact cũ trên máy dev → nay bỏ qua trừ khi opt-in) |
| Sự cố antivirus (ghi nhận) | Kaspersky cách ly 8 đối tượng của bản Store 1.2.29.0 lúc 23:34:32, 2 phút sau lần chạy đầu — `docs/specs/antivirus-false-positive.md` §10 (chờ tên detection, AV-1/AV-2/AV-6) |
| Trạng thái | release candidate: chưa Authenticode (chờ SignPath Foundation), macOS chưa Developer ID/notarization |

### Phát hành 0.2.32

| Hạng mục | Bằng chứng |
|---|---|
| Release | v0.2.32 publish (pre-release) từ tag ký GPG trỏ merge commit `3b1ec8c` (PR #9) — [release-candidate #38190208484](https://github.com/hunglinhpt/TextVN/actions/runs/38190208484) 4/4 job; **34 asset**; gate chữ ký git (commit + tag Verified) PASS |
| CI trên commit gắn tag | push `main` `3b1ec8c` (merge PR #9, Verified): [ci-shared #38189826405](https://github.com/hunglinhpt/TextVN/actions/runs/38189826405), [ci-windows #38189826375](https://github.com/hunglinhpt/TextVN/actions/runs/38189826375), [ci-linux #38189826369](https://github.com/hunglinhpt/TextVN/actions/runs/38189826369), [ci-macos #38189826410](https://github.com/hunglinhpt/TextVN/actions/runs/38189826410), [repo-hygiene #38189826363](https://github.com/hunglinhpt/TextVN/actions/runs/38189826363) — xanh cả 5. PR #9: 5/5 xanh (Windows: OFFLINE/2 tên pipe/session end/nâng cấp khi tray chạy PASS; Linux: e2e "EN/VN giữ sau focus-out/in" PASS) |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`); hash tải về khớp danh sách đã ký |
| Checksums (pin từ `SHA256SUMS.txt` đã ký) | machine exe `49a427a9…473d` · setup exe `fd82517c…745b` · MSIX (nộp Store) `f9831854…6602` · portable `545403b8…4aef` · Linux `5d7711ca…58fa` · pkg `9983ac23…0f75` · **mac zip `4483053f…5ea0`** · mac tar.gz `e215ca97…4be7` |
| Homebrew (B7b) | cask `sha256` = mac zip `4483053f…5ea0` |
| approved (B7c) | `approved/v0.2.32/` (EXE + `-store.msix` + chữ ký) — kèm bù `approved/v0.2.31/` (bản 0.2.31 bỏ sót bước này) |

## Bản 0.2.31 — hoàn tất roadmap audit 3 nền tảng (A10, B7/B8/B9/B11/B12/B13, C1/C3/C7/C8/C9) + gate clippy-linux

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | Đóng nốt các mục còn lại của `docs/release/AUDIT-3PLATFORM-2026-10-07.md`: Windows A10 (catch_unwind cho callback COM + borrow-fail trả E_FAIL), macOS B7/B8/B9/B11/B12/B13 (AX dedupe + timeout, IPC cap, clientPid theo sender, xoá state chết, main-queue precondition, log xoay 1 MiB), Linux C1/C3/C7/C8/C9 (xoá API chết, thống nhất poll IBus↔Fcitx5, hết UAF khi đóng cửa sổ cài đặt, Fcitx5 đọc đúng nhóm, hardening XDG/log/stage/field_detect) |
| Gate local (máy Windows 11, host repo) | `cargo fmt --all -- --check` PASS · `cargo clippy -p textvn-tray --all-targets -D warnings` PASS · `cargo clippy --workspace --exclude textvn-win-hook --all-targets --target x86_64-unknown-linux-gnu -D warnings` PASS (sau fix import gate) · `cargo check -p textvn-win-tsf` PASS · `cargo test --workspace` PASS · `cargo test -p textvn-tray` 65+3=68 PASS · `cargo run -q -p xtask -- check-version-sync` 14/14 = 0.2.31 · `check-mac-targets` PASS |
| Bằng chứng bảng băm | Digest FNV-1a 64 của `data/tables/*.toml` khớp header file sinh (`vowels 0xfe5318e15330844d`, methods `0xfc9e019b7f34691b`, keymap `0x6a6f2f15251c8911`) — `check-tables` tương đương PASS (xem ghi chú dưới) |
| Parse Swift | `swiftc -parse` trên 7 file đã sửa (Diagnostics, FieldDetect, IpcClient, TextVNInputController, AppDelegate, IpcServer, KeyTranslator) — OK |
| Kiểm thủ công C | `clang -fsyntax-only -Wall -Wextra -pedantic` trên `log.c`/`field_detect.c`/`settings_model.c`/`ipc_client.c` — không warning mới (chỉ còn cảnh báo unused-function sẵn có khi biên dịch 1 TU trên Windows) |
| Ghi chú gate không chạy được trên máy này | `cargo xtask check-tables` gọi `rustfmt` qua stdin pipe; máy này chặn tạo pipe stdin cho tiến trình con (`Os error 231 — All pipe instances are busy`) nên gate báo FATAL. Đã chứng minh tương đương PASS bằng bảng băm ở trên. Tương tự `cargo run -p textvn-cli -- replay` cần pipe. CI (ubuntu-24.04) không dính giới hạn này |
| CI trên commit cuối `11a79eb` (B3) | push `main` `11a79eb`: [`ci-shared`](https://github.com/hunglinhpt/TextVN/actions/runs/38132324438) 19/19 job xanh (Windows package 2 kịch bản, Linux IBus/Fcitx5, clippy cross-target), [`ci-macos`](https://github.com/hunglinhpt/TextVN/actions/runs/38132324458) 6/6 job xanh (`swift build + test` **x86_64 + arm64**), [`repo-hygiene`](https://github.com/hunglinhpt/TextVN/actions/runs/38132324466) xanh |
| Lỗi mới gặp trong phiên (B4⑤) | **E14** (clippy-linux đỏ sẵn trên `main` — import chỉ-Windows không gate), **E15** (giới hạn pipe stdin của máy dev → `check-tables`/`replay` FATAL cục bộ), **E16** (`ci-macos` đỏ: `String?` chưa ràng buộc ở `TextVNInputController.swift:547` + `try?` vô nghĩa — `swiftc -parse` không typecheck), **B21** (Windows package CI đỏ do block A6 `ExecAsOriginalUser` ở `usUninstall` giữ `{app}`) — tất cả đã có mã + fix + bằng chứng CI xanh |
| Rút lại A6 (B21) | Fix A6 (`ExecAsOriginalUser` trong `usUninstall`) thua và giữ `{app}` làm Inno không xoá được `TextVN.exe`; đã gỡ block (dư thừa với `[UninstallRun]`) + đặt `WorkingDir` của CLI ra `{tmp}` — CI Windows package 2 kịch bản xanh trở lại |
| Ký số | `gpg --verify SHA256SUMS.txt` → **Good signature** (EDDSA FPR `3921595ABC961199F15303B6C45B84D0C7F4A822`, ký 2026-10-11 18:09 SEAST); `.asc` detached trên `TextVN-macos-universal-v0.2.31.zip` → Good signature; chứng chỉ cosign keyless verify: issuer `O=sigstore.dev, CN=sigstore-intermediate`, SAN URI `https://github.com/hunglinhpt/TextVN/.github/workflows/release.yml@refs/tags/v0.2.31` (ràng buộc chữ ký với workflow trên đúng tag). Publish job CI đã chạy `cosign verify-blob` bắt buộc (fail-closed — R2-74) |
| Trạng thái | release candidate: chưa Authenticode (chờ SignPath Foundation), macOS chưa Developer ID/notarization (`signing-status-mac.md`) |

### Phát hành 0.2.31

| Hạng mục | Bằng chứng |
|---|---|
| Release | v0.2.31 publish 2026-10-11 (pre-release) từ tag `c25617f` — [release-candidate #38133800983](https://github.com/hunglinhpt/TextVN/actions/runs/38133800983) **4/4 job xanh** (macos, linux, windows, publish); **34 asset** = 8 artifact + `SHA256SUMS.txt` (clearsign) + `gpg-release-key.asc` + `.asc`/`.cosign.sig`/`.cosign.cert` cho từng artifact |
| Checksums (pin từ `SHA256SUMS.txt` đã ký) | linux tar.gz `3a96d059…d770` · msix `d53a4d5f…07ba` · mac pkg `cf843c1a…1b6d` · macos tar.gz `b0d1c94f…fe34` · **mac zip `d91362e4…221f`** · portable `c9a4b5d9…9cf3` · setup-machine `693c5cff…b435` · setup exe `b3f3e911…3f49` · `gpg-release-key.asc` `b432cc43…1f29` |
| Homebrew (B7b) | cask `packaging/homebrew/textvn.rb` `sha256` = mac zip `d91362e4…221f` (khớp SHA256SUMS) |

## Bản 0.2.30 — vòng 16 audit 3 nền tảng (A1–A15, B1–B5/B10/B13, C2/C4–C6)

| Hạng mục | Bằng chứng |
|---|---|
| Nội dung | 19 fix audit 3 nền tảng vòng 16 — chi tiết `docs/release/AUDIT-3PLATFORM-2026-10-07.md` §D (mục 1–10 đã sửa) |
| Gate local | `cargo xtask preflight` 22/22 xanh trước tag |
| Ký số | GPG + Sigstore như v0.2.29 |
| Trạng thái | release candidate: chưa Authenticode (chờ SignPath Foundation), macOS chưa Developer ID/notarization |

## Bản 0.2.29 — gỡ cài đặt sạch (thoát tray trước, dọn ngôn ngữ TextVN đã thêm) + MSIX identity Store thật

| Hạng mục | Bằng chứng |
|---|---|
| Release | v0.2.29 publish 2026-10-10 (pre-release) từ `bdd09f0` (merge PR #7) — [release-candidate #38044174107](https://github.com/hunglinhpt/TextVN/actions/runs/38044174107) 4/4 job xanh; **34 asset**; gói `.msix` kèm release mang identity Store thật `23651Linhi.TextVN` (repo variable `MSIX_IDENTITY_NAME`) |
| CI trên commit gắn tag | push `main` `bdd09f0`: [ci-shared #38043830224](https://github.com/hunglinhpt/TextVN/actions/runs/38043830224), [ci-macos #38043830227](https://github.com/hunglinhpt/TextVN/actions/runs/38043830227), [repo-hygiene #38043830221](https://github.com/hunglinhpt/TextVN/actions/runs/38043830221) — xanh. Windows package: `languages before: en-US` → `after install: en-US, vi` → `after uninstall: en-US` (R2-100), `PASS uninstall stopped the running tray first and removed TextVN.exe` (R2-99) |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`); hash tải về khớp danh sách đã ký |
| Checksums (pin) | machine exe `0e227c59…c25d` · setup exe `9543aa53…5100` · MSIX (nộp Store) `435326b6…d5fa` · portable `aff685be…8e0a` · Linux `1547a2df…7690` · pkg `495a90dd…1a77` · mac zip `abd3337c…9064` · mac tar.gz `5a18b138…24cf` |
| Homebrew | cask `sha256` = mac zip `abd3337c…9064` (B7b) |
| Trạng thái | release candidate: chưa Authenticode (chờ SignPath Foundation), macOS chưa Developer ID/notarization; gói MSIX nộp Store = asset `.msix` của chính release này |

## Bản 0.2.28 — rà soát vòng 1 + 2 (kênh Store MSIX, ký số fail-closed, cách gõ mới) + sửa R2-88…R2-98

| Hạng mục | Bằng chứng |
|---|---|
| Release | v0.2.28 publish 2026-10-10 (pre-release) từ `6f0b38b` (merge PR #4) — [release-candidate #38037964686](https://github.com/hunglinhpt/TextVN/actions/runs/38037964686) 4/4 job xanh; **34 asset** = 8 artifact + `SHA256SUMS.txt` + `gpg-release-key.asc` + `.asc`/`.cosign.sig`/`.cosign.cert` cho từng artifact; release notes ghi `.msix` mang identity tạm |
| CI trên commit gắn tag | push `main` `6f0b38b`: [ci-shared #38037620637](https://github.com/hunglinhpt/TextVN/actions/runs/38037620637), [ci-macos #38037620653](https://github.com/hunglinhpt/TextVN/actions/runs/38037620653), [repo-hygiene #38037620690](https://github.com/hunglinhpt/TextVN/actions/runs/38037620690) — xanh. Trên nhánh `0ab8210` (cùng cây): ci-shared 19/19 gồm `PASS portable upgrade from 0.2.27 … (R2-92)`, release-candidate chạy thử (sideload MSIX gồm `stage.json da xoa` — R2-96; `publish` skipped) |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`); hash tải về khớp danh sách đã ký (mac zip, machine exe, msix) |
| Checksums (pin) | machine exe `21ab5bcc…7b96` · setup exe `b83fea94…8616` · MSIX identity tạm `251be560…3320` · portable `27b4ef15…de41` · Linux `27d2a569…c9d0` · pkg `cb34ca84…0131` · mac zip `1e3fa94e…0ee0` · mac tar.gz `744ee8d1…9e62` |
| `RELEASE_REPORT.json` | `version` 0.2.28, `git_commit` `6f0b38b…`, `source_tree_clean` true, `feature_profile` tsf-only, `checks.authenticode` = `not-signed`, `checks.pe_static_crt` = `passed` |
| Gói MSIX nộp Store | Build lại trên tag bằng ô `msix_identity_name` — [release-candidate #38038814173](https://github.com/hunglinhpt/TextVN/actions/runs/38038814173) (`workflow_dispatch`, `publish` skipped): Identity `23651Linhi.TextVN`, Publisher `CN=1A703CAB-…7545`, Version `1.2.28.0`, DisplayName `TextVN`, PublisherDisplayName `LinhBH.CoM`; `verify-msix.py --require-store-identity` OK (CI và máy chủ repo); sideload CI PASS `23651Linhi.TextVN_1.2.28.0_x64__qgnrpq341n182`. SHA-256 `20eced0f…2eed` — gói build riêng cho Store, không nằm trong `SHA256SUMS.txt` của release |
| Homebrew | cask `sha256` = mac zip `1e3fa94e…0ee0` (B7b) |
| Máy Windows 11 thật | `build-release.ps1` 0.2.28: smoke qua, đăng ký HKCU / marker Ctrl+Shift / `Layout Hotkey` không đổi sau smoke (R2-91); 421 test, replay 56 · 134 · 170 · 56; ISCC compile cả hai biến thể bộ cài. Cài `-machine.exe` + gõ TSF Notepad 12/12 đã chạy trên bản từ PR #3 (`code-review-2026-10-09-round2.md` §6) |
| Branch `approved` (B7c) | `v0.2.28/`: setup exe, `-machine.exe`, portable zip + `.asc`/`.cosign.*`, `SHA256SUMS.txt`, `gpg-release-key.asc` (byte-identical, `gpg --verify` từng `.asc` OK, raw URL 200, hash tải lại khớp) + gói nộp Store `TextVN-0.2.28-windows-x64-store.msix` (`.sha256` cạnh nó; ngoại lệ ghi rõ trong README của branch) |
| Trạng thái | release candidate: chưa Authenticode (chờ SignPath Foundation), macOS chưa Developer ID/notarization (`signing-status-mac.md`); gói MSIX nộp Store sẵn sàng — chủ repo upload lên Partner Center (`msix-submission.md` §4). Repo variable `MSIX_IDENTITY_NAME` đã đặt — bản sau kèm gói nộp được ngay trong release |

## Bản 0.2.27 — macOS tôn trọng "Gõ không gạch chân" (BUG-05) + ký số tự động GPG/Sigstore + MSIX có DLL x86

| Hạng mục | Bằng chứng |
|---|---|
| Release | v0.2.27 publish 2026-10-07 (pre-release) từ `0e9c5d3` — [release-candidate #37574962202](https://github.com/hunglinhpt/TextVN/actions/runs/37574962202) 4/4 job xanh; **34 asset** = 8 artifact + `SHA256SUMS.txt` + `gpg-release-key.asc` + `.asc`/`.cosign.sig`/`.cosign.cert` cho từng artifact |
| Ký số | `gpg --verify SHA256SUMS.txt` → Good signature (FPR `3921595A…A822`); mỗi artifact có chữ ký GPG + Sigstore keyless (`signing.md`) |
| Checksums (pin) | machine exe `d14513d8…584c` · setup exe `ffa7ff61…8d89` · MSIX `6ee8c75c…133f` · portable `d3d9a8f3…eff7` · Linux `5ccf9ac8…2971` · pkg `4fbe5725…67c7` · mac zip `532fb651…0f42` · mac tar.gz `7e11d5cf…41db` |
| `RELEASE_REPORT.json` | `version` 0.2.27, `git_commit` `0e9c5d3…`, `source_tree_clean` true, `feature_profile` tsf-only, `checks.authenticode` = `not-signed` |
| Đã biết (sửa ở *Unreleased*, chưa phát hành) | `.msix` 0.2.27 **không nộp Store được** (identity placeholder, Version `0.2.27.0`, `<Description>` mojibake, ảnh 71×71 sai — sửa `bcc452b`/`b10be1c`, build lại theo `msix-submission.md`); binary x64 còn phụ thuộc VC++ runtime (CRT tĩnh từ `ae0a938`); Homebrew cask `sha256` lệch (sửa `de3e1a7`); release notes trên GitHub còn ghi "Unsigned" (workflow đã sửa cho bản sau) |
| Trạng thái | release candidate: chưa Authenticode (chờ SignPath Foundation), macOS chưa Developer ID/notarization (`signing-status-mac.md`) |

## Bản 0.2.26 — dual-DLL (Zalo 32-bit) + ghost uninstall + Linux BUG-03 + policy 10.2.9

**Nguồn tải Partner Center (machine — bản Store chính thức):**
`approved/v0.2.26/TextVN-setup-0.2.26-windows-x64-machine.exe`
SHA-256 `9c7519e1…361b` (200/0 redirect/byte-identical). Lưu ý 10.2.9: gói
**chưa ký** → xem `store-policy-10-2-9.md` (Trusted Signing hoặc MSIX).

| Hạng mục | Bằng chứng |
|---|---|
| Zalo forensics | Zalo.exe `0x014C` (32-bit); WOW6432Node CLSID trống → TIP 64-bit không nạp được (B20) |
| Dual-DLL | `textvn-tsf-x86.dll` i686 (I386 gate, CRT static) trong zip/installer/MSIX; mirror WOW6432Node COM+CTF cả 2 scope; unregister xoá mirror |
| Zalo LIVE trên máy chủ repo | copy x86 DLL vào install dir + `register --dll` → WOW6432Node InprocServer32 trỏ x86 DLL (user test Zalo sau khi restart Zalo) |
| Ghost uninstall (vòng 13) | dump 27 path → unregister mới → **0 path** (elevated); gate quét GUID trong test-installer |
| Linux BUG-03 | IPC client nhận `"app_id":"*"` (StateUpdate+Snapshot) event-based qua seq; engine IBus/Fcitx5 poll tại init/focus/activate; recv_frame phân biệt would-block; test parser trực tiếp + socket test |
| Tray startup | TSF/offer chuyển background thread — `--stop` thoát tức thì (hết "tray still running") |
| CI | ci-shared + repo-hygiene + ci-macos xanh (666bd29); release 4/4 job, 9 asset |
| Policy 10.2.9 | MS từ chối exe chưa ký → kế hoạch Trusted Signing / MSIX trong `store-policy-10-2-9.md` |

## Vòng 12 + bản 0.2.25 — audit MSIX lần đầu soi kỹ: 2 lỗi kiến trúc, đã fix triệt để

Rà từng dòng pipeline MSIX (chưa từng upload nên chưa từng bị soi) tìm ra 2 lỗi
sẽ làm MSIX thành "vô dụng dù cài được":

| # | Lỗi | Hậu quả nếu không fix |
|---|---|---|
| 1 | Tray chạy từ `WindowsApps\<Package>_<version>_…` tự đăng ký TSF **trỏ vào DLL trong package** — thư mục chứa version, bị XOÁ sau mỗi lần Store update; cộng rủi ro registry virtualization (ghi COM/CTF từ tiến trình đóng gói vào hive riêng, vô hình với Notepad/explorer) | Mất gõ ngay sau lần update đầu (đăng ký treo lơ lửng); có thể vô hình hoàn toàn |
| 2 | Payload MSIX **thiếu `resources/` + `data/`** (gồm `data/tables/*.toml` engine đọc từ đĩa) | Tray staged thiếu icon; engine hỏng khi load table |

**Fix (bootstrap stage-out — pattern "kênh phân phối" đã định hướng từ 0.2.17,
giờ thành code thật):** exe trong package chỉ làm một việc — stage toàn bộ
payload (tray, CLI, TSF DLL, resources, data) ra `%LOCALAPPDATA%\Programs\TextVN` rồi spawn bản staged và thoát, TRƯỚC khi tạo single-instance mutex.
Bản staged chạy **non-packaged** → dùng lại nguyên vẹn cơ chế portable đã chứng
minh (tự đăng ký TSF, UAC machine B7, Run key, IPC). File bị khoá khi re-stage
→ rename-then-copy (B8). 4 unit test mới. Đo thật: tiến trình thường đọc được
file trong WindowsApps (READ OK — loại trừ nghi ngờ ACL; nguyên nhân thật là
update-fragility + virtualization).

| Gate | Bằng chứng |
|---|---|
| Unit test bootstrap | 4/4 PASS (nhận diện WindowsApps, stage đủ file/dir, ghi đè, không relaunch ngoài package) — 42/42 test tray |
| Unpack MSIX 0.2.25 | `resources/` ×3 + `data/` ×12 + manifest Version 0.2.25.0; identity placeholder (cảnh báo in to khi build) |
| `cargo xtask preflight` | PASS 22/22 (version-sync = 0.2.25) |
| Release v0.2.25 | 4/4 job PASS, 9 asset; approved byte-identical: MSIX `a077131e…5088`, machine exe `98c993db…be57` |

Còn thiếu duy nhất để nộp MSIX: `Package/Identity/Name` — khi có:
`build-msix.ps1 -Publisher "CN=1A703CAB-…" -IdentityName "<Name>"`.

> **Đã thay (2026-10-08, vòng rà soát 2):** stage-out trong gói ra
> `%LOCALAPPDATA%\Programs\TextVN` bị ảo hoá (tiến trình có package identity) — kiến trúc
> hiện hành chép ra `%USERPROFILE%\.textvn\msix-staging` → `%LOCALAPPDATA%\Programs\TextVN-Store\<V>`
> ngoài container, chỉ đăng ký HKCU, có guard dọn khi gỡ gói và test sideload thật trong CI.
> Trạng thái/lệnh build hiện hành: `msix-submission.md`.

## Vòng 11 + bản 0.2.24 — đóng nốt 3 khe hở cấp dòng (rà lần 3) + bản máy là bản Store

Rà từng dòng lần 3 theo yêu cầu chủ repo tìm ra 3 khe hở THẬT phía gói mà các
lần trước bỏ lỡ — tất cả đều là thứ một checker có thể soi:

| Khe hở | Trước đây | Sau 0.2.24 | Verify |
|---|---|---|---|
| Entry ARP thiếu `DisplayIcon` | không set `UninstallDisplayIcon` | trỏ `{app}\TextVN.exe` | probe end-to-end: `DisplayIcon` xuất hiện đúng trong ARP |
| Chuỗi `FileVersion` của setup.exe TRỐNG | chỉ có số nội bộ | `VersionInfoTextVersion` → `0.2.24` | Get-Item VersionInfo cả 2 biến thể |
| Không có log cài đặt trên VM validator | không `SetupLogging` | log %TEMP% — bằng chứng duy nhất có thể yêu cầu nếu fail tiếp | directive compile OK |

Cùng lần này: **chốt theo quyết định chủ repo** — bản **machine (admin/all-users,
HKLM) là bản Store chính thức** (MS doc: *"UAC prompts are allowed"*; khác
0.2.18 chết vì `requireAdministrator` — bản này asInvoker + Inno tự relaunch);
portable giữ nguyên; GitHub users vẫn nhận bản per-user.

| Gate | Bằng chứng |
|---|---|
| `cargo xtask preflight` | PASS 22/22 (version-sync = 0.2.24) |
| CI `ci-shared` (2c0ae91) | `PASS 2) ARP entry: 'TextVN' | 'LinhBH.CoM' | '0.2.24'` + simulate + machine-variant PASS |
| Release v0.2.24 | 4/4 job PASS (windows đỏ 1 lần do flake typing test B5 — rerun xanh) |
| Nguồn tải Store | `approved/v0.2.24/TextVN-setup-0.2.24-windows-x64-machine.exe` — 200/0 redirect/byte-identical `aa31b2d5…b53` |

## Vòng 10 — rà soát TỪNG DÒNG .iss + build options (yêu cầu chủ repo): không có option nào sai

Bằng chứng lấy THẬT trên máy chủ repo (2026-10-05), không suy đoán:

1. **Entry ARP thật mà checker sẽ đọc** (dump toàn bộ value của entry 0.2.19
   đang cài trên máy, sinh ra từ chính .iss này):
   `DisplayName=TextVN`, `Publisher=LinhBH.CoM`, `DisplayVersion`,
   `UninstallString`, `QuietUninstallString`, `InstallLocation`, `NoModify=1`,
   `NoRepair=1`, `EstimatedSize`, `VersionMajor/Minor` — cấu trúc textbook chuẩn
   Inno, thiếu thứ gì validator cần. **Chỉ tồn tại ở HKCU** (HKLM + WOW6432Node
   = 0 entry) — đúng thiết kế per-user.
2. **Metadata setup exe (cả 2 biến thể)**: `CompanyName=LinhBH.CoM`,
   `ProductName=TextVN`, `FileDescription=TextVN Setup`,
   `ProductVersion=0.2.23` — chuẩn. (Lưu ý nhỏ: chuỗi `FileVersion` trống —
   Inno chỉ ghi số; vô hại với 3 check đang đỏ, không phải nguyên nhân.)

Bảng phán xét từng nhóm directive trong `TextVN-setup.iss`:

| Directive / khối | Giá trị hiện tại | Phán xét với validator silent |
|---|---|---|
| `PrivilegesRequired` | lowest (per-user) / admin (biến thể máy) | Đúng — asInvoker chạy được trong harness non-elevated |
| `PrivilegesRequiredOverridesAllowed` | commandline | Đúng — đây là lý do manifest là asInvoker (cả 2 biến thể) |
| `ShowLanguageDialog` | no | Đúng — không dialog ngôn ngữ theo locale |
| `DisableWelcomePage` | no | Vô hại — /VERYSILENT bỏ qua mọi trang wizard |
| `[Tasks] checkedonce` (autostart, freectrlshift) | mặc định CHECK | Vô hại — chỉ ghi 1 value Run HKCU + [Run] bị skipifsilent |
| `[Run]` | cả 3 entry đều skipifsilent | Đúng — không chạy tiến trình nào trong silent |
| `CurStepChanged` silent | pure copy, `RegistrationOK:=True` | Đúng — không gọi API TSF trong sandbox |
| `GetCustomSetupExitCode` silent | luôn 0 | Đúng |
| `PrepareToInstall` → RenameLockedTsfDll | no-op trên VM sạch | Đúng |
| `[Registry]` Run key (HKA) | non-elevated → HKCU | Đúng |
| `DefaultDirName={autopf}` | %LOCALAPPDATA%\Programs\TextVN | Đúng |
| `Compression/SolidCompression/WizardStyle/Architectures*` | chuẩn | Vô hại |
| `[Languages]` vi+en | compile-time | Vô hại |
| Metadata setup.exe | CompanyName/ProductNamame đúng | Chuẩn |

**Kết luận vòng 10:** không còn option Inno nào để sửa — bộ cài đã đạt tối đa
những gì một EXE non-elevated làm được: cài silent 1s exit 0, ARP textbook đúng
từng chữ, đúng 1 entry, gỡ sạch (đủ bằng chứng CI + máy thật). Ba nguyên nhân
còn lại đều nằm ở môi trường validator, không sửa được từ phía gói:

1. **Validator chỉ đọc HKLM** (hoặc cài dưới account khác rồi kiểm từ chỗ khác)
   — khớp toàn bộ lịch sử: 0.2.18 (requireAdministrator → 740 = launcher
   non-elevated) + mọi vòng per-user đỏ.
2. **Sandbox non-elevated** ⇒ biến thể máy sẽ fail ở bước Inno xin elevation
   (RunAs trong phiên không-interactive → ERROR_CANCELLED → exit 1, message bị
   /SUPPRESSMSGBOXES nuốt). Vẫn đáng nộp 1 lần vì free — nếu PASS thì EXE sống.
3. **SmartScreen/uy tín tệp** chặn exe chưa ký trước khi chạy — không kiểm
   được từ ngoài; MSIX hết dính vì Store tự ký gói.

⇒ Hành trình EXE: nộp machine (thẻ cuối) → MSIX là đích đến chắc chắn
(`CN=1A703CAB-…` đã có, chờ `Package/Identity/Name`).

## Vòng 9 (2026-10-05) — 0.2.23 vẫn đỏ: EXE per-user cạn, leo thang machine → MSIX

0.2.23 per-user (chuỗi `TextVN`/`LinhBH.CoM` ĐÚNG từ nguồn sự thật — chủ tài
khoản đã tự sửa trong account) vẫn đỏ 3 mục. Ghi mã **B19**: gói EXE per-user
đã cạn mọi biến số kiểm soát được; các nguyên nhân còn lại (validator đọc
HKLM / account khác / sandbox non-elevated / SmartScreen với exe chưa ký) đều
không sửa được từ phía gói. Kế hoạch: nộp **machine 0.2.23** (thẻ EXE cuối,
đã publish sẵn) → nếu đỏ, **MSIX** là đường duy nhất còn lại (đã có Publisher
`CN=1A703CAB-…`, chỉ chờ `Package/Identity/Name`).

## Bản 0.2.23 — Publisher display name THẬT = `LinhBH.CoM` (vòng 8, chấm dứt chuỗi đoán)

Chủ tài khoản tìm ĐÚNG trang trong account settings và **tự sửa** Publisher
display name thành `LinhBH.CoM` (2026-10-05). Điều này giải thích trọn 7 vòng:

| Vòng | Chuỗi ARP | Kết quả Store | Nguyên nhân thất bại thật |
|---|---|---|---|
| 0.2.16 | `LinhBH.CoM` | Đỏ 3 mục | exit 10 khi đăng ký TSF trong sandbox (admin) |
| 0.2.17 | `LinhBH.CoM` | Đỏ 3 mục | per-user đầu tiên nhưng thiếu pure-copy silent |
| 0.2.18 | `LinhBH.CoM` | Đỏ 3 mục | admin/`requireAdministrator` → 740 trong VM |
| 0.2.19 | `LinhBH.CoM` | "no changes" | bẫy B13g — nộp lại URL cũ, không revalidate |
| 0.2.20 | `LinhBH.CoM` | (không nộp — chuẩn bị MSIX/VT fix) | — |
| 0.2.21 | `Linh βùi` | Đỏ 3 mục | chuỗi SAI (nhầm font B↔β) |
| 0.2.22 | `Linh Bui` | Đỏ 3 mục | chuỗi SAI (suy đoán từ lần đọc sai thứ hai) |
| **0.2.23** | **`LinhBH.CoM`** | **chờ nộp** | chuỗi đúng + gói mới + không elevation |

| Gate | Bằng chứng |
|---|---|
| `cargo xtask preflight` | PASS 22/22 (version-sync = 0.2.23) |
| Build local cả 2 biến thể | `LinhBH.CoM` utf16 ×1; `Linh Bui`/β = 0 |
| CI `ci-shared` (984c347) | `PASS 2) ARP entry: 'TextVN' | 'LinhBH.CoM' | '0.2.23'` + simulate `PASS B2 … Publisher='LinhBH.CoM'` + machine-variant PASS (job đỏ 1 lần do flake tray — B5, rerun xanh) |
| Release v0.2.23 | 4/4 job PASS, 9 asset |
| Repo VARIABLES | `STORE_APP_NAME=TextVN`, `STORE_PUBLISHER_NAME=LinhBH.CoM` |
| Nguồn tải | `approved/v0.2.23/` — 200/0 redirect/byte-identical: per-user `81b7c3d4…9575`, machine `9fc76774…7ba9a` |

## Bản 0.2.22 — ROOT CAUSE chốt: ARP Publisher = `Linh Bui` (B18)

Chủ tài khoản xác nhận **lần 2**: Publisher display name = `Linh Bui` (ASCII).
`β` ở 0.2.21 là **nhầm font** (B U+0042 nhìn như β U+03B2). Chuỗi publisher
trong ARP vì vậy **chưa từng đúng**: 0.2.16-0.2.20 `LinhBH.CoM`, 0.2.21
`Linh βùi` — giải thích trọn 6 vòng đỏ *"could not identify the app name and
the publisher name"* mà không cần giả thuyết hive.

| Gate | Bằng chứng |
|---|---|
| `cargo xtask preflight` | PASS 22/22 (version-sync 14 site = 0.2.22) |
| Build local cả 2 biến thể | `Linh Bui` utf16 ×1 trong từng exe; `LinhBH.CoM`/β = 0 (DLL bị khoá → rename `.old-` theo B8 rồi build lại) |
| CI `ci-shared` (ff8bc21) | `PASS 2) ARP entry: 'TextVN' | 'Linh Bui' | '0.2.22'` (harness) + `PASS B2 … Publisher='Linh Bui'` (simulate tải từ raw URL) + machine-variant PASS |
| Release v0.2.22 | 4/4 job PASS; **9 asset** — lần đầu có `TextVN-setup-0.2.22-windows-x64-machine.exe` (publish job kiểm đúng 2 setup exe) |
| Repo VARIABLES | `STORE_APP_NAME=TextVN`, `STORE_PUBLISHER_NAME=Linh Bui` → CI đối chiếu giá trị thật mỗi commit |
| Nguồn tải | `approved/v0.2.22/` — 200/0 redirect/byte-identical: per-user `44de0a40…cf2b7`, machine `751e0d76…3a3d7` |

Kế hoạch nộp: **per-user 0.2.22 trước** → nếu vẫn đỏ: machine 0.2.22 → nếu vẫn
đỏ: MSIX (`CN=1A703CAB-…` đã có, chờ `Package/Identity/Name`).

## Vòng 6 — đánh giá 5 gốc rễ của audit ngoài + vá phần còn thiếu

Audit ngoài (gốc rễ 1-5) đối chiếu với trạng thái repo:

| Gốc rễ | Đánh giá | Đã làm |
|---|---|---|
| 1. Gói nộp Store vẫn per-user; machine exe chưa lên approved | **Đã sai từ lượt trước** — machine exe đã nằm ở `approved/v0.2.21/` (commit 250b57a, blob `f254a5e5…`, verify 200/byte-identical). NHƯNG đúng ở chỗ: pipeline release chưa build nó thành asset chuẩn | `build-release.ps1 -MachineInstaller`; release.yml thêm bước "Build + validate machine installer" (build + `-MachineOnly` PASS) và publish job kiểm **đúng 2** setup exe (per-user + machine) — từ tag kế tiếp machine exe là release asset chính thức |
| 2. Validator chỉ quét HKLM + cô lập HKCU | **Giả thuyết dẫn đầu nhưng KHÔNG phải nội dung tài liệu MS** — manual-package-validation không nói hive nào; đây là suy luận (hợp lý) của audit. Lần nộp bản máy sẽ kết luận thật | (giữ nguyên, đã có bằng chứng hai chiều trên CI) |
| 3. β (U+03B2) vs B (U+0042) | **Rủi ro thật, chỉ chủ tài khoản phân giải được** — hiển thị font không phân biệt được; tên pháp lý thường không chứa β nhưng Publisher display name là text tự nhập | B18 + snippet dump codepoint; repo variable `STORE_APP_NAME=TextVN` đã đặt; `STORE_PUBLISHER_NAME` chờ chủ tài khoản dump codepoint xong mới đặt |
| 4. CI tự so code với chính nó | **Valid một phần** — `-MachineOnly` + machine step đã có từ vòng 4, nhưng chưa nối giá trị thật | Job Windows của ci-shared giờ đọc repo VARIABLES `STORE_APP_NAME`/`STORE_PUBLISHER_NAME`; simulate + harness đều tôn trọng |
| 5. Catch-22 EXE per-user/machine | **Đúng** — và là lý do MSIX là lối thoát kiến trúc | Đường B giữ nguyên: chỉ còn thiếu `Package/Identity/Name` (Publisher đã có: `CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545`) |

Lệnh dựng machine variant ở máy local: `build-release.ps1 -BuildInstaller -MachineInstaller`.

## Vòng 5 (0.2.21 vẫn đỏ) — điều tra + phương án máy

Kết quả 0.2.21 trên Store **vẫn đỏ 3 mục** dù ARP `Publisher = "Linh βùi"` đúng
từng codepoint (CI tự chứng minh trên chính file nộp: simulate tải từ raw URL →
`PASS B2 … Publisher='Linh βùi'`). Chuỗi định danh bị loại khỏi nghi vấn →
nghi ngờ dồn về **nơi validator đọc entry**: HKCU của phiên cài vô hình với
validator (cài dưới account/phiên khác hoặc chỉ quét HKLM) — STO-02.

Hành động:
1. Biến thể MÁY của **cùng** 0.2.21 (cùng commit `517db48`, cùng chuỗi
   publisher) đã được CI build + validate: run 37210008091 — `PASS 0-4` với
   `-MachineOnly` (entry ở HKLM, Program Files, silent exit 0 trong 1s, gỡ
   sạch). Byte đó được publish lên
   `approved/v0.2.21/TextVN-setup-0.2.21-windows-x64-machine.exe`
   (SHA-256 `af3c93a8…cf16f6`, HTTP 200/0 redirect/byte-identical đã verify).
2. Windows publisher ID tài khoản (chủ tài khoản cung cấp):
   `CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545` — đủ cho `Identity@Publisher` của
   MSIX; chỉ còn thiếu `Package/Identity/Name` (trang Product identity) để build
   MSIX cuối. `build-msix.ps1` đã có lệnh dựng + cảnh báo placeholder.
3. Ma trận quyết định cho lần nộp kế tiếp ghi ở `store-submission.md` §1a-ter
   (đường A: bản máy; đường B: MSIX).

## Bản 0.2.21 — ARP Publisher khớp ĐÚNG Partner Center (STO-03) + B17

Chủ tài khoản xác nhận 2 chuỗi trên Partner Center (2026-10-04): Product name =
`TextVN` (→ ARP `DisplayName` đã khớp từ trước) và **Publisher display name =
`Linh βùi`** (beta U+03B2, ù U+00F9) — trước đây installer ghi `LinhBH.CoM` nên
validator báo *"could not identify the app name and the publisher name"*.
0.2.21 sửa `AppPublisher` thành đúng chuỗi đó; `.iss` giữ UTF-8 BOM để ISCC đọc
đúng ký tự đặc biệt ở mọi phiên bản Inno.

| Gate | Bằng chứng |
|---|---|
| Verify encoding end-to-end (probe riêng, AppId tạm) | Cài im lặng → đọc ARP → `Publisher` đúng codepoint `U+004C U+0069 U+006E U+0068 U+0020 U+03B2 U+00F9 U+0069` (`-ceq` = True) → gỡ sạch |
| `cargo xtask preflight` (máy chủ repo) | PASS 22/22 (bắt 1 lỗi thật: `Ordering` rò từ cfg(windows) sang Linux — đã sửa) |
| `build-release.ps1 -BuildInstaller` | BUILD_EXIT=0; smoke test bắt được **B17** (xem dưới) |
| CI `ci-shared` job "Windows package" (517db48) | 12/12 step PASS: harness `PASS 2) ARP entry: 'TextVN' | 'Linh βùi' | '0.2.21'`; simulate `PASS B2 … Publisher='Linh βùi'`; machine-variant PASS 0-4 |
| Release workflow v0.2.21 (tag) | 4/4 job PASS, GitHub Release đủ 8 asset |
| Nguồn tải Partner Center | `approved/v0.2.21/TextVN-setup-0.2.21-windows-x64.exe` — HTTP 200, 0 redirect, byte-identical `fa486e6d…d279`; **CI validate lại từ chính URL** (Downloaded 2690740 bytes → silent exit 0 → ARP `Linh βùi`) |

**B17 (lỗi mới bắt được khi dựng 0.2.21):** `WM_REQUEST_EXIT` gửi tới tray khi
cửa sổ CHƯA tồn tại (tray còn đang đăng ký TSF/dò activation >1 s) bị **mất âm
thầm** → `--stop` ngay sau khi start không có tác dụng (smoke báo *"Runtime
smoke did not stop cleanly"* và để lại tiến trình mồ côi). Sửa: latch
`TRAY_EXIT_PENDING` + tray tiêu thụ trước vòng lặp; smoke POLL 10 s thay vì
`sleep 500 ms` cố định và kill tiến trình mồ côi khi fail.

## Bản 0.2.20 — CI xanh lại + validation vòng 3 (harness đúng như Microsoft)

Bối cảnh: CI đỏ job "Windows package" trên `7e2fe53` — `textvn-setup.iss` gọi
`RegDeleteKeyValue`, hàm **không tồn tại** trong Pascal Script của Inno Setup
(chỉ có `RegDeleteValue`). ISCC chỉ chạy trong CI nên máy dev không bắt được →
ghi mã **B14** + luật: biên dịch thật bằng ISCC trước khi push `[Code]`.
Commit sửa: `cd4e141` (12 file code/docs) + `7631ecf` (notice thread riêng) +
`fb3b351` (Resolve-Iscc).

| Gate | Bằng chứng |
|---|---|
| `cargo xtask preflight` (22 bước, máy chủ repo) | PASS 22/22 (gồm `check-version-sync` 14 site = 0.2.20, `store-validate` SKIP trên máy dev đang có TextVN, `iss-tabs`, `ascii-ps1`) |
| ISCC thật (`ISCC.exe /DMyAppVersion=0.2.20`) | Compile OK 1.6s, ra `dist\TextVN-setup-0.2.20-windows-x64.exe` |
| `build-release.ps1 -BuildInstaller` (máy chủ repo) | BUILD_EXIT=0 — fmt/clippy/verify/replay/test/smoke/zip/installer PASS; PE VersionInfo 0.2.20.0 |
| CI `ci-shared` job "Windows package" (commit `7631ecf`) | Step 9 "Store package validation (MS manual checks)" PASS (harness có gate manifest `asInvoker` + ARP exact) và step 10 "Simulate Store validator" PASS (tải raw URL `approved/v0.2.20`, MOTW, silent, ARP, bundleware, uninstall) |
| `ci-macos` (commit `cd4e141`) | Xanh |
| `repo-hygiene` (commit `cd4e141`/`7631ecf`) | Xanh |
| Nguồn tải Partner Center | `https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.20/TextVN-setup-0.2.20-windows-x64.exe` — HTTP 200, 0 redirect, byte-identical (`faac21e3…f112`) |

Gói **MỚI** so với mọi lần nộp Store trước (0.2.16–0.2.19): bắt buộc để Partner
Center chạy lại validation — bẫy "We did not find any changes in the Package or
the Silent install parameters" đã ghi thành **B13g**.
MSIX 0.2.20 (`dist\TextVN-0.2.20-windows-x64.msix`) build sẵn cho đường dự
phòng, nhưng Identity hiện là placeholder (`LinhBH.CoM.TextVN`) — phải build lại
với **đúng** Name/Publisher lấy từ Partner Center trước khi upload (xem
`docs/release/msix-submission.md`).

## Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành

Log thực tế ngày 2026-09-30: ghi COM/CTF dưới HKCU thành công nhưng
`RegisterProfile` báo `0x80004005`, `InstallLayoutOrTip` thành công và hậu kiểm
profile thất bại. Do đó thông báo cũ đổ lỗi quyền ghi HKCU không đúng. Source
đã chuyển installer sang đăng ký TSF machine tại Program Files bằng quyền
Administrator một lần, rồi bật profile bằng `ExecAsOriginalUser`; bản portable
vẫn best-effort, không nâng quyền file nằm trong thư mục người dùng ghi được.

CI Windows tại [run 36866451196](https://github.com/hunglinhpt/TextVN/actions/runs/36866451196)
đã qua Inno Setup và cả hai kịch bản portable/installer, gồm
`test-installer.ps1` (cài, đăng ký, gõ TSF thật vào Notepad, gỡ). Đây là bằng
chứng cho commit `ee09831`, **không phải** cho release/tag v0.2.3. CI
[macOS run 36866451291](https://github.com/hunglinhpt/TextVN/actions/runs/36866451291)
cũng xanh: Swift arm64/x86_64, corpus, staticlib universal và gói unsigned.
Các gate còn lại để gọi là production: thử trên máy từng gặp `0x80004005`,
kiểm tra UAC dùng tài khoản admin khác và gỡ sạch, GUI smoke trên Mac thật,
ký Authenticode và Developer ID/notarization. Không suy diễn từ runner GitHub
sang mọi cấu hình người dùng.

### Gate trước tag 0.2.4

| Gate | Bằng chứng |
|---|---|
| Local Windows `fmt`, `test --workspace`, `clippy -D warnings` | Đạt trên code sửa TSF/Swift; cross-target `cargo check` và `clippy` Linux/macOS cũng đạt |
| ABI + corpus | `verify` khớp 11 export; replay 149/149 mac, 113/113 TSF Windows |
| `ci-shared` code commit `ee09831` | [Run xanh](https://github.com/hunglinhpt/TextVN/actions/runs/36866451196), gồm Windows package/typing và Linux IBus/Fcitx5 e2e/package; perf là job không chặn |
| `ci-macos` code commit `ee09831` | [Run xanh](https://github.com/hunglinhpt/TextVN/actions/runs/36866451291), gồm Swift 2 kiến trúc và gói unsigned |
| `repo-hygiene` code commit `ee09831` | [Run xanh](https://github.com/hunglinhpt/TextVN/actions/runs/36866451224) |
| Native test trên máy gặp `0x80004005`, UAC admin khác, ký số | Chưa đạt; không tuyên bố production |

Perf trên runner dùng chung báo vượt 10% ở một số lần chạy nhưng không chặn
workflow theo thiết kế. Chạy trực tiếp binary benchmark trên host này cho
`ime_key` p50 500 ns (baseline 687 ns), `parse_config` 781 ns (baseline 781 ns),
`strategy_resolve` 10 ns (dưới nhiễu timer); không dùng phép đo đó để khẳng
định hiệu năng trên mọi phần cứng.

### Tiếp tục audit ZCode — engine (2026-10-01)

Đã chạy 22 ca repro tạm ZCode qua `cargo test -p textvn-core --test
audit_r3_tmp -- --nocapture`, rồi chuyển các lỗi thật thành unit test có assert:

| Quan sát | Phân loại | Xử lý |
|---|---|---|
| VNI `200` → `2`, `0912` → `912`, `0` → rỗng | Mất dữ liệu P0: phím xoá dấu `0` bị nuốt ngay cả khi không có dấu | Chỉ nuốt `0` khi thực sự gỡ được dấu; test VNI số và lần `0` thứ hai |
| Macro `too` không bung sau khi Telex fold thành `tô` | Lỗi P1: so trigger với text đã xuất thay vì phím gốc | Thử match raw word đang active; `delete_count` dùng số glyph đã xuất, test `too<Tab>` |
| `ccc` → `cc`, Backspace → `ch` | Theo semantics Quick Telex hiện tại: xoá phím gõ thứ ba làm cặp `cc` có hiệu lực lại | Không đổi nếu chưa có quyết định UX mới |
| `quiet` → `quiêt`, `duo` → `duơ`, `OOps` → `Ốp` | Mơ hồ EN/VI; validator cấu trúc không thể phân biệt từ tiếng Anh với âm tiết trông hợp lệ | Dùng `english_words` opt-in/ESC; không tự thêm từ điển mặc định vì phá từ Việt thật |
| `cy` + Backspace + `t` + Tab trong test tạm → `cCT` | Bộ mô phỏng test tạm không xoá ký tự buffer khi `Action::Pass` cho Backspace | Không phải chứng cứ lỗi engine; test production phải mô phỏng phím này đúng |
| VIQR `chao.` → `chạo` | `.` là marker dấu nặng của VIQR, đồng thời là dấu câu | Cần quyết định UX riêng cho escape/commit, không tự đổi semantics |

`cargo test -p textvn-core`: 109 unit + 7 common-words + 6 keymap đều đạt.
Đây là test headless, **không thay thế** thử gõ qua TSF/IBus/IMK thật.

### Tiếp tục audit ZCode — runtime chưa có bằng chứng production

- Windows TSF: diff ZCode thêm `WaitNamedPipeW` nhưng thiếu feature Cargo
  `Win32_System_Pipes` nên `cargo test -p textvn-cli` không biên dịch. Đã bổ sung
  feature, bỏ import/constant không dùng. Sau push, CI Linux/macOS bắt tiếp
  import Windows và lời gọi `uninstall` không được `cfg` đúng; đã sửa, rồi chạy
  `cargo check` + `clippy --all-targets -D warnings` chéo Linux/macOS tại máy.
- Installer: bước elevated chỉ đăng ký COM/profile ở HKLM; bước người dùng gốc
  chỉ thêm layout/kích hoạt, dùng HKLM và dọn override HKCU cũ. Nếu UAC dùng
  tài khoản admin khác, nhánh cũ vừa kích hoạt sai tài khoản vừa để HKCU COM
  override trỏ DLL đã gỡ. CI installer có assertion không còn override HKCU.
- IPC tray: patch đầu nhả mutex chung trước `WriteFile`, nhưng nhiều broadcast
  đồng thời có thể tạo nhiều thread chờ cùng pipe. Đã thêm tối đa một writer
  và hàng đợi 32 frame mỗi subscriber, gỡ theo identity connection thay vì
  PID, và timeout 5s cho client gửi header rồi bỏ dở payload. Chưa có test
  named-pipe thật trên Windows cho áp lực/backpressure này.
- macOS EventTap: lần check `stopRequested` thứ hai trong patch ZCode nằm cùng
  lock với lần đầu nên không bắt được race trước `CFRunLoopRun`. Đã đổi sang
  `CFRunLoopRunInMode` 100ms và không giữ owner suốt đời thread. CI Swift chỉ ra
  SDK mới yêu cầu `CFRunLoopMode.defaultMode` thay `kCFRunLoopDefaultMode`;
  đã sửa. Vẫn cần Swift CI trên commit cuối và GUI smoke trên Mac thật; không
  tuyên bố đã xác thực trên máy Mac.

Kiểm tra local Windows sau bản vá: `cargo fmt --all -- --check`, `cargo test
--workspace`, `cargo clippy --workspace --all-targets -- -D warnings` đều đạt;
`cargo build --release -p textvn-tray -p textvn-cli -p textvn-win-tsf` đạt.
Máy local này không có `ISCC.exe`, vì thế chưa tự biên dịch/test installer mới
trên máy gặp lỗi TSF gốc; CI Windows ở trên đã làm bước đó trên runner. Không
tạo ZIP/release production từ một bản chưa qua toàn bộ gate. Git qua credential
manager vẫn push được, nhưng GitHub CLI hiện báo token tài khoản `hunglinhpt`
không hợp lệ; thao tác release thủ công bằng CLI cần đăng nhập lại. Workflow
phát hành qua tag vẫn là cơ chế chuẩn sau khi CI xanh.

> `v0.2.16` **đã publish** 2026-10-03 dạng pre-release (mục “Bản 0.2.16” bên dưới). 0.2.15/0.2.14/0.2.13 publish 2026-10-03; 0.2.12–0.2.5 publish 2026-10-02/03. Bằng chứng của bản đó
> nằm ở mục “Bản 0.2.2” bên dưới và
> [audit 2026-09-30](cross-platform-audit-2026-09-30.md) “Vòng 11”.
>
> `v0.2.1` đã được phát hành dạng pre-release. Bằng chứng của bản đó nằm ở
> các phần lịch sử phía dưới. Không dùng số liệu 0.1.0 làm bằng chứng
> production cho 0.2.x.

## Bản 0.2.16 — portable tự chữa trên Win11 24H2+ (B7) + hết ghost khi gỡ (0.2.15)

**Báo cáo chủ repo:** "verify hết portable" → phát hiện + fix 4 vấn đề thật,
kết thúc bằng **portable gõ được lần đầu trên máy chủ repo (build 26300)**.

1. **B7 — portable đứng một mình không gõ được** (activation per-user bị
   Windows từ chối; trước đây chỉ có đường bộ cài). 0.2.16: tray phát hiện
   `activate` fail + chưa có HKLM → hộp thoại MỘT LẦN đề nghị đăng ký phạm vi
   máy (UAC) → `register --scope machine` elevated → chờ HKLM → register lại
   per-user → activate → **tự bật lại chế độ tiếng Việt** (thoát bẫy EN-mode:
   `global_enabled=false` cũng ra chữ raw dù đăng ký đúng — lớp lỗi thứ 2 bắt
   được khi chẩn đoán). Guard: bỏ qua khi `--autostart`, khi đã elevated (CI
   headless — modal từng chặn tray, CI 37112480137), marker
   `uac_prompt_done` chỉ hỏi một lần.
2. **B11 — unregister để lại ghost trong Win+Space**: 0.2.15
   `remove_modern_language_list()` + broadcast chuẩn; verify 2 chiều.
3. **B12 — DLL mồ côi sau khi gỡ**: installer `usPostUninstall` →
   `textvn-cli schedule-delete` (MoveFileEx NULL, elevated); portable →
   RunOnce rmdir; verify sống cả hai nhánh (elevated leg chạy trong uninstaller).
4. **Lớp bug escape**: `\t` bị ghi thành TAB trong `.iss` làm
   `[UninstallDelete]` + `RenameLockedTsfDll` vô hiệu âm thầm (ISCC không bắt)
   → gate `check_iss_tabs.py` trong repo-hygiene + preflight (21 bước).

### Verify cuối — release 0.2.16 portable trên máy chủ repo

- Prompt dé nghị → OK + UAC Yes → HKLM COM trỏ DLL trong thư mục portable;
  `ActivateProfile(VI, session) → OK`; HKL Notepad `0x042A`.
- **E2E gõ thật với BẢN RELEASE: `dduocj text ` → `được text ` PASS** (lần
  đầu tiên portable-only gõ được trên máy này).
- CI: 4/4 job xanh (`cbd4a90` retag), 7 asset, homebrew sha cập nhật (B7b);
  tree sạch.

## Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất

**Yêu cầu chủ repo (2026-10-03):** "đã hoàn thiện EN/VI cho Linux và macOS
chưa; UI/icon 3 nền tảng phải giống nhau; vẫn lỗi DPI — sang màn hình khác
nuốt text 'sai' thành 'sa'; rà soát toàn bộ tài liệu; build phải ghi lại
common mistakes".

### 0.2.13 — EN/VI đa nền tảng + fix macOS nuốt từ điển

- "Từ điển EN..." có đủ 3 nền tảng (macOS sheet + Linux GTK) — FFI mới
  `ime_settings_english_words_text/_set_...` (round-trip test + header check),
  chuẩn hoá cùng quy tắc; **bug thật bắt được: `ConfigModel.swift` (macOS)
  THIẾU `english_words` → mỗi lần `persist()` là xoá từ điển người dùng**.
- CI: `replay corpus/shared --adapter linux` (trước chỉ headless/tsf/mac).
- Swift chỉ compile trên CI macOS — 0.2.14 suýt lọt tiếp lỗi thiếu
  `quick_telex` trong Swift model (ci-macos bắt, retag lại v0.2.14 sau fix).

### 0.2.14 — DPI gốc + UI thống nhất + common errors

- **DPI (B10):** `system_dpi()` dùng `GetDeviceCaps(LOGPIXELSX)` trả
  system-DPI cũ (96) trong khi PerMonitorV2 render theo monitor thật (192 ở
  200%) → chữ to gấp đôi + cắt. Fix: `GetDpiForSystem` + sau `CreateWindowExW`
  đọc `GetDpiForWindow` → khác thì `SetWindowPos` lại cỡ rồi mới layout (cả 3
  cửa sổ). **Xác minh bằng ảnh chụp DPI-AWARE** (`SetThreadDpiAwarenessContext(-4)`
  — chụp từ tiến trình unaware bị ảo hoá, chính là lý do ảnh trước trông như
  cắt): cửa sổ 1226px đúng màn 200%, nhãn "Khôi phục từ tiếng Anh khi gõ sai"
  đủ chữ, hàng nút có "Từ điển EN..." như Linux.
- **B9:** checkbox auto-restore về full bề rộng cột (trước thu 280 để nhét nút
  → cắt "sai"→"sa"); quy tắc mới trong `ui-spec` §1.
- **UI thống nhất 3 nền tảng:** macOS khớp nhãn từng chữ + thứ tự + thêm
  toggle "Bật gõ tiếng Việt" (nối vào AppDelegate như menu bar); tint V/E =
  systemPink/systemBlue khớp badge #C2185B/#0288D1 (ui-spec §7 mới);
  test `labels_match_linux_settings_panel` mở rộng đối chiếu CẢ SettingsView.swift
  → CI chặn lệch nhãn.
- **Common errors theo luật mới (B4 ⑤ — không mã lỗi = không được tag):**
  B8 installer DeleteFile code 5 → rename DLL đang map; B9 nhãn bị cắt;
  B10 DPI system-vs-monitor; E13 "gate CI mới phải vào preflight cùng commit";
  A5 ghi sự cố ps1 non-ASCII + check chung `check_ps1_ascii.py`.
- **Docs rà soát:** README/user-guide hết thông tin cũ "v0.2.4 admin"; ui-spec
  bổ sung §5 (Từ điển EN), §6 (nhận diện EN/VI), §7 (icon); parity-checklist
  3/3 nền tảng; metainfo + CHANGELOG + version 14 chỗ.

### Phát hành + xác minh máy chủ repo

- **v0.2.13** @ `a600d0f` (7 asset) · **v0.2.14** @ `83245cb` (retag sau fix
  Swift quick_telex): [release-candidate #37094883437](https://github.com/hunglinhpt/TextVN/actions/runs/37094883437)
  — 4/4 job, 7 asset; homebrew sha cập nhật (B7b).
- **Máy chủ repo:** cài `TextVN-setup-0.2.14` qua UAC — installer chạy trơn
  (rename-DLL không còn lỗi code 5), Program Files = 0.2.14.0, không sót file
  `.old`. **E2E gõ thật trong Notepad: `dduocj text ` → `được text ` — PASS**;
  tray + TIP + activate đều OK.

## Bản 0.2.12 — hết lỗi nâng cấp DLL khoá + Win+Space store + xác minh máy thật gõ lại được

**Chuỗi báo cáo 2026-10-03 (screenshot bộ cài):** "lỗi không replace được
profile cũ textvn-tsf.dll trong Program Files khi dùng bản cài installer;
dùng bản portable cũng không gõ được tiếng Việt".

### Ba nguyên nhân gốc, ba fixes, xác minh máy thật

1. **Installer DeleteFile code 5** — TSF nạp `textvn-tsf.dll` trong mọi tiến
   trình TSF-aware (explorer, Notepad…) nên `DeleteFile` bị Windows từ chối
   kể cả khi setup chạy elevated. Fix: `RenameLockedTsfDll()` trong
   `PrepareToInstall` — đổi tên DLL cũ thành `.old-<ts>` (Windows CHO PHÉP
   rename file đang map) rồi ghi file mới; `[UninstallDelete]` dọn `.old-*`.
2. **TextVN mất khỏi danh sách Win+Space** — `InstallLayoutOrTip` chỉ ghi
   store CTF; store ngôn ngữ hiện đại (nguồn Win+Space/settings) là chỗ khác
   và bị mất sau các chu kỳ unregister→register. Fix (0.2.12): `register` ghi
   `HKCU\Control Panel\International\User Profile\<tag>` + broadcast
   `WM_SETTINGCHANGE("International")`.
3. **Windows 26300 (24H2+/Insider) từ chối ActivateProfile cho TIP chỉ-HKCU**
   — bắt thật trên máy chủ repo: mọi combo API (VI/EN × cờ, kể cả legacy
   ActivateLanguageProfile) đều E_FAIL/E_INVALIDARG khi chưa có đăng ký HKLM;
   trước đây máy này "gõ được" là nhờ HKLM từ bản cài 0.2.6 (sau khi gỡ bản
   cài → chỉ còn HKCU → chết). Fix vận hành: bộ cài phạm vi máy (admin) —
   đúng luồng CI /ALLUSERS kiểm chứng; ghi thành B7 trong
   `win-test-common-errors.md`; `activate` giờ check-then-activate qua
   `GetActiveLanguageProfile` (profile ĐANG CHỌN, không phải "được phép") +
   ma trận thử + thông báo lỗi nêu rõ việc cần làm.

**Xác minh cuối trên máy chủ repo (sau khi cài 0.2.12 machine-wide):**
E2E gõ thật trong Notepad — `dduocj text ` → `được text ` (Telex + EN-restore,
NFC-normalized) **PASS**; hotkey Ctrl+Shift lật state `True→False→True`
**PASS**; `activate` sau toggle **OK**; HKLM COM = `C:\Program Files\TextVN\textvn-tsf.dll`;
HKCU Run = `C:\Program Files\TextVN\TextVN.exe --autostart`.

**Ghosts dọn trong quá trình chẩn đoán:** explorer giữ DLL 0.2.3 từ OneDrive
Desktop (từ 30/09) — "app cũ ảnh hưởng app mới" thật; autostart Run trỏ
folder 0.2.7 đã xoá; Program Files kẹt DLL 0.2.6. Khuyến nghị: gỡ các thư mục
portable cũ sau khi đã cài bản machine-wide.

### Phát hành — v0.2.12 đã publish

- **Tag:** `v0.2.12` @ `2cd0020`. release-candidate — **4/4 job `success`**,
  7 asset. Setup: `TextVN-setup-0.2.12-windows-x64.exe` (đã dùng để sửa máy
  chủ repo). homebrew sha cập nhật (B7b).

## Bản 0.2.10 + 0.2.11 — toggle = activate bộ gõ, migration nâng cấp, publish hardening

**Phản hồi chủ repo trên 0.2.9:** (1) "đã chuyển EN mà vẫn gõ tiếng Việt";
(2) nâng cấp app mới vẫn bị app cũ ảnh hưởng — phải dọn sạch, chỉ giữ từ điển
user thêm; (3) Partner Center giờ chỉ nhận exe/msi; (4) cert đến từ luồng
chứng nhận Store.

### Nguyên nhân + fix + bằng chứng

- **(1)** Khi bộ gõ active trong app là bàn phím khác (MS Việt / US trong
  Win+Space), toggle TextVN chỉ đổi mode + icon — thứ user gõ do bàn phím cũ
  quyết định. Fix: tray chạy `textvn-cli activate` (subcommand mới —
  ActivateProfile profile VI cho phiên) SAU mỗi lần toggle hotkey → mode đổi
  là bộ gõ active đổi. Bắt thêm: ActivateProfile trên profile ĐANG bật trả
  E_FAIL (0x80004005, thật máy chủ repo) → `activate` check-then-activate
  (0.2.11). Kiểm chứng máy user: `activate` exit 0, "đã là bộ gõ active".
- **(2)** Diverge state tray↔TIP (bấm đôi trong 250ms, tray restart…): tray
  trả SNAPSHOT state chuẩn cho ToggleViEn (kể cả bị debounce bỏ qua), TIP áp
  lại; thêm claim 250ms phía TIP khớp cửa sổ tray. Test:
  `toggle_global_responds_with_authoritative_snapshot` +
  `hotkey_toggle_claim_debounces` (hermetic — cell/đồng hồ riêng, hết flake
  CI run 37057437635).
- **(3)** `state.json` ghi `last_version` — đổi phiên bản kích hoạt migration
  lúc khởi động: reset tuỳ chọn về mặc định, xoá per-app overrides, bật lại
  global, **unregister+register lại TSF**. GIỮ: từ điển EN (`english_words`),
  gõ tắt (`macros`), emoji. Tests ×2 (migrate + no-op cùng phiên), 3× pass.
  Máy user xác nhận: last_version=0.2.11, global=true, apps={} sau nâng cấp.
- **(4)** `store-submission.md` §6 viết lại: exe/msi only, chứng thư theo
  luồng Store, SignPath/cert riêng = tuỳ chọn cho phân phối trực tiếp.

### Sự cố phát hành mới (R4) — publish fail im lặng

Tag v0.2.11 lần đầu bị push mà **quên bump version** → `test tag = Cargo.toml`
fail im lặng, artifact vẫn 0.2.10. Publish job giờ in
`FATAL: tag != version…` rõ ràng; R4 ghi vào incidents. Đúng luồng: bump →
preflight 18/18 → commit → CI xanh → tag → publish.

### Phát hành — v0.2.10 & v0.2.11 đã publish

- **v0.2.10** @ `3bcf785`+`f625023` (fix flake claim): 4/4 job, 7 asset.
- **v0.2.11** @ `bbeed9f` (activate check-then-activate + R4 hardening):
  [release-candidate #37065791246](https://github.com/hunglinhpt/TextVN/actions/runs/37065791246)
  — 4/4 job, 7 asset. Portable
  `TextVN-portable-0.2.11-windows-x64-20261002211642.zip`
  SHA-256 `f6ed4c7d3fad3424…`; homebrew sha cập nhật (B7b).
- **Máy người dùng:** `D:\TextVN` = 0.2.11.0, doctor xanh, activate OK.

## Bản 0.2.9 — từ điển EN cá nhân + quy tắc ưu tiên + preflight 17 gate

**Review 3 lượt toàn bộ phần mới (yêu cầu chủ repo)** — notes:

- **Pass 1 (audit)**: secure field → strategy Passthrough → gợi ý/restore
  không bắn ✓; Linux cross-check không dead_code ✓; charset encode đổi độ
  dài → xoá đúng số glyph owned ✓; gap test `auto_restore_english=false`
  giữ fold kiểu UniKey (đã bổ).
- **Pass 2 (reviewer)**: wordlist editor tái dùng pattern macro editor
  (re-register class an toàn, modal owner lock, ctx poisoned → KHÔNG đóng
  cửa sổ khi lưu lỗi); close_word_list_editor trùng close_macro_editor —
  chấp nhận, đã note.
- **Pass 3 (user view)**: 3 smoke UI — mở/đóng editor, mở lại vẫn đúng 1
  cửa sổ, Lưu ghi đúng `english_words=['cowork','list']` vào config + broadcast
  reload; engine test user-dict thắng vn_common 3×; corpus 3 adapter xanh.
- **Findings đã xử lý**: kỳ vọng test Text/TEST sai (sửa test); ruby thiếu
  trên Windows local → preflight step SKIP, CI ubuntu chặn thật.

**Kiểm soát release chặt hơn** (yêu cầu: "CI phải xanh, fail quá nhiều lần"):

- `cargo xtask preflight` (+ `.cargo` alias): **17 gate mirror đúng ci-shared
  + repo-hygiene**, fail-fast, `Command::status` (exit thật — chặn bài học
  R1: pipe `| tail` che exit code khiến clippy đỏ 2 lần sau khi push).
  Chạy 3 lần trước khi tag: 17/17 PASS.
- Checklist A0 + bảng "bước → validate gì → sự cố thật" + 3 incidents mới
  (R1 pipe, R2 perf baseline chết theo runner, R3 alias) trong
  `docs/release/release-process.md`.
- Ma trận xung đột ngôn ngữ công khai: `docs/specs/language-detection.md` —
  nguyên tắc "mode đang bật thắng cặp mơ hồ", thứ tự secure → chord →
  macro → mode → gợi ý → restore → Escape, cách đổi ý từng trường hợp.

### Phát hành — v0.2.9 đã publish (release candidate, chưa ký số)

- **Tag:** `v0.2.9` @ `6fd12e3`. Workflow release-candidate — **4/4 job
  `success`**, 7 asset.
- **Portable ZIP:** `TextVN-portable-0.2.9-windows-x64-20261002191153.zip`
  SHA-256 `06de3b171c7ba259…` (đầy đủ trong `SHA256SUMS.txt`).
- **homebrew (B7b):** sha256 cập nhật theo zip macOS v0.2.9 ngay sau publish.
- **Máy người dùng:** `D:\TextVN` cập nhật từ ZIP release 0.2.9, tray chạy,
  TIP OK, FileVersion 0.2.9.0.

## Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent

**Bối cảnh (yêu cầu 2026-10-02):** (1) gõ tiếng Việt xong rồi gõ tiếng Anh bên
cạnh thì sai — cần engine tự xác định ngôn ngữ + gợi ý Tab + Escape cứu;
(2) dialog di chuyển giữa màn khác độ phân giải bị lệch; (3) đổi thông tin
phát triển thành LinhBH.CoM; (4) chuẩn bị nộp Microsoft Store (cài im lặng);
(5) PRIVACY_POLICY cho phần khai báo Store.

### Giải pháp + bằng chứng kiểm thử

- **Tự xác định EN/VI**: `data/en_common.txt` (từ EN thông dụng mà Telex biến
  thành âm tiết Việt hợp lệ — text→tẽt, is→í, saw→să…) + `data/vn_common.txt`
  (lưới bảo vệ cặp mơ hồ: cow=cơ, sex=sẽ, queen=quên — GIỮ tiếng Việt, đúng
  UniKey). Oracle test bắt buộc mọi mục en_common phải "bắn" (fold hợp lệ +
  không đụng vn_common). Corpus: `restore_en_vowel_w_01` (bug thật của user),
  `restore_en_vn_protected_01`, `escape_restore_raw_01` — replay 3 adapter
  xanh (win 117 · mac 153 · tsf 117).
- **Tab gợi ý**: `english_tab_complete_01` — "tes" + Tab → "test". Macro vẫn
  ưu tiên; Shift+Tab không đụng (S9); từ chưa biến đổi → Tab đi qua như cũ.
- **DPI đa màn hình**: DIALOG_LAYOUT/MACRO_LAYOUT thành const có ID riêng cho
  static/groupbox; `WM_DPICHANGED` → resize theo RECT đề xuất + re-layout +
  font recreate theo DPI. Smoke test mở dialog OK.
- **Store silent**: `PrivilegesRequiredOverridesAllowed=commandline` —
  `/CURRENTUSER` giờ là cờ thật (trước Inno bỏ qua). Tham số Partner Center:
  `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART (+/CURRENTUSER)` — chi tiết
  `docs/release/store-submission.md`. CI đã chạy 2 kịch bản silent (per-user +
  machine) từ trước.
- **PRIVACY_POLICY.txt** (EN+VI) ở gốc repo — xử lý phím 100% cục bộ, không
  telemetry; link cho phần khai báo Store.
- **Branding LinhBH.CoM**: About/publisher/CompanyName/copyright toàn platform
  (URL repo + ID io.github.hunglinhpt giữ nguyên); `build-release.ps1` assert
  CompanyName khớp — ZIP 0.2.8 xác nhận CompanyName=LinhBH.CoM.

### Sự cố trong phiên — tra trước khi lặp lại

- **perf regression đỏ**: baseline 2026-09 chết theo runner image mới (đỏ trên
  MỌI commit e6e4ca8→be2489f; bisect worktree commit cũ xác nhận không phải
  code mới). Re-record `perf/baseline-win.json` từ số đo CI run 37045279327.
- **clippy pipe che exit code**: chạy `cmd | tail` trong chuỗi `&&` KHÔNG dừng
  khi cmd lỗi (không có `set -o pipefail`) — 2 lần commit đẩy rồi mới thấy đỏ.
  Bài học: set pipefail hoặc không pipe các lệnh gate.

### Phát hành — v0.2.8 đã publish (release candidate, chưa ký số)

- **Tag:** `v0.2.8` @ `27a2e11`. Workflow
  [release-candidate #37048171222](https://github.com/hunglinhpt/TextVN/actions/runs/37048171222)
  — **4/4 job `success`**, 7 asset.
- **Portable ZIP:** `TextVN-portable-0.2.8-windows-x64-20261002183333.zip`
  SHA-256 `3b5b0f74688e5847cd352369bd9cc2362c4e707e2ffcd81696257cdc881ab8a3`.
- **macOS ZIP:** `TextVN-macos-universal-v0.2.8.zip` SHA-256
  `2af5a9c7db1be679cd6628ba5ef98569f123b1f572a1f53d336e3f06d892a71d` — đã cập
  nhật `packaging/homebrew/textvn.rb` (bước B7b).
- **CI cùng commit (`27a2e11`):** `ci-shared` ✅ (perf xanh sau khi baseline
  re-record), `repo-hygiene` ✅.
- **Máy người dùng:** `D:\TextVN` cập nhật từ ZIP release 0.2.8, tray chạy,
  TIP OK, Ctrl+Shift tap xác nhận lật state (VN ON).

## Bản 0.2.7 — Ctrl+Shift đổi mode + icon toàn cục

**Bối cảnh:** báo cáo "Ctrl+Shift trên 0.2.5 không đổi được mode và icon".
Nguyên nhân: sau khi gỡ hook ở 0.2.4, đường toggle duy nhất nằm trong TIP
in-process — chỉ chạy khi TextVN LÀ bộ gõ active; đứng ở bàn phím US/Microsoft
Việt trong Win+Space thì không ai nhận tổ hợp.

**Giải pháp:** tray cài `WH_KEYBOARD_LL` **chỉ quan sát** (không ăn phím,
không inject, mọi event qua `CallNextHookEx`) dò tap Ctrl+Shift ở mọi app;
`try_claim_global_toggle()` (cửa sổ 250ms chéo nguồn) khoá mọi đường toggle
toàn cục — hết E11 double-toggle. Policy AV A3 cập nhật tương ứng.

### Kiểm chứng E2E trên máy thật (2026-10-02, sau khi publish)

Bơm Ctrl+Shift tap qua `keybd_event` (event injected — hook quan sát không
lọc) và đọc `state.json` sau từng tap:

- Tap đơn: `True → False → True` — mỗi tap lật đúng một lần (bản 0.2.7 cài
  tại `D:\TextVN`, tray + TIP đang chạy).
- Double-tap 80ms: chỉ lật MỘT lần (debounce chéo nguồn OK).
- `Layout Hotkey = 3` (Windows không nuốt Ctrl+Shift).

### Phát hành — v0.2.7 đã publish (release candidate, chưa ký số)

- **Tag:** `v0.2.7` @ `e6e4ca8`. Workflow
  [release-candidate #36955880198](https://github.com/hunglinhpt/TextVN/actions/runs/36955880198)
  — **4/4 job `success`** (typing smoke gồm case `Ctrl+Shift -> EN`/`VN` PASS).
- **Release:** [TextVN 0.2.7](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.7)
  — 7 asset. Portable ZIP: `TextVN-portable-0.2.7-windows-x64-20261002023020.zip`
  SHA-256 `55d375830a937b16f3bc99135be80eb5ac35826d9e187de3320777e73076edc3`.
- **CI cùng commit (`e6e4ca8`):** `ci-shared` ✅ (run 36955010527),
  `ci-macos` ✅, `repo-hygiene` ✅.
- **macOS ZIP** `TextVN-macos-universal-v0.2.7.zip` SHA-256
  `bbc106ecb61559b71e13ef4977da3c648e318b83e3e5c2a790b9882dfcc4560b`.
- **Sửa lỗi quy trình phát hiện khi review:** `packaging/homebrew/textvn.rb` giữ
  `sha256` cũ từ 0.2.5 (chưa từng cập nhật) — đã sửa về hash 0.2.7 và thêm bước
  **B7b** vào `release-process.md` (cập nhật hash mỗi bản) để không lặp lại.
- **Nâng cấp máy người dùng:** `D:\TextVN` cập nhật lên 0.2.7 bằng
  rename-then-copy (DLL đang được shell nạp — Windows cho rename file đang
  map, không cho ghi đè); doctor xanh, FileVersion 0.2.7.0.

## Bản 0.2.6 — chốt đăng ký TSF VI+EN, hết ghost registration

**Bối cảnh:** báo cáo "không gõ được tiếng Việt (kể cả admin) + quá nhiều
keyboard layout" trên 0.2.5. Chẩn đoán trên registry thật của máy người dùng:

1. **Ghost registration**: `InprocServer32` trỏ vào thư mục portable **đã bị
   xoá** → TSF không nạp TIP; admin không cứu được (HKCU override HKLM).
   `register status` báo `FILE MISSING` đúng; chạy lại `TextVN.exe` từ thư mục
   còn tồn tại tự sửa.
2. Hai thực nghiệm layout trong 0.2.6-dev đều vỡ và được CI bắt (raw typing):
   UNINSTALL-dedupe làm Windows deactivate TIP (5269ae1); bỏ layout EN mặc
   định khiến app en-US không nhận TextVN (fecf245). Cả hai đã revert —
   **VI + EN DEFPROFILE là thiết kế chốt**, comment tại chỗ chống lặp lại.
   Win+Space hiển thị 4 layout (US · VI MS · VI TextVN · EN TextVN) là thiết
   kế: profile EN cần cho ngữ cảnh nhập tiếng Anh.

### Phát hành — v0.2.6 đã publish (release candidate, chưa ký số)

- **Tag:** `v0.2.6` @ `e5a66f1`. Workflow
  [release-candidate #36951325000](https://github.com/hunglinhpt/TextVN/actions/runs/36951325000)
  — **4/4 job `success`** (typing smoke 9 case × Notepad/WordPad PASS).
- **Release:** [TextVN 0.2.6 (release candidate)](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.6)
  — 7 asset. Portable ZIP: `TextVN-portable-0.2.6-windows-x64-20261002013157.zip`
  SHA-256 `4d10fb426d6973d897842a2dfde7516f4b449a1e026183183508dd7a5ce52fde`.
- **CI cùng commit code (`e5a66f1`):** `ci-shared` ✅, `ci-macos` ✅,
  `repo-hygiene` ✅.
- **Sửa máy người dùng ngay sau publish:** unregister ghost → extract
  portable 0.2.6 vào `D:\TextVN` (thư mục ổn định, không suffix build-id) →
  `register` exit 0, `ActivateProfile(VI, session) → OK`, display-attr
  category per-user hiện diện, tray chạy từ `D:\TextVN\TextVN.exe`,
  doctor: IPC listening + Tray running + TIP registered + Ctrl+Shift dành
  cho TextVN.

## Bản 0.2.5 — vá bản portable (gạch chân) + pipeline ký SignPath

**Bug portable (repro thật trên máy không admin, 2026-10-02):** sau `register`,
cây `HKCU\Software\Microsoft\CTF\TIP` chỉ còn `LanguageProfile` —
`InstallLayoutOrTip` viết lại cây CTF và **xoá mất toàn bộ key `Category`** ghi
trước nó → display-attribute provider không bao giờ được app hỏi → portable
vẫn gạch chân dù TIP có `TF_LS_NONE`. Fix: `RegisterCategory` qua API được thử
độc lập (TSF thường OK non-admin — trước đây dừng sớm ở RegisterProfile), và
khi API thất bại, category HKCU ghi **SAU ILOT**. Verified live: cả 4 category
(kèm `{046B8C80-…}`) sống sót sau ILOT; `register` exit 0, `register status`
OK trên tài khoản thường; probe test đăng ký/dỡ đăng ký sạch.

### Ký số — SignPath opt-in

- Nghiên cứu các đường free: [code-signing-plan.md](code-signing-plan.md) —
  **SignPath Foundation** (miễn phí OSS, khuyến nghị; Azure Artifact Signing
  $9.99/tháng nhưng individual chỉ US/CA; SSL.com/Certum trả phí).
- Pipeline: `tools/win/sign-signpath.ps1` (REST SignPath, skip khi không có
  secret) + `build-release.ps1` ký 3 binary trước khi đóng ZIP và setup exe
  sau ISCC khi `SIGNPATH_*` secrets được cấu hình; `release.yml` truyền secrets
  vào job windows. Không secret → build như cũ (candidate chưa ký).

### Phát hành — v0.2.5 đã publish (release candidate, chưa ký số)

- **Tag:** `v0.2.5` @ `bab5f7c+` (code `a6a5572`). Workflow
  [release-candidate #36905101277](https://github.com/hunglinhpt/TextVN/actions/runs/36905101277)
  — **4/4 job `success`**; publish validate `RELEASE_REPORT.json`.
- **Release:** [TextVN 0.2.5 (release candidate)](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.5)
  — 7 asset. Checksums pin (từ `SHA256SUMS.txt`):

| Asset | SHA-256 |
|---|---|
| `TextVN-portable-0.2.5-windows-x64-20261001181333.zip` | `c85dde7755375c1798dd3f9cc1ed023b559712ece0ff1f938859e8d8fdf13f5e` |
| `TextVN-setup-0.2.5-windows-x64.exe` | `658c6fc4537dbde8d2e7591a5bf51e8ffbd98bae68e8bbb51f5c7476eaabe24b` |
| `TextVN-0.2.5-linux-x86_64.tar.gz` | `694fad017ba047f93bd1e89b0f335725c4b04947d85424bd2e7b1f621393e485` |
| `TextVN-mac-v0.2.5.pkg` | `c1b99d3a8ea2c6dc11e34917180bb30e47a81547485ffb87ef20b12549872e11` |
| `TextVN-macos-universal-v0.2.5.zip` | `e5cee948668f4cccdb8a1f9594ed4a980d385ef1eb5f2960953055a0c400eba6` |

- **CI cùng commit code (`a6a5572`):** `ci-shared` ✅ (portable scenario chạy
  đúng đường category mới), `ci-macos` ✅, `repo-hygiene` ✅ `adcbfd2`.

## Bản 0.2.4 — đăng ký không cần admin + clean composition + Ctrl+Shift single-path

**Bối cảnh:** 3 bug người dùng báo trên bản 0.2.3/0.2.4-dev — (1) phải chạy
bằng admin mới đăng ký được TSF, (2) gõ chữ bị gạch chân, (3) bấm Ctrl+Shift
không đổi mode/icon. Vòng deep-scan tìm ra 3 nguyên nhân gốc còn sót sau nhóm
fix đầu (96364a9..d785ea7) và đã sửa:

| Bug | Nguyên nhân gốc thật | Fix |
|---|---|---|
| Đăng ký cần admin | `.iss` gọi `register --scope machine` vô điều kiện — cài per-user (không UAC) CLI trả exit 3 → "Windows từ chối đăng ký" | Nhánh `IsAdminInstallMode`: per-user chỉ `register`; admin machine + `ExecAsOriginalUser` |
| Gạch chân | `CAT_DISPLAY_ATTRIBUTE_PROVIDER` ghi sai GUID `{2464BEB0-…}` (thật: `{046B8C80-…}`) — fallback per-user đăng ký category rác | Sửa GUID + test regression chốt giá trị |
| Ctrl+Shift "chết" | **Double-toggle**: TSF in-process (`ModifierToggle`) + tray `WH_KEYBOARD_LL` (thêm ở 96364a9) cùng bắn 1 lần bấm = 2 lần toggle; hook vi phạm AV policy A2/A3 | Xoá hook + `WM_TOGGLE_HOTKEY`; toggle single-path in-process, tray đổi icon qua IPC |

### Phát hành — v0.2.4 đã publish (release candidate, chưa ký số)

- **Tag:** `v0.2.4` @ `bab5f7c`. Workflow
  [release-candidate #36899649686](https://github.com/hunglinhpt/TextVN/actions/runs/36899649686)
  — **4/4 job `success`** (windows · linux · macos · publish), publish validate
  `RELEASE_REPORT.json` (version=tag, `source_tree_clean=true`,
  `feature_profile=tsf-only`).
- **Release:** [TextVN 0.2.4 (release candidate)](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.4)
  — 7 asset. Checksums pin (từ `SHA256SUMS.txt`):

| Asset | SHA-256 |
|---|---|
| `TextVN-portable-0.2.4-windows-x64-20261001172918.zip` | `a876635ce7c71e838cee844137b6c81c317da9b8fe6481c125e44c0177708a82` |
| `TextVN-setup-0.2.4-windows-x64.exe` | `96c3f258d058c921a5346249a71a0e815d55f51bfa25b84ad069630231067265` |
| `TextVN-0.2.4-linux-x86_64.tar.gz` | `eb58d8d59346b1b393734d926c3c343f201893d257a2ccdc3f3af0fcf04d8947` |
| `TextVN-mac-v0.2.4.pkg` | `027da7c9541cad437ab8d75407fe8f6277e78fe9dc49ead73327ac9a4551ca90` |
| `TextVN-macos-universal-v0.2.4.zip` | `402b62145bcf40b72e91faa9d2fe327184b9420c5c4f3cd7fb2c8db5037ad6d3` |

- **CI cùng commit phát hành:** `ci-shared` ✅ (job Windows package chạy
  **2 kịch bản installer**: per-user không admin + `/ALLUSERS` machine; typing
  smoke 9 case × Notepad/WordPad xanh qua cơ chế poll mới), `ci-macos` ✅,
  `repo-hygiene` ✅. Job `perf regression` đỏ là `continue-on-error` (nhiễu
  runner dùng chung), không chặn.
- **Bài học mới:** E11 (double-toggle — không cài keyboard hook trong tray; **chính sách này được thay ở 0.2.7**: hook quan sát + debounce — xem mục 0.2.7 ở trên),
  E12 (per-user installer + blind spot /ALLUSERS trong CI),
  [win-test-common-errors](../specs/win-test-common-errors.md) B6 (typing
  harness poll ổn định thay vì đọc 1 lần).

## Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon)

**Mục tiêu bản này:** sau khi v0.2.2 phát hành, user yêu cầu audit thêm 2 vòng
từng dòng code toàn repo dưới view chuyên gia (code + audit + GitHub Actions),
kiểm icon UI Linux/macOS so với Windows, và viết lại quy trình release chuẩn
hoá. Kết quả audit vòng 1 (3 luồng độc lập): **0 blocker, 3 major Rust
(register), 5 major CI, 3 major icon** — tất cả đã fix; vòng 2 là verify độc
lập trên diff trước khi commit.

### Icon — trả lời câu hỏi "Linux/macOS đã có icon như Windows chưa?"

| Nền tảng | Trước 0.2.3 | Sau 0.2.3 |
|---|---|---|
| Windows | ✅ tray 2 trạng thái + PE icon + TSF IconFile; ❌ cửa sổ cài đặt + installer dùng icon mặc định | ✅ đầy đủ: `tray.rc` ID 3 + `hIcon` class + `SetupIconFile` |
| Linux | ✅ cài chuẩn có đủ (hicolor SVG + window icon + panel); ❌ **portable không icon**; thiếu PNG cho panel cũ | ✅ portable stage hicolor vào `$XDG_RUNTIME_DIR` + `XDG_DATA_DIRS` cho daemon; thêm PNG 128 hicolor + AppStream `<icon>` |
| macOS | ❌ **không có icon nào**: 3 plist khai báo `CFBundleIconFile` nhưng không có `.icns` trong repo — pipeline `build-macos.sh` có sẵn nhưng thiếu PNG nguồn nên âm thầm tắt | ✅ `packaging/macos/icons/TextVN-1024.png` (+512, render từ SVG brand bằng `scripts/generate_app_icons.py`) → `.icns` vào cả 2 bundle + `tsInputMethodIconFileKey` cho input menu |

### Sửa theo audit vòng 1 — đăng ký TSF (3 major + 7 minor)

- Lỗi thiếu DLL giờ ghi vào `register.log` (trước đây chỉ stderr → dialog đọc
  nhầm đuôi log run cũ, advice sai nguyên nhân).
- Profile EN chuyển best-effort: Windows Single Language không còn bị fail cả
  đăng ký; hậu kiểm bắt buộc VI.
- `--scope machine` xoá override CLSID per-user cũ (trỏ DLL đã xoá) trước khi
  ghi HKLM; `server_ok` dò cả 2 hive.
- `unregister` trả exit 1 khi còn key sót; machine scope dọn thêm CTF TIP HKLM;
  `Enable=1` ghi lại sau `InstallLayoutOrTip`; `FreeLibrary` input.dll; doctor
  tìm `data/` cạnh exe; `save_macros` không còn coi lỗi khóa nội bộ là thành
  công; FFI dùng hằng `IME_FIELD_SECURE` + doc contract `ime_last_error`.

### Sửa theo audit vòng 1 — GitHub Actions (5 major + minors)

`ci-shared`: `permissions: contents: read` · `cancel-in-progress` chỉ PR ·
`paths-ignore` docs/graphify · timeout 12 job · Swatinem/rust-cache ·
`persist-credentials: false` — `release.yml`: publish **draft-first**
(upload `--clobber` rerun được, public sau cùng) · jq assert
`feature_profile=tsf-only` · macOS test-trước-package · `ci-macos`: bỏ
`needs` vô ích + timeout — `repo-hygiene`/`targets-verify`: timeout/token
chỉ đọc/hostname allowlist.

### Quy trình phát hành mới

`docs/release/release-process.md` (quy tắc **G16** `00-WORKFLOW.md` §12.4) —
Checklist A (local: fmt/clippy/test/version-sync/replay/verify/docs-check) →
Checklist B (CI xanh rồi tag, bộ docs bắt buộc, verify release + checksums,
bằng chứng CI). Chuẩn hoá từ v0.2.0–v0.2.2 đã phát hành thành công, kèm bảng
sự cố (flake B5, perf continue-on-error, draft rerun…).

### Kiểm chứng chạy trên host Windows (2026-10-01, sau toàn bộ thay đổi)

| Gate | Kết quả |
|---|---|
| `cargo fmt --all -- --check` | ✅ sạch |
| `cargo clippy --workspace --exclude textvn-win-hook --all-targets -- -D warnings` | ✅ 0 warning |
| `cargo test --workspace` | ✅ **343 test pass** |
| `cargo run -q -p xtask -- check-version-sync` | ✅ 14 chỗ = 0.2.3 |
| `check-tables` · `check-mac-corpus` (114) · `check-mac-targets` (12) | ✅ |
| `cargo run -q -p textvn-cli -- verify` | ✅ ABI v1 ↔ header khớp, 11 export |
| `replay corpus/shared corpus/mac --adapter mac` | ✅ 149/149 |
| `replay corpus/shared corpus/win --adapter win` · `--adapter tsf` | ✅ 113/113 ×2 |
| `check_doc_links` (39 link) · `check_no_injection_apis` (158 file) | ✅ |
| YAML 6 workflow · `bash -n` 5 script packaging | ✅ |
| Audit vòng 2 (verify độc lập trên diff) | ✅ **ĐẠT để commit** — 0 blocker; 1 major (`&&` short-circuit bỏ qua delete kế tiếp trong `do_unregister`) + 3 minor đã sửa inline ngay trước commit |

### Phát hành — v0.2.3 đã publish (release candidate, chưa ký số)

- **Tag:** `v0.2.3` @ `ae33d7a`. Workflow
  [release-candidate #36766099152](https://github.com/hunglinhpt/TextVN/actions/runs/36766099152)
  build + test cả 3 nền tảng rồi publish: **4/4 job `success`** (windows ·
  linux · macos · publish) — publish job validate `RELEASE_REPORT.json`
  (`version=0.2.3`, `git_commit`=tag, `source_tree_clean=true`,
  `feature_profile=tsf-only`) và ZIP mặc định không có legacy hook.
- **Release:** [TextVN 0.2.3 (release candidate)](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.3)
  (pre-release, draft-first) — 7 asset. Checksums pin (từ `SHA256SUMS.txt`):

| Asset | SHA-256 |
|---|---|
| `TextVN-portable-0.2.3-windows-x64-20260930193029.zip` | `e1452a2125bc90a74105e41f9e345c5970adc5db1aeafef685df796e42ef0309` |
| `TextVN-setup-0.2.3-windows-x64.exe` | `4a37b1e9be4a0333ce2154b0b72fd47cd219e36bcdaa7c248982fff672b1ea5e` |
| `TextVN-0.2.3-linux-x86_64.tar.gz` | `399b2cafb9ea93206ddb20269cac2cfda088520b62aada4e9a353f3420c1f123` |
| `TextVN-mac-v0.2.3.pkg` | `d0fb50875bbc8e5a267a78c8a510b1970689984dccc805093f31d088b435cd00` |
| `TextVN-macos-universal-v0.2.3.zip` | `6ecaa26d989a458f47c0bca9025cb832f08f13c0048ad7e3a9a42aca3839cb3c` |

- **CI cùng commit phát hành (`ae33d7a`):** `ci-shared` ✅ (rerun sau khi job
  typing smoke "Windows package" đỏ chớp — wordpad desync, đúng B5;
  `Linux IBus/Fcitx5` ✅ với icon portable mới), `ci-macos` ✅, `repo-hygiene` ✅.
  Job `perf regression` đỏ là `continue-on-error` theo thiết kế (nhiễu runner),
  không làm đỏ run.
- **Bài học ghi sổ trong đợt này (E9/E10 + sự cố release):** cross-check
  `--target x86_64-unknown-linux-gnu` bắt lỗi cfg trước push; thứ tự bắt buộc
  job macos release: build-macos.sh → swift test → package; backtick trong
  `git commit -m` bị Git Bash nuốt — dùng `git commit -F`.

## Bản 0.2.2 — vá F3-13 (mở nhầm Cài đặt lúc login) + F3-8, nit

**Mục tiêu bản này:** 0.2.1 đã cố vá F3-13 nhưng còn sót — với bản **chưa ký số**
đúng trạng thái release hiện tại, `SMAppService.mainApp.status` là
`.requiresApproval` dù login item vẫn chạy, nên TextVN hiểu là khởi động tay và
**bật cửa sổ Cài đặt mỗi lần đăng nhập**. Đây là lỗi được ưu tiên cao nhất và
đã sửa tận gốc. Đồng thời đóng nốt F3-8 (menu bar lệch spec) và nhóm nit.

### Kiểm chứng chạy trên host Windows (trước commit)

| Gate | Kết quả |
|---|---|
| `cargo fmt --all -- --check` | ✅ sạch |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warning |
| `cargo test --workspace` | ✅ **342 test pass** |
| `cargo run -q -p xtask -- check-version-sync` | ✅ 14 chỗ = 0.2.2 |
| `check-tables` · `check-mac-corpus` (114) · `check-win-corpus` (72) · `check-mac-targets` (12) | ✅ |
| `replay corpus/shared corpus/mac --adapter mac` | ✅ 149/149 |
| `replay corpus/win --adapter win` | ✅ 78/78 |
| `cargo run -q -p textvn-cli -- verify` | ✅ ABI v1 ↔ header: `ime_key_v1=20`, `ime_result_v1=532`, offsets + 11 export khớp |

### Bổ sung sau commit `ec1ac2b` — triệt để hóa gợi ý lỗi đăng ký TSF (ảnh lỗi của người dùng)

Vòng 11 đã sửa hộp thoại “Cài & bật TSF” hiện **lý do thật** từ đuôi
`register.log`, nhưng còn sót một lỗ: log CLI in lỗi registry chỉ dạng hex
`FAIL 0x00000005` không kèm tên ký hiệu, nên nhánh gợi ý `ACCESS_DENIED` là
**dead code** — đúng ca lỗi trong ảnh người dùng gửi (Windows từ chối đăng ký
bộ gõ ở `HKCU\Software\Classes\CLSID`, sandbox/AV/policy chặn ghi) vẫn luôn
rơi vào gợi ý generic. Đã sửa 2 đầu:

- `cli/src/register.rs`: dòng log registry in đủ
  `Registry create HKCU\…\CLSID → FAIL 0x00000005 (ERROR_ACCESS_DENIED)`
  (bảng `lstatus_name` cho các mã registry thường gặp), dòng FAIL nêu đúng
  khoá bị chặn kèm lệnh/API gây ra.
- `tray/src/settings_dialog.rs`: `advice_for_failure` khớp **cả hai** định dạng
  log — bản mới có tên ký hiệu, bản 0.2.0/0.2.1 đã cài trên máy người dùng chỉ
  có hex `0x00000005`.

Kiểm chứng lại toàn bộ gate trên host Windows sau thay đổi (2026-09-30):

| Gate | Kết quả |
|---|---|
| `cargo fmt --all -- --check` | ✅ sạch |
| `cargo clippy --workspace --exclude textvn-win-hook --all-targets -- -D warnings` | ✅ 0 warning |
| `cargo test --workspace` | ✅ **343 test pass** (342 + 1 test mới `lstatus_name_covers_acl_and_common_registry_errors`) |
| `cargo run -q -p xtask -- check-version-sync` | ✅ 14 chỗ = 0.2.2 |
| `cargo run -q -p textvn-cli -- verify` | ✅ ABI v1 ↔ header khớp |
| `replay corpus/shared corpus/mac --adapter mac` | ✅ 149/149 |
| `replay corpus/shared corpus/win --adapter win` | ✅ 113/113 (35 shared + 78 win) |
| `replay corpus/shared corpus/win --adapter tsf` | ✅ 113/113 |
| Regression guard | ✅ `advice_matches_real_cause` mở rộng với đúng chuỗi log thật (bản mới + bản đã cài) |

Kiểm tra registry trên máy phát hành: `reg query` cho thấy TextVN TSF của
người dùng đang trỏ vào portable 0.2.0 cũ (`D:\New folder (2)\…`); máy cho ghi
`HKCU\Software\Classes\CLSID` (probe key tạo/xoá sạch) — xác nhận lỗi trong ảnh
là môi trường chặn ghi, không phải máy không đăng ký được, và bản 0.2.2 cài đè
sẽ repoint CLSID sang DLL mới khi bấm [Cài & bật TSF].

### Perf — job `perf regression` đỏ trên CI là **nhiễu runner**, không phải hồi quy

Job cố ý để `continue-on-error` (runner Windows GHA dùng chung, 2 vCPU; gate cứng
chạy trên self-hosted theo risk RW5 của P1-5 §1). Đo lại trên máy local:

| Phép đo | Baseline p50 | Hiện tại p50 | Chênh lệch |
|---|---|---|---|
| `ime_key` | 687 ns | **500 ns** | **−27,2%** |
| `parse_config` | 781 ns | 768 ns | −1,7% |
| `ime_strategy_resolve` | — | 10 ns | dưới ngưỡng đo |

Không có hồi quy sản phẩm; cấu hình job **không** được nới lỏng. Chi tiết và
ranh giới xác minh: [audit Vòng 11](cross-platform-audit-2026-09-30.md).

### Phát hành — v0.2.2 đã publish (release candidate, chưa ký số)

- **Tag:** `v0.2.2` @ `96083d5` (gồm fix đăng ký `291c6ed`). Workflow
  [release-candidate #36745978568](https://github.com/hunglinhpt/TextVN/actions/runs/36745978568)
  build + test cả 3 nền tảng rồi publish: 4/4 job `success` (windows · linux ·
  macos · publish). Job windows chạy đúng bộ smoke gõ thật (`test-portable.ps1`
  + `test-installer.ps1`) và **xanh**.
- **Release:** [TextVN 0.2.2 (release candidate)](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.2)
  (pre-release) — 7 asset, publish job đã kiểm `RELEASE_REPORT.json`
  (`version=0.2.2`, `git_commit` = tag, `source_tree_clean=true`,
  `status=release-candidate`) và ZIP mặc định không chứa legacy hook.
- **Checksums pin** (từ `SHA256SUMS.txt` đính kèm release):

| Asset | SHA-256 |
|---|---|
| `TextVN-portable-0.2.2-windows-x64-20260930164128.zip` | `f21ba90d373ef94a35c43a75825edb73cc3b263dc242f7d59f9fe4dd84305dcf` |
| `TextVN-setup-0.2.2-windows-x64.exe` | `4efe356920020988948d2e76be80b83229ef15cf5429215c8619ce17622b2bed` |
| `TextVN-0.2.2-linux-x86_64.tar.gz` | `95fe3b33d544261b4b07dfdbb390dae9bd2ea8fdb540a88a316d13da3c1cbfdc` |
| `TextVN-mac-v0.2.2.pkg` | `903befdbaf83a7564a71c1f05e35e1a520a535c7f102304b227e31c4a50dfadc` |
| `TextVN-macos-universal-v0.2.2.zip` | `907af7f71dfbc03fe31c5668ec96017e65d6ce505a86dab22ebe1a0c3664e198` |

- **CI cùng commit fix (`291c6ed`):** `ci-macos` ✅ (Swift arm64 build + test với
  fix mới), `repo-hygiene` ✅. `ci-shared` — mọi job Rust/test/replay/package
  xanh, riêng job **typing smoke "Windows package" đỏ chớp trên runner dùng
  chung** (symptom khác mỗi run: notepad nhận `[]`, wordpad desync; cùng commit
  rerun xanh, và release run chạy đúng bộ test này vẫn xanh) — đã ghi sổ
  [B5](../specs/win-test-common-errors.md). Job `perf regression` đỏ là
  `continue-on-error` theo thiết kế (nhiễu runner; đo local không hồi quy —
  bảng phía trên), **không** làm đỏ run.

### Trạng thái

- **Release candidate, không production.** Chưa có Authenticode (host không có
  code-signing certificate có private key), chưa có Developer ID, notarization.
- **Swift đã compile + test trên CI** cho commit fix (`ci-macos` ✅ `291c6ed`).
  GUI smoke trên máy Mac thật vẫn là **điều kiện production**.
- Người dùng cài bản này lên máy có TSF cũ trỏ DLL đã xoá (thấy qua
  `textvn-cli register status`: `COM server … (FILE MISSING)`): bấm
  **[Cài & bật TSF]** sẽ repoint CLSID sang DLL mới của bản cài.

## Bản 0.2.1 — phát hành pre-release đã xác minh

- Source/tag: `ca22eba2239d9736f51548169f0ce33417954f7a` / `v0.2.1`.
- CI cùng commit: [ci-shared #36718542260](https://github.com/hunglinhpt/TextVN/actions/runs/36718542260),
  [ci-macos #36718542232](https://github.com/hunglinhpt/TextVN/actions/runs/36718542232),
  [repo-hygiene #36718542268](https://github.com/hunglinhpt/TextVN/actions/runs/36718542268) —
  tổng thể đều `success`. Windows portable/installer gõ TSF thật, Linux
  IBus/Fcitx5 chạy package e2e, macOS Swift arm64+x86_64 build/test và đóng gói
  unsigned universal. Chưa có smoke GUI/macOS thật ngoài runner.
- [release-candidate #36719431897](https://github.com/hunglinhpt/TextVN/actions/runs/36719431897)
  build lại từ tag sạch; Windows/Linux/macOS/publish đều `success`.
  [GitHub Release v0.2.1](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.1)
  là **pre-release**, có sáu binary archive/installer + `SHA256SUMS.txt`.
- Checksum đã tải lại và so với digest trên GitHub; Windows portable ZIP
  `e5e5b4fb2062f1c18b5c444247d131276b560d95d7e801f336f92b0f8a9d93b3`,
  setup EXE `f6996d6fba97cf387119f25ddba6c216c9deda582987b9ea2301504b23e34dce`,
  macOS universal ZIP `728788ef1454d213505276b42efc0f897d08580d00f90dd030c428d9c4abbb4e`.
  Mã băm còn lại nằm trong asset `SHA256SUMS.txt` của release.
- Trạng thái: **release candidate, không production**. Chưa có Authenticode
  (host không có code-signing certificate với private key), Developer ID,
  notarization, macOS GUI smoke. Job `perf regression` tham khảo thất bại:
  runner Windows `parse_config` p50 781→1231 ns (+57,6%); đo local
  781→1050 ns (+34,4%). `ci-shared` cấu hình `continue-on-error` cho job này;
  cần đo đối chứng trên cùng phần cứng trước khi kết luận hồi quy sản phẩm.

Trạng thái: **release candidate** cho Windows 10/11 x64 và Linux x86_64 (IBus, Fcitx5).
macOS: mã nguồn đang ở beta; chưa có bằng chứng GUI/package production trên máy Mac thật.

> Audit tiếp diễn 2026-09-30: bảng kết quả CI bên dưới là **bằng chứng lịch sử cho
> commit `766029e`**, không chứng nhận checkout hiện tại hoặc bản ZIP mới. Các sửa
> Windows/Linux/macOS sau commit đó cần CI chạy lại và smoke GUI trên từng hệ điều
> hành. `RELEASE_REPORT.json` nằm trong mỗi ZIP Windows là nguồn trạng thái của
> **chính ZIP ấy**; nếu `status=release-candidate` thì không phát hành là production.
> Xem [audit đa nền tảng 2026-09-30](cross-platform-audit-2026-09-30.md)
> để biết các sửa sau commit lịch sử và phần native chưa xác thực.

Mọi số liệu dưới đây lấy từ CI (`.github/workflows/ci-shared.yml`, workflow `ci-shared`)
và từ máy dựng Linux. Cập nhật file này ở mỗi lần phát hành (xem
[developer-guide.md §5](../developer-guide.md#5-phát-hành)).

| | |
|---|---|
| Nhánh / commit kiểm | `claude/windows-input-method-upgrade-f8bwkv` @ `766029e` |
| Ghi chú | Các commit sau `766029e` chỉ sửa tài liệu và kịch bản test; CI của PR #1 chạy lại toàn bộ trên commit cuối |
| Lần chạy CI | [ci-shared #36458803476](https://github.com/hunglinhpt/TextVN/actions/runs/36458803476) — mọi job PASS (Windows package, Linux adapters…) trừ job hiệu năng tham khảo, xem §5 |
| Toolchain | Rust 1.98.1 stable · MSVC (windows-2022) · GCC/CMake (ubuntu-24.04) |
| Inno Setup | 6.5+ (bản dịch tiếng Việt đi kèm repo) |

## 1. Gói phát hành

| Gói | Tạo bởi | Nội dung |
|---|---|---|
| `TextVN-setup-0.1.0-windows-x64.exe` | `build-release.ps1 -BuildInstaller` | Cài per-user (không cần quyền quản trị), tự đăng ký TSF, khởi động cùng Windows, gỡ qua Settings → Apps |
| `TextVN-portable-0.1.0-windows-x64-<build>.zip` | `build-release.ps1` | `TextVN.exe`, `textvn-tsf.dll`, `textvn-cli.exe`, `install.ps1`/`uninstall.ps1`, `HUONG_DAN_SU_DUNG.txt`, `RELEASE_REPORT.json` |
| `TextVN-0.1.0-linux-x86_64.tar.gz` | `scripts/build-linux.sh` | adapter IBus + Fcitx5, `textvn-settings` (GTK4), `install.sh`, `uninstall.sh`, `textvn-portable.sh` |

Gói Windows mặc định **TSF-only** (không hook bàn phím toàn cục, không `SendInput`). Mã băm:
`SHA256SUMS-<build>.txt` (Windows) và `*.tar.gz.sha256` (Linux), đính kèm cùng artifact CI.

## 2. Kết quả kiểm thử

| Lớp | Phạm vi | Kết quả |
|---|---|---|
| Unit Rust (`cargo test --workspace`) | engine, config, ffi, TSF compose, tray, IPC… — Linux, macOS, Windows | 298 test trên Linux (Windows chạy thêm test riêng của tray/TSF) — **PASS** cả 3 OS |
| Từ vựng thật (`core/tests/common_words.rs`) | ~400 từ × Telex, VNI, dấu kiểu cũ, `uow`, dấu giữa từ, Caps Lock, chữ viết tắt | **PASS** |
| Corpus (`replay`) | 113 kịch bản × 5 mô phỏng adapter (headless, win, tsf, mac, linux) | **113/113 × 5 PASS** |
| ABI (`verify`, `sizes`) | header C ↔ Rust; 20/532 byte | **PASS** |
| fmt · clippy `-D warnings` (Linux + target Windows) · cargo-deny · REUSE · chặn API tiêm mã | | **PASS** |
| Fuzz smoke 60 s | `ffi_key`, `config_parse`, `appdb_parse` | **PASS** |
| Linux e2e (`scripts/e2e-linux.sh`) | ibus-daemon 1.5.29 và fcitx5 5.1.7 thật: gõ, focus-out, reset, Enter, Ctrl+Shift, `state.json`, gõ tắt bật/tắt, Caps Lock, ô mật khẩu | **PASS** |
| Bảng điều khiển Linux (`ctest`) | `settings_model`: vá từng khoá, gõ tắt, file hỏng | **PASS** |
| Hiệu năng (`textvn-bench`) | xem §5 | trong ngân sách |

### Kịch bản 1 — Cài đặt

| Nền tảng | Các bước được kiểm tự động | Kết quả |
|---|---|---|
| Windows (windows-2022, CI job *Windows package*) | Cài im lặng per-user bằng setup `.exe` → file đã cài, CLSID TSF, khởi động cùng Windows → **gõ thật qua TSF** (xem bảng gõ bên dưới) → gỡ im lặng → không còn file/đăng ký, chỉ giữ cấu hình người dùng | **PASS** |
| Linux IBus | `./install.sh` (per-user) → TextVN có trong danh sách bộ gõ → gõ qua ibus-daemon thật → `uninstall.sh` → không còn file nào ngoài cấu hình | **PASS** |
| Linux Fcitx5 | như trên với fcitx5 thật (profile Fcitx5 được thêm/bỏ TextVN) | **PASS** |

### Kịch bản 2 — Giải nén ra dùng luôn

| Nền tảng | Các bước được kiểm tự động | Kết quả |
|---|---|---|
| Windows | Giải nén zip → chạy `TextVN.exe` (tự đăng ký TSF từ thư mục giải nén) → **gõ thật qua TSF** → `uninstall.ps1` → hết đăng ký, cấu hình giữ nguyên | **PASS** |
| Linux IBus | Giải nén vào thư mục **chỉ đọc** → `./textvn-portable.sh` → gõ → `stop` → không ghi gì vào `~/.local` | **PASS** |
| Linux Fcitx5 | như trên | **PASS** |

### Gõ thật trên Windows (mỗi kịch bản, mỗi ứng dụng)

Kết quả dưới đây đúng cho **cả hai** kịch bản (giải nén dùng ngay và cài đặt) trong lần chạy CI ở trên.

`installer/windows/tests/test-typing.ps1` gửi phím bằng `SendInput` (VK + scan code như bàn
phím thật) vào **Notepad** (Win32 Edit — app IMM32 qua CUAS) và **WordPad** (RichEdit —
TSF-aware), rồi đọc lại nội dung:

| Case | Phím | Kỳ vọng | Notepad | WordPad |
|---|---|---|---|---|
| Telex cơ bản | `dduocj␣` | `được␣` | PASS | PASS |
| Chữ hoa đầu | `Vieetj Nam␣` | `Việt Nam␣` | PASS | PASS |
| `uow` | `nguowif␣` | `người␣` | PASS | PASS |
| Vị trí dấu | `cuar␣` | `của␣` | PASS | PASS |
| Tiếng Anh | `hello␣` | `hello␣` | PASS | PASS |
| Dấu câu | `Vieetj, Nam␣` | `Việt, Nam␣` | PASS | PASS |
| Enter + viết hoa đầu câu | `chaof⏎banj␣` | `chào⏎Bạn␣` | PASS | PASS |
| Tab | `tieengs⇥x␣` | `tiếng⇥x␣` | PASS | PASS |
| Home giữa từ | `chaof` Home `x␣` | `x␣chào` | PASS | PASS |
| Ctrl+Shift → E / → V | `as␣` | `as␣` / `á␣` | PASS | PASS |
| Caps Lock | `VIEETJ␣` | `VIỆT␣` | PASS | PASS |

## 3. Lỗi tìm ra nhờ kiểm thử thật (đã sửa trong bản này)

| Tìm bởi | Lỗi | Sửa |
|---|---|---|
| Gõ thật Windows (Notepad) | Dấu cách/dấu câu ra **trước** chữ (`␣được`): app IMM32 nhận kết quả composition sau phím không bị ăn | Ký tự ranh giới in được commit cùng từ (`compose.rs`) |
| Gõ thật Windows (Notepad) | Enter/Tab ra trước chữ, rồi (khi giữ phím ở pha test) mất hẳn Enter/Tab | App CUAS: commit từ ở `OnKeyDown` rồi trả phím gốc cho cửa sổ trong cùng process (`replay.rs`) |
| Gõ thật Windows (WordPad) | Ctrl+Shift không chuyển V/E ở app TSF-aware | `ITfKeyTraceEventSink` (`key_event.rs`) |
| Kịch bản cài đặt sau khi gỡ bản portable | Ctrl+Shift lúc được lúc không: phím tắt đổi bố cục của Windows chuyển đi mất TextVN | Tuỳ chọn "Dành Ctrl + Shift cho TextVN" (`tray/src/hotkey.rs`, bộ cài chọn sẵn) |
| Thử chỉ đăng ký trong vi-VN | App mới mở chạy bàn phím US, TextVN không hoạt động ngay sau khi cài | Giữ đăng ký cả en-US (Windows tiếng Anh) |
| `common_words.rs` | Đặt dấu sai (`cuả`, `nghiã`, `đựơc`…), `d` tự thành `đ`, `gi`/`qu`, `uow` | Viết lại theo quy tắc chính tả |
| e2e ibus/fcitx5 thật | Engine reset mỗi lần caret đổi, addon Fcitx5 không nạp, sai bus name IBus | `1e54df6` |
| Kịch bản cài Linux | ibus-daemon giữ cache registry, bản per-user không hiện sau đăng nhập lại | Xoá cache khi cài/gỡ |
| Dựng installer trên CI | Inno Setup không kèm bản dịch tiếng Việt → không biên dịch được | Kèm `Vietnamese.isl` |

## 4. Phạm vi chưa kiểm tự động

- Windows: Chrome/Edge/Electron, Microsoft Office, ô tìm kiếm Start, ứng dụng UWP chưa có
  kiểm thử gõ tự động (mô hình composition giống WordPad; kiểm tay trước khi phát hành).
- Linux: GNOME Shell/Wayland thật (CI dùng ibus-daemon/fcitx5 không có compositor);
  ứng dụng Qt/Electron.
- `production_blockers` trong `RELEASE_REPORT.json` của gói Windows: UI Automation gốc chưa tích
  hợp (cổng bảo mật dùng tín hiệu TSF trong process), DACL của named pipe chưa được kiểm
  định, bản CI **chưa ký Authenticode**.

## 5. Hiệu năng

`textvn-bench` (p50 mỗi từ gõ; ngân sách `ime_key` p99 < 0,5 ms):

| Đo | main | bản này | Ghi chú |
|---|---|---|---|
| `ime_key` (Linux, máy dựng) | 465 ns | 475 ns | +2 %: quy tắc đặt dấu/Caps Lock làm thêm việc, bù bằng tra bảng nguyên âm trực tiếp |
| `ime_key` (runner Windows, cùng ngày) | 762 ns | 775 ns | runner dùng chung; baseline cũ 687 ns — `main` cũng vượt ngưỡng 10 % |
| `parse_config` (runner Windows) | +56 % so với baseline | tương đương main | trôi của runner: cùng mã trên `main` cũng vượt ngưỡng hôm nay |

Job hiệu năng trên runner GitHub là tham khảo (`continue-on-error`); cổng cứng chạy trên
runner cố định (P1-5 §5).

## 6. Tái lập

```bash
cargo test --workspace && for a in headless win tsf mac linux; do
  cargo run -q -p textvn-cli -- replay corpus --adapter $a; done
scripts/e2e-linux.sh && scripts/build-linux.sh && scripts/test-linux-package.sh dist/TextVN-*.tar.gz
```

```powershell
powershell -File .\build-release.ps1 -BuildInstaller
powershell -File installer\windows\tests\test-portable.ps1 -Zip (Get-Item dist\TextVN-portable-*.zip).FullName
powershell -File installer\windows\tests\test-installer.ps1 -Setup (Get-Item dist\TextVN-setup-*.exe).FullName
```
