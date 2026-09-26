# P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết

> WS2 · `adapters/linux-fcitx5/` (**C++** — `PLAN §4.1`: "Linux Fcitx5: C++, addon API C++").
> ADR-007: dual adapter. Fcitx5 là mặc định của nhiều distro (KDE neon, Arch, openSUSE) →
> bắt buộc có để đủ matrix người dùng (`PLAN §5.3`).
> Định dạng chung với `P3-1-ibus.md`; bản thân engine/thư viện dùng chung: `libvietime_ffi.a` +
> `libvietime-linux-common.a` — **không viết lại logic core** (P0-2).

## 1. Quyết định & capability

| Quyết định | Nội dung | Vì sao |
|---|---|---|
| Addon chạy **trong process fcitx5** (`.so` load bởi daemon) | Không spawn process riêng | Đúng model addon fcitx5; crash = fcitx5 crash → **bắt buộc** `catch` mọi callback (RL7) |
| Caps Fcitx5 v1 | `IME_CAP_PREEDIT \| IME_CAP_SELECTION \| IME_CAP_FIELD_DETECT` (+`INJECT_VK` không dùng) | Giống IBus; surrounding của fcitx5 → BackspaceType |
| **1 instance per `InputContext`** (map `IC* → ime_instance*`) | Tất cả trên main loop fcitx5 (1 thread) — hợp lệ P0-2 §3 | Multi-window không đạp nhau; giải phóng ở `IC` destroy |
| **Pin version** fcitx5-dev (RL2) | Spike `LNX-006` chốt tối thiểu (dự kiến ≥ 5.0.14); dùng macro `FCITX_VERSION` cho nhánh API khác | Addon API đổi giữa 5.0.x/5.1+ |
| C++ exceptions **không** tràn qua C | `try/catch(...)` quanh mọi call xuống `ime_key`/FFI; FFI đã `catch_unwind` (P0-2 §5) | Double safety, fcitx5 build `-fno-exceptions`? spike LNX-003 |

## 2. Cấu trúc module

```
adapters/linux-fcitx5/
├── CMakeLists.txt                  # find_package(Fcitx5Core REQUIRED) + pkg-config glib
├── src/
│   ├── addon.cpp                    # AddonInstance "vietime": đọc config, tạo engine factory
│   ├── engine.cpp                   # InputMethodEngine (keyEvent/reset/activateEvent/focusEvent)
│   ├── keymap.cpp                   # fcitx::Key → ime_key_v1 (chung khung với P3-1 §5.1)
│   ├── apply.cpp                    # §5 — preedit/commit/surrounding (dùng linux-common helpers)
│   └── instance_map.cpp             # IC* → ime_instance* (§3)
├── conf/vietime.conf.in             # addon descriptor cho fcitx5 (/usr/share/fcitx5/addon/)
├── conf/vietime.inputmethod?        # nếu cần đăng ký IM — verify ở spike LNX-006
└── tests/                           # unit keymap + instance lifecycle (link mock fcitx5 API)
```

- **Không** phụ thuộc IBus (2 addon song song, loại trừ nhau qua env/`vietime doctor` — `P3-4 §7`).

## 3. Vòng đời

```text
cài .deb → /usr/share/fcitx5/addon/vietime.conf + /usr/lib/fcitx5/libvietime-fcitx5.so
fcitx5 khởi động → load addon → vietime::AddonInstance init:
  1. EngineShared.ensure(): đọc config 1 lần (cache qua FileWatcher — P3-5 §3)
  2. ipc connect → GetSnapshot (offline nếu tray chưa mở — P0-3 §4)
focusIn(ic) / activate(ic):
  instance = instance_map.get_or_create(ic)      # ime_instance_new với config hiện tại
  AT-SPI resolve → ime_set_context{app_id, field_role, secure, caps}
  probe surrounding (nếu fcitx5 báo hỗ trợ) → caps hint
focusOut(ic) / deactivate(ic):
  preedit ≠ rỗng → COMMIT NGAY (commit-before-hide B2) → ime_reset(instance)
ic destroy → ime_instance_free + xóa khỏi map (không leak)
addon shutdown → free mọi instance còn lại, đóng socket
```

## 4. Key flow

```cpp
// Tên hàm chính xác theo API đã pin — verify LNX-006 (RL2); pseudocode:
bool keyEvent(engine, InputContext *ic, const fcitx::Key &key)
  0. linux-common injected marker (TLS)            → return false (không nuốt — §5.4)
  1. key.isControl/isAlt/isSuper + hotkey hệ thống → return false (B6)
  2. Ctrl+Shift+Space (ADR-011/LNX-018) toggle     → StateUpdate ipc + return true
  3. !key.check(KeyState::NoKeyState) key-up?      → return false (chỉ xử lý down)
  4. ch = keymap(key) theo layout hiện tại (xkbcommon — dùng chung hướng P3-1 §5.1)
  5. ime_key(inst, …)                              → 6. match action như P3-1 §5
     (PASS false / else apply §5 → true)
  7. mọi exception/-1 → false (fail-open S4)
```

- Fcitx5 tự xử lý `Commit`/`Backspace` cho app qua preedit → strategy giữ nguyên như IBus.

## 5. `apply_replace` (đối chiếu `P3-1 §6` — cùng semantics, API khác)

