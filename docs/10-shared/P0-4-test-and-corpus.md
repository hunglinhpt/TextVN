# P0-4 — Test strategy · Định dạng corpus `.keys` · Replay simulator

> Corpus là **tài sản quan trọng nhất** của dự án: mọi bug kinh niên (B1–B13) phải được
> mã hoá thành corpus trước khi fix (Handbook §9). Tài liệu này là **spec chính thức** của định dạng —
> parser nằm ở `cli/src/replay.rs` và **phải bám sát 100%**.

## 1. 7 lớp test (chi tiết trong PLAN §4.3, bổ sung chỗ chạy)

| Lớp | Nội dung | Lệnh | Chạy khi nào |
|---|---|---|---|
| L1 Unit + property | core, config, strategy, appdb | `cargo test --workspace` | mọi PR |
| L2 **Golden corpus** | `.keys` → simulator → so `:expect` | `cargo run -p textvn-cli -- replay corpus/shared` | mọi PR |
| L3 Cross-OS conformance | cùng corpus, 3 adapter mode (headless sim) | `replay corpus/shared corpus/{win,mac,linux}` trên 3 CI OS | mọi PR |
| L4 Fuzz | `ffi_key`, `config_parse`, `appdb_parse` | `cargo fuzz run ffi_key --features ffi-fuzz -- -max_total_time=60` (tương tự `config_parse`); `cargo fuzz run appdb_parse -- -max_total_time=60` | PR chạm parser; 10p nightly |
| L5 **App-compat automation** | gõ thật vào app qua UIA/AX/AT-SPI | `tools/appcomptest` (P1-5) | PR chạm adapter; nightly 12 app |
| L6 Soak | replay 24h + leak check | `nightly.yml` | nightly |
| L7 Manual matrix | 40 app × 3 OS checklist | `docs/compat.md` | trước release |

**Trật tự khi dev:** L2 phải có TRƯỚC khi viết code fix (reproduces trước, fix sau).

## 2. Định dạng corpus `.keys`

### 2.1 Tổng quan

- File UTF-8, phần mở rộng `.keys`, **mỗi dòng 1 lệnh**, `#` bắt đầu dòng = comment.
- Lệnh đều bắt đầu bằng `:`. Lệnh không hợp lệ → **replay exit 2** (báo file, dòng — không được "im lặng bỏ qua").
- Kết quả assertion ngay trong file → không cần file `.expect` riêng.

### 2.2 Bảng lệnh (chính thức)

| Lệnh | Ví dụ | Ý nghĩa |
|---|---|---|
| `:config k=v [k=v...]` | `:config method=telex diacritic_style=new` | Set option engine cho case này (cùng field với `config.v1.json`) |
| `:app <app_id> field=<role>` | `:app chrome.exe field=address_bar` | Set `ime_context.app_id` + `field_role` (bảng role: P0-3 §2.1) |
| `:caps <flags>` | `:caps preedit,selection,field_detect,inject_vk` | Set `ime_context.caps` (mặc định: `field_detect,inject_vk` — **không** có preedit) |
| `:enabled on\|off` | `:enabled off` | `ime_context.enabled` |
| `:secure on\|off` | `:secure on` | `ime_context.secure` → mọi phím sau PASS tới khi `:secure off` |
| `:type "<chuỗi>"` | `:type "dduocj"` | Gõ từng ký tự (mỗi ký tự = key down/up có `ch`) |
| `:key <tên>` | `:key Enter` · `:key Escape` · `:key Backspace` · `:key Tab` · `:key Space` | Phím đặc biệt (danh sách §2.4) |
| `:combo <chuỗi phím tắt>` | `:combo Ctrl+Shift+Space` | Phím tắt modifier — luôn kỳ vọng `PASS` (nếu engine nuốt = FAIL) |
| `:expect "<chuỗi>"` | `:expect "được"` | So nội dung buffer mô phỏng (§3) tại thời điểm này |
| `:expect_preedit "<chuỗi>"` | `:expect_preedit "đượ"` | So `preedit[]` do engine trả về lần cuối |
| `:expect_action <PASS\|REPLACE\|COMMIT\|RESTORE>` | `:expect_action PASS` | Action của lần `:key/:type` **tiếp theo** — dùng cho test đơn phím |
| `:expect_cursor <n>` | `:expect_cursor 5` | Vị trí con trỏ trong buffer |
| `:reset` | `:reset` | Gọi `ime_reset()` (mô phỏng focus change) |
| `:engine_new` | `:engine_new` | Tạo instance mới (test load config) |

### 2.3 Ví dụ đầy đủ (phải chạy được ngay)

```keys
# corpus/shared/telex_basic_01.keys
:config method=telex diacritic_style=new
:app notepad.exe field=body
:type "dduocj"
:expect "được"
:expect_preedit "được"

# corpus/win/chrome_address_bar_01.keys  (bug B1)
:config method=telex
:caps field_detect,inject_vk,selection     # adapter này có UIA + chọn vùng
:app chrome.exe field=address_bar
:type "viet"
:expect "viêt"
:key Space
:type "nam"
:expect "viêt nam"

# corpus/win/secure_field_passthrough.keys  (S3)
:config method=telex
:app browser.exe field=secure
:secure on
:type "ass"
:expect "ass"
:expect_action PASS

# corpus/shared/injected_no_loop.keys  (P0-3 §6)
:type "abc"
:reset
# phím do adapter tự bơm (is_injected=1) phải PASS nguyên vẹn
:injected "s"
:expect "abc"
```

