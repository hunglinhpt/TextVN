# P0-6 — Solution spec: `textvn-session` (một nghiệp vụ cho 3 OS) + build ARM

> Trạng thái: **spec sẵn sàng triển khai** (2026-11-27). Đọc trước: `P0-5-ARM-PLATFORMS-PLAN.md`
> (vì sao), `P0-2-engine-ffi-contract.md` (C ABI hiện có), `../01-AGENT-HANDBOOK.md` (DoD 8 mục),
> `../00-WORKFLOW.md` (G1–G20). Danh sách việc + DoR/DoD/audit: `P0-7-TASKS-ARM-SF.md`.
> Mọi tên tệp/hàm nhắc dưới đây đã đối chiếu với repo ở commit `c507af31`; tên MỚI được đánh dấu
> **(mới)**.

## 0. Thuật ngữ

| Từ | Nghĩa |
|---|---|
| **session** | trạng thái nghiệp vụ của MỘT ô nhập (input context) — một instance `textvn-session` / context TSF / `IBusEngine` / `fcitx::InputContext` / `IMKInputController` |
| **sự kiện** (`SessionEvent`) | thứ adapter đưa vào: phím, focus, caret, surrounding text, reset, timer, IPC… |
| **lệnh** (`SessionCommand`) | thứ session trả ra để adapter ÁP: preedit, commit, xoá, forward phím, chữ mờ, danh sách trợ lý, timer, IPC |
| **năng lực** (`Caps`) | adapter khai báo nó làm được gì (preedit, surrounding, xoá trước caret, chữ mờ tại caret…) |
| **vết vàng** (golden trace) | chuỗi lệnh chuẩn hoá ứng với một ca corpus — tệp `corpus/traces/**.jsonl` **(mới)** |
| **shadow mode** | adapter chạy cả mã cũ lẫn session, so lệnh, chỉ áp lệnh mã cũ (an toàn khi chuyển) |

## 1. Mục tiêu đo được

| KPI | Hiện tại | Mục tiêu 0.3.0 |
|---|---|---|
| Dòng nghiệp vụ giao diện viết lặp ở adapter | ~12 nghìn (§3.2 P0-5) | adapter ≤ ~1,5 nghìn dòng/OS, 0 nghiệp vụ lặp |
| `replay --adapter mac/linux` mô phỏng adapter thật | **không** (chỉ đổi caps, `cli/src/replay.rs` `AdapterProfile`) | replay chạy CHÍNH `textvn-session` cho mọi adapter |
| Lỗi "lệch 3 OS" mới / bản | có (R-3, R-5, R2-143, R2-155, R2-171…) | 0 (một bản mã) |
| Kiến trúc phát hành | Win x64/x86 · mac universal · Linux x86_64 | + Windows ARM64 (Win 11) · Linux aarch64 |
| Hiệu năng | `ime_key` p50 ≈ 0,59 µs (x64) | `session_event` p99 ≤ `ime_key` p99 + 10 µs trên mọi target |

## 2. Kiến trúc

