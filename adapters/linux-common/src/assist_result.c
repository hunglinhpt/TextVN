/* assist_result.c — Hàm THUẦN cho kết quả "Trợ lý ngữ pháp" (dùng chung IBus + Fcitx5)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Vì sao tách khỏi compose.c / ipc_client.c (G15 — một quyết định, một nơi):
 *  - compose.c và ipc_client.c có I/O + header POSIX ⇒ **không** test được trên
 *    Windows; hai adapter Linux lại phải dùng **đúng cùng một** luật lọc/khuôn
 *    JSON, nếu chép sang C và C++ thì sớm muộn lệch nhau (parity 3 OS).
 *  - File này chỉ nhận struct thuần (không file, không socket, không engine)
 *    ⇒ `tests/test_assist_capitalize.c` chạy được **không cần IBus/Fcitx5**.
 *
 * Nội dung:
 *  1. D16 (Fassist-20) — loại lỗi L6 khi `config.auto_capitalize` bật.
 *     Crate `assist` **không** đọc config (0-dep, không I/O) nên nó **luôn**
 *     trả L6 ("chữ thường đầu câu"); khi `auto_capitalize=true` thì IME đã tự
 *     viết hoa ⇒ báo L6 là **trùng chức năng** ⇒ lớp adapter (chỗ đọc được
 *     config) phải lọc. Lọc **sau** khi engine trả kết quả (không lọc trước khi
 *     check) ⇒ các luật khác giữ nguyên hành vi.
 *  2. Q-14 — dựng payload `AssistReport` có `issues[]` thật (tối đa
 *     `LC_ASSIST_REPORT_MAX_ISSUES` lỗi) thay cho `"issues":[]` rỗng, khớp
 *     `schemas/ipc.v1.md` + crate `textvn-ipc` (`Message::AssistReport`).
 */

#include "lc_compose.h"
#include "linux_common.h"

#include <stdio.h>
#include <string.h>

/* ---- D16: lọc L6 khi auto_capitalize bật ---- */

const char *lc_assist_kind_name(uint32_t kind) {
    /* Khớp `IssueKind::as_str()` (crate `assist`) và macOS `assistKindName`.
     * Mã lạ (ABI mới hơn adapter) → "unknown": tray chỉ hiển thị, không đoán. */
    switch (kind) {
    case IME_ASSIST_KIND_DUPLICATE_WORD:     return "duplicate_word";
    case IME_ASSIST_KIND_SPACE_BEFORE_PUNCT: return "space_before_punct";
    case IME_ASSIST_KIND_SPACE_AFTER_PUNCT:  return "space_after_punct";
    case IME_ASSIST_KIND_DOUBLE_SPACE:       return "double_space";
    case IME_ASSIST_KIND_LOWERCASE_SENTENCE: return "lowercase_sentence_start";
    case IME_ASSIST_KIND_DIACRITIC_TYPO:     return "diacritic_typo";
    case IME_ASSIST_KIND_SENTENCE_REPEAT:    return "sentence_repeat";
    case IME_ASSIST_KIND_DOUBLE_NEGATIVE:    return "double_negative";
    case IME_ASSIST_KIND_COMMA_AT_END:       return "comma_at_end";
    case IME_ASSIST_KIND_SPELLING:           return "spelling";
    default:                                 return "unknown";
    }
}

int lc_assist_auto_capitalize_enabled(const lc_config_state *st) {
    /* Chưa nạp config (lần đầu chạy / chưa có config.json) ⇒ mặc định của
     * `Config::default()` là **bật** (crate `config`, P0-3 §1.1) — không được
     * coi 0 chỉ vì struct được memset. */
    if (!st || !st->loaded) return LC_ASSIST_DEFAULT_AUTO_CAPITALIZE;
    return st->auto_capitalize != 0;
}

uint32_t lc_assist_filter_auto_capitalize(ime_assist_result_v1 *result, int auto_capitalize) {
    if (!result) return 0;
    /* Chống đọc ngoài mảng nếu engine/ABI mới trả count lớn hơn mảng tĩnh. */
    if (result->count > IME_MAX_ASSIST_ISSUES) result->count = IME_MAX_ASSIST_ISSUES;
    if (!auto_capitalize) return result->count;

    uint32_t kept = 0;
    for (uint32_t i = 0; i < result->count; ++i) {
        if (result->issues[i].kind == IME_ASSIST_KIND_LOWERCASE_SENTENCE) continue;
        if (kept != i) result->issues[kept] = result->issues[i];
        kept++;
    }
    result->count = kept;
    return kept;
}

/* ---- Q-14: payload `AssistReport` ---- */

/* Ghi có kiểm biên: hết chỗ ⇒ `ok = 0` và caller **không gửi** (fail-safe).
 * Buffer luôn giữ NUL ở cuối để debug/in ra an toàn. */
typedef struct json_buf {
    char  *buf;
    size_t cap;
    size_t len;
    int    ok;
} json_buf;

static void jb_raw(json_buf *jb, const char *s, size_t n) {
    if (!jb->ok) return;
    if (jb->len + n >= jb->cap) {
        jb->ok = 0;
        return;
    }
    memcpy(jb->buf + jb->len, s, n);
    jb->len += n;
    jb->buf[jb->len] = '\0';
}

