/* test_compose.c — Mô hình preedit Linux (mirror test của windows-tsf/src/compose.rs)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Chạy với engine Rust THẬT (libtextvn_ffi): mô phỏng tài liệu gồm text đã commit +
 * preedit, áp đúng kế hoạch như adapter IBus/Fcitx5.
 */

#define _POSIX_C_SOURCE 200809L

#include "lc_compose.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

typedef struct doc {
    uint32_t committed[1024];
    size_t   committed_len;
    lc_comp  comp;
} doc;

static void doc_text(const doc *d, char *out, size_t n) {
    uint32_t all[2048];
    size_t len = 0;
    memcpy(all, d->committed, d->committed_len * sizeof(uint32_t));
    len = d->committed_len;
    memcpy(all + len, d->comp.text, d->comp.len * sizeof(uint32_t));
    len += d->comp.len;
    /* UTF-32 → UTF-8 tối giản cho so sánh. */
    size_t o = 0;
    for (size_t i = 0; i < len && o + 5 < n; ++i) {
        uint32_t c = all[i];
        if (c < 0x80) {
            out[o++] = (char)c;
        } else if (c < 0x800) {
            out[o++] = (char)(0xC0 | (c >> 6));
            out[o++] = (char)(0x80 | (c & 0x3F));
        } else if (c < 0x10000) {
            out[o++] = (char)(0xE0 | (c >> 12));
            out[o++] = (char)(0x80 | ((c >> 6) & 0x3F));
            out[o++] = (char)(0x80 | (c & 0x3F));
        } else {
            out[o++] = (char)(0xF0 | (c >> 18));
            out[o++] = (char)(0x80 | ((c >> 12) & 0x3F));
            out[o++] = (char)(0x80 | ((c >> 6) & 0x3F));
            out[o++] = (char)(0x80 | (c & 0x3F));
        }
    }
    out[o] = '\0';
}

static void apply_plan(doc *d, ime_instance *inst, const lc_plan *plan) {
    if (plan->delete_before) {
        assert(plan->delete_before <= d->committed_len);
        d->committed_len -= plan->delete_before;
    }
    if (plan->end) {
        size_t n = 0;
        const uint32_t *t = lc_plan_commit_text(&d->comp, plan, &n);
        memcpy(d->committed + d->committed_len, t, n * sizeof(uint32_t));
        d->committed_len += n;
    }
    lc_comp_apply(&d->comp, plan);
    if (plan->reset_engine) ime_reset(inst);
}

static lc_plan press(doc *d, ime_instance *inst, uint32_t vk, uint32_t ch) {
    lc_key key = lc_key_classify(vk, ch);
    ime_key_v1 k;
    lc_key_to_ime(key, vk, 0, &k);
    ime_result_v1 r;
    assert(ime_key(inst, &k, &r) == IME_OK);
    lc_plan plan;
    lc_plan_key(&d->comp, key, &r, &plan);

    apply_plan(d, inst, &plan);
    if (!plan.eaten) {
        /* Bất biến B2: app chỉ nhận phím khi preedit đã đóng. */
        assert(d->comp.len == 0);
        switch (key.kind) {
        case LC_KEY_CHAR:  d->committed[d->committed_len++] = key.ch; break;
        case LC_KEY_ENTER: d->committed[d->committed_len++] = '\n'; break;
        case LC_KEY_TAB:   d->committed[d->committed_len++] = '\t'; break;
        case LC_KEY_BACKSPACE:
            if (d->committed_len) d->committed_len--;
            break;
        default: break;
        }
    }
    return plan;
}

static void type_str(doc *d, ime_instance *inst, const char *s) {
    /* Chỉ ASCII + các ký tự điều khiển mô phỏng phím. */
    for (const char *p = s; *p; ++p) {
        switch (*p) {
        case '\n': press(d, inst, LC_VK_RETURN, 0); break;
        case '\b': press(d, inst, LC_VK_BACK, 0); break;
        case 0x1B: press(d, inst, LC_VK_ESCAPE, 0); break;
        case '\t': press(d, inst, LC_VK_TAB, 0); break;
        default:   press(d, inst, (uint32_t)(unsigned char)*p, (uint32_t)(unsigned char)*p); break;
        }
    }
}

static ime_instance *engine_with(const char *cfg) {
    ime_instance *inst = NULL;
    int32_t rc = ime_instance_new((const uint8_t *)cfg, strlen(cfg), &inst);
    assert(rc == IME_OK && inst);
    return inst;
}

static void expect(const char *cfg, const char *input, const char *want) {
    ime_instance *inst = engine_with(cfg);
    doc d;
    memset(&d, 0, sizeof(d));
    type_str(&d, inst, input);
    char got[4096];
    doc_text(&d, got, sizeof(got));
    if (strcmp(got, want) != 0) {
        fprintf(stderr, "FAIL input=%s\n  want=%s\n  got =%s\n", input, want, got);
        abort();
    }
    ime_instance_free(inst);
}

#define NO_CAPS "{\"config_version\":1,\"auto_capitalize\":false}"

static void test_classify(void) {
    assert(lc_key_classify(0xFFBE /* F1 keysym */, 0).kind == LC_KEY_OTHER);
    assert(lc_key_classify(0x2E, 0).kind == LC_KEY_OTHER);
    assert(lc_key_classify(LC_VK_RETURN, '\r').kind == LC_KEY_ENTER);
    assert(lc_key_classify(0, '\n').kind == LC_KEY_ENTER);
    assert(lc_key_classify(LC_VK_BACK, 8).kind == LC_KEY_BACKSPACE);
    lc_key a = lc_key_classify('A', 'a');
    assert(a.kind == LC_KEY_CHAR && a.ch == 'a');
    assert(lc_key_classify(0x20, ' ').kind == LC_KEY_CHAR);
    assert(lc_key_classify_with_mods(0x20, ' ', IME_MOD_CTRL).kind == LC_KEY_CTRL_SPACE);
    assert(lc_key_classify_with_mods(0x20, ' ', IME_MOD_CTRL | IME_MOD_CAPS).kind == LC_KEY_CTRL_SPACE);
    assert(lc_key_classify_with_mods(0x20, ' ', 0).kind == LC_KEY_CHAR);
    assert(lc_key_classify(0, 0x7F).kind == LC_KEY_OTHER);
    assert(lc_key_engine_ch((lc_key){ LC_KEY_ENTER, 0 }) == 0);
}

static void test_typing(void) {
    expect(NO_CAPS, "dduocj ", "được ");
    expect(NO_CAPS, "Vieetj Nam", "Việt Nam");
    expect(NO_CAPS, "hello world ", "hello world ");
    expect(NO_CAPS, "chaof banj\n", "chào bạn\n");
    expect(NO_CAPS, "dduocj\b", "đượ"); /* R2-55: xoá ký tự cuối như UniKey */
    expect(NO_CAPS, "ab\b\b\b", "");
    expect(NO_CAPS, "ab \b\b", "a");
    expect(NO_CAPS, "dduocj\x1b", "dduocj");
    expect(NO_CAPS, "vieetj2026.", "việt2026.");
    expect("{\"config_version\":1,\"auto_capitalize\":false,\"auto_restore_english\":true}",
           "asdf ", "asdf ");
    expect("{\"config_version\":1,\"auto_capitalize\":false,\"auto_restore_english\":true}",
           "asdf\n", "asdf\n");
    expect("{\"config_version\":1,\"method\":\"vni\",\"auto_capitalize\":false}",
           "tie6ng1 Vie65t ", "tiếng Việt ");
    expect("{\"config_version\":1,\"auto_capitalize\":false,\"macro_trigger\":\"tab\","
           "\"macros\":[{\"trigger\":\"cty\",\"expand\":\"C\xc3\xb4ng ty TNHH\",\"when\":\"always\"}]}",
           "cty\t", "Công ty TNHH");
    expect("{\"config_version\":1,\"auto_capitalize\":true}", "chao. ban", "chao. Ban");
}

static void test_composition_holds_whole_word(void) {
    ime_instance *inst = engine_with(NO_CAPS);
    doc d;
    memset(&d, 0, sizeof(d));
    type_str(&d, inst, "Vieetj");
    assert(d.committed_len == 0 && "chưa commit giữa từ");
    /* Phím điều hướng: commit nguyên văn, từ mới không sửa text cũ. */
    lc_plan p = press(&d, inst, 0x25, 0);
    assert(!p.eaten && p.end);
    type_str(&d, inst, "ej");
    char got[256];
    doc_text(&d, got, sizeof(got));
    assert(strcmp(got, "Việtẹ") == 0);
    ime_instance_free(inst);
}

