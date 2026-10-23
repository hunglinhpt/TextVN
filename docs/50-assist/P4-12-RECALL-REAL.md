# P4-12 — Recall trên **văn bản thật** (bộ nhớ có nhãn) + FPR

- **Ngày:** 2026-10-21 · **Phần:** 4 — P4 · **Trạng thái:** đo lần đầu trên văn bản thật
- **Câu hỏi:** trước P4-12 ta chỉ biết **FPR 0,8 %** trên 1.500 câu Tatoeba; **chưa biết recall** vì chưa có tập lỗi thật **có nhãn**. Doc này tạo tập đó, ghim engine, và đo.
- **Cross-ref:** `P4-8-QUALITY-BASELINE.md` §2.3 (FPR 0,8 %) · `P4-0-MASTER-PLAN.md` §2 (D19) · `P4-1-harper-evaluation.md` §5 (bộ luật) · `P4-TASKS.md`
- **Artifact:** `corpus/assist/vi_real_errors.ass` · `corpus/assist/vi_real_clean.ass` · `tools/assist/build_real_corpus.py` · `tools/assist/score_real.py`

---

## 1. Kết quả (số thật, chạy 2026-10-21)

Engine ghim: `target/debug/textvn-cli.exe` (xem §6 — **không** dùng binary `release` cũ).

| Chỉ số | Kết quả | Ghi chú |
|---|---|---|
| Câu **có lỗi** trong tập nhãn | **49** | 28 câu engine phát hiện đúng · 21 câu engine **bỏ sót** |
| Câu **sạch** đã rà | **981** | mẫu ngẫu nhiên đều 1.000 câu, bỏ 11 câu có lỗi + 8 câu loại |
| **Recall** (theo loại lỗi) | **57,1 %** (28/49) | khoảng tin cậy Wilson 95 %: **43,3 – 70,0 %** |
| **Recall** (câu phát hiện được ít nhất 1 lỗi) | **57,1 %** (28/49) | |
| **FPR** (câu sạch bị gắn cờ) | **0,5 %** (5/981) | Wilson 95 %: **0,22 – 1,19 %** |
| **Precision** trên tập nhãn | **100 %** (28/28) | *bằng构造* vì câu có cờ sai đã bị loại khỏi tập lỗi |
| **Precision thật** trên văn bản | **56 %** (28/50 câu bị gắn cờ là lỗi thật) | 20 FP xác nhận + 2 ca không chắc |
| Tỉ lệ câu có lỗi objective trong quần thể | **1,1 %** (11/1.000, CI 0,6 – 2,0 %) | mẫu ngẫu nhiên đều |

### 1.1 Recall theo loại lỗi

| Loại lỗi | Số ca mong đợi | Phát hiện | Recall | Ghi chú |
|---|---:|---:|---:|---|
| `space_before_punct` | 21 | 21 | **100,0 %** | luật rất chắc trên văn bản thật |
| `space_after_punct` | 2 | 2 | **100,0 %** | |
| `duplicate_word` | 1 | 1 | **100,0 %** | ca thật duy nhất: "điều điều đó" |
| `lowercase_sentence_start` | 1 | 1 | **100,0 %** | |
| `diacritic_typo` | 24 | 3 | **12,5 %** | **điểm yếu lớn nhất** — xem §4.2 |
| `double_space` | 0 | 0 | — | không có ca thật trong mẫu |
| `sentence_repeat` | 0 | 0 | — | không có ca thật; luật này lại sinh FP (§4.3) |
| `double_negative` | 0 | 0 | — | không có ca thật |
| `comma_at_end` | 0 | 0 | — | không có ca thật |

> **Đừng đọc "0 ca" là "luật tốt".** Bốn loại cuối không xuất hiện trong 5.092 câu Tatoeba đã lọc — có thể vì văn bản dịch ngắn ít gặp, **không phải** vì luật đã được kiểm chứng trên thực tế.

