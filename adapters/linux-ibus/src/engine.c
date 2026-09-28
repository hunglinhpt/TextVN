/* engine.c — IBusEngine của TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Mô hình: cả từ đang gõ nằm trong PREEDIT (lc_compose.h). Mỗi phím:
 *   modifier → chỉ theo dõi Ctrl+Shift | hotkey → đảo VN/EN
 *   chord/secure/tắt VN → commit preedit nguyên văn, phím đi thẳng (B6, S3)
 *   còn lại → ime_key → lc_plan_key → cập nhật/commit preedit → eaten?
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
static gboolean       s_vi_enabled = TRUE;
static lc_ipc_client *s_ipc = NULL;

#define PROP_MODE "TextVN.InputMode"

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
    ibus_text_append_attribute(t, IBUS_ATTR_TYPE_UNDERLINE, IBUS_ATTR_UNDERLINE_SINGLE, 0,
                               (gint)len);
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
    ibus_engine_update_property((IBusEngine *)self, self->mode_prop);
}

static void toggle_vietnamese(TextVNIbusEngine *self) {
    finish_word(self);
    s_vi_enabled = !s_vi_enabled;
    if (s_ipc) lc_ipc_client_toggle_vi_en(s_ipc, "*", s_vi_enabled ? 1 : 0);
    update_mode_prop(self);
}

static void push_context(TextVNIbusEngine *self) {
    if (!self->inst) return;
    ime_context_v1 ctx;
    memset(&ctx, 0, sizeof(ctx));
    ctx.abi_version = IME_ABI_VERSION;
    ctx.enabled = 1;
    ctx.secure = 0;
    ctx.field_role = IME_FIELD_BODY;
    ctx.caps = IME_CAP_PREEDIT | IME_CAP_SELECTION;
    ctx.hint = -1;
    ime_set_context(self->inst, &ctx);
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
    push_context(self);
    if (!s_ipc) s_ipc = lc_ipc_client_new("ibus", NULL);

    self->props = ibus_prop_list_new();
    g_object_ref_sink(self->props);
    self->mode_prop = ibus_property_new(PROP_MODE, PROP_TYPE_NORMAL,
                                        ibus_text_new_from_static_string("VN"), "textvn_v",
                                        ibus_text_new_from_static_string(
                                            "Bật/tắt tiếng Việt (Ctrl+Shift)"),
                                        TRUE, TRUE, PROP_STATE_UNCHECKED, NULL);
    g_object_ref_sink(self->mode_prop);
    ibus_prop_list_append(self->props, self->mode_prop);
}

static void textvn_ibus_engine_finalize(GObject *object) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)object;
    if (self->inst) {
        ime_instance_free(self->inst);
        self->inst = NULL;
    }
    g_clear_object(&self->mode_prop);
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
        toggle_vietnamese(self);
        return TRUE;
    }

    /* B6: chord hệ thống; S3: ô mật khẩu; VN tắt; engine hỏng → commit, phím đi thẳng. */
    if ((mods & (IME_MOD_CTRL | IME_MOD_ALT | IME_MOD_SUPER | IME_MOD_META)) || self->secure ||
        !s_vi_enabled || !self->inst) {
        finish_word(self);
        return FALSE;
    }

    if (self->comp.len == 0) lc_config_sync(self->inst, &self->config, NULL);

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
