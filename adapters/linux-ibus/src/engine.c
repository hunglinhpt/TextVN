/* engine.c — IBusEngine của TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Mô hình: cả từ đang gõ nằm trong PREEDIT (lc_compose.h). Mỗi phím:
 *   modifier → chỉ theo dõi Ctrl+Shift | hotkey → đảo VN/EN
 *   chord/secure/tắt VN → commit preedit nguyên văn, phím đi thẳng (B6, S3)
 *   còn lại → ime_key → lc_plan_key → cập nhật/commit preedit → eaten?
 *
 * Trạng thái VN/EN nằm ở ~/.config/TextVN/state.json (như tray Windows): nhớ qua lần
 * khởi động sau, và bảng cài đặt đổi được khi IME đang chạy. Tắt VN nhưng bật
 * "Gõ tắt cả khi tắt tiếng Việt" → phím vẫn qua engine (passthrough) để bung gõ tắt.
 *
 * Commit-before-hide (B2): preedit gửi với IBUS_ENGINE_PREEDIT_COMMIT nên ibus-daemon
 * commit nó khi client reset / mất focus; engine chỉ quên từ ở các sự kiện đó.
 * Các commit chủ động (ranh giới từ, chord, tắt VN, disable) do engine làm.
 * Không reset theo set_surrounding_text: app báo caret mới sau MỖI lần commit, bản
 * cũ reset engine ở đó nên không bao giờ biến đổi được chữ (không gõ được tiếng Việt).
 */

#include "engine.h"
#include "keymap.h"

#include "lc_assist_text.h"

#include <string.h>

G_DEFINE_TYPE(TextVNIbusEngine, textvn_ibus_engine, IBUS_TYPE_ENGINE)

/* Trạng thái VN/EN dùng chung cho mọi input context của process (như UniKey). */
static gboolean        s_vi_enabled = TRUE;
static gboolean        s_state_loaded = FALSE;
static lc_config_state s_state_watch;
static lc_ipc_client  *s_ipc = NULL;
static uint32_t       s_global_seq_seen = 0; /* BUG-03: seq "*" da tieu thu */
static int            s_base_vi = TRUE;  /* C4: trang thai tu state.json/broadcast "*" */

/* Trạng thái Assist cấp tiến trình (AS-060 / P4-6) */
static gboolean        s_assist_enabled = FALSE;
static gboolean        s_assist_state_loaded = FALSE;
static lc_config_state s_assist_state_watch;
static uint32_t        s_assist_seq_seen = 0;
static int             s_base_assist = FALSE;

/* Buffer ghép "text trước caret (cache) + text vừa commit" khi kiểm tra
 * precondition D12. Chỉ so ĐUÔI nên cắt phần đầu là an toàn. */
#define ASSIST_SURROUNDING_MAX 4096

#define PROP_MODE  "TextVN.InputMode"
#define PROP_SETUP "TextVN.Setup"

/* ---- Tiện ích text ---- */

static IBusText *text_from_ucs4(const uint32_t *text, size_t len) {
    gunichar buf[LC_COMP_MAX + 1];
    if (len > LC_COMP_MAX) len = LC_COMP_MAX;
    memcpy(buf, text, len * sizeof(gunichar));
    buf[len] = 0;
    return ibus_text_new_from_ucs4(buf);
}

static void show_preedit(TextVNIbusEngine *self, const uint32_t *text, size_t len) {
    IBusEngine *engine = (IBusEngine *)self;
    if (len == 0) {
        ibus_engine_hide_preedit_text(engine);
        ibus_engine_update_preedit_text(engine, ibus_text_new_from_static_string(""), 0, FALSE);
        return;
    }
    IBusText *t = text_from_ucs4(text, len);
    /* BUG-06 (audit 2026-10-04): preedit Ở ĐÂY do CLIENT vẽ (client preedit
     * chuẩn) — không set attribute nào nên app tự quyết định gạch chân hay
     * không; TextVN KHÔNG tắt được gạch chân qua API IBus. Muốn "không gạch
     * chân" thật phải chuyển sang non-preedit (delete+commit) — đã cân nhắc và
     * HOÃN vì surrounding không đáng tin ở Chromium/Electron/Qt (xem
     * lc_compose.h:11-14, RL5/B7). */
    /* PREEDIT_COMMIT: khi mất focus, ibus-daemon/client tự commit preedit — engine
     * commit sau focus-out bị daemon bỏ qua (đã kiểm chứng bằng tests/e2e_ibus.c). */
    ibus_engine_update_preedit_text_with_mode(engine, t, (guint)len, TRUE,
                                              IBUS_ENGINE_PREEDIT_COMMIT);
}

/* Commit `text` và đóng preedit. */
static void commit_text(TextVNIbusEngine *self, const uint32_t *text, size_t len) {
    show_preedit(self, NULL, 0);
    if (len > 0) {
        ibus_engine_commit_text((IBusEngine *)self, text_from_ucs4(text, len));
    }
}

