# P0-5 — Nền tảng thống nhất + chip ARM (Apple M · Windows on ARM · Linux aarch64)

> Trạng thái: **kế hoạch v2** (2026-11-27, sau 0.2.93) — thay bản v1 cùng ngày (chủ repo: "chưa
> tốt, cần nghiên cứu sâu, ưu tiên cái OS đã hỗ trợ, có thể xây lại nền tảng native thống nhất nếu
> chất lượng vượt trội"). Chưa có mã. Mỗi mục `SF-*`/`ARM-*` xong phải vào CHANGELOG +
> `build-release-report.md` và đi đủ quy trình phát hành (G20).
>
> Triển khai chi tiết: **`P0-6-SESSION-FRAMEWORK-SPEC.md`** (giải pháp, C ABI, ARM64X, CI) và
> **`P0-7-TASKS-ARM-SF.md`** (task, DoR, DoD, audit). Giao nhận việc: `../../coordination/README.md` (G21).
>
> Quyết định chủ repo đã chốt: **không armhf** (Raspberry Pi OS 32-bit); không theo đuổi hook trên
> ARM; kiểm trên **CI trước** (máy Snapdragon/Jetson/DGX sau); MSIX: theo khuyến nghị §6.4.

## 1. Tóm tắt khuyến nghị

1. **Không viết lại engine, không bỏ framework gõ của OS** — trên cả 3 OS, thứ "OS hỗ trợ sẵn" chính
   là framework IME: TSF (Windows, chạy native ARM64), InputMethodKit (macOS, universal), IBus
   (GNOME — **đường duy nhất**: Mutter không có giao thức input-method cho bên thứ ba) và Fcitx5 /
   Wayland input-method-v2 (KDE, wlroots). Bỏ chúng = mất GNOME/Ubuntu và mất tích hợp của OS.
2. **Viết lại tầng NGHIỆP VỤ giao diện thành một framework dùng chung** — hiện cùng một nghiệp vụ được
   viết 3 lần (Rust trong TSF, C trong `linux-common`/IBus/Fcitx5, Swift trong IMK): ~12 nghìn dòng
   (§3.2), nguồn chính của lỗi "lệch 3 OS". Xây **`textvn-session`** (Rust, C ABI) làm bản DUY NHẤT;
   adapter OS chỉ còn là cầu nối mỏng. Đây là "nền tảng native thống nhất" đúng nghĩa: native ở mỗi OS,
   một nghiệp vụ cho mọi OS.
3. **ARM là việc build + đóng gói + kiểm thử**, làm theo tiền lệ đã chạy thật:
   - Windows ARM64: TSF + **DLL chuyển hướng ARM64X** (Weasel/Rime 0.16+ và VietType đang phát hành) —
     không hook, không ARM64EC thuần.
   - Linux aarch64: build native trên runner ARM (Ngó Sen đã phát hành deb/rpm arm64).
   - macOS: đã universal; chỉ còn notarization + e2e arm64.
4. Thứ tự: **Linux aarch64 (nhanh, rủi ro thấp) ∥ nền móng `textvn-session`** → Windows ARM64 →
   chuyển dần nghiệp vụ vào `textvn-session` → (tuỳ chọn) backend Wayland input-method-v2 riêng.

## 2. Nghiên cứu bộ gõ — mới & nổi tiếng (2026-11-27)

| Bộ gõ | Cách bắt phím | ARM | Rút ra cho TextVN |
|---|---|---|---|
| **UniKey** 4.6 RC2 | hook toàn cục (Windows) | **ARM64 native** (06/2025) | người dùng Snapdragon đã có bản native ⇒ TextVN phải native; hook chạy ngoài tiến trình nên dễ port — TSF cần ARM64X (§6) |
| **EVKey** | hook | không (issue #71 mở) | khoảng trống |
| **OpenKey** | hook (Win x64), event tap (mac) | mac universal; lỗi M1 do event tap/helper Intel (#189, #193 Bitwarden, #197 sau khoá màn hình) | giữ IMK làm đường chính; gate "mọi binary universal" |
| **Gõ Nhanh** (mới, lõi Rust) | Windows: **WPF .NET 8 + keyboard hook** x64; Linux: addon Fcitx5 (phát hành x86_64); macOS: app Swift + Rust | chưa có ARM Windows/Linux phát hành | cùng mô hình "lõi Rust + vỏ OS" như TextVN; TextVN đi trước ở TSF + 3 OS |
| **VietType** (Vietnamese TSF) | **TSF** | **MSI ARM64** riêng, DLL build từ cấu hình **ARM64EC/ARM64X** | tiền lệ TSF tiếng Việt trên ARM |
| **Weasel / Rime** | **TSF** | x86/x64/arm/arm64 một bộ cài; **`weasel.dll` = ARM64X forwarder** → `weaselARM64.dll` (tiến trình ARM64) / `weaselx64.dll` (x64 giả lập); `Register.cpp` ghi đường dẫn forwarder chứ không phải DLL thật | thiết kế mẫu §6; issue #803: chỉ ARM64 thì app x64 mất bộ gõ, chỉ x64 thì app ARM64 không gọi được bộ gõ |
| **Squirrel / Rime**, **fcitx5-macos** | IMK | universal / tarball theo arch | giữ universal |
| **PHTV**, **xkey**, **goxkey** | macOS (event tap / IMK) | PHTV phát hành dmg arm64 + intel riêng | universal 1 gói tốt hơn 2 gói |
| **Ngó Sen** (fcitx5) | Fcitx5 | **deb arm64** (bookworm, forky, noble), **rpm aarch64** (Fedora 43–45) | ma trận gói ARM Linux tham chiếu |
| **fcitx5-lotus**, **fcitx5-unikey**, **ibus-bamboo** | Fcitx5 / IBus | lotus: x86_64; unikey/bamboo: distro tự build; bamboo từng thiếu gói ARM (#120) và cross-build hỏng (#37) | build native trên runner ARM, không cross-compile |
| **kime** (Hàn, 634★) | **framework riêng**: module GTK3/4, Qt5/6, XIM, Wayland | — | tiền lệ "tự viết thay IBus/Fcitx": **không gắn được vào GNOME Wayland** (Mutter không có input-method v1/v2) ⇒ bằng chứng không nên thay IBus/Fcitx5 |
| **Microsoft Vietnamese Telex** | TSF | ARM64 native | chuẩn hành vi TSF trên Snapdragon |

## 3. Hiện trạng TextVN (0.2.93)

### 3.1 Ma trận kiến trúc

| OS | Framework | Kiến trúc phát hành | Thiếu |
|---|---|---|---|
| Windows | TSF (mặc định) · hook (opt-in) | x64 + DLL x86 (WOW64) | ARM64: app ARM64 native (Edge, Chrome, Notepad, ô tìm kiếm Start) **không nạp được TIP** |
| macOS | IMK | **universal arm64 + x86_64** | notarization; e2e gõ thật arm64 |
| Linux | IBus + Fcitx5 (`linux-common`) | x86_64 | aarch64 (`debian/control` đã ghi `arm64`, `build-linux.sh` dùng `uname -m`, chưa có CI/release ARM) |

### 3.2 Nghiệp vụ bị viết lại 3 lần (đo 2026-11-27)

| Nghiệp vụ | Windows (Rust) | Linux (C) | macOS (Swift) |
|---|---|---|---|
| Kế hoạch composition/preedit theo kết quả engine | `compose.rs` (`plan_key`) | `compose.c` (`lc_plan_key`) | `ApplyReplace.swift`, `Marked.swift` |
| Máy trạng thái Ctrl+Shift / Ctrl tap | `ModifierToggle` | `compose.c` | `TextVNInputController.swift` |
| Trợ lý: trigger, idle 1 s, popup, Mode 2 | `assist.rs`, `panel_idle.rs` | `assist_*.c`, engine.c/.cpp | `AssistModel.swift` |
| Chữ mờ / preview dự đoán | `ghost.rs` + `edit_session.rs` | `update_preview` (IBus/Fcitx5) | `updatePredictionPreview` |
| Nhận diện ô (mật khẩu, terminal) | crate `field-detect` | `field_detect.c` | `FieldDetect.swift` |
| IPC client tới tray/app | `ipc_client.rs` | `ipc_client.c` (652 dòng) | `IpcClient.swift` (562 dòng) |
| Emoji picker | `emoji_picker.rs` | `emoji_picker.c` | `EmojiPickerPanel.swift` |

Tổng ~12 nghìn dòng adapter. Nhiều mã lỗi lịch sử là lệch 3 bản (R-3, R-5, R2-143, R2-155, R2-171…).

## 4. "Native từ đầu" — đánh giá theo từng OS

| Lựa chọn | Windows | macOS | Linux |
|---|---|---|---|
| Framework IME của OS | **TSF** — duy nhất chính thống, chạy ARM64 native; app ARM64/x64/ARM64EC/x86/AppContainer đều qua TSF | **IMK** — duy nhất chính thống cho bộ gõ hệ thống | **IBus**: đường duy nhất trên GNOME (Mutter không có input-method v1/v2 cho bên thứ ba); **Fcitx5**: KWin khởi chạy qua "Virtual Keyboard"; wlroots (Sway, Hyprland, labwc) có **input-method-v2** |
| Viết framework riêng thay thế | không có API thay TSF; hook = ngoài chuẩn (AV, game, Backspace giả) | event tap = ngoài chuẩn (secure input, Bitwarden — OpenKey #193) | kime: module GTK/Qt + XIM + Wayland ⇒ vẫn không chạy GNOME Wayland, phải duy trì 6 frontend |
| Kết luận | **giữ TSF**, thêm ARM64X | **giữ IMK** | **giữ IBus + Fcitx5** làm vỏ; thêm backend **input-method-v2** trực tiếp (tuỳ chọn, §7.3) cho wlroots |

Tận dụng thêm cái OS đã có (hiệu năng + đúng chuẩn): kiểm tra chính tả của OS (đã có `os_spell.rs`,
`MacSpellProvider`; Linux: enchant/hunspell khi có), bảng ứng viên/dòng phụ của IBus (đã dùng auxiliary
text), surrounding text của IBus/Fcitx5/text-input-v3 (giảm Backspace giả), display attributes TSF,
`ITfCandidateListUIElement` cho chế độ UI-less (game toàn màn hình, trình đọc màn hình).

## 5. Nền tảng thống nhất: `textvn-session` (SF)

### 5.1 Kiến trúc

```
                 ┌──────────────── textvn-session (Rust, C ABI, không I/O) ───────────────┐
OS event ──▶ key/focus/caret/surrounding ─▶ engine (core) ─▶ ra LỆNH chuẩn hoá:            │
                 │  Preedit{text,cursor} · Commit{text} · DeleteBefore{n} · Forward{key}   │
                 │  Ghost{text} · AssistList{items} · Toast{id} · Timer{ms} · Ipc{msg}      │
                 └────────────────────────────────────────────────────────────────────────┘
adapter mỏng: TSF (Rust) · IMK (Swift) · IBus (C) · Fcitx5 (C++) · [Wayland IM v2 (Rust)] chỉ ÁP lệnh
```

- Một bản cho: kế hoạch composition, gom chỉnh sửa (`lc_edit_batch` thành luật chung — mỗi phím ≤ 1
  xoá + 1 commit), máy trạng thái phím chuyển, trợ lý (trigger/idle/Mode 2/undo), chính sách chữ mờ,
  nhận diện ô (crate `field-detect` có sẵn), IPC client, emoji picker (logic tìm/xếp hạng).
- Thời gian (`Timer`) và vẽ UI vẫn ở adapter (mỗi OS có vòng lặp riêng); session chỉ quyết định.
- C ABI ổn định như `ffi/` (P0-2): struct có version, không panic qua ABI, không cấp phát chéo.

### 5.2 Chất lượng "vượt trội" — gate bắt buộc trước khi thay một khối

| Gate | Nội dung |
|---|---|
| SF-G1 Vết vàng | corpus (shared + win + mac + linux) chạy qua `textvn-session` ⇒ chuỗi lệnh; so khớp 100% với vết của adapter cũ trước khi xoá mã cũ |
| SF-G2 Một vết cho 3 OS | cùng corpus, cùng chuỗi lệnh trên Windows/macOS/Linux (khác biệt chỉ được ở bảng năng lực OS khai báo) |
| SF-G3 e2e thật | e2e hiện có (TSF Notepad/Chrome…, IBus/Fcitx5 GTK/Qt, IMK) xanh trên x64 **và** ARM |
| SF-G4 Fuzz + đột biến | fuzz chuỗi phím/caret/focus vào session; đột biến từng luật phải làm test đỏ |
| SF-G5 Hiệu năng | `ime_key` + session p50/p99 không tăng so baseline từng target (§8) |
| SF-G6 Dream RSI | savings-rate trên corpus không giảm (G19c) |

Quy tắc mới (đưa vào `00-WORKFLOW`): **nghiệp vụ giao diện mới chỉ được viết trong `textvn-session`**;
adapter chỉ áp lệnh + vẽ. Gate `xtask` cảnh báo khi adapter thêm logic trùng (từ khoá trong §3.2).

### 5.3 Thứ tự chuyển (mỗi bước một bản phát hành)

| Bước | Khối | Lý do thứ tự |
|---|---|---|
| SF-1 | khung crate + C ABI + harness vết vàng (SF-G1/G2) | nền đo |
| SF-2 | máy trạng thái phím chuyển + gom chỉnh sửa | nhỏ, thuần, lệch nhiều nhất |
| SF-3 | kế hoạch composition/preedit | lõi gõ — cần vết vàng đầy đủ |
| SF-4 | chữ mờ + dự đoán (R2-173) | vừa thống nhất ở core |
| SF-5 | trợ lý (trigger/idle/Mode 2/undo) | lớn nhất, nhiều lệch lịch sử |
| SF-6 | IPC client (bỏ `ipc_client.c`, `IpcClient.swift`) + field-detect qua FFI | gộp hạ tầng |
| SF-7 | emoji picker logic | cuối |

## 6. Windows on ARM (Snapdragon X, NVIDIA RTX Spark/N1X)

### 6.1 Quy tắc nạp DLL (Windows 11 ARM64)

| Tiến trình | Nạp được | Registry |
|---|---|---|
| ARM64 native (Edge, Chrome, Notepad, Explorer, ô tìm kiếm Start) | ARM64, ARM64X | 64-bit |
| x64 giả lập + ARM64EC (Office trên ARM là ARM64EC) | x64, ARM64EC, ARM64X | **cùng** 64-bit |
| x86 giả lập | x86 | WOW64 (riêng) |

⇒ một CLSID 64-bit phải trỏ tới **một tệp nạp được ở cả ARM64 lẫn x64** = ARM64X.

### 6.2 Thiết kế (theo Weasel)

| Tệp | Build |
|---|---|
| `textvn-tsf.dll` (đăng ký `InprocServer32`) | **ARM64X pure forwarder**: `link /dll /noentry /machine:arm64x /defArm64Native:tsf_arm64.def /def:tsf_x64.def` + 2 obj rỗng (MSVC ARM64/ARM64EC tools) — xuất `DllGetClassObject`, `DllCanUnloadNow`, `DllRegisterServer`, `DllUnregisterServer` |
| `textvn-tsf-arm64.dll` | `cargo build --target aarch64-pc-windows-msvc -p textvn-win-tsf` |
| `textvn-tsf-x64.dll` | DLL x64 hiện có (đổi tên) |
| `textvn-tsf-x86.dll` | giữ nguyên (WOW64) |
| Tray / CLI / settings | `aarch64-pc-windows-msvc` native |
| Hook | **không build ARM64** (TSF phủ mọi loại tiến trình; hook vẫn opt-in trên x64) |

Bẫy đã biết (Weasel `Register.cpp`): trong DLL thật, `GetModuleFileName` trả tên DLL thật, không phải
forwarder ⇒ `register` phải ghi đường dẫn **forwarder**. ACL AppContainer (`S-1-15-2-1/2`) cấp cho
**cả 4** DLL (bài R2-26: ô tìm kiếm Start trên Snapdragon là tiến trình ARM64 AppContainer).
Windows 10 on ARM không giả lập x64 và không có ARM64X ⇒ hỗ trợ **Windows 11 ARM64**; trên Windows 10
ARM chỉ cài DLL ARM64 + x86.

### 6.3 Kiểm thử (CI `windows-11-arm`, miễn phí cho repo public)

`cargo test` native ARM64 · `replay --adapter tsf` · e2e gõ thật: Notepad (ARM64), một app x64 giả
lập, một app x86, `sandbox_e2e` (AppContainer + LPAC ARM64) · `check-pe-imports` kiểm machine type
từng tệp (ARM64 / ARM64X / x64 / x86) · đột biến: đăng ký DLL ARM64 thay forwarder ⇒ e2e app x64 đỏ.

### 6.4 MSIX: **một `.msixbundle` (x64 + arm64)** — quyết định

| | Bundle x64 + arm64 (chọn) | Hai gói riêng |
|---|---|---|
| Người dùng | một trang Store, Store tự chọn đúng kiến trúc | dễ cài nhầm x64 trên Snapdragon (chạy giả lập, tốn pin) |
| Nâng cấp | cùng identity + version ⇒ máy ARM đang dùng gói x64 được Store chuyển sang arm64 khi cập nhật | phải hướng dẫn gỡ/cài lại |
| Dung lượng tải | Store chỉ tải gói đúng kiến trúc | như nhau |
| Chi phí build | phải build + ký + kiểm cả hai gói **cùng version** trong một lần phát hành | pipeline đơn giản hơn |
| Rủi ro | một gói lỗi chặn cả bundle ⇒ CI kiểm cả hai trước khi bundle | lệch version giữa hai gói |

Gói arm64 chứa: tray/CLI ARM64 + forwarder ARM64X + DLL ARM64 + DLL x64 + DLL x86. Bộ cài Inno:
`ArchitecturesAllowed=x64compatible arm64`, chọn file theo `IsArm64`; portable thêm
`TextVN-portable-*-windows-arm64.zip`.

## 7. Linux aarch64 (NVIDIA Jetson / DGX Spark, Raspberry Pi 5, Snapdragon X Linux)

### 7.1 Build + gói (native trên runner ARM — không cross-build, bài ibus-bamboo #37)

| Đích | Môi trường | Gói |
|---|---|---|
| Ubuntu 22.04 (= Jetson JetPack 6) · 24.04 (= DGX OS) | `ubuntu-22.04-arm`, `ubuntu-24.04-arm` | tarball `linux-aarch64`, deb arm64 (PPA Launchpad tự build arm64) |
| Debian 12/13 (= Raspberry Pi OS 64-bit) | container arm64 | deb arm64 |
| Fedora / RHEL 9 / AlmaLinux 8 | container aarch64 | tarball compat glibc cũ (như `build-linux-el8.sh`) |
| Arch Linux ARM | container `archlinuxarm` | `PKGBUILD arch=('x86_64' 'aarch64')` |

### 7.2 Lớp lỗi riêng ARM — chặn bằng gate, không đợi người dùng

| Lỗi | Gate |
|---|---|
| `char` không dấu trên aarch64 Linux | build C test trên x86 với `-fsigned-char` **và** `-funsigned-char` + `-Wsign-compare` (đã soát: mã hiện ép `(unsigned char)` đúng) |
| ABI FFI | `verify-abi` + `sizes` 20/532 chạy trên mọi target |
| Trang 16K/64K (Asahi, một số kernel Jetson/Grace) | `dict/src/mmap.rs` map offset 0, không giả định 4096 — giữ quy tắc, test trên runner ARM |
| armhf | **không làm** (chủ repo); `mmap(…, offset: i64)` tự khai chỉ đúng trên 64-bit |

### 7.3 Backend Wayland input-method-v2 trực tiếp (tuỳ chọn, sau SF-3)

Cho wlroots (Sway, Hyprland, labwc của Raspberry Pi OS) khi không cài IBus/Fcitx5: adapter Rust mỏng
trên `textvn-session`, không daemon. **Không** thay IBus (GNOME) hay Fcitx5 (KDE); không phục vụ app
X11/XWayland. Chỉ bật khi đạt SF-G1…G6 + e2e Sway/labwc headless trên CI.

## 8. Hiệu năng & đo đạc đa kiến trúc

- Baseline perf theo target triple (`perf/baseline-<triple>.json`); cổng `perf` so đúng máy đang chạy.
- Kiểm tất định: replay corpus + Dream RSI trên mọi target ⇒ hash đầu ra giống x86_64.
- `core/` không thêm mã theo kiến trúc (`target_arch`/SIMD); muốn tối ưu thì có bản thuần + test tương đương.
- Doctor/gói chẩn đoán ghi kiến trúc tiến trình, kiến trúc DLL đã nạp, chế độ giả lập (`IsWow64Process2`).

## 9. Lộ trình phát hành

| Bản | Nội dung | Điều kiện xong |
|---|---|---|
| **0.2.94** | ARM-L: CI `ubuntu-24.04-arm` + `ubuntu-22.04-arm` (test, e2e IBus/Fcitx5, replay, verify-abi, perf baseline) · gate `-funsigned-char` · **SF-1** khung `textvn-session` + vết vàng | CI ARM xanh; vết vàng = adapter cũ |
| **0.2.95** | asset `linux-aarch64` + deb arm64 + AUR aarch64 + E2E 6 container trên ARM · **SF-2** phím chuyển + gom chỉnh sửa trên 3 OS | 34 asset + asset ARM ký GPG/Sigstore |
| **0.2.96** | **ARM-W**: DLL ARM64, forwarder ARM64X, `register`, Inno + portable arm64, `.msixbundle`, CI `windows-11-arm` e2e (ARM64 / x64 giả lập / x86 / AppContainer) | ma trận §6.3 xanh |
| **0.2.97 → 0.3.0** | **SF-3 … SF-7**; adapter còn ≤ ~1,5 nghìn dòng mỗi OS; quy tắc "nghiệp vụ chỉ trong session" thành gate | SF-G1…G6 xanh mỗi bước |
| sau 0.3.0 | backend Wayland input-method-v2 (tuỳ chọn); notarization macOS khi có secrets; kiểm tay Snapdragon/Jetson/DGX | §7.3 |

## 9b. Tương thích & chống tái diễn — yêu cầu xuyên suốt (chủ repo 2026-11-27)

> "Kế hoạch mới phải cover tương thích và build test CI cho toàn bộ Windows, Linux distro phổ biến của cá
> nhân và doanh nghiệp như đã làm ở các version trước, không chỉ Ubuntu; lỗi đã fix không được tái hiện."
> Tái cấu trúc (`textvn-session`) và ARM KHÔNG được làm hẹp ma trận đã có; mọi target mới chạy CÙNG bộ test.

### Ma trận phải GIỮ (0.2.94) và MỞ RỘNG

| Nền tảng | Đang có (giữ nguyên) | Mở rộng trong kế hoạch |
|---|---|---|
| Windows phiên bản/bản | 10 1507/LTSB → 22H2 và 11 21H2 → 24H2+, Home/Pro/Enterprise/LTSC (E65/E66: sàn API, delay-load, `check-pe-imports`); app x86 (WOW64); AppContainer/LPAC (`sandbox_e2e`); per-user + machine + portable + MSIX; Store validation | CI `windows-2025` cạnh `windows-2022`; **Windows 11 ARM64** (`windows-11-arm`: ARM64 native + x64 giả lập + x86 + AppContainer); kiểm tay trước bản lớn: Win10 22H2 Home, Win11 24H2 Enterprise/LTSC (VM) |
| Linux doanh nghiệp | RHEL/AlmaLinux 8.10 + 9 (Rocky/Oracle cùng họ), SLES/openSUSE Leap 15.6 — E2E container mỗi bản; tarball `glibc ≥ 2.28` | AlmaLinux 10 (RHEL 10), SLES 16 khi có ảnh; **ảnh arm64** của mọi distro có ảnh |
| Linux cá nhân | Ubuntu 22.04/Mint 21, Debian 12, Fedora 41 — E2E; Ubuntu 20.04/Mint 20, Debian 11 — smoke; Arch (`ci-aur`), Debian trixie (`ci-deb`) | Ubuntu 24.04 (= DGX OS) + 26.04 (GNOME 50, mutter#4853 — R2-166), Debian 13, Fedora 42/43, openSUSE Tumbleweed, Linux Mint 22; Raspberry Pi OS 64-bit (Debian arm64) |
| Môi trường desktop | IBus + Fcitx5, GTK3/GTK4/Qt, X11 + Wayland (e2e container) | GNOME Wayland/X11 (IBus), KDE Plasma Wayland (Fcitx5 qua Virtual Keyboard), Xfce/Cinnamon X11, wlroots (Sway/labwc) headless trên CI |
| macOS | universal arm64 + x86_64, CI arm64 | e2e IMK arm64 (ARM-M2) |

### Chống tái diễn (lỗi đã fix không quay lại)

1. **Sổ hồi quy (REG-1)**: `cargo xtask regression-index` đọc mã đã sửa trong
   `docs/specs/project-common-errors.md` (E*), `docs/specs/win-test-common-errors.md` (B*),
   `docs/release/release-process.md` (R*), mã R2-* trong CHANGELOG — mỗi mã phải trỏ tới ≥ 1 test/corpus/gate
   (`corpus/**/bug_*`, `r2_*`, `core/tests/*`, e2e, gate `xtask`) qua bảng `docs/specs/regression-index.md`;
   mã "đã sửa" thiếu test ⇒ preflight + CI đỏ. Mã chỉ-quy-trình ghi rõ gate quy trình tương ứng.
2. **Cùng bộ test cho mọi target**: x86_64 và ARM chạy TOÀN BỘ `cargo test`, replay 4 adapter, e2e — không
   tập con; distro mới vào ma trận chạy đủ e2e như distro cũ.
3. **Chuyển nghiệp vụ sang session không xoá test cũ**: test hồi quy của adapter cũ được chuyển thành test
   của session/vết vàng TRƯỚC khi xoá mã cũ (SF-G1…G6), và e2e trên toàn ma trận phải xanh.
4. **B4b (G23)** chạy release.yml đầy đủ (mọi distro container + Windows + macOS) trên commit sẽ tag.

## 10. Rủi ro

| Rủi ro | Giảm thiểu |
|---|---|
| Thay nghiệp vụ gây hồi quy | vết vàng + chuyển từng khối + giữ adapter cũ sau cờ tới khi vết khớp 100% |
| Runner ARM chậm hơn ⇒ timeout | ma trận ARM chạy song song, timeout riêng, E76 watchdog |
| Công cụ ARM64EC/ARM64X không có trên runner | cài `VC.Tools.ARM64EC` qua vswhere như Weasel `arm64x_wrapper/build.bat`; thất bại ⇒ job đỏ, không phát hành nửa vời |
| AV gắn cờ DLL forwarder rỗng | ký Authenticode (SignPath) trước khi Store; theo dõi `antivirus-false-positive.md` |
| Swift/C gọi session qua C ABI lệch | `verify-abi` mở rộng cho header session; test Swift + C trên CI |

Nguồn: Microsoft Learn "Build Arm64X binaries" / "Arm64X PE files"; rime/weasel `arm64x_wrapper/`,
`WeaselTSF/Register.cpp`, issue #803, CHANGELOG 0.16.0; dinhngtu/VietType `VietTypeSetup/Product.wxs`;
UniKey ARM64 (unikey.org); EVKey #71; OpenKey #189/#193/#197; khaphanspace/gonhanh.org
`platforms/{windows,linux}`; ngosen 1.14.3 assets; LotusInputMethod/fcitx5-lotus 7.0.1; Riey/kime
(GNOME Wayland); GNOME Discourse (Mutter không có input-method v2); KDE (KWin quản lý IM qua Virtual
Keyboard); GitHub changelog 2025-08-07 (runner arm64 public); Rust platform support
`aarch64-pc-windows-msvc`, `arm64ec-pc-windows-msvc`.