static void test_plan_edge_cases(void) {
    lc_comp cur;
    memset(&cur, 0, sizeof(cur));
    cur.text[0] = 'a';
    cur.text[1] = 'b';
    cur.len = 2;
    ime_result_v1 r;
    memset(&r, 0, sizeof(r));
    lc_plan p;

    r.action = 99;
    lc_plan_key(&cur, (lc_key){ LC_KEY_CHAR, 'c' }, &r, &p);
    assert(!p.eaten && p.end && !p.has_text);

    memset(&r, 0, sizeof(r));
    r.action = IME_ACTION_REPLACE;
    r.delete_count = 4;
    r.insert[0] = 0x1F600;
    r.insert_len = 1;
    r.flags = IME_FLAG_WORD_END;
    cur.len = 1;
    lc_plan_key(&cur, (lc_key){ LC_KEY_TAB, 0 }, &r, &p);
    assert(p.delete_before == 3 && p.end && p.eaten && p.text_len == 1);

    /* R2-121: Ctrl+Space PASS (không khớp dự đoán) → không chèn space, không
     * nuốt phím, CHỐT preedit đang mở (engine đã nhả từ) và reset engine. */
    memset(&r, 0, sizeof(r));
    r.action = IME_ACTION_PASS;
    r.flags = IME_FLAG_WORD_END;
    lc_plan_key(&cur, (lc_key){ LC_KEY_CTRL_SPACE, 0 }, &r, &p);
    assert(!p.eaten && p.end && !p.has_text && p.reset_engine);
    /* Không có preedit: không có gì để chốt. */
    cur.len = 0;
    lc_plan_key(&cur, (lc_key){ LC_KEY_CTRL_SPACE, 0 }, &r, &p);
    assert(!p.eaten && !p.end && !p.has_text && p.reset_engine);
}

static void test_modifier_toggle(void) {
    lc_modifier_toggle t = { 0 };
    lc_modifier_toggle_down(&t, LC_MOD_KEY_CTRL, 0, 0, 0);
    lc_modifier_toggle_down(&t, LC_MOD_KEY_SHIFT, 1, 0, 0);
    assert(lc_modifier_toggle_up(&t, LC_MOD_KEY_SHIFT) == 1);
    assert(lc_modifier_toggle_up(&t, LC_MOD_KEY_CTRL) == 0);

    lc_modifier_toggle_down(&t, LC_MOD_KEY_SHIFT, 0, 0, 0);
    lc_modifier_toggle_down(&t, LC_MOD_KEY_CTRL, 0, 1, 0);
    lc_modifier_toggle_down(&t, LC_MOD_NONE, 1, 1, 0); /* Ctrl+Shift+Z */
    assert(lc_modifier_toggle_up(&t, LC_MOD_KEY_CTRL) == 0);

    lc_modifier_toggle_down(&t, LC_MOD_KEY_SHIFT, 1, 0, 1); /* kèm Alt */
    assert(lc_modifier_toggle_up(&t, LC_MOD_KEY_SHIFT) == 0);
    lc_modifier_toggle_down(&t, LC_MOD_KEY_SHIFT, 1, 0, 0);
    lc_modifier_toggle_reset(&t);
    assert(lc_modifier_toggle_up(&t, LC_MOD_KEY_SHIFT) == 0);

    /* Ctrl+Shift+Space is a separate shortcut. Adapters disarm the pending
     * modifier-only chord before handling it, so neither key release can
     * toggle a second time. */
    lc_modifier_toggle_down(&t, LC_MOD_KEY_CTRL, 0, 0, 0);
    lc_modifier_toggle_down(&t, LC_MOD_KEY_SHIFT, 1, 0, 0);
    lc_modifier_toggle_down(&t, LC_MOD_NONE, 1, 1, 0); /* Space */
    lc_modifier_toggle_reset(&t);
    assert(lc_modifier_toggle_up(&t, LC_MOD_KEY_SHIFT) == 0);
    assert(lc_modifier_toggle_up(&t, LC_MOD_KEY_CTRL) == 0);
}

static void test_config_sync(void) {
    char path[] = "/tmp/textvn_cfg_XXXXXX";
    int fd = mkstemp(path);
    assert(fd >= 0);
    const char *vni = "{\"config_version\":1,\"method\":\"vni\",\"auto_capitalize\":false}";
    assert(write(fd, vni, strlen(vni)) == (ssize_t)strlen(vni));
    close(fd);

    ime_instance *inst = engine_with(NO_CAPS);
    lc_config_state st;
    memset(&st, 0, sizeof(st));
    /* D16: chưa nạp config ⇒ auto_capitalize hiệu lực = mặc định schema (1). */
    assert(lc_assist_auto_capitalize_enabled(&st) == 1);
    assert(lc_config_sync(inst, &st, path) == 1);
    assert(lc_config_sync(inst, &st, path) == 0 && "không đổi → không nạp lại");
    /* D16: đọc qua đúng đường config của assist (lc_config_sync → lc_config_state). */
    assert(st.auto_capitalize == 0);
    assert(lc_assist_auto_capitalize_enabled(&st) == 0);

    doc d;
    memset(&d, 0, sizeof(d));
    type_str(&d, inst, "a1 ");
    char got[64];
    doc_text(&d, got, sizeof(got));
    assert(strcmp(got, "\xc3\xa1 ") == 0 && "VNI từ config.json");
    assert(lc_config_sync(inst, &st, "/nonexistent/config.json") == 0);
    assert(st.allow_macro_when_vi_off == 0);

    /* Đổi file → nạp lại và đọc cờ gõ tắt khi tắt VN. Khoá `auto_capitalize`
     * thiếu ⇒ mặc định schema = bật (D16). */
    const char *with_flag = "{\"config_version\":1,\"method\":\"vni\",\"allow_macro_when_vi_off\":true}";
    FILE *f = fopen(path, "wb");
    assert(f && fwrite(with_flag, 1, strlen(with_flag), f) == strlen(with_flag));
    fclose(f);
    assert(lc_config_sync(inst, &st, path) == 1);
    assert(st.allow_macro_when_vi_off == 1);
    assert(st.auto_capitalize == 1);
    assert(lc_assist_auto_capitalize_enabled(&st) == 1);

    /* Config hỏng: engine giữ config cũ → cờ cũng giữ nguyên. */
    f = fopen(path, "wb");
    assert(f && fwrite("{oops", 1, 5, f) == 5);
    fclose(f);
    assert(lc_config_sync(inst, &st, path) == 1);
    assert(st.allow_macro_when_vi_off == 1);
    assert(st.auto_capitalize == 1);
    ime_instance_free(inst);
    unlink(path);
}

static void test_state_file(void) {
    char dir[] = "/tmp/textvn_state_XXXXXX";
    assert(mkdtemp(dir));
    char path[256];
    snprintf(path, sizeof(path), "%s/sub/state.json", dir);

    assert(lc_state_read_enabled(path, 1) == 1 && "chưa có file → mặc định");
    assert(lc_state_read_enabled(path, 0) == 0);
    lc_config_state st;
    memset(&st, 0, sizeof(st));
    int enabled = 1;
    assert(lc_state_sync(&st, path, &enabled) == 0);

    assert(lc_state_write_enabled(path, 0) == 0 && "tạo cả thư mục cha");
    assert(lc_state_read_enabled(path, 1) == 0);
    assert(lc_state_sync(&st, path, &enabled) == 1 && enabled == 0);
    assert(lc_state_sync(&st, path, &enabled) == 0);

    /* Giữ khoá khác (apps{} do tray Windows / người dùng ghi). */
    FILE *f = fopen(path, "wb");
    const char *js = "{\"global_enabled\":false,\"apps\":{\"x\":true}}";
    assert(f && fwrite(js, 1, strlen(js), f) == strlen(js));
    fclose(f);
    assert(lc_state_write_enabled(path, 1) == 0);
    f = fopen(path, "rb");
    char buf[512] = {0};
    assert(f && fread(buf, 1, sizeof(buf) - 1, f) > 0);
    fclose(f);
    assert(strstr(buf, "\"apps\"") && strstr(buf, "\"global_enabled\": true"));
    assert(lc_state_sync(&st, path, &enabled) == 1 && enabled == 1);

    char cmd[300];
    snprintf(cmd, sizeof(cmd), "rm -rf %s", dir);
    assert(system(cmd) == 0);
}

/* Gõ tắt khi tắt tiếng Việt: engine chạy passthrough, từ nằm trong preedit tới ranh giới,
 * Tab bung gõ tắt ngay trong preedit — không xoá chữ đã commit của app. */
static void test_macro_when_vi_off(void) {
    ime_instance *inst = engine_with(
        "{\"config_version\":1,\"auto_capitalize\":false,\"allow_macro_when_vi_off\":true,"
        "\"macros\":[{\"trigger\":\"vn\",\"expand\":\"Vi\u1ec7t Nam\"}]}");
    ime_context_v1 ctx;
    memset(&ctx, 0, sizeof(ctx));
    ctx.abi_version = IME_ABI_VERSION;
    ctx.enabled = 0;
    ctx.field_role = IME_FIELD_BODY;
    ctx.caps = IME_CAP_PREEDIT;
    ctx.hint = -1;
    assert(ime_set_context(inst, &ctx) == IME_OK);

    doc d;
    memset(&d, 0, sizeof(d));
    type_str(&d, inst, "xin vieet vn\t");
    char got[128];
    doc_text(&d, got, sizeof(got));
    if (strcmp(got, "xin vieet Vi\xe1\xbb\x87t Nam") != 0) {
        fprintf(stderr, "macro vi-off: got=%s\n", got);
        abort();
    }
    assert(d.comp.len == 0);
    ime_instance_free(inst);
}