/* Ẩn giao diện gợi ý Mode 1 */
static void clear_assist_ui(TextVNIbusEngine *self) {
    if (!self->assist_ui_showing) return;
    self->assist_ui_showing = FALSE;
    self->assist_candidate_count = 0;
    IBusEngine *engine = (IBusEngine *)self;
    if (self->lookup_table) {
        ibus_lookup_table_clear(self->lookup_table);
        ibus_engine_hide_lookup_table(engine);
    }
    ibus_engine_hide_auxiliary_text(engine);
}

/* Commit preedit hiện tại nguyên văn + reset engine (focus/chord/secure/toggle). */
static void finish_word(TextVNIbusEngine *self) {
    clear_assist_ui(self);
    /* R-4: chốt từ => trạng thái hoàn tác cũng hết hiệu lực (caret có thể đã đổi) */
    self->assist_undo_valid = FALSE;
    if (self->comp.len > 0) {
        commit_text(self, self->comp.text, self->comp.len);
        self->comp.len = 0;
    }
    if (self->inst) ime_reset(self->inst);
}

/* ---- Trạng thái VN/EN ---- */

static void update_mode_prop(TextVNIbusEngine *self) {
    if (!self->mode_prop) return;
    ibus_property_set_label(self->mode_prop,
                            ibus_text_new_from_static_string(s_vi_enabled ? "VN" : "EN"));
    ibus_property_set_symbol(self->mode_prop,
                             ibus_text_new_from_static_string(s_vi_enabled ? "V" : "E"));
    ibus_property_set_icon(self->mode_prop, s_vi_enabled ? "textvn_v" : "textvn_e");
    ibus_engine_update_property((IBusEngine *)self, self->mode_prop);
}

static void set_vietnamese(TextVNIbusEngine *self, gboolean on, gboolean persist) {
    if (on == s_vi_enabled) return;
    finish_word(self);
    s_vi_enabled = on;
    if (persist) {
        /* R2-107: phím chuyển/menu là công tắc TOÀN CỤC → cập nhật cả base (C4). Lần
         * ghi state.json dưới đây bị lc_state_sync "hấp thụ" nên sync_state không bao
         * giờ thấy nó; thiếu dòng này, lần focus kế tiếp gán s_vi_enabled = base CŨ —
         * đổi V/E rồi chuyển cửa sổ là bị lật lại. */
        s_base_vi = on;
        lc_state_write_enabled(NULL, on ? 1 : 0);
        int ignored = on;
        lc_state_sync(&s_state_watch, NULL, &ignored); /* hấp thụ lần ghi của chính mình */
    }
    if (s_ipc) lc_ipc_client_toggle_vi_en(s_ipc, "*", on ? 1 : 0);
    update_mode_prop(self);
}

static void toggle_vietnamese(TextVNIbusEngine *self) {
    set_vietnamese(self, !s_vi_enabled, TRUE);
}

/* Bảng cài đặt vừa đổi state.json → theo ngay (một stat mỗi lần gọi). */
/* Vong 13 (BUG-03): tieu thu broadcast toan cuc "*" cua tray — ap dung dung
 * MOT lan moi su kien (so sanh seq, khong sticky de tranh xung dot voi kenh
 * state.json). Poll chi chay tai init/focus, khong tren duong phim. */
static void consume_global_ipc(void) {
    if (!s_ipc) return;
    lc_ipc_client_poll(s_ipc);
    int g = 0;
    uint32_t seq = 0;
    if (lc_ipc_client_get_global_override(s_ipc, &g, &seq) && seq != s_global_seq_seen) {
        s_global_seq_seen = seq;
        s_base_vi = g ? TRUE : FALSE; /* C4: broadcast "*" = base */
        s_vi_enabled = g ? TRUE : FALSE;
    }
}

static void sync_state(TextVNIbusEngine *self) {
    if (!s_state_loaded) {
        s_vi_enabled = lc_state_read_enabled(NULL, 1) ? TRUE : FALSE;
        s_state_loaded = TRUE;
    }
    int enabled = s_vi_enabled;
    if (lc_state_sync(&s_state_watch, NULL, &enabled)) {
        s_base_vi = enabled ? TRUE : FALSE; /* C4: file = nguon cua base */
        set_vietnamese(self, enabled ? TRUE : FALSE, FALSE);
    }
}

static void consume_assist_ipc(void) {
    if (!s_ipc) return;
    int a = 0;
    uint32_t seq = 0;
    if (lc_ipc_client_get_assist_override(s_ipc, &a, &seq) && seq != s_assist_seq_seen) {
        s_assist_seq_seen = seq;
        s_base_assist = (a != 0);
        s_assist_enabled = (a != 0);
    }
}

static void sync_assist_state(TextVNIbusEngine *self) {
    (void)self;
    if (!s_assist_state_loaded) {
        s_assist_enabled = lc_state_read_assist_enabled(NULL, 1) ? TRUE : FALSE;
        s_base_assist = s_assist_enabled;
        s_assist_state_loaded = TRUE;
    }
    int enabled = s_assist_enabled ? 1 : 0;
    if (lc_state_sync_assist(&s_assist_state_watch, NULL, &enabled)) {
        s_base_assist = (enabled != 0);
        s_assist_enabled = (enabled != 0);
    }
}

