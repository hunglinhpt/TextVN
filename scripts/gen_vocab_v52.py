#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Vòng 52 (2026-11-04) — sinh dữ liệu từ vựng prediction:

1. data/vn_syllables.txt  — thay thế data/vn_freq3000.txt:
   3000 âm tiết tần suất cao (giữ NGUYÊN thứ tự + byte, nguồn
   hermitdave/FrequencyWords OpenSubtitles2018) + toàn bộ âm tiết còn thiếu
   lấy từ data/assist/vi-DauMoi.dic (dict FST kiểm chính tả — nguồn đáng tin),
   lọc: chỉ chữ thường, 1 âm tiết (không khoảng trắng), đổi về KIỂU DẤU CŨ
   (hòa/thủy — quy ước dữ liệu prediction, xem core/src/transform/diacritic_style.rs
   quy tắc 5: chỉ vần mở 2 âm oa/oe/uy đổi vị trí dấu), rồi sắp alphabet.
2. data/en_words10k.txt — google-10000-english (no-swears, first20hours),
   lọc a-z, len>=2, lowercase, dedup. Chỉ phục vụ PREDICTION (prefix + Tab EN);
   KHÔNG đụng chính sách restore/assist (en_freq3000.txt giữ nguyên).

Chạy: python scripts/gen_vocab_v52.py  (idempotent — sinh lại byte-identical)
"""
import os
import sys
import unicodedata

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

TONE_MARKS = {0x0300, 0x0301, 0x0309, 0x0303, 0x0323}  # huyền sắc hỏi ngã nặng
# Mục trong vi-DauMoi.dic KHÔNG phải âm tiết đơn hợp lệ (loanword viết liền /
# typo) — core::validate::is_valid_word từ chối, loại khi sinh vn_syllables.
INVALID_DIC_ENTRIES = {
    "email", "internet", "intranet", "tivi", "gram", "gip", "basoi", "tout",
    "palăng", "têt",
    # nhóm viết "qu+y" bằng u-thay-ư (chính tả đúng: quỵ/quỷ/quý/quỹ/quỷ)
    "qùy", "qúy", "qũy", "qụy", "qủy",
"web",
    "xit",
}
VN_VOWELS = set("aăâeêioôơuưy")


def vowel_base(ch: str) -> str:
    """Base vowel của ký tự (bỏ dấu phụ + dấu thanh)."""
    d = unicodedata.normalize("NFD", ch)
    return "".join(c for c in d if c.isalpha())


def is_vowel(ch: str) -> bool:
    return vowel_base(ch) in VN_VOWELS


def split_marks(ch: str):
    """(phần gốc giữ dấu phụ, dấu thanh | None) theo NFD."""
    d = unicodedata.normalize("NFD", ch)
    tone = next((c for c in d if ord(c) in TONE_MARKS), None)
    base = "".join(c for c in d if ord(c) not in TONE_MARKS)
    return base, tone


def join_marks(base: str, tone) -> str:
    return unicodedata.normalize("NFC", base + (tone or ""))


def new_to_old(word: str) -> str:
    """Đổi kiểu dấu MỚI → CŨ cho vần mở 2 âm oa/oe/uy (quy tắc 5
    diacritic_style.rs: (O,A|E)|(U,Y), không phụ âm cuối, dấu đang ở âm 2).

    Bán âm của phụ âm đầu KHÔNG thuộc vần (vowel_span): `u` của `qu`,
    `i` của `gi` khi sau nó còn nguyên âm — "quỳ/quý" giữ nguyên, KHÔNG dời
    dấu sang `u` (bản trước thiếu quy tắc này làm hỏng "quỳ" → "qùy")."""
    chars = list(word)
    vowel_idx = [i for i, c in enumerate(chars) if is_vowel(c)]
    if len(vowel_idx) < 2:
        return word
    last = vowel_idx[-1]
    if last + 1 != len(chars):  # có phụ âm cuối → quy tắc 3, hai kiểu giống nhau
        return word
    run = vowel_idx[-2:]
    # qu/gi: bán âm đầu vần thuộc phụ âm đầu — bỏ khỏi run.
    first = run[0]
    if first > 0:
        prev = chars[first - 1].lower()
        b0 = vowel_base(chars[first])
        if (prev == "q" and b0 in "uư") or (prev == "g" and b0 == "i"):
            run = run[1:]
            if len(run) < 2:
                return word
    pair = (vowel_base(chars[run[0]]), vowel_base(chars[run[1]]))
    glide = (
        pair[0] in "oôơ" and pair[1] in "aăâeê"
    ) or (pair[0] in "uư" and pair[1] == "y")
    if not glide:
        return word
    base1, tone1 = split_marks(chars[run[0]])
    base2, tone2 = split_marks(chars[run[1]])
    if tone2 and not tone1:  # kiểu mới: dấu ở âm 2 → dời về âm 1
        chars[run[0]] = join_marks(base1, tone2)
        chars[run[1]] = join_marks(base2, None)
    return "".join(chars)


def gen_vn_syllables() -> int:
    freq_path = os.path.join(ROOT, "data", "vn_freq3000.txt")
    dic_path = os.path.join(ROOT, "data", "assist", "vi-DauMoi.dic")
    out_path = os.path.join(ROOT, "data", "vn_syllables.txt")

    freq = [l.rstrip("\n") for l in open(freq_path, encoding="utf-8")]
    header = [l for l in freq if l.startswith("#")]
    freq_words = [
        l.strip() for l in freq if l.strip() and not l.strip().startswith("#")
    ]
    seen = set(freq_words)

    dic_added = []
    for line in open(dic_path, encoding="utf-8"):
        w = unicodedata.normalize("NFC", line.strip())
        if not w or w != w.lower() or " " in w or not all(c.isalpha() for c in w):
            continue  # HOA/số/từ ghép — không phải âm tiết gõ hoàn tất
        if len(w) == 1 and w not in "aăâeêioôơuưy":
            continue  # 1 ký tự không phải nguyên âm ("v" trong dic) — vô âm
        if w in INVALID_DIC_ENTRIES:
            continue  # loanword đa âm tiết/typo trong dic — core validate sẽ từ chối
        old = new_to_old(w)
        if old not in seen:
            seen.add(old)
            dic_added.append(old)
    # Audit vòng 52 GHI-NHẬN 6: dic thiếu âm tiết trần ("ổ", "ở", "ạ") mà
    # compound/phrase data dùng ("ổ cứng", "với ạ") — bổ sung thủ công
    # (đều là âm tiết hợp lệ theo core::validate).
    for extra in ("ổ", "ở", "ạ", "ả", "ã", "á", "à"):
        if extra not in seen:
            seen.add(extra)
            dic_added.append(extra)
    dic_added.sort()

    with open(out_path, "w", encoding="utf-8", newline="\n") as f:
        f.write("# Âm tiết tiếng Việt — dữ liệu prediction/Tab-completion (vòng 52)\n")
        f.write("# 3000 đầu = tần suất cao nhất (hermitdave/FrequencyWords, OPUS 2018,\n")
        f.write("#   thứ tự + byte giữ nguyên từ vn_freq3000.txt); phần sau = bổ sung từ\n")
        f.write("#   data/assist/vi-DauMoi.dic (đã lọc thường/1 âm tiết, kiểu dấu CŨ).\n")
        f.write("# Sinh bởi scripts/gen_vocab_v52.py — KHÔNG sửa tay, chạy lại script.\n")
        for h in header:
            f.write(h + "\n")
        for w in freq_words:
            f.write(w + "\n")
        for w in dic_added:
            f.write(w + "\n")
    print(f"vn_syllables.txt: {len(freq_words)} freq + {len(dic_added)} dic = {len(freq_words) + len(dic_added)}")
    return len(freq_words) + len(dic_added)


def gen_en_words() -> int:
    g10k = os.path.join(os.environ.get("TEMP", "/tmp"), "g10k.txt")
    out_path = os.path.join(ROOT, "data", "en_words10k.txt")
    words, seen = [], set()
    for line in open(g10k, encoding="utf-8"):
        w = line.strip().lower()
        if len(w) < 2 or not w.isascii() or not w.isalpha():
            continue
        if w in seen:
            continue
        seen.add(w)
        words.append(w)
    with open(out_path, "w", encoding="utf-8", newline="\n") as f:
        f.write("# Từ tiếng Anh tần suất cao (google-10000-english, first20hours,\n")
        f.write("# https://github.com/first20hours/google-10000-english — public domain;\n")
        f.write("# biến thể no-swears). Vòng 52: nguồn PREDICTION (prefix/Tab EN).\n")
        f.write("# KHÔNG dùng cho chính sách restore/assist — en_freq3000.txt giữ riêng.\n")
        f.write("# Sinh bởi scripts/gen_vocab_v52.py — KHÔNG sửa tay.\n")
        for w in words:
            f.write(w + "\n")
    print(f"en_words10k.txt: {len(words)} từ")
    return len(words)


def gen_vn_compounds() -> int:
    """Mở rộng data/vn_compounds.txt: giữ nguyên các mục cũ (weight tay),
    thêm ~1500 từ ghép/cụm từ đơn vị văn phòng — văn phòng, hành chính, kế
    toán, nhân sự, IT, kinh doanh, giao tiếp, đời sống (vòng 52)."""
    path = os.path.join(ROOT, "data", "vn_compounds.txt")
    old = {}

    # Whitelist ký tự tiếng Việt — chặn dòng rác (CJK/ký tự lạ) lọt vào file cũ.
    VN_CHARS = set(range(0x20, 0x7F))
    VN_CHARS.update([0x00C0, 0x00C1, 0x00C2, 0x00C3, 0x00C8, 0x00C9, 0x00CA, 0x00CC,
                     0x00CD, 0x00D2, 0x00D3, 0x00D4, 0x00D5, 0x00D9, 0x00DA, 0x00DD,
                     0x00E0, 0x00E1, 0x00E2, 0x00E3, 0x00E8, 0x00E9, 0x00EA, 0x00EC,
                     0x00ED, 0x00F2, 0x00F3, 0x00F4, 0x00F5, 0x00F9, 0x00FA, 0x00FD])
    VN_CHARS.update(range(0x1EA0, 0x1EFA))
    VN_CHARS.update([0x0102, 0x0103, 0x0110, 0x0111, 0x0128, 0x0129, 0x0168, 0x0169,
                     0x01A0, 0x01A1, 0x01AF, 0x01B0])

    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        w = parts[0].strip()
        if any(ord(c) not in VN_CHARS and not c.isspace() for c in w):
            print(f"  bỏ mục ký tự lạ: {w!r}")
            continue
        old[w] = int(parts[1]) if len(parts) > 1 and parts[1].strip().isdigit() else 50

    # Audit vòng 52 GHI-NHẬN 5: mục HOA đầu từ ("RAM", "CV xin việc") không bao
    # giờ khớp typed-lowercase → hạ về lowercase để completion sống lại.
    for dead in ("RAM", "CV xin việc"):
        if dead in old:
            old[dead.lower()] = old.pop(dead)

    # (nhóm, [(từ ghép, weight)]): weight 100 = văn phòng dùng nhiều nhất.
    groups = [
        ("văn phòng - công văn", [
            ("công văn", 96), ("tờ trình", 96), ("nghị quyết", 94), ("chỉ thị", 88),
            ("quyết định", 100), ("thông tư", 90), ("nghị định", 90),
            ("quy chế", 86), ("quy định", 100), ("quy trình", 100), ("hướng dẫn", 96),
            ("đề án", 84), ("dự thảo", 88), ("dự án", 100), ("phương án", 92),
            ("kế hoạch", 100), ("chương trình", 96), ("lộ trình", 88), ("định hướng", 88),
            ("mục tiêu", 100), ("nhiệm vụ", 100), ("trách nhiệm", 94), ("thẩm quyền", 84),
            ("phạm vi", 88), ("căn cứ pháp lý", 82), ("lý do", 88), ("lãnh đạo", 96),
            ("cấp trên", 78), ("cấp dưới", 74), ("phòng ban", 94), ("bộ phận", 94),
            ("chuyên môn", 90), ("nghiệp vụ", 92), ("công việc", 100), ("đầu việc", 84),
            ("việc đang làm", 78), ("vấn đề", 100), ("vướng mắc", 82), ("khó khăn", 90),
            ("ý kiến", 94), ("đề xuất", 96), ("kiến nghị", 88), ("phản ánh", 84),
            ("phản hồi", 100), ("góp ý", 88), ("phê bình", 78), ("tự phê bình", 74),
            ("đánh giá", 96), ("nhận xét", 92), ("kết luận", 94), ("khuyến nghị", 86),
            ("đề nghị", 96), ("kiến nghị", 88), ("thỉnh cầu", 74), ("đơn từ", 84),
            ("giấy tờ", 90), ("chứng từ", 92), ("hồ sơ", 100), ("sổ sách", 86),
            ("văn bản", 96), ("tài liệu", 100), ("bản sao", 84), ("bản gốc", 86),
            ("bản sao y", 78), ("bản mềm", 80), ("bản cứng", 76), ("file đính kèm", 86),
            ("đính kèm", 94), ("phụ lục", 86), ("biểu mẫu", 92), ("mẫu đơn", 84),
            ("mẫu số", 78), ("con dấu", 84), ("chữ ký", 92), ("chữ ký số", 90),
            ("đóng dấu", 80), ("trình ký", 84), ("phê duyệt", 98), ("phê bình", 78),
            ("xét duyệt", 88), ("thẩm định", 86), ("kiểm duyệt", 82), ("duyệt đơn", 82),
            ("trình duyệt", 80), ("công bố", 84), ("ban hành", 92), ("có hiệu lực", 88),
            ("áp dụng", 92), ("sửa đổi", 88), ("bổ sung", 90), ("thay thế", 86),
            ("bãi bỏ", 78), ("hết hiệu lực", 82), ("tổng hợp", 96), ("phân tích", 100),
            ("đánh giá", 96), ("so sánh", 90), ("thống kê", 94), ("bảng biểu", 88),
            ("biểu đồ", 90), ("số liệu", 100), ("chỉ số", 90), ("thành phần", 84),
            ("tỷ trọng", 78), ("tỷ lệ", 94), ("tỷ suất", 78), ("mức độ", 90),
            ("khối lượng", 86), ("chất lượng", 100), ("hiệu quả", 100), ("hiệu suất", 90),
            ("năng suất", 92), ("tiến độ", 100), ("thời hạn", 94), ("hạn chót", 88),
            ("deadline", 88), ("gia hạn", 90), ("kéo dài", 82), ("rút ngắn", 84),
            ("tạm dừng", 84), ("tạm hoãn", 82), ("hoãn lại", 82), ("tiếp tục", 96),
            ("khởi động", 84), ("khởi công", 82), ("khánh thành", 80), ("bàn giao", 96),
            ("nghiệm thu", 90), ("đưa vào sử dụng", 82), ("vận hành", 94),
            ("bảo trì", 86), ("bảo dưỡng", 82), ("sửa chữa", 88), ("nâng cấp", 92),
            ("kiểm tra", 100), ("kiểm soát", 94), ("giám sát", 94), ("giám định", 82),
            ("soi chiếu", 74), ("đối chiếu", 92), ("rà soát", 90), ("kiểm điểm", 78),
            ("xác nhận", 100), ("xác minh", 86), ("chứng thực", 82), ("công chứng", 82),
            ("thông qua", 88), ("chấp thuận", 84), ("đồng ý", 96), ("tán thành", 80),
            ("bày tỏ", 78), ("phát biểu", 92), ("phát ngôn", 76), ("phát biểu ý kiến", 72),
        ]),
        ("họp hành - giao tiếp nội bộ", [
            ("cuộc họp", 100), ("họp giao ban", 88), ("họp hội đồng", 84),
            ("họp trực tuyến", 86), ("họp nhóm", 90), ("họp khẩn", 88), ("họp nhanh", 86),
            ("họp mặt", 82), ("họp bàn", 80), ("họp triển khai", 84), ("họp tổng kết", 84),
            ("lịch họp", 92), ("phòng họp", 92), ("thành phần tham dự", 80),
            ("chủ trì", 88), ("thư ký", 86), ("biên bản họp", 92), ("nội dung họp", 90),
            ("bảng chia sẻ", 76), ("trình chiếu", 88), ("bản trình bày", 88), ("slide", 88),
            ("thuyết trình", 94), ("trình bày", 96), ("bày tỏ quan điểm", 82),
            ("trao đổi", 96), ("thảo luận", 100), ("bàn bạc", 88), ("cạnh tranh", 88),
            ("đấu giá", 74), ("xung đột", 82), ("hòa giải", 78), ("dàn xếp", 74),
            ("thông cảm", 84), ("thấu hiểu", 86), ("hỗ trợ", 100), ("giúp đỡ", 92),
            ("giúp việc", 76), ("hợp tác", 98), ("phối hợp", 96), ("kết hợp", 92),
            ("phân công", 94), ("phân chia", 88), ("chia sẻ", 94), ("tiếp nhận", 94),
            ("chuyển giao", 88), ("ủy quyền", 84), ("thay mặt", 90), ("đại diện", 94),
            ("thay đổi", 96), ("điều chỉnh", 92), ("cập nhật", 100), ("nâng cấp", 92),
            ("thông báo", 100), ("công văn", 96), ("lời mời", 88), ("thư mời", 88),
            ("giấy mời", 84), ("thiệp mời", 74), ("triệu tập", 74), ("mời họp", 88),
            ("xin phép", 94), ("cho phép", 90), ("giấy phép", 92), ("được phép", 88),
            ("nghiêm cấm", 76), ("từ chối", 90), ("khước từ", 76),
            ("chấp nhận", 94), ("đồng thuận", 88), ("khó chịu", 80), ("vui lòng", 100),
            ("rất vui", 88), ("rất tiếc", 86), ("xin lỗi", 100), ("cảm ơn", 100),
            ("xin chào", 100), ("chào buổi sáng", 88), ("chào tạm biệt", 84),
            ("hẹn gặp lại", 92), ("chúc sức khỏe", 84), ("chúc ngủ ngon", 80),
            ("chúc mừng", 100), ("chúc thọ", 74), ("trân trọng", 98), ("kính thư", 82),
            ("thân gửi", 88), ("thân mến", 84), ("kính thưa", 86), ("thưa anh", 84),
            ("thưa chị", 84), ("thưa quý vị", 84), ("kính dâng", 70),
            ("trân trọng kính mời", 82), ("rất mong", 92), ("hy vọng", 92),
            ("tin rằng", 80), ("tin chắc", 82), ("cho biết", 90), ("nêu rõ", 84),
            ("nhấn mạnh", 90), ("lưu ý", 98), ("chú ý", 92), ("chú trọng", 86),
            ("đặc biệt", 92), ("cực kỳ", 84), ("hết sức", 86), ("tối đa", 88),
        ]),
        ("kế toán - tài chính", [
            ("hóa đơn", 100), ("hóa đơn đỏ", 90), ("hóa đơn điện tử", 92),
            ("hóa đơn giấy", 78), ("biên lai", 84), ("phiếu thu", 88), ("phiếu chi", 88),
            ("phiếu nhập kho", 82), ("phiếu xuất kho", 82), ("phiếu giao hàng", 84),
            ("chứng từ kế toán", 88), ("sổ cái", 84), ("sổ kế toán", 86),
            ("sổ phụ ngân hàng", 86), ("sổ tay", 82), ("bút toán", 84), ("định khoản", 80),
            ("bảng cân đối", 86), ("báo cáo tài chính", 94), ("báo cáo thuế", 90),
            ("báo cáo lãi lỗ", 82), ("báo cáo lưu chuyển", 78), ("báo cáo nội bộ", 82),
            ("tờ khai thuế", 86), ("thuế môn bài", 78), ("thuế thu nhập", 88),
            ("thuế giá trị", 80), ("thuế suất", 78), ("miễn thuế", 76), ("giảm thuế", 78),
            ("hoàn thuế", 78), ("nộp thuế", 86), ("trốn thuế", 74), ("truy thu", 74),
            ("công nợ", 94), ("nợ phải thu", 84), ("nợ phải trả", 84), ("nợ xấu", 80),
            ("trả nợ", 84), ("đòi nợ", 78), ("vay nợ", 80), ("vay vốn", 86),
            ("tiền mặt", 92),
            ("tiền lương", 92), ("tiền thưởng", 88), ("tiền phạt", 80), ("tiền cọc", 84),
            ("tiền điện", 84), ("tiền nước", 82), ("tiền nhà", 82), ("tiền điện thoại", 80),
            ("lương tháng", 90), ("lương năm", 84), ("lương cơ bản", 90), ("lương khoán", 80),
            ("lương thưởng", 92), ("tính lương", 88), ("trả lương", 88), ("ứng lương", 82),
            ("bảng lương", 90), ("thang lương", 86), ("bậc lương", 84), ("phụ cấp", 88),
            ("trợ cấp", 84), ("cấp bậc", 78), ("thưởng nóng", 80), ("thưởng lễ tết", 82),
            ("thưởng quý", 82), ("thưởng năm", 84), ("tính thưởng", 76), ("ngân sách", 100),
            ("dự toán", 86), ("quyết toán", 86), ("tổng kết", 90), ("kê khai", 88),
            ("kê đơn", 70), ("kê khai thuế", 86), ("khấu trừ", 82), ("trích trước", 76),
            ("trích khấu hao", 78), ("khấu hao", 82), ("hao mòn", 74), ("tài sản", 96),
            ("tài sản cố định", 88), ("tài sản lưu động", 78), ("tài sản thế chấp", 82),
            ("bán tài sản", 76), ("tăng tài sản", 74), ("vốn chủ sở hữu", 84),
            ("vốn điều lệ", 84), ("vốn đầu tư", 88), ("vốn lưu động", 80),
            ("góp vốn", 82), ("rút vốn", 80), ("thu hồi vốn", 80), ("chia cổ tức", 82),
            ("cổ phần", 90), ("cổ phiếu", 88), ("cổ đông", 88), ("thị trường chứng khoán", 84),
            ("tăng vốn", 78), ("lợi nhuận", 100), ("lỗ vốn", 78), ("lãi gộp", 80),
            ("lãi ròng", 78), ("lãi kép", 78), ("lãi suất", 92), ("lãi xuất", 70),
            ("số dư", 90), ("số dư đầu kỳ", 84), ("số dư cuối kỳ", 84), ("số dư nợ", 82),
            ("dư nợ", 80), ("dư địa", 74), ("đơn giá", 88), ("đơn hàng", 100),
            ("giá bán", 90), ("giá mua", 86), ("giá thành", 86), ("giá vốn", 84),
            ("giá trị", 100), ("giá cả", 92), ("tăng giá", 84), ("giảm giá", 88),
            ("khuyến mãi", 92), ("khuyến mại", 90), ("ưu đãi", 94), ("trợ giá", 74),
            ("bình ổn giá", 76), ("chênh lệch", 86), ("thanh toán", 100),
            ("thanh toán trước", 84), ("thanh toán sau", 82), ("thanh toán online", 84),
            ("thanh toán dư", 76), ("thanh lý", 84), ("thanh lý hợp đồng", 86),
            ("tạm ứng", 86), ("tạm thu", 76), ("tạm nợ", 78), ("ứng trước", 80),
            ("trả sau", 80), ("trả trước", 82), ("trả góp", 86), ("góp vốn", 82),
            ("kỳ hạn", 86), ("hạn mức", 84), ("hạn thanh toán", 88), ("hạn chót", 88),
            ("quá hạn", 86), ("trễ hạn", 78), ("đáo hạn", 78), ("số dư kỳ", 76),
            ("tỷ giá", 88), ("tỷ giá USD", 78), ("ngoại tệ", 80), ("chuyển tiền", 92),
            ("chuyển khoản", 100), ("nhận chuyển", 74), ("thu hộ", 78), ("chi hộ", 78),
            ("ngân hàng", 100), ("ngân quỹ", 76), ("quỹ dự phòng", 82), ("quỹ lương", 80),
            ("ngân khố", 74), ("kho bạc", 78), ("tín dụng", 86), ("tín phiếu", 74),
            ("thế chấp", 82), ("cầm cố", 74), ("bảo lãnh", 86), ("bảo hiểm", 96),
            ("bảo hiểm xã hội", 92), ("bảo hiểm y tế", 88), ("bảo hiểm thất nghiệp", 84),
            ("bảo hiểm nhân thọ", 82), ("bảo hiểm tài sản", 80), ("mức đóng", 82),
            ("mức phạt", 78), ("vi phạm", 92), ("xử phạt", 84), ("hồ sơ thuế", 80),
        ]),
        ("nhân sự - tổ chức", [
            ("nhân sự", 100), ("nhân viên", 100), ("nhân công", 82), ("nhân lực", 90),
            ("nhân tài", 82), ("nhân khẩu", 76), ("cán bộ", 94), ("công nhân", 88),
            ("công chức", 84), ("viên chức", 80), ("lao động", 96), ("người lao động", 90),
            ("người quản lý", 84), ("quản lý", 100), ("cấp quản lý", 82), ("quản đốc", 74),
            ("giám đốc", 96), ("tổng giám đốc", 90), ("phó giám đốc", 88), ("giám đốc điều hành", 84),
            ("ban giám đốc", 92), ("ban lãnh đạo", 90), ("ban biên tập", 74),
            ("ban tổ chức", 82), ("ban chấp hành", 80), ("ban thư ký", 76),
            ("hội đồng", 90), ("hội đồng quản trị", 86), ("hội đồng sơ thẩm", 72),
            ("tổ trưởng", 78), ("trưởng phòng", 92), ("phó phòng", 86), ("trưởng nhóm", 90),
            ("trưởng ban", 84), ("chủ tịch", 90), ("phó chủ tịch", 84), ("thư ký hội đồng", 74),
            ("kế toán trưởng", 88), ("kiểm soát viên", 74), ("thực tập sinh", 88),
            ("nhân viên thử việc", 82), ("nhân viên chính thức", 84), ("nhân viên bán thời gian", 80),
            ("nhân viên toàn thời gian", 76), ("freelancer", 82), ("cộng tác viên", 84),
            ("tuyển dụng", 100), ("tuyển sinh", 84), ("tuyển chọn", 82), ("tuyển mộ", 74),
            ("phỏng vấn", 96), ("phỏng vấn sơ bộ", 80), ("phỏng vấn vòng cuối", 80),
            ("buổi phỏng vấn", 84), ("hồ sơ ứng tuyển", 84), ("đơn ứng tuyển", 84),
            ("sơ yếu lý lịch", 86), ("giấy tờ tùy thân", 82),
            ("thư mời phỏng vấn", 80), ("thư từ chối", 74), ("thư mời làm việc", 80),
            ("thư báo nhận việc", 78), ("thư nhận việc", 80), ("thư cảm ơn", 84),
            ("đề nghị tuyển", 80), ("nhu cầu tuyển", 76), ("vị trí tuyển", 82),
            ("mô tả công việc", 90), ("bản mô tả", 80), ("yêu cầu công việc", 86),
            ("quyền lợi", 90), ("chế độ đãi ngộ", 86), ("đãi ngộ", 80), ("phúc lợi", 86),
            ("thử việc", 88), ("chính thức", 96), ("nhận việc", 90), ("nghỉ việc", 88),
            ("xin nghỉ việc", 88), ("đơn xin nghỉ", 84), ("thôi việc", 82),
            ("chấm dứt hợp đồng", 84), ("sa thải", 76), ("cắt giảm", 80),
            ("nghỉ phép", 100), ("nghỉ thai sản", 84), ("nghỉ ốm", 84), ("nghỉ không lương", 82),
            ("nghỉ bù", 84), ("nghỉ rota", 70), ("xin nghỉ phép", 96), ("đăng ký nghỉ", 84),
            ("phép năm", 88), ("ngày phép", 84), ("dư phép", 74), ("hết phép", 76),
            ("chấm công", 90), ("bảng chấm công", 88), ("quét thẻ", 74), ("tang chức", 70),
            ("đào tạo", 100), ("bồi dưỡng", 84), ("huấn luyện", 82), ("đào tạo nội bộ", 84),
            ("đào tạo lại", 78), ("kỹ năng mềm", 86), ("kỹ năng cứng", 84),
            ("năng lực", 98), ("chuyên cần", 74), ("thi đua", 80), ("khen thưởng", 90),
            ("kỷ luật", 86), ("khiển trách", 76), ("cảnh cáo", 80), ("hồ sơ nhân sự", 88),
            ("hợp đồng lao động", 94), ("hợp đồng thử việc", 84), ("hợp đồng xác định thời hạn", 80),
            ("cam kết", 94), ("cam kết thực hiện", 84), ("thỏa thuận", 94),
            ("thỏa thuận lao động", 82), ("biên bản làm việc", 84), ("quy chế thưởng", 78),
            ("nội quy", 86), ("nội quy lao động", 84), ("cơ cấu tổ chức", 84),
            ("sơ đồ tổ chức", 86), ("chức danh", 88), ("chức vụ", 90), ("nhiệm kỳ", 78),
            ("bổ nhiệm", 88), ("bãi nhiệm", 74), ("thuyên chuyển", 82), ("luân chuyển", 78),
            ("thăng chức", 84), ("giáng chức", 74), ("nâng bậc", 78), ("xếp loại", 86),
            ("đánh giá nhân viên", 86), ("đánh giá định kỳ", 84), ("đánh giá cuối kỳ", 84),
            ("mục tiêu công việc", 80), ("đánh giá KPI", 80), ("phỏng vấn sơ tuyển", 74),
        ]),
        ("công nghệ - IT", [
            ("phần mềm", 100), ("phần cứng", 96), ("phần mềm độc hại", 82),
            ("mã nguồn", 94), ("mã hóa", 88), ("mã lỗi", 82), ("mã vạch", 82),
            ("mã QR", 84), ("thư viện mã nguồn", 76), ("kho mã", 78),
            ("trình duyệt", 96), ("trình biên dịch", 84), ("trình quản lý", 86),
            ("trình cài đặt", 84), ("trình gỡ lỗi", 76), ("trình chạy nền", 74),
            ("cơ sở dữ liệu", 96), ("truy vấn", 90), ("bảng dữ liệu", 84),
            ("cột dữ liệu", 78), ("dòng dữ liệu", 78), ("khóa chính", 82), ("khóa ngoại", 80),
            ("chỉ mục", 80), ("sao lưu", 94), ("phục hồi", 90), ("khôi phục", 92),
            ("đồng bộ", 92), ("bất đồng bộ", 80), ("xử lý ngoại lệ", 80),
            ("xử lý dữ liệu", 88), ("xử lý ảnh", 80), ("xử lý ngôn ngữ", 80),
            ("hệ thống", 100), ("hệ điều hành", 92), ("hệ thống mạng", 88),
            ("hệ thống tệp", 78), ("hệ thống con", 76), ("hệ thống giám sát", 80),
            ("mạng nội bộ", 84), ("mạng xã hội", 92), ("mạng không dây", 80),
            ("địa chỉ mạng", 78), ("tường lửa", 82), ("định tuyến", 78), ("cổng kết nối", 80),
            ("máy chủ", 98), ("máy khách", 86), ("máy ảo", 88), ("máy in", 92),
            ("máy quét", 80), ("máy chiếu", 84), ("thiết bị", 100), ("thiết bị đầu cuối", 78),
            ("thiết bị ngoại vi", 76), ("thiết bị đeo", 74), ("điện thoại", 98),
            ("máy tính", 100), ("laptop", 90), ("bàn phím", 88), ("chuột máy tính", 82),
            ("màn hình", 96), ("ổ cứng", 86), ("ổ đĩa", 86), ("bộ nhớ", 92),
            ("bộ xử lý", 82), ("bộ vi xử lý", 78), ("chip xử lý", 78),
            ("file", 100), ("thư mục", 96), ("đường dẫn", 88), ("đuôi file", 82),
            ("nén file", 80), ("giải nén", 82), ("tải file", 82), ("tải xuống", 88),
            ("tải lên", 86), ("chia sẻ file", 84), ("quyền truy cập", 88),
            ("đăng nhập", 100), ("đăng xuất", 88), ("đăng ký", 100), ("tài khoản", 100),
            ("mật khẩu", 98), ("xác thực", 88), ("xác thực hai lớp", 76),
            ("quên mật khẩu", 84), ("đổi mật khẩu", 84), ("khôi phục mật khẩu", 80),
            ("phiên đăng nhập", 78), ("bảo mật", 100), ("rò rỉ dữ liệu", 82),
            ("tin tặc", 78), ("vi rút", 76), ("phần mềm gián điệp", 74),
            ("cập nhật bản vá", 78), ("bản vá lỗi", 82), ("sửa lỗi", 96),
            ("gỡ lỗi", 80), ("bản dựng", 84), ("triển khai", 98), ("phát hành", 90),
            ("phát hành bản mới", 80), ("phiên bản", 100), ("bản cập nhật", 88),
            ("bản chính thức", 82), ("bản dùng thử", 80), ("kiểm thử", 94),
            ("kiểm thử tự động", 82), ("kiểm thử hiệu năng", 78), ("kiểm thử bảo mật", 78),
            ("mã nguồn mở", 84), ("trí tuệ nhân tạo", 92), ("học máy", 86),
            ("học sâu", 80), ("mạng nơ ron", 78), ("dữ liệu lớn", 84),
            ("điện toán đám mây", 84), ("lưu trữ đám mây", 80), ("danh tính số", 72),
            ("giao diện", 96), ("giao diện người dùng", 84), ("trải nghiệm người dùng", 84),
            ("trò chuyện trực tuyến", 76), ("hộp thư", 82), ("hộp thư đến", 86),
            ("thư rác", 84), ("thư nháp", 78), ("chuyển tiếp thư", 76),
            ("địa chỉ thư điện tử", 78), ("danh bạ", 80), ("lịch làm việc", 88),
            ("lịch cá nhân", 80), ("họp online", 84), ("chia sẻ màn hình", 82),
            ("bảng tin", 78), ("thanh công cụ", 76), ("thanh trạng thái", 76),
            ("cửa sổ", 90), ("cửa sổ bật lên", 76), ("thanh cuộn", 74),
            ("nút bấm", 84), ("biểu tượng", 88), ("biểu đồ trực quan", 78),
            ("bản in", 84), ("bản nháp", 88), ("bản cuối", 84), ("xuất bản", 86),
            ("đóng gói", 86), ("cài đặt", 100), ("gỡ cài đặt", 84), ("cấu hình", 98),
            ("thiết lập", 94), ("cổng USB", 80), ("bộ nhớ đệm", 78), ("bộ nhớ ngoài", 74),
        ]),
        ("kinh doanh - bán hàng", [
            ("khách hàng", 100), ("khách hàng tiềm năng", 86), ("khách hàng thân thiết", 84),
            ("khách hàng mới", 80), ("khách hàng cũ", 76), ("quý khách", 98),
            ("quý khách hàng", 98), ("thượng khách", 70), ("nhà cung cấp", 94),
            ("nhà phân phối", 86), ("nhà đầu tư", 88), ("nhà thầu", 84),
            ("đối tác", 98), ("đối thủ", 88), ("đối thủ cạnh tranh", 86), ("đối trọng", 70),
            ("đại lý", 86), ("chi nhánh", 90), ("văn phòng đại diện", 82),
            ("trung tâm bảo hành", 80), ("cửa hàng", 94), ("chợ đầu mối", 74),
            ("sàn giao dịch", 82), ("sàn thương mại", 84), ("gian hàng", 84),
            ("đơn hàng", 100), ("giỏ hàng", 88), ("đặt hàng", 94), ("đặt trước", 84),
            ("hủy đơn", 82), ("hủy hàng", 78), ("trả hàng", 82), ("đổi hàng", 82),
            ("giao hàng", 96), ("giao hàng tận nơi", 82), ("nhận hàng", 90),
            ("kiểm hàng", 76), ("gói hàng", 78), ("đóng gói hàng", 76), ("vận đơn", 76),
            ("vận chuyển", 92), ("phí vận chuyển", 84), ("freeship", 84),
            ("giao nhanh", 82), ("giao hỏa tốc", 78), ("tình trạng đơn", 78),
            ("xác nhận đơn", 82), ("xuất hóa đơn", 84), ("thanh toán khi nhận hàng", 82),
            ("sale", 88), ("giảm giá sốc", 78), ("mã giảm giá", 84), ("mã ưu đãi", 82),
            ("voucher", 82), ("phiếu quà tặng", 76), ("chương trình khuyến mãi", 82),
            ("bán hàng", 100), ("bán buôn", 80), ("bán lẻ", 84), ("bán sỉ", 80),
            ("đại sứ thương hiệu", 74), ("thương hiệu", 98), ("nhãn hiệu", 88),
            ("thương hiệu riêng", 78), ("xây dựng thương hiệu", 80), ("định vị thương hiệu", 78),
            ("tiếp thị", 92), ("quảng cáo", 96), ("ra mắt sản phẩm", 82),
            ("sản phẩm", 100), ("sản phẩm mới", 88), ("dịch vụ", 100), ("dịch vụ khách hàng", 88),
            ("chăm sóc khách hàng", 92), ("hồ sơ khách hàng", 80), ("dữ liệu khách hàng", 80),
            ("tư vấn", 98), ("tư vấn viên", 84), ("báo giá", 94), ("báo giá chính thức", 80),
            ("yêu cầu báo giá", 80), ("soạn báo giá", 76), ("chốt đơn", 92),
            ("chốt hợp đồng", 88), ("ký hợp đồng", 92), ("hợp đồng", 100),
            ("phụ lục hợp đồng", 82), ("dự thảo hợp đồng", 80), ("rút khỏi hợp đồng", 72),
            ("đàm phán", 90), ("thương lượng", 78), ("giá sàn", 74), ("chiết khấu", 86),
            ("hoa hồng", 84), ("doanh số", 88), ("doanh thu", 100), ("lợi nhuận", 100),
            ("chi phí", 100), ("chi phí vận hành", 84), ("chi phí cố định", 78),
            ("chi phí phát sinh", 82), ("chi phí dự kiến", 78), ("khoản mục chi", 78),
            ("khoản mục thu", 76), ("dòng tiền", 88), ("lưu chuyển tiền tệ", 78),
            ("tăng trưởng", 94), ("tăng trưởng âm", 74), ("sụt giảm", 80),
            ("thị trường", 100), ("thị phần", 90), ("thị trường nội địa", 80),
            ("thị trường quốc tế", 82), ("thị trường ngách", 74), ("xâm nhập thị trường", 76),
            ("mở rộng thị trường", 80), ("thấu hiểu khách hàng", 78), ("hành trình khách hàng", 76),
            ("trải nghiệm khách hàng", 80), ("giữ chân khách hàng", 80),
            ("tỷ lệ chuyển đổi", 82), ("đầu mối", 78), ("nguồn khách", 76),
            ("cuộc gọi", 88), ("cuộc gọi nhỡ", 76), ("gọi lại", 82), ("hotline", 88),
            ("tổng đài", 82), ("khiếu nại", 90), ("giải quyết khiếu nại", 82), ("bảo hành", 92), ("hậu mãi", 78),
            ("phiếu bảo hành", 82), ("thẻ thành viên", 80), ("tích điểm", 80),
        ]),
        ("giáo dục - đào tạo", [
            ("bài giảng", 88), ("bài tập", 96), ("bài tập về nhà", 88),
            ("bài kiểm tra", 94), ("bài thi", 88), ("bài luận", 82), ("bài thuyết trình", 84),
            ("giáo án", 82), ("giáo trình", 84), ("giáo viên", 96), ("giảng viên", 88),
            ("trợ giảng", 76), ("học sinh", 96), ("sinh viên", 96), ("học viên", 86),
            ("học phí", 88), ("học bổng", 84), ("học kỳ", 86), ("năm học", 90),
            ("thời khóa biểu", 84), ("tín chỉ", 80), ("điểm số", 86), ("điểm danh", 80),
            ("xếp loại học lực", 78), ("nghỉ học", 84), ("đi học muộn", 76),
            ("buổi học", 84), ("lớp học", 90), ("phòng học", 84), ("trường học", 92),
            ("nộp bài", 88), ("chấm bài", 84), ("sửa bài", 78), ("bài làm", 82),
            ("đề cương", 80), ("đề thi", 84), ("đáp án", 84), ("thang điểm", 76),
            ("tốt nghiệp", 88), ("tuyển sinh đại học", 78), ("hồ sơ nhập học", 80),
        ]),
        ("y tế - sức khỏe", [
            ("khám bệnh", 86), ("khám định kỳ", 82), ("khám tổng quát", 82),
            ("khám sức khỏe", 84), ("bệnh án", 78), ("toa thuốc", 76), ("kê đơn", 78),
            ("viện phí", 76), ("viện viên", 70), ("bệnh viện", 92), ("phòng khám", 90),
            ("cơ sở y tế", 82), ("nhân viên y tế", 82), ("y tá", 76), ("bác sĩ", 92),
            ("bảo hiểm y tế", 86), ("sổ khám bệnh", 74), ("lịch hẹn khám", 78),
            ("tái khám", 78), ("chuyển viện", 74), ("cấp cứu", 84), ("lây nhiễm", 76),
            ("dịch bệnh", 82), ("phòng dịch", 76), ("cách ly", 78), ("thuốc men", 78),
        ]),
        ("pháp lý - giấy tờ", [
            ("giấy tờ", 94), ("giấy chứng nhận", 88), ("giấy phép kinh doanh", 84),
            ("giấy ủy quyền", 84), ("giấy giới thiệu", 82), ("giấy xác nhận", 84),
            ("giấy tờ tùy thân", 80), ("căn cước", 86), ("hộ khẩu", 84), ("hộ chiếu", 84),
            ("thẻ căn cước", 82), ("sổ đỏ", 78), ("sổ hồng", 76), ("hợp đồng thuê", 80),
            ("hợp đồng mua bán", 84), ("hợp đồng nguyên tắc", 78), ("hợp đồng điều khoản", 76),
            ("công chứng nhà nước", 76), ("tòa án", 84), ("viện kiểm sát", 78),
            ("luật sư", 86), ("đơn kiện", 76), ("đơn khởi kiện", 78), ("bản án", 80),
            ("quyết định xử", 70), ("bản án sơ thẩm", 74), ("kháng cáo", 74),
            ("tranh chấp", 84), ("hòa giải", 78), ("trọng tài", 76),
            ("nghĩa vụ", 88), ("quyền lợi", 92), ("quyền và nghĩa vụ", 80),
            ("pháp luật", 92), ("pháp lý", 84), ("pháp nhân", 76), ("thân nhân", 74),
            ("người thừa kế", 76), ("di chúc", 76), ("đăng ký kết hôn", 76),
        ]),
        ("đời sống - giao tiếp chung", [
            ("bữa sáng", 84), ("bữa trưa", 86), ("bữa tối", 84), ("bữa ăn", 84),
            ("cơm nhà", 80), ("cơm trưa", 84), ("cơm tối", 78), ("nấu ăn", 86),
            ("đi chợ", 76), ("đi siêu thị", 80), ("mua sắm", 88), ("mua nhà", 78),
            ("thuê nhà", 80), ("sửa nhà", 76), ("dọn nhà", 76), ("dọn dẹp", 82),
            ("giặt ủi", 76), ("gia đình", 96), ("ông bà", 84), ("cha mẹ", 84),
            ("bố mẹ", 88), ("anh chị em", 78), ("con cái", 82), ("cháu nội", 74),
            ("họ hàng", 80), ("bạn bè", 90), ("bạn thân", 84), ("người yêu", 84),
            ("vợ chồng", 84), ("kết hôn", 82), ("cưới hỏi", 76), ("đám cưới", 82),
            ("đám giỗ", 70), ("sinh nhật", 90), ("lễ tết", 86), ("Tết Nguyên Đán", 82),
            ("nghỉ lễ", 84), ("nghỉ hè", 76), ("kỳ nghỉ", 86), ("du lịch", 90),
            ("đi chơi", 80), ("cà phê", 88), ("cà phê sữa", 78), ("trà sữa", 84),
            ("bún phở", 74), ("gọi đồ", 76), ("ship đồ", 74), ("giao đồ ăn", 78),
            ("thời tiết", 88), ("nắng nóng", 80), ("mưa gió", 76), ("bão bùng", 70),
            ("giao thông", 88), ("kẹt xe", 84), ("đường trường", 74), ("xăng xe", 76),
            ("đi làm", 92), ("tan làm", 82), ("ca đêm", 78), ("ca sáng", 78),
            ("thức khuya", 82), ("dậy sớm", 80), ("ngủ trưa", 78), ("tập thể dục", 82),
            ("thể thao", 86), ("bóng đá", 84), ("cầu lông", 78), ("gym fitness", 74),
            ("sức khỏe", 94), ("tinh thần", 88), ("cảm cúm", 78), ("đau đầu", 80),
            ("mệt mỏi", 84), ("thư giãn", 86), ("nghỉ ngơi", 88), ("xả hơi", 74),
            ("điện ảnh", 78), ("xem phim", 84), ("nghe nhạc", 82), ("đọc sách", 84),
            ("chơi game", 82), ("mạng xã hội", 88), ("bài đăng", 78), ("bình luận", 86),
            ("chia sẻ bài", 74), ("tin nhắn", 90), ("cuộc gọi", 88), ("cuộc họp mặt", 76),
            ("nhắn tin", 84), ("gọi điện", 84), ("liên lạc", 88), ("kết nối", 90),
            ("thân thiết", 84), ("tạm biệt", 86), ("hẹn sớm", 74),
        ]),
        ("quốc gia - tổ chức - bổ sung vòng 52", [
            ("quốc gia", 96), ("quốc tế", 96), ("quốc phòng", 88), ("quốc khánh", 84),
            ("quốc hiệu", 80), ("quốc ngữ", 82), ("quốc hội", 84), ("quốc lộ", 78),
            ("quốc dân", 80), ("quốc ca", 80), ("quốc kỳ", 82), ("quốc tang", 74),
            ("hòa bình", 94), ("hòa đồng", 82), ("hòa hợp", 86), ("hòa nhã", 78),
            ("hòa giải", 84), ("hòa quyện", 74), ("hòa tan", 76), ("hòa nhập", 82),
            ("hội nghị", 92), ("hội thảo", 90), ("hội trường", 88), ("hội viên", 84),
            ("hội thẩm", 76), ("hội hè", 72), ("hội kiều", 70), ("hội luận", 72),
            ("hội nhập", 84), ("hội tụ", 74), ("tiến triển", 88), ("tiến hành", 90),
            ("tiến lên", 84), ("tiến tới", 82), ("tiến lùi", 74), ("tiến vượt", 70),
            ("tiến trình", 86), ("tiến nhanh", 76), ("tiến mạnh", 74),
            ("tổ chức", 100), ("tổ quốc", 90), ("tổ ong", 74), ("tổ tiên", 84),
            ("tổ hợp", 84), ("tổ đội", 74), ("tổ dân phố", 76), ("tổ y tế", 76),
            ("nhà nước", 94), ("nhà văn hóa", 84), ("nhà máy", 92), ("nhà ga", 84),
            ("nhà xe", 78), ("nhà trẻ", 74), ("nhà văn", 76), ("nhà báo", 78),
            ("nhà khoa học", 80), ("nhà kinh doanh", 80), ("nhà đầu tư", 84),
            ("chính phủ", 94), ("chính quyền", 90), ("chính sách", 100),
            ("chính thức", 96), ("chính xác", 96), ("chính đáng", 84), ("chính trực", 80),
            ("chính hiện", 70), ("chính chuyên", 70), ("chỉnh sửa", 90), ("chỉnh lý", 78),
            ("chỉnh đốn", 80), ("chỉnh trang", 76), ("cộng đồng", 94), ("cộng hòa", 96),
            ("cộng sự", 78), ("cộng hưởng", 76), ("xã hội", 96), ("xã giao", 84),
            ("xã đoàn", 74), ("xã luận", 78), ("đoàn kết", 92), ("đoàn thể", 80),
            ("đoàn viên", 76), ("đoàn tàu", 84), ("đoàn xe", 80), ("đoàn kết viên", 70),
        ]),
        ("thời gian - tần suất", [
            ("hôm nay", 100), ("hôm qua", 94), ("hôm kia", 80), ("ngày mai", 96),
            ("ngày mốt", 78), ("ngày kia", 76), ("tuần này", 90), ("tuần trước", 88),
            ("tuần sau", 88), ("tuần tới", 88), ("cuối tuần", 94), ("đầu tuần", 86),
            ("giữa tuần", 80), ("tháng này", 88), ("tháng trước", 86), ("tháng sau", 86),
            ("tháng tới", 86), ("đầu tháng", 84), ("giữa tháng", 80), ("cuối tháng", 90),
            ("đầu quý", 78), ("cuối quý", 80), ("quý này", 82), ("quý trước", 80),
            ("quý sau", 80), ("năm nay", 84), ("năm ngoái", 84), ("năm sau", 84),
            ("năm tới", 84), ("đầu năm", 84), ("cuối năm", 88), ("giữa năm", 76),
            ("sáng nay", 82), ("sáng mai", 82), ("chiều nay", 82), ("chiều mai", 82),
            ("tối nay", 82), ("tối qua", 78), ("buổi sáng", 88), ("buổi trưa", 82),
            ("buổi chiều", 88), ("buổi tối", 84), ("ban ngày", 80), ("ban đêm", 80),
            ("ban đầu", 92), ("sau này", 84), ("trước đây", 86), ("hiện nay", 88),
            ("hiện tại", 94), ("bây giờ", 96), ("ngay bây giờ", 84), ("lúc này", 88),
            ("lúc đó", 84), ("khi đó", 84), ("đến nay", 88), ("từ trước đến nay", 80),
            ("hàng ngày", 88), ("hàng tuần", 88), ("hàng tháng", 88), ("hàng quý", 80),
            ("hàng năm", 88), ("thường xuyên", 92), ("liên tục", 92), ("thỉnh thoảng", 82),
            ("đôi khi", 86), ("hầu như", 78), ("gần đây", 88), ("mới đây", 78),
            ("trong tương lai", 80), ("tương lai gần", 74), ("lâu dài", 84),
            ("ngắn hạn", 80), ("dài hạn", 84), ("trung hạn", 80), ("trong ngắn hạn", 74),
            ("tạm thời", 86), ("vĩnh viễn", 82), ("mãi mãi", 84), ("mãi mặn mà", 70),
            ("đúng giờ", 86), ("đúng hạn", 88), ("trễ hạn", 78), ("trễ deadline", 74),
            ("trước hạn", 82), ("quá muộn", 80), ("sớm nhất", 88), ("sớm mai", 74),
        ]),
    ]

    added = 0
    for _name, entries in groups:
        for w, weight in entries:
            if w in old:
                continue
            old[w] = weight
            added += 1

    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("# Từ ghép tiếng Việt thông dụng — văn phòng/hành chính/kế toán/nhân sự/\n")
        f.write("# IT/kinh doanh/giáo dục/y tế/pháp lý/đời sống/thời gian (vòng 52).\n")
        f.write("# Format: từ_ghép<TAB>weight (weight 1-100). Sinh bởi scripts/gen_vocab_v52.py\n")
        f.write("# — KHÔNG sửa tay (các mục cũ giữ weight tay nguyên vẹn).\n")
        for w, weight in old.items():
            f.write(f"{w}\t{weight}\n")
    print(f"vn_compounds.txt: {len(old)} mục (thêm {added})")
    return len(old)


if __name__ == "__main__":
    n1 = gen_vn_syllables()
    n2 = gen_en_words()
    n3 = gen_vn_compounds()
    if n1 < 6500 or n2 < 9000 or n3 < 1200:
        print("SỐ LƯỢNG KHÔNG ĐẠT", file=sys.stderr)
        sys.exit(1)