| Strategy | Fcitx5 API (concept) | Ghi chú |
|---|---|---|
| Preedit | `ic->inputPanel().setClientPreedit(text)` + `updatePreedit()` | underline attrs qua `TextFormatFlag::Underline` |
| SelectionReplace | `commitString(insert)` — commit thay selection (verify LNX-006: GTK/Qt/Chromium) | KHÔNG giả lập phím (B1) |
| BackspaceType | `ic->surroundingText().deleteAround(offset, len)` nếu probe OK; không OK → resolve downgrade | RL5, đối chiếu `P3-1 §6.3` |
| ForwardAsCommit | `commitString` không preedit | terminal (B8/B11) |
| Passthrough | trả false cho mọi phím | secure / EN mode |

`ime_result` → UTF-8/UTF-32 convert **dùng chung helper linux-common** (không viết 2 lần).

## 6. Link FFI từ C++ (RL10 — spike `LNX-003`)

```cmake
target_link_libraries(vietime-fcitx5 PRIVATE
    ${CMAKE_SOURCE_DIR}/../../target/release/libvietime_ffi.a      # hoặc .so (fallback)
    vietime-linux-common)
# FFI header có extern "C" sẵn (P0-2 §1) — chỉ cần include, không wrapper mới
```
- Trường hợp fcitx5 link `-fno-exceptions` / xung đột symbol → fallback dùng `cdylib`
  `libvietime_ffi.so` (P0-2 đã cho crate-type cả hai) — **không đổi FFI**.

## 7. Tương thích 2 adapter song song (điều phối IBus + Fcitx5)

```text
Quy tắc: CHỈ MỘT framework active tại 1 thời điểm (người dùng chọn trong Settings
         → state.json "linux_framework": "ibus"|"fcitx5"|"auto").
- "auto": detect qua env (QT_IM_MODULE/GTK_IM_MODULE trỏ tới fcitx5 hay ibus) — P3-4 §7
- preset engine_owner: "ibus" | "fcitx5" | "x11" (P0-3 §2.1)
- Khi framework B đang active → engine của framework A vẫn sống nhưng receive key = 0
  (fcitx5/ibus chỉ dispatch cho engine đang chọn) → không cần kill process.
- Corpus `linux/owner_no_double` kiểm 2 framework không cùng ăn 1 phím.
```

## 8. Spike checklist (task `LNX-006` — tuần 1–2)

| # | Câu hỏi | Kỳ vọng |
|---|---|---|
| F1 | Thêm addon fcitx5 rỗng (CMake theo template addon mẫu) → fcitx5 thấy, không crash | ✅ |
| F2 | API version nào sẵn trên Ubuntu 22.04/24.04/Fedora/Arch → chốt pin + macro | bảng version |
| F3 | `keyEvent` nhận đủ key (có key-up không? state flags?) | ghi |
| F4 | Preedit hiển thị qua classic/GTK/Qt UI | ✅ trong gedit + konsole |
| F5 | `commitString` thay selection? `surroundingText().deleteAround` hoạt động ở GTK/Qt? | bảng |
| F6 | Link `libvietime_ffi.a` + exceptions/symbols (RL10) | ✅ hoặc chốt fallback `.so` |
| F7 | 2 addon (IBus+Fcitx5) cùng cài trên 1 máy → conflict? | ghi cách xử lý (§7) |
| F8 | fcitx5 crash khi engine ném exception giữa keyEvent? (bẫy `catch(...)`) | 0 crash |

**Exit:** 8/8 trong `docs/specs/linux-fcitx5-spike.md`.

## 9. Task (chi tiết `P3-7-TASKS.md`)

| Task | Nội dung | Acceptance |
|---|---|---|
| LNX-006 | Spike F1–F8 (§8) | `linux-fcitx5-spike.md` đủ 8 mục |
| LNX-020 | Addon skeleton + lifecycle §3 | fcitx5 load/ unload 100 lần không leak (valgrind) |
| LNX-021 | keyEvent → PASS toàn bộ §4 | corpus `combo_pass` trên fcitx5 (konsole/gedit) |
| LNX-022 | Preedit + commit + word boundary | corpus `fcitx_preedit_*` ≥ 40 case |
| LNX-023 | SelectionReplace + BackspaceType/surrounding §5 | corpus `bug_B1_*`, `linux_bs_*` pass |
| LNX-024 | focus/commit-before-hide + instance_map | corpus `bug_B2_enter_commit` + `bug_B13_*` pass |
| LNX-025 | IpcClient + framework switch §7 | Đổi framework trong Settings → 2 adapter không đôi phím |

## 10. Failure modes

| Tình huống | Xử lý |
|---|---|
| Ném C++ exception trong callback | `try/catch(...)` từng callback → false + counter (không lan ra fcitx5 — RL7) |
| API khác version (RL2) | `#if FCITX_VERSION` + pin tối thiểu trong `Depends:` của .deb |
| Xung đột symbol / `-fno-exceptions` (RL10) | Fallback `libvietime_ffi.so` (P0-2 crate-type) |
| Surrounding không hỗ trợ (RL5) | probe → caps → resolve downgrade (giống P3-1) |
| Cả 2 framework cùng chạy | state `linux_framework` (§7) + corpus `owner_no_double` |
| fcitx5 bị user tắt giữa preedit | commit-before-hide ở `deactivate`/shutdown |
