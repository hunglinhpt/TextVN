/* test_assist_text.c — Test hàm thuần so text trước caret (D12/R-2)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Chạy được KHÔNG cần bộ gõ/IBus: chỉ gọi hàm thuần của `lc_assist_text.h`.
 * Đây là "vanh an toàn" của Mode 2: nếu precondition sai ⇒ sửa nhầm text.
 */

#include "lc_assist_text.h"

#include <assert.h>
#include <stdio.h>
#include <string.h>

static void test_ends_with(void) {
    /* Khớp: đuôi bằng nhau. */
    assert(lc_text_ends_with("toi la", 6, "toi la", 6) == 1);
    /* Khớp: đuôi là hậu tố. */
    assert(lc_text_ends_with("xin chao toi la", strlen("xin chao toi la"),
                             "toi la", strlen("toi la")) == 1);
    /* Lệch: nội dung khác. */
    assert(lc_text_ends_with("toi la", 6, "toi lam", 7) == 0);
    /* Ngắn hơn hậu tố. */
    assert(lc_text_ends_with("toi", 3, "toi la", 6) == 0);
    /* Rỗng ⇒ không xác minh được ⇒ fail-safe (0). */
    assert(lc_text_ends_with("toi la", 6, "", 0) == 0);
    /* NULL ⇒ fail-safe. */
    assert(lc_text_ends_with(NULL, 0, "toi", 3) == 0);
    assert(lc_text_ends_with("toi", 3, NULL, 3) == 0);
    /* UTF-8 nhiều byte: so byte vẫn đúng (ký tự cuối là 'á'). */
    assert(lc_text_ends_with("chào", strlen("chào"), "ào", strlen("ào")) == 1);
    assert(lc_text_ends_with("chào", strlen("chào"), "chào", strlen("chào")) == 1);
    /* Cắt giữa ký tự UTF-8 không được khớp (byte cuối khác). */
    assert(lc_text_ends_with("chào", strlen("chào"), "o", 1) == 1);
    assert(lc_text_ends_with("chào", strlen("chào"), "\xC3", 1) == 0);
}

static void test_expected_tail_committed(void) {
    char out[64];

    /* Trigger ĐÃ có trong document ⇒ đuôi document = đuôi engine. */
    assert(lc_assist_expected_document_tail("toi toi.", '.', 1, out, sizeof(out)) == 1);
    assert(strcmp(out, "toi toi.") == 0);

    /* Trigger CHƯA có trong document ⇒ bỏ ký tự trigger ở cuối. */
    assert(lc_assist_expected_document_tail("toi toi.", '.', 0, out, sizeof(out)) == 1);
    assert(strcmp(out, "toi toi") == 0);

    /* Ký tự trigger nhiều byte (ví dụ '…' — không phải trigger thật nhưng hàm
     * phải xử lý đúng). */
    assert(lc_assist_expected_document_tail("chao \xE2\x80\xA6", 0x2026, 0, out, sizeof(out)) == 1);
    assert(strcmp(out, "chao ") == 0);
}

static void test_expected_tail_fail_safe(void) {
    char out[64];

    /* Đuôi rỗng ⇒ không xác minh được. */
    assert(lc_assist_expected_document_tail("", '.', 1, out, sizeof(out)) == 0);
    assert(lc_assist_expected_document_tail("", '.', 0, out, sizeof(out)) == 0);
    /* NULL ⇒ không xác minh được. */
    assert(lc_assist_expected_document_tail(NULL, '.', 1, out, sizeof(out)) == 0);
    /* Buffer nhỏ hơn cần ⇒ fail-safe (không cắt cụt ⇒ không xác minh sai). */
    assert(lc_assist_expected_document_tail("toi toi.", '.', 1, out, 4) == 0);
    /* Ký tự trigger không khớp ký tự cuối của đuôi engine ⇒ không xác minh
     * được (engine tail và phím trigger lệch nhau). */
    assert(lc_assist_expected_document_tail("toi toi.", '!', 0, out, sizeof(out)) == 0);
    /* Trigger không phải scalar hợp lệ ⇒ fail-safe. */
    assert(lc_assist_expected_document_tail("toi toi.", 0x110000u, 0, out, sizeof(out)) == 0);
    /* surrogate ⇒ fail-safe. */
    assert(lc_assist_expected_document_tail("toi toi.", 0xD800u, 0, out, sizeof(out)) == 0);
}

int main(void) {
    test_ends_with();
    test_expected_tail_committed();
    test_expected_tail_fail_safe();
    printf("test_assist_text: OK\n");
    return 0;
}
