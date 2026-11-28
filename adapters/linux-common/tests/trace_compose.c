/* trace_compose.c — SF-1d: ghi vết vàng adapter Linux (C11, P0-6 §6.1)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Chạy engine Rust THẬT (`libtextvn_ffi`) cùng `lc_plan_key` + `lc_edit_batch` trên
 * luồng phím do Rust xuất (`textvn-cli replay … --adapter linux --emit-keys <dir>`, SF-1c) —
 * KHÔNG viết lại bộ đọc `.keys` bằng C (G22). Mỗi thao tác được chuẩn hoá thành đúng 5 lệnh
 * của SF-1c/SF-1e (`delete_before`, `commit`, `set_preedit`, `forward_key`, `eat_key`),
 * một dòng JSONL `{"case","step","cmd","text","n"}` ⇒ `corpus/traces/linux/<rel>.jsonl`.
 *
 * Biến môi trường:
 *   TEXTVN_EMIT_KEYS=<dir>   (tuỳ chọn) quét đệ quy *.json, kiểm tra tất định 2 lần + bất biến
 *   TEXTVN_TRACE_OUT=<dir>   (tuỳ chọn, cần TEXTVN_EMIT_KEYS) ghi <dir>/<case>.jsonl
 * Không đặt TEXTVN_EMIT_KEYS ⇒ chạy bộ self-test nội tại (ASan/UBSan/unsigned-char).
 * G8: chỉ xử lý corpus test cố định, không log nội dung gõ của người dùng.
 */

#define _POSIX_C_SOURCE 200809L

#include "lc_compose.h"
#include "lc_edit_batch.h"
#include "linux_common.h"
#include "textvn_ffi.h"

#include <assert.h>
#include <dirent.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

/* ---- Dynamic string buffer ---- */

typedef struct strbuf {
    char  *data;
    size_t len;
    size_t cap;
} strbuf;

static void sb_init(strbuf *b) {
    b->data = NULL;
    b->len = 0;
    b->cap = 0;
}

static void sb_free(strbuf *b) {
    free(b->data);
    b->data = NULL;
    b->len = 0;
    b->cap = 0;
}

static int sb_reserve(strbuf *b, size_t extra) {
    if (extra > SIZE_MAX - b->len - 1) return 0;
    size_t need = b->len + extra + 1;
    if (need <= b->cap) return 1;
    size_t ncap = b->cap ? b->cap : 256u;
    while (ncap < need) {
        if (ncap > SIZE_MAX / 2) {
            ncap = need;
            break;
        }
        ncap *= 2;
    }
    char *p = (char *)realloc(b->data, ncap);
    if (!p) return 0;
    b->data = p;
    b->cap = ncap;
    return 1;
}

static int sb_push_char(strbuf *b, char c) {
    if (!sb_reserve(b, 1u)) return 0;
    b->data[b->len++] = c;
    b->data[b->len] = '\0';
    return 1;
}

static int sb_push_n(strbuf *b, const char *s, size_t n) {
    if (!sb_reserve(b, n)) return 0;
    if (n > 0) memcpy(b->data + b->len, s, n);
    b->len += n;
    b->data[b->len] = '\0';
    return 1;
}

static int sb_push_str(strbuf *b, const char *s) {
    return sb_push_n(b, s, strlen(s));
}

static int sb_push_u32(strbuf *b, uint32_t v) {
    char tmp[32];
    int n = snprintf(tmp, sizeof(tmp), "%u", (unsigned)v);
    if (n <= 0 || (size_t)n >= sizeof(tmp)) return 0;
    return sb_push_n(b, tmp, (size_t)n);
}

static int sb_push_utf8(strbuf *b, uint32_t cp) {
    char u[4];
    size_t k = 0;
    if (cp <= 0x7Fu) {
        u[k++] = (char)cp;
    } else if (cp <= 0x7FFu) {
        u[k++] = (char)(0xC0u | (cp >> 6));
        u[k++] = (char)(0x80u | (cp & 0x3Fu));
    } else if (cp <= 0xFFFFu) {
        u[k++] = (char)(0xE0u | (cp >> 12));
        u[k++] = (char)(0x80u | ((cp >> 6) & 0x3Fu));
        u[k++] = (char)(0x80u | (cp & 0x3Fu));
    } else if (cp <= 0x10FFFFu) {
        u[k++] = (char)(0xF0u | (cp >> 18));
        u[k++] = (char)(0x80u | ((cp >> 12) & 0x3Fu));
        u[k++] = (char)(0x80u | ((cp >> 6) & 0x3Fu));
        u[k++] = (char)(0x80u | (cp & 0x3Fu));
    } else {
        return 0;
    }
    return sb_push_n(b, u, k);
}

static int sb_push_json_escaped(strbuf *b, const char *s, size_t len) {
    static const char hex[] = "0123456789abcdef";
    for (size_t i = 0; i < len; ++i) {
        unsigned char c = (unsigned char)s[i];
        switch (c) {
        case '"':
            if (!sb_push_n(b, "\\\"", 2u)) return 0;
            break;
        case '\\':
            if (!sb_push_n(b, "\\\\", 2u)) return 0;
            break;
        case '\n':
            if (!sb_push_n(b, "\\n", 2u)) return 0;
            break;
        case '\r':
            if (!sb_push_n(b, "\\r", 2u)) return 0;
            break;
        case '\t':
            if (!sb_push_n(b, "\\t", 2u)) return 0;
            break;
        default:
            if (c < 0x20u) {
                char esc[6] = { '\\', 'u', '0', '0', hex[c >> 4], hex[c & 0xFu] };
                if (!sb_push_n(b, esc, 6u)) return 0;
            } else {
                if (!sb_push_char(b, (char)c)) return 0;
            }
            break;
        }
    }
    return 1;
}

static char *xstrdup(const char *s) {
    size_t n = strlen(s);
    char *p = (char *)malloc(n + 1u);
    assert(p != NULL);
    memcpy(p, s, n + 1u);
    return p;
}

/* ---- Strict compact JSON reader for --emit-keys ---- */

typedef struct json_cursor {
    const char *p;
    const char *end;
} json_cursor;

static void j_skip_ws(json_cursor *jc) {
    while (jc->p < jc->end) {
        unsigned char c = (unsigned char)*jc->p;
        if (c == ' ' || c == '\t' || c == '\n' || c == '\r') {
            jc->p++;
        } else {
            break;
        }
    }
}

static int hex_val(unsigned char c, uint32_t *out) {
    if (c >= '0' && c <= '9') { *out = (uint32_t)(c - '0'); return 1; }
    if (c >= 'a' && c <= 'f') { *out = 10u + (uint32_t)(c - 'a'); return 1; }
    if (c >= 'A' && c <= 'F') { *out = 10u + (uint32_t)(c - 'A'); return 1; }
    return 0;
}

