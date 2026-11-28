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
/* EN vòng 36: cặp từ dễ nhầm + đại từ "i" thường (en_confusions.rs). */
#define IME_ASSIST_KIND_WORD_CONFUSION  12u
#define IME_ASSIST_KIND_LOWERCASE_PRONOUN 13u
/* Vòng 45 (đề xuất persona): định dạng vô lý — suggestion-only. */
#define IME_ASSIST_KIND_INVALID_DATE    14u
#define IME_ASSIST_KIND_INVALID_MONEY   15u

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
  uint16_t cursor_left;          /* số bước dịch con trỏ sang trái sau chèn (Math)    */
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
/* P7 (vòng 54) — User Style Profile (OPT-IN, G8: chỉ từ + cặp sai→đúng +
 * count; KHÔNG nội dung câu/phím). Học: ghi cặp user CHẤP NHẬN vào pending
 * (validate 2..=32 ký tự alphabetic/space; sai = IME_ERR_INVALID_ARG).
 * Drain: trả JSON {"words":[{word,count}..],"corrections":[{from,to,count}..]}
 * hoặc "{}" rồi XÓA phần vừa xuất (phần vượt trần ở lại lượt sau) — adapter gọi định kỳ. */
int32_t      ime_style_note_correction(ime_instance *inst,
                                       const uint8_t *from_utf8, size_t from_len,
                                       const uint8_t *to_utf8, size_t to_len);
int32_t      ime_style_profile_drain(ime_instance *inst,
                                     uint8_t *out_buf, size_t buf_len,
                                     size_t *out_len);
/* R2-152 — lưu pending vào file config.json (macOS/Linux, không có tray): ghi nguyên tử,
 * 0600; pending chỉ xoá sau khi ghi xong. Không có gì để lưu cũng trả IME_OK. */
int32_t      ime_style_profile_persist(ime_instance *inst,
                                       const uint8_t *path_utf8, size_t path_len);
/* P12 — Emoji Picker nhẹ (danh mục ~280 emoji OS/Teams/iOS, tìm kiếm VI/EN không dấu,
 * MRU 32 phần tử + Dream RSI). Ghi các dòng UTF-8 `glyph\tshortcode\tcategory\n` NUL-terminated.
 * `inst` có thể NULL (không dùng MRU); `query_utf8` có thể NULL khi `query_len == 0`. */
int32_t      ime_emoji_search(const ime_instance *inst,
                              const uint8_t *query_utf8, size_t query_len,
                              uint8_t category, uint32_t limit,
                              uint8_t *out_buf, size_t buf_len,
                              size_t *out_len);
int32_t      ime_emoji_record_use(ime_instance *inst,
                                  const uint8_t *glyph_utf8, size_t glyph_len);
/* ---- SF-1b (P0-6 §4) — session nghiệp vụ dùng chung 3 OS (`textvn-session`) ----
 * Không callback, không cấp phát chéo: adapter cấp ime_session_out_v1 (stack), session ghi vào.
 * Tràn IME_SESSION_MAX_CMDS/ARENA ⇒ IME_ERR_BUFFER, out->count = 0, session KHÔNG đổi trạng thái
 * (adapter coi như ForwardKey). Chữ (preedit/commit/ghost/JSON IPC/mục trợ lý "text\treplace"
 * cách nhau '\n') nằm ở arena[text_off .. text_off+text_len], UTF-8, không NUL. */
#define IME_SESSION_ABI_VERSION 1u
#define IME_SESSION_MAX_CMDS 16
#define IME_SESSION_ARENA 2048
#define IME_ERR_BUFFER       -5      /* SF-1b: ime_session_out_v1 không đủ chỗ; session không đổi */

/* Caps (bit) */
#define IME_SCAP_PREEDIT            0x01u
#define IME_SCAP_SURROUNDING_TEXT   0x02u
#define IME_SCAP_DELETE_SURROUNDING 0x04u
#define IME_SCAP_INLINE_GHOST       0x08u
#define IME_SCAP_KEY_UP_EVENTS      0x10u
#define IME_SCAP_FORWARD_KEY        0x20u

/* Sự kiện (ime_session_event_v1.kind) */
#define IME_SEV_KEYDOWN        1u  /* vk, ch, mods, now_ms */
#define IME_SEV_KEYUP          2u  /* vk, mods, now_ms */
#define IME_SEV_FOCUS_IN       3u  /* text = app_id, arg = field_role, mods = IME_SFOCUS_* */
#define IME_SEV_FOCUS_OUT      4u
#define IME_SEV_CARET_MOVED    5u
#define IME_SEV_SURROUNDING    6u  /* text = chữ trước caret */
#define IME_SEV_RESET          7u  /* arg = IME_SRESET_*, now_ms */
#define IME_SEV_TIMER          8u  /* arg = timer id, now_ms */
#define IME_SEV_IPC            9u  /* text = JSON một message IPC v1 (thân frame, không prefix) */
#define IME_SEV_ASSIST_ACTION 10u  /* arg = IME_SASSIST_*, ch = chỉ số gợi ý (Select) */
#define IME_SFOCUS_SECURE   0x1u
#define IME_SFOCUS_TERMINAL 0x2u
#define IME_SRESET_TOOLKIT 0u
#define IME_SRESET_FOCUS   1u
#define IME_SRESET_APP     2u
#define IME_SASSIST_SELECT  0u
#define IME_SASSIST_DISMISS 1u
#define IME_SASSIST_UNDO    2u