/* R-2: So sánh text trước con trỏ với chuỗi mong đợi (Precondition D12)
 *
 * IBus **không** cho truy vấn trực tiếp: surrounding text là BẢN CACHE do
 * client gửi lên (`ibus_input_context_set_surrounding_text`) ⇒ có thể cũ.
 * Không xác minh được (client không hỗ trợ / cache rỗng / caret nằm ngoài text)
 * ⇒ trả FALSE ⇒ **không sửa** (fail-safe, D12).
 *
 * `committed` = text engine **vừa commit trong phím này** (rỗng nếu không có).
 * Client (GTK) gửi surrounding text **TRƯỚC** khi xử lý phím ⇒ cache chưa có
 * phần engine vừa commit (ví dụ preedit được chốt khi gõ dấu câu) ⇒ phải ghép
 * thêm phần đó mới so được với đuôi câu engine biết.
 *
 * `cursor_pos` của IBus đếm KÝ TỰ Unicode (không phải byte) — phải dịch sang
 * byte offset. Dịch bằng vòng lặp tay: `g_utf8_offset_to_pointer` trả NULL khi
 * offset == số ký tự (caret ở cuối text) ⇒ phép trừ con trỏ thành UB. */
static gboolean text_before_caret_matches(TextVNIbusEngine *self, const char *expected,
                                         const char *committed) {
    if (!self || !expected || expected[0] == '\0') return FALSE;
    IBusEngine *engine = (IBusEngine *)self;
    if (!(engine->client_capabilities & IBUS_CAP_SURROUNDING_TEXT)) return FALSE;

    IBusText *text = NULL;
    guint cursor_pos = 0;
    guint anchor_pos = 0;
    ibus_engine_get_surrounding_text(engine, &text, &cursor_pos, &anchor_pos);
    if (!text) return FALSE;

    /* ibus_text_get_text trả con trỏ NỘI bộ (không copy, không free). */
    const gchar *str = ibus_text_get_text(text);
    if (!str) return FALSE;

    const gchar *cursor_ptr = str;
    guint i = 0;
    while (i < cursor_pos && *cursor_ptr) {
        cursor_ptr = g_utf8_next_char(cursor_ptr);
        i++;
    }
    /* Caret nằm ngoài text ⇒ dữ liệu không nhất quán ⇒ không xác minh được. */
    if (i != cursor_pos) return FALSE;

    size_t before_len = (size_t)(cursor_ptr - str);
    size_t committed_len = committed ? strlen(committed) : 0;

    /* Ghép: text trước caret (cache) + text vừa commit. Chỉ cần ĐUÔI nên cắt
     * phần đầu (không ảnh hưởng phép so — ranh giới char ở cuối giữ nguyên). */
    char merged[ASSIST_SURROUNDING_MAX];
    size_t keep = before_len;
    size_t room = sizeof(merged) - committed_len - 1;
    if (keep > room) keep = room;
    memcpy(merged, str + (before_len - keep), keep);
    size_t merged_len = keep;
    if (committed_len > 0) {
        memcpy(merged + merged_len, committed, committed_len);
        merged_len += committed_len;
    }
    merged[merged_len] = '\0';

    /* Hàm so ĐUÔI dùng chung (linux-common) — cùng quy tắc với Fcitx5/macOS. */
    return lc_text_ends_with(merged, merged_len, expected, strlen(expected)) != 0;
}

/* Áp dụng ứng viên được chọn trong Mode 1 */
static void apply_assist_candidate(TextVNIbusEngine *self, guint index) {
    if (!self || index >= self->assist_candidate_count) return;
    IBusEngine *engine = (IBusEngine *)self;
    TextVNAssistCandidate *cand = &self->assist_candidates[index];

    if (cand->delete_glyphs > 0) {
        if (!(engine->client_capabilities & IBUS_CAP_SURROUNDING_TEXT)) {
            clear_assist_ui(self);
            return;
        }
        if (cand->original_tail[0] != '\0' && !text_before_caret_matches(self, cand->original_tail, "")) {
            clear_assist_ui(self);
            return;
        }
        /* W1-audit: FFI chỉ trả kế hoạch khi `delete_glyphs` == số char Unicode
           của phần bị xoá (bảng mã xuất ≠ Unicode ⇒ không có kế hoạch), nên
           dùng `delete_glyphs` làm số **char** cho IBus là đúng ngữ nghĩa. */
        ibus_engine_delete_surrounding_text(engine, -(gint)cand->delete_glyphs, cand->delete_glyphs);
    }
    if (cand->replacement[0] != '\0') {
        ibus_engine_commit_text(engine, ibus_text_new_from_string(cand->replacement));
    }
    clear_assist_ui(self);
}