static int j_read_hex4(json_cursor *jc, uint32_t *out) {
    if ((size_t)(jc->end - jc->p) < 4u) return 0;
    uint32_t v = 0;
    for (size_t i = 0; i < 4u; ++i) {
        uint32_t d = 0;
        if (!hex_val((unsigned char)jc->p[i], &d)) return 0;
        v = (v << 4) | d;
    }
    jc->p += 4;
    *out = v;
    return 1;
}

static int j_expect(json_cursor *jc, char ch) {
    j_skip_ws(jc);
    if (jc->p >= jc->end || *jc->p != ch) return 0;
    jc->p++;
    return 1;
}

static int j_parse_string(json_cursor *jc, char **out_str) {
    j_skip_ws(jc);
    if (jc->p >= jc->end || *jc->p != '"') return 0;
    jc->p++;

    strbuf sb;
    sb_init(&sb);
    if (!sb_reserve(&sb, 16u)) {
        sb_free(&sb);
        return 0;
    }
    sb.data[0] = '\0';

    while (jc->p < jc->end) {
        unsigned char c = (unsigned char)*jc->p++;
        if (c == '"') {
            *out_str = sb.data;
            return 1;
        }
        if (c < 0x20u) {
            sb_free(&sb);
            return 0;
        }
        if (c == '\\') {
            if (jc->p >= jc->end) {
                sb_free(&sb);
                return 0;
            }
            unsigned char e = (unsigned char)*jc->p++;
            switch (e) {
            case '"':  if (!sb_push_char(&sb, '"'))  { sb_free(&sb); return 0; } break;
            case '\\': if (!sb_push_char(&sb, '\\')) { sb_free(&sb); return 0; } break;
            case '/':  if (!sb_push_char(&sb, '/'))  { sb_free(&sb); return 0; } break;
            case 'b':  if (!sb_push_char(&sb, '\b')) { sb_free(&sb); return 0; } break;
            case 'f':  if (!sb_push_char(&sb, '\f')) { sb_free(&sb); return 0; } break;
            case 'n':  if (!sb_push_char(&sb, '\n')) { sb_free(&sb); return 0; } break;
            case 'r':  if (!sb_push_char(&sb, '\r')) { sb_free(&sb); return 0; } break;
            case 't':  if (!sb_push_char(&sb, '\t')) { sb_free(&sb); return 0; } break;
            case 'u': {
                uint32_t hi = 0;
                if (!j_read_hex4(jc, &hi)) {
                    sb_free(&sb);
                    return 0;
                }
                if (hi >= 0xD800u && hi <= 0xDBFFu) {
                    if ((size_t)(jc->end - jc->p) < 2u || jc->p[0] != '\\' || jc->p[1] != 'u') {
                        sb_free(&sb);
                        return 0;
                    }
                    jc->p += 2;
                    uint32_t lo = 0;
                    if (!j_read_hex4(jc, &lo) || lo < 0xDC00u || lo > 0xDFFFu) {
                        sb_free(&sb);
                        return 0;
                    }
                    uint32_t cp = 0x10000u + (((hi - 0xD800u) << 10) | (lo - 0xDC00u));
                    if (!sb_push_utf8(&sb, cp)) {
                        sb_free(&sb);
                        return 0;
                    }
                } else {
                    if (!sb_push_utf8(&sb, hi)) {
                        sb_free(&sb);
                        return 0;
                    }
                }
                break;
            }
            default:
                sb_free(&sb);
                return 0;
            }
        } else {
            if (!sb_push_char(&sb, (char)c)) {
                sb_free(&sb);
                return 0;
            }
        }
    }
    sb_free(&sb);
    return 0;
}

static int j_parse_u32(json_cursor *jc, uint32_t *out) {
    j_skip_ws(jc);
    if (jc->p >= jc->end) return 0;
    unsigned char first = (unsigned char)*jc->p;
    if (first < '0' || first > '9') return 0;
    uint64_t val = 0;
    while (jc->p < jc->end) {
        unsigned char c = (unsigned char)*jc->p;
        if (c < '0' || c > '9') break;
        val = val * 10u + (uint64_t)(c - '0');
        if (val > UINT32_MAX) return 0;
        jc->p++;
    }
    *out = (uint32_t)val;
    return 1;
}

static int j_parse_bool(json_cursor *jc, int *out) {
    j_skip_ws(jc);
    if ((size_t)(jc->end - jc->p) >= 4u && memcmp(jc->p, "true", 4u) == 0) {
        jc->p += 4;
        *out = 1;
        return 1;
    }
    if ((size_t)(jc->end - jc->p) >= 5u && memcmp(jc->p, "false", 5u) == 0) {
        jc->p += 5;
        *out = 0;
        return 1;
    }
    return 0;
}

static int j_skip_value(json_cursor *jc);

static int j_skip_array(json_cursor *jc) {
    if (!j_expect(jc, '[')) return 0;
    j_skip_ws(jc);
    if (jc->p < jc->end && *jc->p == ']') {
        jc->p++;
        return 1;
    }
    for (;;) {
        if (!j_skip_value(jc)) return 0;
        j_skip_ws(jc);
        if (jc->p >= jc->end) return 0;
        if (*jc->p == ',') {
            jc->p++;
            continue;
        }
        if (*jc->p == ']') {
            jc->p++;
            return 1;
        }
        return 0;
    }
}

static int j_skip_object(json_cursor *jc) {
    if (!j_expect(jc, '{')) return 0;
    j_skip_ws(jc);
    if (jc->p < jc->end && *jc->p == '}') {
        jc->p++;
        return 1;
    }
    for (;;) {
        char *k = NULL;
        if (!j_parse_string(jc, &k)) return 0;
        free(k);
        if (!j_expect(jc, ':')) return 0;
        if (!j_skip_value(jc)) return 0;
        j_skip_ws(jc);
        if (jc->p >= jc->end) return 0;
        if (*jc->p == ',') {
            jc->p++;
            continue;
        }
        if (*jc->p == '}') {
            jc->p++;
            return 1;
        }
        return 0;
    }
}

static int j_skip_number(json_cursor *jc) {
    j_skip_ws(jc);
    if (jc->p >= jc->end) return 0;
    if (*jc->p == '-') jc->p++;
    if (jc->p >= jc->end || (unsigned char)*jc->p < '0' || (unsigned char)*jc->p > '9') return 0;
    while (jc->p < jc->end && (unsigned char)*jc->p >= '0' && (unsigned char)*jc->p <= '9') {
        jc->p++;
    }
    if (jc->p < jc->end && *jc->p == '.') {
        jc->p++;
        if (jc->p >= jc->end || (unsigned char)*jc->p < '0' || (unsigned char)*jc->p > '9') return 0;
        while (jc->p < jc->end && (unsigned char)*jc->p >= '0' && (unsigned char)*jc->p <= '9') {
            jc->p++;
        }
    }
    if (jc->p < jc->end && (*jc->p == 'e' || *jc->p == 'E')) {
        jc->p++;
        if (jc->p < jc->end && (*jc->p == '+' || *jc->p == '-')) jc->p++;
        if (jc->p >= jc->end || (unsigned char)*jc->p < '0' || (unsigned char)*jc->p > '9') return 0;
        while (jc->p < jc->end && (unsigned char)*jc->p >= '0' && (unsigned char)*jc->p <= '9') {
            jc->p++;
        }
    }
    return 1;
}