/* Lệnh (ime_session_cmd_v1.kind) */
#define IME_SCMD_SET_PREEDIT        1u  /* arg0 = cursor; text rỗng = ẩn preedit */
#define IME_SCMD_COMMIT             2u
#define IME_SCMD_DELETE_BEFORE      3u  /* arg0 = số ký tự */
#define IME_SCMD_FORWARD_KEY        4u
#define IME_SCMD_EAT_KEY            5u
#define IME_SCMD_GHOST              6u  /* text rỗng = ẩn */
#define IME_SCMD_GHOST_AFTER_KEYUP  7u
#define IME_SCMD_ASSIST_SHOW        8u  /* arg0 = số mục */
#define IME_SCMD_ASSIST_HIDE        9u
#define IME_SCMD_TIMER_START       10u  /* arg0 = id, arg1 = ms */
#define IME_SCMD_TIMER_CANCEL      11u  /* arg0 = id */
#define IME_SCMD_IPC               12u  /* text = JSON message IPC v1 */
#define IME_SCMD_MODE_INDICATOR    13u  /* arg0 = 1 tiếng Việt / 0 tiếng Anh */
#define IME_SCMD_ENGINE_RESET      14u  /* SF-3 R2-171/E78: arg0 = 1 giữ mốc đầu dòng (reset_with_tail "\n") / 0 xoá sạch */

typedef struct ime_session ime_session;

typedef struct ime_session_config {
    uint32_t abi_version;     /* IME_SESSION_ABI_VERSION */
    uint32_t caps;            /* bit IME_SCAP_* */
    const uint8_t *config_json; /* UTF-8, có thể NULL (config_len = 0) */
    size_t config_len;
} ime_session_config_v1;

typedef struct ime_session_event {
    uint32_t abi_version;
    uint32_t kind;            /* IME_SEV_* */
    uint32_t vk;
    uint32_t ch;
    uint32_t mods;
    uint64_t now_ms;
    const uint8_t *text;      /* UTF-8, có thể NULL (text_len = 0) */
    size_t text_len;
    uint32_t arg;
} ime_session_event_v1;

typedef struct ime_session_cmd {
    uint32_t kind;            /* IME_SCMD_* */
    uint32_t arg0;
    uint32_t arg1;
    uint32_t text_off;        /* vào arena */
    uint32_t text_len;
} ime_session_cmd_v1;

typedef struct ime_session_out {
    uint32_t abi_version;     /* adapter đặt = IME_SESSION_ABI_VERSION trước khi gọi */
    uint32_t count;
    ime_session_cmd_v1 cmds[IME_SESSION_MAX_CMDS];
    uint8_t arena[IME_SESSION_ARENA];
} ime_session_out_v1;

int32_t      ime_session_new(const ime_session_config_v1 *cfg, ime_session **out);
void         ime_session_free(ime_session *s);
int32_t      ime_session_event(ime_session *s, const ime_session_event_v1 *ev,
                               ime_session_out_v1 *out);
int32_t      ime_session_reload_config(ime_session *s, const uint8_t *json, size_t len);
/* ---- SF-3 (P0-6 §5 compose) — planner preedit/commit dùng chung (`textvn_session::compose`) ----
 * Hàm thuần, không trạng thái: adapter cấp ime_compose_plan_v1 (stack, abi_version đặt trước).
 * `cur` = text đang soạn (UTF-32, có thể NULL khi cur_len = 0); `r` = kết quả ime_key của phím.
 * Text kế hoạch > IME_COMPOSE_MAX ⇒ IME_ERR_BUFFER (POLICY_LINUX luôn ≤ 256). */
#define IME_COMPOSE_ABI_VERSION 1u
#define IME_COMPOSE_MAX 256
#define IME_COMPOSE_POLICY_TSF   0u  /* ranh giới in được commit cùng từ, ăn phím (CUAS) */
#define IME_COMPOSE_POLICY_LINUX 1u  /* commit từ rồi forward phím; tối đa 256; Ctrl+Space reset */
#define IME_CKEY_CHAR       0u
#define IME_CKEY_BACKSPACE  1u
#define IME_CKEY_ESCAPE     2u
#define IME_CKEY_ENTER      3u
#define IME_CKEY_TAB        4u
#define IME_CKEY_CTRL_SPACE 5u
#define IME_CKEY_OTHER      6u
#define IME_CKEY_ALTGR      7u