static void test_find_settings_binary(void) {
    char dir[] = "/tmp/textvn_bin_XXXXXX";
    assert(mkdtemp(dir));
    char bin[300], out[512];
    snprintf(bin, sizeof(bin), "%s/textvn-settings", dir);
    FILE *f = fopen(bin, "wb");
    assert(f && fputs("#!/bin/sh\n", f) >= 0);
    fclose(f);
    assert(lc_find_settings_binary(out, sizeof(out), dir) != 0 || strcmp(out, bin) != 0);
    assert(chmod(bin, 0755) == 0);
    assert(lc_find_settings_binary(out, sizeof(out), dir) == 0 && strcmp(out, bin) == 0);

    /* Tìm qua PATH khi không có self_dir. */
    const char *old = getenv("PATH");
    char saved[4096];
    snprintf(saved, sizeof(saved), "%s", old ? old : "");
    char newpath[4400];
    snprintf(newpath, sizeof(newpath), ".:%s:%s", dir, saved);
    setenv("PATH", newpath, 1);
    assert(lc_find_settings_binary(out, sizeof(out), NULL) == 0 && strcmp(out, bin) == 0);
    setenv("PATH", saved, 1);

    unlink(bin);
    rmdir(dir);
}

static void test_assist_state_file(void) {
    char dir[] = "/tmp/textvn_state_assist_XXXXXX";
    assert(mkdtemp(dir));
    char path[300];
    snprintf(path, sizeof(path), "%s/state.json", dir);

    assert(lc_state_read_assist_enabled(path, 0) == 0);
    assert(lc_state_read_assist_enabled(path, 1) == 1);

    lc_config_state st;
    memset(&st, 0, sizeof(st));
    int assist_en = 0;
    assert(lc_state_sync_assist(&st, path, &assist_en) == 0);

    /* Write assist_enabled = 1 */
    assert(lc_state_write_assist_enabled(path, 1) == 0);
    assert(lc_state_read_assist_enabled(path, 0) == 1);
    assert(lc_state_sync_assist(&st, path, &assist_en) == 1 && assist_en == 1);

    /* Write assist_enabled = 0 */
    assert(lc_state_write_assist_enabled(path, 0) == 0);
    assert(lc_state_read_assist_enabled(path, 1) == 0);

    /* Helper lc_assist_is_active */
    assert(lc_assist_is_active(0, LC_ASSIST_MODE_HIGHLIGHT) == 0);
    assert(lc_assist_is_active(1, LC_ASSIST_MODE_OFF) == 0);
    assert(lc_assist_is_active(1, LC_ASSIST_MODE_HIGHLIGHT) == 1);
    assert(lc_assist_is_active(1, LC_ASSIST_MODE_AUTOCORRECT) == 1);

    char cmd[300];
    snprintf(cmd, sizeof(cmd), "rm -rf %s", dir);
    assert(system(cmd) == 0);
}

static void test_assist_auto_fix_sentence(void) {
    const char *cfg = "{\"config_version\":1,\"auto_capitalize\":false,"
                      "\"assist\":{\"mode\":\"autocorrect\",\"auto_fix\":\"safe\"}}";
    ime_instance *inst = engine_with(cfg);
    doc d;
    memset(&d, 0, sizeof(d));

    /* Type duplicate word with Vietnamese marks: "tooi tooi ddi hocj ." -> "tôi tôi đi học ." */
    type_str(&d, inst, "tooi tooi ddi hocj .");

    ime_assist_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    int rc = ime_assist_check_instance(inst, &res);
    assert(rc == IME_OK);
    assert(res.count > 0 && "phải phát hiện lỗi lặp từ");

    char tail[256];
    size_t tail_len = 0;
    assert(ime_assist_sentence_tail(inst, 256, tail, sizeof(tail), &tail_len) == IME_OK);
    assert(tail_len > 0);

    lc_sentence_fix_plan plan;
    int ok = lc_plan_sentence_auto_fix(inst, tail, &res, &plan);
    assert(ok == 1);
    assert(plan.fixed_count >= 1);
    assert(plan.delete_glyphs > 0);
    assert(strstr(plan.replacement, "đi") != NULL);

    /* R-2: đuôi câu GỐC phải được lưu lại (adapter dùng để xác minh vùng sắp
     * xoá trong document trước khi xoá + để hoàn tác). */
    assert(plan.original_tail[0] != '\0');
    assert(strstr(plan.original_tail, "tôi") != NULL);
    /* Đuôi gốc phải là hậu tố của câu đã kiểm tra. */
    size_t tail_str_len = strlen(tail);
    size_t orig_len = strlen(plan.original_tail);
    assert(orig_len > 0 && orig_len <= tail_str_len);
    assert(strcmp(tail + (tail_str_len - orig_len), plan.original_tail) == 0);

    ime_instance_free(inst);
}

/* R-5/R-6: ngữ nghĩa Mode 1/Mode 2 **duy nhất** cho 3 OS (P4-0 §4.3). */
static void test_assist_mode_semantics(void) {
    /* Mode 2 (tự sửa cả câu): mode "autocorrect" VÀ auto_fix "safe" VÀ trigger
     * kết thúc câu. */
    assert(lc_assist_should_auto_fix(LC_ASSIST_MODE_AUTOCORRECT, LC_ASSIST_AUTO_FIX_SAFE, 1) == 1);
    /* Thiếu mode autocorrect (chỉ auto_fix=safe) ⇒ KHÔNG tự sửa. */
    assert(lc_assist_should_auto_fix(LC_ASSIST_MODE_HIGHLIGHT, LC_ASSIST_AUTO_FIX_SAFE, 1) == 0);
    /* Thiếu auto_fix=safe ⇒ KHÔNG tự sửa. */
    /* 0.2.75: auto_fix không còn quyết định — autocorrect luôn tự sửa an toàn. */
    assert(lc_assist_should_auto_fix(LC_ASSIST_MODE_AUTOCORRECT, LC_ASSIST_AUTO_FIX_OFF, 1) == 1);
    /* Trigger không phải kết thúc câu (space/,) ⇒ KHÔNG tự sửa. */
    assert(lc_assist_should_auto_fix(LC_ASSIST_MODE_AUTOCORRECT, LC_ASSIST_AUTO_FIX_SAFE, 0) == 0);
    assert(lc_assist_should_auto_fix(LC_ASSIST_MODE_OFF, LC_ASSIST_AUTO_FIX_OFF, 1) == 0);

    /* Mode 1 (điểm hiển thị) — R2-129: danh sách SỐNG ở mọi điểm kiểm tra
     * (ranh giới từ lẫn kết thúc câu) khi mode != off. */
    assert(lc_assist_should_display(LC_ASSIST_MODE_HIGHLIGHT, 1) == 1);
    assert(lc_assist_should_display(LC_ASSIST_MODE_AUTOCORRECT, 1) == 1);
    assert(lc_assist_should_display(LC_ASSIST_MODE_HIGHLIGHT, 0) == 1);
    assert(lc_assist_should_display(LC_ASSIST_MODE_OFF, 1) == 0);
    assert(lc_assist_should_display(LC_ASSIST_MODE_OFF, 0) == 0);

    /* R2-129: phím khi danh sách sống đang hiện — không chiếm phím gõ. */
    assert(lc_assist_live_key('a', 0, 0, 0) == LC_LIVE_KEEP);
    assert(lc_assist_live_key('1', 0, 0, 0) == LC_LIVE_KEEP);      /* số trần = gõ số */
    assert(lc_assist_live_key(' ', 0, 0, 0) == LC_LIVE_KEEP);
    assert(lc_assist_live_key('.', 0, 0, 0) == LC_LIVE_KEEP);
    assert(lc_assist_live_key(0xff08, 0, 0, 0) == LC_LIVE_KEEP);   /* BackSpace */
    assert(lc_assist_live_key(0x01001ea1, 0, 0, 0) == LC_LIVE_KEEP); /* ạ (XKB vn) */
    assert(lc_assist_live_key(0xff1b, 0, 0, 0) == LC_LIVE_CLOSE);  /* Escape */
    assert(lc_assist_live_key('1', 0, 1, 0) == LC_LIVE_APPLY_1);
    assert(lc_assist_live_key('3', 0, 1, 0) == LC_LIVE_APPLY_1 + 2);
    assert(lc_assist_live_key('4', 0, 1, 0) == LC_LIVE_DISMISS);
    assert(lc_assist_live_key(0xff0d, 0, 0, 0) == LC_LIVE_DISMISS); /* Return */
    assert(lc_assist_live_key(0xff09, 0, 0, 0) == LC_LIVE_DISMISS); /* Tab */
    assert(lc_assist_live_key(0xff52, 0, 0, 0) == LC_LIVE_DISMISS); /* Up */
    assert(lc_assist_live_key('c', 1, 0, 0) == LC_LIVE_DISMISS);    /* Ctrl+C */
}

static void test_assist_auto_fix_spacing_punct(void) {
    const char *cfg = "{\"config_version\":1,\"auto_capitalize\":false,"
                      "\"assist\":{\"mode\":\"autocorrect\",\"auto_fix\":\"safe\"}}";
    ime_instance *inst = engine_with(cfg);
    doc d;
    memset(&d, 0, sizeof(d));

    /* Type space before comma with Vietnamese marks: "chaof banj ," -> "chào bạn ," */
    type_str(&d, inst, "chaof banj ,");

    ime_assist_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    int rc = ime_assist_check_instance(inst, &res);
    assert(rc == IME_OK);
    assert(res.count > 0 && "phải phát hiện khoảng trắng trước dấu phẩy");

    char tail[256];
    size_t tail_len = 0;
    assert(ime_assist_sentence_tail(inst, 256, tail, sizeof(tail), &tail_len) == IME_OK);

    lc_sentence_fix_plan plan;
    int ok = lc_plan_sentence_auto_fix(inst, tail, &res, &plan);
    assert(ok == 1);
    assert(plan.fixed_count >= 1);
    assert(strstr(plan.replacement, ",") != NULL);

    /* L3: kế hoạch phải XOÁ khoảng trắng, KHÔNG được nhân đôi dấu câu.
     * Span của L3 là KHOẢNG TRẮNG còn `suggest` chỉ là dấu câu để hiển thị ⇒
     * thay span bằng `suggest` sinh ra "chào bạn,," (hỏng text người dùng).
     * Cùng lỗi ở macOS (AssistModel) — cần sửa cả 2 bên để giữ parity. */
    assert(strstr(plan.replacement, ",,") == NULL);
    assert(plan.replacement[0] != '\0');
    assert(plan.replacement[strlen(plan.replacement) - 1] == ',');

    ime_instance_free(inst);
}