```
              ┌───────────────────────── crate session/ (textvn-session) (mới) ─────────────────────────┐
              │  Session::event(SessionEvent) -> Commands      (thuần, không I/O, không OS, không log text) │
              │   ├─ toggle   : Ctrl+Shift / Ctrl tap (từ compose.rs ModifierToggle, compose.c, Swift)    │
              │   ├─ compose  : kế hoạch preedit/commit (từ compose.rs plan_key, compose.c lc_plan_key,    │
              │   │             ApplyReplace.swift/Marked.swift)                                          │
              │   ├─ batch    : ≤ 1 DeleteBefore + ≤ 1 Commit mỗi sự kiện, commit trước khi ẩn preedit     │
              │   │             (R2-165 lc_edit_batch, R2-167) — luật CHUNG mọi OS                          │
              │   ├─ ghost    : chính sách chữ mờ (R2-134, R2-173, E79)                                    │
              │   ├─ assist   : trigger / idle 1 s / Mode 2 / undo (assist.rs, panel_idle.rs, assist_*.c,  │
              │   │             AssistModel.swift)                                                        │
              │   ├─ field    : crate field-detect (đã có) — C/Swift bỏ bản riêng                            │
              │   └─ ipc      : client tray/app theo textvn_ipc (TSF đã Rust; bỏ ipc_client.c, IpcClient.swift)│
              │  dùng: textvn-core, textvn-config, textvn-field-detect, textvn-strategy, textvn-assist,     │
              │        textvn-ipc (codec)                                                                 │
              └───────────────▲──────────────────────────────────────────────────────▲───────────────────┘
                              │ Rust trực tiếp                                         │ C ABI ime_session_* (mới)
                     adapters/windows-tsf                               ffi/ (libtextvn_ffi: staticlib+cdylib)
                                                                        ├─ adapters/linux-ibus (C)
                                                                        ├─ adapters/linux-fcitx5 (C++)
                                                                        ├─ adapters/macos-imk (Swift, CTextVNFFI)
                                                                        └─ [adapters/linux-wayland-im (Rust, mới, tuỳ chọn)]
```

Bất biến:

1. `session/` không I/O, không thread, không đồng hồ hệ thống — thời gian đến qua sự kiện (`now_ms`)
   ⇒ tất định, test được, chạy được trong `cli replay`.
2. Adapter KHÔNG được ra quyết định nghiệp vụ; chỉ (a) dịch sự kiện OS → `SessionEvent`, (b) áp
   `SessionCommand` bằng API OS, (c) vẽ UI, (d) cấp timer/IPC transport.
3. Không log nội dung gõ (G8) — session không có API log; adapter chỉ log mã lệnh/độ dài.
4. Một session = một luồng (thread của context). Không `Send` qua FFI; adapter giữ đúng luồng.
5. Panic trong session ⇒ `catch_unwind` ở biên FFI ⇒ `IME_ERR_INTERNAL` + session tự reset (như `ffi`).

## 3. Mô hình dữ liệu (Rust)

```rust
// session/src/lib.rs (mới) — phác thảo, tên chốt ở SF-1a
pub struct Caps {
    pub preedit: bool,               // TSF composition · IBus/Fcitx5 preedit · IMK marked text
    pub surrounding_text: bool,      // đọc được chữ trước caret
    pub delete_surrounding: bool,    // xoá trước caret không cần Backspace giả
    pub inline_ghost: bool,          // vẽ chữ mờ tại caret (TSF) — false: dòng phụ (IBus aux, IMK panel)
    pub key_up_events: bool,         // adapter nhận key-up (E79: vẽ chữ mờ ở key-up)
    pub forward_key: bool,           // trả phím cho app (IBus/Fcitx5 forward, TSF không ăn)
}

pub enum SessionEvent<'a> {
    KeyDown { vk: u32, ch: u32, mods: u32, now_ms: u64 },
    KeyUp   { vk: u32, mods: u32, now_ms: u64 },
    FocusIn { app_id: &'a str, field_role: u32, secure: bool, terminal: bool },
    FocusOut,
    CaretMoved,                                   // click / phím điều hướng / app tự dời
    Surrounding { before: &'a str },              // chữ trước caret (≤ 256 ký tự)
    Reset { kind: ResetKind, now_ms: u64 },       // ResetKind::{Toolkit, Focus, App}
    Timer { id: TimerId, now_ms: u64 },
    Ipc(textvn_ipc::Message),
    AssistAction { action: AssistAction },        // chọn gợi ý, Esc, Ctrl+Z…
}

pub enum SessionCommand {
    SetPreedit { text: String, cursor: u16 },     // rỗng = ẩn
    Commit { text: String },
    DeleteBefore { chars: u16 },
    ForwardKey,                                   // phím hiện tại đi thẳng app (không ăn)
    EatKey,
    Ghost { text: String },                       // rỗng = ẩn; adapter vẽ theo Caps.inline_ghost
    GhostAfterKeyUp,                              // E79: vẽ ở key-up của phím hiện tại
    AssistShow { items: Vec<AssistItem> }, AssistHide,
    TimerStart { id: TimerId, ms: u32 }, TimerCancel { id: TimerId },
    Ipc(textvn_ipc::Message),
    ModeIndicator { vi_on: bool },
}
```

