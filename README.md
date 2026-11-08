# TextVN — Bộ gõ tiếng Việt thế hệ mới cho Windows, macOS và Linux

Tiếng Việt · English (mục [English](#english) cuối trang)

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![CI Windows](https://img.shields.io/github/actions/workflow/status/hunglinhpt/TextVN/ci-windows.yml?branch=main&label=CI%20Windows)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-windows.yml?query=branch%3Amain)
[![CI Linux](https://img.shields.io/github/actions/workflow/status/hunglinhpt/TextVN/ci-linux.yml?branch=main&label=CI%20Linux)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-linux.yml?query=branch%3Amain)
[![CI macOS](https://img.shields.io/github/actions/workflow/status/hunglinhpt/TextVN/ci-macos.yml?branch=main&label=CI%20macOS)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-macos.yml?query=branch%3Amain)
[![CI engine (3 OS)](https://img.shields.io/github/actions/workflow/status/hunglinhpt/TextVN/ci-shared.yml?branch=main&label=CI%20engine%20%283%20OS%29)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-shared.yml?query=branch%3Amain)

[![Tải TextVN trên Microsoft Store](https://get.microsoft.com/images/en-us%20dark.svg)](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)

**TextVN** là bộ gõ tiếng Việt mã nguồn mở thế hệ mới, được xây dựng trên một lõi engine **Rust** duy nhất (an toàn bộ nhớ tuyệt đối, 0 I/O trong engine, không panic qua FFI với `catch_unwind`) dùng chung cho cả ba hệ điều hành: **Windows** (Text Services Framework thuần), **macOS** (Input Method Kit native), và **Linux** (IBus & Fcitx5). TextVN mang lại tốc độ phản hồi micro-giây, độ chính xác ngữ âm tuyệt đối, triệt tiêu hoàn toàn các lỗi kinh điển của bộ gõ hook truyền thống (mất chữ, lặp chữ, gạch chân preedit), đồng thời tích hợp hệ thống dự đoán ngữ cảnh song ngữ và trợ lý ngữ pháp thông minh chạy hoàn toàn offline ngay trên máy người dùng.

---

## 4 Trụ Cột Vượt Trội Cốt Lõi

### 1. Kiến trúc In-Process thuần khiết & An toàn bộ nhớ tuyệt đối
- **Windows TSF thuần (`ITfTextInputProcessorEx`)**: Bản cài mặc định không cài global keyboard hook ăn phím, không tiêm mã DLL vào process khác, không giả lập phím bằng `SendInput` để gửi chuỗi Backspace xoá ngược ([nguyên nhân kiến trúc](docs/specs/antivirus-false-positive.md)).
- **Không mất chữ, không lặp chữ, không gạch chân preedit**: Cả từ nằm trọn vẹn trong vùng soạn thảo nội tại không đường kẻ viền (`TF_LS_NONE` trên Windows TSF; chế độ non-preedit trên macOS; không kềm gạch trên Linux). Triệt tiêu vĩnh viễn lỗi nuốt chữ, lặp chữ khi gõ nhanh trên Google Chrome, Microsoft Edge, ứng dụng Electron (VS Code, Slack, Discord), Microsoft Excel, thanh địa chỉ hay game toàn màn hình.
- **Tránh nhận diện nhầm mã độc (Anti-False-Positive)**: Hoạt động đúng chuẩn giao diện IME của hệ điều hành, tuân thủ nghiêm ngặt chính sách diệt virus của Microsoft Store và các giải pháp bảo mật doanh nghiệp.

### 2. Hệ thống Dự đoán Ngữ cảnh Song ngữ & Công thái học Dream RSI (P10)
- **Tập tri thức chuẩn mực 41.313 mục**: Bao gồm 6.672 âm tiết Việt chuẩn mực, 1.941 từ ghép Việt (`vn_compounds.txt`, 13 chuyên ngành), 3.346 từ Anh tần suất cao (`en_freq3000.txt`), 9.868 từ đơn Anh (`en_words10k.txt`), 1.957 từ ghép & cụm danh từ Anh (`en_compounds.txt`, 16 domain hiện đại: AI, Cloud, Dev, Fin, HR, Legal…), cùng 17.529 cặp câu/cụm/thành ngữ song ngữ (8.782 mẫu Việt `vn_phrases.tsv` và 8.747 mẫu Anh `en_phrases.tsv`).
- **Đồng bộ song hành 2 chế độ (Mode VI & Mode EN)**:
  - **Mode EN (`E`)**: Hoàn tất từ đơn, từ ghép, gợi ý cụm từ tiếp theo, duyệt vòng 5 ứng viên bằng Tab, chữ mờ xem trước (ghost preview) siêu tốc 75 µs.
  - **Mode VI (`V`)**: Nhận diện thông minh từ ngữ tiếng Anh xen kẽ. Tự động phục hồi từ/cụm tiếng Anh bị gõ theo Telex mà không phải âm tiết tiếng Việt hợp lệ (ví dụ: `customer` thành `cútomer` + Tab → `customer service`; `looking forward` thành `looking fỏwad` + Tab → `looking forward to hearing from you soon`).
- **Phím tiện ích Standalone `Ctrl` Tap**: Bấm thả phím `Ctrl` đơn lẻ để chốt từ hợp lệ (`tunx` + Ctrl → chốt âm tiết `tũn`), hoặc lập tức khôi phục từ sai chính tả tiếng Việt về nguyên bản tiếng Anh (`sort` → `sỏt` + Ctrl → `sort`).
- **Công nghệ đo lường công thái học Dream RSI (P10 / G19)**: Đo lường khách quan số phím bấm thực tế tiết kiệm được. Đạt tỷ lệ tiết kiệm 22% số phím gõ trong tác vụ văn phòng thực tế; kiểm soát chống hồi quy hiệu năng nghiêm ngặt qua CI (`perf/baseline-win.json`).
- **Hồ sơ phong cách người dùng (User Style Profile P7)**: Tự động ghi nhận từ vựng riêng và cặp sửa lỗi người dùng chấp nhận, học 100% offline, chỉ kích hoạt khi người dùng cho phép (opt-in G8), bảo vệ nghiêm ngặt quyền riêng tư.

### 3. Trợ lý Ngữ pháp & Ngữ cảnh On-Device (Grammarly-Grade Offline)
- **Kiểm soát 15 nhóm lỗi toàn diện**: Bắt lặp từ, lặp cụm 2–4 từ, khoảng trắng quanh dấu câu, viết hoa đầu câu, sai vị trí dấu thanh ("nghiã" → "nghĩa"), **sai dấu thanh theo từ điển âm tiết dựng sẵn ("chềt" → "chết", bắt 99% lỗi dấu thanh tổng hợp, 0% báo nhầm trên corpus sạch — không cần từ điển hệ điều hành)**, phủ định kép, phẩy sau liên từ, mạo từ tiếng Anh a/an, cặp từ dễ nhầm lẫn EN, đại từ "i", chính tả EN/VI, cảnh báo ngày tháng phi lý (31/02, 32/13 — calendar-aware), số tiền phân cách hỗn loạn (1.250.00đ).
- **Cơ chế Idle 1 giây trên cả 3 hệ điều hành**: Dừng gõ 1 giây → panel trợ lý tự động xuất hiện ngay tại con trỏ (caret), gạch chân đỏ chính xác từ sai kèm tổng số lỗi; gõ tiếp → panel lập tức ẩn đi để không làm gián đoạn dòng văn bản.
- **Hai chế độ linh hoạt**: Chế độ *Chỉ gợi ý* (Mode 1) hoặc *Tự sửa lỗi an toàn* (Mode 2) tự động sửa lỗi khách quan và cho phép hoàn tác 1 chạm bằng Ctrl+Z trong vòng 5 giây.
- **Tiêu chuẩn vàng không báo nhầm (0% FPR)**: Đạt tỷ lệ 0 % câu đúng bị báo nhầm trên tập kiểm định 981 câu tiếng Việt và 295 câu tiếng Anh ([P4-12 §11](docs/50-assist/P4-12-RECALL-REAL.md)).

### 4. Di sản Chuẩn UniKey 20 Năm & Tiện ích Văn phòng Hiện đại
- **Chuẩn mực gõ dấu UniKey**: Đầy đủ các kiểu gõ Telex, VNI, VIQR, Simple Telex; hỗ trợ lựa chọn dấu cũ (`hòa`, `thủy`) hoặc dấu mới (`hoà`, `thuỷ`); chuẩn vần `uo` (`luong`/`uong` giữ nguyên, gõ `w` hoặc dấu mới thành `ươ`).
- **Tiện ích Telex thông minh**: Gõ nhanh `[` / `]` thành `ơ` / `ư` nhưng bảo vệ tuyệt đối dấu ngoặc vuông trong lập trình (`arr[0]`, Markdown `- [ ] task`); gõ `a--b` thành gạch nối dài `a—b` nhưng giữ nguyên cờ lệnh code `--force`; gõ hoa thông minh `DD` → `Đ`; dead-key passthrough (`Ctrl+'`+`e` → `é`).
- **Phím tắt chuyển mã thông minh Ctrl+Shift+F9**: Tự động chuyển đổi qua lại giữa Unicode, TCVN3, VNI, Unicode tổ hợp; tự động nhận diện bảng mã nguồn bị lỗi hiển thị (`Tiáº¿ng Viá»‡t` → `Tiếng Việt`); tự động sửa chuỗi gõ nhầm khi quên bật tiếng Việt (`tooi ddi hocj` → `tôi đi học`).
- **Bảng điều khiển đồng bộ 3 OS (0.2.72)**: 100% nút bấm có biểu tượng và tooltip gợi ý; nút "Ẩn vào khay" và "Thoát" phân định rõ ràng; tray tự động đăng ký TSF cho tài khoản mà không cần quyền quản trị (Admin/UAC).

---

## Điểm Vượt Trội So Với Các Bộ Gõ Khác

Bảng so sánh đối đầu trực tiếp giữa TextVN và các bộ gõ truyền thống (UniKey, EVKey, OpenKey):

| Tiêu chí so sánh | TextVN (v0.2.72) | Bộ gõ truyền thống (UniKey / EVKey / OpenKey) |
|---|---|---|
| **Cơ chế xử lý phím** | In-process TSF (Windows), native IMK (macOS), IBus/Fcitx5 (Linux) | Global keyboard hook ăn phím + `SendInput` xoá ngược |
| **Lỗi mất chữ, lặp chữ (Chrome/Excel/Electron)** | **Triệt tiêu hoàn toàn** (composition nội tại không gạch chân) | Thường xuyên gặp khi gõ nhanh hoặc chuyển cửa sổ |
| **Tự nhận diện EN/VI khi gõ xen kẽ** | Luật ngữ âm Việt + 103 từ dễ nhầm + 116 thuật ngữ + từ điển EN cá nhân + lưới bảo vệ 94 âm tiết: trả lại từ gốc khi gõ dấu cách | Telex áp đặt toàn bộ — `text` thành `tẽt`, `is` thành `í`; buộc phải chuyển chế độ thủ công |
| **Dự đoán từ & cụm song ngữ (Tab/Space)** | Đầy đủ từ đơn, từ ghép, cụm từ ngữ cảnh ở cả Mode VI & Mode EN; 5 ứng viên Tab; ghost preview 75 µs | Không hỗ trợ hoặc chỉ gõ tắt tĩnh |
| **Công thái học Dream RSI (P10)** | Đo lường khách quan số phím tiết kiệm (tỷ lệ tiết kiệm 22% trên văn phòng); chặn hồi quy qua CI | Không có |
| **Trợ lý ngữ pháp & chính tả on-device** | 15 loại lỗi (lặp từ, khoảng trắng, viết hoa, sai dấu, a/an, ngày tháng, tiền tệ); panel dừng gõ 1 giây trên 3 OS; 0% báo nhầm; Ctrl+Z 5s | Không có |
| **Khôi phục phím Standalone Ctrl Tap** | `tunx` + Ctrl → `tũn` (chốt âm tiết); `sort` → `sỏt` + Ctrl → `sort` (khôi phục từ sai) | Không có |
| **Escape khôi phục nguyên phím đã gõ** | Nhấn Escape trả lại đúng chuỗi ký tự gốc tức thì | Không có hoặc chỉ hỗ trợ xoá cả câu |
| **Chuyển đổi V/E toàn cục (Ctrl+Shift)** | Bộ dò tap chỉ quan sát (không ăn phím) + debounce chống lật đôi giữa tray và engine | Dễ xung đột lật đôi trạng thái hoặc bị Antivirus chặn |
| **Tiện ích lập trình & soạn thảo** | Phím `[` `]` thành `ơ`/`ư` thông minh (bảo vệ `arr[0]`); `a--b` thành `a—b` (bảo vệ `--flag`); `DD` → `Đ` | Biến đổi mù quáng làm hỏng mã nguồn và cú pháp Markdown |
| **Chuyển mã thông minh (Ctrl+Shift+F9)** | Tự động đoán mã nguồn lỗi (`Tiáº¿ng Viá»‡t` → `Tiếng Việt`), sửa lỗi gõ sai khi tắt IME (`tooi ddi hocj` → `tôi đi học`) | Chỉ chuyển mã clipboard thủ công cơ bản |
| **Quyền cài đặt & UAC** | Per-user HKCU, portable giải nén chạy ngay, không cần Admin/UAC | Đa số yêu cầu quyền Admin hoặc cài đặt vào Program Files |
| **Tự sửa lỗi đăng ký (Ghost Registration)** | Tự phát hiện và cập nhật đường dẫn khi di chuyển thư mục | Báo lỗi hoặc phải gỡ ra cài lại thủ công |
| **Chính sách riêng tư (G8)** | **100% Offline**, zero keylogging, không telemetry; tự tắt ở ô mật khẩu; mã nguồn mở minh bạch | Nhiều bộ gõ đóng mã nguồn, không có cam kết riêng tư minh bạch |

---

## Tính Năng Đầy Đủ (Tích luỹ đến 0.2.72)

### 1. Công thái học Dream RSI & Kiểm chuẩn Hiệu năng (P10 / G19)
- Tích hợp công nghệ đo lường công thái học Dream RSI: 4 chỉ số cốt lõi (`keys_typed`, `accepts`, `chars_saved`, `savings_rate_pct`).
- Phân tích hiệu quả gợi ý theo thời gian thực: tiết kiệm 22% số phím gõ trong tác vụ văn phòng thực tế (36 lượt nhận / 72 phím tiết kiệm trên corpus benchmark chuẩn).
- 5 đường nhận diện gợi ý: Tab hoàn tất tiếng Việt, Tab hoàn tất tiếng Anh/style-word, Space nhận từ, duyệt ứng viên (cycle), gợi ý cụm từ tiếp theo (next-phrase).
- Chặn hồi quy tự động trong CI: mọi cải tiến gợi ý bắt buộc phải duy trì hoặc tăng tỷ lệ tiết kiệm phím so với baseline (`perf/baseline-win.json`).

### 2. Mô hình Dự đoán Ngữ cảnh P6 & Hồ sơ Phong cách P7 (Corpora 41.313 mục)
- **Tập dữ liệu tri thức 41.313 mục** song ngữ chất lượng tương đương:
  - 6.672 âm tiết Việt chuẩn mực + 1.941 từ ghép Việt (`vn_compounds.txt`, 13 domain chuyên ngành).
  - 3.346 từ Anh tần suất cao (`en_freq3000.txt`) + 9.868 từ đơn Anh (`en_words10k.txt`) + 1.957 từ ghép & cụm danh từ Anh (`en_compounds.txt`, 16 domain chuyên sâu).
  - 17.529 cặp câu/cụm/thành ngữ (8.782 mẫu Việt `vn_phrases.tsv` + 8.747 mẫu Anh `en_phrases.tsv` — câu thật chia mọi ranh giới từ, thành ngữ thật, mẫu câu công sở theo vai trò).
- Hoạt động đồng bộ ở **cả 2 chế độ**:
  - **Mode EN (`E`)**: Hoàn tất từ đơn/từ ghép, duyệt vòng 5 ứng viên bằng Tab, đoán câu tiếp theo, ghost preview 75 µs, học từ Dream RSI.
  - **Mode VI (`V`)**: Tự động nhận diện từ/câu tiếng Anh kể cả khi bị Telex fold (ví dụ: `customer` → `cútomer` + Tab → `customer service`; `looking forward` → `looking fỏwad` + Tab → `looking forward to hearing from you soon`).
- **User Style Profile (P7)**: Tự học từ cá nhân & cặp sửa lỗi, opt-in, 100% offline, giới hạn an toàn 200 từ / 50 cặp.
- Tra cứu cấu trúc Trie + Frecency tốc độ micro-giây, xếp hạng mở rộng đúng nghĩa ngữ cảnh.

### 3. Kiểu Gõ & Biến Đổi Âm Vị Học Chuẩn Mực
- Vần `uo` chuẩn UniKey 20 năm: `luong`/`uong`/`truong.giap` giữ nguyên khi không dấu; chỉ khi có biến đổi VN hoặc gõ `w`/dấu chủ động mới thành `ươ` (`luonwg`, `tuowi`, `tuowng`, `duongf` → `dường`).
- Chống xung đột 2 bộ gõ (swallow-and-unicode): trên app bypass TSF (MobaXterm, KiTTY…), hook tự nuốt phím và inject qua KEYEVENTF_UNICODE ngăn Windows VIE IME cướp phím và hiện ô chữ nhật nhỏ.
- Khôi phục dead-key Windows (`Ctrl+'`+`e` → `é`) trên bàn phím quốc tế.
- Chặn hoàn tất token ID/username: không bao giờ tự động hoàn tất qua ranh giới dấu chấm, gạch ngang, email (`truong.giap`, `t.ho`).
- Đầy đủ Telex, VNI, TCVN3 (VIQR đọc được), Quick Telex (cc→ch, gg→gi…), gõ tự do (free marking).
- 3 bảng mã xuất: Unicode dựng sẵn, TCVN3, VNI (+ Unicode tổ hợp) — chuyển đổi qua lại giữ nguyên chữ.
- Kiểu dấu cũ/mới chọn được (`hoà`/`hòa`, `thuỷ`/`thủy`) — toàn bộ dữ liệu dự đoán tự động khớp theo kiểu đã chọn.
- Chữ hoa thông minh: `DD` → `Đ`, Caps Lock/Shift từng phím, tự viết hoa đầu câu (tắt được).
- Escape khôi phục nguyên chuỗi phím đã gõ ở mọi thời điểm.
- Chống vòng lặp phím (E28): gõ lặp dấu không sinh chu kỳ vô tận trên cả Telex, VNI và VIQR.
- Gõ tắt (macro) theo kiểu chữ (HD→HỢP ĐỒNG), hoạt động được cả khi tắt tiếng Việt.
- Emoji gõ tắt; số tiền `=1.550.00đ` → chữ Việt; bảo vệ công thức Excel và định danh lập trình.

### 4. Trợ Lý Ngữ Pháp Câu & Từ Ngữ Cảnh On-Device
- 15 loại lỗi phát hiện tự động: lặp từ, lặp cụm 2–4 từ, khoảng trắng quanh dấu câu, chữ thường đầu câu, sai dấu thanh, phủ định kép, phẩy cuối câu, phẩy sau liên từ, a/an, cặp từ dễ nhầm EN, đại từ "i", chính tả EN (OS spell), chính tả VI (từ điển âm tiết dựng sẵn), ngày tháng vô lý (31/02, 32/13), số tiền phân cách hỗn loạn (1.250.00đ).
- Panel gợi ý khi **dừng gõ 1 giây** (cả 3 hệ điều hành): mọi phím gõ chữ/dấu câu/xoá hẹn lại đồng hồ, gõ tiếp là ẩn ngay, dừng đủ 1 s mới kiểm tra từ + câu rồi hiện (cả chế độ gợi ý lẫn sửa lỗi an toàn, kể cả sau `.` `,`); giữa từ đang gõ chỉ Windows hiện (Linux/macOS đợi từ chốt).
- Panel hỗ trợ chuột bấm, Alt+1…3, Enter áp "Sửa tất cả".
- Chế độ 1 chỉ gợi ý / Chế độ 2 tự sửa lỗi khách quan — hoàn tác Ctrl+Z trong 5 giây.
- Kiểm tra theo vùng bôi đen (Shift+mũi tên → popup + Sửa tất cả).
- Tuỳ chọn tắt riêng từng ứng dụng (per-app); tự tắt ở ô mật khẩu; 0% báo nhầm (FPR) trên tập kiểm thử 2.739 câu chuẩn mực.

### 5. Tiện Ích Chuyển Mã & Văn Phòng Đột Phá (P5)
- Ctrl+Shift+F9 chuyển mã vùng bôi đen: TCVN3/VNI/Unicode tổ hợp → Unicode dựng sẵn, tự nhận diện bảng mã (`Tiáº¿ng Viá»‡t` → `Tiếng Việt`), gõ lại chữ gõ nhầm khi quên bật (`tooi ddi hocj` → `tôi đi học`) trên cả 3 hệ điều hành.
- Khôi phục tiếng Anh thông minh: luật ngữ âm Việt + 103 từ Anh dễ nhầm + 116 thuật ngữ EN chuyên ngành + từ điển cá nhân.
- Cơ chế bảo vệ: bảo vệ định danh camelCase, chuỗi nguyên văn trong ngoặc kép.

### 6. Kiến Trúc Hệ Thống, Điều Khiển & Riêng Tư Tuyệt Đối
- Phím nóng: Ctrl+Space sửa/dự đoán · Ctrl+Shift gạt VN/EN (nhớ trạng thái) · Ctrl+Shift+F9 chuyển mã.
- Bảng cài đặt native từng OS: Windows khay hệ thống (7 nút có icon + tooltip, phân định Ẩn vào khay/Thoát), Linux GTK4, macOS menu bar SwiftUI.
- Offline 100%, không gửi dữ liệu ra mạng, không lưu nội dung phím gõ; tự ngắt khi gặp trường mật khẩu; chính sách bảo mật công khai trong repo.

---

## Tải và Cài Đặt

Bản mới nhất: **[GitHub Release v0.2.73](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.73)** (release candidate do CI build và kiểm thử tự động). **Mọi file phát hành đều có chữ ký GPG (`.asc`) và Sigstore (`.cosign.sig` + `.cosign.cert`)**, file kiểm tra `SHA256SUMS.txt` được clearsign — hướng dẫn kiểm tra: [signing.md](docs/release/signing.md).

| Nền tảng | Cài đặt chính thức | Dùng ngay không cần cài đặt |
|---|---|---|
| **Windows 10/11 x64** | `TextVN-setup-<ver>-windows-x64.exe` (cho tài khoản cá nhân, không cần quyền admin/UAC)<br>`TextVN-setup-<ver>-windows-x64-machine.exe` (cho mọi người dùng máy, Program Files, UAC một lần) | `TextVN-portable-<ver>-windows-x64-*.zip` → giải nén chạy ngay `TextVN.exe` |
| **Windows Microsoft Store** | ✅ **[TextVN trên Microsoft Store](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)** (miễn phí, Store ID `9NV2R7JNNFGK`) — tự động cập nhật, gói do Microsoft ký số xác thực. Cài nhanh qua terminal: `winget install --id 9NV2R7JNNFGK -s msstore`. *(Gói `.msix` trên GitHub Release là bản nộp Store chưa ký)* | — |
| **macOS 13+ (Apple Silicon & Intel)** | **Homebrew (khuyên dùng): `brew install --cask hunglinhpt/tap/textvn`** — tự động cài bộ gõ vào `~/Library/Input Methods` và ứng dụng thanh menu vào `/Applications`; nâng cấp qua `brew upgrade`.<br>Hoặc dùng file gói `TextVN-mac-v<ver>.pkg` (cài đặt per-user, không cần quyền root/sudo) | `TextVN-macos-universal-v<ver>.zip` (chứa `TextVN-IM.app` + `TextVN.app` universal binary) — xem [Hướng dẫn sử dụng macOS](docs/user-guide.md) |
| **Linux (IBus / Fcitx5)** | **Arch Linux (AUR)**: `paru -S textvn` (hoặc `yay -S textvn`) — build từ mã nguồn tag sạch sẽ trong CI.<br>**Ubuntu / Debian PPA**: `sudo add-apt-repository ppa:hunglinhpt/textvn && sudo apt install textvn`<br>Hoặc tải tarball: `tar xzf TextVN-<ver>-linux-x86_64.tar.gz` → chạy `./install.sh` | Chạy trực tiếp từ tarball giải nén: `./textvn-portable.sh` |

Chi tiết cài đặt, gỡ bỏ, hướng dẫn sử dụng và khắc phục sự cố: **[Tài liệu Hướng dẫn sử dụng](docs/user-guide.md)**.

> [!NOTE]
> **Lưu ý Windows 11 24H2+**: Một số bản cập nhật mới của Windows từ chối kích hoạt bộ gõ chỉ đăng ký trong phạm vi tài khoản per-user (mã lỗi B7 trong `docs/specs/win-test-common-errors.md`). Bản cài đặt tài khoản và bản portable của TextVN **không bao giờ đòi hỏi UAC** — khi phát hiện điều kiện này, ứng dụng sẽ thông báo rõ ràng và hướng dẫn người dùng sử dụng bản cài đặt `-machine.exe`.

---

## Bản Phát Hành Gần Đây

Chi tiết đầy đủ lịch sử phát triển: [CHANGELOG.md](CHANGELOG.md).

| Phiên bản | Điểm nhấn tính năng |
|---|---|
| [0.2.73](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.73) | Trợ lý ngữ pháp **bắt lỗi dấu thanh** ("chềt" → "chết") bằng từ điển âm tiết dựng sẵn trên cả 3 OS — không cần từ điển hệ điều hành, 99% lỗi dấu thanh tổng hợp được bắt, 0% báo nhầm trên corpus sạch; "Sửa tất cả" bỏ qua gợi ý mơ hồ · sửa nhầm lặp từ láy còn sót |
| [0.2.72](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.72) | Bảng điều khiển: toàn bộ nút bấm có **biểu tượng + gợi ý tooltip** (cả 3 OS); Windows đổi nút thành **"Ẩn vào khay"/"Thoát"** để tránh nhầm lẫn đóng là tắt ứng dụng, bỏ nút "Cài & bật TSF" (tray tự động đăng ký) · Cập nhật README chính xác tính năng và số liệu thật |
| [0.2.71](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.71) | Linux (IBus/Fcitx5) và macOS: danh sách gợi ý của trợ lý **hiện khi dừng gõ 1 giây**, ẩn ngay khi gõ tiếp — đạt đồng bộ hoàn toàn với hành vi Windows 0.2.70; phím kết câu tự sửa ngay ở chế độ sửa lỗi an toàn |
| [0.2.70](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.70) | Windows: bảng trợ lý ngữ pháp **hiện lại khi dừng gõ 1 giây** (kể cả giữa từ, sau dấu chấm phẩy; chế độ gợi ý lẫn sửa lỗi an toàn) · "Sửa tất cả"/Alt+1…3 giữa từ không cắt đôi từ · Kiểm thử E2E thật trên Microsoft Edge 18/18 |
| [0.2.69](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.69) | Audit toàn bộ tính năng mới (R2-140…R2-153, DFMEA/DRBFM): đồng bộ 3 OS — dự đoán Mode EN, `Ctrl` đơn, tự học lưu được trên macOS/Linux · nhận diện terminal Windows/Linux (giữ Tab cho shell, không học mật khẩu) · Telex không còn phá code/Markdown (`arr[0]`, `--force`) · trợ lý không sửa oan 40 mẫu câu đúng · `peek_completion` 225 → 75 µs · savings-rate 22% |
| [0.2.68](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.68) | Âm vị học `sort` → `sỏt` khôi phục đúng · tiện ích Telex `[ ]`, `Ctrl`, `a--b` · dự đoán tiếng Anh 2 chế độ, từ ghép và mẫu câu Anh |
| [0.2.67](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.67) | P10 Dream RSI (G19): đo lường tiết kiệm phím (22% savings-rate trên corpus văn phòng) · Gate chống hồi quy `perf/baseline-win.json` · Chuẩn hoá `keys_typed` 3 OS · Hoàn thiện pipeline tự học P7 (IPC broadcast `ConfigReload`, opt-in consent G8, FFI `catch_unwind` an toàn) |
| [0.2.66](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.66) | Tự học P7 hoàn chỉnh qua phím thật (drain cap 200 từ/50 cặp, IPC merge, live reload) · P9 debounce 1s cho word-trigger · Dead-key armed disarm |
| [0.2.65](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.65) | Chuẩn UniKey vần uo ("luong"/"uong" plain, `luonwg`/`tuowi`/`tuowng`) · Hook swallow-and-unicode chống xung đột OS-IME (MobaXterm) · Dead-key Windows (`Ctrl+'`+`e`→`é`) · Chặn hoàn tất username/id (`truong.giap`) |
| [0.2.60–0.2.64](CHANGELOG.md) | Corpora 41.313 mục · 13 domain từ ghép mới · User Style Profile · Cảnh báo ngày vô lý (31/02) · Tối ưu hiệu năng tuyến tính O(n) |
| [0.2.50–0.2.59](CHANGELOG.md) | Ngữ pháp câu EN + SVA · Lặp cụm 2–4 từ · Chuyển mã thông minh Ctrl+Shift+F9 · Ghost preview 3 OS · Gõ tắt kiểu chữ |

---

## Trạng Thái & Hỗ Trợ Nền Tảng

| Nền tảng | Trạng thái phát hành |
|---|---|
| **Windows 10/11 x64 (TSF)** | ✅ Release candidate — gõ thật qua TSF được kiểm thử liên tục trong CI trên Windows (bộ cài, portable, MSIX cài thử). **Đã phát hành trên [Microsoft Store](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)** (gói MSIX chính thức). Chữ ký Authenticode trực tiếp đang chờ SignPath Foundation duyệt. |
| **macOS 13+ (Apple Silicon & Intel)** | ✅ Release candidate — bộ gõ IMK + app thanh menu + gói cài đặt `.pkg` per-user (không cần sudo); CI tự động build 2 kiến trúc (arm64 + x86_64) và chạy `swift test`. Chữ ký Apple Developer ID / Notarization đang chờ duyệt ([signing-status-mac.md](docs/release/signing-status-mac.md)). |
| **Linux (IBus / Fcitx5)** | ✅ Release candidate — hỗ trợ cài đặt per-user hoặc `--system`, bảng cài đặt GTK4; CI kiểm thử tự động với `ibus-daemon` và `fcitx5` thật. Phân phối chính thức qua AUR (`paru -S textvn`) và PPA Ubuntu/Debian (`ppa:hunglinhpt/textvn`). |

Báo cáo kiểm thử chi tiết của bản phát hành: [build-release-report.md](docs/release/build-release-report.md).

---

## Dựng Từ Mã Nguồn

```powershell
# Windows: Cần Rust stable + Visual Studio 2022 Build Tools (C++) + Inno Setup 6.5+
powershell -NoProfile -ExecutionPolicy Bypass -File .\build-release.ps1 -BuildInstaller -MachineInstaller
```

```bash
# macOS 13+: Cần Xcode Command Line Tools 15+, rustup target aarch64-apple-darwin & x86_64-apple-darwin
cd adapters/macos-imk
./build-rust.sh                   # Dựng engine FFI + Swift build + tạo TextVN-IM.app
./build-rust.sh --lib-only        # Chỉ dựng libtextvn_ffi.a (phục vụ swift test)
./build-rust.sh --release         # Dựng bản tối ưu phát hành

# Đóng gói phân phối macOS (chạy từ thư mục gốc)
scripts/build-macos.sh            # Đóng gói Universal zip
scripts/package-macos-pkg.sh      # Tạo file cài đặt .pkg per-user
```

```bash
# Linux (Ubuntu/Debian / Fedora / Arch)
scripts/install_linux.sh          # Dựng mã nguồn và cài đặt cho tài khoản hiện tại
scripts/build-linux.sh            # Tạo tarball phát hành hoàn chỉnh
```

Kiến trúc, kiểm thử, chuẩn mã và quy trình phát hành: xem **[Hướng dẫn phát triển](docs/developer-guide.md)**.

---

## Cấu Trúc Dự Án

```
core/              Engine xử lý ngữ âm cốt lõi (Rust) — không I/O, không phụ thuộc hệ điều hành
config/  ffi/      Đặc tả cấu hình config.v1 · C ABI và shim an toàn cho các adapter
adapters/          windows-tsf (TSF thuần) · windows-hook (gói tương thích ứng dụng legacy)
                   macos-imk (Input Method Kit) · macos-app (SwiftUI menu bar) · macos-tap
                   linux-ibus · linux-fcitx5 · linux-common · linux-settings (GTK4)
tray/  cli/        Khay hệ thống & bảng cài đặt Windows · textvn-cli (đăng ký TSF, doctor, replay)
corpus/            Hơn 230 kịch bản gõ *.keys (shared · win · mac) chạy qua bộ mô phỏng adapter trong CI
installer/  packaging/  scripts/    Kịch bản đóng gói Inno Setup, MSIX, PKGBUILD, Homebrew Cask, Debian PPA
docs/              Tài liệu đặc tả kiến trúc (10-shared, 20-windows, 30-macos, 40-linux), specs, release
```

---

## Đóng Góp & Giấy Phép

- Đóng góp phát triển: xem [CONTRIBUTING.md](CONTRIBUTING.md) và [SECURITY.md](SECURITY.md).
- Khi báo lỗi gõ phím: vui lòng ghi rõ chuỗi phím đã bấm, kết quả thực tế, kết quả mong đợi, tên ứng dụng và phiên bản hệ điều hành.
- **Giấy phép**: [GPL-3.0-or-later](LICENSE) © 2026 LinhBH.CoM

---

# English

Tiếng Việt · English

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![CI Windows](https://img.shields.io/github/actions/workflow/status/hunglinhpt/TextVN/ci-windows.yml?branch=main&label=CI%20Windows)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-windows.yml?query=branch%3Amain)
[![CI Linux](https://img.shields.io/github/actions/workflow/status/hunglinhpt/TextVN/ci-linux.yml?branch=main&label=CI%20Linux)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-linux.yml?query=branch%3Amain)
[![CI macOS](https://img.shields.io/github/actions/workflow/status/hunglinhpt/TextVN/ci-macos.yml?branch=main&label=CI%20macOS)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-macos.yml?query=branch%3Amain)
[![CI engine (3 OS)](https://img.shields.io/github/actions/workflow/status/hunglinhpt/TextVN/ci-shared.yml?branch=main&label=CI%20engine%20%283%20OS%29)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-shared.yml?query=branch%3Amain)

[![Get TextVN on Microsoft Store](https://get.microsoft.com/images/en-us%20dark.svg)](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)

**TextVN** is a next-generation open-source Vietnamese input method built upon a single, unified **Rust** engine (strictly memory-safe, zero I/O inside core, zero panics across FFI via `catch_unwind`) shared across all three platforms: **Windows** (pure Text Services Framework), **macOS** (native Input Method Kit), and **Linux** (IBus & Fcitx5). TextVN delivers microsecond-level responsiveness, uncompromising phonotactic accuracy, and completely eliminates the classic flaws of legacy hook-based IMEs (lost keystrokes, duplicated letters, and preedit underlines), while integrating an on-device contextual bilingual prediction system and an intelligent offline grammar assistant.

---

## 4 Core Pillars of Excellence

### 1. Pure In-Process Architecture & Uncompromising Memory Safety
- **Pure Windows TSF (`ITfTextInputProcessorEx`)**: The default package installs no global keyboard hook to eat keys, injects no DLLs into foreign processes, and never simulates fake keys via `SendInput` backspace sequences ([architectural rationale](docs/specs/antivirus-false-positive.md)).
- **No Lost Keys, No Doubled Letters, No Preedit Underline**: The entire word remains inside an internal composition range without any underline (`TF_LS_NONE` on Windows TSF; non-preedit mode on macOS; no underline on Linux). Permanently eliminates key eating, character duplication, and caret jumping in Google Chrome, Microsoft Edge, Electron apps (VS Code, Slack, Discord), Microsoft Excel, browser address bars, and full-screen games.
- **Antivirus False-Positive Immunity**: Operates strictly within official OS IME architecture boundaries, complying with Microsoft Store certification standards and enterprise antivirus policies.

### 2. Contextual Bilingual Prediction & Dream RSI Ergonomics (P10)
- **41,313-entry Curated Bilingual Knowledge Base**: Includes 6,672 standard Vietnamese syllables, 1,941 Vietnamese compounds (`vn_compounds.txt`, 13 specialized domains), 3,346 high-frequency English words (`en_freq3000.txt`), 9,868 English single words (`en_words10k.txt`), 1,957 English compounds & collocations (`en_compounds.txt`, 16 modern domains: AI, Cloud, Dev, Finance, HR, Legal…), and 17,529 context→phrase pairs (8,782 Vietnamese in `vn_phrases.tsv` and 8,747 English in `en_phrases.tsv`).
- **Synchronous Dual-Mode Parity (Mode VI & Mode EN)**:
  - **Mode EN (`E`)**: Single-word & compound completion, next-phrase prediction, 5-candidate Tab cycling, and ultra-fast 75 µs ghost preview.
  - **Mode VI (`V`)**: Intelligent mixed English detection. Automatically rescues English words/phrases mistransformed by Telex rules when they do not form valid Vietnamese syllables (e.g., `customer` typed as `cútomer` + Tab → expands to `customer service`; `looking forward` typed as `looking fỏwad` + Tab → `looking forward to hearing from you soon`).
- **Standalone `Ctrl` Tap Utility**: Tap and release standalone `Ctrl` to lock valid syllables (`tunx` + Ctrl → commits `tũn`), or instantly restore mistyped Vietnamese words to their original English form (`sort` → `sỏt` + Ctrl → `sort`).
- **Dream RSI Ergonomic Measurement (P10 / G19)**: Objectively measures physical keystrokes saved. Delivers a 22% keystroke savings rate on realistic office corpora; strictly guarded against performance regressions via automated CI gates (`perf/baseline-win.json`).
- **User Style Profile (P7)**: Automatically learns personal vocabulary and accepted correction pairs, 100% offline, strictly opt-in per privacy rule G8, with safe memory caps (200 words / 50 pairs).

### 3. On-Device Grammar & Context Assistant (Grammarly-Grade Offline)
- **Comprehensive 15-Category Issue Detection**: Detects word repeats, 2–4-word phrase repeats, punctuation spacing, sentence capitalization, misplaced tone marks ("nghiã" → "nghĩa"), **wrong tone marks via a built-in syllable dictionary ("chềt" → "chết", 99% of synthetic tone errors caught, 0% false positives on the clean corpus — no OS dictionary needed)**, double negatives, post-conjunction commas, English a/an articles, easily-confused English pairs, lowercase pronoun "i", English/Vietnamese spelling, calendar-aware invalid dates (31/02, 32/13), and inconsistent money grouping (1.250.00đ).
- **1-Second Idle Trigger Across All 3 OSes**: Pause typing for 1 second → assistant panel appears at the caret, with red underlines on errors and total issue count; resume typing → panel instantly dismisses without disrupting focus.
- **Two Flexible Operating Modes**: *Suggest-Only* (Mode 1) for unobtrusive assistance or *Safe Auto-Fix* (Mode 2) for automatic correction of clear-cut errors with a 1-step Ctrl+Z undo window for 5 seconds.
- **Zero False-Positive Standard (0% FPR)**: Verified 0 % false alarms on a curated benchmark of 981 Vietnamese and 295 English clean sentences ([P4-12 §11](docs/50-assist/P4-12-RECALL-REAL.md)).

### 4. 20-Year UniKey Heritage Standard & Modern Office Utilities
- **UniKey Standard Tone Placement**: Comprehensive support for Telex, VNI, VIQR, and Simple Telex; old-style (`hòa`, `thủy`) vs new-style (`hoà`, `thuỷ`) diacritics; standard `uo` rhyme (`luong`/`uong` plain by default, explicit `w` produces `ươ`).
- **Smart Telex Conveniences**: Fast `[` / `]` for `ơ` / `ư` with full protection for programming code brackets (`arr[0]`, Markdown `- [ ] task`); fast `a--b` for em-dash `a—b` while preserving CLI flags (`--force`); smart uppercase `DD` → `Đ`; dead-key passthrough (`Ctrl+'`+`e` → `é`).
- **Smart Transcoding Shortcut Ctrl+Shift+F9**: Lossless bidirectional conversion between Unicode, TCVN3, VNI, and decomposed Unicode; automatic source charset detection (`Tiáº¿ng Viá»‡t` → `Tiếng Việt`); automatic retyping of mistyped text when the IME was off (`tooi ddi hocj` → `tôi đi học`) across all 3 platforms.
- **Unified 3-OS Settings Panel (v0.2.72)**: Every button features an icon + hint tooltip; explicit "Hide to tray" vs "Quit" options; tray automatically self-registers TSF without requiring administrator privileges (Admin/UAC).

---

## Highlights Over Traditional Vietnamese IMEs

Direct head-to-head comparison between TextVN and legacy input engines (UniKey, EVKey, OpenKey):

| Comparison Criteria | TextVN (v0.2.72) | Traditional IMEs (UniKey / EVKey / OpenKey) |
|---|---|---|
| **Input Processing Engine** | In-process TSF (Windows), native IMK (macOS), IBus/Fcitx5 (Linux) | Global keyboard hook + `SendInput` backspace replay |
| **Lost & Duplicated Letters (Chrome/Excel/Electron)** | **Completely eliminated** (clean composition, no underline) | Common occurrence during rapid typing or window switching |
| **Automatic EN/VI Detection in Mixed Text** | Vietnamese phonotactics + 103 confused words + 116 terms + personal dictionary + 94-syllable protection grid: restored on Space | Telex applies indiscriminately — `text` becomes `tẽt`, `is` becomes `í`; requires manual toggle |
| **Bilingual Word & Phrase Prediction (Tab/Space)** | Full single-word, compound, and contextual phrase prediction in both Mode VI & Mode EN; 5-candidate cycling; 75 µs ghost preview | Not supported or limited to static text macros |
| **Dream RSI Ergonomics (P10)** | Objectively measures physical keystrokes saved (22% savings rate on office workloads); CI regression gates | Not supported |
| **On-Device Grammar & Spell Assistant** | 15 issue types (word repeats, spacing, caps, misplaced tones, a/an, dates, money); 1s idle popup on 3 OSes; 0% FPR; Ctrl+Z 5s | Not supported |
| **Standalone Ctrl Tap Restore** | `tunx` + Ctrl → `tũn` (syllable lock); `sort` → `sỏt` + Ctrl → `sort` (phonotactic restore) | Not supported |
| **Raw Keystroke Restore (Escape)** | Pressing Escape immediately restores raw original keystrokes | Not supported or limited to full-sentence deletion |
| **Global Mode Toggle (Ctrl+Shift)** | Observer-only modifier tap detector + debounce prevents double-toggling between tray and engine | Prone to double-toggle glitches or antivirus interception |
| **Coding & Markdown Conveniences** | Smart `[` `]` for `ơ`/`ư` (protects `arr[0]`); `a--b` for `a—b` (protects `--flag`); `DD` → `Đ` | Blind transformation corrupts source code and Markdown syntax |
| **Smart Transcoding (Ctrl+Shift+F9)** | Auto-detects broken charsets (`Tiáº¿ng Viá»‡t` → `Tiếng Việt`), fixes text typed with IME off (`tooi ddi hocj` → `tôi đi học`) | Basic manual clipboard transcoding only |
| **Installation Permissions & UAC** | Per-user HKCU, portable extract-and-run, zero Admin/UAC required | Mostly requires Admin privileges or Program Files installer |
| **Ghost Registration Auto-Repair** | Automatically repairs registry paths when the executable folder is moved | Crashes, fails silently, or requires manual reinstall |
| **Privacy Policy & Telemetry (G8)** | **100% Offline**, zero keylogging, zero telemetry; auto-off in password fields; transparent open-source code | Many closed-source tools lack clear, verifiable privacy audits |

---

## Full Feature Set (Accumulated Through 0.2.72)

### 1. Dream RSI Ergonomic Metrics & Performance Benchmarks (P10 / G19)
- Integrated Dream RSI ergonomic measurement: 4 core metrics (`keys_typed`, `accepts`, `chars_saved`, `savings_rate_pct`).
- Real-time keystroke reduction analysis: achieves a 22% keystroke savings rate on realistic office workloads (36 accepts / 72 keys saved on benchmark corpus).
- 5 prediction accept pathways: Vietnamese word completion, English/style completion, Space-accept, candidate cycling, and multi-word next-phrase prediction.
- Automated CI anti-regression gate: all prediction improvements must maintain or increase the keystroke savings rate against the baseline (`perf/baseline-win.json`).

### 2. Contextual Prediction Model P6 & User Style Profile P7 (41,313-Entry Corpora)
- **Curated 41,313-entry knowledge base** with complete bilingual parity:
  - 6,672 standard Vietnamese syllables + 1,941 Vietnamese compounds (`vn_compounds.txt`, 13 specialized domains).
  - 3,346 high-frequency English words (`en_freq3000.txt`) + 9,868 English general vocabulary (`en_words10k.txt`) + 1,957 English compounds & collocations (`en_compounds.txt`, 16 modern domains).
  - 17,529 context→phrase pairs (8,782 Vietnamese rows in `vn_phrases.tsv` + 8,747 English rows in `en_phrases.tsv` — real sentences split at all word boundaries, idioms, role-based workplace patterns).
- Seamless dual-mode operation:
  - **Mode EN (`E`)**: Single-word and compound completion, 5-candidate Tab cycling, next-phrase prediction, 75 µs ghost text preview, Dream RSI personal learning.
  - **Mode VI (`V`)**: Automatic English word and phrase detection even when Telex-folded (e.g., `customer` → `cútomer` + Tab → `customer service`; `looking forward` → `looking fỏwad` + Tab → `looking forward to hearing from you soon`).
- **User Style Profile (P7)**: Personal vocabulary and correction pair learning, strictly opt-in, 100% offline, safe memory cap (200 words / 50 pairs).
- Microsecond lookups via Trie + Frecency structures, meaningful-extension contextual ranking.

### 3. Typing Engines & Phonotactic Transformation Standard
- 20-year UniKey-standard `uo` rhyme: `luong`/`uong`/`truong.giap` remain unaccented by default; only explicit `w` or tone keys produce `ươ` (`luonwg`, `tuowi`, `tuowng`, `duongf` → `dường`).
- Anti-conflict mechanism (swallow-and-unicode): on TSF-bypassing apps (MobaXterm, KiTTY…), hook swallows keystrokes and injects via KEYEVENTF_UNICODE to prevent Windows VIE IME from stealing keys and opening composition boxes.
- Windows dead-key passthrough (`Ctrl+'`+`e` → `é`) on international keyboard layouts.
- Identifier & username boundary gate: Space and Tab never complete across dots, dashes, or email addresses (`truong.giap`, `t.ho`).
- Full support for Telex, VNI, TCVN3 (VIQR readable), Quick Telex (cc→ch, gg→gi…), free marking.
- 3 output character sets: Pre-composed Unicode, TCVN3, VNI (+ decomposed Unicode) with lossless round-trip transcoding.
- Old vs new diacritic style options (`hoà`/`hòa`, `thuỷ`/`thủy`) — all prediction datasets automatically adjust to the selected style.
- Smart capitalization: `DD` → `Đ`, per-key Caps Lock/Shift, auto-capitalize sentence starts (optional).
- Escape key immediately restores raw original keystrokes at any point.
- Anti-key-loop protection (E28): repeated tone keys never produce infinite cycles across Telex, VNI, and VIQR.
- Text macros with case style awareness (HD → HỢP ĐỒNG), functional even when Vietnamese typing is toggled off.
- Emoji shortcuts; `=1.550.00đ` currency to Vietnamese words; Excel formula and programming identifier protection.

### 4. On-Device Grammar & Context Assistant
- 15 issue types: word repeats, 2–4-word phrase repeats, punctuation spacing, sentence-case, diacritic typos, double negatives, end-of-sentence commas, post-conjunction commas, a/an articles, easily-confused English words, lowercase pronoun "i", English spelling (OS spell), Vietnamese spelling (built-in syllable dictionary), impossible dates (31/02, 32/13), inconsistent money grouping (1.250.00đ).
- Suggestion panel on **1-second typing pause** across all 3 platforms: every keystroke/punctuation/backspace resets the timer, typing resumes hides it immediately, only a full 1-second pause checks the word + sentence and displays the panel (suggestion and safe auto-fix modes alike, including after `.` `,`); mid-word display is supported on Windows.
- Mouse-clickable UI, Alt+1…3 shortcuts, Enter to apply "Fix All".
- Mode 1 suggest-only / Mode 2 safe auto-fix with a 5-second Ctrl+Z undo window.
- Selection checking (Shift+arrow keys → popup + Fix All) and long-sentence checks.
- Per-application disable list; auto-off in password fields; verified 0% false alarms (FPR) on 2,739 clean benchmark sentences.

### 5. Smart Transcoding & Office Utilities (P5)
- Ctrl+Shift+F9 selection transcoding: converts TCVN3/VNI/decomposed Unicode → Pre-composed Unicode with automatic source charset detection (`Tiáº¿ng Viá»‡t` → `Tiếng Việt`), re-types text typed with IME off (`tooi ddi hocj` → `tôi đi học`) across all 3 platforms.
- Smart English restore: Vietnamese phonotactics + 103 confused English words + 116 technical terms + personal dictionary.
- Protection mechanisms: camelCase identifiers, quoted strings preserved verbatim.

### 6. System Architecture, Control & Total Privacy
- Hotkeys: Ctrl+Space fix/predict · Ctrl+Shift VI/EN toggle (state remembered) · Ctrl+Shift+F9 transcode.
- Native settings per OS: Windows system tray (7 buttons with icons + tooltips, explicit Hide to tray / Quit), Linux GTK4, macOS SwiftUI menu bar.
- 100% offline, zero telemetry, no network calls; auto-off in password fields; public in-repo privacy policy.

---

## Download & Installation

Latest release: **[GitHub Release v0.2.73](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.73)** (release candidate built and tested automatically by CI). **Every release artifact carries a GPG signature (`.asc`) and a Sigstore signature (`.cosign.sig` + `.cosign.cert`)**; `SHA256SUMS.txt` is clear-signed — verification guide: [signing.md](docs/release/signing.md).

| Platform | Official Installer | Run Without Installing (Portable) |
|---|---|---|
| **Windows 10/11 x64** | `TextVN-setup-<ver>-windows-x64.exe` (current user, no admin/UAC required)<br>`TextVN-setup-<ver>-windows-x64-machine.exe` (all users, Program Files, one UAC prompt) | `TextVN-portable-<ver>-windows-x64-*.zip` → extract and run `TextVN.exe` directly |
| **Windows Microsoft Store** | ✅ **[TextVN on Microsoft Store](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)** (free, Store ID `9NV2R7JNNFGK`) — automatic updates, package verified and signed by Microsoft. CLI: `winget install --id 9NV2R7JNNFGK -s msstore`. *(The `.msix` file on GitHub Release is the unsigned Store submission package)* | — |
| **macOS 13+ (Apple Silicon & Intel)** | **Homebrew (recommended): `brew install --cask hunglinhpt/tap/textvn`** — installs input method to `~/Library/Input Methods` and menu bar app to `/Applications`; updates via `brew upgrade`.<br>Or use the `TextVN-mac-v<ver>.pkg` installer (per-user installation, no root/sudo required) | `TextVN-macos-universal-v<ver>.zip` (universal binary with `TextVN-IM.app` + `TextVN.app`) — see [macOS User Guide](docs/user-guide.md) |
| **Linux (IBus / Fcitx5)** | **Arch Linux (AUR)**: `paru -S textvn` (or `yay -S textvn`) — clean build from tagged source in CI.<br>**Ubuntu / Debian PPA**: `sudo add-apt-repository ppa:hunglinhpt/textvn && sudo apt install textvn`<br>Or download tarball: `tar xzf TextVN-<ver>-linux-x86_64.tar.gz` → run `./install.sh` | Run directly from extracted tarball: `./textvn-portable.sh` |

Full installation, uninstallation, usage instructions and troubleshooting: **[User Guide](docs/user-guide.md)**.

> [!NOTE]
> **Windows 11 24H2+ Note**: Recent Windows updates may refuse to activate input methods registered solely in per-user scope (issue B7 in `docs/specs/win-test-common-errors.md`). TextVN's per-user installer and portable package **never prompt for UAC** — when this condition is encountered, TextVN notifies you with clear guidance to use the `-machine.exe` installer.

---

## Recent Release Highlights

Full development history: [CHANGELOG.md](CHANGELOG.md).

| Version | Key Highlights |
|---|---|
| [0.2.73](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.73) | Grammar assistant **catches wrong tone marks** ("chềt" → "chết") with a built-in syllable dictionary on all 3 OSes — no OS dictionary needed, 99% of synthetic tone errors caught, 0% false positives on the clean corpus; "Fix all" skips ambiguous suggestions · leftover reduplication false positives fixed |
| [0.2.72](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.72) | Control panel: all buttons equipped with **icons + hint tooltips** (all 3 OSes); Windows renames to **"Hide to tray"/"Quit"** to prevent accidental exits, drops the "Install & enable TSF" button (tray registers automatically) · Complete README overhaul matching actual capabilities |
| [0.2.71](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.71) | Linux (IBus/Fcitx5) and macOS: assistant suggestion list **appears on 1-second typing pause**, dismisses on typing — 100% parity with Windows 0.2.70; sentence-ending keys auto-fix instantly in safe mode |
| [0.2.70](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.70) | Windows: grammar assistant panel **re-appears on 1-second typing pause** (including mid-word, after punctuation; suggestion and safe modes) · "Fix All"/Alt+1…3 mid-word no longer splits words · Real-world E2E Edge 18/18 tests |
| [0.2.69](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.69) | Full audit of new features (R2-140…R2-153, DFMEA/DRBFM): 3-OS parity — Mode EN prediction, standalone `Ctrl`, persisted self-learning on macOS/Linux · terminal detection (Tab preserved for shell, passwords unlearned) · Telex code/Markdown safety (`arr[0]`, `--force`) · 0 false flags on 40 clean patterns · `peek_completion` 225 → 75 µs · 22% savings rate |
| [0.2.68](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.68) | Phonotactic restore `sort` → `sỏt` fixed · Telex conveniences `[ ]`, `Ctrl`, `a--b` · English prediction in both modes, English compounds and phrases |
| [0.2.67](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.67) | P10 Dream RSI (G19): keystroke savings measurement (22% savings rate on office corpus) · Anti-regression gate in `perf/baseline-win.json` · Normalized 3-OS `keys_typed` · Completed P7 self-learning pipeline (IPC broadcast `ConfigReload`, opt-in consent G8, safe FFI `catch_unwind`) |
| [0.2.66](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.66) | Full P7 self-learning from real keystrokes (drain cap 200 words / 50 pairs, IPC merge, live reload) · P9 1s debounce on word triggers · Dead-key armed disarm |
| [0.2.65](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.65) | UniKey-standard `uo` rhyme (`luong` plain, `luonwg`/`tuowng`) · Hook swallow-and-unicode fixes OS-IME conflict (MobaXterm) · Windows dead-key restored · Username/ID punctuation gate |
| [0.2.60–0.2.64](CHANGELOG.md) | 41,313-entry corpora · 13 new compound domains · User Style Profile · Impossible-date warnings (31/02) · Linear O(n) performance optimizations |
| [0.2.50–0.2.59](CHANGELOG.md) | Full EN grammar + SVA · 2–4-word phrase repeats · Smart Ctrl+Shift+F9 transcoding · 3-OS ghost preview · Case-aware text macros |

---

## Platform Support & Status

| Platform | Release Status |
|---|---|
| **Windows 10/11 x64 (TSF)** | ✅ Release candidate — real TSF typing is CI-verified on Windows (installer, portable, MSIX trial install). **Officially published on the [Microsoft Store](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)** (MSIX package). Direct Authenticode signature awaiting SignPath Foundation approval. |
| **macOS 13+ (Apple Silicon & Intel)** | ✅ Release candidate — IMK input method + menu bar app + per-user `.pkg` installer (no sudo); CI automatically builds both architectures (arm64 + x86_64) and runs `swift test`. Apple Developer ID / Notarization awaiting review ([signing-status-mac.md](docs/release/signing-status-mac.md)). |
| **Linux (IBus / Fcitx5)** | ✅ Release candidate — supports per-user or `--system` installation, GTK4 control panel; CI automated testing with real `ibus-daemon` and `fcitx5`. Officially distributed via AUR (`paru -S textvn`) and Ubuntu/Debian PPA (`ppa:hunglinhpt/textvn`). |

Detailed release test report: [build-release-report.md](docs/release/build-release-report.md).

---

## Build from Source

```powershell
# Windows: Requires Rust stable + Visual Studio 2022 Build Tools (C++) + Inno Setup 6.5+
powershell -NoProfile -ExecutionPolicy Bypass -File .\build-release.ps1 -BuildInstaller -MachineInstaller
```

```bash
# macOS 13+: Requires Xcode Command Line Tools 15+, rustup targets aarch64-apple-darwin & x86_64-apple-darwin
cd adapters/macos-imk
./build-rust.sh                   # Build FFI engine + Swift build + assemble TextVN-IM.app
./build-rust.sh --lib-only        # Build libtextvn_ffi.a only (for swift test)
./build-rust.sh --release         # Build optimized release binaries

# Package macOS releases (run from repo root)
scripts/build-macos.sh            # Package Universal zip
scripts/package-macos-pkg.sh      # Create per-user .pkg installer
```

```bash
# Linux (Ubuntu/Debian / Fedora / Arch)
scripts/install_linux.sh          # Build from source and install for current user
scripts/build-linux.sh            # Produce complete release tarball
```

Architecture, testing, code conventions and release process: see the **[Developer Guide](docs/developer-guide.md)**.

---

## Project Structure

```
core/              Core phonetic & linguistic engine (Rust) — zero I/O, OS-independent
config/  ffi/      Config specification config.v1 · Safe C ABI and shims for adapters
adapters/          windows-tsf (pure TSF) · windows-hook (legacy app compatibility package)
                   macos-imk (Input Method Kit) · macos-app (SwiftUI menu bar) · macos-tap
                   linux-ibus · linux-fcitx5 · linux-common · linux-settings (GTK4)
tray/  cli/        Windows tray & settings UI · textvn-cli (TSF registration, doctor, replay)
corpus/            230+ typing scenarios (*.keys) executed across adapter simulations in CI
installer/  packaging/  scripts/    Inno Setup, MSIX, PKGBUILD, Homebrew Cask, Debian PPA packaging scripts
docs/              Architecture specs (10-shared, 20-windows, 30-macos, 40-linux), specs, release
```

---

## Contributing & License

- Contributing guidelines: see [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md).
- Reporting typing bugs: please include the exact keystroke sequence, actual result, expected result, application name, and OS version.
- **License**: [GPL-3.0-or-later](LICENSE) © 2026 LinhBH.CoM
