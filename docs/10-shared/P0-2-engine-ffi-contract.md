# P0-2 — Hợp đồng FFI v1 (nguồn sự thật duy nhất)

> Mọi adapter (Windows/macOS/Linux) và mọi test harness giao tiếp với engine **chỉ** qua ABI này.
> Header chính thức: `ffi/include/vietime_ffi.h` (generate bằng `cargo xtask cbindgen`, CI có test
> `abi_size` đối chiếu). Tài liệu này giải thích **ngữ nghĩa** — khi mâu thuẫn, **header + test size thắng**.

## 0. Bất biến (invariants)

1. **Không alloc chéo ranh giới**: mọi `*_v1` do caller cấp phát (trên stack của adapter); Rust không trả pointer chứa dữ liệu người dùng ra ngoài.
2. **Không callback** từ Rust sang adapter trong v1 (tránh vòng đời COM/Swift/IBus chết giữa chừng).
3. **Fail-open**: `ime_key()` không bao giờ "không trả lời". Lỗi → trả `IME_OK` với `action=IME_ACTION_PASS` + cờ `IME_FLAG_ERROR`.
4. **Không network, không I/O, không global mutable state** trong instance (ngoại trừ bảng const).
5. **Không ghi text người dùng vào `ime_last_error()`**.

## 1. Header (bản 1:1 với `ffi/include/vietime_ffi.h`)