Luật đầu ra (kiểm bằng `debug_assert` + test thuộc tính):

- Mỗi sự kiện: ≤ 1 `DeleteBefore`, ≤ 1 `Commit`, `Commit` đứng TRƯỚC `SetPreedit{""}` (R2-165, R2-167).
- Đúng MỘT trong `ForwardKey`/`EatKey` cho mỗi `KeyDown`.
- `Ghost.text` = đúng phần chữ Tab sẽ nối (`Engine::peek_completion`, R2-173) hoặc rỗng.
- Ô `secure` ⇒ không `Ghost`, không `AssistShow`, không `Ipc(StyleUpdate)` (S3).

## 4. C ABI (`ffi/`, header `ffi/include/textvn_ffi.h`)

Thêm vào thư viện hiện có (một `libtextvn_ffi` cho mọi OS — không thêm thư viện thứ hai):

```c
/* (mới) — mọi struct có abi_version + size, kiểm bởi `textvn-cli verify` như ime_key_v1 */
typedef struct ime_session ime_session;

typedef struct {
    uint32_t abi_version;     /* IME_SESSION_ABI_VERSION */
    uint32_t caps;            /* bit theo Caps */
    const char *config_json;  /* có thể NULL */
    size_t config_len;
} ime_session_config_v1;

typedef struct {
    uint32_t abi_version;
    uint32_t kind;            /* IME_SEV_KEYDOWN, … */
    uint32_t vk, ch, mods;
    uint64_t now_ms;
    const char *text;         /* app_id / surrounding (UTF-8), có thể NULL */
    size_t text_len;
    uint32_t arg;             /* field_role / reset kind / timer id … */
} ime_session_event_v1;

typedef struct {
    uint32_t kind;            /* IME_SCMD_SET_PREEDIT, … */
    uint32_t arg0, arg1;      /* cursor / chars / timer id / ms … */
    uint32_t text_off, text_len; /* vào `arena` */
} ime_session_cmd_v1;

#define IME_SESSION_MAX_CMDS 16
#define IME_SESSION_ARENA 2048
typedef struct {
    uint32_t abi_version;
    uint32_t count;
    ime_session_cmd_v1 cmds[IME_SESSION_MAX_CMDS];
    char arena[IME_SESSION_ARENA];  /* UTF-8, không NUL giữa chừng */
} ime_session_out_v1;

int32_t ime_session_new(const ime_session_config_v1 *cfg, ime_session **out);
void    ime_session_free(ime_session *s);
int32_t ime_session_event(ime_session *s, const ime_session_event_v1 *ev, ime_session_out_v1 *out);
int32_t ime_session_reload_config(ime_session *s, const char *json, size_t len);
```

- Không callback, không cấp phát chéo: adapter cấp `ime_session_out_v1` (stack), session ghi vào.
- Tràn `IME_SESSION_MAX_CMDS`/`ARENA` ⇒ `IME_ERR_BUFFER` + session không đổi trạng thái (adapter
  coi như `ForwardKey`) — có test.
- Header Swift `adapters/macos-imk/Sources/CTextVNFFI/include/textvn_ffi.h` tiếp tục được CI
  `ci-macos.yml` so với `ffi/include/` (gate đã có).
- `textvn-cli verify` mở rộng: kiểm sizeof/offset 4 struct mới trên MỌI target (x86_64, i686,
  aarch64 Windows/Linux/macOS).

## 5. Đặc tả từng khối (nguồn hiện tại → hợp nhất → luật phải giữ)