/* Hoàn tác 1 bước (Single-step Undo Ctrl+Z trong 5s) */
static gboolean apply_assist_undo(TextVNIbusEngine *self) {
    if (!self || !self->inst || !self->assist_undo_valid) return FALSE;
    guint64 now_ms = (guint64)(g_get_monotonic_time() / 1000);
    if (now_ms - self->assist_undo_timestamp_ms > 5000) {
        self->assist_undo_valid = FALSE;
        return FALSE;
    }
    if (!text_before_caret_matches(self, self->assist_undo_replacement, "")) {
        return FALSE;
    }
    IBusEngine *engine = (IBusEngine *)self;
    if (self->assist_undo_replacement_glyphs > 0) {
        ibus_engine_delete_surrounding_text(engine,
            -(gint)self->assist_undo_replacement_glyphs,
            self->assist_undo_replacement_glyphs);
    }
    ibus_engine_commit_text(engine, ibus_text_new_from_string(self->assist_undo_original_tail));
    self->assist_undo_valid = FALSE;
    ime_reset(self->inst);
    return TRUE;
}

/* Kiểm tra và thực thi Trợ lý ngữ pháp (AS-060 / P4-6)
 *
 * `committed` = text engine vừa commit trong phím này (UTF-8, rỗng nếu không
 * có) — dùng để dựng trạng thái document khi so precondition (xem
 * `text_before_caret_matches`). */
