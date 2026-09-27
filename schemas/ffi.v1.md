# `ffi.v1` — TextVN engine C-ABI v1 (bản chú giải cho adapter)

> **Đây là bản chép + chú giải của `ffi/include/textvn_ffi.h`** (deliverable `P0-1 §1`).
> Nguồn sự thật vẫn là header C + `docs/10-shared/P0-2-engine-ffi-contract.md`:
> - Sửa **header** → sửa file này theo (hoặc `cargo xtask cbindgen` regenerate cả hai, `P0-1 §3`).
> - **Không** thêm hằng/struct/hàm chỉ ở file này — đó là cách chắc chắn nhất để lệch ABI.
> - CI gate: `cargo run -p textvn-cli -- sizes` (20/532) + `cargo run -p textvn-cli -- verify`
>   (hằng + trường struct + **thứ tự 11 hàm export**) + `ffi/tests/abi_invariants.rs` + fuzz `ffi_key`.

## 1. Bất biến (P0-2 §0) — đọc trước khi viết adapter

| # | Bất biến | Hệ quả cho adapter |
|---|---|---|
| 1 | **Không alloc chéo ranh giới** | Mọi `*_v1` do **caller** cấp phát; engine chỉ ghi vào. Không `free()`/`delete` trong lib. |
| 2 | **Không callback** Rust → adapter | Engine không gọi lại code của bạn → không deadlock, không cần reentrancy guard. |
| 3 | **Fail-open** | Lỗi/panic → `IME_OK` + `action=PASS` + `IME_FLAG_ERROR`. Phím phải đi tới app. |
| 4 | **Không I/O / global state** | Mọi thứ nằm trong `ime_instance`; 1 instance = 1 thread. |
| 5 | **`ime_last_error` không chứa text người dùng** (S2) | Chỉ log *kind* lỗi; không kỳ vọng thấy nội dung. |
| 6 | **`secure=1` ⇒ luôn PASS** (S3) | Không gửi chuỗi vào ô mật khẩu; engine tự chặn, adapter không cần lọc. |

## 2. Header (nguyên văn — sinh từ `ffi/include/textvn_ffi.h`)

<!-- BEGIN textvn_ffi.h (copy — sửa header rồi copy lại, đừng sửa trong khối này) -->

```c
/* textvn_ffi.h — TextVN engine C ABI v1
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: docs/10-shared/P0-2-engine-ffi-contract.md §1.
 * Sau này do `cargo xtask cbindgen` regenerate (P0-1 §3) — hiện file này là
 * bản chép tay đã review; CI test `abi_size` đối chiếu (P0-2 §6).
 */
#ifndef TEXTVN_FFI_H
#define TEXTVN_FFI_H
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
#define IME_FIELD_ADDRESS_BAR 3u  /* thanh địa chỉ / combo có autocomplete           */
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
#define IME_CAP_FIELD_DETECT 0x4u /* adapter detect được field_role (UIA/AX/AT-SPI)    */
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
/* Chỉ trả IME_OK khi Ed25519 hợp lệ với data/preset.pub của release. Bản chưa
 * provision public key sẽ từ chối (IME_ERR_INTERNAL), không được fail-open. */
int32_t      ime_appdb_verify(const uint8_t *json, size_t json_len,
                              const uint8_t *sig, size_t sig_len);
/* Resolver dùng chung cho adapter không phải Rust. appdb_utf8=NULL,len=0 nghĩa
 * là không preset; mọi lỗi vẫn đặt *out_strategy=IME_STRATEGY_PASSTHROUGH. */
int32_t      ime_strategy_resolve(const ime_context_v1 *ctx,
                                  const uint8_t *appdb_utf8, size_t len,
                                  int64_t *out_strategy);
const char  *ime_last_error(const ime_instance *inst);  /* UTF-8, hợp lệ đến lần gọi kế tiếp
                                                           trên CÙNG instance; không chứa text
                                                           người dùng */
#ifdef __cplusplus
}
#endif
#endif /* TEXTVN_FFI_H */

```
<!-- END textvn_ffi.h -->

## 3. Chú giải: vòng đời chuẩn của adapter