| Khối | Nguồn hiện tại (đã đối chiếu) | Luật/ID PHẢI giữ (mỗi cái có test trong session) |
|---|---|---|
| **toggle** | `adapters/windows-tsf/src/compose.rs` (`ModifierToggle`, `on_key_up`, `on_ctrl_tap_up`, `ctrl_held_too_long`), `adapters/linux-common/src/compose.c`, `TextVNInputController.swift` (`otherKeyPressed`) | Ctrl+Shift nhấn-thả KHÔNG kèm phím khác; Cmd/Option/phím khác huỷ; debounce 250 ms (tray `try_claim_global_toggle`, TIP `try_claim_hotkey_toggle`); Ctrl tap khoá từ (`sort`→khôi phục, `tunx`→`tũn`); pha thứ hai OnKeyUp không toggle lại |
| **compose** | `compose.rs` `plan_key`/`text_after`/`CompositionPlan`; `compose.c` `lc_plan_key`/`lc_comp_apply`; `ApplyReplace.swift`, `Marked.swift` | ranh giới in được commit CÙNG từ khi đang composing (CUAS: tránh " được"); Enter/Tab trả phím thật, bỏ `\n`/`\t` khỏi text (B2); `PASS && consumed` ⇒ ăn im lặng (E62 Math); Ctrl+Space không giữ composition (R2-121); dead key armed (P8); caret rời composition ⇒ commit nguyên văn (self-heal) |
| **batch** | `adapters/linux-common/src/edit_batch.c` (`lc_edit_batch`, R2-165) | ≤ 1 xoá + ≤ 1 commit/phím; flush trước forward phím; commit trước ẩn preedit (R2-167); áp dụng cho CẢ TSF/IMK (hiện TSF dùng 1 edit session nên đã đúng — test chứng minh) |
| **reset** | IBus `engine_reset`/Fcitx5 `reset` + `LC_RESET_AFTER_ENTER_US` (R2-171) | reset trong 250 ms sau Enter, không phím/đổi focus xen giữa ⇒ giữ mốc đầu dòng (`reset_with_tail("\n")`); reset khác xoá sạch (E78) — luật chuyển vào session, adapter chỉ gửi `Reset{kind, now_ms}` |
| **ghost** | `edit_session.rs` `update_ghost`, `ghost_waits_for_key_up`, `refresh_ghost_after_key_up`; IBus/Fcitx5 `update_preview`; IMK `updatePredictionPreview` | chữ mờ = `peek_completion` (R2-173, chỉ nối thêm); không vẽ khi sau caret còn chữ / ô secure / terminal / trợ lý đang hiện; phím đi thẳng app ⇒ `GhostAfterKeyUp` (E79) |
| **assist** | `adapters/windows-tsf/src/assist.rs`, `panel_idle.rs` (`idle_arms`), `linux-common/src/assist_*.c`, IBus/Fcitx5 `assist_idle_arm`, `AssistModel.swift` | trigger theo phím (AS-022), idle 1 s mọi phím chữ/xoá (R2-154/155), Mode 2 tự sửa an toàn chỉ khi `mode == autocorrect && auto_fix == safe` (R-5), undo chỉ sống tới phím kế (R-3), Ctrl+Space sửa câu (R2-131), ẩn ngay khi gõ tiếp (P9), Mode EN không sửa câu (R2-143) |
| **field** | crate `field-detect` (Rust); `linux-common/src/field_detect.c`; `FieldDetect.swift` | ô mật khẩu/PIN ⇒ passthrough tuyệt đối; terminal ⇒ chiến lược Backspace-type, không dự đoán (R2-144/145) |
| **ipc** | `adapters/windows-tsf/src/ipc_client.rs` + `ipc/src/sandbox.rs`; `linux-common/src/ipc_client.c`; `IpcClient.swift` | xác thực server (A7b, R2-110), pipe sandbox chỉ khi pipe đầy đủ lỗi 5 (R2-26), client sandbox không gửi nội dung (E80/E81), không bao giờ chặn luồng gõ (thread nền ở adapter — session chỉ ra `Ipc(msg)`) |
| **emoji** | `emoji_picker.rs`, `emoji_picker.c`, `EmojiPickerPanel.swift` | chỉ phần tìm/xếp hạng/MRU vào session (`Engine::emoji_search` đã ở core); vẽ panel ở adapter |