/* Q-6 / D16 (Fassist-20): L6 ("chữ thường đầu câu") phải bị lọc ở adapter khi
 * `config.auto_capitalize=true`, cho **cả** Mode 1 (hiển thị) và Mode 2 (tự sửa)
 * — dùng đúng chuỗi "Xin chào. bạn khỏe không" của corpus/assist/vi_basic.ass.
 *
 * Mode 2 dùng `lc_plan_sentence_auto_fix` (compose.c — cần POSIX) nên test này
 * nằm ở đây (chạy CI Linux); phần hàm thuần + engine nằm ở test_assist_result.c
 * (chạy được cả Windows). */
static void test_assist_l6_auto_capitalize_mode2(void) {
    const char *cfg = "{\"config_version\":1,\"auto_capitalize\":false,"
                      "\"assist\":{\"mode\":\"autocorrect\",\"auto_fix\":\"safe\"}}";
    ime_instance *inst = engine_with(cfg);
    doc d;
    memset(&d, 0, sizeof(d));
    /* Telex: "Xin chaof. bajn khoer khoong." → "Xin chào. bạn khỏe không." */
    type_str(&d, inst, "Xin chaof. bajn khoer khoong.");

    ime_assist_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    assert(ime_assist_check_instance(inst, &res) == IME_OK);

    int l6 = 0;
    for (uint32_t i = 0; i < res.count; ++i) {
        if (res.issues[i].kind == IME_ASSIST_KIND_LOWERCASE_SENTENCE) l6++;
    }
    assert(l6 == 1 && "câu corpus phải có đúng 1 lỗi L6");

    char tail[256 * 4 + 1];
    size_t tail_len = 0;
    assert(ime_assist_sentence_tail(inst, 256, tail, sizeof(tail), &tail_len) == IME_OK);
    assert(tail_len > 0 && tail_len < sizeof(tail));
    tail[tail_len] = '\0';

    /* auto_capitalize=false ⇒ Mode 2 VẪN tự sửa L6 (hành vi cũ, không đổi). */
    lc_sentence_fix_plan plan;
    assert(lc_plan_sentence_auto_fix(inst, tail, &res, &plan) == 1);
    assert(plan.fixed_count == 1);
    assert(plan.delete_glyphs > 0);
    assert(plan.replacement[0] == 'B'); /* "bạn" → "Bạn" (span 10..11) */

    /* auto_capitalize=true ⇒ adapter lọc L6 TRƯỚC khi lập kế hoạch ⇒ Mode 2
     * không sửa gì (không trùng chức năng với auto_capitalize). */
    ime_assist_result_v1 filtered = res;
    assert(lc_assist_filter_auto_capitalize(&filtered, 1) == 0);
    lc_sentence_fix_plan blocked;
    assert(lc_plan_sentence_auto_fix(inst, tail, &filtered, &blocked) == 0);

    ime_instance_free(inst);
}

/* OP-007 (R2-139): phím tắt Ctrl+Shift+F9 chuyển mã vùng bôi đen — chỉ đúng
 * tổ hợp Ctrl+Shift (không Alt/Super); Ctrl+F9 / Shift+F9 là Recall không dính. */
static void test_transcode_hotkey(void) {
    assert(lc_is_transcode_hotkey(0xffc6u, 1, 0, 1, 0) == 1); /* Ctrl+Shift+F9 */
    assert(lc_is_transcode_hotkey(0xffc6u, 1, 1, 1, 0) == 0); /* + Alt */
    assert(lc_is_transcode_hotkey(0xffc6u, 1, 0, 1, 1) == 0); /* + Super */
    assert(lc_is_transcode_hotkey(0xffc6u, 1, 0, 0, 0) == 0); /* Ctrl+F9 (Recall) */
    assert(lc_is_transcode_hotkey(0xffc6u, 0, 0, 1, 0) == 0); /* Shift+F9 (Recall) */
    assert(lc_is_transcode_hotkey(0xffc6u, 0, 0, 0, 0) == 0); /* F9 trần */
    assert(lc_is_transcode_hotkey(0xffc7u, 1, 0, 1, 0) == 0); /* F10 ≠ F9 */
    assert(lc_is_transcode_hotkey(0x20u, 1, 0, 1, 0) == 0);   /* Ctrl+Shift+Space = toggle */
}

/* OP-007 (R2-139): cắt selection từ surrounding text của IBus/Fcitx5 —
 * cursor/anchor đếm theo KÝ TỰ Unicode; del_offset theo quy ước
 * deleteSurroundingText (âm = bắt đầu TRƯỚC caret). */
static void test_transcode_selection_slice(void) {
    lc_selection_info info;

    /* ASCII: "Tieng Viet", cursor=10, anchor=6 → "Viet". */
    const char *ascii = "Tieng Viet";
    assert(lc_selection_slice(ascii, 10, 6, 64000, &info) == 1);
    assert(info.start_byte == 6 && info.byte_len == 4 && info.char_len == 4);
    assert(info.del_offset == -4); /* 6 − 10 */

    /* UTF-8 3-byte ('ộ' = U+1ED9 → E1 BB 99): slice cắt ĐÚNG ranh giới byte. */
    const char *utf8 = "Di h\xe1\xbb\x99" "c."; /* "Di học." */
    assert(lc_selection_slice(utf8, 7, 4, 64000, &info) == 1);
    assert(info.start_byte == 4 && info.byte_len == 5 && info.char_len == 3);
    assert(info.del_offset == -3);

    /* Chọn ngược (anchor > cursor): start = cursor ⇒ del_offset == 0.
     * Invariant: caret luôn là MỘT đầu selection ⇒ del_offset ∈ {0, −char_len},
     * KHÔNG BAO GIỜ dương ⇒ không bao giờ xoá chữ SAU caret. */
    assert(lc_selection_slice(ascii, 6, 10, 64000, &info) == 1);
    assert(info.start_byte == 6 && info.char_len == 4 && info.del_offset == 0);
    assert(info.del_offset == 0 || info.del_offset == -(long)info.char_len);

    /* Không có selection (anchor == cursor). */
    assert(lc_selection_slice(ascii, 5, 5, 64000, &info) == 0);
    /* Vượt trần 64 000 ký tự. */
    assert(lc_selection_slice(ascii, 10, 0, 3, &info) == 0);
    /* Cursor nằm ngoài text (dữ liệu lệch). */
    assert(lc_selection_slice(ascii, 99, 6, 64000, &info) == 0);
    /* UTF-8 lỗi giữa chừng → fail-safe. */
    assert(lc_selection_slice("ab\xc3", 3, 1, 64000, &info) == 0);
    assert(lc_selection_slice(NULL, 1, 0, 64000, &info) == 0);
}

/* R2-131/R2-134/R2-136: phím tắt trợ lý (chỉ Ctrl+Space), kế hoạch
 * sửa CHỦ ĐỘNG (gồm lỗi chính tả), câu sau sửa, nhãn xem trước dự đoán. */
