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
#define PROP_DIAG  "TextVN.Diagnose"

/* ---- Tiện ích text ---- */

static IBusText *text_from_ucs4(const uint32_t *text, size_t len) {
    gunichar buf[LC_COMP_MAX + 1];
    if (len > LC_COMP_MAX) len = LC_COMP_MAX;
    memcpy(buf, text, len * sizeof(gunichar));
    buf[len] = 0;
    return ibus_text_new_from_ucs4(buf);
}

static void flush_edits(TextVNIbusEngine *self);

static void show_preedit(TextVNIbusEngine *self, const uint32_t *text, size_t len) {
    IBusEngine *engine = (IBusEngine *)self;
    /* R2-165: preedit MỚI phải tới sau phần commit đã gom (giữ đúng thứ tự trên dây). */
    if (len > 0) flush_edits(self);
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

/* ---- R2-165: mỗi phím gửi tối đa 1 xoá + 1 commit (lc_edit_batch.h) ----
 *
 * Mọi xoá-trước-caret và commit đi qua 3 hàm dưới. Đang trong một phím (batch active) thì
 * gom lại; hết phím (`edits_end`) gửi đúng MỘT `delete_surrounding_text` rồi MỘT
 * `commit_text`. Ngoài phím (focus/reset/timer) gửi thẳng như cũ. Trước khi làm việc phụ
 * thuộc thứ tự trên dây (forward phím, vẽ preedit mới, xoá vùng bôi đen) phải `flush_edits`. */
static void send_edits_now(TextVNIbusEngine *self, uint32_t del, const char *text, size_t len) {
    IBusEngine *engine = (IBusEngine *)self;
    if (del > 0) ibus_engine_delete_surrounding_text(engine, -(gint)del, del);
    if (len > 0) {
        gchar *s = g_strndup(text, len);
        ibus_engine_commit_text(engine, ibus_text_new_from_string(s));
        g_free(s);
    }
}

/* R2-167 (ibus-bamboo#574/#589, PR#581): commit TRƯỚC rồi mới đóng preedit. Đóng preedit
 * trước ⇒ Chrome/Electron coi là huỷ composition rồi chèn chữ mới riêng ⇒ editor React
 * (Messenger, Instagram, GitHub) lặp lại từ cuối sau Enter hoặc đẩy từ lên đầu câu. */
static void settle_preedit_after_commit(TextVNIbusEngine *self) {
    if (!self->hide_preedit_after_commit) return;
    self->hide_preedit_after_commit = FALSE;
    show_preedit(self, NULL, 0);
}

static void flush_edits(TextVNIbusEngine *self) {
    uint32_t del = 0;
    const char *text = NULL;
    size_t len = 0;
    if (lc_edit_batch_take(&self->edits, &del, &text, &len)) send_edits_now(self, del, text, len);
    settle_preedit_after_commit(self);
}

static void edits_begin(TextVNIbusEngine *self) {
    lc_edit_batch_begin(&self->edits);
}

static void edits_end(TextVNIbusEngine *self) {
    uint32_t del = 0;
    const char *text = NULL;
    size_t len = 0;
    if (lc_edit_batch_end(&self->edits, &del, &text, &len)) send_edits_now(self, del, text, len);
    if (!lc_edit_batch_active(&self->edits)) settle_preedit_after_commit(self);
}

/* Xoá `n` ký tự Unicode trước caret (đã kiểm IBUS_CAP_SURROUNDING_TEXT ở caller). */
static void send_delete_before(TextVNIbusEngine *self, uint32_t n) {
    if (n == 0) return;
    if (lc_edit_batch_active(&self->edits)) {
        lc_edit_batch_delete(&self->edits, n);
        return;
    }
    ibus_engine_delete_surrounding_text((IBusEngine *)self, -(gint)n, n);
}

static void send_commit_utf8(TextVNIbusEngine *self, const char *utf8) {
    if (!utf8 || !utf8[0]) return;
    const size_t len = strlen(utf8);
    if (lc_edit_batch_commit(&self->edits, utf8, len)) return;
    /* Không gom (ngoài phím) hoặc hết chỗ: giữ thứ tự — gửi phần đã gom trước. */
    flush_edits(self);
    ibus_engine_commit_text((IBusEngine *)self, ibus_text_new_from_string(utf8));
}

/* Commit `text` rồi đóng preedit (R2-167: thứ tự commit → ẩn preedit). */
static void commit_text(TextVNIbusEngine *self, const uint32_t *text, size_t len) {
    if (len > 0) {
        char utf8[LC_COMP_MAX * 4 + 1];
        if (len > LC_COMP_MAX) len = LC_COMP_MAX;
        lc_utf32_to_utf8(text, len, utf8, sizeof(utf8));
        send_commit_utf8(self, utf8);
    }
    if (lc_edit_batch_active(&self->edits)) {
        self->hide_preedit_after_commit = TRUE; /* ẩn ngay sau khi commit đã gom được gửi */
    } else {
        show_preedit(self, NULL, 0);
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

/* P12: ẩn/vẽ Bảng chọn Emoji nhẹ qua IBusLookupTable (chạy mượt X11 + Wayland). */
static void hide_emoji_ui(TextVNIbusEngine *self) {
    if (!self->emoji.active) return;
    lc_emoji_close(&self->emoji);
    IBusEngine *engine = (IBusEngine *)self;
    if (self->lookup_table) {
        ibus_lookup_table_clear(self->lookup_table);
        ibus_engine_hide_lookup_table(engine);
    }
    ibus_engine_hide_auxiliary_text(engine);
}

static void render_emoji_ui(TextVNIbusEngine *self) {
    IBusEngine *engine = (IBusEngine *)self;
    if (!self->emoji.active) return;
    if (!self->lookup_table) {
        self->lookup_table = ibus_lookup_table_new(LC_EMOJI_PAGE_SIZE, 0, TRUE, TRUE);
        g_object_ref_sink(self->lookup_table);
    }
    ibus_lookup_table_set_page_size(self->lookup_table, LC_EMOJI_PAGE_SIZE);
    ibus_lookup_table_clear(self->lookup_table);

    size_t page_start = (self->emoji.selected / LC_EMOJI_PAGE_SIZE) * LC_EMOJI_PAGE_SIZE;
    size_t page_end = page_start + LC_EMOJI_PAGE_SIZE;
    if (page_end > self->emoji.count) page_end = self->emoji.count;
    for (size_t i = page_start; i < page_end; i++) {
        char label[96];
        g_snprintf(label, sizeof(label), "%s :%s:",
                   self->emoji.items[i].glyph, self->emoji.items[i].shortcode);
        ibus_lookup_table_append_candidate(self->lookup_table, ibus_text_new_from_string(label));
    }
    if (page_end > page_start) {
        ibus_lookup_table_set_cursor_pos(self->lookup_table,
                                         (guint)(self->emoji.selected - page_start));
        ibus_engine_update_lookup_table(engine, self->lookup_table, TRUE);
    } else {
        ibus_engine_hide_lookup_table(engine);
    }
    char aux[256];
    lc_emoji_format_aux(&self->emoji, aux, sizeof(aux));
    ibus_engine_update_auxiliary_text(engine, ibus_text_new_from_string(aux), TRUE);
}

/* Commit preedit hiện tại nguyên văn + reset engine (focus/chord/secure/toggle). */
/* R2-134: ẩn bản xem trước dự đoán (dòng phụ) nếu đang hiện. */
static void hide_preview(TextVNIbusEngine *self) {
    if (!self->preview_showing) return;
    self->preview_showing = FALSE;
    if (!self->assist_ui_showing) ibus_engine_hide_auxiliary_text((IBusEngine *)self);
}

/* R2-134 (chủ repo: "chữ mờ gợi ý sẵn"): xem trước phần chữ Tab / phím tắt SẼ
 * nối thêm ở dòng phụ của bảng IBus cạnh con trỏ. Không chèn vào preedit: một
 * số toolkit tự chốt preedit khi mất focus ⇒ chữ gợi ý lọt vào văn bản. */
static void update_preview(TextVNIbusEngine *self) {
    if (!self->inst || self->assist_ui_showing || self->secure || self->terminal ||
        !(s_vi_enabled || self->config.suggest_enabled)) {
        hide_preview(self);
        return;
    }
    char ghost[512];
    char label[540];
    if (ime_peek_completion(self->inst, ghost, sizeof(ghost)) <= 0 ||
        !lc_preview_label(ghost, label, sizeof(label))) {
        hide_preview(self);
        return;
    }
    ibus_engine_update_auxiliary_text((IBusEngine *)self, ibus_text_new_from_string(label), TRUE);
    self->preview_showing = TRUE;
}

static void finish_word(TextVNIbusEngine *self) {
    hide_preview(self);
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

    /* Vòng 41 (Pass C C-41/1): `text` là REF transfer-full (ibusengine.c g_object_ref)
     * ⇒ MỌI exit path phải g_object_unref — như perform_selection_convert. Thiếu
     * unref = rò 1 IBusText (toàn bộ surrounding text) mỗi trigger assist ⇒ RSS
     * ibus-engine tăng dần trong phiên. Dùng goto-sweep cuối hàm. */
    gboolean matched = FALSE;
    /* ibus_text_get_text trả con trỏ NỘI bộ (không copy, không free). */
    const gchar *str = ibus_text_get_text(text);
    if (str) {
        const gchar *cursor_ptr = str;
        guint i = 0;
        while (i < cursor_pos && *cursor_ptr) {
            cursor_ptr = g_utf8_next_char(cursor_ptr);
            i++;
        }
        /* Caret nằm ngoài text ⇒ dữ liệu không nhất quán ⇒ không xác minh được. */
        if (i == cursor_pos) {
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
            matched = lc_text_ends_with(merged, merged_len, expected, strlen(expected)) != 0;
        }
    }
    g_object_unref(text);
    return matched;
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
        send_delete_before(self, cand->delete_glyphs);
    }
    if (cand->replacement[0] != '\0') {
        send_commit_utf8(self, cand->replacement);
    }
    /* G19: note correction to style profile for continuous learning */
    if (self->inst && cand->delete_glyphs > 0 && cand->original_tail[0] != '\0' && cand->replacement[0] != '\0') {
        ime_style_note_correction(self->inst, (const uint8_t *)cand->original_tail, strlen(cand->original_tail),
                                  (const uint8_t *)cand->replacement, strlen(cand->replacement));
    }
    /* R-4: sau khi sửa câu trong document, buffer câu của engine đã cũ ⇒
     * phải reset để câu tiếp theo không bị lệch span (parity với Fcitx5/Windows). */
    if (self->inst) {
        ime_reset(self->inst);
    }
    self->assist_undo_valid = FALSE;
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
    if (self->assist_undo_replacement_glyphs > 0) {
        send_delete_before(self, self->assist_undo_replacement_glyphs);
    }
    send_commit_utf8(self, self->assist_undo_original_tail);
    self->assist_undo_valid = FALSE;
    ime_reset(self->inst);
    return TRUE;
}

/* OP-007 (P5 §7.4) · R2-139: Ctrl+Shift+F9 — chuyển mã vùng bôi đen (như
 * Windows/macOS). IBus surrounding text là BẢN CACHE client đẩy lên TRƯỚC
 * phím (xem text_before_caret_matches) ⇒ selection có thể stale (RL5/B7);
 * đọc/xoá/commit đi qua `lc_selection_slice` để slice luôn nằm trên ranh
 * giới ký tự. Trần 64 000 ký tự — `selection_check::MAX_TRANSCODE_CHARS`
 * của Windows.
 *
 * cursor/anchor đếm theo KÝ TỰ Unicode; anchor == cursor khi không có
 * selection; selection = đoạn [min, max).
 *
 * Không selection / trần vượt / UTF-8 lỗi ⇒ KHÔNG làm gì và KHÔNG nuốt phím
 * (fail-open: F9 tới app — khác Windows 1 nhí vì API IBus đồng bộ, không có
 * RequestEditSession async). Đọc được selection mà không đổi gì ⇒ không ghi
 * document nhưng VẪN nuốt (khớp "đã xử lý đường convert"). Cache stale của
 * IBus ⇒ phần xoá có thể lệch — chấp nhận theo chính sách RL5/B7 như đường
 * assist (đặt ở comment delete_surrounding_text dùng chung). */
#define TRANSCODE_MAX_CHARS 64000u

static gboolean perform_selection_convert(TextVNIbusEngine *self) {
    IBusEngine *engine = (IBusEngine *)self;
    if (!self || !self->inst) return FALSE;
    if (!(engine->client_capabilities & IBUS_CAP_SURROUNDING_TEXT)) return FALSE;
    /* S3 + E64: ô mật khẩu / terminal — không đọc hay xoá surrounding text trong terminal. */
    if (self->secure || self->terminal) return FALSE;

    /* Chụp MỘT lần surrounding: text/cursor/anchor phải cùng chụp. */
    IBusText *text = NULL;
    guint cursor_pos = 0;
    guint anchor_pos = 0;
    ibus_engine_get_surrounding_text(engine, &text, &cursor_pos, &anchor_pos);
    if (!text) return FALSE;
    /* ibus_text_get_text trả con trỏ NỘI BỘ (không copy, không free) —
     * `text` là REF transfer-full (ibusengine.c: `g_object_ref(priv->...)`)
     * ⇒ MỌI exit path sau get phải `g_object_unref(text)` (trừ khi chưa ref). */
    const gchar *str = ibus_text_get_text(text);
    if (!str) {
        g_object_unref(text);
        return FALSE;
    }

    lc_selection_info info;
    if (!lc_selection_slice(str, cursor_pos, anchor_pos, TRANSCODE_MAX_CHARS, &info)) {
        g_object_unref(text);
        return FALSE; /* không selection / trần / UTF-8 lỗi */
    }
    const char *sel = str + info.start_byte;

    /* FFI ime_convert_selection: UTF-8 vào/ra; cap 2× đầu vào + 32 (khớp
     * CoreBridge Swift — đầu ra không dài hơn đáng kể); 0 = không đổi /
     * không fit. */
    size_t cap = info.byte_len * 2 + 32;
    char *out = g_malloc(cap);
    const int n = ime_convert_selection(self->inst, sel, info.byte_len, out, cap);
    if (n <= 0 || (size_t)n >= cap) {
        g_free(out);
        g_object_unref(text);
        return TRUE; /* không đổi gì — nuốt phím nhưng không ghi document */
    }
    out[n] = '\0';
    if ((size_t)n == info.byte_len && memcmp(out, sel, info.byte_len) == 0) {
        g_free(out);
        g_object_unref(text);
        return TRUE; /* không đổi gì */
    }
    /* Không còn dùng `str`/`sel` — hạ ref TRƯỚC khi sửa document. */
    g_object_unref(text);

    /* Xoá selection: offset từ caret (âm nếu bắt đầu TRƯỚC caret) — parity
     * `delete_surrounding_text` của đường assist; rồi commit bản mới. */
    /* R2-165: vùng bôi đen có thể nằm SAU caret — không gom được; gửi phần đã gom trước
     * (từ vừa chốt) rồi mới xoá + commit (đúng thứ tự cũ). */
    flush_edits(self);
    ibus_engine_delete_surrounding_text(engine, (gint)info.del_offset, (guint)info.char_len);
    ibus_engine_commit_text(engine, ibus_text_new_from_string(out));
    g_free(out);
    /* Document đã đổi: buffer câu engine stale + undo trợ lý không còn đúng
     * (Windows: reset_engine + clear_assist_undo). */
    ime_reset(self->inst);
    self->assist_undo_valid = FALSE;
    return TRUE;
}


/* R2-131 (chủ repo: "Ctrl+Space phải xác định từ lỗi trong câu để sửa hết cùng
 * với text prediction"): bước "sửa hết" của phím tắt trợ lý (Ctrl+Space). Câu có lỗi người dùng sửa được ⇒ sửa cả câu (kể cả chính tả) và
 * trả TRUE; không thì FALSE (engine dự đoán tiếp như Tab). Precondition D12
 * kiểm TRƯỚC khi đụng document: đuôi document = surrounding (cache) + từ còn ở
 * preedit. */
static gboolean hotkey_fix_sentence(TextVNIbusEngine *self) {
    IBusEngine *engine = (IBusEngine *)self;
    if (!self->inst || !s_vi_enabled || self->secure || self->terminal ||
        !lc_assist_is_active(s_assist_enabled ? 1 : 0, self->config.assist_mode) ||
        !(engine->client_capabilities & IBUS_CAP_SURROUNDING_TEXT)) {
        return FALSE;
    }
    ime_assist_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    if (ime_assist_check_instance(self->inst, &res) != IME_OK || res.count == 0) return FALSE;
    if (lc_assist_filter_auto_capitalize(&res, lc_assist_auto_capitalize_enabled(&self->config)) == 0) {
        return FALSE;
    }
    char sentence[256 * 4 + 1];
    size_t actual_len = 0;
    if (ime_assist_sentence_tail(self->inst, 256, sentence, sizeof(sentence), &actual_len) != IME_OK ||
        actual_len == 0 || actual_len >= sizeof(sentence)) {
        return FALSE;
    }
    sentence[actual_len] = '\0';
    lc_sentence_fix_plan plan;
    if (!lc_plan_sentence_fix_ex(self->inst, sentence, &res, 1, &plan) || plan.delete_glyphs == 0) {
        return FALSE;
    }
    char pending[LC_COMP_MAX * 4 + 1];
    pending[0] = '\0';
    if (self->comp.len > 0) lc_utf32_to_utf8(self->comp.text, self->comp.len, pending, sizeof(pending));
    if (!text_before_caret_matches(self, plan.original_tail, pending)) return FALSE;

    /* Precondition khớp: chốt từ đang ở preedit (document = câu engine biết),
     * xoá đúng số glyph rồi gõ lại bản sửa. */
    if (self->comp.len > 0) {
        commit_text(self, self->comp.text, self->comp.len);
        self->comp.len = 0;
    }
    clear_assist_ui(self);
    /* R2-165: chốt preedit + xoá + commit đều đi qua bộ gom ⇒ trên dây chỉ còn
     * delete(phần đã có trong document) + commit(bản sửa) — mutter không đè mất chữ. */
    send_delete_before(self, plan.delete_glyphs);
    send_commit_utf8(self, plan.replacement);

    size_t repl_glyphs = 0;
    if (ime_assist_glyph_len(self->inst, plan.replacement, strlen(plan.replacement), &repl_glyphs) ==
            IME_OK &&
        repl_glyphs > 0) {
        self->assist_undo_valid = TRUE;
        g_strlcpy(self->assist_undo_original_tail, plan.original_tail,
                  sizeof(self->assist_undo_original_tail));
        g_strlcpy(self->assist_undo_replacement, plan.replacement, sizeof(self->assist_undo_replacement));
        self->assist_undo_replacement_glyphs = (uint32_t)repl_glyphs;
        self->assist_undo_timestamp_ms = (guint64)(g_get_monotonic_time() / 1000);
    }
    /* Nạp câu đã sửa làm ngữ cảnh: bấm phím tắt lần nữa ⇒ dự đoán tiếp. */
    char fixed[256 * 4 + LC_MAX_SENTENCE_FIX_CHARS];
    if (lc_fixed_sentence(sentence, plan.original_tail, plan.replacement, fixed, sizeof(fixed))) {
        ime_reset_with_tail(self->inst, fixed, strlen(fixed));
    } else {
        ime_reset(self->inst);
    }
    return TRUE;
}

/* Kiểm tra và thực thi Trợ lý ngữ pháp (AS-060 / P4-6)
 *
 * `committed` = text engine vừa commit trong phím này (UTF-8, rỗng nếu không
 * có) — dùng để dựng trạng thái document khi so precondition (xem
 * `text_before_caret_matches`). */
static gboolean check_and_apply_assist(TextVNIbusEngine *self, uint32_t trigger_char,
                                       gboolean is_sentence_trigger, gboolean plan_eaten,
                                       const char *committed, gboolean allow_display) {
    if (!self || !self->inst || self->secure || self->terminal) return plan_eaten;
    const int dbg = lc_assist_debug_enabled();
    if (dbg) {
        lc_log(LC_LOG_INFO, "IBus", "assist: trigger=%u vi=%d state=%d mode=%d",
               trigger_char, s_vi_enabled, s_assist_enabled, self->config.assist_mode);
    }
    if (!s_vi_enabled || !lc_assist_is_active(s_assist_enabled ? 1 : 0, self->config.assist_mode)) {
        if (dbg) {
            lc_log(LC_LOG_INFO, "IBus", "assist skip: vi=%d state=%d mode=%d",
                   s_vi_enabled, s_assist_enabled, self->config.assist_mode);
        }
        clear_assist_ui(self);
        return plan_eaten;
    }

    ime_assist_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    if (ime_assist_check_instance(self->inst, &res) != IME_OK || res.count == 0) {
        /* R2-129: câu đã đúng ⇒ ẩn danh sách sống ngay. */
        if (dbg) lc_log(LC_LOG_INFO, "IBus", "assist: engine im lặng (count=0)");
        clear_assist_ui(self);
        return plan_eaten;
    }
    if (dbg) lc_log(LC_LOG_INFO, "IBus", "assist: phát hiện %u lỗi", res.count);

    /* D16 (Fassist-20) — Q-6: crate `assist` luôn trả L6 ("chữ thường đầu
     * câu"); khi `config.auto_capitalize` bật thì IME đã tự viết hoa ⇒ báo/sửa
     * L6 là trùng chức năng. Lọc **sau** khi engine đã check (giữ nguyên các
     * luật khác) và **trước** cả Mode 2 lẫn Mode 1 ⇒ cả hai đều không thấy L6.
     * Cùng hàm với Fcitx5 (`lc_assist_filter_auto_capitalize`). */
    if (lc_assist_filter_auto_capitalize(&res, lc_assist_auto_capitalize_enabled(&self->config)) == 0) {
        clear_assist_ui(self);
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

    /* Vòng 32 (audit F-G): parity Windows D11 — câu < 3 từ không popup/không
     * tự sửa (đoạn ngắn đang gõ dở, thiếu ngữ cảnh; Windows chặn ở
     * `check_text` còn 2 nền tảng còn lại phải tự chặn như nhau). Đếm từ
     * trên đuôi đã đọc: byte >= 0x80 là lead/continuation UTF-8 = chữ có dấu. */
    {
        int words = 0;
        gboolean in_word = FALSE;
        for (size_t wi = 0; wi < actual_len; wi++) {
            unsigned char c = (unsigned char)sentence[wi];
            /* A7 (vòng 34): chữ số ĐẾM LÀ TỪ (parity Windows split_whitespace —
             * "học lớp 12 năm 2024" = 4 từ mọi nền tảng). */
            gboolean is_word_c = (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
                                 (c >= '0' && c <= '9') || c >= 0x80;
            if (is_word_c) {
                if (!in_word) {
                    words++;
                    in_word = TRUE;
                }
            } else {
                in_word = FALSE;
            }
        }
        if (words < 3) {
            if (dbg) lc_log(LC_LOG_INFO, "IBus", "assist skip: câu < 3 từ (D11)");
            clear_assist_ui(self);
            return plan_eaten;
        }
    }

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
                    char trig[8];
                    lc_utf32_to_utf8(&trigger_char, 1, trig, sizeof(trig));
                    send_commit_utf8(self, trig);
                }
                if (fix_plan.delete_glyphs > 0) {
                    /* W1-audit: như `apply_assist_candidate` — FFI đã bảo đảm
                       glyph == char Unicode trước khi trả kế hoạch. */
                    send_delete_before(self, fix_plan.delete_glyphs);
                }
                /* R2-165: commit(từ) + commit(trigger) + xoá + commit(bản sửa) gom thành MỘT
                 * delete + MỘT commit ở cuối phím (edits_end). */
                send_commit_utf8(self, fix_plan.replacement);
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
                self->assist_autofixed = TRUE; /* không hẹn timer idle chồng lên */
                return TRUE;
            }
        }
    }

    if (allow_display &&
        lc_assist_should_display(self->config.assist_mode, is_sentence_trigger ? 1 : 0)) {
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
            /* R2-131: chọn gợi ý = hành động CHỦ ĐỘNG ⇒ gồm cả lỗi chính tả. */
            lc_sentence_fix_plan single_plan;
            if (!lc_plan_sentence_fix_ex(self->inst, sentence, &single, 1, &single_plan)) {
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
        } else {
            clear_assist_ui(self); /* không còn gợi ý áp được ⇒ không để danh sách cũ */
        }
    }

    return plan_eaten;
}