static gboolean check_and_apply_assist(TextVNIbusEngine *self, uint32_t trigger_char,
                                       gboolean is_sentence_trigger, gboolean plan_eaten,
                                       const char *committed) {
    if (!self || !self->inst) return plan_eaten;
    if (!s_vi_enabled || !lc_assist_is_active(s_assist_enabled ? 1 : 0, self->config.assist_mode)) {
        return plan_eaten;
    }

    ime_assist_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    if (ime_assist_check_instance(self->inst, &res) != IME_OK || res.count == 0) {
        return plan_eaten;
    }

    /* D16 (Fassist-20) — Q-6: crate `assist` luôn trả L6 ("chữ thường đầu
     * câu"); khi `config.auto_capitalize` bật thì IME đã tự viết hoa ⇒ báo/sửa
     * L6 là trùng chức năng. Lọc **sau** khi engine đã check (giữ nguyên các
     * luật khác) và **trước** cả Mode 2 lẫn Mode 1 ⇒ cả hai đều không thấy L6.
     * Cùng hàm với Fcitx5 (`lc_assist_filter_auto_capitalize`). */
    if (lc_assist_filter_auto_capitalize(&res, lc_assist_auto_capitalize_enabled(&self->config)) == 0) {
        return plan_eaten;
    }

    const gboolean is_auto_correct_mode =
        lc_assist_should_auto_fix(self->config.assist_mode, self->config.assist_auto_fix,
                                  is_sentence_trigger ? 1 : 0) != 0;

    char sentence[256 * 4 + 1];
    size_t actual_len = 0;
    if (ime_assist_sentence_tail(self->inst, 256, sentence, sizeof(sentence), &actual_len) != IME_OK ||
        actual_len == 0 || actual_len >= sizeof(sentence)) {
        return plan_eaten;
    }
    sentence[actual_len] = '\0';

    IBusEngine *engine = (IBusEngine *)self;

    if (is_auto_correct_mode) {
        gboolean surrounding = (engine->client_capabilities & IBUS_CAP_SURROUNDING_TEXT) != 0;
        if (surrounding) {
            lc_sentence_fix_plan fix_plan;
            if (lc_plan_sentence_auto_fix(self->inst, sentence, &res, &fix_plan) && fix_plan.fixed_count > 0) {
                /* R-2 (D12) + khác biệt IBus: surrounding text là BẢN CACHE,
                 * **không** phản ánh commit vừa rồi. Vì vậy kiểm tra
                 * precondition **TRƯỚC** khi commit ký tự trigger, và so với
                 * đuôi document **không** chứa ký tự trigger khi trigger chưa
                 * nằm trong document (app sẽ tự chèn sau).
                 *
                 * Sai lệch cốt lõi: commit trigger TRƯỚC rồi mới kiểm tra ⇒
                 * (1) cache không có trigger ⇒ điều kiện không khớp ⇒ bỏ sửa,
                 * (2) bỏ sửa nhưng đã commit trigger ⇒ trả FALSE ⇒ app chèn
                 * thêm ⇒ **nhân đôi ký tự**. */
                char expected[LC_MAX_SENTENCE_FIX_CHARS];
                if (!lc_assist_expected_document_tail(fix_plan.original_tail, trigger_char,
                                                       plan_eaten ? 1 : 0,
                                                       expected, sizeof(expected))) {
                    return plan_eaten; /* không xác minh được ⇒ KHÔNG sửa */
                }
                if (!text_before_caret_matches(self, expected, committed)) {
                    return plan_eaten; /* lệch ⇒ KHÔNG sửa (fail-safe, D12) */
                }
                /* Precondition đã khớp: commit ký tự trigger (nếu chưa có trong
                 * document) để document khớp buffer câu của engine, rồi xoá đúng
                 * số glyph và gõ lại bản sửa. */
                if (!plan_eaten && trigger_char != 0) {
                    ibus_engine_commit_text(engine, text_from_ucs4(&trigger_char, 1));
                }
                if (fix_plan.delete_glyphs > 0) {
                    /* W1-audit: như `apply_assist_candidate` — FFI đã bảo đảm
                       glyph == char Unicode trước khi trả kế hoạch. */
                    ibus_engine_delete_surrounding_text(engine, -(gint)fix_plan.delete_glyphs,
                                                        fix_plan.delete_glyphs);
                }
                ibus_engine_commit_text(engine, ibus_text_new_from_string(fix_plan.replacement));
                if (self->inst) ime_reset(self->inst);

                size_t repl_glyphs = 0;
                if (ime_assist_glyph_len(self->inst, fix_plan.replacement,
                                         strlen(fix_plan.replacement), &repl_glyphs) == IME_OK &&
                    repl_glyphs > 0) {
                    self->assist_undo_valid = TRUE;
                    g_strlcpy(self->assist_undo_original_tail, fix_plan.original_tail,
                              sizeof(self->assist_undo_original_tail));
                    g_strlcpy(self->assist_undo_replacement, fix_plan.replacement,
                              sizeof(self->assist_undo_replacement));
                    self->assist_undo_replacement_glyphs = (uint32_t)repl_glyphs;
                    self->assist_undo_timestamp_ms = (guint64)(g_get_monotonic_time() / 1000);
                }
                if (s_ipc) {
                    lc_ipc_client_send_assist_report(s_ipc, &res, fix_plan.fixed_count);
                }
                return TRUE;
            }
        }
    }

    if (lc_assist_should_display(self->config.assist_mode, is_sentence_trigger ? 1 : 0)) {
        if (!self->lookup_table) {
            self->lookup_table = ibus_lookup_table_new(3, 0, TRUE, TRUE);
            ibus_lookup_table_set_orientation(self->lookup_table, IBUS_ORIENTATION_VERTICAL);
            g_object_ref_sink(self->lookup_table);
        } else {
            ibus_lookup_table_clear(self->lookup_table);
        }

        guint added = 0;
        for (uint32_t i = 0; i < res.count && added < 3; ++i) {
            const ime_assist_issue_v1 *issue = &res.issues[i];
            if (issue->suggest[0] == '\0') continue;

            ime_assist_result_v1 single;
            memset(&single, 0, sizeof(single));
            single.abi_version = IME_ABI_VERSION;
            single.count = 1;
            single.issues[0] = *issue;

            /* Không lập được kế hoạch sửa ⇒ KHÔNG hiện ứng viên (nếu hiện,
             * chọn sẽ **chèn** gợi ý vào caret thay vì thay lỗi — sai).
             * Cùng quy tắc với macOS (AssistModel). */
            lc_sentence_fix_plan single_plan;
            if (!lc_plan_sentence_auto_fix(self->inst, sentence, &single, &single_plan)) {
                continue;
            }
            uint32_t del_glyphs = single_plan.delete_glyphs;
            const char *rep = single_plan.replacement;
            const char *orig_tail = single_plan.original_tail;

            TextVNAssistCandidate *cand = &self->assist_candidates[added];
            cand->delete_glyphs = del_glyphs;
            g_strlcpy(cand->replacement, rep, sizeof(cand->replacement));
            g_strlcpy(cand->original_tail, orig_tail, sizeof(cand->original_tail));

            char label[256];
            if (issue->message[0] != '\0') {
                g_snprintf(label, sizeof(label), "%s (%s)", issue->suggest, issue->message);
            } else {
                g_strlcpy(label, issue->suggest, sizeof(label));
            }
            ibus_lookup_table_append_candidate(self->lookup_table, ibus_text_new_from_string(label));
            added++;
        }

        if (added > 0) {
            self->assist_candidate_count = added;
            self->assist_ui_showing = TRUE;

            char aux[256];
            g_snprintf(aux, sizeof(aux), "Trợ lý TextVN: %s", res.issues[0].message);
            ibus_engine_update_auxiliary_text(engine, ibus_text_new_from_string(aux), TRUE);
            ibus_engine_update_lookup_table(engine, self->lookup_table, TRUE);

            if (s_ipc) {
                lc_ipc_client_send_assist_report(s_ipc, &res, 0);
            }
        }
    }

    return plan_eaten;
}

static void push_context(TextVNIbusEngine *self, int enabled) {
    if (!self->inst) return;
    ime_context_v1 ctx;
    memset(&ctx, 0, sizeof(ctx));
    ctx.abi_version = IME_ABI_VERSION;
    ctx.enabled = enabled ? 1 : 0;
    ctx.secure = 0;
    ctx.field_role = IME_FIELD_BODY;
    ctx.caps = IME_CAP_PREEDIT | IME_CAP_SELECTION;
    ctx.hint = -1;
    ime_set_context(self->inst, &ctx);
    self->ctx_enabled = enabled ? 1 : 0;
}

