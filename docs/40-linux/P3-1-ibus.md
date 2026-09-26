# P3-1 — IBUS ADAPTER (Linux) — Solution chi tiết

> WS1 · `adapters/linux-ibus/` (**C** — `PLAN §4.1`: "Linux IBus: C, chuẩn IBus; Rust bindings chưa ổn định").
> ADR-007: IBus + Fcitx5 dual; IBus là adapter primary trên GNOME (GNOME Wayland dùng IBus mặc định).
> Hợp đồng core: `../10-shared/P0-2-engine-ffi-contract.md`. Engine link `libvietime_ffi.a`
> (xác minh spike `LNX-003`, xem RL10) + `libvietime-linux-common.a` (AT-SPI — `P3-4 §5`).

## 1. Quyết định & capability

| Quyết định | Nội dung | Vì sao |
|---|---|---|
| Đi chuẩn IM protocol, **không grab phím** | Mọi text app qua `ibus-daemon` → engine process | `PLAN §5.2`, fix B10 (Wayland không cho grab) |
| IBus engine = **process riêng** (XML `<exec>`) | Crash của engine không giết ibus-daemon; daemon respawn | RL7; giống mô hình `vietime-hook.exe` tách process (P1-0 §2) |
| Caps của IBus v1 | `IME_CAP_PREEDIT \| IME_CAP_SELECTION \| IME_CAP_FIELD_DETECT` | Preedit/selection/native; field detect qua AT-SPI (`P3-4`) |
| 1 instance / context, 1 thread | Mỗi `IBusEngine` object (= 1 input context) có **1 `ime_instance` riêng** (callback cùng main loop → đúng P0-2 §3 thread-model); giải phóng khi `finalize` | P0-2 §3; multi-window không đạp nhau (B13) — verify số object ở spike LNX-002 |
| Gõ tắt/emoji/khôi phục EN | Toàn bộ trong core (không code riêng Linux) | Một implementation 3 OS (P0-2 §3) |

## 2. Cấu trúc module & repo

```
adapters/linux-ibus/
├── Cargo.toml-build/ (không có — thuần C)
├── meson.build | CMakeLists.txt        # build với pkg-config: ibus-1.0, glib-2.0
├── include/vietime_ibus_engine.h
├── src/
│   ├── main.c                          # ibus_main loop: connect bus, register component
│   ├── engine.c                        # IBusEngine subclass — mọi callback (§3–§6)
│   ├── engine.h
│   ├── keymap.c                        # keyval+state → ime_key_v1 (§5.1) — SPIKE LNX-004
│   ├── apply.c                         # §6 — nơi duy nhất sửa text (preedit/commit/surrounding)
│   └── ipc_client.c                    # unix socket → schema ipc.v1.md (dùng chung linux-common)
└── tests/unit/                         # keymap + apply với mock IBusText
```

- `libvietime-linux-common.a` (C, `adapters/linux-common/` — **dùng chung** với fcitx5/x11):
  AT-SPI field detect (`P3-4 §5`), IPC client, log (không text), env doctor helper.
- Engine **không** phụ thuộc GTK/Qt (chỉ glib + ibus) → chạy được trên mọi distro.

## 3. Vòng đời IBus

```text
cài .deb → /usr/share/ibus/component/vietime.xml → postinst: ibus restart (nếu daemon đang chạy)
ibus-daemon đọc XML → spawn /usr/lib/vietime/vietime-ibus-engine (process)
  main(): ibus_bus_new() → ibus_bus_register_component(bus, component) → ibus_main()
  IBusEngine.new(engine, connection, object_path)          # mỗi context 1 object → 1 instance (§1)
enable()  / focus_in(engine):
  1. engine_shared.ensure_instance(this): đọc ~/.config/VietIME/config.json (một lần/process, cache)
  2. ipc connect ~/.config/VietIME/ipc.sock → GetSnapshot (thất bại → offline — P0-3 §4)
  3. AT-SPI: focused element (thread riêng, cache) → ime_set_context{app_id=WM_CLASS/DESKTOP_FILE,
     field_role, secure, caps}
  4. đọc surrounding_text (nếu bus báo hỗ trợ) → capability hint (RL5)
focus_out(engine) / disable():
  nếu preedit đang có → COMMIT NGAY (commit-before-hide, fix B2) → ime_reset()
finalize(engine): free instance (không leak — P3-6 §7 soak)
```

