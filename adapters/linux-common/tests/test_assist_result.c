/* test_assist_result.c — D16 (lọc L6) + Q-14 (payload `AssistReport`)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Ba nhóm test:
 *  1. `lc_assist_filter_auto_capitalize` / `lc_assist_auto_capitalize_enabled`
 *     — hàm THUẦN: dựng `ime_assist_result_v1` bằng tay, không cần engine.
 *  2. `lc_assist_report_json` — payload Q-14: đủ trường, escape đúng, tối đa
 *     `LC_ASSIST_REPORT_MAX_ISSUES` lỗi, **không** có trường chứa cả câu,
 *     buffer hẹp ⇒ 0 (không gửi).
 *  3. Hồi quy Q-6 với **engine Rust thật** trên câu corpus
 *     `corpus/assist/vi_basic.ass` → "Xin chào. bạn khỏe không":
 *     `auto_capitalize=false` ⇒ còn L6; `true` ⇒ adapter lọc sạch L6 (cả Mode 1
 *     hiển thị và Mode 2 tự sửa).
 *
 * File này KHÔNG dùng API POSIX (không file/socket/process) nên chạy được cả
 * Windows lẫn Linux — xem cách chạy trên Windows trong báo cáo Q-6/Q-14.
 */

#include "lc_compose.h"
#include "linux_common.h"

#include <assert.h>
#include <stdio.h>
#include <string.h>

/* ---- 1. Lọc L6 (hàm thuần) ---- */

static void set_issue(ime_assist_issue_v1 *iss, uint32_t kind, uint32_t start,
                      uint32_t end, const char *message, const char *suggest,
                      uint32_t auto_fixable, uint32_t confidence) {
    memset(iss, 0, sizeof(*iss));
    iss->kind = kind;
    iss->start = start;
    iss->end = end;
    iss->auto_fixable = auto_fixable;
    iss->confidence = confidence;
    snprintf(iss->message, sizeof(iss->message), "%s", message);
    snprintf(iss->suggest, sizeof(iss->suggest), "%s", suggest);
    iss->suggest_len = (uint32_t)strlen(iss->suggest);
}

static void test_filter_l6(void) {
    /* Câu corpus "Xin chào. bạn khỏe không" → L6 ở span 10..11 ("b") + 1 lỗi
     * lặp từ: 3 lỗi, trong đó 2 lỗi L6. */
    ime_assist_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    res.count = 3;
    set_issue(&res.issues[0], IME_ASSIST_KIND_LOWERCASE_SENTENCE, 10, 11,
              "Chữ thường ở đầu câu", "B", 1, 100);
    set_issue(&res.issues[1], IME_ASSIST_KIND_DUPLICATE_WORD, 0, 3,
              "Lặp từ: \"của của\"", "của", 1, 90);
    set_issue(&res.issues[2], IME_ASSIST_KIND_LOWERCASE_SENTENCE, 20, 21,
              "Chữ thường ở đầu câu", "K", 1, 100);

    /* auto_capitalize BẬT ⇒ bỏ hết L6, giữ nguyên thứ tự lỗi khác. */
    ime_assist_result_v1 on = res;
    assert(lc_assist_filter_auto_capitalize(&on, 1) == 1);
    assert(on.count == 1);
    assert(on.issues[0].kind == IME_ASSIST_KIND_DUPLICATE_WORD);
    assert(on.issues[0].start == 0 && on.issues[0].end == 3);
    assert(strcmp(on.issues[0].suggest, "của") == 0);

    /* auto_capitalize TẮT ⇒ không đổi gì (L6 vẫn được báo/sửa như cũ). */
    ime_assist_result_v1 off = res;
    assert(lc_assist_filter_auto_capitalize(&off, 0) == 3);
    assert(off.count == 3);
    assert(off.issues[0].kind == IME_ASSIST_KIND_LOWERCASE_SENTENCE);
    assert(off.issues[2].kind == IME_ASSIST_KIND_LOWERCASE_SENTENCE);

    /* Toàn L6 ⇒ còn 0 lỗi (adapter phải im lặng, không mở panel). */
    ime_assist_result_v1 only_l6;
    memset(&only_l6, 0, sizeof(only_l6));
    only_l6.abi_version = IME_ABI_VERSION;
    only_l6.count = 1;
    set_issue(&only_l6.issues[0], IME_ASSIST_KIND_LOWERCASE_SENTENCE, 0, 1,
              "Chữ thường ở đầu câu", "X", 1, 100);
    assert(lc_assist_filter_auto_capitalize(&only_l6, 1) == 0);
    assert(only_l6.count == 0);

    /* count rác lớn hơn mảng (ABI mới) ⇒ kẹp về IME_MAX_ASSIST_ISSUES, không OOB. */
    ime_assist_result_v1 bogus = res;
    bogus.count = 99;
    assert(lc_assist_filter_auto_capitalize(&bogus, 0) == IME_MAX_ASSIST_ISSUES);
    assert(bogus.count == IME_MAX_ASSIST_ISSUES);

    /* NULL ⇒ 0, không crash. */
    assert(lc_assist_filter_auto_capitalize(NULL, 1) == 0);
}