static void test_assist_hotkey_explicit_preview(void) {
    assert(lc_is_assist_hotkey(0x20, 1, 0, 0, 0) == 1);    /* Ctrl+Space */
    assert(lc_is_assist_hotkey(0xff80, 1, 0, 0, 0) == 1);  /* Ctrl+KP_Space */
    assert(lc_is_assist_hotkey(0x20, 0, 1, 0, 0) == 0);    /* Alt+Space: KHÔNG (R2-136) */
    assert(lc_is_assist_hotkey(0x20, 0, 0, 0, 0) == 0);    /* Space trần */
    assert(lc_is_assist_hotkey(0x20, 1, 1, 0, 0) == 0);    /* Ctrl+Alt+Space */
    assert(lc_is_assist_hotkey(0x20, 1, 0, 1, 0) == 0);    /* Ctrl+Shift+Space = toggle */
    assert(lc_is_assist_hotkey(0x20, 0, 1, 0, 1) == 0);    /* Super */
    assert(lc_is_assist_hotkey('a', 1, 0, 0, 0) == 0);

    char out[256];
    assert(lc_strip_suffix("abc def", " def", out, sizeof(out)) == 1 && strcmp(out, "abc") == 0);
    assert(lc_strip_suffix("abc", "x", out, sizeof(out)) == 0);
    assert(lc_fixed_sentence("toi toi di hoc", " toi di hoc", " di hoc", out, sizeof(out)) == 1);
    assert(strcmp(out, "toi di hoc") == 0);
    assert(lc_fixed_sentence("toi di hoc", "xyz", "abc", out, sizeof(out)) == 0);
    assert(lc_preview_label(" phan mem", out, sizeof(out)) == 1);
    assert(strcmp(out, "\xe2\x87\xa5 phan mem") == 0);
    assert(lc_preview_label("   ", out, sizeof(out)) == 0);
    char tiny[4];
    assert(lc_preview_label("phan mem", tiny, sizeof(tiny)) == 0);
    /* SF-4: cổng chữ mờ do session quyết — bất kỳ cổng nào cũng ẩn. */
    assert(lc_preview_label_gated("phan mem", IME_GHOST_ASSIST_VISIBLE, out, sizeof(out)) == 0);
    assert(lc_preview_label_gated("phan mem", IME_GHOST_SECURE | IME_GHOST_TERMINAL, out,
                                  sizeof(out)) == 0);
    assert(lc_preview_label_gated("phan mem", 0u, out, sizeof(out)) == 1);

    /* Lỗi chính tả = chỉ-gợi-ý: Mode 2 (không cờ) không sửa, hành động chủ động thì sửa. */
    ime_instance *inst = engine_with(NO_CAPS);
    const char *sentence = "nganh nghe nghanh toi";
    ime_assist_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    res.count = 1;
    res.issues[0].kind = IME_ASSIST_KIND_SPELLING;
    res.issues[0].start = 11;
    res.issues[0].end = 17;
    res.issues[0].auto_fixable = 0;
    res.issues[0].confidence = 60;
    strcpy(res.issues[0].suggest, "nganh");
    res.issues[0].suggest_len = 5;
    lc_sentence_fix_plan plan;
    assert(lc_plan_sentence_auto_fix(inst, sentence, &res, &plan) == 0);
    assert(lc_plan_sentence_fix_ex(inst, sentence, &res, 1, &plan) == 1);
    assert(strcmp(plan.replacement, "nganh toi") == 0);
    assert(strcmp(plan.original_tail, "nghanh toi") == 0);
    ime_instance_free(inst);
}

/* R2-155: bảng phím hẹn timer idle (khớp panel_idle::idle_arms Windows). */
static void test_assist_idle_arms(void) {
    assert(LC_ASSIST_IDLE_MS == 1000);
    assert(lc_assist_idle_arms(1, 0, 1) == 1);  /* chữ/dấu câu */
    assert(lc_assist_idle_arms(0, 1, 1) == 1);  /* xoá */
    assert(lc_assist_idle_arms(0, 0, 1) == 0);  /* Enter/Tab/Esc/điều hướng */
    assert(lc_assist_idle_arms(1, 0, 0) == 0);  /* tiếng Việt tắt */
    assert(lc_assist_idle_arms(0, 1, 0) == 0);
}

static void test_modifier_ctrl_tap(void) {
    lc_modifier_toggle t = { 0 };
    /* Standalone Ctrl down then up */
    lc_modifier_toggle_down(&t, LC_MOD_KEY_CTRL, 0, 0, 0);
    assert(t.ctrl_armed == 1);
    assert(lc_modifier_ctrl_tap_up(&t, LC_MOD_KEY_CTRL) == 1);
    assert(t.ctrl_armed == 0);
    assert(lc_modifier_ctrl_tap_up(&t, LC_MOD_KEY_CTRL) == 0);

    /* R2-143: giữ Ctrl quá lâu (Ctrl+click, Ctrl+lăn chuột) không phải tap. */
    lc_modifier_toggle_down(&t, LC_MOD_KEY_CTRL, 0, 0, 0);
    t.ctrl_down_us -= LC_CTRL_TAP_MAX_US + 1000;
    assert(lc_modifier_ctrl_tap_up(&t, LC_MOD_KEY_CTRL) == 0);
    assert(t.ctrl_armed == 0);

    /* Ctrl down with Shift down -> not armed for ctrl tap */
    lc_modifier_toggle_down(&t, LC_MOD_KEY_CTRL, 0, 1, 0);
    assert(t.ctrl_armed == 0);
    assert(lc_modifier_ctrl_tap_up(&t, LC_MOD_KEY_CTRL) == 0);

    /* Ctrl down with Alt -> not armed */
    lc_modifier_toggle_down(&t, LC_MOD_KEY_CTRL, 0, 0, 1);
    assert(t.ctrl_armed == 0);
    assert(lc_modifier_ctrl_tap_up(&t, LC_MOD_KEY_CTRL) == 0);

    /* Ctrl down, then normal key arrives (which == LC_MOD_NONE) -> disarmed */
    lc_modifier_toggle_down(&t, LC_MOD_KEY_CTRL, 0, 0, 0);
    assert(t.ctrl_armed == 1);
    lc_modifier_toggle_down(&t, LC_MOD_NONE, 1, 0, 0);
    assert(t.ctrl_armed == 0);
    assert(lc_modifier_ctrl_tap_up(&t, LC_MOD_KEY_CTRL) == 0);
}

/* ---- SF-2: shadow lc_modifier_toggle ↔ textvn-session (qua C ABI ime_session_*) ----
 * Lái cả hai giống hệt adapter IBus/Fcitx5: key-down ⇒ toggle_down + shadow_key_down;
 * key-up ⇒ ctrl_tap_up, rồi toggle_up nếu không phải tap, rồi shadow_key_up. */
typedef struct shadow_pair {
    lc_modifier_toggle t;
    lc_session_shadow s;
    uint64_t step_ms; /* thời gian giữa 2 sự kiện */
} shadow_pair;

static int sp_down(shadow_pair *p, lc_modifier which, int is_space, int ctrl, int shift,
                   int alt) {
    lc_modifier_toggle_down(&p->t, which, ctrl, shift, alt);
    int legacy = which == LC_MOD_NONE && is_space && ctrl && shift; /* nhánh ADR-011 */
    if (legacy) lc_modifier_toggle_reset(&p->t);
    lc_session_shadow_key_down(&p->s, which, is_space, 0, ctrl, shift, alt, legacy);
    p->s.now_override_ms += p->step_ms;
    return legacy;
}

static int sp_up(shadow_pair *p, lc_modifier which) {
    int tap = lc_modifier_ctrl_tap_up(&p->t, which);
    int toggled = !tap && lc_modifier_toggle_up(&p->t, which);
    lc_session_shadow_key_up(&p->s, which, toggled);
    p->s.now_override_ms += p->step_ms;
    return toggled;
}