### 2.3b Lệnh bổ sung (bắt buộc có trong parser)

| Lệnh | Ý nghĩa |
|---|---|
| `:injected "<chuỗi>"` | Gõ với `ime_key.is_injected = 1` → **bắt buộc** engine PASS (chống loop) |
| `:mods +Shift` / `:mods -Shift` | Giữ/nhả modifier cho `:key` kế tiếp |
| `:note "<lý do>"` | Ghi chú hiển thị trong báo cáo fail (không ảnh hưởng kết quả) |

### 2.4 Tên phím chuẩn (case-sensitive)

`Space` · `Enter` · `Escape` · `Backspace` · `Tab` · `Delete` · `Left` `Right` `Up` `Down`
`Shift` `Ctrl` `Alt` `Super` `CapsLock` · `F1`…`F12`
Chữ/số/ký tự gõ trực tiếp trong `:type "..."` (UTF-8). Ký tự đặc biệt trong `:type` dùng escape
`\"`, `\\`, `\n` (Enter).

## 3. Simulator semantics (`cli/src/replay.rs`) — mô hình tính toán

Buffer mô phỏng = `Vec<char>` + cursor. Với **mỗi** phím vào engine:

```
result = ime_key(inst, key, &out)
match out.action:
  PASS      → nếu key có ký tự in được: chèn ký tự đó vào cursor (xử lý cả Enter/Space/Bksp
              bằng chuẩn: Backspace xoá trước cursor, Delete xoá tại cursor)
  REPLACE   → xóa out.delete_count ký tự TRƯỚC cursor; chèn out.insert[0..insert_len]
  COMMIT    → chèn out.insert[0..insert_len] (không xóa); preedit cleared
  RESTORE   → như REPLACE
preedit hiện tại = out.preedit[] (để `:expect_preedit` so)
```

- Nếu `:expect` fail → in **diff rõ ràng**: `expected: "..." / actual: "..."` + context 3 dòng quanh.
- Cursor clamp trong [0, len]; thao tác vượt → FAIL test (đừng mask bug bằng clamp im lặng).
- Simulator **không** mô phỏng app-specific behavior (autocomplete…) — cái đó là việc của L5 (P1-5).

## 4. Lệnh CLI & exit code

```bash
textvn replay <dir-or-file...> [--adapter headless|win|mac|linux] [--json] [--filter <substring>]
```

- `--adapter` = **profile mô phỏng strategy/capability** của adapter thật (không cần OS thật):
  `headless` = engine thuần; `win` = capability TSF/Hook + preset Windows (`P1-3 §1`);
  `mac`/`linux` = tương ứng Phần 2/3. Giá trị khác → exit 2.

| Exit | Ý nghĩa |
|---|---|
| `0` | tất cả pass |
| `1` | có case fail (in diff từng case) |
| `2` | lỗi dùng/lỗi parse file corpus (danh sách file+dòng) |

`--json` in kết quả theo case cho CI report (id, status, expected, actual, ms).

## 5. Quy ước thư mục & đặt tên corpus

```
corpus/
├── shared/        # hành vi cốt lõi không phụ thuộc OS
│   ├── telex_*.keys · vni_*.keys · viqr_*.keys
│   ├── undo_*.keys        (ass→as, ww→w)
│   ├── restore_en_*.keys  (B5)
│   ├── macro_*.keys · emoji_*.keys · caps_*.keys
│   └── loop_guard_*.keys · secure_*.keys · combo_pass_*.keys (B6)
├── win/           # case Windows (cùng định dạng; `:app` dùng exe Windows)
│   ├── bug_B1_chrome_address_bar_*.keys
│   ├── bug_B2_chat_enter_*.keys
│   ├── bug_B9_*.keys ...
│   └── preset_override_*.keys
├── mac/           # (Phần 2)
└── linux/         # (Phần 3)
```

**Tên file:** `{chủ-đề}_{mô-tả}_{số}.keys`; case reproduces bug B{n} **bắt buộc** tên chứa `bug_B{n}`.

## 6. Definition of Done cho lớp test

- [ ] Mọi lệnh trong §2.2/§2.3b có parser + unit test (kể cả lệnh sai → exit 2).
- [ ] Corpus `shared/` ≥ 200 case và **đối chiếu oracle UniKey** (`docs/specs/oracle-unikey.md` mô tả cách build x-unikey CLI để sinh kết quả mong đợi — xem Handbook §9).
- [ ] CI chạy `replay corpus/shared` trên 3 OS; fail = đỏ build.
- [ ] Fuzz target `config_parse`/`appdb_parse` không panic 60s.
- [ ] `docs/compat.md` có hướng dẫn map bug → corpus → fix.