/* R2-155: timer idle — dừng gõ đủ LC_ASSIST_IDLE_MS thì kiểm tra câu rồi hiện danh sách.
 * Chỉ khi không còn preedit mở: áp gợi ý xoá text đã commit bằng surrounding text, còn từ
 * đang gõ dở nằm trong preedit — chốt giữa chừng sẽ cắt đôi từ (Windows có đường riêng). */
static void assist_idle_cancel(TextVNIbusEngine *self) {
    if (self->assist_idle_id != 0) {
        g_source_remove(self->assist_idle_id);
        self->assist_idle_id = 0;
    }
}

static gboolean assist_idle_fire(gpointer data) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)data;
    self->assist_idle_id = 0;
    if (self->inst && !self->secure && !self->terminal && s_vi_enabled && self->comp.len == 0) {
        edits_begin(self); /* R2-165 */
        (void)check_and_apply_assist(self, 0, FALSE, FALSE, "", TRUE);
        edits_end(self);
    }
    return G_SOURCE_REMOVE;
}

static void assist_idle_arm(TextVNIbusEngine *self) {
    assist_idle_cancel(self);
    if (!self->inst || self->secure || self->terminal || !s_vi_enabled ||
        !lc_assist_is_active(s_assist_enabled ? 1 : 0, self->config.assist_mode)) {
        return;
    }
    self->assist_idle_id = g_timeout_add(LC_ASSIST_IDLE_MS, assist_idle_fire, self);
}

