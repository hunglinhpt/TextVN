# Attribution dữ liệu Tatoeba (CC BY 2.0 FR)

> **Mã task:** Q-7 (audit compliance — corpus Tatoeba chưa có attribution thật)
> **Phạm vi:** attribution bắt buộc cho corpus đo FPR/recall lấy từ Tatoeba
> **Ngày:** 2026-10-22 · **Trạng thái:** ✅ ĐẠT — REUSE khai đúng license, có file license riêng, có attribution thật
> **Liên quan:** [`naming-check.md`](naming-check.md) · [`../../LICENSES/LicenseRef-Tatoeba.txt`](../../LICENSES/LicenseRef-Tatoeba.txt) · [`../../REUSE.toml`](../../REUSE.toml)

---

## 1. Bản attribution ngắn (để dán vào README / CHANGELOG / gói phát hành)

> Câu tiếng Việt trong `corpus/assist/vi_tatoeba_clean.ass`, `corpus/assist/vi_real_clean.ass`
> và `corpus/assist/vi_real_errors.ass` lấy từ **Tatoeba** (<https://tatoeba.org/>), theo export
> per-language `vie_sentences.tsv.bz2`, phát hành dưới giấy phép **Creative Commons
> Attribution 2.0 France (CC BY 2.0 FR)** — <https://creativecommons.org/licenses/by/2.0/fr/>.
> Đã **lọc, chọn mẫu, gán nhãn lỗi và tách thành nhiều file**; **không sửa nội dung câu**.

---

## 2. Nguồn dữ liệu

| Mục | Nội dung |
|---|---|
| **Tên tập dữ liệu** | **Tatoeba** — tập câu ví dụ + bản dịch do cộng đồng đóng góp, vận hành bởi **Tatoeba Association** (hiệp hội phi lợi nhuận đăng ký tại Pháp) |
| **Trang chủ** | <https://tatoeba.org/> |
| **Export đã dùng** | per-language, tiếng Việt — `vie_sentences.tsv.bz2` |
| **URL export** | <https://downloads.tatoeba.org/exports/per_language/vie/vie_sentences.tsv.bz2> |
| **Ngày truy cập / tải** | **2026-10-21** — export hằng tuần (thứ Bảy 06:30 UTC); 3 file corpus trong repo này được sinh từ **bản snapshot tải ngày 2026-10-21** |
| **Lịch cập nhật export** | hằng tuần — xem <https://tatoeba.org/en/downloads> |
| **Giấy phép** | **CC BY 2.0 FR** (Creative Commons Attribution 2.0 France) |
| **Văn bản giấy phép** | <https://creativecommons.org/licenses/by/2.0/fr/legalcode> |
| **Trích dẫn (citation)** | *Tatoeba contributors. Tatoeba — tập câu ví dụ tiếng Việt. <https://tatoeba.org/>. Xuất bản theo CC BY 2.0 FR.* |

> **Ghi chú về tính tái lập:** export Tatoeba được làm mới hằng tuần ⇒ chạy lại
> `tools/assist/gen_corpus_tatoeba.py` ở một ngày khác sẽ ra tập câu **không giống hệt**.
> Bản dùng cho các số đo đã ghi trong repo là snapshot **2026-10-21**.

---

## 3. Vì sao attribution là bắt buộc

Trích nguyên văn từ <https://tatoeba.org/en/downloads> (mục *"General information about the files"*):

> These files are released under CC BY 2.0 FR.

và từ *Terms of Use* của Tatoeba, §6.2 *"Creative Commons licenses applicable to sentences"*
(<https://tatoeba.org/en/terms_of_use>):

> Tatoeba's technical infrastructure uses the default Creative Commons Attribution 2.0 France
> license (CC-BY 2.0 FR) for the use of textual sentences. The BY mention implies a single
> restriction on the use, reuse, modification and distribution of the sentence: a condition of
> attribution. That is, using, reusing, modifying and distributing the sentence is only allowed
> if the name of the author is cited.

**Nghĩa vụ của CC BY 2.0 FR khi tái sử dụng / phát hành lại** (rút gọn từ §3 của giấy phép):

1. ghi công **tên tác giả** của tác phẩm gốc;
2. kèm **link tới giấy phép** (<https://creativecommons.org/licenses/by/2.0/fr/>);
3. kèm **link tới nguồn** gốc (<https://tatoeba.org/>);
4. **nêu rõ** rằng tác phẩm **đã bị sửa**;
5. **không** đặt thêm điều kiện hạn chế mới lên người nhận.

TextVN thực hiện đủ 5 điều trên ở **mức tập dữ liệu**: ghi nguồn + link nguồn + link giấy phép +
nêu rõ các thay đổi đã làm (§5) + không đặt thêm ràng buộc nào.

> **Hạn chế đã biết (trung thực, không che):** export per-language **không** chứa tên tác giả của
> từng câu (chỉ có `id câu` + `mã ngôn ngữ` + `nội dung câu`). Vì vậy attribution ở đây là ở
> **mức tập dữ liệu** (nguồn + giấy phép + link + thay đổi), và **người phát hành lại từng câu
> riêng lẻ** thì **phải** tra tên tác giả theo `id câu` tại <https://tatoeba.org/> rồi ghi kèm.

---

## 4. File nào trong repo chứa dữ liệu Tatoeba

### 4.1. File **CÓ** dữ liệu Tatoeba ⇒ license `LicenseRef-Tatoeba` (CC BY 2.0 FR)

| File | Nội dung | Số dòng dữ liệu | License |
|---|---|---:|---|
| `corpus/assist/vi_tatoeba_clean.ass` | 1.500 câu sạch (mọi dòng có tiền tố `~ `) | 1.500 | `LicenseRef-Tatoeba` |
| `corpus/assist/vi_real_clean.ass` | 981 câu đã rà soát (976 dòng assert cứng + 5 dòng `~`) | 981 | `LicenseRef-Tatoeba` |
| `corpus/assist/vi_real_errors.ass` | 49 câu có lỗi thật (28 dòng assert cứng + 21 dòng `~`) | 49 | `LicenseRef-Tatoeba` |

⇒ **2.530 dòng dữ liệu** (có overlap giữa các file: cả 3 đều lấy từ cùng quần thể đã lọc).

### 4.2. File corpus **KHÔNG** lấy từ Tatoeba ⇒ giữ `GPL-3.0-or-later`

| File | Nguồn | Số dòng dữ liệu | License |
|---|---|---:|---|
| `corpus/assist/vi_basic.ass` | tự viết (câu mẫu + loại lỗi mong đợi) | 67 | `GPL-3.0-or-later` |
| `corpus/assist/vi_clean_synthetic.ass` | sinh tự động theo template (không nguồn ngoài) | 1.408 | `GPL-3.0-or-later` |
| `corpus/assist/vi_docs_clean.ass` | trích từ `docs/` **của chính repo này** | 55 | `GPL-3.0-or-later` |
| `corpus/assist/vi_grammar.ass` | tự viết (luật ngữ pháp L9/L10/L12) | 13 | `GPL-3.0-or-later` |
| `corpus/assist/vi_regressions.ass` | tự viết (hồi quy lỗi) | 20 | `GPL-3.0-or-later` |

Các file `corpus/{shared,win,mac}/*.keys` là **keystroke corpus tự sinh của TextVN** ⇒ `GPL-3.0-or-later`.

Cách khai trong [`../../REUSE.toml`](../../REUSE.toml) (annotation `precedence = "override"`, đặt **sau**
blanket `path = ["**"]` ⇒ reuse chỉ dùng annotation khớp **cuối cùng**):

```toml
[[annotations]]
path = [
    "corpus/assist/vi_tatoeba_clean.ass",
    "corpus/assist/vi_real_clean.ass",
    "corpus/assist/vi_real_errors.ass",
]
precedence = "override"
SPDX-FileCopyrightText = [
    "Tatoeba contributors (tatoeba.org) — câu tiếng Việt trong export vie_sentences.tsv.bz2",
    "2026 LinhBH.CoM — tập lọc/gán nhãn (đạo hình thức)",
]
SPDX-License-Identifier = "LicenseRef-Tatoeba"
```

---

## 5. Thay đổi đã làm với dữ liệu (CC BY yêu cầu phải nêu "đã sửa")

1. **Lọc (filter)** — hàm `clean_sentence()` trong [`../../tools/assist/gen_corpus_tatoeba.py`](../../tools/assist/gen_corpus_tatoeba.py):
   - bỏ khoảng trắng đầu/cuối câu;
   - giữ câu dài **25–200 ký tự**;
   - bỏ câu chứa `http`, `<`, `>`, `|`;
   - bỏ câu có **số dấu nháy kép lẻ** (dấu nháy kép trong tiếng Việt là dấu câu, không phải trích dẫn);
   - giữ câu có **≥ 35 % chữ cái có dấu** tiếng Việt (loại câu không phải tiếng Việt);
   - bỏ **trùng lặp** (so sánh nguyên văn), giữ nguyên thứ tự gốc.
   ⇒ Quần thể sau lọc: **5.092 câu**; file `vi_tatoeba_clean.ass` lấy **1.500 câu đầu**.
2. **Chọn mẫu** — [`../../tools/assist/build_real_corpus.py`](../../tools/assist/build_real_corpus.py):
   mẫu ngẫu nhiên **đều 1.000/5.092 câu**, seed cố định `20261021`, sắp xếp lại ⇒ tái lập 100 %.
   Từ mẫu này tách ra **tập lỗi** (49 câu) và **tập sạch** (981 câu).
3. **Gán nhãn (tay, KHÔNG theo output engine)**:
   - cột `<kind>` = loại lỗi objective (`space_before_punct`, `space_after_punct`, `duplicate_word`,
     `lowercase_sentence_start`, `diacritic_typo`, `double_space`, `sentence_repeat`,
     `double_negative`, `comma_at_end`);
   - tiền tố `~ ` = **dòng mềm**: lỗi thật mà engine **bỏ sót** (file lỗi) hoặc câu sạch mà engine
     **báo nhầm** (file sạch) ⇒ dùng để **đo**, không assert cứng.
   - Ba câu không chắc chắn + 5 câu lỗi ngoài phạm vi luật bị **loại** khỏi cả 2 file.
4. **Tách file**:
   - `vi_tatoeba_clean.ass` — 1.500 câu sạch (tiền tố `~ ` = "sạch **có thể** chứa lỗi thật");
   - `vi_real_clean.ass` — 981 câu sạch đã rà soát;
   - `vi_real_errors.ass` — 49 câu có lỗi thật.
5. **KHÔNG sửa nội dung câu**: không sửa chính tả, không thêm/bớt dấu câu, không dịch.
   Chỉ thêm tiền tố `~ ` và cột nhãn ⇒ **văn bản câu giữ nguyên** so với Tatoeba.

---

## 6. Phạm vi sử dụng

Dữ liệu Tatoeba trong repo **chỉ dùng để đo (measurement)**, không dùng làm dữ liệu huấn luyện
và không nhúng vào sản phẩm:

| Mục đích | Công cụ | Tài liệu kết quả |
|---|---|---|
| Đo **FPR** (false-positive rate) của bộ luật trên **văn bản thật** | `tools/assist/score_real.py` · `cargo run -p textvn-cli -- assist score --corpus corpus/assist` | [`../50-assist/P4-8-QUALITY-BASELINE.md`](../50-assist/P4-8-QUALITY-BASELINE.md) §2.3 |
| Đo **recall** trên tập lỗi **có nhãn tay** + kiểm chứng tập nhãn | `tools/assist/build_real_corpus.py --verify` · `tools/assist/score_real.py --verify` | [`../50-assist/P4-12-RECALL-REAL.md`](../50-assist/P4-12-RECALL-REAL.md) |

> Corpus này **không** dùng để huấn luyện bất kỳ mô hình nào (TextVN v1 không có mô hình học
> từ corpus) và **không** được nhúng vào binary phát hành.

---

## 7. Không phát hành dữ liệu gốc trong gói phát hành

Đã kiểm tra từng đường đóng gói:

| Nền tảng | Script đóng gói | Có `corpus/` trong gói? |
|---|---|---|
| Windows | [`../../installer/windows/TextVN-setup.iss`](../../installer/windows/TextVN-setup.iss) — chỉ `Source: "..\..\data\*"` (loại `data/assist/*.dic`) | **KHÔNG** |
| Linux (.deb / PPA) | [`../../debian/rules`](../../debian/rules) — `override_dh_auto_install` chỉ cài binary, icon, desktop file, appstream metainfo, `LICENSE` | **KHÔNG** |
| Linux (AUR) | [`../../packaging/arch/PKGBUILD`](../../packaging/arch/PKGBUILD) — `package()` chỉ cài binary, icon, XML, README/CHANGELOG/user-guide, `LICENSE` | **KHÔNG** |
| macOS | `packaging/macos/*` + `adapters/macos-imk/Resources` — chỉ binary + tài nguyên | **KHÔNG** |
| Homebrew cask | [`../../packaging/homebrew/textvn.rb`](../../packaging/homebrew/textvn.rb) — tải artifact đã ký | **KHÔNG** |

`grep -r corpus packaging/ installer/ debian/ tools/release/` ⇒ **không** có script đóng gói nào
tham chiếu `corpus/`. ⇒ **Corpus chỉ nằm trong kho mã nguồn (git), không đi kèm gói phát hành.**

**Nếu sau này** quyết định phát hành kèm corpus (ví dụ bộ corpus đo kèm tài liệu) thì **bắt buộc**:

1. đính kèm `docs/compliance/tatoeba-attribution.md` (bản attribution ở §1) và
   `LICENSES/LicenseRef-Tatoeba.txt` vào gói;
2. ghi rõ trong CHANGELOG / `build-release-report.md` rằng gói chứa dữ liệu CC BY 2.0 FR;
3. giữ nguyên bản câu (không sửa) và kèm link nguồn + link giấy phép.

---

## 8. Việc còn lại (không chặn repo, cần làm khi phát hành)

| # | Việc | Vì sao | Mốc |
|---|---|---|---|
| 1 | Thêm mục "Dữ liệu bên thứ ba" (bản attribution §1) vào **README** + **CHANGELOG** + `build-release-report.md` | CC BY yêu cầu attribution ở **nơi phát hành**; hiện repo mới chỉ có attribution ở tầng kỹ thuật | Checklist B4 trong [`../release/release-process.md`](../release/release-process.md) |
| 2 | Nếu bao gồm corpus trong gói ⇒ làm mục 3 ở §7 | Bắt buộc theo CC BY | Khi có quyết định đóng gói corpus |
| 3 | Tra tên tác giả theo `id câu` nếu phát hành lại **từng câu riêng lẻ** | Export không chứa tên tác giả | Khi có quyết định phát hành lại từng câu |
| 4 | Ghi nhận "sửa" cụ thể trong từng lần tái sinh corpus (ngày + seed) | Export đổi hằng tuần | Mỗi lần chạy `gen_corpus_tatoeba.py` |

---

## 9. Liên kết nội bộ

| File | Vai trò |
|---|---|
| [`../../LICENSES/LicenseRef-Tatoeba.txt`](../../LICENSES/LicenseRef-Tatoeba.txt) | Text giấy phép tùy biến `LicenseRef-Tatoeba` (bắt buộc có cho `reuse lint`) |
| [`../../REUSE.toml`](../../REUSE.toml) | Annotation `precedence = "override"` khai CC BY 2.0 FR cho đúng 3 file Tatoeba |
| [`naming-check.md`](naming-check.md) | Báo cáo compliance khác (tên/namespace) |
| [`../50-assist/P4-12-RECALL-REAL.md`](../50-assist/P4-12-RECALL-REAL.md) §2.1 | Nguồn + cách lấy mẫu + quy ước `~` |
| [`../50-assist/P4-8-QUALITY-BASELINE.md`](../50-assist/P4-8-QUALITY-BASELINE.md) §2.3 | Kết quả đo FPR trên 1.500 câu Tatoeba |
| [`../../tools/assist/gen_corpus_tatoeba.py`](../../tools/assist/gen_corpus_tatoeba.py) | Script sinh corpus (URL export + bộ lọc) |
| [`../../tools/assist/build_real_corpus.py`](../../tools/assist/build_real_corpus.py) | Script sinh tập có nhãn (seed `20261021`) |
