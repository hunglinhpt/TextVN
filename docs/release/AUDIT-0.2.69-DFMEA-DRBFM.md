<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# Audit sau 0.2.68 — DFMEA + DRBFM (R2-140…R2-153)

> Phạm vi: mọi tính năng thêm ở 0.2.67/0.2.68 và phần đồng bộ 3 OS chưa phát hành
> (`66521cdf`). Mỗi lượt audit đọc toàn bộ diff từ đầu, tái hiện lỗi bằng probe chạy thật,
> rồi sửa, viết test, đẩy CI 3 OS. Gate hồi quy: `core/tests/audit_r2_140.rs`,
> `assist/tests/audit_r2_142.rs`, `config/tests/style_merge.rs`, test FFI `r2_152_*`,
> `test_field_detect.c`, `testEngineHint…`/`testCtrlGiuLau…` (Swift).

## 1. Bảng mã sự cố

| Mã | Mức | Lỗi (tái hiện được trên 0.2.68) | Nguyên nhân gốc | Cách sửa | Chặn tái diễn |
|---|---|---|---|---|---|
| R2-140 | nghiêm trọng | Telex mặc định phá chữ thường: `arr[0]` → `arr[0ư`, `- [ ] việc` → `- ơ ư việc`, `[tôi]` không gõ được | `[`/`]` luôn fold, không xét âm tiết, không có ngữ cảnh | Ngoặc là ranh giới trừ khi tạo được âm tiết Việt giữa từ; `]` đóng `[` đang mở trên dòng; từ có `[` gặp dấu hiệu chỉ mục thì trả nguyên văn | `r2_140_*` + `abnormal_ngoac_gach_ngang…` (4.000 chuỗi ngẫu nhiên) |
| R2-141 | nghiêm trọng | `--force` → `—force`, `i--;`, `<!-- -->`, `\|--\|`, `=A1--B1` bị đổi | Đổi ngay ở `-` thứ hai, không xét hai bên | Chỉ `chữ--chữ` (trả lại nếu phím kế không phải chữ) và `chữ-Việt -- ` | `r2_141_*` |
| R2-142 | cao | Trợ lý sửa oan câu đúng (32 mẫu), có cả sửa tự động; quét liên từ O(n³) | Luật thêm ở 0.2.68 thiếu dừng ở từ đi cặp, bỏ guard dấu phẩy, bảng động từ có `read`/`set` | Cửa sổ quét một lượt, `joins_clauses`, vị trí động từ, `at_clause_start` | `audit_r2_142.rs` (câu đúng + lỗi thật + thời gian) |
| R2-143 | cao | macOS: Mode EN không dự đoán, Ctrl tap không bao giờ chạy; Windows: trợ lý tự sửa ở Mode EN; Ctrl giữ lâu tính là tap | macOS gửi hint Passthrough khi tắt tiếng Việt; `otherKeyPressed` không xoá khi bắt đầu lần bấm mới | `engineHint`, xoá cờ khi modifier đi xuống, trần 400 ms cả 3 OS, trợ lý chỉ khi tiếng Việt bật | Test Swift, `test_modifier_ctrl_tap`, test compose |
| R2-144 | cao | Tab bị cướp khỏi shell (`git che` → `git chef`); Mode EN Space viết lại `func impl` → `function implied`; Ctrl+Space hết hoàn tất khi dự đoán tắt | Mode EN thiếu mọi gate ngữ cảnh | `prediction_blocked()` dùng cho mọi đường dự đoán; Space không viết lại ở Mode EN | `r2_144_*` |
| R2-145 | cao (G8) | Mật khẩu ở prompt `sudo` có thể vào hồ sơ học | Terminal không phải ô secure, không có gate | Không học khi `prediction_blocked()`; `peek` chạy `probe` không học | `r2_145_*` |
| R2-146 | trung bình | Hồ sơ đầy: 7 từ mới xoá sạch từ dùng nhiều nhất, từ mới loại lẫn nhau | Giảm nửa trọng số ở mỗi lần thêm | Thay từ trong phần cũ, miễn 32 từ mới, giảm mỗi 64 lần thay | `r2_146_*` |
| R2-147 | trung bình | Ứng viên bị Tab bỏ qua vẫn vào hồ sơ; Dream RSI đếm cả ứng viên bị xoá | Học lúc chèn | Một đường `settle_predict`: phím kế chốt, Backspace/Esc từ chối và gỡ khỏi Dream RSI; `reset` không học | `r2_147_*` |
| R2-148 | trung bình | Cụm rác `abc- dè`; drain cắt theo ABC, mất từ cuối vần | Không kiểm khoảng trắng; xoá cả pending dù chỉ xuất 200 | `style_note_bigram`; drain ưu tiên số lần gõ, giữ phần dư | `r2_148_*` |
| R2-149 | trung bình | `user`+Tab → `user interface`, `sort` sau khi học → `sorts` | Từ gốc nằm dưới từ ghép và từ cá nhân | Từ gốc đứng đầu khi fold ra chuỗi KHÔNG phải tiếng Việt; không còn gì để kéo dài thì Tab tới app | `r2_149_*` |
| R2-150 | hiệu năng | `peek_completion` 225 µs/phím với hồ sơ đầy | Chuẩn hoá lại 500 mục, quét tuyến tính từ điển, `to_lowercase` mỗi thuật ngữ | Loại nhanh theo từ cuối, tìm nhị phân, `Arc` cho options/pending, canonicalize sau khi cắt | `peek_completion_ho_so_day…` |
| R2-151 | cao | Bộ chặn terminal chỉ có tác dụng trên macOS | Windows gửi role 0, Linux gửi BODY | Windows `is_terminal_exe` (+ gate khớp appdb), IBus input purpose, Fcitx5 Capability/`program()`; Mode EN đưa phím thẳng tới app trong terminal | Test compose, `test_field_detect.c` |
| R2-152 | cao | macOS/Linux không lưu dữ liệu học, pending phình vô hạn, TextVN.app đè mất | Chỉ Windows có đường drain → tray | FFI `ime_style_profile_persist` (`atomic_write` 0600, xoá pending sau khi ghi); trần pending 2.000; `TextVNConfig.save` giữ dữ liệu trên đĩa | `style_merge.rs`, `r2_152_*` |
| R2-153 | trung bình | macOS: Ctrl tap reset buffer câu; không self-heal/fail-open; học cặp sửa chưa áp | Lệch với Windows/Linux | Chỉ chốt marked, self-heal, `engine.reset()` khi áp lỗi, học sau khi áp | Test Swift + đọc code |
| E36 | CI | `ci-linux/ci-deb/ci-aur` đỏ (`unused variable`) ở `ee321b34` | Sửa C không biên dịch được trên máy Windows dev | Sửa ngay; xem `project-common-errors.md` | Đẩy CI sớm mỗi đợt |