static int j_skip_value(json_cursor *jc) {
    j_skip_ws(jc);
    if (jc->p >= jc->end) return 0;
    unsigned char c = (unsigned char)*jc->p;
    if (c == '"') {
        char *tmp = NULL;
        if (!j_parse_string(jc, &tmp)) return 0;
        free(tmp);
        return 1;
    }
    if (c == '{') return j_skip_object(jc);
    if (c == '[') return j_skip_array(jc);
    if (c == 't' || c == 'f') {
        int b = 0;
        return j_parse_bool(jc, &b);
    }
    if (c == 'n') {
        if ((size_t)(jc->end - jc->p) >= 4u && memcmp(jc->p, "null", 4u) == 0) {
            jc->p += 4;
            return 1;
        }
        return 0;
    }
    return j_skip_number(jc);
}

/* ---- Parsed emit-keys model ---- */

typedef struct emit_app {
    char    *bundle;
    uint32_t field_role;
    uint32_t caps;
    int      secure;
    int      enabled;
} emit_app;

typedef enum emit_event_kind {
    EV_CONFIG = 0,
    EV_CONTEXT,
    EV_RESET,
    EV_ENGINE_NEW,
    EV_KEY,
    EV_SKIP
} emit_event_kind;

typedef struct emit_event {
    emit_event_kind kind;
    uint32_t        step;
    char           *config_json;
    emit_app        ctx;
    uint32_t        vk;
    uint32_t        ch;
    uint32_t        mods;
    uint8_t         down;
    int             injected;
} emit_event;

typedef struct emit_case {
    char       *case_rel;
    char       *init_config_json;
    emit_app    init_app;
    emit_event *events;
    size_t      event_count;
    size_t      event_cap;
    uint32_t    key_step_count;
} emit_case;

static void free_app(emit_app *a) {
    free(a->bundle);
    a->bundle = NULL;
}

static void free_case(emit_case *ec) {
    free(ec->case_rel);
    free(ec->init_config_json);
    free_app(&ec->init_app);
    for (size_t i = 0; i < ec->event_count; ++i) {
        free(ec->events[i].config_json);
        free_app(&ec->events[i].ctx);
    }
    free(ec->events);
    memset(ec, 0, sizeof(*ec));
}

static int parse_app_object(json_cursor *jc, emit_app *app) {
    memset(app, 0, sizeof(*app));
    app->enabled = 1;
    if (!j_expect(jc, '{')) return 0;
    j_skip_ws(jc);
    if (jc->p < jc->end && *jc->p == '}') {
        jc->p++;
        return 1;
    }
    for (;;) {
        char *key = NULL;
        if (!j_parse_string(jc, &key)) {
            free_app(app);
            return 0;
        }
        if (!j_expect(jc, ':')) {
            free(key);
            free_app(app);
            return 0;
        }
        int ok = 1;
        if (strcmp(key, "bundle") == 0) {
            free(app->bundle);
            app->bundle = NULL;
            ok = j_parse_string(jc, &app->bundle);
        } else if (strcmp(key, "field_role") == 0) {
            ok = j_parse_u32(jc, &app->field_role);
        } else if (strcmp(key, "caps") == 0) {
            ok = j_parse_u32(jc, &app->caps);
        } else if (strcmp(key, "secure") == 0) {
            ok = j_parse_bool(jc, &app->secure);
        } else if (strcmp(key, "enabled") == 0) {
            ok = j_parse_bool(jc, &app->enabled);
        } else {
            ok = j_skip_value(jc);
        }
        free(key);
        if (!ok) {
            free_app(app);
            return 0;
        }
        j_skip_ws(jc);
        if (jc->p >= jc->end) {
            free_app(app);
            return 0;
        }
        if (*jc->p == ',') {
            jc->p++;
            continue;
        }
        if (*jc->p == '}') {
            jc->p++;
            return 1;
        }
        free_app(app);
        return 0;
    }
}

static int parse_event_object(json_cursor *jc, emit_event *ev) {
    memset(ev, 0, sizeof(*ev));
    ev->kind = EV_SKIP;
    ev->ctx.enabled = 1;
    if (!j_expect(jc, '{')) return 0;
    j_skip_ws(jc);
    if (jc->p < jc->end && *jc->p == '}') {
        jc->p++;
        return 1;
    }

    char *kind_str = NULL;
    for (;;) {
        char *key = NULL;
        if (!j_parse_string(jc, &key)) goto fail;
        if (!j_expect(jc, ':')) {
            free(key);
            goto fail;
        }
        int ok = 1;
        if (strcmp(key, "kind") == 0) {
            free(kind_str);
            kind_str = NULL;
            ok = j_parse_string(jc, &kind_str);
        } else if (strcmp(key, "step") == 0) {
            ok = j_parse_u32(jc, &ev->step);
        } else if (strcmp(key, "config_json") == 0) {
            free(ev->config_json);
            ev->config_json = NULL;
            ok = j_parse_string(jc, &ev->config_json);
        } else if (strcmp(key, "bundle") == 0) {
            free(ev->ctx.bundle);
            ev->ctx.bundle = NULL;
            ok = j_parse_string(jc, &ev->ctx.bundle);
        } else if (strcmp(key, "field_role") == 0) {
            ok = j_parse_u32(jc, &ev->ctx.field_role);
        } else if (strcmp(key, "caps") == 0) {
            ok = j_parse_u32(jc, &ev->ctx.caps);
        } else if (strcmp(key, "secure") == 0) {
            ok = j_parse_bool(jc, &ev->ctx.secure);
        } else if (strcmp(key, "enabled") == 0) {
            ok = j_parse_bool(jc, &ev->ctx.enabled);
        } else if (strcmp(key, "vk") == 0) {
            ok = j_parse_u32(jc, &ev->vk);
        } else if (strcmp(key, "ch") == 0) {
            ok = j_parse_u32(jc, &ev->ch);
        } else if (strcmp(key, "mods") == 0) {
            ok = j_parse_u32(jc, &ev->mods);
        } else if (strcmp(key, "down") == 0) {
            uint32_t d = 0;
            ok = j_parse_u32(jc, &d);
            ev->down = d ? 1u : 0u;
        } else if (strcmp(key, "injected") == 0) {
            ok = j_parse_bool(jc, &ev->injected);
        } else {
            ok = j_skip_value(jc);
        }
        free(key);
        if (!ok) goto fail;

        j_skip_ws(jc);
        if (jc->p >= jc->end) goto fail;
        if (*jc->p == ',') {
            jc->p++;
            continue;
        }
        if (*jc->p == '}') {
            jc->p++;
            break;
        }
        goto fail;
    }

    if (!kind_str) goto fail;
    if (strcmp(kind_str, "config") == 0) {
        ev->kind = EV_CONFIG;
    } else if (strcmp(kind_str, "context") == 0) {
        ev->kind = EV_CONTEXT;
    } else if (strcmp(kind_str, "reset") == 0) {
        ev->kind = EV_RESET;
    } else if (strcmp(kind_str, "engine_new") == 0) {
        ev->kind = EV_ENGINE_NEW;
    } else if (strcmp(kind_str, "key") == 0) {
        ev->kind = EV_KEY;
    } else if (strcmp(kind_str, "expect") == 0 ||
               strcmp(kind_str, "expect_preedit") == 0 ||
               strcmp(kind_str, "expect_cursor") == 0) {
        ev->kind = EV_SKIP;
    } else {
        goto fail;
    }
    free(kind_str);
    return 1;

fail:
    free(kind_str);
    free(ev->config_json);
    ev->config_json = NULL;
    free_app(&ev->ctx);
    return 0;
}