```c
/* vietime_ffi.h — VietIME engine C ABI v1
 * SPDX-License-Identifier: GPL-3.0-or-later
 */
#ifndef VIETIME_FFI_H
#define VIETIME_FFI_H
#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define IME_ABI_VERSION      1u
#define IME_MAX_TEXT         64      /* UTF-32 code points */
#define IME_MAX_SUGGEST      8

/* ---- error codes ---- */
#define IME_OK                0
#define IME_ERR_INVALID_ARG  -1
#define IME_ERR_CONFIG       -2
#define IME_ERR_ABI          -3
#define IME_ERR_INTERNAL     -4      /* đã nuốt panic; engine ở trạng thái fail-open */

/* ---- action ---- */
#define IME_ACTION_PASS     0u  /* không biến đổi: cho app xử lý phím bình thường      */
#define IME_ACTION_REPLACE  1u  /* xóa delete_count ký tự trước con trỏ, chèn insert[] */
#define IME_ACTION_COMMIT   2u  /* giữ nguyên preedit hiển thị, chèn insert[] và đóng  */
#define IME_ACTION_RESTORE  3u  /* gỡ biến đổi trước đó (auto-restore tiếng Anh)       */

/* ---- result flags ---- */
#define IME_FLAG_CONSUMED   0x1u  /* phím đã bị engine nuốt (adapter không forward)   */
#define IME_FLAG_WORD_END   0x2u  /* sau action này là ranh giới từ (macro/space...)  */
#define IME_FLAG_ERROR      0x4u  /* engine gặp lỗi → adapter nên treat như PASS      */
#define IME_FLAG_SUGGEST    0x8u  /* có gợi ý sẵn (chỉ khi feature "suggest" bật)     */

/* ---- key mods ---- */
#define IME_MOD_SHIFT 0x1u
#define IME_MOD_CTRL  0x2u
#define IME_MOD_ALT   0x4u
#define IME_MOD_SUPER 0x8u
#define IME_MOD_META  0x10u   /* macOS Cmd */
#define IME_MOD_CAPS  0x20u
#define IME_MOD_FN    0x40u

/* ---- field roles (adapter điền từ OS API: UIA / AX / AT-SPI) ---- */
#define IME_FIELD_UNKNOWN    0u
#define IME_FIELD_BODY       1u   /* vùng soạn thảo thông thường                     */
#define IME_FIELD_EDITBOX    2u   /* ô input đơn dòng                                 */
#define IME_FIELD_ADDRESS_BAR 3u   /* thanh địa chỉ / combo có autocomplete           */
#define IME_FIELD_SEARCH     4u   /* ô tìm kiếm có gợi ý                              */
#define IME_FIELD_COMBO      5u   /* combobox / dropdown                              */
#define IME_FIELD_CANDIDATE  6u   /* ô gợi ý của chính app (Excel cell, IDE complete) */
#define IME_FIELD_TEXTAREA   7u
#define IME_FIELD_WEB        8u   /* contenteditable trong browser                    */
#define IME_FIELD_TERMINAL   9u
#define IME_FIELD_SECURE     10u  /* mật khẩu → engine PHẢI PASS ngay                 */

/* ---- adapter capabilities (adapter điền trước khi resolve strategy) ---- */
#define IME_CAP_PREEDIT     0x1u  /* adapter có hiển thị preedit/gạch chân được        */
#define IME_CAP_SELECTION   0x2u  /* adapter chọn/xoá vùng được (Shift+Left...)        */
#define IME_CAP_FIELD_DETECT 0x4u  /* adapter detect được field_role (UIA/AX/AT-SPI)    */
#define IME_CAP_INJECT_VK   0x8u  /* adapter gửi được VK thật (không chỉ Unicode)      */

/* ---- strategy ids (dùng cho ime_context.hint; JSON side dùng tên chuỗi) ---- */
#define IME_STRATEGY_PREEDIT           0u  /* TSF composition / IBus preedit             */
#define IME_STRATEGY_BACKSPACE_TYPE    1u  /* xóa bằng Backspace rồi gõ lại              */
#define IME_STRATEGY_SELECTION_REPLACE 2u  /* Shift+Left×n rồi chèn (field autocomplete)  */
#define IME_STRATEGY_FORWARD_AS_COMMIT 3u  /* gõ thẳng, không xóa (terminal)             */
#define IME_STRATEGY_PASSTHROUGH       4u  /* bỏ qua hoàn toàn                           */
/* hint == -1  =>  để engine tự resolve (xem P0-3 §3.1) */

/* ---- structs ---- */
typedef struct ime_instance ime_instance;   /* opaque */

typedef struct ime_key {
  uint32_t abi_version;   /* = IME_ABI_VERSION                                        */
  uint32_t vk;            /* virtual key đã normalize (Win VK / mac keycode / keysym) */
  uint32_t ch;            /* ký tự Unicode sau layout; 0 nếu không sinh ký tự          */
  uint32_t mods;          /* IME_MOD_*                                                 */
  uint8_t  key_down;      /* 1 = down, 0 = up                                          */
  uint8_t  is_repeat;
  uint8_t  is_injected;   /* adapter tự bơm phím này trước đó → engine BỎ QUA (chống loop) */
  uint8_t  _reserved;
} ime_key_v1;             /* 20 bytes */

typedef struct ime_context {
  uint32_t     abi_version;
  uint32_t     enabled;          /* 1 = đang ở chế độ tiếng Việt cho context này      */
  uint32_t     secure;           /* 1 => engine trả PASS bất kể enabled              */
  uint32_t     field_role;       /* IME_FIELD_*                                       */
  uint32_t     caps;             /* IME_CAP_* mà adapter hỗ trợ                       */
  /* từ đây chứa con trỏ → kích thước struct PHỤ THUỘC platform (x86/x64),
   * KHÔNG assert size; chỉ 2 struct không chứa con trỏ được enforce:
   * ime_key_v1 (20) và ime_result_v1 (532). */
  const char  *app_id;           /* UTF-8, 0-terminated: "chrome.exe"/"com.google.Chrome"/class */
  const char  *element_name;     /* UTF-8 hoặc NULL — tên element (dùng để detect role) */
  int64_t      hint;             /* strategy adapter gợi ý: 0..4 (Preedit..Passthrough),
                                    -1 = để engine tự resolve theo §P0-3 */
} ime_context_v1;

typedef struct ime_result {
  uint32_t abi_version;
  uint32_t action;               /* IME_ACTION_*                                      */
  uint16_t delete_count;          /* số ký tự phía TRƯỚC con trỏ cần xóa              */
  uint16_t insert_len;
  uint16_t preedit_len;          /* preedit cần hiển thị (0 nếu adapter không preedit) */
  uint16_t _reserved;
  uint32_t flags;                /* IME_FLAG_*                                        */
  uint32_t insert[IME_MAX_TEXT]; /* UTF-32                                             */
  uint32_t preedit[IME_MAX_TEXT];/* UTF-32                                             */
} ime_result_v1;                 /* sizeof == 532 (CI enforce) */

typedef struct ime_suggest {     /* feature "suggest" — nếu adapter không dùng: truyền NULL */
  uint32_t abi_version;
  uint32_t count;                                   /* 0..IME_MAX_SUGGEST */
  uint32_t lens[IME_MAX_SUGGEST];
  uint32_t items[IME_MAX_SUGGEST][IME_MAX_TEXT];    /* UTF-32, 0-terminated */
} ime_suggest_v1;

/* ---- API ---- */
uint32_t     ime_abi_version(void);                     /* luôn IME_ABI_VERSION của lib */
/* config sai schema: VẪN tạo instance với config mặc định, trả IME_ERR_CONFIG (non-fatal)
 * và ghi chi tiết (không chứa text người dùng) vào ime_last_error(). *out luôn được set. */
int32_t      ime_instance_new(const uint8_t *config_utf8, size_t len, ime_instance **out);
void         ime_instance_free(ime_instance *inst);
int32_t      ime_set_context(ime_instance *inst, const ime_context_v1 *ctx);
int32_t      ime_key(ime_instance *inst, const ime_key_v1 *k, ime_result_v1 *out);
int32_t      ime_reset(ime_instance *inst);             /* xóa buffer (word boundary/focus change) */
int32_t      ime_reload_config(ime_instance *inst, const uint8_t *cfg, size_t len);
int32_t      ime_suggest(ime_instance *inst, const uint32_t *word, uint32_t word_len,
                         ime_suggest_v1 *out);          /* trả IME_ERR_INTERNAL nếu feature tắt */
const char  *ime_last_error(const ime_instance *inst);  /* UTF-8, hợp lệ đến lần gọi kế tiếp
                                                           trên CÙNG instance; không chứa text
                                                           người dùng */
#ifdef __cplusplus
}
#endif
#endif /* VIETIME_FFI_H */
```