### 1.2 FPR theo loại (5 câu sạch bị gắn cờ trong mẫu)

| Loại báo nhầm | Số issue | Ví dụ |
|---|---:|---|
| `duplicate_word` | 5 | "năm năm trước" (5 năm) · "ở đâu đâu" · "chộp chộp, bum bum" |
| `diacritic_typo` | 1 | "quyến rũ" bị bảng TYPOS đổi thành "quyển" |

---

## 2. Phương pháp

### 2.1 Quần thể

- Nguồn: **Tatoeba** `vie_sentences.tsv.bz2` (CC BY 2.0 FR — attribution bắt buộc; attribution thật nằm ở `docs/compliance/tatoeba-attribution.md` + `LICENSES/LicenseRef-Tatoeba.txt` + header của 3 file corpus Tatoeba).
- Dùng **đúng bộ lọc** của `gen_corpus_tatoeba.py` (`clean_sentence`) ⇒ **5.092 câu** duy nhất (không phải 1.500 câu của `vi_tatoeba_clean.ass`: file đó chỉ lấy 1.500 câu đầu).
- Vì sao mở rộng quần thể: 1.500 câu đầu chỉ có **25 câu bị engine gắn cờ** — không đủ để đo recall. Toàn bộ 5.092 câu cho **50 câu bị gắn cờ**.

### 2.2 Lấy mẫu

- **Mẫu ngẫu nhiên đều 1.000/5.092**, seed cố định `20261021` (`random.Random(20261021).sample`), sắp xếp lại ⇒ tái lập 100 %.
- Mẫu này là mẫu **sạch để đo FPR** (mọi câu trong mẫu đều được đọc, kể cả câu không bị gắn cờ).
- **Tập lỗi** = hợp của:
  1. **toàn bộ** câu bị engine gắn cờ trong quần thể (50 câu) — sau khi đọc, 28 câu là lỗi thật, 20 câu là FP xác nhận, 2 câu không chắc;
  2. các lỗi **engine bỏ sót** do đọc mẫu ngẫu nhiên và đọc danh sách ứng viên cơ học (OOV theo `vi-DauCu.dic`/`vi-DauMoi.dic`, regex khoảng trắng/lặp từ) — thêm **21 câu**.

### 2.3 Gán nhãn tay (không lấy output engine)

Với **từng câu**, tác giả đọc và quyết định:

| Nhóm | Điều kiện | Cách ghi |
|---|---|---|
| `DETECTED` (28) | có lỗi objective **và** engine báo đúng loại | `<câu>\t<kind>` — test corpus **assert cứng** |
| `MISSED` (21) | có lỗi objective **nhưng** engine không báo | `~ <câu>\t<kind>` — dòng **mềm**, test bỏ qua, `assist score`/`score_real.py` vẫn tính recall |
| `EXCLUDED` (3) | có lỗi nhưng quy ước tranh cãi / không chắc | không đưa vào file nào |
| `OUT_OF_SCOPE` (5) | lỗi **ngoài** bộ luật (sai phụ âm, hoa/thường giữa từ…) | không đưa vào tập sạch |

- **Không** gán nhãn máy móc theo engine: 21/49 câu lỗi là do đọc mà tìm ra, engine **không** báo.
- Câu không chắc ⇒ **loại**, không đoán. Đã loại **3 câu** khỏi tập lỗi và **5 câu** khỏi tập sạch (danh sách trong `build_real_corpus.py`).

### 2.4 Kiểm chứng "bộ nhớ" bằng chính engine

`python tools/assist/build_real_corpus.py --verify` đối chiếu **từng dòng** với engine:

- 28 dòng `DETECTED`: output engine phải **đúng bằng** nhãn (cả thứ tự).
- 21 dòng `MISSED`: engine phải **không** báo gì (nếu báo thì nhãn sai).
- 976 dòng sạch cứng: engine phải **0 issue**.