static void push_context(TextVNIbusEngine *self, int enabled) {
    if (!self->inst) return;
    ime_context_v1 ctx;
    memset(&ctx, 0, sizeof(ctx));
    ctx.abi_version = IME_ABI_VERSION;
    ctx.enabled = enabled ? 1 : 0;
    ctx.secure = 0;
    /* Terminal: engine không dự đoán/học (R2-144/R2-145); hint giữ nguyên cách xuất chữ. */
    ctx.field_role = self->terminal ? IME_FIELD_TERMINAL : IME_FIELD_BODY;
    ctx.caps = IME_CAP_PREEDIT | IME_CAP_SELECTION;
    ctx.hint = self->terminal ? (int64_t)IME_STRATEGY_PREEDIT : -1;
    ime_set_context(self->inst, &ctx);
    self->ctx_enabled = enabled ? 1 : 0;
}

/* Mở bảng điều khiển (textvn-settings cạnh bản cài / trong PATH), tùy chọn truyền cờ (vd --diagnose). */
static void launch_settings_arg(const gchar *extra_arg) {
    gchar *exe = g_file_read_link("/proc/self/exe", NULL);
    gchar *dir = exe ? g_path_get_dirname(exe) : NULL;
    char path[1024];
    if (lc_find_settings_binary(path, sizeof(path), dir) == 0) {
        gchar *argv[3] = {path, (gchar *)extra_arg, NULL};
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
    lc_session_shadow_init(&self->shadow);
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
    self->diag_prop = ibus_property_new(PROP_DIAG, PROP_TYPE_NORMAL,
                                        ibus_text_new_from_static_string("Chẩn đoán & xuất log…"),
                                        "dialog-information",
                                        ibus_text_new_from_static_string(
                                            "Kiểm tra lỗi, xung đột và xuất gói log chẩn đoán"),
                                        TRUE, TRUE, PROP_STATE_UNCHECKED, NULL);
    g_object_ref_sink(self->diag_prop);
    ibus_prop_list_append(self->props, self->diag_prop);

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
    assist_idle_cancel(self);
    clear_assist_ui(self);
    if (self->lookup_table) {
        g_clear_object(&self->lookup_table);
    }
    if (self->inst) {
        ime_instance_free(self->inst);
        self->inst = NULL;
    }
    lc_session_shadow_free(&self->shadow);
    g_clear_object(&self->mode_prop);
    g_clear_object(&self->setup_prop);
    g_clear_object(&self->diag_prop);
    g_clear_object(&self->props);
    G_OBJECT_CLASS(textvn_ibus_engine_parent_class)->finalize(object);
}

/* ---- IBusEngine vfuncs ---- */

/* Quên từ đang gõ mà KHÔNG commit: preedit (PREEDIT_COMMIT) do ibus-daemon commit khi
 * client reset/mất focus — engine commit thêm sẽ thành chữ lặp (kiểm chứng e2e). */
static void forget_word(TextVNIbusEngine *self) {
    assist_idle_cancel(self);
    clear_assist_ui(self);
    hide_emoji_ui(self);
    self->assist_undo_valid = FALSE;
    self->comp.len = 0;
    if (self->inst) ime_reset(self->inst);
}

/* E67: truyền con trỏ hợp lệ thay vì NULL để tránh g_return_if_fail trên IBus 1.5.19 (RHEL 8.10). */
static void request_surrounding_text(IBusEngine *engine) {
    IBusText *text = NULL;
    guint cursor_pos = 0;
    guint anchor_pos = 0;
    ibus_engine_get_surrounding_text(engine, &text, &cursor_pos, &anchor_pos);
    if (text) g_object_unref(text);
}

static void engine_enable(IBusEngine *engine) {
    /* Đăng ký yêu cầu surrounding text với ibus-daemon và client application */
    request_surrounding_text(engine);
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
        /* Vòng 32 (audit F-B, R2-28): TOÀN CỤC AND override — khớp Windows
         * `effective_enabled`: override chỉ TẮT riêng được app, không "hồi
         * sinh" app khi toàn cục đang EN (kill-switch là master). */
        s_vi_enabled = s_base_vi && (app_enabled != 0);
    } else {
        /* C4 (audit): override da bi clear (app khong co trong snapshot) —
         * phai ve ve BASE (state.json/broadcast "*"), khong de ke thừa EN cua
         * app truoc. */
        s_vi_enabled = s_base_vi;
    }
    request_surrounding_text(engine);
    ibus_engine_register_properties(engine, self->props);
    update_mode_prop(self);
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->focus_in(engine);
}

