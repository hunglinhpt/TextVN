/* lc_compose.h — Mô hình preedit dùng chung cho IBus / Fcitx5 + nạp config.json
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Port 1:1 của `adapters/windows-tsf/src/compose.rs` (nguồn sự thật, kiểm chứng bằng
 * `textvn-cli replay --adapter tsf` trên toàn bộ corpus):
 *
 *   Cả từ đang gõ nằm trong PREEDIT do engine sở hữu; mọi `delete_count` của engine
 *   rơi trong preedit; tới ranh giới từ (Space, dấu câu, Enter, phím điều hướng,
 *   chord…) preedit được COMMIT rồi phím ranh giới mới đi tới app.
 *
 * Vì sao không "gõ không gạch chân" (delete_surrounding + commit) làm mặc định:
 * surrounding text không đáng tin ở Chromium/Electron/Qt/XWayland (RL5, B7) — khi
 * app báo sai vị trí caret, xóa lùi sẽ xóa nhầm chữ của người dùng. Preedit là
 * giao thức mọi client IBus/Fcitx5 đều hỗ trợ.
 *
 * Bất biến (test ở tests/test_compose.c):
 * - plan.eaten == 0 ⇒ plan.end == 1 hoặc không composing: app không bao giờ nhận
 *   phím khi preedit còn mở (Enter gửi tin nhắn không để lại chữ treo — B2).
 * - RESTORE/COMMIT ở ranh giới: bỏ ký tự ranh giới khỏi text và để app nhận phím
 *   THẬT (Enter là Enter, không phải '\n' chèn vào).
 */

#ifndef TEXTVN_LC_COMPOSE_H
#define TEXTVN_LC_COMPOSE_H

#include <stddef.h>
#include <stdint.h>

#include "textvn_ffi.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Win VK canonical (P0-2 §1) — adapter Linux map keysym về đây. */
#define LC_VK_BACK   0x08u
#define LC_VK_TAB    0x09u
#define LC_VK_RETURN 0x0Du
#define LC_VK_ESCAPE 0x1Bu
#define LC_VK_SPACE  0x20u

typedef enum lc_key_kind {
    LC_KEY_CHAR = 0,  /* sinh ký tự in được (gồm Space) */
    LC_KEY_BACKSPACE,
    LC_KEY_ESCAPE,
    LC_KEY_ENTER,
    LC_KEY_TAB,
    LC_KEY_OTHER      /* mũi tên, Delete, Home/End, F-key… */
} lc_key_kind;

typedef struct lc_key {
    lc_key_kind kind;
    uint32_t    ch;   /* chỉ có nghĩa khi kind == LC_KEY_CHAR */
} lc_key;

/* `ch` = ký tự Unicode của keysym (0 nếu không sinh ký tự). Ký tự điều khiển được
 * quy về phím tương ứng; phím không sinh ký tự không bao giờ bị coi là chữ. */
lc_key   lc_key_classify(uint32_t vk, uint32_t ch);
/* `ime_key_v1.ch` gửi engine: chỉ ký tự in được (Enter/Tab để engine nhận qua vk). */
uint32_t lc_key_engine_ch(lc_key key);
/* Dựng `ime_key_v1` (key-down) cho phím đã phân loại. */
void     lc_key_to_ime(lc_key key, uint32_t vk, uint32_t mods, ime_key_v1 *out);

#define LC_COMP_MAX 256

/* Text preedit hiện tại (UTF-32). len == 0 ⇒ không composing. */
typedef struct lc_comp {
    uint32_t text[LC_COMP_MAX];
    size_t   len;
} lc_comp;

typedef struct lc_plan {
    /* Ký tự đã commit NGAY TRƯỚC preedit phải xóa (macro/emoji vượt ranh giới).
     * Adapter chỉ xóa khi client hỗ trợ surrounding text; không thì fail-open. */
    uint16_t delete_before;
    int      has_text;              /* 1 = cập nhật preedit thành `text` */
    uint32_t text[LC_COMP_MAX];
    size_t   text_len;
    int      end;                   /* commit `text` (hoặc preedit hiện tại) rồi đóng */
    int      eaten;                 /* 1 = nuốt phím; 0 = app nhận phím gốc */
    int      reset_engine;          /* preedit tràn LC_COMP_MAX: commit + ime_reset */
} lc_plan;

