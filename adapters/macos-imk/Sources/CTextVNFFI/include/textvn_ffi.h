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

/* ---- assist constants (P4-0 §4 / §5) ---- */
#define IME_ASSIST_KIND_DUPLICATE_WORD     1u
#define IME_ASSIST_KIND_SPACE_BEFORE_PUNCT 2u
#define IME_ASSIST_KIND_SPACE_AFTER_PUNCT  3u
#define IME_ASSIST_KIND_DOUBLE_SPACE       4u
#define IME_ASSIST_KIND_LOWERCASE_SENTENCE 5u
#define IME_ASSIST_KIND_DIACRITIC_TYPO     6u
#define IME_ASSIST_KIND_SENTENCE_REPEAT    7u
#define IME_ASSIST_KIND_DOUBLE_NEGATIVE   8u
#define IME_ASSIST_KIND_COMMA_AT_END      9u
#define IME_ASSIST_KIND_SPELLING         10u
#define IME_ASSIST_KIND_ARTICLE_AN      11u

#define IME_MAX_ASSIST_ISSUES              8
#define IME_MAX_ASSIST_MSG                 128
#define IME_MAX_ASSIST_SUGGEST             64
#define IME_MAX_SENTENCE_FIX_CHARS         1024

#define IME_ASSIST_TRIGGER_NONE            0u
#define IME_ASSIST_TRIGGER_WORD            1u
#define IME_ASSIST_TRIGGER_SENTENCE        2u

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

typedef struct ime_assist_issue {
  uint32_t kind;                                    /* IME_ASSIST_KIND_* */
  uint32_t start;                                   /* offset ký tự Unicode bắt đầu (inclusive) */
  uint32_t end;                                     /* offset ký tự Unicode kết thúc (exclusive) */
  uint32_t auto_fixable;                            /* 1 = có thể tự sửa, 0 = chỉ gợi ý */
  uint32_t confidence;                              /* 0..100 (D28) */
  char     message[IME_MAX_ASSIST_MSG];             /* UTF-8, 0-terminated */
  uint32_t suggest_len;                             /* độ dài UTF-8 của gợi ý đầu */
  char     suggest[IME_MAX_ASSIST_SUGGEST];         /* UTF-8, 0-terminated */
} ime_assist_issue_v1;

/*
 * ime_assist_result_v1:
 * Caller-owned, value-only result buffer for assist inspections.
 * Ownership: Allocated and managed entirely by caller (stack or heap).
 * Lifetime: Engine never retains pointers to this struct or its contents.
 * Capacity: Capped at IME_MAX_ASSIST_ISSUES (8) issues. Extra issues beyond
 *           capacity are dropped safely.
 */
typedef struct ime_assist_result {
  uint32_t            abi_version;                  /* = IME_ABI_VERSION */
  uint32_t            count;                        /* 0..IME_MAX_ASSIST_ISSUES */
  ime_assist_issue_v1 issues[IME_MAX_ASSIST_ISSUES];
} ime_assist_result_v1;

typedef struct ime_assist_plan {
  uint32_t abi_version;
  uint32_t fixed_count;
  uint32_t delete_from;    /* vị trí char bắt đầu xóa */
  uint32_t delete_chars;   /* số ký tự char cần xóa */
  uint32_t delete_glyphs;  /* số glyph cần xóa lùi (surrounding text) */
  char     replacement[IME_MAX_SENTENCE_FIX_CHARS];   /* chuỗi UTF-8 thay thế */
  char     original_tail[IME_MAX_SENTENCE_FIX_CHARS]; /* chuỗi UTF-8 đuôi câu cũ */
} ime_assist_plan_v1;

typedef struct ime_os_spell {
  uint32_t abi_version;                                     /* = IME_ABI_VERSION */
  int32_t  (*contains)(void *ctx, const char *word, uint32_t byte_len);  /* 1 = dung, 0 = sai, -1 = khong ro / fail-open */
  uint32_t (*suggest)(void *ctx, const char *word, uint32_t byte_len,
                      char *out, uint32_t out_len, uint32_t limit);       /* tra so goi y NUL-separated ghi vao out */
  void     *ctx;
} ime_os_spell_v1;

/* ---- API ---- */
uint32_t     ime_abi_version(void);                     /* luôn IME_ABI_VERSION của lib */
/* config sai schema: VẪN tạo instance với config mặc định, trả IME_ERR_CONFIG (non-fatal)
 * và ghi chi tiết (không chứa text người dùng) vào ime_last_error(). *out luôn được set. */
int32_t      ime_instance_new(const uint8_t *config_utf8, size_t len, ime_instance **out);
void         ime_instance_free(ime_instance *inst);
int32_t      ime_set_context(ime_instance *inst, const ime_context_v1 *ctx);
int32_t      ime_key(ime_instance *inst, const ime_key_v1 *k, ime_result_v1 *out);
int32_t      ime_reset(ime_instance *inst);             /* xóa buffer (word boundary/focus change) */
/* R2-131: ime_reset + ghi nhận utf8 (câu trợ lý vừa sửa, đã nằm trước con
 * trỏ) làm đuôi buffer câu — dự đoán tiếp theo ngữ cảnh. NULL ⇒ chỉ reset. */