typedef struct ime_compose_plan {
    uint32_t abi_version;     /* IME_COMPOSE_ABI_VERSION */
    uint32_t delete_before;   /* ký tự đã commit trước preedit phải xoá */
    uint32_t cursor_left;     /* P11 math: số bước mũi tên trái sau commit */
    uint32_t has_text;        /* 1 = cập nhật preedit/commit thành text */
    uint32_t end;             /* 1 = commit rồi đóng preedit */
    uint32_t eaten;           /* 1 = nuốt phím; 0 = app nhận phím gốc */
    uint32_t reset_engine;    /* 1 = commit + ime_reset */
    uint32_t text_len;
    uint32_t text[IME_COMPOSE_MAX];
} ime_compose_plan_v1;

int32_t      ime_compose_plan(uint32_t policy, const uint32_t *cur, size_t cur_len,
                              uint32_t key_kind, uint32_t key_ch, const ime_result_v1 *r,
                              ime_compose_plan_v1 *out);
/* ---- SF-3 pha 3b — kế hoạch áp outcome engine cho macOS IMK (`textvn_session::imk`) ----
 * Hàm thuần: quyết định chuỗi thao tác (thu hồi marked, xoá, đặt marked, chèn, reset); adapter
 * Swift thực thi trên IMKTextInput. Chữ của thao tác ở arena[text_off..+text_len] (UTF-8).
 * Tràn IME_IMK_MAX_OPS/ARENA ⇒ IME_ERR_BUFFER, count = 0 (adapter fail-open). */
#define IME_IMK_ABI_VERSION 1u
#define IME_IMK_MAX_OPS 8
#define IME_IMK_ARENA 1024
#define IME_IMK_PASS    0u
#define IME_IMK_REPLACE 1u
#define IME_IMK_COMMIT  2u
#define IME_IMK_RESTORE 3u
#define IME_IMK_STRATEGY_PREEDIT           0u
#define IME_IMK_STRATEGY_BACKSPACE_TYPE    1u
#define IME_IMK_STRATEGY_SELECTION_REPLACE 2u
#define IME_IMK_STRATEGY_FORWARD_AS_COMMIT 3u
#define IME_IMK_STRATEGY_PASSTHROUGH       4u
#define IME_IMK_MARKED_RANGE       0x1u  /* app còn marked range */
#define IME_IMK_SELECTION          0x2u  /* app giữ selection không rỗng */
#define IME_IMK_PREEDIT_OVER_LIMIT 0x4u  /* preedit mới vượt B11 (8 grapheme, adapter đếm) */
#define IME_IMK_OP_RETRACT_MARKED      1u
#define IME_IMK_OP_DELETE_BACKWARD     2u  /* arg = số ký tự */
#define IME_IMK_OP_SET_MARKED          3u
#define IME_IMK_OP_INSERT              4u  /* arg = IME_IMK_AT_* */
#define IME_IMK_OP_CLEAR_MARKED_STATE  5u
#define IME_IMK_OP_RESET_ENGINE        6u
#define IME_IMK_AT_CARET        0u
#define IME_IMK_AT_MARKED_RANGE 1u
#define IME_IMK_AT_SELECTION    2u

typedef struct ime_imk_input {
    uint32_t abi_version;     /* IME_IMK_ABI_VERSION */
    uint32_t action;          /* IME_IMK_PASS … RESTORE */
    uint32_t strategy;        /* IME_IMK_STRATEGY_* */
    uint32_t state;           /* bit IME_IMK_MARKED_RANGE | SELECTION | PREEDIT_OVER_LIMIT */
    uint32_t delete_count;    /* code point */
    const uint8_t *insert;    /* UTF-8, có thể NULL khi len = 0 */
    size_t insert_len;
    const uint8_t *preedit;
    size_t preedit_len;
    const uint8_t *marked;    /* marked adapter đang theo dõi */
    size_t marked_len;
} ime_imk_input_v1;

typedef struct ime_imk_op {
    uint32_t kind;            /* IME_IMK_OP_* */
    uint32_t arg;
    uint32_t text_off;
    uint32_t text_len;
} ime_imk_op_v1;

typedef struct ime_imk_plan {
    uint32_t abi_version;     /* adapter đặt = IME_IMK_ABI_VERSION trước khi gọi */
    uint32_t count;
    ime_imk_op_v1 ops[IME_IMK_MAX_OPS];
    uint8_t arena[IME_IMK_ARENA];
} ime_imk_plan_v1;

int32_t      ime_imk_plan(const ime_imk_input_v1 *in, ime_imk_plan_v1 *out);
#ifdef __cplusplus
}
#endif
#endif /* TEXTVN_FFI_H */

