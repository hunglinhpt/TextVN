/* test_edit_batch.c — R2-165: một phím chỉ gửi tối đa 1 xoá + 1 commit, kết quả như từng bước
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Mô phỏng document: áp từng lệnh (đường cũ, nhiều commit) và áp kết quả gom (đường mới, như
 * text-input-v3: xoá rồi commit) — hai document phải giống hệt. Kèm mô phỏng mutter gom lệnh
 * ("chỉ giữ commit/xoá cuối") cho thấy đường cũ MẤT CHỮ còn đường mới thì không.
 */
#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "lc_edit_batch.h"

typedef struct {
    char s[512];
} doc_t;

static void doc_delete(doc_t *d, uint32_t n) {
    size_t len = strlen(d->s);
    while (n-- > 0 && len > 0) {
        len--;
        while (len > 0 && ((unsigned char)d->s[len] & 0xC0u) == 0x80u) len--;
    }
    d->s[len] = '\0';
}

static void doc_commit(doc_t *d, const char *t, size_t n) {
    size_t len = strlen(d->s);
    assert(len + n < sizeof(d->s));
    memcpy(d->s + len, t, n);
    d->s[len + n] = '\0';
}

/* Kịch bản trợ lý tự sửa câu (IBus check_and_apply_assist): document "Tôi tôi đi ", preedit
 * "học" chốt + trigger "." + xoá 15 ký tự ("Tôi tôi đi học.") + commit "Tôi đi học." */
static void scenario(lc_edit_batch *b, doc_t *stepwise) {
    const char *word = "h\xe1\xbb\x8d" "c";   /* học */
    const char *fix = "T\xc3\xb4i \xc4\x91i h\xe1\xbb\x8d" "c."; /* Tôi đi học. */
    doc_commit(stepwise, word, strlen(word));
    assert(lc_edit_batch_commit(b, word, strlen(word)));
    doc_commit(stepwise, ".", 1);
    assert(lc_edit_batch_commit(b, ".", 1));
    doc_delete(stepwise, 15);
    lc_edit_batch_delete(b, 15);
    doc_commit(stepwise, fix, strlen(fix));
    assert(lc_edit_batch_commit(b, fix, strlen(fix)));
}

int main(void) {
    static lc_edit_batch b;
    /* "Xin chào. Tôi tôi đi " — có chữ phía trước để thấy lệnh xoá ăn lẹm khi bị gộp. */
    const char *start = "Xin ch\xc3\xa0o. T\xc3\xb4i t\xc3\xb4i \xc4\x91i ";

    /* 1. Gom = từng bước. */
    doc_t stepwise, batched;
    strcpy(stepwise.s, start);
    strcpy(batched.s, start);
    lc_edit_batch_begin(&b);
    scenario(&b, &stepwise);
    uint32_t del = 0;
    const char *txt = NULL;
    size_t n = 0;
    assert(lc_edit_batch_end(&b, &del, &txt, &n) == 1);
    doc_delete(&batched, del);
    doc_commit(&batched, txt, n);
    assert(strcmp(stepwise.s, batched.s) == 0);
    assert(strcmp(batched.s, "Xin ch\xc3\xa0o. T\xc3\xb4i \xc4\x91i h\xe1\xbb\x8d" "c.") == 0);
    assert(del == 11); /* chỉ phần đã có trong document thật: "Tôi tôi đi " */
    printf("PASS batch == stepwise (del=%u, commit 1 lần)\n", del);

    /* 2. mutter gộp (chỉ giữ commit cuối + xoá cuối, áp lên document trước phím): đường cũ hỏng. */
    doc_t coalesced;
    strcpy(coalesced.s, start);
    doc_delete(&coalesced, 15);
    doc_commit(&coalesced, "T\xc3\xb4i \xc4\x91i h\xe1\xbb\x8d" "c.", strlen("T\xc3\xb4i \xc4\x91i h\xe1\xbb\x8d" "c."));
    assert(strcmp(coalesced.s, stepwise.s) != 0); /* chứng minh lỗi có thật nếu không gom */
    printf("PASS old multi-commit path breaks under coalescing (\"%s\")\n", coalesced.s);

    /* 3. Lồng nhau: chỉ tầng ngoài gửi. */
    lc_edit_batch_begin(&b);
    lc_edit_batch_begin(&b);
    assert(lc_edit_batch_commit(&b, "a", 1));
    assert(lc_edit_batch_end(&b, &del, &txt, &n) == 0);
    assert(lc_edit_batch_active(&b));
    assert(lc_edit_batch_end(&b, &del, &txt, &n) == 1 && n == 1 && del == 0);
    assert(!lc_edit_batch_active(&b));

    /* 4. Không gom ⇒ commit trả 0 (caller gửi thẳng); phím không sửa gì ⇒ không gửi. */
    assert(lc_edit_batch_commit(&b, "x", 1) == 0);
    lc_edit_batch_begin(&b);
    assert(lc_edit_batch_end(&b, &del, &txt, &n) == 0);

    /* 5. Hết chỗ ⇒ trả 0, nội dung cũ còn nguyên để flush. */
    static char big[LC_EDIT_BATCH_MAX];
    memset(big, 'x', sizeof(big));
    lc_edit_batch_begin(&b);
    assert(lc_edit_batch_commit(&b, "ab", 2));
    assert(lc_edit_batch_commit(&b, big, sizeof(big)) == 0);
    assert(lc_edit_batch_take(&b, &del, &txt, &n) == 1 && n == 2);
    assert(lc_edit_batch_end(&b, &del, &txt, &n) == 0);

    /* 6. Xoá nhiều hơn phần chờ: phần dư sang document thật; đếm theo KÝ TỰ (không byte). */
    lc_edit_batch_begin(&b);
    assert(lc_edit_batch_commit(&b, "\xc4\x91\xc6\xb0", 4)); /* "đư" = 2 ký tự / 4 byte */
    lc_edit_batch_delete(&b, 3);
    assert(lc_edit_batch_end(&b, &del, &txt, &n) == 1 && del == 1 && n == 0);
    assert(lc_utf8_char_count("\xc4\x91\xc6\xb0" "c", 5) == 3);

    printf("test_edit_batch: OK\n");
    return 0;
}