static void test_auto_capitalize_default(void) {
    lc_config_state st;
    memset(&st, 0, sizeof(st));

    /* Chưa nạp config ⇒ mặc định của schema là BẬT (Config::default). */
    assert(lc_assist_auto_capitalize_enabled(NULL) == LC_ASSIST_DEFAULT_AUTO_CAPITALIZE);
    assert(lc_assist_auto_capitalize_enabled(&st) == LC_ASSIST_DEFAULT_AUTO_CAPITALIZE);
    assert(LC_ASSIST_DEFAULT_AUTO_CAPITALIZE == 1);

    /* Đã nạp: theo config.json. */
    st.loaded = 1;
    st.auto_capitalize = 0;
    assert(lc_assist_auto_capitalize_enabled(&st) == 0);
    st.auto_capitalize = 1;
    assert(lc_assist_auto_capitalize_enabled(&st) == 1);
}

/* ---- 2. Payload AssistReport (Q-14) ---- */

static void test_report_json(void) {
    ime_assist_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    res.count = 1;
    /* message có `"` + newline: phải escape, nếu không server coi là frame hỏng. */
    set_issue(&res.issues[0], IME_ASSIST_KIND_DUPLICATE_WORD, 4, 6,
              "Lặp \"từ\"\n", "của", 1, 100);

    char buf[LC_ASSIST_REPORT_JSON_MAX];
    size_t len = lc_assist_report_json(&res, 2, buf, sizeof(buf));
    assert(len > 0 && len == strlen(buf));
    const char *want =
        "{\"type\":\"AssistReport\",\"issues\":["
        "{\"kind\":\"duplicate_word\",\"span_start\":4,\"span_end\":6,"
        "\"message\":\"Lặp \\\"từ\\\"\\n\",\"suggestions\":[\"của\"],"
        "\"auto_fixable\":true,\"confidence\":100}"
        "],\"auto_fixed\":2}";
    if (strcmp(buf, want) != 0) {
        fprintf(stderr, "FAIL report json\n  want=%s\n  got =%s\n", want, buf);
        assert(0);
    }

    /* Privacy (D8/S2): KHÔNG có trường mang cả câu. */
    assert(strstr(buf, "\"text\"") == NULL);
    assert(strstr(buf, "\"sentence\"") == NULL);
    assert(strstr(buf, "\"content\"") == NULL);

    /* Tối đa LC_ASSIST_REPORT_MAX_ISSUES lỗi (Q-14: toast chỉ cần vài lỗi đầu). */
    ime_assist_result_v1 many;
    memset(&many, 0, sizeof(many));
    many.abi_version = IME_ABI_VERSION;
    many.count = 5;
    for (uint32_t i = 0; i < 5; ++i) {
        set_issue(&many.issues[i], IME_ASSIST_KIND_DOUBLE_SPACE, i, i + 1,
                  "Khoảng trắng kép", " ", 1, 80);
    }
    len = lc_assist_report_json(&many, 0, buf, sizeof(buf));
    assert(len > 0);
    size_t entries = 0;
    for (const char *p = buf; (p = strstr(p, "\"kind\":\"double_space\"")) != NULL; ++p) {
        entries++;
    }
    assert(entries == LC_ASSIST_REPORT_MAX_ISSUES);

    /* suggestions rỗng (lỗi chỉ để báo) + auto_fixable=false + confidence kẹp 100. */
    ime_assist_result_v1 notify_only;
    memset(&notify_only, 0, sizeof(notify_only));
    notify_only.abi_version = IME_ABI_VERSION;
    notify_only.count = 1;
    set_issue(&notify_only.issues[0], IME_ASSIST_KIND_DOUBLE_NEGATIVE, 0, 2,
              "Phủ định kép", "", 0, 250);
    len = lc_assist_report_json(&notify_only, 0, buf, sizeof(buf));
    assert(len > 0);
    assert(strstr(buf, "\"suggestions\":[]") != NULL);
    assert(strstr(buf, "\"auto_fixable\":false") != NULL);
    assert(strstr(buf, "\"confidence\":100") != NULL);

    /* Kẹp count rác (ABI mới) — không đọc ngoài mảng. */
    ime_assist_result_v1 bogus = many;
    bogus.count = 99;
    len = lc_assist_report_json(&bogus, 0, buf, sizeof(buf));
    assert(len > 0);
    assert(strstr(buf, "\"kind\":\"unknown\"") == NULL);

    /* Không có lỗi / result NULL ⇒ issues:[] (tương thích lời gọi cũ). */
    len = lc_assist_report_json(NULL, 0, buf, sizeof(buf));
    assert(len > 0);
    assert(strcmp(buf, "{\"type\":\"AssistReport\",\"issues\":[],\"auto_fixed\":0}") == 0);

    /* Buffer hẹp ⇒ 0 + out rỗng ⇒ caller KHÔNG gửi (fail-safe). */
    char tiny[32];
    assert(lc_assist_report_json(&res, 0, tiny, sizeof(tiny)) == 0);
    assert(tiny[0] == '\0');
    assert(lc_assist_report_json(&res, 0, NULL, 0) == 0);
    assert(lc_assist_report_json(&res, 0, tiny, 0) == 0);

    /* Kind lạ (mã > bảng hiện tại) → "unknown" (tray chỉ hiển thị). */
    ime_assist_result_v1 unknown;
    memset(&unknown, 0, sizeof(unknown));
    unknown.abi_version = IME_ABI_VERSION;
    unknown.count = 1;
    set_issue(&unknown.issues[0], 999u, 1, 2, "Lạ", "", 0, 10);
    len = lc_assist_report_json(&unknown, 0, buf, sizeof(buf));
    assert(len > 0 && strstr(buf, "\"kind\":\"unknown\"") != NULL);
}