static void engine_focus_out(IBusEngine *engine) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    self->enter_at_us = 0; /* R2-171: đổi focus ⇒ ngữ cảnh mới */
    forget_word(self);
    lc_modifier_toggle_reset(&self->toggle);
    lc_session_shadow_reset(&self->shadow);
    lc_style_persist(self->inst);
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->focus_out(engine);
}

static void engine_reset(IBusEngine *engine) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    /* R2-171: reset GtkTextView gửi ngay sau Enter ⇒ giữ mốc đầu dòng (tự viết hoa). */
    const gboolean line_start =
        self->enter_at_us != 0 &&
        g_get_monotonic_time() - self->enter_at_us < LC_RESET_AFTER_ENTER_US;
    forget_word(self);
    if (line_start && self->inst) ime_reset_with_tail(self->inst, LC_LINE_START_TAIL, 1);
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->reset(engine);
}

static void engine_disable(IBusEngine *engine) {
    assist_idle_cancel((TextVNIbusEngine *)engine);
    finish_word((TextVNIbusEngine *)engine);
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->disable(engine);
}

static void engine_set_content_type(IBusEngine *engine, guint purpose, guint hints) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    gboolean secure = purpose == IBUS_INPUT_PURPOSE_PASSWORD || purpose == IBUS_INPUT_PURPOSE_PIN;
    if (secure && !self->secure) finish_word(self);
    self->secure = secure;
    /* IBUS_INPUT_PURPOSE_TERMINAL (= GTK_INPUT_PURPOSE_TERMINAL) — header cũ chưa có tên. */
    gboolean terminal = purpose == 10;
    if (terminal != self->terminal) {
        self->terminal = terminal;
        if (terminal) {
            assist_idle_cancel(self);
            clear_assist_ui(self);
            hide_emoji_ui(self);
            hide_preview(self);
        }
        push_context(self, s_vi_enabled);
    }
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->set_content_type(engine, purpose, hints);
}