/* Mở bảng điều khiển (textvn-settings cạnh bản cài / trong PATH). */
static void launch_settings(void) {
    gchar *exe = g_file_read_link("/proc/self/exe", NULL);
    gchar *dir = exe ? g_path_get_dirname(exe) : NULL;
    char path[1024];
    if (lc_find_settings_binary(path, sizeof(path), dir) == 0) {
        gchar *argv[] = {path, NULL};
        GError *err = NULL;
        /* Không DO_NOT_REAP: GLib tự thu dọn tiến trình con (không để zombie). */
        if (!g_spawn_async(NULL, argv, NULL, G_SPAWN_DEFAULT, NULL, NULL, NULL, &err)) {
            lc_log(LC_LOG_WARN, "IBus", "cannot launch textvn-settings");
            g_clear_error(&err);
        }
    } else {
        lc_log(LC_LOG_WARN, "IBus", "textvn-settings not found");
    }
    g_free(dir);
    g_free(exe);
}

/* ---- GObject ---- */

static void textvn_ibus_engine_init(TextVNIbusEngine *self) {
    memset(&self->comp, 0, sizeof(self->comp));
    memset(&self->config, 0, sizeof(self->config));
    /* Vong 26/28: fresh-install (chưa có config) -> HIGHLIGHT — struct zeroed
     * là OFF trước đây, lệch mặc định 3 nen tang. */
    self->config.assist_mode = LC_ASSIST_MODE_HIGHLIGHT;
    lc_modifier_toggle_reset(&self->toggle);
    self->secure = FALSE;
    self->inst = NULL;
    if (ime_instance_new(NULL, 0, &self->inst) != IME_OK || !self->inst) {
        lc_log(LC_LOG_ERROR, "IBus", "ime_instance_new failed; engine fail-open");
        self->inst = NULL;
    }
    lc_config_sync(self->inst, &self->config, NULL);
    push_context(self, 1);
    if (!s_ipc) s_ipc = lc_ipc_client_new("ibus", NULL);
    consume_global_ipc();
    if (!s_state_loaded) {
        s_vi_enabled = lc_state_read_enabled(NULL, 1) ? TRUE : FALSE;
        s_state_loaded = TRUE;
    }

    self->props = ibus_prop_list_new();
    g_object_ref_sink(self->props);
    self->mode_prop = ibus_property_new(PROP_MODE, PROP_TYPE_NORMAL,
                                        ibus_text_new_from_static_string("VN"), "textvn_v",
                                        ibus_text_new_from_static_string(
                                            "Bật/tắt tiếng Việt (Ctrl+Shift)"),
                                        TRUE, TRUE, PROP_STATE_UNCHECKED, NULL);
    g_object_ref_sink(self->mode_prop);
    ibus_prop_list_append(self->props, self->mode_prop);
    self->setup_prop = ibus_property_new(PROP_SETUP, PROP_TYPE_NORMAL,
                                         ibus_text_new_from_static_string("Cài đặt TextVN…"),
                                         "preferences-system",
                                         ibus_text_new_from_static_string(
                                             "Mở bảng điều khiển TextVN"),
                                         TRUE, TRUE, PROP_STATE_UNCHECKED, NULL);
    g_object_ref_sink(self->setup_prop);
    ibus_prop_list_append(self->props, self->setup_prop);

    /* Trợ lý ngữ pháp (AS-060 / P4-6) */
    self->lookup_table = ibus_lookup_table_new(3, 0, TRUE, TRUE);
    ibus_lookup_table_set_orientation(self->lookup_table, IBUS_ORIENTATION_VERTICAL);
    g_object_ref_sink(self->lookup_table);
    self->assist_ui_showing = FALSE;
    self->assist_candidate_count = 0;
    self->assist_undo_valid = FALSE;
    self->assist_undo_timestamp_ms = 0;
    self->assist_undo_replacement_glyphs = 0;
    self->assist_undo_original_tail[0] = '\0';
    self->assist_undo_replacement[0] = '\0';
    sync_assist_state(self);
    consume_assist_ipc();
}

static void textvn_ibus_engine_finalize(GObject *object) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)object;
    clear_assist_ui(self);
    if (self->lookup_table) {
        g_clear_object(&self->lookup_table);
    }
    if (self->inst) {
        ime_instance_free(self->inst);
        self->inst = NULL;
    }
    g_clear_object(&self->mode_prop);
    g_clear_object(&self->setup_prop);
    g_clear_object(&self->props);
    G_OBJECT_CLASS(textvn_ibus_engine_parent_class)->finalize(object);
}

/* ---- IBusEngine vfuncs ---- */

/* Quên từ đang gõ mà KHÔNG commit: preedit (PREEDIT_COMMIT) do ibus-daemon commit khi
 * client reset/mất focus — engine commit thêm sẽ thành chữ lặp (kiểm chứng e2e). */
static void forget_word(TextVNIbusEngine *self) {
    clear_assist_ui(self);
    self->assist_undo_valid = FALSE;
    self->comp.len = 0;
    if (self->inst) ime_reset(self->inst);
}