```c
ime_instance *inst = NULL;
/* config_utf8 = NULL, len = 0 → dùng config mặc định.
 * Trả IME_ERR_CONFIG: instance VẪN dùng được (fail-open) — chỉ log kind lỗi. */
int32_t rc = ime_instance_new(cfg, cfg_len, &inst);

ime_context_v1 ctx = {
    .abi_version = IME_ABI_VERSION,     // sai → IME_ERR_ABI
    .enabled = 1, .secure = 0,
    .field_role = IME_FIELD_BODY,
    .caps = IME_CAP_PREEDIT | IME_CAP_INJECT_VK,
    .app_id = "notepad.exe",            // hoặc NULL
    .element_name = NULL,
    .hint = -1,                         // -1 = để engine tự resolve (P0-3 §3.1)
};
ime_set_context(inst, &ctx);            // gọi lại mỗi khi focus đổi (thread khác → instance riêng)

ime_key_v1 k = { IME_ABI_VERSION, vk, ch, mods, /*key_down=*/1, 0, 0, 0 };
ime_result_v1 r;                        // caller cấp phát
int32_t rc2 = ime_key(inst, &k, &r);
// rc2 == IME_OK → áp `r` theo action; rc khác → coi như PASS (bất biến 3).
```

**Bảng áp dụng sau mỗi `ime_key`** (dùng cho TSF `P1-1` và hook `P1-2`):

| `action` | Việc adapter phải làm | Forward phím cho app? |
|---|---|---|
| `PASS` | Không động vào document. | **Có** (trừ khi `IME_FLAG_CONSUMED` được bật) |
| `REPLACE` | Xoá `delete_count` ký tự **trước** con trỏ → chèn `insert[0..insert_len]` → cập nhật vùng preedit nếu `preedit_len > 0`. | Không |
| `COMMIT` | Chèn `insert[]`, đóng composition/preedit, cập nhật con trỏ. | Không |
| `RESTORE` | Như `REPLACE` nhưng mang nghĩa "gỡ biến đổi cũ" (auto-restore tiếng Anh — bug B5). | Không |

> `insert[]`/`preedit[]` là **UTF-32 code point**, không phải byte. Adapter đổi sang
> UTF-16/UTF-8 theo API OS (`TsfInsertText` nhận UTF-16, `CGEventKeyboardSetUnicodeString`
> nhận UTF-16, `ibus_input_context_commit_text` nhận UTF-8). Nhầm endianness là bug
> kinh điển → luôn kiểm bằng 1 ký tự ASCII trước khi tin phần còn lại.

## 4. Lỗi thường gặp (rút từ bug B1–B13)

| Triệu chứng | Nguyên nhân gần như luôn đúng | Cách tránh |
|---|---|---|
| Đọc tràn / crash trong adapter | Đọc `insert[]` vượt `insert_len` | Chỉ đọc `[0..insert_len]`; fuzz `ffi_key` assert `insert_len ≤ IME_MAX_TEXT` |
| Chữ bị nhân đôi | Forward phím **và** còn áp `REPLACE` | Chỉ forward khi `action == PASS` và không có `IME_FLAG_CONSUMED` |
| Vòng lặp bơm phím (bug B6) | Không đánh dấu phím mình bơm | `is_injected = 1` khi bơm; engine **luôn** PASS mọi phím có cờ này |
| Mất phím khi engine lỗi | Coi `rc != IME_OK` là "nuốt phím" | `rc != IME_OK` ⇒ coi như `PASS` (bất biến 3) |
| Gõ trong ô mật khẩu | Gửi chuỗi khi `secure=1` | Engine tự PASS khi `secure=1`; adapter không log/ghi text ở context này (S2/S3) |
| Mất tiếng Việt sau Alt-Tab | Không gọi `ime_reset` | `ime_reset` khi focus rời app (bug B2) |
| Rối loạn khi nhiều thread | Chia sẻ 1 `ime_instance` cho nhiều thread | 1 instance = 1 thread (P0-2 §3) |
| Engine chết sau 1 panic | Adapter dựa vào việc `ime_key` trả lỗi | Panic đã bị `catch_unwind`; xem thêm `IME_FLAG_ERROR` |

## 5. Khi nào phải bump `IME_ABI_VERSION`

Sửa **bất kỳ** trường nào của `ime_key_v1` / `ime_result_v1`, đổi giá trị hằng, thêm
trường mới vào struct, **đổi tên/thêm/bớt/đảo thứ tự hàm export** → bump `IME_ABI_VERSION` (P0-2 §6).
Adapter kiểm tra qua
`ime_abi_version()` và `abi_version` trong mọi struct; lệch là `IME_ERR_ABI` + PASS.
Gate phát hiện: `cargo run -p textvn-cli -- sizes` (exit 1 khi size lệch 20/532)
+ `cargo run -p textvn-cli -- verify` (exit 1 khi hằng/trường/thứ tự hàm lệch).