static void engine_property_activate(IBusEngine *engine, const gchar *name, guint state) {
    if (g_strcmp0(name, PROP_MODE) == 0) {
        toggle_vietnamese((TextVNIbusEngine *)engine);
        return;
    }
    if (g_strcmp0(name, PROP_SETUP) == 0) {
        launch_settings_arg(NULL);
        return;
    }
    if (g_strcmp0(name, PROP_DIAG) == 0) {
        launch_settings_arg("--diagnose");
        return;
    }
    IBUS_ENGINE_CLASS(textvn_ibus_engine_parent_class)->property_activate(engine, name, state);
}

static void candidate_clicked_inner(IBusEngine *engine, guint index, guint button, guint state) {
    (void)button;
    (void)state;
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    if (self->emoji.active) {
        size_t page_start = (self->emoji.selected / LC_EMOJI_PAGE_SIZE) * LC_EMOJI_PAGE_SIZE;
        char glyph[LC_EMOJI_GLYPH_MAX];
        if (lc_emoji_pick_index(&self->emoji, self->inst, page_start + index, glyph, sizeof(glyph))) {
            if (self->lookup_table) {
                ibus_lookup_table_clear(self->lookup_table);
                ibus_engine_hide_lookup_table(engine);
            }
            ibus_engine_hide_auxiliary_text(engine);
            send_commit_utf8(self, glyph);
        }
        return;
    }
    if (self->assist_ui_showing && index < self->assist_candidate_count) {
        apply_assist_candidate(self, index);
    }
}