/* ---- 3. Hồi quy Q-6 với engine thật (câu corpus) ---- */

static ime_instance *engine_with(const char *cfg) {
    ime_instance *inst = NULL;
    int32_t rc = ime_instance_new((const uint8_t *)cfg, strlen(cfg), &inst);
    assert(rc == IME_OK && inst);
    return inst;
}

/* Gõ ASCII (Telex) như adapter: mỗi byte = một phím key-down. */
static void type_ascii(ime_instance *inst, const char *s) {
    for (const char *p = s; *p; ++p) {
        ime_key_v1 k;
        memset(&k, 0, sizeof(k));
        k.abi_version = IME_ABI_VERSION;
        k.vk = (uint32_t)(unsigned char)*p;
        k.ch = (uint32_t)(unsigned char)*p;
        k.key_down = 1;
        ime_result_v1 r;
        assert(ime_key(inst, &k, &r) == IME_OK);
    }
}

static int count_kind(const ime_assist_result_v1 *res, uint32_t kind) {
    int n = 0;
    for (uint32_t i = 0; i < res->count && i < IME_MAX_ASSIST_ISSUES; ++i) {
        if (res->issues[i].kind == kind) n++;
    }
    return n;
}

/* Đếm ứng viên mà Mode 1 sẽ hiện (chỉ lỗi có `suggest` — cùng quy tắc với
 * `apply_assist_candidate` / Fcitx5 `AssistCandidateWord`). */
static int count_candidates(const ime_assist_result_v1 *res) {
    int n = 0;
    for (uint32_t i = 0; i < res->count && i < IME_MAX_ASSIST_ISSUES; ++i) {
        if (res->issues[i].suggest[0] != '\0') n++;
    }
    return n;
}