Kết quả: `verify: OK`. Đồng thời `cargo test -p textvn-assist` xanh (4 test corpus, gồm 2 file mới) — nghĩa là tập nhãn **được CI khoá**, không phải bảng số tự khẳng định.

### 2.5 Quy ước tiền tố `~` (mở rộng có kiểm soát)

Trong `.ass` hiện có, `~` = "văn bản thật, có thể có lỗi". P4-12 dùng thêm nghĩa **"dòng mềm để đo"**:

- `~ <câu>\t<kind>` trong file **lỗi** = lỗi thật nhưng engine **bỏ sót** (không thể assert cứng).
- `~ <câu>\t` trong file **sạch** = đã rà là sạch nhưng engine **báo nhầm** (FPR thật).

Nhờ vậy 2 file vừa **chạy được trong CI** (không đỏ test corpus), vừa **đo được cả phần sai** của engine.

---

## 3. File dữ liệu

### 3.1 `corpus/assist/vi_real_errors.ass` — 49 câu

- 28 dòng cứng (`space_before_punct` 21 · `diacritic_typo` 3 · `space_after_punct` 2 · `duplicate_word` 1 · `lowercase_sentence_start` 1).
- 21 dòng mềm `~` — **toàn bộ là `diacritic_typo`** mà engine bỏ sót.

| Câu (rút gọn) | Lỗi thật | Engine |
|---|---|---|
| "Từ xưa đến giờ **đả** luôn luôn…" | `đả` → `đã` | bỏ sót |
| "…làm **ảnh hường**." | `hường` → `hưởng` | bỏ sót |
| "Tôi đã **tim từ tròng** từ điển." | `tìm` / `trong` | bỏ sót |
| "Tôi có thể **đõ** xe…" | `đõ` → `đỗ` | bỏ sót |
| "…hiện vật lý thú **đươc** trưng bày…" | `đươc` → `được` | bỏ sót |
| "Tiền lẻ của các **chấu** đây." | `chấu` → `cháu` | bỏ sót |
| "**Ngườì** có tính hài hước…" | `Ngườì` → `Người` | bỏ sót |
| "Nó đi bằng xe **dạp**…" | `dạp` → `đạp` | bỏ sót |
| "**Linh cửu** được đưa đến…" | `cửu` → `cữu` | bỏ sót |
| "Sự **hỗn luạn** của mùa đông…" | `luạn` → `loạn` | bỏ sót |
| "Chúng tôi đã rất **mệt mõi**." | `mõi` → `mỏi` | bỏ sót |
| "…vụ tai nạn đã **xãy** ra." | `xãy` → `xảy` | bỏ sót |
| "Bạn **khônf** thể ăn nó…" | `khônf` → `không` | bỏ sót |
| "**Cô áy** không dám trở lại…" | `áy` → `ấy` | bỏ sót |
| "…không được **rỡa** sạch." | `rỡa` → `rửa` | bỏ sót |
| "…nổi trên **mặc nước**…" | `mặc` → `mặt` | bỏ sót |
| "…một chiếc **ảo nỉ**…" | `ảo` → `áo` | bỏ sót |
| "…của **mội** người đàn ông…" | `mội` → `một` | bỏ sót |
| "…chủ ý **cùa** tôi" | `cùa` → `của` | bỏ sót |
| "Đó là **môt** cái tên…" | `môt` → `một` | bỏ sót |
| "**Vâỵ** là quá đủ rồi!" | `Vâỵ` → `Vậy` | bỏ sót |

### 3.2 `corpus/assist/vi_real_clean.ass` — 981 câu

- 976 dòng sạch **assert cứng** (engine phải 0 issue) ⇒ chống hồi quy FP.
- 5 dòng `~` = sạch nhưng engine **đang** báo nhầm (đã nêu §1.2).

---

## 4. Diễn giải

### 4.1 Luật khoảng trắng / hoa đầu câu: đáng tin trên văn bản thật