## 5b. UI dùng chung nguyên lý (G22)

Nghiệp vụ UI (chữ hiển thị, thứ tự/nhóm cài đặt, dòng gợi ý trợ lý, nội dung popup "Thông tin", nhãn
menu khay, thông báo lỗi người dùng thấy) hiện được gõ lại ở 3 nơi rồi giữ đồng bộ bằng test (ví dụ
`ABOUT_DETAILS` trong `tray/src/settings_dialog.rs`, `tv_about_text` trong
`adapters/linux-settings/src/settings_model.c`, `AppInfo.swift`; dòng gợi ý Mode trợ lý). Thay bằng
**một nguồn dữ liệu + sinh mã**, cùng cơ chế đã chạy cho bảng phím (`data/tables/keymap_mac.toml` →
`core/src/keymap_mac_generated.rs` + `KeyMapMacGenerated.swift`):

| Thành phần | Nguồn duy nhất | Sinh ra |
|---|---|---|
| Chữ UI (VI/EN), id ổn định | `data/ui/strings.toml` **(mới)** | `ui_strings_generated.rs` (tray, session), `ui_strings_generated.h/.c` (Linux), `UiStringsGenerated.swift` (macOS) |
| Mô hình cài đặt (nhóm, mục, kiểu, mặc định, phụ thuộc bật/tắt) | `data/ui/settings_model.toml` **(mới)** + schema `config.v1` | bảng mô tả cho 3 bộ dựng UI native |
| Hành vi popup/menu (mục nào hiện khi nào) | hàm thuần trong `textvn-session` (ví dụ `menu_items(state)`) qua C ABI | adapter chỉ vẽ |

Lệnh `cargo xtask gen-ui` / `check-ui` **(mới)**, thêm vào preflight như `check-tables`. Vẽ UI vẫn native
(Win32, GTK, AppKit) — chỉ nội dung và luật là chung.

## 6. Vết vàng + shadow mode (cách chuyển KHÔNG gây hồi quy)

1. **Ghi vết từ mã cũ** (SF-1c/d/e): mỗi adapter có bộ ghi chuẩn hoá lệnh nó áp:
   - TSF: `cli replay --adapter tsf --trace <out>` dùng `textvn_win_tsf::compose` (đã có mô phỏng).
   - Linux: test C `adapters/linux-common/tests/trace_compose.c` **(mới)** chạy `lc_plan_key` +
     `lc_edit_batch` trên corpus đã tách phím (xuất JSONL).
   - macOS: XCTest `TraceTests.swift` **(mới)** chạy `ApplyReplace`/`Marked` trên corpus.
   Định dạng một dòng: `{"case":"…","step":n,"cmd":"commit","text":"…","n":0}`; tệp ở
   `corpus/traces/{tsf,linux,mac}/<case>.jsonl`.
2. **So khác biệt**: `cargo xtask trace-diff` **(mới)** so vết 3 adapter với nhau; khác biệt hợp lệ
   (do năng lực OS) ghi trong `corpus/traces/ALLOWED_DIFF.md` kèm lý do + người duyệt.
3. **Session phải tái tạo vết** của mỗi adapter khi chạy với `Caps` của adapter đó (SF-G1).
4. **Shadow mode** trong adapter: cờ biên dịch/biến môi trường `TEXTVN_SESSION_SHADOW=1` (chỉ CI +
   bản dev): chạy cả hai, so lệnh, lệch ⇒ ghi mã lỗi (không ghi chữ — G8) và áp lệnh mã cũ.