static void handle_ctrl_tap(TextVNIbusEngine *self) {
    if (!self || !self->inst || self->comp.len == 0) return;
    ime_result_v1 r;
    if (lc_engine_ctrl_tap(self->inst, &r) != IME_OK) return;
    lc_plan plan;
    lc_plan_key(&self->comp, (lc_key){ LC_KEY_OTHER, 0 }, &r, &plan);
    if (plan.end) {
        size_t n = 0;
        const uint32_t *t = lc_plan_commit_text(&self->comp, &plan, &n);
        commit_text(self, t, n);
    } else if (plan.has_text) {
        show_preedit(self, plan.text, plan.text_len);
    }
    lc_comp_apply(&self->comp, &plan);
    if (plan.reset_engine) ime_reset(self->inst);
    update_preview(self);
}

static gboolean handle_release(TextVNIbusEngine *self, guint keyval) {
    lc_modifier which = textvn_ibus_modifier_kind(keyval);
    int ctrl_tap = lc_modifier_ctrl_tap_up(&self->toggle, which);
    int toggled = !ctrl_tap && lc_modifier_toggle_up(&self->toggle, which);
    lc_session_shadow_key_up(&self->shadow, which, toggled);
    if (ctrl_tap) {
        handle_ctrl_tap(self);
    } else if (toggled) {
        toggle_vietnamese(self);
    }
    return FALSE; /* không bao giờ nuốt phím nhả */
}