## 2. Ngữ nghĩa `action` — adapter PHẢI làm đúng bảng này

| `action` | Adapter xử lý | Ghi chú |
|---|---|---|
| `PASS` | Forward phím nguyên vẹn cho OS/app | Include cả `flags & IME_FLAG_ERROR` |
| `REPLACE` | 1) Xóa `delete_count` ký tự phía trước con trỏ 2) Chèn `insert[0..insert_len]` 3) Cập nhật preedit = `preedit[]` (nếu adapter dùng preedit thì **không** thao tác xóa/chèn trực tiếp với app — xem §4) | `delete_count` luôn ≤ số ký tự engine tin là đã gõ trong context hiện tại |
| `COMMIT` | Chèn `insert[]` (không xóa), đóng composition/preedit | Dùng cho Enter-in-chat (bug **B2**: commit-before-hide) |
| `RESTORE` | Xóa `delete_count`, chèn `insert[]` = chuỗi gốc | Bug **B5**: ESC/Space auto-restore tiếng Anh |

**Quy tắc chung:** mọi thao tác thay thế text phải **tối thiểu hóa round-trip** (xem chiến lược trong `P0-3`).
`insert_len`/`preedit_len` luôn ≤ `IME_MAX_TEXT`; adapter kiểm tra trước khi dùng.

## 3. Vòng đời & thread model

```
ime_instance_new(cfg) ──►ime_set_context(ctx) ──►ime_key()* ──►ime_reset() (focus change)
        │                                                                  │
        └──ime_reload_config() (luồng khác được phép? KHÔNG — gọi từ luồng sở hữu)
        └──ime_instance_free()  (sau khi chắc không còn ime_key nào chạy)
```

- **1 instance = 1 thread.** `ime_key/ime_set_context/ime_reset` KHÔNG thread-safe.
  - Windows TSF: instance tạo trong `Activate()` của TIP (mỗi thread làm việc 1 instance) — xem `P1-1`.
  - Hook: 1 instance duy nhất, mọi truy cập từ luồng hook (callback của hệ điều hành).
  - Tray/CLI: instance riêng cho replay/preview.
- `ime_reload_config` phải chờ mọi `ime_key` xong (mutex **bên adapter**, engine không tự lock).
- `ime_instance_free(NULL)` là no-op (giản lược adapter khỏi cần check).

## 4. Mapping sang adapter preedit-vs-replace (quan trọng)

Adapter có 2 cách hiện chữ; **chọn theo strategy** (P0-3 §3):

| Strategy | Dùng `preedit[]`? | Dùng `delete_count/insert[]`? |
|---|---|---|
| `Preedit` (TSF composition, IBus preedit) | ✅ hiển thị `preedit[]` dưới gạch chân | Chỉ khi `action=COMMIT/RESTORE` |
| `BackspaceType` (hook) | ❌ | ✅ `delete_count` × Backspace rồi gõ `insert[]` |
| `SelectionReplace` (field autocomplete) | ❌ | ✅ xóa bằng Shift+Left × n rồi chèn (không gửi Backspace để app không gợi ý lại) |
| `ForwardAsCommit` (terminal) | ❌ | ✅ gõ `insert[]` ngay, không xóa gì cả (xem P1-2 §5) |