static void jb_cstr(json_buf *jb, const char *s) {
    if (s) jb_raw(jb, s, strlen(s));
}

static void jb_u32(json_buf *jb, uint32_t v) {
    char tmp[16];
    int n = snprintf(tmp, sizeof(tmp), "%u", (unsigned)v);
    if (n <= 0 || (size_t)n >= sizeof(tmp)) {
        jb->ok = 0;
        return;
    }
    jb_raw(jb, tmp, (size_t)n);
}

/* Chuỗi JSON: escape `"` `\` `\b` `\f` `\n` `\r` `\t` + mọi ký tự điều khiển
 * (`< 0x20` → `\u00XX`). Byte >= 0x20 đi thẳng: message/suggest của engine là
 * UTF-8 hợp lệ (crate `assist` sinh từ câu Unicode) và JSON cho phép UTF-8 thô.
 * `max_len` = kích thước mảng nguồn (không tin NUL nếu engine/ABI lạ). */
static void jb_json_string(json_buf *jb, const char *s, size_t max_len) {
    jb_raw(jb, "\"", 1);
    for (size_t i = 0; s && i < max_len && s[i] != '\0'; ++i) {
        char c = s[i];
        switch (c) {
        case '"':  jb_raw(jb, "\\\"", 2); break;
        case '\\': jb_raw(jb, "\\\\", 2); break;
        case '\b': jb_raw(jb, "\\b", 2);  break;
        case '\f': jb_raw(jb, "\\f", 2);  break;
        case '\n': jb_raw(jb, "\\n", 2);  break;
        case '\r': jb_raw(jb, "\\r", 2);  break;
        case '\t': jb_raw(jb, "\\t", 2);  break;
        default:
            if ((unsigned char)c < 0x20) {
                char esc[8];
                int n = snprintf(esc, sizeof(esc), "\\u%04X", (unsigned)(unsigned char)c);
                if (n != 6) {
                    jb->ok = 0;
                    return;
                }
                jb_raw(jb, esc, 6);
            } else {
                jb_raw(jb, &s[i], 1);
            }
        }
    }
    jb_raw(jb, "\"", 1);
}

size_t lc_assist_report_json(const ime_assist_result_v1 *result, uint32_t auto_fixed_count,
                             char *out, size_t out_len) {
    if (!out || out_len == 0) return 0;
    out[0] = '\0';

    json_buf jb;
    jb.buf = out;
    jb.cap = out_len;
    jb.len = 0;
    jb.ok = 1;

    uint32_t count = result ? result->count : 0;
    if (count > IME_MAX_ASSIST_ISSUES) count = IME_MAX_ASSIST_ISSUES;
    /* Q-14: tray chỉ hiển thị vài lỗi đầu (toast 1 dòng + "… và N lỗi khác") —
     * gửi cả 8 chỉ làm payload to hơn mà không thêm thông tin hiển thị. */
    if (count > LC_ASSIST_REPORT_MAX_ISSUES) count = LC_ASSIST_REPORT_MAX_ISSUES;

    /* Không có trường nào mang toàn bộ câu (D8/S2): chỉ kind/span/message/
     * suggestions/auto_fixable/confidence — đúng `AssistIssue` của
     * `textvn-ipc`. Thứ tự khoá = thứ tự field của struct đó (serde). */
    jb_cstr(&jb, "{\"type\":\"AssistReport\",\"issues\":[");
    for (uint32_t i = 0; i < count; ++i) {
        const ime_assist_issue_v1 *iss = &result->issues[i];
        const char *kind_name = lc_assist_kind_name(iss->kind);
        if (i > 0) jb_raw(&jb, ",", 1);
        jb_cstr(&jb, "{\"kind\":");
        jb_json_string(&jb, kind_name, strlen(kind_name));
        jb_cstr(&jb, ",\"span_start\":");
        jb_u32(&jb, iss->start);
        jb_cstr(&jb, ",\"span_end\":");
        jb_u32(&jb, iss->end);
        jb_cstr(&jb, ",\"message\":");
        jb_json_string(&jb, iss->message, IME_MAX_ASSIST_MSG);
        jb_cstr(&jb, ",\"suggestions\":[");
        if (iss->suggest[0] != '\0') jb_json_string(&jb, iss->suggest, IME_MAX_ASSIST_SUGGEST);
        jb_cstr(&jb, "],\"auto_fixable\":");
        jb_cstr(&jb, iss->auto_fixable ? "true" : "false");
        jb_cstr(&jb, ",\"confidence\":");
        /* `confidence: u8` ở crate `ipc` + thang 0–100 (D28): kẹp lại để payload
         * luôn decode được (sai kiểu ⇒ server coi là protocol violation). */
        jb_u32(&jb, iss->confidence > 100 ? 100u : iss->confidence);
        jb_raw(&jb, "}", 1);
    }
    jb_cstr(&jb, "],\"auto_fixed\":");
    jb_u32(&jb, auto_fixed_count);
    jb_raw(&jb, "}", 1);

    if (!jb.ok) {
        out[0] = '\0';
        return 0; /* không vừa buffer ⇒ KHÔNG gửi (fail-safe, cùng tinh thần R-10) */
    }
    return jb.len;
}