- `focus_in/focus_out` = ranh giới context; switch app giữa chừng → commit + reset (fix B13).
- **Không** đăng ký timeout/idle nào không clear khi finalize (leak giữ engine sống mãi → `P3-6 §7` soak).

## 4. State machine (giống `P1-1 §4`, preedit thay composition)

```
       phím sinh text (REPLACE/COMMIT)
Idle ───────────────────────────────► Preedit
  ▲                                     │
  │ Space/Enter/động từ (WORD_END)      │ phím khác với preedit
  └──────── COMMIT (commit_text) ◄──────┘
  phím hệ thống/Ctrl+combo → không đổi state (PASS)
  focus_out/enable→disable → COMMIT + Idle (không mất chữ — B13/B2)
```

- `ibus_engine_get_surrounding_text` báo preedit bị app tự xóa → **self-heal**: `ime_reset` + Idle
  (không đoán text của app — cùng triết lý `P1-1 §4`).

## 5. Key flow

```text
gboolean process_key_event(IBusEngine *e, guint keyval, guint keycode, guint state, guint delay)
  0. injected_marker (linux-common: biến TLS đánh dấu XTEST/commit tự tạo) → return FALSE  (§6.4)
  1. state & (IBUS_CONTROL_MASK|IBUS_MOD1_MASK|IBUS_SUPER_MASK) + keyval là hotkey
     hệ thống (đổi workspace, Alt+Tab, Ctrl+C…)                       → return FALSE (B6)
  2. Ctrl+Shift+Space (hoặc CapsLock mode — ADR-011/LNX-018) → toggle EN/VN
     → broadcast StateUpdate qua ipc + ibus_engine_property_update      → return TRUE
  3. state & IBUS_RELEASE_MASK → return FALSE   (engine chỉ xử lý key-down)
  4. ch/vk = keymap.c translate (§5.1); secure/enabled/app_id từ cache context
  5. r = ime_key(inst, {vk=keyval→Vk, ch, mods, key_down=1}, &out)
  6. match out.action:
       PASS     → return FALSE
       REPLACE  → apply_replace(e, &out, ctx)  → return TRUE   (§6)
       COMMIT   → commit_text(utf32→String(out.insert))         → return TRUE
       RESTORE  → apply_replace (delete_count + insert)          → return TRUE
  7. mọi lỗi C (glib warning, -1 từ FFI) → return FALSE (fail-open — S4)
```

### 5.1 Translate `keyval` → `ime_key_v1` (SPIKE `LNX-004` — RL1)

```text
- keyval là X keysym; VERIFIED spike: IBus truyền keyval ĐÃ apply modifier hay chưa
  (test: shift+a → 'a' hay 'A'?). ghi vào docs/specs/linux-keyval-spike.md.
- Cách robust (chọn sau spike): giữ keycode+state → translate bằng xkbcommon
  (xkb_state_key_get_utf32 với keymap hiện tại) — weak feature detection (PLAN §5.3);
  fallback: xkb_keysym_to_utf32(keyval) + tự cộng shift cho letters.
- keyval >= 0x20 printable → ch; special: BackSpace=0xff08, Return=0xff0d, Escape=0xff1b,
  Tab=0xff09, space=0x20 → vk tương ứng (bảng keymap.c, unit test 100 phím).
- Ctrl/Alt/Super giữ trong mods; keyval chữ HOA vẫn cho engine (engine tự case).
```

## 6. `apply_replace` — cách sửa text trên IBus (P0-2 §7 trỏ tới đây)

### 6.1 `Preedit` (mặc định)
```text
ReplaceEditPlan (chỉ preedit):
  IBusText *t = ibus_text_new_from_string(preedit); + attrs UNDERLINE_SINGLE
  ibus_engine_update_preedit_text(e, t, len, TRUE); ibus_engine_show_preedit_text(e)
  khi WORD_END → §6.4 commit
```

