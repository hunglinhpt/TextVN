/* engine.c — IBusEngine subclass implementation for TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Implements:
 * - 1 instance per engine context (no memory leak B13)
 * - Commit-before-hide (Bug B2)
 * - Surrounding text synchronization and self-healing
 * - Hotkey toggle Ctrl+Shift+Space (ADR-011)
 * - System shortcuts passthrough (Bug B6)
 * - Password / secure field passthrough (S8)
 */

#include "engine.h"
#include "keymap.h"
#include "apply.h"
#include <string.h>

#ifndef IBUS_KEY_Return
#define IBUS_KEY_Return   0xff0d
#define IBUS_KEY_KP_Enter 0xff8d
#define IBUS_KEY_space    0x0020
#define IBUS_KEY_KP_Space 0xff80
#endif

G_DEFINE_TYPE(TextVNIbusEngine, textvn_ibus_engine, IBUS_TYPE_ENGINE)

static void textvn_ibus_engine_init(TextVNIbusEngine *self) {
    self->inst = NULL;
    self->ipc_client = lc_ipc_client_new("ibus", NULL);
    self->vi_enabled = TRUE;
    self->non_preedit = TRUE;
    self->has_surrounding = FALSE;
    self->field_role = IME_FIELD_UNKNOWN;
    self->secure = 0;
    self->strategy_hint = -1;
    self->cursor_pos = 0;

    int32_t rc = ime_instance_new(NULL, 0, &self->inst);
    if (rc != IME_OK && rc != IME_ERR_CONFIG) {
        lc_log(LC_LOG_ERROR, "IBus", "Failed to create ime_instance: rc=%d", (int)rc);
        self->inst = NULL;
    }
}

static void textvn_ibus_engine_finalize(GObject *object) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)object;

    if (self->inst) {
        ime_instance_free(self->inst);
        self->inst = NULL;
    }

    if (self->ipc_client) {
        lc_ipc_client_free(self->ipc_client);
        self->ipc_client = NULL;
    }

    G_OBJECT_CLASS(textvn_ibus_engine_parent_class)->finalize(object);
}

void textvn_ibus_engine_focus_in(IBusEngine *engine) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    if (!self) return;

    if (!self->inst) {
        ime_instance_new(NULL, 0, &self->inst);
    }

    /* Check config reload from IPC */
    uint64_t new_config_ver = 0;
    if (self->ipc_client && lc_ipc_client_check_config_reload(self->ipc_client, &new_config_ver)) {
        if (self->inst) ime_reload_config(self->inst, NULL, 0);
    }

    /* Check app override from IPC */
    int app_en = 0;
    if (self->ipc_client && lc_ipc_client_get_app_override(self->ipc_client, &app_en)) {
        self->vi_enabled = (app_en != 0);
    }

    /* Probe surrounding text capability */
    IBusText *surr_text = NULL;
    guint cur = 0, anc = 0;
    ibus_engine_get_surrounding_text(engine, &surr_text, &cur, &anc);
    self->has_surrounding = (surr_text != NULL);
    self->cursor_pos = cur;

    /* Detect field role via AT-SPI or heuristics */
    lc_field_ctx fctx;
    lc_field_detect(&fctx);

    self->field_role = fctx.field_role;
    self->secure = fctx.secure;
    self->strategy_hint = fctx.strategy_hint;

    /* Configure context in core engine */
    if (self->inst) {
        ime_context_v1 ctx;
        memset(&ctx, 0, sizeof(ctx));
        ctx.abi_version = IME_ABI_VERSION;
        ctx.enabled = self->vi_enabled ? 1 : 0;
        ctx.secure = self->secure;
        ctx.field_role = self->field_role;
        ctx.caps = IME_CAP_PREEDIT | IME_CAP_FIELD_DETECT | IME_CAP_SELECTION;
        ctx.app_id = fctx.app_id[0] ? fctx.app_id : "ibus-app";
        ctx.hint = self->strategy_hint;

        ime_set_context(self->inst, &ctx);
    }
}

void textvn_ibus_engine_focus_out(IBusEngine *engine) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    if (!self) return;

    /* Bug B2 & Bug B13: Commit-before-hide */
    apply_ibus_commit(self);
    apply_ibus_reset_preedit(self);

    if (self->inst) {
        ime_reset(self->inst);
    }
}

void textvn_ibus_engine_enable(IBusEngine *engine) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    if (!self) return;
    self->vi_enabled = TRUE;
}

void textvn_ibus_engine_disable(IBusEngine *engine) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    if (!self) return;

    /* Commit-before-hide */
    apply_ibus_commit(self);
    apply_ibus_reset_preedit(self);

    if (self->inst) {
        ime_reset(self->inst);
    }
    self->vi_enabled = FALSE;
}