static void test_session_shadow_parity(void) {
    /* Mặc định (không env) shadow tắt: mọi hàm là no-op. */
    lc_session_shadow off;
    unsetenv("TEXTVN_SESSION_SHADOW");
    lc_session_shadow_init(&off);
    assert(off.session == NULL);
    lc_session_shadow_key_down(&off, LC_MOD_KEY_CTRL, 0, 0, 0, 0, 0, 0);
    lc_session_shadow_key_up(&off, LC_MOD_KEY_CTRL, 1);
    lc_session_shadow_reset(&off);
    assert(off.mismatches == 0 && off.toggles == 0);
    lc_session_shadow_free(&off);

    shadow_pair p;
    memset(&p, 0, sizeof(p));
    assert(lc_session_shadow_init_force(&p.s) == 0);
    assert(p.s.session != NULL);
    p.s.now_override_ms = 1000;
    p.step_ms = 300; /* > debounce 250 ms của session */

    /* 1. Ctrl → Shift, nhả Shift trước: bật/tắt đúng 1 lần. */
    sp_down(&p, LC_MOD_KEY_CTRL, 0, 0, 0, 0);
    sp_down(&p, LC_MOD_KEY_SHIFT, 0, 1, 0, 0);
    sp_up(&p, LC_MOD_KEY_SHIFT);
    sp_up(&p, LC_MOD_KEY_CTRL);
    /* 2. Shift → Ctrl, nhả Ctrl trước. */
    sp_down(&p, LC_MOD_KEY_SHIFT, 0, 0, 0, 0);
    sp_down(&p, LC_MOD_KEY_CTRL, 0, 0, 1, 0);
    sp_up(&p, LC_MOD_KEY_CTRL);
    sp_up(&p, LC_MOD_KEY_SHIFT);
    /* 3. Phím thứ ba (Ctrl+Shift+Z) huỷ. */
    sp_down(&p, LC_MOD_KEY_CTRL, 0, 0, 0, 0);
    sp_down(&p, LC_MOD_KEY_SHIFT, 0, 1, 0, 0);
    sp_down(&p, LC_MOD_NONE, 0, 1, 1, 0);
    sp_up(&p, LC_MOD_NONE);
    sp_up(&p, LC_MOD_KEY_CTRL);
    sp_up(&p, LC_MOD_KEY_SHIFT);
    /* 4. Kèm Alt/Super huỷ. */
    sp_down(&p, LC_MOD_KEY_CTRL, 0, 0, 0, 1);
    sp_down(&p, LC_MOD_KEY_SHIFT, 0, 1, 0, 1);
    sp_up(&p, LC_MOD_KEY_SHIFT);
    sp_up(&p, LC_MOD_KEY_CTRL);
    /* 5. Alt chen giữa rồi nhả. */
    sp_down(&p, LC_MOD_KEY_CTRL, 0, 0, 0, 0);
    sp_down(&p, LC_MOD_KEY_OTHER_MODIFIER, 0, 1, 0, 0);
    sp_up(&p, LC_MOD_KEY_OTHER_MODIFIER);
    sp_up(&p, LC_MOD_KEY_CTRL);
    /* 6. Phím giữ lặp không đổi kết quả. */
    sp_down(&p, LC_MOD_KEY_CTRL, 0, 0, 0, 0);
    sp_down(&p, LC_MOD_KEY_CTRL, 0, 1, 0, 0);
    sp_down(&p, LC_MOD_KEY_SHIFT, 0, 1, 0, 0);
    sp_down(&p, LC_MOD_KEY_SHIFT, 0, 1, 1, 0);
    sp_up(&p, LC_MOD_KEY_SHIFT);
    sp_up(&p, LC_MOD_KEY_CTRL);
    /* 7. Ctrl+Shift+Space (ADR-011) bật/tắt ở key-down; nhả sau đó không bật lại. */
    sp_down(&p, LC_MOD_KEY_CTRL, 0, 0, 0, 0);
    sp_down(&p, LC_MOD_KEY_SHIFT, 0, 1, 0, 0);
    sp_down(&p, LC_MOD_NONE, 1, 1, 1, 0);
    sp_up(&p, LC_MOD_NONE);
    sp_up(&p, LC_MOD_KEY_SHIFT);
    sp_up(&p, LC_MOD_KEY_CTRL);
    /* 8. Mất focus giữa chừng huỷ lần nhấn. */
    sp_down(&p, LC_MOD_KEY_CTRL, 0, 0, 0, 0);
    sp_down(&p, LC_MOD_KEY_SHIFT, 0, 1, 0, 0);
    lc_modifier_toggle_reset(&p.t);
    lc_session_shadow_reset(&p.s);
    sp_up(&p, LC_MOD_KEY_SHIFT);
    sp_up(&p, LC_MOD_KEY_CTRL);
    /* 9. Ctrl tap riêng không bật/tắt. */
    sp_down(&p, LC_MOD_KEY_CTRL, 0, 0, 0, 0);
    sp_up(&p, LC_MOD_KEY_CTRL);
    assert(p.s.toggles == 4);
    assert(p.s.mismatches == 0);

    /* Khác biệt ĐÃ BIẾT (shadow phải bắt được, ghi ALLOWED trong changes/SF-2.md):
     * (a) Linux cũ bật/tắt cả Ctrl+Shift+Alt+Space; session (= TSF preserved key
     *     TF_MOD_CONTROL|TF_MOD_SHIFT) thì không. */
    sp_down(&p, LC_MOD_NONE, 1, 1, 1, 1);
    sp_up(&p, LC_MOD_NONE);
    assert(p.s.mismatches == 1);
    /* (b) Linux cũ không debounce; session chặn lần bật/tắt thứ hai trong < 250 ms. */
    p.step_ms = 1;
    sp_down(&p, LC_MOD_KEY_CTRL, 0, 0, 0, 0);
    sp_down(&p, LC_MOD_KEY_SHIFT, 0, 1, 0, 0);
    sp_up(&p, LC_MOD_KEY_SHIFT); /* bật/tắt — khớp */
    sp_up(&p, LC_MOD_KEY_CTRL);
    assert(p.s.mismatches == 1);
    sp_down(&p, LC_MOD_KEY_SHIFT, 0, 0, 0, 0);
    sp_down(&p, LC_MOD_KEY_CTRL, 0, 0, 1, 0);
    sp_up(&p, LC_MOD_KEY_CTRL); /* mã cũ bật/tắt lần nữa sau 4 ms; session chặn */
    sp_up(&p, LC_MOD_KEY_SHIFT);
    assert(p.s.mismatches == 2);

    lc_session_shadow_free(&p.s);
    assert(p.s.session == NULL);
    lc_session_shadow_free(&p.s); /* lần hai vô hại */
}

/* SF-2 · SF-G1/G2: corpus toggle dùng chung 3 OS (`corpus/toggle/toggle.cases`, cú pháp ở đầu
 * tệp). Mã cũ Linux phải ra đúng số lần ghi (hoặc cột `linux=`), session đúng vết vàng.
 * Lái giống `session::toggle_corpus::events`: modifier đang giữ TRƯỚC sự kiện, K/P xuống+lên
 * cùng mốc, mặc định cách 300 ms. Gốc repo suy từ __FILE__ (CMake truyền đường dẫn tuyệt đối). */
static char *trim_ws(char *s) {
    while (*s == ' ' || *s == '\t') s++;
    size_t n = strlen(s);
    while (n > 0 && (s[n - 1] == ' ' || s[n - 1] == '\t' || s[n - 1] == '\n' || s[n - 1] == '\r')) {
        s[--n] = '\0';
    }
    return s;
}

static void test_toggle_corpus(void) {
    static const char suffix[] = "adapters/linux-common/tests/test_compose.c";
    char path[1024];
    const char *file = __FILE__;
    size_t flen = strlen(file), slen = sizeof(suffix) - 1;
    assert(flen >= slen && strcmp(file + (flen - slen), suffix) == 0);
    snprintf(path, sizeof(path), "%.*scorpus/toggle/toggle.cases", (int)(flen - slen), file);
    FILE *f = fopen(path, "r");
    if (!f) fprintf(stderr, "không mở được %s\n", path);
    assert(f != NULL);
    char line[512];
    int ncases = 0, failures = 0;
    while (fgets(line, sizeof(line), f)) {
        char *l = trim_ws(line);
        if (*l == '\0' || *l == '#') continue;
        char *cols[8];
        int ncols = 0;
        char *csave = NULL;
        for (char *tok = strtok_r(l, "|", &csave); tok && ncols < 8; tok = strtok_r(NULL, "|", &csave)) {
            cols[ncols++] = trim_ws(tok);
        }
        assert(ncols >= 3);
        const char *name = cols[0];
        unsigned expect_session = (unsigned)strtoul(cols[2], NULL, 10);
        unsigned expect_legacy = expect_session;
        for (int i = 3; i < ncols; i++) {
            if (strncmp(cols[i], "linux=", 6) == 0) {
                expect_legacy = (unsigned)strtoul(cols[i] + 6, NULL, 10);
            }
        }

        shadow_pair p;
        memset(&p, 0, sizeof(p));
        assert(lc_session_shadow_init_force(&p.s) == 0);
        uint64_t now = 1000, gap = 300;
        int ctrl = 0, shift = 0, alt = 0;
        unsigned legacy = 0;
        char *save = NULL;
        for (char *ev = strtok_r(cols[1], " ", &save); ev; ev = strtok_r(NULL, " ", &save)) {
            if (ev[0] == 'w') {
                gap = strtoull(ev + 1, NULL, 10);
                continue;
            }
            p.s.now_override_ms = now;
            if (strcmp(ev, "+C") == 0) {
                legacy += (unsigned)sp_down(&p, LC_MOD_KEY_CTRL, 0, 0, shift, alt);
                ctrl = 1;
            } else if (strcmp(ev, "+S") == 0) {
                legacy += (unsigned)sp_down(&p, LC_MOD_KEY_SHIFT, 0, ctrl, 0, alt);
                shift = 1;
            } else if (strcmp(ev, "+A") == 0) {
                legacy += (unsigned)sp_down(&p, LC_MOD_KEY_OTHER_MODIFIER, 0, ctrl, shift, 0);
                alt = 1;
            } else if (strcmp(ev, "-C") == 0) {
                ctrl = 0;
                legacy += (unsigned)sp_up(&p, LC_MOD_KEY_CTRL);
            } else if (strcmp(ev, "-S") == 0) {
                shift = 0;
                legacy += (unsigned)sp_up(&p, LC_MOD_KEY_SHIFT);
            } else if (strcmp(ev, "-A") == 0) {
                alt = 0;
                legacy += (unsigned)sp_up(&p, LC_MOD_KEY_OTHER_MODIFIER);
            } else if (strcmp(ev, "K") == 0 || strcmp(ev, "P") == 0) {
                int space = ev[0] == 'P';
                legacy += (unsigned)sp_down(&p, LC_MOD_NONE, space, ctrl, shift, alt);
                p.s.now_override_ms = now;
                legacy += (unsigned)sp_up(&p, LC_MOD_NONE);
            } else if (strcmp(ev, "R") == 0) {
                lc_modifier_toggle_reset(&p.t);
                lc_session_shadow_reset(&p.s);
            } else {
                fprintf(stderr, "toggle.cases: sự kiện lạ '%s' ở ca %s\n", ev, name);
                assert(0);
            }
            now += gap;
        }
        if (legacy != expect_legacy || p.s.session_toggles != expect_session) {
            fprintf(stderr, "FAIL toggle corpus %s: Linux cũ %u (mong %u), session %u (mong %u)\n",
                    name, legacy, expect_legacy, (unsigned)p.s.session_toggles, expect_session);
            failures++;
        }
        lc_session_shadow_free(&p.s);
        ncases++;
    }
    fclose(f);
    assert(ncases >= 16);
    assert(failures == 0);
}

/* ---- SF-3: oracle = bản C `lc_plan_key` CŨ (trước khi chuyển sang `ime_compose_plan`), giữ
 * nguyên văn để so sai khác với planner dùng chung trên ma trận đầu vào. Parity tạm (G22): xoá
 * cùng lúc xoá mã cũ ở bản sau khi session thành mặc định. ---- */
static int legacy_boundary_char(lc_key key, uint32_t *out) {
    switch (key.kind) {
    case LC_KEY_CHAR:  *out = key.ch; return 1;
    case LC_KEY_ENTER: *out = '\n';   return 1;
    case LC_KEY_TAB:   *out = '\t';   return 1;
    default: return 0;
    }
}