### 6.2 `SelectionReplace` (bug B1 — thanh địa chỉ/ô tìm kiếm)
```text
  if (surrounding có sẵn && selection ≠ rỗng):            # đọc từ surrounding_text
      ibus_engine_delete_surrounding_text(e, -sel_len, 0)  # xóa selection (nếu app hỗ trợ)
      commit_text(insert)                                   # HOẶC chỉ commit_text nếu
                                                            # commit đã tự thay selection
  else:
      commit_text(insert)   # GTK/Qt/Chromium: commit thay selection mặc định — verify LNX-004
  # KHÔNG bao giờ giả lập Backspace → autocomplete không bị kích hoạt lại (cùng lý do P1-1 §6.2)
```

### 6.3 `BackspaceType` (dùng khi không có preedit/selection — fallback)
```text
  if (đã probe được surrounding mode):
      ibus_engine_delete_surrounding_text(e, -delete_count, 0)   # không cần inject phím
  else:
      strategy fallback đã bị downgrade ở resolve (P0-3 §3.1) → chỉ được dùng Preedit/Commit;
      nếu preset bắt BackspaceType mà không surrounding → commit trước (không bao giờ
      giả lập phím qua XTEST trong adapter IBus — XTEST chỉ nằm ở vietime-x11, P3-3)
```
- RL5: probe surrounding tại `focus_in` (bus có `surrounding-text` capability không, app có
  set không) → giữ trong `ctx.caps` — không hỗ trợ → resolve downgrade (P0-3 §3.1).

### 6.4 Chèn text (dùng chung)
```text
commit_text: IBusText từ UTF-32 (khớp ime_result.insert, ime_result.insert_len)
  - surrogate pair + emoji: dùng g_utf8 + gunichar → unit test 100 chuỗi
  - sau khi commit: nếu preedit visible → hide_preedit + ime_reset (word boundary)
```

## 7. Commit-before-hide & B2 (Lặp từ cuối khi Enter — bamboo-viet)

| Sự kiện | Hành động |
|---|---|
| `focus_out` khi preedit ≠ rỗng | COMMIT ngay (Enter trong chat → không mất/ lặp chữ) |
| `disable` (user tắt IM giữa chừng) | COMMIT + reset |
| Enter/Space word boundary | COMMIT phần đầu + giữ buffer (như TSF `P1-1 §3`) |
| App tự xóa preedit (báo qua surrounding lệch) | self-heal `ime_reset` |

Corpus: `bug_B2_enter_commit`, `bug_B13_focus_loss` (`P3-6 §2`).

## 8. Component XML & đăng ký

```xml
<!-- /usr/share/ibus/component/vietime.xml (spec đóng gói — P3-0 §2) -->
<component>
  <name>VietIME</name><exec>/usr/lib/vietime/vietime-ibus-engine</exec>
  <version>1.0</version><author>VietIME contributors</author>
  <license>GPL-3.0-or-later</license><homepage>…</homepage>
  <textdomain>vietime</textdomain>
  <engines><engine>
    <name>vietime</name><longname>VietIME (Telex/VNI/VIQR)</longname>
    <language>vi</language><layout>us</layout><symbol>VN</symbol><rank>0</rank>
    <icon>vietime</icon>
  </engine></engines>
</component>
```

```text
postinst .deb: if pgrep ibus-daemon → ibus restart (ignore fail) + in hướng dẫn:
  GNOME: Settings → Keyboard → Input Sources → Add "VietIME"
  KDE: System Settings → Input Method → Add IBus → VietIME
gỡ: xóa XML + binary → ibus restart — 0 residue (P3-6 §6)
```
- `vietime doctor` (P3-5 §6) kiểm: XML tồn tại? daemon chạy? env `GTK_IM_MODULE` đúng?

## 9. Spike checklist (tasks `LNX-002/003/004/005/008` — tuần 1–2, chặn WS1)

