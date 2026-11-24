/* SPDX-License-Identifier: GPL-3.0-or-later */
/* test_emoji_picker.c — Unit tests cho P12 Emoji Picker trong linux-common. */

#include "lc_emoji.h"

#include <assert.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    ime_instance *inst = NULL;
    assert(ime_instance_new(NULL, 0, &inst) == IME_OK);
    assert(inst != NULL);

    /* 1. Kiểm tra hotkey Ctrl+Alt+E (bỏ qua CapsLock, chặn Ctrl+Alt+Shift+E). */
    assert(lc_is_emoji_hotkey('e', 1, 1, 0, 0) == 1);
    assert(lc_is_emoji_hotkey('E', 1, 1, 0, 0) == 1);
    assert(lc_is_emoji_hotkey('e', 1, 1, 1, 0) == 0);
    assert(lc_is_emoji_hotkey('e', 1, 0, 0, 0) == 0);

    /* 2. Mở picker, kiểm tra danh mục mặc định không rỗng và aux text hợp lệ. */
    lc_emoji_picker p;
    lc_emoji_init(&p);
    lc_emoji_open(&p, inst);
    assert(p.active == 1);
    assert(p.count > 0);
    char aux[256];
    lc_emoji_format_aux(&p, aux, sizeof(aux));
    assert(aux[0] != '\0');

    /* 3. Gõ tìm kiếm "fire", Enter → chèn 🔥 và cập nhật MRU. */
    char glyph[LC_EMOJI_GLYPH_MAX];
    const char *q = "fire";
    for (size_t i = 0; q[i] != '\0'; i++) {
        assert(lc_emoji_handle_key(&p, inst, 0, (uint32_t)(unsigned char)q[i], 0, glyph, sizeof(glyph)) == LC_EMOJI_UPDATE);
    }
    assert(p.count > 0);
    assert(strcmp(p.items[0].shortcode, "fire") == 0);
    assert(lc_emoji_handle_key(&p, inst, 0x0Du, 0, 0, glyph, sizeof(glyph)) == LC_EMOJI_COMMIT);
    assert(strcmp(glyph, "🔥") == 0);
    assert(p.active == 0);

    /* 4. Mở lại: 🔥 đứng đầu nhóm Gần đây (MRU), phím '1' chọn ngay! */
    lc_emoji_open(&p, inst);
    assert(p.count > 0);
    assert(strcmp(p.items[0].glyph, "🔥") == 0);
    assert(lc_emoji_handle_key(&p, inst, 0, '1', 0, glyph, sizeof(glyph)) == LC_EMOJI_COMMIT);
    assert(strcmp(glyph, "🔥") == 0);

    /* 5. Mở lại, Tab chuyển nhóm 0 → 1, Esc đóng sạch. */
    lc_emoji_open(&p, inst);
    assert(lc_emoji_handle_key(&p, inst, 0x09u, 0, 0, glyph, sizeof(glyph)) == LC_EMOJI_UPDATE);
    assert(p.category == 1);
    assert(lc_emoji_handle_key(&p, inst, 0x1Bu, 0, 0, glyph, sizeof(glyph)) == LC_EMOJI_CLOSED);
    assert(p.active == 0);

    ime_instance_free(inst);
    printf("test_emoji_picker: OK\n");
    return 0;
}