int32_t      ime_reset_with_tail(ime_instance *inst, const char *utf8, size_t len);
/* P6-11 · R2-134: ghost text — phần chữ Tab SẼ nối thêm sau con trỏ (UTF-8 + NUL,
 * tối đa cap byte). Trả số byte (0 = không có gợi ý); không đổi trạng thái engine. */
int32_t      ime_peek_completion(const ime_instance *inst, char *out, size_t cap);
/* OP-007b · R2-139: chuyển đổi vùng bôi đen (chữ lạ do đọc nhầm UTF-8, bảng mã cũ →
 * bảng mã đang dùng, chữ ASCII gõ nhầm chế độ E → gõ lại theo kiểu gõ). UTF-8 vào/ra
 * (+ NUL, tối đa cap byte). Trả số byte; 0 = không đổi gì. */
int32_t      ime_convert_selection(const ime_instance *inst, const char *text_utf8, size_t len,
                                   char *out, size_t cap);
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
/* ---- Assist API (P4-2 · D23) ---- */
uint32_t     ime_assist_trigger_kind(uint32_t ch);      /* NONE(0), WORD(1), SENTENCE(2) */
int32_t      ime_assist_check(const char *text_utf8, size_t len,
                              ime_assist_result_v1 *out);
int32_t      ime_assist_check_instance(ime_instance *inst,
                                       ime_assist_result_v1 *out);
int32_t      ime_assist_sentence_tail(const ime_instance *inst, size_t max_chars,
                                      char *out_buf, size_t buf_len,
                                      size_t *out_len);
int32_t      ime_assist_glyph_len(const ime_instance *inst,
                                  const char *text_utf8, size_t len,
                                  size_t *out_glyphs);
int32_t      ime_assist_set_spell_provider(ime_instance *inst,
                                           const ime_os_spell_v1 *provider);
/* Kiểm tra câu ở dạng UTF-16 (D29 — biên chuẩn TSF/AX/NSString): trả về SỐ ISSUE
 * (≥0) đã ghi vào *out; mã IME_ERR_* (âm) chỉ cho lỗi caller (NULL/ABI — P0-2 §0).
 * Lỗi dữ liệu (UTF-16 lệch, panic) → fail-open: 0 issue, không panic. Không log câu (S2). */
int32_t      ime_assist_check_v1(const uint16_t *text_utf16, int32_t len,
                                 ime_assist_result_v1 *out);
/* D39 (P4-10 §5.2): như ime_assist_check_v1 + provider từ điển OS tùy chọn
 * (ime_os_spell_v1, xem ime_assist_set_spell_provider). provider == NULL
 * làm việc đúng như ime_assist_check_v1; ABI lệch = IME_ERR_ABI. */
int32_t      ime_assist_check_v1_with(const uint16_t *text_utf16, int32_t len,
                                 const ime_os_spell_v1 *provider,
                                 ime_assist_result_v1 *out);
/* Lấy phần đuôi câu của text_utf16, tối đa `max_chars` ký tự Unicode (D27/D29),
 * ghi dạng UTF-16 kèm NUL kết thúc vào out_utf16 (cap = out_cap code unit).
 * Trả về số code unit đã ghi (không tính NUL); lỗi caller = IME_ERR_*; lỗi dữ
 * liệu/panic → 0 (fail-open). Cắt luôn tại ranh giới char, không chẻ surrogate. */
int32_t      ime_assist_sentence_tail_v1(const uint16_t *text_utf16, int32_t len,
                                         int32_t max_chars,
                                         uint16_t *out_utf16, int32_t out_cap);
int32_t      ime_assist_plan_sentence_fix(const ime_instance *inst,
                                          const char *sentence_utf8,
                                          const ime_assist_result_v1 *result,
                                          ime_assist_plan_v1 *out_plan);
/* R2-131: như ime_assist_plan_sentence_fix kèm flags. IME_ASSIST_PLAN_EXPLICIT
 * = hành động người dùng chủ động (nút "Sửa tất cả", phím tắt, chọn một gợi ý):
 * gồm cả lỗi chính tả có gợi ý và dấu phẩy cuối câu (Mode 2 tự sửa: flags = 0). */
#define IME_ASSIST_PLAN_EXPLICIT 1u
int32_t      ime_assist_plan_sentence_fix_ex(const ime_instance *inst,
                                             const char *sentence_utf8,
                                             const ime_assist_result_v1 *result,
                                             uint32_t flags,
                                             ime_assist_plan_v1 *out_plan);
#ifdef __cplusplus
}
#endif
#endif /* TEXTVN_FFI_H */