static void engine_enable(IBusEngine *engine) {
    /* Đăng ký yêu cầu surrounding text với ibus-daemon và client application */
    ibus_engine_get_surrounding_text(engine, NULL, NULL, NULL);
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->enable(engine);
}

static void engine_focus_in(IBusEngine *engine) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    lc_config_sync(self->inst, &self->config, NULL);
    sync_state(self);
    consume_global_ipc();
    sync_assist_state(self);
    consume_assist_ipc();
    int app_enabled = 1;
    if (s_ipc && lc_ipc_client_get_app_override(s_ipc, &app_enabled)) {
        s_vi_enabled = app_enabled != 0;
    } else {
        /* C4 (audit): override da bi clear (app khong co trong snapshot) —
         * phai ve ve BASE (state.json/broadcast "*"), khong de ke thừa EN cua
         * app truoc. */
        s_vi_enabled = s_base_vi;
    }
    ibus_engine_get_surrounding_text(engine, NULL, NULL, NULL);
    ibus_engine_register_properties(engine, self->props);
    update_mode_prop(self);
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->focus_in(engine);
}

static void engine_focus_out(IBusEngine *engine) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    forget_word(self);
    lc_modifier_toggle_reset(&self->toggle);
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->focus_out(engine);
}

static void engine_reset(IBusEngine *engine) {
    forget_word((TextVNIbusEngine *)engine);
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->reset(engine);
}

static void engine_disable(IBusEngine *engine) {
    finish_word((TextVNIbusEngine *)engine);
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->disable(engine);
}

static void engine_set_content_type(IBusEngine *engine, guint purpose, guint hints) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    gboolean secure = purpose == IBUS_INPUT_PURPOSE_PASSWORD || purpose == IBUS_INPUT_PURPOSE_PIN;
    if (secure && !self->secure) finish_word(self);
    self->secure = secure;
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->set_content_type(engine, purpose, hints);
}

static void engine_property_activate(IBusEngine *engine, const gchar *name, guint state) {
    if (g_strcmp0(name, PROP_MODE) == 0) {
        toggle_vietnamese((TextVNIbusEngine *)engine);
        return;
    }
    if (g_strcmp0(name, PROP_SETUP) == 0) {
        launch_settings();
        return;
    }
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->property_activate(engine, name, state);
}

static void engine_candidate_clicked(IBusEngine *engine, guint index, guint button, guint state) {
    (void)button;
    (void)state;
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    if (self->assist_ui_showing && index < self->assist_candidate_count) {
        apply_assist_candidate(self, index);
    }
}

static gboolean handle_release(TextVNIbusEngine *self, guint keyval) {
    if (lc_modifier_toggle_up(&self->toggle, textvn_ibus_modifier_kind(keyval))) {
        toggle_vietnamese(self);
    }
    return FALSE; /* không bao giờ nuốt phím nhả */
}

