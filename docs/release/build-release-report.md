# Báo cáo dựng & kiểm thử — TextVN

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

> `v0.2.11` **đã publish** 2026-10-03 dạng pre-release (mục “Bản 0.2.10 + 0.2.11” bên dưới). 0.2.10/0.2.9/0.2.8/0.2.7/0.2.6/0.2.5 publish 2026-10-02. Bằng chứng của bản đó
> nằm ở mục “Bản 0.2.2” bên dưới và
> [audit 2026-09-30](cross-platform-audit-2026-09-30.md) “Vòng 11”.
>
> `v0.2.1` đã được phát hành dạng pre-release. Bằng chứng của bản đó nằm ở
> các phần lịch sử phía dưới. Không dùng số liệu 0.1.0 làm bằng chứng
> production cho 0.2.x.

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