5. **Bật mặc định** khi shadow 0 lệch trên toàn bộ e2e + corpus ≥ 1 bản phát hành; **xoá mã cũ** ở bản
   kế tiếp.

## 7. Windows ARM64 — đặc tả build & cài

### 7.1 Tệp

| Máy cài | `textvn-tsf.dll` (tên đăng ký, giữ nguyên) | DLL thật |
|---|---|---|
| x64 | DLL x64 thật (như hiện tại) | — |
| ARM64 (Windows 11) | **ARM64X pure forwarder** | `textvn-tsf-arm64.dll` (ARM64) · `textvn-tsf-x64.dll` (x64) |
| mọi máy | — | `textvn-tsf-x86.dll` (WOW64, như hiện tại) |

Giữ tên đăng ký `textvn-tsf.dll` ⇒ `cli/src/register.rs::resolve_dll_path` (tìm `textvn-tsf.dll`
trước) không đổi; cài đè/nâng cấp từ x64 sang ARM64 không phải đổi CLSID.

### 7.2 Build forwarder (MSVC chuẩn, không Python — E59)

Thư mục **(mới)** `installer/windows/arm64x/`: `dummy.c` (rỗng), `tsf_arm64.def`, `tsf_x64.def`:

```
EXPORTS
    DllGetClassObject   = textvn-tsf-arm64.DllGetClassObject
    DllCanUnloadNow     = textvn-tsf-arm64.DllCanUnloadNow
    DllRegisterServer   = textvn-tsf-arm64.DllRegisterServer
    DllUnregisterServer = textvn-tsf-arm64.DllUnregisterServer
```

(`tsf_x64.def` giống, trỏ `textvn-tsf-x64`). Lệnh (theo Weasel `arm64x_wrapper/build.bat`):

```
cl /c /Fo:dummy_arm64.obj dummy.c
cl /c /arm64EC /Fo:dummy_x64.obj dummy.c
link /lib /machine:arm64ec /def:tsf_x64.def   /out:tsf_x64.lib
link /lib /machine:arm64   /def:tsf_arm64.def /out:tsf_arm64.lib
link /dll /noentry /machine:arm64x /defArm64Native:tsf_arm64.def /def:tsf_x64.def ^
     /out:textvn-tsf.dll dummy_arm64.obj dummy_x64.obj tsf_x64.lib tsf_arm64.lib
```

Đóng gói thành `cargo xtask build-arm64x` **(mới)**: tìm toolchain qua `vswhere -requires
Microsoft.VisualStudio.Component.VC.Tools.ARM64EC`, chạy các lệnh trên, kiểm đầu ra bằng
`xtask check-pe-imports` (machine `ARM64X`, chỉ forwarder, không import CRT).

### 7.3 Đăng ký & quyền

- `register` trên máy ARM64 (phát hiện bằng `IsWow64Process2` → native machine `IMAGE_FILE_MACHINE_ARM64`):
  ghi `InprocServer32` = đường dẫn **forwarder** (bẫy Weasel: trong DLL thật `GetModuleFileName`
  trả tên DLL thật — không dùng nó để ghi registry).
- `grant_appcontainer_read` cho **cả 4** DLL (forwarder, arm64, x64, x86).
- Gỡ cài: xoá cả 4; `unregister_if_owned_by` nhận cả tên forwarder lẫn tên DLL thật.
- Windows 10 on ARM: chỉ DLL ARM64 + x86 (không ARM64X/x64) — bộ cài phát hiện bản build < 22000.

### 7.4 Bộ cài / gói

- Inno Setup `installer/windows/TextVN-setup.iss`: `ArchitecturesAllowed=x64compatible arm64`,
  `ArchitecturesInstallIn64BitMode=x64compatible arm64`; nhóm `[Files]` theo `IsArm64`.