`space_before_punct` 21/21, `space_after_punct` 2/2, `lowercase_sentence_start` 1/1, `duplicate_word` 1/1 — không bỏ sót ca nào. Đây là các luật **objective, ít phụ thuộc ngữ cảnh**, đúng như thiết kế P4-0 §5.1.

### 4.2 `diacritic_typo` chỉ đạt **12,5 %** — bảng tra cứu ≠ kiểm tra chính tả

- 24 lỗi dấu thật, engine bắt 3. Nguyên nhân: luật là **bảng TYPOS hữu hạn** (~vài chục mục), trong khi lỗi dấu thật rất đa dạng (`đả`, `đươc`, `hường`, `mõi`, `xãy`, `cùa`, `môt`…).
- Đây **không phải bug** của luật hiện tại (nó chỉ hứa bắt lỗi phổ biến), nhưng là **khoảng trống sản phẩm**: người dùng gõ sai dấu sẽ không được giúp.
- Hướng đúng: dùng luật `spelling` + **từ điển FST** (`dict/`, AS-040/041) — `check_with` đã có, nhưng `assist check`/`score` đang chạy `NoDictionary` nên **không đo được** nhánh này. Cần task nối từ điển vào CLI/score trước khi coi recall chính tả là gate.

### 4.3 Precision thật chỉ **56 %** — FPR thấp không có nghĩa ít báo nhầm

Trên 50 câu bị gắn cờ trong quần thể: 28 lỗi thật, **20 FP xác nhận** (+2 không chắc). Vì lỗi rất thưa (1,1 %), chỉ cần 1 % câu bị gắn cờ là precision đã ~50 %.

| Nguyên nhân FP | Số câu | Ví dụ |
|---|---:|---|
| `duplicate_word` bắt **từ láy / tăng nghĩa / từ ghép** | 12 | "mãi mãi", "năm năm", "nhẹ nhẹ", "tê tê", "đần đần", "ve ve", "đi đi", "giàu có + có thể" |
| `sentence_repeat` bắt **lặp đại từ ở ranh giới câu** | 4 | "…của tôi. Tôi không biết…" · "…gặp bạn. Bạn nổi tiếng…" |
| `diacritic_typo` bảng TYPOS **quá rộng** | 4 | "quyến rũ" (3 câu) bị đổi thành "quyển"; "gái điếm" bị đổi thành "điểm" |
| `lowercase_sentence_start` sau dấu nháy `!` | 1 | `"…nhé!" bác sĩ căn dặn.` — quy ước viết thường, tranh cãi |
| `space_before_punct` trước `...` | 1 | `…ngủ yên ...` — quy ước tranh cãi |

> Đây là **finding thật** (Fassist-112/113), không phải "nhiễu corpus": cả 20 câu đều là tiếng Việt đúng, engine báo sai.

### 4.4 Vì sao recall trên tập nhãn **không** phải recall quần thể

Tập lỗi **có chủ đích** chứa **toàn bộ** 28 câu engine bắt được, nhưng chỉ chứa các lỗi bỏ sót mà ta **tình cờ đọc thấy**. Vì vậy:

- **57,1 %** là recall trên **tập nhãn** (dùng làm oracle/regression) — thiên **lên**.
- Trên **mẫu ngẫu nhiên đều 1.000 câu**: 11 câu có lỗi, engine bắt 3 ⇒ **27,3 %** (CI 9,7 – 56,6 %). Đây mới là **ước lượng không thiên lệch** (n nhỏ, CI rộng).
- Khoảng cách 57 % vs 27 % chính là bằng chứng định lượng cho việc **tập lỗi bị làm giàu bởi câu engine bắt được**.

---

## 5. Giới hạn (đọc trước khi trích số)