/* Tính kế hoạch cho một phím từ kết quả engine. Hàm thuần. */
void lc_plan_key(const lc_comp *cur, lc_key key, const ime_result_v1 *r, lc_plan *out);
/* Kế hoạch "commit nguyên văn, phím đi thẳng" (chord, field bảo mật, tắt VN…). */
void lc_plan_pass_through(const lc_comp *cur, lc_plan *out);
/* Cập nhật model sau khi adapter áp kế hoạch thành công. */
void lc_comp_apply(lc_comp *cur, const lc_plan *plan);
/* Text sẽ được commit khi `plan.end` (text mới nếu có, ngược lại preedit hiện tại). */
const uint32_t *lc_plan_commit_text(const lc_comp *cur, const lc_plan *plan, size_t *out_len);

/* ---- Phím chuyển Ctrl+Shift kiểu UniKey (nhấn cả hai rồi nhả, không kèm phím khác) ---- */
typedef struct lc_modifier_toggle {
    int armed;
} lc_modifier_toggle;

typedef enum lc_modifier {
    LC_MOD_NONE = 0,
    LC_MOD_KEY_CTRL,
    LC_MOD_KEY_SHIFT,
    LC_MOD_KEY_OTHER_MODIFIER   /* Alt/Super/CapsLock… tự đứng một mình */
} lc_modifier;

/* Key-down. `which` = phím hiện tại là Ctrl/Shift/modifier khác/phím thường;
 * ctrl_down/shift_down/alt_or_super = trạng thái các phím còn lại. */
void lc_modifier_toggle_down(lc_modifier_toggle *t, lc_modifier which,
                             int ctrl_down, int shift_down, int alt_or_super);
/* Key-up: trả 1 đúng MỘT lần khi nhả Ctrl/Shift của một lần bấm hợp lệ. */
int  lc_modifier_toggle_up(lc_modifier_toggle *t, lc_modifier which);
void lc_modifier_toggle_reset(lc_modifier_toggle *t);

/* ---- config.json / state.json (do textvn-settings ghi — cùng định dạng tray Windows) ---- */
typedef struct lc_config_state {
    long long mtime_ns;
    long long size;
    int       loaded;
    /* config.allow_macro_when_vi_off của lần nạp hợp lệ gần nhất (adapter cần biết để
     * vẫn đưa phím qua engine khi tắt tiếng Việt). */
    int       allow_macro_when_vi_off;
} lc_config_state;

/* $XDG_CONFIG_HOME/TextVN/<name> hoặc ~/.config/TextVN/<name>. */
int lc_textvn_file_path(char *out, size_t max_len, const char *name);
/* $XDG_CONFIG_HOME/TextVN/config.json hoặc ~/.config/TextVN/config.json. */
int lc_config_resolve_path(char *out, size_t max_len);
/* Nạp lại config vào engine khi file đổi (mtime/size). Trả 1 nếu đã nạp lại.
 * `path` NULL = đường dẫn mặc định. Rẻ (một stat) — gọi ở focus-in và đầu mỗi từ. */
int lc_config_sync(ime_instance *inst, lc_config_state *st, const char *path);

/* state.json: {"global_enabled": bool, "apps": {...}} — trạng thái bật/tắt tiếng Việt,
 * nhớ qua các lần khởi động và chia sẻ với bảng cài đặt (giống tray Windows). */
int lc_state_resolve_path(char *out, size_t max_len);
/* global_enabled; file thiếu/hỏng → `default_enabled`. `path` NULL = mặc định. */
int lc_state_read_enabled(const char *path, int default_enabled);
/* Ghi global_enabled nguyên tử (giữ các khoá khác). 0 = OK. */
int lc_state_write_enabled(const char *path, int enabled);
/* File đổi kể từ lần trước → đọc lại, gán *enabled, trả 1. Không đổi/không có → 0. */
int lc_state_sync(lc_config_state *st, const char *path, int *enabled);

/* Tìm binary bảng cài đặt `textvn-settings` (theo thứ tự): cạnh `self_dir`,
 * `self_dir/../../bin` (prefix/lib/textvn → prefix/bin), $PATH, ~/.local/bin.
 * `self_dir` có thể NULL. 0 = tìm thấy (đường dẫn tuyệt đối trong `out`). */
int lc_find_settings_binary(char *out, size_t max_len, const char *self_dir);

#ifdef __cplusplus
}
#endif

#endif /* TEXTVN_LC_COMPOSE_H */