static void check_corpus_sentence(void) {
    const char *cfg_off =
        "{\"config_version\":1,\"auto_capitalize\":false,"
        "\"assist\":{\"mode\":\"highlight\",\"auto_fix\":\"off\"}}";
    ime_instance *inst = engine_with(cfg_off);
    /* "Xin chaof. bajn khoer khoong." (Telex) → "Xin chào. bạn khỏe không."
     * = câu corpus `vi_basic.ass` (L6 ở span 10..11). */
    type_ascii(inst, "Xin chaof. bajn khoer khoong.");

    ime_assist_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    assert(ime_assist_check_instance(inst, &res) == IME_OK);
    assert(count_kind(&res, IME_ASSIST_KIND_LOWERCASE_SENTENCE) == 1);

    char tail[256 * 4 + 1];
    size_t tail_len = 0;
    assert(ime_assist_sentence_tail(inst, 256, tail, sizeof(tail), &tail_len) == IME_OK);
    assert(tail_len > 0);
    tail[tail_len] = '\0';
    assert(strstr(tail, "chào") != NULL); /* câu đã biến đổi đúng */

    /* auto_capitalize=false ⇒ L6 vẫn còn: Mode 1 vẫn có ứng viên cho nó. */
    ime_assist_result_v1 keep = res;
    assert(lc_assist_filter_auto_capitalize(&keep, 0) == res.count);
    assert(count_kind(&keep, IME_ASSIST_KIND_LOWERCASE_SENTENCE) == 1);
    assert(count_candidates(&keep) == count_candidates(&res));

    /* auto_capitalize=true ⇒ adapter lọc sạch L6: Mode 1 không còn ứng viên nào
     * cho L6 (trước fix Q-6, Linux vẫn gợi ý "chữ thường đầu câu" ⇒ trùng chức
     * năng với auto_capitalize). */
    ime_assist_result_v1 filtered = res;
    assert(lc_assist_filter_auto_capitalize(&filtered, 1) == res.count - 1);
    assert(count_kind(&filtered, IME_ASSIST_KIND_LOWERCASE_SENTENCE) == 0);
    assert(count_candidates(&filtered) == count_candidates(&res) - 1);

    ime_instance_free(inst);
}

static void check_engine_autocapitalize_on(void) {
    /* auto_capitalize=true + câu vẫn có L6 (chữ đầu văn bản gõ thường — engine
     * chỉ viết hoa sau ". ! ?" + khoảng trắng, KHÔNG viết hoa chữ đầu đoạn).
     * Đây là ca thật của Q-6 ⇒ adapter phải lọc. */
    const char *cfg = "{\"config_version\":1,\"auto_capitalize\":true,"
                      "\"assist\":{\"mode\":\"highlight\",\"auto_fix\":\"off\"}}";
    ime_instance *inst = engine_with(cfg);
    type_ascii(inst, "xin chaof. bajn khoer khoong.");

    ime_assist_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    assert(ime_assist_check_instance(inst, &res) == IME_OK);
    /* Engine đã viết hoa sau dấu chấm ("Bạn") nhưng chữ đầu đoạn vẫn thường. */
    assert(count_kind(&res, IME_ASSIST_KIND_LOWERCASE_SENTENCE) == 1);

    char tail[256 * 4 + 1];
    size_t tail_len = 0;
    assert(ime_assist_sentence_tail(inst, 256, tail, sizeof(tail), &tail_len) == IME_OK);
    tail[tail_len] = '\0';
    assert(strstr(tail, "Bạn") != NULL); /* auto_capitalize của engine có hiệu lực */

    /* Lọc theo đúng giá trị config ⇒ còn 0 lỗi, 0 ứng viên (im lặng). */
    assert(lc_assist_filter_auto_capitalize(&res, 1) == 0);
    assert(count_candidates(&res) == 0);

    ime_instance_free(inst);
}

int main(void) {
    test_filter_l6();
    test_auto_capitalize_default();
    test_report_json();
    check_corpus_sentence();
    check_engine_autocapitalize_on();
    printf("test_assist_result: OK\n");
    return 0;
}