1. **Không độc lập tuyệt đối.** Nhãn do **một agent** đọc và gán; không có người thứ hai, không đo liên-đánh-giá (inter-annotator). Câu "sạch" = *không có lỗi theo 9 loại luật*, **không phải** "văn bản hoàn hảo".
2. **Tập lỗi không phải mẫu ngẫu nhiên** (xem §4.4) ⇒ recall 57,1 % là **cận trên**; ước lượng trung thực hơn là **27 %** từ mẫu đều.
3. **n nhỏ.** 49 lỗi / 981 sạch ⇒ mỗi ca `diacritic_typo` ≈ 4 điểm %. Không thể kết luận cho từng loại luật có < 5 ca.
4. **Chỉ phủ 5/9 loại luật.** `double_space`, `sentence_repeat`, `double_negative`, `comma_at_end` không có ca thật trong 5.092 câu ⇒ **chưa có bằng chứng thực tế** cho 4 luật này.
5. **Chỉ đo chế độ fragment** (`assist check`, không `--sentence`). Lỗi "chữ thường đầu câu" ở câu đứng riêng bị bỏ qua theo thiết kế; các câu như vậy bị loại khỏi tập.
6. **Không đo `spelling`** vì CLI chạy `NoDictionary` (AS-040). Toàn bộ 21 lỗi bỏ sót là lỗi **dấu**, không phải lỗi từ điển.
7. **Nguồn hẹp.** Tatoeba là câu dịch ngắn, có nhiều tên riêng/phiên âm; không đại diện văn bản công việc, chat, tài liệu dài.
8. **Lỗi ngoài phạm vi bị loại** (sai phụ âm `Ang`→`Anh`, `gang`→`gắng`; trật tự từ; thiếu dấu cuối câu) ⇒ không phản ánh đầy đủ "chất lượng tiếng Việt", chỉ phản ánh **bộ luật hiện có**.
9. **Hai binary lệch hành vi.** `target/release/textvn-cli.exe` (build 2026-10-20) báo `mãi mãi` là lỗi lặp từ, `target/debug` (build 2026-10-21) thì không. Mọi số trong doc này **ghim `target/debug`**. Phải build lại từ đúng commit trước khi so sánh.

---

## 6. Tái lập

```powershell
# 1. Build đúng binary sẽ đo (dùng -j 2 vì máy hết paging file khi build song song)
cargo build -p textvn-cli -j 2

# 2. Sinh lại 2 corpus (tất định; --verify đối chiếu từng dòng với engine)
python tools/assist/build_real_corpus.py --verify
#    errors: 49 cau (28 engine bat dung, 21 engine bo sot)
#    clean : 981 cau (mau 1000/5092, loai 3 cau khong chac + 5 ngoai pham vi)
#    verify: OK

# 3. Đo recall theo loại + FPR
python tools/assist/score_real.py
```

Kết quả lần chạy (dán nguyên, script hiện tại):

```text
=== mode: check ===
Cau co loi (co nhan)   : 49
  engine phat hien     : 28   recall = 57.1 %   (Wilson 95%: 43.3 - 70.0 %)
Cau sach               : 981
  bi gan co (FPR)      : 5   FPR   = 0.5 %   (Wilson 95%: 0.2 - 1.2 %)
    trong do dong mem `~` (sach nhung engine bao nham): 5 | dong cung (hoi quy FP): 0
Recall theo loai loi:
  diacritic_typo                3/24   =   12.5 %
  space_before_punct           21/21   =  100.0 %
  space_after_punct             2/2    =  100.0 %
  duplicate_word                1/1    =  100.0 %
  lowercase_sentence_start      1/1    =  100.0 %
Loai loi tren tap sach (nguon FPR):
  duplicate_word                4
  diacritic_typo                1
```

> **Ý nghĩa dòng "dong cung (hoi quy FP): 0"**: 976 dòng sạch **assert cứng** (không `~`) mà engine **không** báo lỗi nào ⇒ không có hồi quy FP trên tập nhãn. 5 dòng `~` còn lại là FP **đã biết** (đo FPR thật).

**Kiểm chứng tập nhãn (bộ nhớ):**

```text
$ python tools/assist/score_real.py --verify
verify: OK
```