static int parse_emit_case(const char *json, size_t len, emit_case *out) {
    memset(out, 0, sizeof(*out));
    out->init_app.enabled = 1;

    json_cursor jc = { json, json + len };
    if (!j_expect(&jc, '{')) goto fail;
    j_skip_ws(&jc);
    if (jc.p < jc.end && *jc.p == '}') goto fail;

    for (;;) {
        char *key = NULL;
        if (!j_parse_string(&jc, &key)) goto fail;
        if (!j_expect(&jc, ':')) {
            free(key);
            goto fail;
        }
        int ok = 1;
        if (strcmp(key, "case") == 0) {
            free(out->case_rel);
            out->case_rel = NULL;
            ok = j_parse_string(&jc, &out->case_rel);
        } else if (strcmp(key, "config_json") == 0) {
            free(out->init_config_json);
            out->init_config_json = NULL;
            ok = j_parse_string(&jc, &out->init_config_json);
        } else if (strcmp(key, "app") == 0) {
            free_app(&out->init_app);
            ok = parse_app_object(&jc, &out->init_app);
        } else if (strcmp(key, "events") == 0) {
            ok = j_expect(&jc, '[');
            if (ok) {
                j_skip_ws(&jc);
                if (jc.p < jc.end && *jc.p == ']') {
                    jc.p++;
                } else {
                    for (;;) {
                        if (out->event_count == out->event_cap) {
                            size_t ncap = out->event_cap ? out->event_cap * 2u : 32u;
                            emit_event *np = (emit_event *)realloc(
                                out->events, ncap * sizeof(emit_event));
                            if (!np) {
                                ok = 0;
                                break;
                            }
                            out->events = np;
                            out->event_cap = ncap;
                        }
                        emit_event ev;
                        if (!parse_event_object(&jc, &ev)) {
                            ok = 0;
                            break;
                        }
                        if (ev.kind == EV_KEY && ev.step + 1u > out->key_step_count) {
                            out->key_step_count = ev.step + 1u;
                        }
                        out->events[out->event_count++] = ev;
                        j_skip_ws(&jc);
                        if (jc.p >= jc.end) {
                            ok = 0;
                            break;
                        }
                        if (*jc.p == ',') {
                            jc.p++;
                            continue;
                        }
                        if (*jc.p == ']') {
                            jc.p++;
                            break;
                        }
                        ok = 0;
                        break;
                    }
                }
            }
        } else {
            ok = j_skip_value(&jc);
        }
        free(key);
        if (!ok) goto fail;

        j_skip_ws(&jc);
        if (jc.p >= jc.end) goto fail;
        if (*jc.p == ',') {
            jc.p++;
            continue;
        }
        if (*jc.p == '}') {
            jc.p++;
            break;
        }
        goto fail;
    }

    j_skip_ws(&jc);
    if (jc.p != jc.end) goto fail;
    if (!out->case_rel || !out->case_rel[0] || !out->init_config_json) goto fail;
    return 1;

fail:
    free_case(out);
    return 0;
}

/* ---- Trace emission & real Linux compose/batch runner ---- */

static int emit_trace_cmd(strbuf *trace, const char *case_rel, uint32_t step,
                          const char *cmd, const char *text_utf8, size_t text_len,
                          uint32_t n) {
    if (!sb_push_str(trace, "{\"case\":\"")) return 0;
    if (!sb_push_json_escaped(trace, case_rel, strlen(case_rel))) return 0;
    if (!sb_push_str(trace, "\",\"step\":")) return 0;
    if (!sb_push_u32(trace, step)) return 0;
    if (!sb_push_str(trace, ",\"cmd\":\"")) return 0;
    if (!sb_push_str(trace, cmd)) return 0;
    if (!sb_push_str(trace, "\",\"text\":\"")) return 0;
    if (text_utf8 && text_len > 0) {
        if (!sb_push_json_escaped(trace, text_utf8, text_len)) return 0;
    }
    if (!sb_push_str(trace, "\",\"n\":")) return 0;
    if (!sb_push_u32(trace, n)) return 0;
    if (!sb_push_str(trace, "}\n")) return 0;
    return 1;
}

static int json_config_bool_true(const char *cfg, const char *key) {
    if (!cfg || !key) return 0;
    size_t klen = strlen(key);
    const char *p = cfg;
    while ((p = strstr(p, key)) != NULL) {
        if (p > cfg && p[-1] == '"' && p[klen] == '"') {
            const char *q = p + klen + 1;
            while (*q == ' ' || *q == '\t' || *q == '\n' || *q == '\r') q++;
            if (*q == ':') {
                q++;
                while (*q == ' ' || *q == '\t' || *q == '\n' || *q == '\r') q++;
                if (strncmp(q, "true", 4u) == 0) {
                    unsigned char after = (unsigned char)q[4];
                    if (after == ',' || after == '}' || after == ' ' || after == '\n' ||
                        after == '\r' || after == '\t' || after == '\0') {
                        return 1;
                    }
                }
            }
        }
        p += klen;
    }
    return 0;
}

static int is_modifier_vk(uint32_t vk) {
    return (vk >= 0x10u && vk <= 0x12u) || vk == 0x5Bu || vk == 0x5Cu ||
           (vk >= 0xA0u && vk <= 0xA5u);
}