static void legacy_plan_key(const lc_comp *cur, lc_key key, const ime_result_v1 *r, lc_plan *out) {
    const int composing = cur->len > 0;
    const int word_end = (r->flags & IME_FLAG_WORD_END) != 0;
    memset(out, 0, sizeof(*out));

    if (r->action == IME_ACTION_PASS) {
        /* E62 (P11 Math): engine nuốt phím im lặng (vd. backtick `` ` `` arm chord hoặc Esc huỷ)
         * → đóng preedit nếu đang mở, KHÔNG đưa vào preedit và nuốt phím (eaten = 1). */
        if (r->flags & IME_FLAG_CONSUMED) {
            out->end = composing;
            out->eaten = 1;
            return;
        }
        if (key.kind == LC_KEY_CHAR && !word_end) {
            /* Engine nhận ký tự vào từ nhưng chưa biến đổi → đưa vào preedit. */
            if (cur->len + 1 >= LC_COMP_MAX) {
                /* C5 (audit): cur->len == LC_COMP_MAX thi ghi text[cur->len] la
                 * OOB intra-object (vuot mang text[256]) — commit nguyen chuoi
                 * hien tai va KHONG noi phim moi vao: eaten = 0 nen phim di tiep
                 * toi app, ky tu hien SAU phan vua commit (khong mat chu). */
                memcpy(out->text, cur->text, (size_t)cur->len * sizeof(uint32_t));
                out->text_len = cur->len;
                out->has_text = 1;
                out->end = 1;
                out->eaten = (key.kind == LC_KEY_CHAR) ? 0 : 1;
                out->reset_engine = 1;
                return;
            }
            memcpy(out->text, cur->text, cur->len * sizeof(uint32_t));
            out->text[cur->len] = key.ch;
            out->text_len = cur->len + 1;
            out->has_text = 1;
            out->eaten = 1;
            return;
        }
        if (key.kind == LC_KEY_BACKSPACE && composing) {
            /* Engine bỏ 1 ký tự chưa biến đổi → bỏ khỏi preedit. */
            memcpy(out->text, cur->text, (cur->len - 1) * sizeof(uint32_t));
            out->text_len = cur->len - 1;
            out->has_text = 1;
            out->end = out->text_len == 0;
            out->eaten = 1;
            return;
        }
        lc_plan_pass_through(cur, out);
        if (key.kind == LC_KEY_CTRL_SPACE) {
            /* R2-121: Ctrl+Space không khớp dự đoán → như chord B6: chốt preedit
             * (engine đã nhả từ — WORD_END), phím tới app và quên đuôi text (app có
             * thể chèn dấu cách / dời caret mà engine không thấy). Giữ preedit mở
             * thì phím kế tiếp lệch model của engine. */
            out->reset_engine = 1;
        }
        return;
    }

    if (r->action == IME_ACTION_REPLACE || r->action == IME_ACTION_RESTORE ||
        r->action == IME_ACTION_COMMIT) {
        size_t keep = cur->len;
        if (r->action != IME_ACTION_COMMIT) {
            size_t del = r->delete_count;
            if (del <= cur->len) {
                keep = cur->len - del;
            } else {
                keep = 0;
                out->delete_before = (uint16_t)(del - cur->len);
            }
        }
        size_t ins = r->insert_len;
        if (ins > IME_MAX_TEXT) ins = IME_MAX_TEXT;
        if (keep + ins > LC_COMP_MAX) {
            lc_plan_pass_through(cur, out);
            out->reset_engine = 1;
            return;
        }
        memcpy(out->text, cur->text, keep * sizeof(uint32_t));
        memcpy(out->text + keep, r->insert, ins * sizeof(uint32_t));
        out->text_len = keep + ins;
        out->has_text = 1;
        out->eaten = 1;
        out->cursor_left = r->cursor_left;

        uint32_t b = 0;
        if (word_end && legacy_boundary_char(key, &b) &&
            (r->action != IME_ACTION_REPLACE || b == '\n') &&
            out->text_len > 0 && out->text[out->text_len - 1] == b) {
            out->text_len -= 1;
            out->eaten = 0;
        }
        out->end = word_end || out->text_len == 0 || !out->eaten;
        return;
    }

    /* Action lạ (ABI mới hơn adapter): commit và để phím đi thẳng. */
    lc_plan_pass_through(cur, out);
}


static int plans_equal(const lc_plan *a, const lc_plan *b) {
    if (a->delete_before != b->delete_before || a->has_text != b->has_text ||
        a->end != b->end || a->eaten != b->eaten || a->reset_engine != b->reset_engine ||
        a->cursor_left != b->cursor_left) {
        return 0;
    }
    if (!a->has_text) return 1;
    return a->text_len == b->text_len &&
           memcmp(a->text, b->text, a->text_len * sizeof(uint32_t)) == 0;
}

/* Mọi tổ hợp (độ dài preedit × loại phím × action × cờ × delete_count × insert): planner dùng
 * chung (lc_plan_key → ime_compose_plan POLICY_LINUX) phải ra ĐÚNG kế hoạch của bản C cũ. */
static void test_sf3_plan_parity_with_legacy(void) {
    static const size_t lens[] = { 0, 1, 3, 254, 255 };
    static const lc_key keys[] = {
        { LC_KEY_CHAR, 'a' }, { LC_KEY_CHAR, ' ' }, { LC_KEY_CHAR, '.' }, { LC_KEY_CHAR, 0x1EC7 },
        { LC_KEY_BACKSPACE, 0 }, { LC_KEY_ESCAPE, 0 }, { LC_KEY_ENTER, 0 }, { LC_KEY_TAB, 0 },
        { LC_KEY_CTRL_SPACE, 0 }, { LC_KEY_OTHER, 0 },
    };
    static const uint32_t actions[] = { IME_ACTION_PASS, IME_ACTION_REPLACE, IME_ACTION_COMMIT,
                                        IME_ACTION_RESTORE, 99 };
    static const uint32_t flagset[] = { 0, IME_FLAG_WORD_END, IME_FLAG_CONSUMED,
                                        IME_FLAG_WORD_END | IME_FLAG_CONSUMED };
    static const uint16_t dels[] = { 0, 1, 3, 5, 300 };
    static const char *inserts[] = { "", "a", "x ", "x.", "\n", "\t", "sort ", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" };
    static const uint16_t lefts[] = { 0, 2 };
    unsigned n = 0, bad = 0;
    lc_comp cur;
    for (size_t li = 0; li < sizeof(lens) / sizeof(lens[0]); li++) {
        cur.len = lens[li];
        for (size_t i = 0; i < cur.len; i++) cur.text[i] = 'a' + (uint32_t)(i % 26);
        for (size_t ki = 0; ki < sizeof(keys) / sizeof(keys[0]); ki++)
        for (size_t ai = 0; ai < sizeof(actions) / sizeof(actions[0]); ai++)
        for (size_t fi = 0; fi < sizeof(flagset) / sizeof(flagset[0]); fi++)
        for (size_t di = 0; di < sizeof(dels) / sizeof(dels[0]); di++)
        for (size_t ii = 0; ii < sizeof(inserts) / sizeof(inserts[0]); ii++)
        for (size_t ci = 0; ci < sizeof(lefts) / sizeof(lefts[0]); ci++) {
            ime_result_v1 r;
            memset(&r, 0, sizeof(r));
            r.abi_version = IME_ABI_VERSION;
            r.action = actions[ai];
            r.flags = flagset[fi];
            r.delete_count = dels[di];
            r.cursor_left = lefts[ci];
            const char *ins = inserts[ii];
            size_t il = strlen(ins);
            for (size_t k = 0; k < il; k++) r.insert[k] = (unsigned char)ins[k];
            /* insert kết thúc bằng ký tự của phím (ranh giới) — ca Linux bỏ ra và forward. */
            if (il > 0 && il < IME_MAX_TEXT && keys[ki].kind == LC_KEY_CHAR && (ii == 2 || ii == 3)) {
                r.insert[il - 1] = keys[ki].ch;
            }
            r.insert_len = (uint16_t)il;
            lc_plan got, want;
            lc_plan_key(&cur, keys[ki], &r, &got);
            legacy_plan_key(&cur, keys[ki], &r, &want);
            n++;
            if (!plans_equal(&got, &want)) {
                if (bad < 5) {
                    fprintf(stderr, "SF-3 lệch: len=%zu key=%d/%u action=%u flags=%u del=%u ins=%zu "
                            "left=%u | got end=%d eaten=%d reset=%d text=%zu | want end=%d eaten=%d reset=%d text=%zu\n",
                            cur.len, (int)keys[ki].kind, (unsigned)keys[ki].ch, (unsigned)r.action,
                            (unsigned)r.flags, (unsigned)r.delete_count, il, (unsigned)r.cursor_left,
                            got.end, got.eaten, got.reset_engine, got.text_len,
                            want.end, want.eaten, want.reset_engine, want.text_len);
                }
                bad++;
            }
        }
    }
    printf("SF-3 parity lc_plan_key: %u tổ hợp, %u lệch\n", n, bad);
    assert(n > 50000);
    assert(bad == 0);
}

/* SF-3 (R2-171/E78): quyết định reset của session (IME_SCMD_ENGINE_RESET) qua shadow phải khớp
 * luật cũ của IBus/Fcitx5: reset toolkit < 250 ms sau Enter, không phím/focus xen giữa ⇒ giữ mốc. */
static void test_session_shadow_reset_parity(void) {
    lc_session_shadow s;
    memset(&s, 0, sizeof(s));
    assert(lc_session_shadow_init_force(&s) == 0);
    /* Mô hình cũ: enter_at = mốc Enter (0 = không), xoá khi phím khác / đổi focus. */
    uint64_t enter_at = 0;
#define KEY(enter, t) do { s.now_override_ms = (t); \
        lc_session_shadow_key_down(&s, LC_MOD_NONE, 0, (enter), 0, 0, 0, 0); \
        enter_at = (enter) ? (t) : 0; } while (0)
#define TOOLKIT_RESET(t) do { s.now_override_ms = (t); \
        lc_session_shadow_toolkit_reset(&s, enter_at != 0 && (t) - enter_at < 250); } while (0)
    KEY(1, 1000); TOOLKIT_RESET(1010);           /* ngay sau Enter: giữ mốc */
    TOOLKIT_RESET(1020);                         /* reset thứ hai trong cửa sổ: vẫn giữ */
    TOOLKIT_RESET(1300);                         /* quá 250 ms: xoá sạch */
    KEY(1, 2000); KEY(0, 2005); TOOLKIT_RESET(2010); /* phím khác xen giữa */
    KEY(1, 3000); s.now_override_ms = 3005; lc_session_shadow_reset(&s); enter_at = 0;
    TOOLKIT_RESET(3010);                         /* mất focus xen giữa */
    TOOLKIT_RESET(4000);                         /* không có Enter */
    KEY(1, 5000); TOOLKIT_RESET(5000 + 249);     /* biên 249 ms */