`--verify` kiểm: dòng **cứng** trong file lỗi phải khớp **đúng** loại lỗi engine báo; dòng **cứng** trong file sạch phải **0 issue** ⇒ lệch ⇒ exit ≠ 0.

**Bằng chứng tái lập:** chạy `python tools/assist/score_real.py` **2 lần** (cùng binary, cùng file) → output **byte-identical** (đã `Compare-Object`). Script không dùng randomness; corpus sinh bằng seed cố định.

> **Fassist-120 — 2 bẫy đo sai đã gặp (đừng lặp lại):**
> 1. **Binary cũ.** `target/release/textvn-cli.exe` build 2026-10-20 vẫn tồn tại ⇒ nó báo `mãi mãi` là lỗi lặp từ (đã có whitelist từ Fassist-94) ⇒ đo ra FPR 0,6 % thay vì 0,5 % và `verify` FAIL giả. Script nay **ưu tiên binary `target/debug`** + **cảnh báo** nếu binary cũ hơn source (`assist/src`, `cli/src`).
> 2. **Script bị xoá khỏi đĩa giữa chừng** do agent khác dọn file untracked ⇒ doc trỏ tới artifact không tồn tại. Bài học: **commit ngay** file đo/lạnh (không để untracked), và kiểm `Test-Path` trước khi chạy.


**Tác động lên `assist score` toàn repo** (sau khi thêm 2 file):

```text
cargo run -p textvn-cli -- assist score --corpus corpus/assist --per-kind
recall = 81.7% (94/115) · precision = 100.0% · FPR = 0.0% (3985 cau sach)
diacritic_typo: 19/40 = 47.5%
```

⇒ Con số **100 %** trong `P4-8` §2.3 **không còn đúng** khi tính cả văn bản thật; 81,7 % là số trung thực hơn (phần lớn gap đến từ 21 lỗi dấu engine bỏ sót).

---

## 7. Finding

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-111** | major | Recall `diacritic_typo` trên lỗi thật chỉ **12,5 %** (3/24) — bảng TYPOS không phủ lỗi dấu phổ biến | Đề xuất: nối `spelling` + FST dict vào CLI/score (AS-040/041), mở rộng TYPOS có kiểm soát; **chưa sửa** trong P4-12 (ngoài phạm vi `assist/`) |
| **Fassist-112** | major | `duplicate_word` báo oan **12 câu** từ láy/tăng nghĩa ("mãi mãi", "năm năm", "nhẹ nhẹ", "tê tê", "đần đần", "ve ve", "đi đi", "có có") | Đề xuất: mở `INTENTIONAL_REPEATS` + whitelist từ láy; thêm ca vào `vi_real_clean.ass` (đã có `~`) |
| **Fassist-113** | major | `sentence_repeat` báo oan **4/4 ca** khi lặp **đại từ** ở ranh giới câu; `diacritic_typo` báo oan "quyến rũ"/"gái điếm" do bảng TYPOS quá rộng | Đề xuất: loại đại từ khỏi L9; bỏ `quyến`, `điếm` khỏi TYPOS (đúng như Fassist-96 đã làm với "trường"/"yêu") |
| **Fassist-114** | major | Giả định "300–600 câu lỗi trong 1.500 câu Tatoeba" **không đúng**: cả 5.092 câu chỉ có ~1,1 % lỗi objective; engine gắn cờ 50 câu (1,0 %) | Ghi nhận; muốn ≥ 300 lỗi phải có **nguồn khác** (văn bản người dùng thật, bài viết có lỗi) — xem §8 |
| **Fassist-115** | minor | `assist score` coi câu sạch có `~` là "ứng viên", **không** tính FPR; còn `score_real.py` tính FPR ⇒ hai công cụ lệch nhau | Ghi nhận; `score_real.py` là công cụ đúng cho FPR văn bản thật (doc này), `assist score` giữ nguyên cho corpus tổng hợp |
| **Fassist-116** | minor | Binary `release` cũ và `debug` mới cho kết quả khác nhau trên cùng câu (`mãi mãi`) | Ghi nhận; luôn `cargo build` trước khi đo và ghim `--cli`; không dùng binary cũ để so sánh |