- Portable: thêm `TextVN-portable-<v>-windows-arm64.zip`.
- MSIX: `TextVN-<v>-windows.msixbundle` = gói x64 (như hiện tại) + gói arm64 (tray/CLI ARM64 +
  4 DLL); cùng identity `MSIX_IDENTITY_NAME`, cùng version; `store-publish.yml` +
  `tools/release/verify-store-package.sh` nhận `.msixbundle` (kiểm từng gói bên trong).

## 8. CI & phát hành ARM

| Workflow | Thêm |
|---|---|
| `ci-linux.yml` | job ma trận `runs-on: [ubuntu-24.04, ubuntu-24.04-arm, ubuntu-22.04-arm]` cho test + build adapter + e2e IBus/Fcitx5; E2E distro container chạy lại trên `ubuntu-24.04-arm` (image arm64) |
| `ci-windows.yml` | job `windows-arm64` (`runs-on: windows-11-arm`): `cargo test --workspace` native, `replay --adapter tsf`, `verify`, gõ TSF thật vào Notepad ARM64 + app x64 giả lập + app x86, `sandbox_e2e` |
| `ci-shared.yml` | `verify-abi`/`sizes` + replay + Dream RSI trên mỗi target; so hash đầu ra replay giữa target |
| `release.yml` | job `linux-arm64` (tarball + test gói), `windows` thêm build aarch64 + forwarder + bundle; `publish` kiểm số asset mới; `SHA256SUMS`, `.asc`, `.cosign.*` cho asset ARM |
| perf | baseline theo target `perf/baseline-<triple>.json` **(mới)**; `xtask preflight` bước 17 chọn đúng baseline |

Gate C cho ARM Linux (chặn `char` không dấu): biên dịch test C `adapters/linux-common/tests` thêm một
lượt `-funsigned-char -Wsign-compare -Werror` trên x86 (bắt lỗi trước khi lên máy ARM).

Asset mới mỗi bản: `TextVN-<v>-linux-aarch64.tar.gz`, `textvn_<v>-0ppa1_arm64.deb` (nếu CI build deb),
`TextVN-setup-<v>-windows-arm64.exe` (hoặc một bộ cài đa kiến trúc — quyết ở ARM-W5),
`TextVN-portable-<v>-windows-arm64.zip`, `TextVN-<v>-windows.msixbundle` — kèm `.asc` + `.cosign.*`.
Nhánh `approved/vX.Y.Z/` thêm asset Windows ARM64 (B7c).

## 9. Hiệu năng & đo

| Đo | Ngân sách |
|---|---|
| `ime_session_event` (KeyDown chữ thường) p99 | ≤ `ime_key` p99 + 10 µs, mọi target |
| cấp phát mỗi KeyDown | 0 ở đường nóng (arena cố định) — kiểm bằng bộ đếm cấp phát trong bench |
| `peek_completion` (chữ mờ) | giữ ≤ 75 µs (README) trên ARM |
| Dream RSI corpus | savings-rate ≥ baseline (22% ở 0.2.93) trên mọi target, kết quả giống hệt x86_64 |

## 10. An toàn & riêng tư

- Session không có API log; FFI không trả chuỗi người dùng ngoài lệnh áp (G8).
- Ô secure: không ghost/assist/IPC chứa chữ (S3).
- IPC sandbox (R2-26/E80/E81) giữ nguyên: session trả `Ipc(msg)` đã lọc theo cờ `sandboxed` adapter
  cung cấp; adapter KHÔNG tự thêm nội dung.
- Forwarder ARM64X không có mã; ký Authenticode cùng lượt với DLL thật khi SignPath sẵn sàng.

## 11. Việc chưa chốt (ghi ADR khi chốt)

1. Một bộ cài đa kiến trúc hay bộ cài riêng `-windows-arm64.exe` (ARM-W5; khuyến nghị: một bộ cài,
   như Weasel "單安裝包支持 win32/x64/arm/arm64").
2. Backend Wayland input-method-v2 (`adapters/linux-wayland-im`) — chỉ sau SF-3 (P0-5 §7.3).