void textvn_ibus_engine_reset(IBusEngine *engine) {
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    if (!self) return;

    apply_ibus_reset_preedit(self);
    if (self->inst) {
        ime_reset(self->inst);
    }
}

void textvn_ibus_engine_set_surrounding_text(IBusEngine *engine,
                                             IBusText   *text,
                                             guint       cursor_pos,
                                             guint       anchor_pos)
{
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    if (!self) return;
    (void)anchor_pos;

    self->has_surrounding = (text != NULL);

    /* Self-healing: if cursor jumped unexpectedly, reset composition buffer */
    if (cursor_pos != self->cursor_pos) {
        self->cursor_pos = cursor_pos;
        if (self->inst) {
            ime_reset(self->inst);
        }
        apply_ibus_reset_preedit(self);
    }
}

gboolean textvn_ibus_engine_process_key_event(IBusEngine *engine,
                                              guint       keyval,
                                              guint       keycode,
                                              guint       state)
{
    TextVNIbusEngine *self = (TextVNIbusEngine *)engine;
    if (!self) return FALSE;

    /* Ignore key release events */
    if (state & IBUS_RELEASE_MASK) {
        return FALSE;
    }

    /* Hotkey: Ctrl+Shift+Space to toggle Vietnamese/English (ADR-011) */
    if ((state & IBUS_CONTROL_MASK) &&
        (state & IBUS_SHIFT_MASK)   &&
        (keyval == IBUS_KEY_space || keyval == IBUS_KEY_KP_Space))
    {
        self->vi_enabled = !self->vi_enabled;
        if (self->ipc_client) {
            lc_ipc_client_toggle_vi_en(self->ipc_client, "ibus-app", self->vi_enabled ? 1 : 0);
        }
        if (self->inst) {
            ime_reset(self->inst);
        }
        apply_ibus_reset_preedit(self);
        return TRUE;
    }

    /* Bug B6: System shortcuts & hotkeys (Ctrl/Alt/Super combinations)
     * Must pass through directly to avoid breaking Ctrl+C, Ctrl+V, Alt+Tab, etc. */
    if (state & (IBUS_CONTROL_MASK | IBUS_MOD1_MASK | IBUS_SUPER_MASK | IBUS_MOD4_MASK)) {
        if (self->inst) {
            ime_reset(self->inst);
        }
        apply_ibus_reset_preedit(self);
        return FALSE; /* Let application handle hotkey */
    }

    /* Bug B2: Enter key handling in chat applications (Slack, Discord, Telegram, Zalo)
     * If user presses Enter while composing, commit word immediately and pass Enter through. */
    if (keyval == IBUS_KEY_Return || keyval == IBUS_KEY_KP_Enter) {
        if (self->inst) {
            ime_reset(self->inst);
        }
        apply_ibus_reset_preedit(self);
        return FALSE; /* Pass Enter directly to application to submit message */
    }

    /* S8: Password / secure field passthrough */
    if (self->secure) {
        return FALSE;
    }

    /* If Vietnamese mode is disabled, pass key through */
    if (!self->vi_enabled || !self->inst) {
        return FALSE;
    }

    /* Map IBus key to standardized ime_key_v1 */
    ime_key_v1 k;
    if (!ibus_keyval_to_ime_key(keyval, keycode, state, &k)) {
        return FALSE;
    }

    /* Call core Rust engine via C-ABI */
    ime_result_v1 res;
    int32_t rc = ime_key(self->inst, &k, &res);
    if (rc != IME_OK || (res.flags & IME_FLAG_ERROR)) {
        return FALSE; /* Fail-open */
    }

    if (res.action == IME_ACTION_PASS) {
        return FALSE;
    }

    apply_ibus_result(self, &res);
    return TRUE;
}

static void textvn_ibus_engine_class_init(TextVNIbusEngineClass *klass) {
    GObjectClass *gobject_class = G_OBJECT_CLASS(klass);
    gobject_class->finalize = textvn_ibus_engine_finalize;

    IBusEngineClass *engine_class = IBUS_ENGINE_CLASS(klass);
    engine_class->process_key_event = textvn_ibus_engine_process_key_event;
    engine_class->focus_in = textvn_ibus_engine_focus_in;
    engine_class->focus_out = textvn_ibus_engine_focus_out;
    engine_class->enable = textvn_ibus_engine_enable;
    engine_class->disable = textvn_ibus_engine_disable;
    engine_class->reset = textvn_ibus_engine_reset;
    engine_class->set_surrounding_text = textvn_ibus_engine_set_surrounding_text;
}

IBusEngine *textvn_ibus_engine_new(void) {
    return (IBusEngine *)g_object_new(TEXTVN_TYPE_IBUS_ENGINE, NULL);
}