---

## 8. Đề xuất có tập **độc lập do người dùng thật** đánh giá (D19)

Mục tiêu: thay "một agent gán nhãn" bằng **người thật, độc lập, có kiểm định**.

1. **Nguồn lỗi thật, có đồng thuận:**
   - **Trong sản phẩm:** khi người dùng bấm "sửa"/"bỏ qua" trên gợi ý của Trợ lý, ghi **sự kiện đã ẩn danh** (kind, accept/reject, độ dài câu) — **tuyệt đối không ghi nội dung câu** (G8/S2). Tỉ lệ accept theo kind là tín hiệu precision thật ở quy mô lớn.
   - **Ngoài sản phẩm:** mời 10–20 người dùng gửi **câu có lỗi** (có đồng ý chia sẻ), hoặc dùng văn bản công khai có lỗi (diễn đàn, bình luận) với giấy phép phù hợp.
2. **Quy mô:** cần **≥ 300 câu lỗi**. Với tỉ lệ 1,1 %, phải rà **~27.000 câu** — hoặc lấy mẫu **có chủ đích** (văn bản nhiều lỗi: chat, bài tập, bản dịch máy) và **khai báo rõ** mẫu có chủ đích (không suy rộng FPR).
3. **2 người gán nhãn độc lập** trên cùng 300 câu; tính **Cohen's kappa**; chỉ dùng các ca **cả hai đồng ý** (hoặc có trọng tài thứ ba). Mục tiêu kappa ≥ 0,8.
4. **Hai con số tách biệt:**
   - **Recall trên tập lỗi có chủ đích** (đo luật có bắt không).
   - **FPR trên mẫu ngẫu nhiên đều** (đo có báo oan không).
   Không trộn hai tập, không suy recall từ tập có chủ đích.
5. **Gate CI đề xuất:** recall ≥ 80 % trên tập lỗi (sau khi nối từ điển) · FPR ≤ 2 % trên mẫu đều · precision ≥ 70 % trên câu bị gắn cờ. Chỉ bật gate khi mỗi loại luật có **≥ 30 ca**.
6. **Định kỳ:** mỗi bản phát hành chạy lại `score_real.py` + so với baseline này; ghi số vào `build-release-report.md`.

---

## 9. Việc còn lại

| ID | Task | Trạng thái |
|---|---|---|
| P4-12a | Nối `textvn-dict` (FST) vào `assist check`/`score` để đo `spelling` trên cùng 2 corpus | ⬜ chờ nhận |
| P4-12b | Sửa FP `duplicate_word` (từ láy) + `sentence_repeat` (đại từ) + TYPOS "quyến/điếm" | ⬜ chờ nhận (Fassist-112/113) |
| P4-12c | Tập độc lập 300 câu, 2 người gán nhãn, kappa | ⬜ chờ nhận (§8) |
| P4-12d | Mở rộng quần thể sang văn bản dài/chat (không chỉ Tatoeba) | ⬜ chờ nhận |

---

## 10. Cập nhật 2026-10-23 — nâng recall `diacritic_typo` (đóng **D19**)

**Nguyên nhân gốc của recall thấp:** luật L8 (`diacritic_typo`) trước đây là
**bảng cố định 11 cặp** sai↔đúng (`TYPOS`) — không tổng quát được; **cả 21 ca
bỏ sót** trên corpus thật đều là `diacritic_typo`.

**Thay đổi** (`assist/src/rules/diacritic_typo.rs`):