## 2. DFMEA (S/O/D 1–10, RPN sau khi sửa)

| Hạng mục | Failure mode | Hậu quả | S | O | D | RPN | Hành động / gate |
|---|---|---|---|---|---|---|---|
| Ngoặc Telex | Code/Markdown bị đổi chữ | Hỏng tài liệu người dùng | 9 | 2 | 2 | 36 | `r2_140_*`, fuzz 4.000 chuỗi |
| Ngoặc Telex | Chữ Việt gõ bằng ngoặc thành raw | Lỗi chính tả | 6 | 2 | 2 | 24 | `hơn-kem`, `"thư"` trong test |
| Ngoặc Telex | `[` thừa trên dòng làm `]` không ra `ư` | Phải gõ `uw` | 3 | 3 | 3 | 27 | Chấp nhận: xuống dòng là hết tác dụng |
| Gạch nối dài | Đổi nhầm code | Hỏng lệnh | 8 | 1 | 2 | 16 | `r2_141_*` |
| Gạch nối dài | `foo--bar` (CSS BEM) ở ô thường | Đổi thành `—`, Ctrl+Z hoàn tác | 5 | 3 | 3 | 45 | Chấp nhận: đúng tính năng `a--b` đã chốt |
| Mode EN Windows/Linux | Chữ giữ trong composition khi `suggest.enabled` | App nhận phím muộn ở ranh giới | 5 | 2 | 3 | 30 | Terminal bỏ qua hoàn toàn; opt-in |
| Học cá nhân | Lộ bí mật | Vi phạm G8 | 10 | 2 | 3 | 60 | Opt-in + chặn terminal; IBus với terminal Qt/X11 không gửi purpose: rủi ro còn lại, xem §4 |
| Lưu cấu hình | Mất/hỏng config.json | Mất cài đặt | 9 | 1 | 2 | 18 | `atomic_write` + fsync, config hỏng thì không ghi |
| Lưu cấu hình | Hai tiến trình cùng ghi | Mất một lượt học | 3 | 3 | 4 | 36 | Lần ghi sau đọc lại đĩa; pending chỉ xoá khi ghi xong |
| Dream RSI | Số đo phóng đại | Gate G19 sai | 5 | 2 | 2 | 20 | `r2_147_tu_choi…` |
| Hiệu năng | Gõ trễ khi ghost bật | Khó chịu | 6 | 1 | 2 | 12 | Cap trong `prediction_perf.rs` |

