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

#include <string.h>

G_DEFINE_TYPE(TextVNIbusEngine, textvn_ibus_engine, IBUS_TYPE_ENGINE)

/* Trạng thái VN/EN dùng chung cho mọi input context của process (như UniKey). */
static gboolean        s_vi_enabled = TRUE;
static gboolean        s_state_loaded = FALSE;
static lc_config_state s_state_watch;
static lc_ipc_client  *s_ipc = NULL;
static uint32_t       s_global_seq_seen = 0; /* BUG-03: seq "*" da tieu thu */

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

/* Commit preedit hiện tại nguyên văn + reset engine (focus/chord/secure/toggle). */
static void finish_word(TextVNIbusEngine *self) {
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
        set_vietnamese(self, enabled ? TRUE : FALSE, FALSE);
    }
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
}

static void textvn_ibus_engine_finalize(GObject *object) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)object;
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
    self->comp.len = 0;
    if (self->inst) ime_reset(self->inst);
}

static void engine_focus_in(IBusEngine *engine) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    lc_config_sync(self->inst, &self->config, NULL);
    sync_state(self);
    consume_global_ipc();
    int app_enabled = 1;
    if (s_ipc && lc_ipc_client_get_app_override(s_ipc, &app_enabled)) {
        s_vi_enabled = app_enabled != 0;
    }
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

    uint32_t mods = textvn_ibus_mods(state);

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

    if (plan.end) {
        size_t n = 0;
        const uint32_t *t = lc_plan_commit_text(&self->comp, &plan, &n);
        commit_text(self, t, n);
    } else if (plan.has_text) {
        show_preedit(self, plan.text, plan.text_len);
    }
    lc_comp_apply(&self->comp, &plan);
    if (plan.reset_engine) ime_reset(self->inst);
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
    engine_class->disable = engine_disable;
    engine_class->set_content_type = engine_set_content_type;
    engine_class->property_activate = engine_property_activate;
}
