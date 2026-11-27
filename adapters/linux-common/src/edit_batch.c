/* edit_batch.c — gom chỉnh sửa của một phím (R2-165), xem lc_edit_batch.h
 * SPDX-License-Identifier: GPL-3.0-or-later
 */
#include "lc_edit_batch.h"

#include <string.h>

static int is_cont(unsigned char c) {
    return (c & 0xC0u) == 0x80u;
}

size_t lc_utf8_char_count(const char *s, size_t len) {
    size_t n = 0;
    for (size_t i = 0; s && i < len; ++i) {
        if (!is_cont((unsigned char)s[i])) n++;
    }
    return n;
}

void lc_edit_batch_begin(lc_edit_batch *b) {
    if (!b) return;
    if (b->depth <= 0) {
        b->depth = 0;
        b->delete_before = 0;
        b->len = 0;
    }
    b->depth++;
}

int lc_edit_batch_active(const lc_edit_batch *b) {
    return b && b->depth > 0;
}

void lc_edit_batch_delete(lc_edit_batch *b, uint32_t n) {
    if (!b || n == 0) return;
    /* Ăn vào phần chờ commit trước (nó nằm ngay trước caret của document ảo). */
    while (n > 0 && b->len > 0) {
        size_t i = b->len - 1;
        while (i > 0 && is_cont((unsigned char)b->text[i])) i--;
        b->len = i;
        n--;
    }
    b->delete_before += n;
}

int lc_edit_batch_commit(lc_edit_batch *b, const char *utf8, size_t len) {
    if (!lc_edit_batch_active(b) || !utf8) return 0;
    if (len == 0) return 1;
    if (len > sizeof(b->text) - b->len) return 0;
    memcpy(b->text + b->len, utf8, len);
    b->len += len;
    return 1;
}

int lc_edit_batch_take(lc_edit_batch *b, uint32_t *delete_before, const char **text, size_t *len) {
    if (!b) return 0;
    const int pending = b->delete_before > 0 || b->len > 0;
    if (delete_before) *delete_before = b->delete_before;
    if (text) *text = b->text;
    if (len) *len = b->len;
    b->delete_before = 0;
    b->len = 0;
    return pending;
}

int lc_edit_batch_end(lc_edit_batch *b, uint32_t *delete_before, const char **text, size_t *len) {
    if (!b || b->depth <= 0) return 0;
    b->depth--;
    if (b->depth > 0) return 0;
    return lc_edit_batch_take(b, delete_before, text, len);
}