#undef KEY
#undef TOOLKIT_RESET
    assert(s.mismatches == 0);
    /* Shadow phải BẮT lệch: mã cũ nói giữ mốc khi session không. */
    s.now_override_ms = 9000;
    lc_session_shadow_toolkit_reset(&s, 1);
    assert(s.mismatches == 1);
    lc_session_shadow_free(&s);
}

static void test_engine_ctrl_tap(void) {
    ime_instance *inst = engine_with(NO_CAPS);
    doc d;
    memset(&d, 0, sizeof(d));

    /* Type tunx -> preedit becomes tũn */
    type_str(&d, inst, "tunx");
    char got[64];
    doc_text(&d, got, sizeof(got));
    assert(strcmp(got, "t\xc5\xa9n") == 0);
    assert(d.comp.len > 0);

    /* Ctrl tap locks valid syllable */
    ime_result_v1 r;
    assert(lc_engine_ctrl_tap(inst, &r) == IME_OK);
    lc_plan plan;
    lc_plan_key(&d.comp, (lc_key){ LC_KEY_OTHER, 0 }, &r, &plan);
    assert(plan.end == 1);
    apply_plan(&d, inst, &plan);
    assert(d.comp.len == 0);

    /* Type s -> tũns */
    type_str(&d, inst, "s");
    doc_text(&d, got, sizeof(got));
    assert(strcmp(got, "t\xc5\xa9ns") == 0);

    /* Reset for second scenario: sort -> sỏt */
    d.committed_len = 0;
    d.comp.len = 0;
    ime_reset(inst);
    type_str(&d, inst, "sort");
    doc_text(&d, got, sizeof(got));
    assert(strcmp(got, "s\xe1\xbb\x8ft") == 0);

    /* Ctrl tap restores sort */
    assert(lc_engine_ctrl_tap(inst, &r) == IME_OK);
    assert(r.action == IME_ACTION_RESTORE);
    lc_plan_key(&d.comp, (lc_key){ LC_KEY_OTHER, 0 }, &r, &plan);
    assert(plan.has_text == 1);
    apply_plan(&d, inst, &plan);
    doc_text(&d, got, sizeof(got));
    assert(strcmp(got, "sort") == 0);

    ime_instance_free(inst);
}

static void test_mode_en_prediction_passthrough(void) {
    const char *cfg = "{\"config_version\":1,\"auto_capitalize\":false,"
                      "\"suggest\":{\"enabled\":true,\"trigger\":\"tab_and_space\"}}";
    ime_instance *inst = engine_with(cfg);
    lc_config_state st;
    memset(&st, 0, sizeof(st));
    st.suggest_enabled = 1;

    ime_context_v1 ctx;
    memset(&ctx, 0, sizeof(ctx));
    ctx.abi_version = IME_ABI_VERSION;
    ctx.enabled = 0; /* Mode EN */
    ctx.field_role = IME_FIELD_BODY;
    ctx.caps = IME_CAP_PREEDIT | IME_CAP_SELECTION;
    ctx.hint = -1;
    assert(ime_set_context(inst, &ctx) == IME_OK);

    /* Type devel */
    const char *p = "devel";
    while (*p) {
        ime_key_v1 k;
        memset(&k, 0, sizeof(k));
        k.abi_version = IME_ABI_VERSION;
        k.vk = (uint32_t)(*p);
        k.ch = (uint32_t)(*p);
        k.key_down = 1;
        ime_result_v1 r;
        assert(ime_key(inst, &k, &r) == IME_OK);
        assert(r.action == IME_ACTION_PASS);
        p++;
    }

    /* Peek completion */
    char ghost[512];
    int gn = ime_peek_completion(inst, ghost, sizeof(ghost));
    assert(gn > 0);
    assert(strcmp(ghost, "op") == 0);

    /* Press Tab to accept completion */
    ime_key_v1 k;
    memset(&k, 0, sizeof(k));
    k.abi_version = IME_ABI_VERSION;
    k.vk = 0x09; /* Tab */
    k.ch = 0x09;
    k.key_down = 1;
    ime_result_v1 r;
    assert(ime_key(inst, &k, &r) == IME_OK);
    assert(r.action == IME_ACTION_REPLACE);
    assert(r.delete_count == 5);
    /* Insert is "develop" */
    char word[64];
    size_t o = 0;
    for (size_t i = 0; i < r.insert_len; ++i) word[o++] = (char)r.insert[i];
    word[o] = '\0';
    assert(strcmp(word, "develop") == 0);

    ime_instance_free(inst);
}

static void test_e62_math_plan(void) {
    /* 1) Backtick arm chord: ACTION_PASS | IME_FLAG_CONSUMED → eaten=1, không đưa vào preedit. */
    lc_comp cur;
    memset(&cur, 0, sizeof(cur));
    ime_result_v1 r;
    memset(&r, 0, sizeof(r));
    r.abi_version = IME_ABI_VERSION;
    r.action = IME_ACTION_PASS;
    r.flags = IME_FLAG_CONSUMED;
    lc_plan plan;
    lc_plan_key(&cur, (lc_key){ LC_KEY_CHAR, '`' }, &r, &plan);
    assert(plan.eaten == 1 && plan.has_text == 0 && plan.end == 0);

    /* 2) `\frac{}{}`: ACTION_REPLACE giữ nguyên cursor_left = 3. */
    memset(&r, 0, sizeof(r));
    r.abi_version = IME_ABI_VERSION;
    r.action = IME_ACTION_REPLACE;
    r.flags = IME_FLAG_CONSUMED | IME_FLAG_WORD_END;
    r.cursor_left = 3;
    const char *frac = "\\frac{}{}";
    r.insert_len = (uint16_t)strlen(frac);
    for (size_t i = 0; i < r.insert_len; i++) r.insert[i] = (uint32_t)(unsigned char)frac[i];
    lc_plan_key(&cur, (lc_key){ LC_KEY_CHAR, '/' }, &r, &plan);
    assert(plan.eaten == 1 && plan.end == 1 && plan.cursor_left == 3);
    assert(plan.text_len == strlen(frac));

    /* 3) `\can` + Enter: ACTION_REPLACE có `\n` cuối → bỏ `\n`, eaten=0 trả Enter thật. */
    memset(&r, 0, sizeof(r));
    r.abi_version = IME_ABI_VERSION;
    r.action = IME_ACTION_REPLACE;
    r.flags = IME_FLAG_CONSUMED | IME_FLAG_WORD_END;
    r.insert[0] = 0x221Au; /* √ */
    r.insert[1] = 'x';
    r.insert[2] = '\n';
    r.insert_len = 3;
    lc_plan_key(&cur, (lc_key){ LC_KEY_ENTER, 0 }, &r, &plan);
    assert(plan.eaten == 0 && plan.end == 1 && plan.text_len == 2);
}

int main(void) {
    test_classify();
    test_typing();
    test_composition_holds_whole_word();
    test_plan_edge_cases();
    test_modifier_toggle();
    test_assist_idle_arms();
    test_modifier_ctrl_tap();
    test_session_shadow_parity();
    test_session_shadow_reset_parity();
    test_toggle_corpus();
    test_sf3_plan_parity_with_legacy();
    test_engine_ctrl_tap();
    test_mode_en_prediction_passthrough();
    test_config_sync();
    test_state_file();
    test_assist_state_file();
    test_assist_auto_fix_sentence();
    test_assist_mode_semantics();
    test_assist_auto_fix_spacing_punct();
    test_assist_l6_auto_capitalize_mode2();
    test_assist_hotkey_explicit_preview();
    test_transcode_hotkey();
    test_transcode_selection_slice();
    test_macro_when_vi_off();
    test_e62_math_plan();
    test_find_settings_binary();
    printf("test_compose: OK\n");
    return 0;
}
