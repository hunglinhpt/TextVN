/* assist_text.c — So khớp text trước caret cho "Trợ lý ngữ pháp" (D12/R-2)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Xem `lc_assist_text.h` để hợp đồng. Hàm thuần — không I/O, không log nội
 * dung người dùng (S2).
 */

#include "lc_assist_text.h"

#include "linux_common.h"

#include <string.h>

int lc_text_ends_with(const char *before, size_t before_len,
                      const char *expected, size_t expected_len) {
    if (!before || !expected) return 0;
    /* Không có gì để đối chiếu ⇒ không xác minh được ⇒ fail-safe (D12). */
    if (expected_len == 0) return 0;
    if (before_len < expected_len) return 0;
    /* Cả hai chuỗi là UTF-8 hợp lệ và điểm cắt của `before` nằm trên ranh giới
     * char ⇒ so byte tương đương so chuỗi. */
    return memcmp(before + (before_len - expected_len), expected, expected_len) == 0;
}

/* Số byte của ký tự UTF-8 CUỐI CÙNG trong `s[0..len)`.
 * Trả 0 nếu chuỗi rỗng hoặc byte cuối không phải ký tự UTF-8 hợp lệ. */
static size_t utf8_last_char_len(const char *s, size_t len) {
    if (!s || len == 0) return 0;
    size_t i = len;
    /* Bỏ các byte tiếp tục (10xxxxxx) — byte đầu của ký tự nằm ngay trước. */
    while (i > 0 && ((unsigned char)s[i - 1] & 0xC0u) == 0x80u) i--;
    if (i == 0) return 0; /* không có byte đầu ⇒ UTF-8 hỏng */
    unsigned char lead = (unsigned char)s[i - 1];
    if (lead < 0x80u) return 1;                  /* ASCII: 1 byte */
    if (lead < 0xC2u || lead > 0xF4u) return 0;  /* byte đầu không hợp lệ */
    return len - i + 1;
}

int lc_assist_expected_document_tail(const char *engine_tail,
                                     uint32_t trigger_char,
                                     int trigger_committed,
                                     char *out, size_t out_len) {
    if (!engine_tail || !out || out_len == 0) return 0;
    size_t len = strlen(engine_tail);
    if (len == 0) return 0;            /* không có đuôi ⇒ không xác minh được */
    if (len + 1 > out_len) return 0;   /* buffer quá nhỏ ⇒ fail-safe */

    if (trigger_committed) {
        /* Ký tự trigger ĐÃ nằm trong document ⇒ đuôi document = đuôi engine. */
        memcpy(out, engine_tail, len);
        out[len] = '\0';
        return 1;
    }

    /* Ký tự trigger CHƯA nằm trong document (adapter sẽ tự commit) ⇒ đuôi
     * document phải là đuôi engine BỎ ký tự trigger. */
    size_t last = utf8_last_char_len(engine_tail, len);
    if (last == 0) return 0;

    char trig[8];
    uint32_t cp = trigger_char;
    size_t tlen = lc_utf32_to_utf8(&cp, 1, trig, sizeof(trig));
    if (tlen == 0 || tlen >= sizeof(trig)) return 0; /* scalar không hợp lệ */
    if (tlen != last) return 0;                       /* trigger không phải ký tự cuối */
    if (memcmp(engine_tail + (len - last), trig, last) != 0) return 0;

    memcpy(out, engine_tail, len - last);
    out[len - last] = '\0';
    return 1;
}