| # | Câu hỏi | Kỳ vọng | Task |
|---|---|---|---|
| S1 | Project IBus engine C tối thiểu (mẫu `ibusengine` của ibus) gõ được, preedit hiện trong gedit | ✅ demo | LNX-002 |
| S2 | Đúng component XML → hiện trong Input Sources | ✅ | LNX-002 |
| S3 | Link `libvietime_ffi.a` từ **C** (cmake/meson + pkg-config) → gọi `ime_key`, `vietime sizes` pass | ✅ | LNX-003 |
| S4 | **keyval/shift semantics** thật (§5.1) — 3 layout | ghi rõ | LNX-004 |
| S5 | `commit_text` có tự thay **selection** trong GTK Entry/Chromium address bar? | ✅/ghi | LNX-004 |
| S6 | `delete_surrounding_text` hoạt động ở GTK/Qt/Chromium? (RL5) | bảng kết quả | LNX-004 |
| S7 | AT-SPI từ engine process: hỏi focused element → có bị a11y từ chối? cách prompt? | ghi rõ | LNX-005 |
| S8 | Password field: engine có nhận key không? (GTK tắt IM ở input-purpose password) | ghi + verify | LNX-005 |
| S9 | GHA `ubuntu-latest` + xvfb: build + `replay corpus` + AT-SPI demo app | ✅/ghi → RM5-P3 | LNX-008 |
| S10 | Engine process bị kill giữa preedit → ibus-daemon respawn bao nhanh? mất chữ dở? | ghi | LNX-008 |

**Exit:** 10/10 có kết quả trong `docs/specs/linux-spike.md`. S4/S6 fail → điều chỉnh caps/strategy trong `P3-4 §1`.

## 10. Playbook triển khai (mapping `P3-7-TASKS.md`)

| Bước | Task | Nội dung | Acceptance tóm tắt |
|---|---|---|---|
| 1 | LNX-001..009 | Env + 7 spike + corpus linux đầu | `docs/specs/linux-spike.md` đủ 10 mục §9 |
| 2 | LNX-010 | Component + engine rỗng | Cài vào Input Sources, gedit không crash |
| 3 | LNX-011 | process_key_event → PASS toàn bộ | corpus `combo_pass` trên gedit/Firefox |
| 4 | LNX-012 | Preedit + commit ngắn (§6.1) | `duocj` → `được`; corpus `ibus_preedit_*` ≥ 40 case |
| 5 | LNX-013 | keymap (§5.1) + layout đổi | Unit 100 phím; layout vi/typewriter OK |
| 6 | LNX-014 | SelectionReplace §6.2 | corpus `bug_B1_*` (Firefox/Chrome/ô tìm kiếm) pass |
| 7 | LNX-015 | BackspaceType qua surrounding §6.3 | corpus `linux_bs_*` ≥ 30 case |
| 8 | LNX-016 | focus_in/out + commit-before-hide (§7) | corpus `bug_B2_enter_commit`, `bug_B13_*` pass |
| 9 | LNX-017 | Secure/password → `secure=1` (§S8) | corpus `secure_field_passthrough` pass |
| 10 | LNX-018 | Toggle EN/VN + hotkey + property menu | corpus `restore_en_*` pass (B5) |
| 11 | LNX-019 | IpcClient unix socket (dùng linux-common) | Đổi method trong Settings → gõ đổi <1s |

## 11. Chẩn đoán

- `VIETIME_LOG=1` → `~/.local/state/VietIME/log/ibus-<pid>.log` (state, action, **độ dài** — không text, S2).
- Tray → "Sức khỏe": engine PID + heartbeat, AT-SPI permission, socket state.
- `vietime doctor` (P3-5 §6): component XML? env matrix? daemon state? (task LNX-055).

## 12. Failure modes

| Tình huống | Xử lý |
|---|---|
| `ime_key` lỗi/panic | `catch_unwind` trong FFI → PASS + crash counter → tray cảnh báo (RL7) |
| Surrounding không hỗ trợ (RL5) | probe ở focus_in → `ctx.caps` thiếu → resolve downgrade (P0-3 §3.1) |
| App từ chối preedit (báo surrounding lệch) | self-heal reset → preset override `ForwardAsCommit` cho app đó |
| Socket chết/tray không chạy | Offline với config đã đọc — không block (P0-3 §4) |
| ibus-daemon restart / logout | engine process chết theo → respawn khi chọn input source |
| env sai (`GTK_IM_MODULE`…) | vẫn gõ được GTK (GNOME trực tiếp IBus); app không thấy → `doctor` đề nghị fix (RL9) |