**Preedit contract:** sau mỗi `ime_key` trả `REPLACE`, `preedit[]` là **chuỗi hoàn chỉnh** adapter phải hiển thị
(không phải delta). `COMMIT` = preedit hiện tại trở thành text vĩnh viễn (adapter tự chép `preedit[]` nếu `insert_len==0`).

## 5. Mã lỗi & hành vi khi lỗi

| Trả về | Ý nghĩa | Adapter xử lý |
|---|---|---|
| `IME_OK` | Hợp lệ (kể cả `IME_FLAG_ERROR`) | Đọc `out->action` |
| `IME_ERR_INVALID_ARG` | tham số NULL/sai size | Log (không text), treat mọi phím là PASS cho tới `ime_reset` |
| `IME_ERR_CONFIG` | config không hợp lệ schema | **Không fatal**: engine đã tự chạy default (xem §1); log `ime_last_error`, hiện banner trong tray, **tiếp tục chạy** |
| `IME_ERR_ABI` | `abi_version` của struct đầu vào không khớp lib | Không dùng instance; hiện cảnh báo trong `vietime doctor` |
| `IME_ERR_INTERNAL` | panic đã bị `catch_unwind` nuốt | Gọi `ime_reset()`, fail-open, tạo crash counter → tray cảnh báo |

> Panic policy: `panic = "unwind"` ở `vietime-ffi` + `catch_unwind` quanh mọi entry point (đây là lý do
> **không** đặt `panic=abort` cho staticlib).

## 6. Versioning & kiểm chứng bằng CI

1. Sửa struct/enum = **bump `IME_ABI_VERSION`** và thêm `*_v2` (không sửa v1 tại chỗ — giữ nhịp:
   adapter 3 OS update theo 1 lịch, xem `P0-1 §5`).
2. Test Rust trong `ffi`:
   `assert_eq!(size_of::<ime_result_v1>(), 532);` · `assert_eq!(size_of::<ime_key_v1>(), 20);`
   + `offset_of!` từng field (chống đổi thứ tự field ngoài ý muốn).
3. Test C: `tests/conformance/abi_size.c` compile header thật, `_Static_assert(sizeof(ime_result_v1)==532)`.
4. `cargo xtask cbindgen` + `git diff --exit-code` → header không được lệch so với source.
5. Fuzz targets: `fuzz/ffi_key` (nguồn: `ime_key` với byte ngẫu nhiên + corpus replay cũ),
   `fuzz/config_parse`, `fuzz/appdb_parse`. **PR chạm `ffi/` phải xanh fuzz smoke 60s.**

## 7. Adapter walkthrough (Windows TSF — mẫu để adapter khác làm theo)

```
ITfKeystrokeMgr::OnKeyDown / OnTestKeyDown
  └─ if (ctx.secure || !ctx.enabled || chord hệ thống)      → return FALSE (cho qua)
  └─ ime_key(inst, &k, &r)                                  // r trên stack
  └─ if r.flags & IME_FLAG_ERROR  → treat như PASS
  └─ match r.action:
       PASS      → return FALSE
       REPLACE   → edit session:
                     if strategy==Preedit:  ensure composition(r.preedit)
                     else:                  apply_replace(delete_count, r.insert)  (P1-1 §6)
       COMMIT    → end composition, chèn insert  (B2)
       RESTORE   → apply_replace(delete_count, r.insert)
  └─ return TRUE (đã nuốt phím)
  // Mọi early-return PHẢI để engine state không bị treo: gọi ime_reset() nếu abandon giữa chừng
```

Walkthrough của hook/IMK/IBus: xem `P1-2` (Win hook), `P2-*` (macOS), `P3-*` (Linux) — cùng format.

## 8. Ngoài phạm vi v1 (không tự thêm)

- Callback/async suggest (v1: adapter gọi `ime_suggest` đồng bộ, hoặc bỏ qua).
- Nhận text phía app (context BEFORE cursor) — adapter không gửi; engine chỉ tự quản buffer của nó.
- Shared memory / out-of-process engine (TSF không hỗ trợ in-proc bắt buộc; hook chế độ out-proc bằng process riêng — xem P1-2).