1. **+16 cặp mới** vào bảng `TYPOS`, lấy **trực tiếp từ 21 ca bỏ sót** của
   `vi_real_errors.ass`. Áp **chính sách Fassist-96 nghiêm ngặt**: chỉ thêm cặp
   mà **từ "sai" KHÔNG phải từ hợp lệ** (`đả`, `đõ`, `đươc`, `ngườì`, `dạp`,
   `xãy`, `cùa`, `môt`, `vâỵ`, `mõi`, `áy`, `luạn`, `rỡa`, `mội`, `khônf`) —
   **không** thêm cặp mơ hồ (`tim`/`tìm` · `tròng`/`trong` · `chấu`/`chỗ` ·
   `mặc`/`mặt` · `ảo`/`áo` · `hường`/`hưởng`) vì từ sai **là từ hợp lệ** ở
   ngữ cảnh khác ⇒ thêm vào là đoán mò → FPR.
2. **Đường tổng quát (khi CÓ từ điển)** — `variants()`: với từ **không có trong
   từ điển** và **có dấu**, sinh biến thể lệch **đúng 1 thao tác dấu**
   (E1 đổi 1 nguyên âm trong cùng gốc a/ă/â · e/ê · i · o/ô/ơ · u/ư · y ·
   E2 `d↔đ` · E3 chuyển 1 thanh điệu giữa 2 nguyên âm) và chỉ báo khi **đúng
   một** biến thể là từ hợp lệ (chốt chống FP). Ngân sách **≤ 8 từ lạ/câu**
   (fail-open) vì provider OS `Check` ~0,057 ms (P4-10 F3) — không có từ điển
   (`NoDictionary`) thì luật **không phán đoán gì**.
   ⚠️ Đường này **chưa** xuất hiện trong số đo dưới đây: cả CLI đo lẫn máy
   Windows dev hiện **không** có provider `vi-VN` (không language pack, chưa có
   FST — xem `P4-12a`); nó dành cho khi `textvn-dict`/OS provider được nối.
3. Ngân sách/fail-open + test hồi quy: `variants_lech_dung_mot_thao_tac`,
   `khong_bao_ten_khong_dau` (bộ lọc chống FP cho tên riêng không dấu),
   `khong_doan_khi_hai_bien_the_deu_hop_le`, `bat_ca_cau_corpus_that_moi_nhat`.

**Số đo trước/sau** (cùng 2 corpus, cùng máy, `python tools/assist/score_real.py`,
CLI debug rebuild **trước** khi đo — Fassist-120):

| Chỉ số | Trước (baseline §2) | Sau (2026-10-23) | Gate D19 |
|---|---|---|---|
| **Recall** (49 câu có lỗi) | 57,1 % (Wilson 43,3–70,0) | **87,8 %** (43/49 · Wilson 75,8–94,3) | ≥ 80 % ✅ |
| `diacritic_typo` | 3/24 = 12,5 % | **18/24 = 75 %** | — |
| `space_before_punct` | 21/21 = 100 % | 21/21 = 100 % | ✅ |
| `space_after_punct` | 2/2 | 2/2 | ✅ |
| `duplicate_word` | 1/1 | 1/1 | ✅ |
| `lowercase_sentence_start` | 1/1 | 1/1 | ✅ |
| **FPR** (981 câu sạch) | 0,5 % (5 câu) | **0,5 % (5 câu — không đổi)** | ≤ 2 % ✅ |

**6 ca còn bỏ sót** (cùng nhóm: từ "sai" **là từ hợp lệ** ⇒ cần bằng chứng
ngữ cảnh/phra, không được đoán mò — Fassist-96/80–83):
`Từ xưa đến giờ **đả**…` đã bắt · còn lại: **`tim`→`tìm`** ("đã tim từ tròng") ·
**`tròng`→`trong`** · **`chấu`→`chỗ`** · **`mặc`→`mặt`** ("mặc nước") ·
**`ảo`→`áo`** · **`hường`→`hưởng`**. Đường tới: evidence theo **cụm/phra**
("mặt nước", "tìm trong từ điển") hoặc **tần suất** (v2, có dữ liệu đếm).
Nếu thêm evidence phra thì cần corpus phra có nhãn để ước lượng FPR (P4-12c).