static int is_ctrl_space(uint32_t vk, uint32_t ch, uint32_t mods) {
    return (vk == LC_VK_SPACE || ch == ' ') && ((mods & ~IME_MOD_CAPS) == IME_MOD_CTRL);
}

typedef struct trace_runner {
    const char  *case_rel;
    ime_instance *inst;
    lc_comp      comp;
    lc_edit_batch edits;
    int          hide_preedit_after_commit;
    int          allow_macro_when_vi_off;
    emit_app     app;
    char        *last_config;
    strbuf       trace;
} trace_runner;

static void runner_apply_context(trace_runner *tr) {
    ime_context_v1 ctx;
    memset(&ctx, 0, sizeof(ctx));
    ctx.abi_version = IME_ABI_VERSION;
    ctx.enabled = tr->app.enabled ? 1u : 0u;
    ctx.secure = tr->app.secure ? 1u : 0u;
    ctx.field_role = tr->app.field_role;
    ctx.caps = tr->app.caps;
    ctx.app_id = (tr->app.bundle && tr->app.bundle[0]) ? tr->app.bundle : NULL;
    ctx.element_name = NULL;
    ctx.hint = -1;
    assert(ime_set_context(tr->inst, &ctx) == IME_OK);
}

static void runner_set_config(trace_runner *tr, const char *cfg) {
    free(tr->last_config);
    tr->last_config = xstrdup(cfg);
    int32_t rc = ime_reload_config(tr->inst, (const uint8_t *)cfg, strlen(cfg));
    assert(rc == IME_OK);
    tr->allow_macro_when_vi_off = json_config_bool_true(cfg, "allow_macro_when_vi_off");
}

static void runner_flush_batch(trace_runner *tr, uint32_t step) {
    uint32_t del = 0;
    const char *txt = NULL;
    size_t len = 0;
    if (lc_edit_batch_end(&tr->edits, &del, &txt, &len)) {
        if (del > 0) {
            assert(emit_trace_cmd(&tr->trace, tr->case_rel, step, "delete_before", "", 0u, del));
        }
        if (len > 0) {
            assert(emit_trace_cmd(&tr->trace, tr->case_rel, step, "commit", txt, len, 0u));
        }
    }
    if (!lc_edit_batch_active(&tr->edits) && tr->hide_preedit_after_commit) {
        tr->hide_preedit_after_commit = 0;
        assert(emit_trace_cmd(&tr->trace, tr->case_rel, step, "set_preedit", "", 0u, 0u));
    }
}

static void runner_finish_word(trace_runner *tr, uint32_t step) {
    if (tr->comp.len > 0) {
        char utf8[LC_COMP_MAX * 4 + 1];
        size_t ulen = lc_utf32_to_utf8(tr->comp.text, tr->comp.len, utf8, sizeof(utf8));
        if (lc_edit_batch_active(&tr->edits)) {
            if (ulen > 0) {
                assert(lc_edit_batch_commit(&tr->edits, utf8, ulen));
            }
            tr->hide_preedit_after_commit = 1;
        } else {
            if (ulen > 0) {
                assert(emit_trace_cmd(&tr->trace, tr->case_rel, step, "commit", utf8, ulen, 0u));
            }
            assert(emit_trace_cmd(&tr->trace, tr->case_rel, step, "set_preedit", "", 0u, 0u));
        }
        tr->comp.len = 0;
    }
    ime_reset(tr->inst);
}

static void runner_apply_linux_plan(trace_runner *tr, lc_key key, const ime_result_v1 *r,
                                    uint32_t step, int emit_key_disp) {
    lc_plan plan;
    lc_plan_key(&tr->comp, key, r, &plan);
    size_t prev_len = tr->comp.len;

    if (plan.delete_before > 0) {
        lc_edit_batch_delete(&tr->edits, (uint32_t)plan.delete_before);
    }
    if (plan.end) {
        size_t n = 0;
        const uint32_t *t = lc_plan_commit_text(&tr->comp, &plan, &n);
        if (n > 0) {
            char utf8[LC_COMP_MAX * 4 + 1];
            size_t clen = n > (size_t)LC_COMP_MAX ? (size_t)LC_COMP_MAX : n;
            size_t ulen = lc_utf32_to_utf8(t, clen, utf8, sizeof(utf8));
            if (ulen > 0) {
                assert(lc_edit_batch_commit(&tr->edits, utf8, ulen));
            }
        }
        if (prev_len > 0) {
            tr->hide_preedit_after_commit = 1;
        }
    }
    lc_comp_apply(&tr->comp, &plan);
    if (plan.reset_engine) {
        ime_reset(tr->inst);
    }

    runner_flush_batch(tr, step);
    if (!plan.end && plan.has_text && !(prev_len == 0 && plan.text_len == 0)) {
        char utf8[LC_COMP_MAX * 4 + 1];
        size_t plen = plan.text_len > (size_t)LC_COMP_MAX ? (size_t)LC_COMP_MAX : plan.text_len;
        size_t ulen = lc_utf32_to_utf8(plan.text, plen, utf8, sizeof(utf8));
        assert(emit_trace_cmd(&tr->trace, tr->case_rel, step, "set_preedit",
                              utf8, ulen, (uint32_t)plen));
    }
    if (emit_key_disp) {
        assert(emit_trace_cmd(&tr->trace, tr->case_rel, step,
                              plan.eaten ? "eat_key" : "forward_key", "", 0u, 0u));
    }
}