## 3. DRBFM — điểm thay đổi có chủ đích

| Thay đổi | Lý do | Tác động người dùng | Xác minh |
|---|---|---|---|
| `[`/`]` đầu từ luôn là ngoặc | `[` đầu từ trong văn bản hầu hết là ngoặc thật; đoán sai phá chữ, không đoán thì chỉ gõ `ow`/`uw` | `[n` không còn ra `ơn`; `ch]a`, `m[f`, `h[n`, `th][ng` giữ nguyên | Corpus `telex_horn_w_01` + `r2_140_*` |
| `[[` đầu từ giữ `[[` | `[[wiki]]`, `[[ -f x ]]` | Thay cho `[[` → `[` của 0.2.68 | Test lib + corpus |
| Tab đầu trả từ gốc khi fold hỏng | Không chèn chữ người dùng chưa thấy (không có ghost ở trạng thái này) | `customer`+Tab → `customer`, Tab nữa → `customer service` | `text_prediction.rs`; savings-rate 21% → 22% |
| Học khi chốt thay vì khi chèn | Chỉ học chữ người dùng giữ | Trễ một phím, drain sau phím kế | `r2_147_*` |
| Mode EN Space không hoàn tất | Người dùng tắt tiếng Việt để gõ đúng chữ mình gõ | `suggest.trigger=tab_and_space` chỉ có tác dụng ở Mode VI | `r2_144_*` |
| FFI thêm `ime_style_profile_persist` | Engine không I/O; đường ghi dùng chung của `config::doc` | ABI thêm 1 hàm, không đổi hàm cũ | `cli verify` 28 hàm, 2 header giống nhau |

## 4. Rủi ro còn lại (đã biết, chưa xử lý)

- **IBus với terminal Qt/X11** (Konsole, xterm, Alacritty): không gửi input purpose, IBus không
  cho biết tên app ⇒ vẫn coi là ô thường. Chỉ ảnh hưởng người bật `style_profile` hoặc
  `suggest.enabled`. Fcitx5 không bị (có `program()`).
- **Chỉ mục mảng có dấu phẩy hoặc biến không phải `i/j/k`**: `m[i, j]`, `m[n-1]` vẫn fold vì
  trùng hình với chữ Việt (`mơi,`, `hơn-kém`). Ctrl+Z hoàn tác; gõ code nên dùng Mode EN.
- **Terminal tích hợp của VS Code** trên Windows: cùng tiến trình `code.exe`, không phân biệt
  được với ô soạn thảo qua exe.
- **Ghost nhận bằng chuột trên macOS** vẫn tính 1 phím Tab vào `keys_typed` (giữ Tab để đúng
  đường macro của `peek_completion`).