static gboolean engine_process_key_event(IBusEngine *engine, guint keyval, guint keycode,
                                         guint state) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    (void)keycode;

    if (state & IBUS_RELEASE_MASK) return handle_release(self, keyval);

    lc_modifier which = textvn_ibus_modifier_kind(keyval);
    lc_modifier_toggle_down(&self->toggle, which, (state & IBUS_CONTROL_MASK) != 0,
                            (state & IBUS_SHIFT_MASK) != 0,
                            (state & (IBUS_MOD1_MASK | IBUS_SUPER_MASK | IBUS_MOD4_MASK)) != 0);
    if (which != LC_MOD_NONE) return FALSE; /* modifier đứng riêng: không commit */

    /* Xử lý phím khi giao diện gợi ý trợ lý đang mở */
    if (self->assist_ui_showing) {
        if (keyval == IBUS_KEY_Escape) {
            clear_assist_ui(self);
            return TRUE;
        }
        if (!(state & (IBUS_CONTROL_MASK | IBUS_MOD1_MASK | IBUS_SUPER_MASK)) &&
            keyval >= IBUS_KEY_1 && keyval <= IBUS_KEY_3) {
            guint idx = keyval - IBUS_KEY_1;
            if (idx < self->assist_candidate_count) {
                apply_assist_candidate(self, idx);
                return TRUE;
            }
        }
        clear_assist_ui(self);
    }

    uint32_t mods = textvn_ibus_mods(state);

    /* R-4 (AS-050): Ctrl+Z hoàn tác tự sửa cả câu (1 bước, hạn 5 s).
     * R-3: mọi phím KHÁC => huỷ state hoàn tác. */
    gboolean is_assist_undo_chord =
        ((state & (IBUS_CONTROL_MASK | IBUS_MOD1_MASK | IBUS_SUPER_MASK | IBUS_SHIFT_MASK)) == IBUS_CONTROL_MASK) &&
        (keyval == IBUS_KEY_z || keyval == IBUS_KEY_Z);
    if (is_assist_undo_chord) {
        if (apply_assist_undo(self)) {
            return TRUE;
        }
    } else {
        self->assist_undo_valid = FALSE;
    }

    /* Ctrl+Shift+Space (ADR-011). */
    if ((mods & (IME_MOD_CTRL | IME_MOD_SHIFT)) == (IME_MOD_CTRL | IME_MOD_SHIFT) &&
        (keyval == IBUS_KEY_space || keyval == IBUS_KEY_KP_Space)) {
        /* The modifiers were armed for the Ctrl+Shift tap before Space arrived.
         * This shortcut already toggles now, so disarm it before the later
         * key-release notifications; otherwise releasing Shift/Control toggles
         * a second time and leaves the mode unchanged. */
        lc_modifier_toggle_reset(&self->toggle);
        toggle_vietnamese(self);
        return TRUE;
    }

    if (self->comp.len == 0) {
        lc_config_sync(self->inst, &self->config, NULL);
        sync_state(self);
        /* C3 (audit): khop diem poll voi Fcitx5 — toggle tray toi IBus ngay ca
         * khi user dang go lien (khong doi focus). */
        consume_global_ipc();
        sync_assist_state(self);
        consume_assist_ipc();
    }

    /* B6: chord hệ thống; S3: ô mật khẩu; VN tắt (trừ khi còn gõ tắt); engine hỏng →
     * commit, phím đi thẳng. */
    gboolean macro_only = !s_vi_enabled && self->config.allow_macro_when_vi_off;
    if ((mods & (IME_MOD_CTRL | IME_MOD_ALT | IME_MOD_SUPER | IME_MOD_META)) || self->secure ||
        !(s_vi_enabled || macro_only) || !self->inst) {
        finish_word(self);
        return FALSE;
    }
    if (self->ctx_enabled != (s_vi_enabled ? 1 : 0)) push_context(self, s_vi_enabled);

    uint32_t vk = 0, ch = 0;
    textvn_ibus_map_key(keyval, &vk, &ch);
    lc_key key = lc_key_classify(vk, ch);
    ime_key_v1 k;
    lc_key_to_ime(key, vk, mods & (IME_MOD_SHIFT | IME_MOD_CAPS), &k);

    ime_result_v1 r;
    if (ime_key(self->inst, &k, &r) != IME_OK || (r.flags & IME_FLAG_ERROR)) {
        finish_word(self);
        return FALSE;
    }

    lc_plan plan;
    lc_plan_key(&self->comp, key, &r, &plan);

    if (plan.delete_before > 0) {
        /* Chỉ xóa text đã commit khi client thật sự hỗ trợ surrounding text. */
        if (!(engine->client_capabilities & IBUS_CAP_SURROUNDING_TEXT)) {
            finish_word(self);
            return FALSE;
        }
        ibus_engine_delete_surrounding_text(engine, -(gint)plan.delete_before,
                                            plan.delete_before);
    }

    /* Text engine vừa commit trong phím này (rỗng nếu không commit) — dùng để
     * dựng trạng thái document khi kiểm tra precondition (D12): surrounding
     * text của IBus là bản cache client gửi lên TRƯỚC phím ⇒ chưa có phần này
     * (ví dụ preedit được chốt khi gõ dấu câu). */
    char committed[LC_COMP_MAX * 4 + 1];
    committed[0] = '\0';

    if (plan.end) {
        size_t n = 0;
        const uint32_t *t = lc_plan_commit_text(&self->comp, &plan, &n);
        commit_text(self, t, n);
        lc_utf32_to_utf8(t, n, committed, sizeof(committed));
    } else if (plan.has_text) {
        show_preedit(self, plan.text, plan.text_len);
    }
    lc_comp_apply(&self->comp, &plan);
    if (plan.reset_engine) ime_reset(self->inst);

    /* Trợ lý ngữ pháp (AS-060 / P4-6): trigger kiểm tra khi commit kết thúc câu hoặc dấu cách */
    uint32_t trigger_kind = (key.kind == LC_KEY_CHAR) ? ime_assist_trigger_kind(key.ch) : IME_ASSIST_TRIGGER_NONE;
    if (trigger_kind != IME_ASSIST_TRIGGER_NONE && s_vi_enabled && self->comp.len == 0 && self->inst) {
        gboolean is_sentence = (trigger_kind == IME_ASSIST_TRIGGER_SENTENCE);
        return check_and_apply_assist(self, key.ch, is_sentence, plan.eaten ? TRUE : FALSE,
                                      committed);
    }

    return plan.eaten ? TRUE : FALSE;
}

static void textvn_ibus_engine_class_init(TextVNIbusEngineClass *klass) {
    GObjectClass *gobject_class = G_OBJECT_CLASS(klass);
    gobject_class->finalize = textvn_ibus_engine_finalize;

    IBusEngineClass *engine_class = IBUS_ENGINE_CLASS(klass);
    engine_class->process_key_event = engine_process_key_event;
    engine_class->focus_in = engine_focus_in;
    engine_class->focus_out = engine_focus_out;
    engine_class->reset = engine_reset;
    engine_class->enable = engine_enable;
    engine_class->disable = engine_disable;
    engine_class->set_content_type = engine_set_content_type;
    engine_class->property_activate = engine_property_activate;
    engine_class->candidate_clicked = engine_candidate_clicked;
}