static void runner_step_key(trace_runner *tr, const emit_event *ev) {
    if (ev->down == 0) {
        ime_key_v1 k;
        memset(&k, 0, sizeof(k));
        k.abi_version = IME_ABI_VERSION;
        k.vk = ev->vk;
        k.ch = ev->ch;
        k.mods = ev->mods;
        k.key_down = 0;
        k.is_injected = ev->injected ? 1u : 0u;

        ime_result_v1 r;
        memset(&r, 0, sizeof(r));
        assert(ime_key(tr->inst, &k, &r) == IME_OK);
        if (r.action != IME_ACTION_PASS) {
            lc_edit_batch_begin(&tr->edits);
            tr->hide_preedit_after_commit = 0;
            runner_apply_linux_plan(tr, (lc_key){ LC_KEY_OTHER, 0 }, &r, ev->step, 0);
        } else if ((r.flags & IME_FLAG_WORD_END) != 0 && tr->comp.len > 0) {
            char utf8[LC_COMP_MAX * 4 + 1];
            size_t ulen = lc_utf32_to_utf8(tr->comp.text, tr->comp.len, utf8, sizeof(utf8));
            if (ulen > 0) {
                assert(emit_trace_cmd(&tr->trace, tr->case_rel, ev->step, "commit", utf8, ulen, 0u));
            }
            assert(emit_trace_cmd(&tr->trace, tr->case_rel, ev->step, "set_preedit", "", 0u, 0u));
            tr->comp.len = 0;
        }
        return;
    }

    lc_edit_batch_begin(&tr->edits);
    tr->hide_preedit_after_commit = 0;

    if (is_modifier_vk(ev->vk)) {
        ime_key_v1 k;
        memset(&k, 0, sizeof(k));
        k.abi_version = IME_ABI_VERSION;
        k.vk = ev->vk;
        k.ch = ev->ch;
        k.mods = ev->mods;
        k.key_down = 1;
        k.is_injected = ev->injected ? 1u : 0u;

        ime_result_v1 r;
        memset(&r, 0, sizeof(r));
        assert(ime_key(tr->inst, &k, &r) == IME_OK);
        if (r.action == IME_ACTION_PASS) {
            runner_flush_batch(tr, ev->step);
            assert(emit_trace_cmd(&tr->trace, tr->case_rel, ev->step, "forward_key", "", 0u, 0u));
            return;
        }
        runner_apply_linux_plan(tr, (lc_key){ LC_KEY_OTHER, 0 }, &r, ev->step, 1);
        return;
    }

    int ctrl_space = is_ctrl_space(ev->vk, ev->ch, ev->mods);
    int non_cs_chord = ((ev->mods & (IME_MOD_CTRL | IME_MOD_ALT | IME_MOD_SUPER | IME_MOD_META)) != 0) &&
                       !ctrl_space;
    if (ev->injected || non_cs_chord || tr->app.secure ||
        !(tr->app.enabled || tr->allow_macro_when_vi_off)) {
        /* Gọi engine ở chế độ key-down giống replay để giữ trạng thái ngữ cảnh/lỗi. */
        ime_key_v1 k;
        memset(&k, 0, sizeof(k));
        k.abi_version = IME_ABI_VERSION;
        k.vk = ev->vk;
        k.ch = ev->ch;
        k.mods = ev->mods;
        k.key_down = 1;
        k.is_injected = ev->injected ? 1u : 0u;
        ime_result_v1 r;
        memset(&r, 0, sizeof(r));
        assert(ime_key(tr->inst, &k, &r) == IME_OK);

        runner_finish_word(tr, ev->step);
        runner_flush_batch(tr, ev->step);
        assert(emit_trace_cmd(&tr->trace, tr->case_rel, ev->step, "forward_key", "", 0u, 0u));
        return;
    }

    lc_key key = lc_key_classify_with_mods(ev->vk, ev->ch, ev->mods);
    ime_key_v1 k;
    lc_key_to_ime(key, ev->vk, ev->mods, &k);
    k.is_injected = ev->injected ? 1u : 0u;

    ime_result_v1 r;
    memset(&r, 0, sizeof(r));
    if (ime_key(tr->inst, &k, &r) != IME_OK || (r.flags & IME_FLAG_ERROR) != 0) {
        runner_finish_word(tr, ev->step);
        runner_flush_batch(tr, ev->step);
        assert(emit_trace_cmd(&tr->trace, tr->case_rel, ev->step, "forward_key", "", 0u, 0u));
        return;
    }

    runner_apply_linux_plan(tr, key, &r, ev->step, 1);
}

static char *run_case_once(const emit_case *ec) {
    trace_runner tr;
    memset(&tr, 0, sizeof(tr));
    tr.case_rel = ec->case_rel;
    sb_init(&tr.trace);
    assert(ime_instance_new(NULL, 0u, &tr.inst) == IME_OK);

    tr.app.bundle = ec->init_app.bundle ? xstrdup(ec->init_app.bundle) : NULL;
    tr.app.field_role = ec->init_app.field_role;
    tr.app.caps = ec->init_app.caps;
    tr.app.secure = ec->init_app.secure;
    tr.app.enabled = ec->init_app.enabled;

    runner_set_config(&tr, ec->init_config_json);
    runner_apply_context(&tr);

    for (size_t i = 0; i < ec->event_count; ++i) {
        const emit_event *ev = &ec->events[i];
        switch (ev->kind) {
        case EV_CONFIG:
            runner_set_config(&tr, ev->config_json);
            break;
        case EV_CONTEXT:
            free_app(&tr.app);
            tr.app.bundle = ev->ctx.bundle ? xstrdup(ev->ctx.bundle) : NULL;
            tr.app.field_role = ev->ctx.field_role;
            tr.app.caps = ev->ctx.caps;
            tr.app.secure = ev->ctx.secure;
            tr.app.enabled = ev->ctx.enabled;
            runner_apply_context(&tr);
            break;
        case EV_RESET:
            runner_finish_word(&tr, ev->step);
            break;
        case EV_ENGINE_NEW:
            runner_finish_word(&tr, ev->step);
            ime_instance_free(tr.inst);
            tr.inst = NULL;
            assert(ime_instance_new(NULL, 0u, &tr.inst) == IME_OK);
            runner_set_config(&tr, tr.last_config);
            runner_apply_context(&tr);
            break;
        case EV_KEY:
            runner_step_key(&tr, ev);
            break;
        case EV_SKIP:
            break;
        }
    }

    ime_instance_free(tr.inst);
    free(tr.last_config);
    free_app(&tr.app);
    return tr.trace.data ? tr.trace.data : xstrdup("");
}

/* ---- Per-step invariant verifier ---- */

typedef struct step_stats {
    size_t dels;
    size_t commits;
    size_t key_disp;
    size_t first_commit_idx;
    size_t first_hide_idx;
    size_t count;
} step_stats;

