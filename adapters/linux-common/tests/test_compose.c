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

static lc_plan press(doc *d, ime_instance *inst, uint32_t vk, uint32_t ch) {
    lc_key key = lc_key_classify(vk, ch);
    ime_key_v1 k;
    lc_key_to_ime(key, vk, 0, &k);
    ime_result_v1 r;
    assert(ime_key(inst, &k, &r) == IME_OK);
    lc_plan plan;
    lc_plan_key(&d->comp, key, &r, &plan);

    if (plan.delete_before) {
        assert(plan.delete_before <= d->committed_len);
        d->committed_len -= plan.delete_before;
    }
    if (plan.end) {
        size_t n = 0;
        const uint32_t *t = lc_plan_commit_text(&d->comp, &plan, &n);
        memcpy(d->committed + d->committed_len, t, n * sizeof(uint32_t));
        d->committed_len += n;
    }
    lc_comp_apply(&d->comp, &plan);
    if (plan.reset_engine) ime_reset(inst);
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
    assert(lc_key_classify(0, 0x7F).kind == LC_KEY_OTHER);
    assert(lc_key_engine_ch((lc_key){ LC_KEY_ENTER, 0 }) == 0);
}

static void test_typing(void) {
    expect(NO_CAPS, "duocj ", "được ");
    expect(NO_CAPS, "Vieetj Nam", "Việt Nam");
    expect(NO_CAPS, "hello world ", "hello world ");
    expect(NO_CAPS, "chaof banj\n", "chào bạn\n");
    expect(NO_CAPS, "duocj\b", "đươc");
    expect(NO_CAPS, "ab\b\b\b", "");
    expect(NO_CAPS, "ab \b\b", "a");
    expect(NO_CAPS, "duocj\x1b", "duocj");
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
    assert(lc_config_sync(inst, &st, path) == 1);
    assert(lc_config_sync(inst, &st, path) == 0 && "không đổi → không nạp lại");

    doc d;
    memset(&d, 0, sizeof(d));
    type_str(&d, inst, "a1 ");
    char got[64];
    doc_text(&d, got, sizeof(got));
    assert(strcmp(got, "\xc3\xa1 ") == 0 && "VNI từ config.json");
    assert(lc_config_sync(inst, &st, "/nonexistent/config.json") == 0);
    assert(st.allow_macro_when_vi_off == 0);

    /* Đổi file → nạp lại và đọc cờ gõ tắt khi tắt VN. */
    const char *with_flag = "{\"config_version\":1,\"method\":\"vni\",\"allow_macro_when_vi_off\":true}";
    FILE *f = fopen(path, "wb");
    assert(f && fwrite(with_flag, 1, strlen(with_flag), f) == strlen(with_flag));
    fclose(f);
    assert(lc_config_sync(inst, &st, path) == 1);
    assert(st.allow_macro_when_vi_off == 1);

    /* Config hỏng: engine giữ config cũ → cờ cũng giữ nguyên. */
    f = fopen(path, "wb");
    assert(f && fwrite("{oops", 1, 5, f) == 5);
    fclose(f);
    assert(lc_config_sync(inst, &st, path) == 1);
    assert(st.allow_macro_when_vi_off == 1);
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

int main(void) {
    test_classify();
    test_typing();
    test_composition_holds_whole_word();
    test_plan_edge_cases();
    test_modifier_toggle();
    test_config_sync();
    test_state_file();
    test_macro_when_vi_off();
    test_find_settings_binary();
    printf("test_compose: OK\n");
    return 0;
}