static gboolean process_key_inner(IBusEngine *engine, guint keyval, guint keycode,
                                         guint state) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    (void)keycode;

    if (state & IBUS_RELEASE_MASK) return handle_release(self, keyval);

    lc_modifier which = textvn_ibus_modifier_kind(keyval);
    const int ctrl_down = (state & IBUS_CONTROL_MASK) != 0;
    const int shift_down = (state & IBUS_SHIFT_MASK) != 0;
    const int alt_or_super = (state & (IBUS_MOD1_MASK | IBUS_SUPER_MASK | IBUS_MOD4_MASK)) != 0;
    lc_modifier_toggle_down(&self->toggle, which, ctrl_down, shift_down, alt_or_super);
    if (self->shadow.session) {
        /* Cùng điều kiện nhánh Ctrl+Shift+Space bên dưới (ADR-011). */
        const int is_space = keyval == IBUS_KEY_space || keyval == IBUS_KEY_KP_Space;
        lc_session_shadow_key_down(&self->shadow, which, is_space, ctrl_down, shift_down,
                                   alt_or_super,
                                   which == LC_MOD_NONE && is_space && ctrl_down && shift_down);
    }
    if (which != LC_MOD_NONE) return FALSE; /* modifier đứng riêng: không commit */
    /* R2-134: bản xem trước chỉ sống tới phím kế tiếp (vẽ lại sau phím gõ). */
    hide_preview(self);

    /* Phím khi danh sách gợi ý SỐNG đang hiện (R2-129 — parity Windows): không
     * chiếm phím gõ; Esc đóng; Alt+1..3 áp gợi ý (phím số trần là gõ số — trước
     * đây "giá 100" áp nhầm gợi ý); Enter/Tab/mũi tên/chord đóng rồi đi tiếp. */
    if (self->assist_ui_showing) {
        int live = lc_assist_live_key((uint32_t)keyval, (state & IBUS_CONTROL_MASK) != 0,
                                      (state & IBUS_MOD1_MASK) != 0,
                                      (state & (IBUS_SUPER_MASK | IBUS_MOD4_MASK)) != 0);
        if (live == LC_LIVE_CLOSE) {
            clear_assist_ui(self);
            return TRUE;
        }
        if (live >= LC_LIVE_APPLY_1) {
            guint idx = (guint)(live - LC_LIVE_APPLY_1);
            if (idx < self->assist_candidate_count) {
                apply_assist_candidate(self, idx);
                return TRUE;
            }
            live = LC_LIVE_DISMISS;
        }
        if (live == LC_LIVE_DISMISS) clear_assist_ui(self);
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
        lc_modifier_toggle_reset(&self->toggle);
        toggle_vietnamese(self);
        return TRUE;
    }

    /* P12: Ctrl+Alt+E mở/đóng Bảng Emoji nhẹ (X11 + Wayland qua IBusLookupTable);
     * tắt trong terminal / ô mật khẩu (E64). */
    if (!self->secure && !self->terminal &&
        lc_is_emoji_hotkey((uint32_t)keyval, (state & IBUS_CONTROL_MASK) != 0,
                           (state & IBUS_MOD1_MASK) != 0,
                           (state & IBUS_SHIFT_MASK) != 0,
                           (state & (IBUS_SUPER_MASK | IBUS_MOD4_MASK)) != 0)) {
        lc_modifier_toggle_reset(&self->toggle);
        if (self->comp.len > 0) finish_word(self);
        clear_assist_ui(self);
        if (self->emoji.active) {
            hide_emoji_ui(self);
        } else {
            lc_emoji_open(&self->emoji, self->inst);
            render_emoji_ui(self);
        }
        return TRUE;
    }
    if (self->emoji.active) {
        if (mods & (IME_MOD_CTRL | IME_MOD_ALT | IME_MOD_SUPER | IME_MOD_META)) {
            hide_emoji_ui(self);
            return FALSE;
        }
        uint32_t evk = 0, ech = 0;
        textvn_ibus_map_key(keyval, &evk, &ech);
        if (keyval == IBUS_KEY_Left) evk = 0x25u;
        else if (keyval == IBUS_KEY_Up) evk = 0x26u;
        else if (keyval == IBUS_KEY_Right) evk = 0x27u;
        else if (keyval == IBUS_KEY_Down) evk = 0x28u;
        char glyph[LC_EMOJI_GLYPH_MAX];
        int res = lc_emoji_handle_key(&self->emoji, self->inst, evk, ech,
                                      (state & IBUS_SHIFT_MASK) != 0 ? 1 : 0,
                                      glyph, sizeof(glyph));
        if (res == LC_EMOJI_COMMIT) {
            if (self->lookup_table) {
                ibus_lookup_table_clear(self->lookup_table);
                ibus_engine_hide_lookup_table(engine);
            }
            ibus_engine_hide_auxiliary_text(engine);
            send_commit_utf8(self, glyph);
        } else if (res == LC_EMOJI_CLOSED) {
            if (self->lookup_table) {
                ibus_lookup_table_clear(self->lookup_table);
                ibus_engine_hide_lookup_table(engine);
            }
            ibus_engine_hide_auxiliary_text(engine);
        } else {
            render_emoji_ui(self);
        }
        return TRUE;
    }

    /* OP-007 (P5 §7.4) · R2-139: Ctrl+Shift+F9 — chuyển mã vùng bôi đen.
     * Chạy TRƯỚC nhánh chord hệ thống (parity Windows key_event.rs §613-638);
     * chốt preedit trước khi thay đoạn text (như `end_composition`). Chỉ nuốt
     * phím khi đã chạy đường convert tới cùng; còn lại F9 tới app. */
    if (lc_is_transcode_hotkey((uint32_t)keyval, (state & IBUS_CONTROL_MASK) != 0,
                               (state & IBUS_MOD1_MASK) != 0,
                               (state & IBUS_SHIFT_MASK) != 0,
                               (state & (IBUS_SUPER_MASK | IBUS_MOD4_MASK)) != 0)) {
        if (self->comp.len > 0) finish_word(self);
        if (perform_selection_convert(self)) return TRUE;
        return FALSE;
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
     * commit, phím đi thẳng. Ngoại lệ: Ctrl+Space được giữ lại cho Text Prediction & Text Correct. */
    uint32_t vk = 0, ch = 0;
    textvn_ibus_map_key(keyval, &vk, &ch);
    /* R2-131/R2-136: phím tắt trợ lý = Ctrl+Space (Alt+Space không thuộc TextVN). */
    gboolean is_ctrl_space = lc_is_assist_hotkey((uint32_t)keyval, (state & IBUS_CONTROL_MASK) != 0,
                                                 (state & IBUS_MOD1_MASK) != 0,
                                                 (state & IBUS_SHIFT_MASK) != 0,
                                                 (state & (IBUS_SUPER_MASK | IBUS_MOD4_MASK)) != 0) != 0;

    /* Terminal: Mode EN đưa phím thẳng tới app (vim, less), trừ khi Math hoặc macro_when_off bật. */
    gboolean passthrough_on = !s_vi_enabled && (self->config.allow_macro_when_vi_off ||
                                                self->config.math_enabled ||
                                                (self->config.suggest_enabled && !self->terminal));
    if (((mods & (IME_MOD_CTRL | IME_MOD_ALT | IME_MOD_SUPER | IME_MOD_META)) && !is_ctrl_space) || self->secure ||
        !(s_vi_enabled || passthrough_on) || !self->inst) {
        finish_word(self);
        return FALSE;
    }
    if (self->ctx_enabled != (s_vi_enabled ? 1 : 0)) push_context(self, s_vi_enabled);

    /* R2-131: phím tắt — câu có lỗi sửa được ⇒ sửa hết TRƯỚC; không ⇒ dự đoán. */
    if (is_ctrl_space && hotkey_fix_sentence(self)) return TRUE;
    if (is_ctrl_space) {
        vk = 0x20;
        ch = 0x20;
        mods = (mods & IME_MOD_CAPS) | IME_MOD_CTRL;
    }

    lc_key key = lc_key_classify_with_mods(vk, ch, mods);
    ime_key_v1 k;
    uint32_t ime_mods = mods & (IME_MOD_SHIFT | IME_MOD_CAPS);
    if (is_ctrl_space) {
        ime_mods |= IME_MOD_CTRL;
    }
    lc_key_to_ime(key, vk, ime_mods, &k);

    ime_result_v1 r;
    if (ime_key(self->inst, &k, &r) != IME_OK || (r.flags & IME_FLAG_ERROR)) {
        finish_word(self);
        return FALSE;
    }

    lc_plan plan;
    lc_plan_key(&self->comp, key, &r, &plan);

    if (plan.delete_before > 0) {
        /* Chỉ xóa text đã commit khi client thật sự hỗ trợ surrounding text và KHÔNG phải terminal (E64). */
        if (self->terminal || !(engine->client_capabilities & IBUS_CAP_SURROUNDING_TEXT)) {
            finish_word(self);
            return FALSE;
        }
        send_delete_before(self, plan.delete_before);
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
        if (plan.cursor_left > 0) {
            /* R2-166: commit đã gom phải tới app TRƯỚC các phím Left. Mỗi phím gửi cả nhấn
             * LẪN nhả, kèm keycode evdev thật: chỉ gửi nhấn thì GNOME (thiết bị ảo của
             * gnome-shell) coi Left còn đang giữ — app tự lặp phím, và lần gửi sau bị bỏ vì
             * "phím đang giữ" (ngosen#1020); keycode 0 thì mutter không dựng được wl_keyboard.key. */
            flush_edits(self);
            uint16_t steps = plan.cursor_left > 64 ? 64 : plan.cursor_left;
            for (uint16_t i = 0; i < steps; i++) {
                ibus_engine_forward_key_event(engine, IBUS_KEY_Left, LC_EVDEV_KEY_LEFT, 0);
                ibus_engine_forward_key_event(engine, IBUS_KEY_Left, LC_EVDEV_KEY_LEFT,
                                              IBUS_RELEASE_MASK);
            }
        }
    } else if (plan.has_text) {
        show_preedit(self, plan.text, plan.text_len);
    }
    lc_comp_apply(&self->comp, &plan);
    if (plan.reset_engine) ime_reset(self->inst);

    /* Trợ lý ngữ pháp (AS-060 / P4-6): trigger kiểm tra khi commit kết thúc câu hoặc dấu cách;
     * tuyệt đối không chạy trong terminal (E64). */
    uint32_t trigger_kind = (key.kind == LC_KEY_CHAR) ? ime_assist_trigger_kind(key.ch) : IME_ASSIST_TRIGGER_NONE;
    if (!self->terminal && trigger_kind != IME_ASSIST_TRIGGER_NONE && s_vi_enabled &&
        self->comp.len == 0 && self->inst) {
        gboolean is_sentence = (trigger_kind == IME_ASSIST_TRIGGER_SENTENCE);
        /* R2-155: phím trigger chỉ kiểm tra + tự sửa (Mode 2); danh sách hiện khi dừng gõ. */
        self->assist_autofixed = FALSE;
        gboolean eaten = check_and_apply_assist(self, key.ch, is_sentence, plan.eaten ? TRUE : FALSE,
                                                committed, FALSE);
        if (self->assist_autofixed) {
            self->assist_autofixed = FALSE;
            assist_idle_cancel(self);
        } else {
            clear_assist_ui(self); /* gõ tiếp là ẩn ngay; timer mới hiện lại sau 1 s */
            assist_idle_arm(self);
        }
        update_preview(self);
        return eaten;
    }
    /* R2-155: mọi phím gõ chữ/xoá hẹn lại timer idle (danh sách đang hiện đã bị ẩn ở đầu hàm). */
    if (!self->terminal &&
        lc_assist_idle_arms(key.kind == LC_KEY_CHAR, key.kind == LC_KEY_BACKSPACE, s_vi_enabled)) {
        clear_assist_ui(self); /* phím số/chữ trần không đóng danh sách sống — ẩn ở đây */
        assist_idle_arm(self);
    }

    if (key.kind == LC_KEY_CHAR || key.kind == LC_KEY_BACKSPACE || key.kind == LC_KEY_TAB ||
        key.kind == LC_KEY_CTRL_SPACE) {
        update_preview(self);
    }
    return plan.eaten ? TRUE : FALSE;
}

/* R2-165: mọi chỉnh sửa document trong MỘT phím gom lại, gửi tối đa 1 xoá + 1 commit
 * TRƯỚC khi trả lời phím (app nhận phím không nuốt SAU phần đã commit — như cũ). */
static gboolean engine_process_key_event(IBusEngine *engine, guint keyval, guint keycode,
                                         guint state) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    if (!(state & IBUS_RELEASE_MASK)) {
        /* R2-171: nhớ lúc nhấn Enter; phím khác xen giữa ⇒ reset sau đó không phải của Enter. */
        self->enter_at_us = (keyval == IBUS_KEY_Return || keyval == IBUS_KEY_KP_Enter ||
                             keyval == IBUS_KEY_ISO_Enter)
                                ? g_get_monotonic_time()
                                : 0;
    }
    edits_begin(self);
    gboolean eaten = process_key_inner(engine, keyval, keycode, state);
    edits_end(self);
    return eaten;
}

static void engine_candidate_clicked(IBusEngine *engine, guint index, guint button, guint state) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    edits_begin(self);
    candidate_clicked_inner(engine, index, button, state);
    edits_end(self);
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