static void verify_trace_invariants(const char *jsonl, const emit_case *ec, int *saw_commit_then_hide) {
    size_t slots = (size_t)ec->key_step_count + 2u;
    step_stats *stats = (step_stats *)calloc(slots, sizeof(step_stats));
    assert(stats != NULL);
    for (size_t i = 0; i < slots; ++i) {
        stats[i].first_commit_idx = SIZE_MAX;
        stats[i].first_hide_idx = SIZE_MAX;
    }

    const char *p = jsonl;
    while (*p != '\0') {
        const char *nl = strchr(p, '\n');
        size_t llen = nl ? (size_t)(nl - p) : strlen(p);
        assert(llen > 0u);

        json_cursor jc = { p, p + llen };
        assert(j_expect(&jc, '{'));
        char *lcase = NULL;
        char *cmd = NULL;
        char *text = NULL;
        uint32_t step = 0;
        uint32_t n = 0;
        int seen_fields = 0;
        for (;;) {
            char *k = NULL;
            assert(j_parse_string(&jc, &k));
            assert(j_expect(&jc, ':'));
            if (strcmp(k, "case") == 0) {
                free(lcase);
                assert(j_parse_string(&jc, &lcase));
                seen_fields |= 1;
            } else if (strcmp(k, "step") == 0) {
                assert(j_parse_u32(&jc, &step));
                seen_fields |= 2;
            } else if (strcmp(k, "cmd") == 0) {
                free(cmd);
                assert(j_parse_string(&jc, &cmd));
                seen_fields |= 4;
            } else if (strcmp(k, "text") == 0) {
                free(text);
                assert(j_parse_string(&jc, &text));
                seen_fields |= 8;
            } else if (strcmp(k, "n") == 0) {
                assert(j_parse_u32(&jc, &n));
                seen_fields |= 16;
            } else {
                assert(0 && "unexpected trace JSON field");
            }
            free(k);
            j_skip_ws(&jc);
            if (*jc.p == ',') {
                jc.p++;
                continue;
            }
            assert(*jc.p == '}');
            jc.p++;
            break;
        }
        j_skip_ws(&jc);
        assert(jc.p == jc.end);
        assert(seen_fields == 31);
        assert(strcmp(lcase, ec->case_rel) == 0);
        assert((size_t)step < slots);

        step_stats *st = &stats[step];
        size_t idx = st->count++;
        if (strcmp(cmd, "delete_before") == 0) {
            st->dels++;
            assert(n > 0u);
        } else if (strcmp(cmd, "commit") == 0) {
            st->commits++;
            if (st->first_commit_idx == SIZE_MAX) st->first_commit_idx = idx;
        } else if (strcmp(cmd, "set_preedit") == 0) {
            if (text[0] == '\0' && n == 0u && st->first_hide_idx == SIZE_MAX) {
                st->first_hide_idx = idx;
            }
        } else if (strcmp(cmd, "forward_key") == 0 || strcmp(cmd, "eat_key") == 0) {
            st->key_disp++;
        } else {
            assert(0 && "unknown SF-1d command outside 5-cmd vocabulary");
        }

        free(lcase);
        free(cmd);
        free(text);
        p = nl ? (nl + 1) : (p + llen);
    }

    for (uint32_t s = 0; s < ec->key_step_count; ++s) {
        const step_stats *st = &stats[s];
        assert(st->dels <= 1u);
        assert(st->commits <= 1u);
        assert(st->key_disp == 1u);
        if (st->first_commit_idx != SIZE_MAX && st->first_hide_idx != SIZE_MAX) {
            assert(st->first_commit_idx < st->first_hide_idx);
            if (saw_commit_then_hide) *saw_commit_then_hide = 1;
        }
    }
    if ((size_t)ec->key_step_count < slots) {
        const step_stats *tail = &stats[ec->key_step_count];
        assert(tail->dels <= 1u);
        assert(tail->commits <= 1u);
        assert(tail->key_disp == 0u);
        if (tail->first_commit_idx != SIZE_MAX && tail->first_hide_idx != SIZE_MAX) {
            assert(tail->first_commit_idx < tail->first_hide_idx);
            if (saw_commit_then_hide) *saw_commit_then_hide = 1;
        }
    }

    free(stats);
}

/* ---- File IO & recursive directory sweep ---- */

static char *read_entire_file(const char *path, size_t *out_len) {
    FILE *f = fopen(path, "rb");
    assert(f != NULL);
    assert(fseek(f, 0, SEEK_END) == 0);
    long sz = ftell(f);
    assert(sz >= 0);
    assert(fseek(f, 0, SEEK_SET) == 0);
    char *buf = (char *)malloc((size_t)sz + 1u);
    assert(buf != NULL);
    size_t got = (sz > 0) ? fread(buf, 1u, (size_t)sz, f) : 0u;
    assert(got == (size_t)sz);
    buf[got] = '\0';
    fclose(f);
    *out_len = got;
    return buf;
}

static int ends_with(const char *s, const char *suffix) {
    size_t sl = strlen(s);
    size_t xl = strlen(suffix);
    return sl >= xl && strcmp(s + sl - xl, suffix) == 0;
}

static int mkdir_p(const char *dir) {
    char tmp[4096];
    size_t len = strlen(dir);
    if (len == 0u || len >= sizeof(tmp)) return -1;
    memcpy(tmp, dir, len + 1u);
    for (size_t i = 1u; i < len; ++i) {
        if (tmp[i] == '/') {
            tmp[i] = '\0';
            if (mkdir(tmp, 0755) != 0 && errno != EEXIST) return -1;
            tmp[i] = '/';
        }
    }
    if (mkdir(tmp, 0755) != 0 && errno != EEXIST) return -1;
    return 0;
}

static void write_trace_file(const char *out_root, const char *case_rel, const char *content) {
    size_t rlen = strlen(out_root);
    size_t clen = strlen(case_rel);
    char *path = (char *)malloc(rlen + 1u + clen + 6u + 1u);
    assert(path != NULL);
    memcpy(path, out_root, rlen);
    path[rlen] = '/';
    memcpy(path + rlen + 1u, case_rel, clen);
    memcpy(path + rlen + 1u + clen, ".jsonl", 7u);

    char *slash = strrchr(path, '/');
    if (slash && slash != path) {
        *slash = '\0';
        assert(mkdir_p(path) == 0);
        *slash = '/';
    }

    FILE *f = fopen(path, "wb");
    assert(f != NULL);
    size_t n = strlen(content);
    if (n > 0u) assert(fwrite(content, 1u, n, f) == n);
    assert(fclose(f) == 0);
    free(path);
}

typedef struct path_list {
    char **items;
    size_t count;
    size_t cap;
} path_list;

static void collect_json_files(const char *dir, path_list *pl) {
    DIR *d = opendir(dir);
    assert(d != NULL);
    size_t dlen = strlen(dir);
    struct dirent *ent = NULL;
    while ((ent = readdir(d)) != NULL) {
        if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0) continue;
        size_t nlen = strlen(ent->d_name);
        char *child = (char *)malloc(dlen + 1u + nlen + 1u);
        assert(child != NULL);
        memcpy(child, dir, dlen);
        child[dlen] = '/';
        memcpy(child + dlen + 1u, ent->d_name, nlen + 1u);

        struct stat st;
        if (stat(child, &st) == 0) {
            if (S_ISDIR(st.st_mode)) {
                collect_json_files(child, pl);
                free(child);
            } else if (S_ISREG(st.st_mode) && ends_with(ent->d_name, ".json")) {
                if (pl->count == pl->cap) {
                    size_t ncap = pl->cap ? pl->cap * 2u : 64u;
                    char **np = (char **)realloc(pl->items, ncap * sizeof(char *));
                    assert(np != NULL);
                    pl->items = np;
                    pl->cap = ncap;
                }
                pl->items[pl->count++] = child;
            } else {
                free(child);
            }
        } else {
            free(child);
        }
    }
    closedir(d);
}

