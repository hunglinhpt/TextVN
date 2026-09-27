/* utf.c — UTF-32 and UTF-8 encoding conversion helpers
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "linux_common.h"

#include <string.h>

size_t lc_utf32_to_utf8(const uint32_t *src, size_t src_len, char *dst, size_t dst_len) {
    if (!dst || dst_len == 0) return 0;
    if (!src || src_len == 0) {
        dst[0] = '\0';
        return 0;
    }

    size_t out_idx = 0;
    for (size_t i = 0; i < src_len && src[i] != 0; ++i) {
        uint32_t cp = src[i];

        if (cp <= 0x7F) {
            if (out_idx + 1 >= dst_len) break;
            dst[out_idx++] = (char)cp;
        } else if (cp <= 0x7FF) {
            if (out_idx + 2 >= dst_len) break;
            dst[out_idx++] = (char)(0xC0 | ((cp >> 6) & 0x1F));
            dst[out_idx++] = (char)(0x80 | (cp & 0x3F));
        } else if (cp <= 0xFFFF) {
            if (out_idx + 3 >= dst_len) break;
            dst[out_idx++] = (char)(0xE0 | ((cp >> 12) & 0x0F));
            dst[out_idx++] = (char)(0x80 | ((cp >> 6) & 0x3F));
            dst[out_idx++] = (char)(0x80 | (cp & 0x3F));
        } else if (cp <= 0x10FFFF) {
            if (out_idx + 4 >= dst_len) break;
            dst[out_idx++] = (char)(0xF0 | ((cp >> 18) & 0x07));
            dst[out_idx++] = (char)(0x80 | ((cp >> 12) & 0x3F));
            dst[out_idx++] = (char)(0x80 | ((cp >> 6) & 0x3F));
            dst[out_idx++] = (char)(0x80 | (cp & 0x3F));
        }
    }

    dst[out_idx] = '\0';
    return out_idx;
}

size_t lc_utf8_to_utf32(const char *src, size_t src_len, uint32_t *dst, size_t max_dst) {
    if (!dst || max_dst == 0) return 0;
    if (!src || src_len == 0) {
        dst[0] = 0;
        return 0;
    }

    size_t in_idx = 0;
    size_t out_idx = 0;

    while (in_idx < src_len && src[in_idx] != '\0' && out_idx < max_dst) {
        unsigned char c = (unsigned char)src[in_idx];
        uint32_t cp = 0;
        size_t bytes = 0;

        if (c < 0x80) {
            cp = c;
            bytes = 1;
        } else if ((c & 0xE0) == 0xC0) {
            cp = c & 0x1F;
            bytes = 2;
        } else if ((c & 0xF0) == 0xE0) {
            cp = c & 0x0F;
            bytes = 3;
        } else if ((c & 0xF8) == 0xF0) {
            cp = c & 0x07;
            bytes = 4;
        } else {
            /* Invalid UTF-8 start byte; skip */
            in_idx++;
            continue;
        }

        if (in_idx + bytes > src_len) break;

        bool valid = true;
        for (size_t b = 1; b < bytes; ++b) {
            unsigned char next = (unsigned char)src[in_idx + b];
            if ((next & 0xC0) != 0x80) {
                valid = false;
                break;
            }
            cp = (cp << 6) | (next & 0x3F);
        }

        if (valid) {
            dst[out_idx++] = cp;
            in_idx += bytes;
        } else {
            in_idx++;
        }
    }

    if (out_idx < max_dst) {
        dst[out_idx] = 0;
    }
    return out_idx;
}