static int cmp_str_ptr(const void *a, const void *b) {
    const char *const *sa = (const char *const *)a;
    const char *const *sb = (const char *const *)b;
    return strcmp(*sa, *sb);
}

/* ---- Built-in self-tests ---- */

static void test_json_parser_fail_closed(void) {
    emit_case ec;
    assert(parse_emit_case("{invalid", 8u, &ec) == 0);
    assert(parse_emit_case("{\"case\":\"\",\"config_json\":\"{}\",\"app\":{},\"events\":[]}", 53u, &ec) == 0);

    strbuf sb;
    sb_init(&sb);
    assert(sb_push_json_escaped(&sb, "a\"b\\c\n\x01", 7u));
    assert(strcmp(sb.data, "a\\\"b\\\\c\\n\\u0001") == 0);
    sb_free(&sb);
    assert(json_config_bool_true("{\"method\":\"telex\",\"allow_macro_when_vi_off\":true}", "allow_macro_when_vi_off") == 1);
    assert(json_config_bool_true("{\"allow_macro_when_vi_off\":false}", "allow_macro_when_vi_off") == 0);
}

static void test_synthetic_determinism_and_commit_before_hide(void) {
    const char *sample =
        "{"
        "\"case\":\"shared/sf1d_selftest\","
        "\"config_json\":\"{\\\"config_version\\\":1,\\\"method\\\":\\\"telex\\\"}\","
        "\"app\":{\"bundle\":\"gedit\",\"field_role\":1,\"caps\":1,\"secure\":false,\"enabled\":true},"
        "\"keys\":[],"
        "\"resets\":[7],"
        "\"events\":["
          "{\"kind\":\"key\",\"step\":0,\"vk\":0,\"ch\":100,\"mods\":0,\"down\":1,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":0,\"vk\":0,\"ch\":100,\"mods\":0,\"down\":0,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":1,\"vk\":0,\"ch\":100,\"mods\":0,\"down\":1,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":1,\"vk\":0,\"ch\":100,\"mods\":0,\"down\":0,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":2,\"vk\":0,\"ch\":117,\"mods\":0,\"down\":1,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":2,\"vk\":0,\"ch\":117,\"mods\":0,\"down\":0,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":3,\"vk\":0,\"ch\":111,\"mods\":0,\"down\":1,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":3,\"vk\":0,\"ch\":111,\"mods\":0,\"down\":0,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":4,\"vk\":0,\"ch\":99,\"mods\":0,\"down\":1,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":4,\"vk\":0,\"ch\":99,\"mods\":0,\"down\":0,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":5,\"vk\":0,\"ch\":106,\"mods\":0,\"down\":1,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":5,\"vk\":0,\"ch\":106,\"mods\":0,\"down\":0,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":6,\"vk\":32,\"ch\":32,\"mods\":0,\"down\":1,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":6,\"vk\":32,\"ch\":32,\"mods\":0,\"down\":0,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":7,\"vk\":0,\"ch\":97,\"mods\":0,\"down\":1,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":7,\"vk\":0,\"ch\":97,\"mods\":0,\"down\":0,\"injected\":false},"
          "{\"kind\":\"reset\",\"step\":8},"
          "{\"kind\":\"context\",\"step\":8,\"bundle\":\"gedit\",\"field_role\":10,\"caps\":1,\"secure\":true,\"enabled\":true},"
          "{\"kind\":\"key\",\"step\":8,\"vk\":0,\"ch\":98,\"mods\":0,\"down\":1,\"injected\":false},"
          "{\"kind\":\"key\",\"step\":8,\"vk\":0,\"ch\":98,\"mods\":0,\"down\":0,\"injected\":false}"
        "]"
        "}";
    emit_case ec;
    assert(parse_emit_case(sample, strlen(sample), &ec));
    char *t1 = run_case_once(&ec);
    char *t2 = run_case_once(&ec);
    assert(t1[0] != '\0');
    assert(strcmp(t1, t2) == 0);
    int saw_commit_then_hide = 0;
    verify_trace_invariants(t1, &ec, &saw_commit_then_hide);
    assert(saw_commit_then_hide == 1);
    free(t1);
    free(t2);
    free_case(&ec);
}

static void test_edit_batch_commit_before_hide(void) {
    trace_runner tr;
    memset(&tr, 0, sizeof(tr));
    tr.case_rel = "shared/batch_check";
    sb_init(&tr.trace);
    lc_edit_batch_begin(&tr.edits);
    assert(lc_edit_batch_commit(&tr.edits, "cu", 2u));
    lc_edit_batch_delete(&tr.edits, 5u);
    assert(lc_edit_batch_commit(&tr.edits, "moi", 3u));
    tr.hide_preedit_after_commit = 1;
    runner_flush_batch(&tr, 4u);

    const char *want =
        "{\"case\":\"shared/batch_check\",\"step\":4,\"cmd\":\"delete_before\",\"text\":\"\",\"n\":3}\n"
        "{\"case\":\"shared/batch_check\",\"step\":4,\"cmd\":\"commit\",\"text\":\"moi\",\"n\":0}\n"
        "{\"case\":\"shared/batch_check\",\"step\":4,\"cmd\":\"set_preedit\",\"text\":\"\",\"n\":0}\n";
    assert(tr.trace.data != NULL && strcmp(tr.trace.data, want) == 0);
    sb_free(&tr.trace);
}

int main(void) {
    test_json_parser_fail_closed();
    test_synthetic_determinism_and_commit_before_hide();
    test_edit_batch_commit_before_hide();

    const char *emit_dir = getenv("TEXTVN_EMIT_KEYS");
    const char *trace_out = getenv("TEXTVN_TRACE_OUT");
    if (emit_dir && emit_dir[0] != '\0') {
        path_list pl = { NULL, 0u, 0u };
        collect_json_files(emit_dir, &pl);
        assert(pl.count > 0u); /* E94: fail-closed nếu thư mục rỗng */
        qsort(pl.items, pl.count, sizeof(char *), cmp_str_ptr);

        for (size_t i = 0; i < pl.count; ++i) {
            size_t flen = 0u;
            char *raw = read_entire_file(pl.items[i], &flen);
            emit_case ec;
            assert(parse_emit_case(raw, flen, &ec));
            free(raw);

            char *t1 = run_case_once(&ec);
            char *t2 = run_case_once(&ec);
            assert(strcmp(t1, t2) == 0);
            free(t2);
            verify_trace_invariants(t1, &ec, NULL);
            if (trace_out && trace_out[0] != '\0') {
                write_trace_file(trace_out, ec.case_rel, t1);
            }
            free(t1);
            free_case(&ec);
            free(pl.items[i]);
        }
        free(pl.items);
        printf("PASS trace_compose (%zu emitted cases verified & deterministic)\n", pl.count);
    } else {
        printf("PASS trace_compose self-tests\n");
    }
    return 0;
}
