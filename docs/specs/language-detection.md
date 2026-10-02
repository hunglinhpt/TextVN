# Xác định ngôn ngữ khi gõ — ma trận xung đột và thứ tự ưu tiên

> Áp dụng từ 0.2.8, bổ sung UI từ điển từ 0.2.9. Câu hỏi trung tâm khi có xung
> đột: **ưu tiên tiếng nào?** Trả lời ngắn: **mode đang bật thắng cặp mơ hồ**
> (mode VN → tiếng Việt thắng; mode EN → engine passthrough, tiếng Anh thắng
> tự nhiên). Dictionary người dùng là "quyết định tường minh" — chỉ đứng dưới
> secure field và macro.

## 1. Thứ tự xử lý mỗi phím (engine `core/src/lib.rs`)

| Cấp | Bước | Quyết định | Lý do |
|---|---|---|---|
| 0 | Secure field (mật khẩu…) | Engine TẮT — mọi phím pass, không nhớ gì (S3) | An toàn tuyệt đối, không có xung đột |
| 1 | Modifier đơn / chord (Ctrl+V, Ctrl+Z, Alt+Tab…) | Pass, xoá đuôi `recent` | Phím hệ thống (S9/B6); Ctrl+Z = undo của ứng dụng |
| 2 | **Macro** (Tab/Space trigger, không kèm Shift) | Macro khớp → expand, kết thúc | Dữ liệu tường minh của người dùng, khớp chính xác → thắng mọi phỏng đoán |
| 3 | **Shift+Tab / Shift+Space** | Pass | Combo hệ thống (S9) |
| 4 | VN tắt (`global_enabled=false` hoặc app đang tắt) | Passthrough toàn bộ; macro chỉ chạy nếu `allow_macro_when_vi_off` | **Mode EN: tiếng Anh thắng tự nhiên** — không transform, không gợi ý |
| 5 | Tab gợi ý EN (`try_english_complete`) | Chỉ khi từ đã bị transform + tiền tố nghiêm ngặt trong từ điển + fold **không** trùng âm tiết Việt thông dụng | Không bao giờ cướp Tab khi người dùng đang gõ tiếng Việt bình thường |
| 6 | Ranh giới từ (Space/Enter/Tab/dấu câu) — `should_restore` theo thứ tự con 6a→6d bên dưới | | |
| 7 | Escape | Restore nguyên chuỗi phím đã gõ | **Cứu thủ công thắng tất cả** — người dùng luôn có lối thoát |

## 2. Thứ tự con tại ranh giới từ (`restore_en::should_restore`)

| Thứ tự | Điều kiện | Kết quả | Ví dụ |
|---|---|---|---|
| 6a | `raw == display` (không biến đổi) | Giữ nguyên | `hello`, `may` |
| 6b | fold **không phải** âm tiết Việt hợp lệ | Restore raw | `download`→`dơwnload`, `water`→`watẻ`, `asdf`→`àd` |
| 6c | raw ∈ `config.english_words` (danh sách người dùng) | **Restore raw — thắng cả vn_common** | `cow` trong dict → `cow ` dù fold là `cơ` |
| 6d | raw ∈ `en_common` (từ điển dựng sẵn) VÀ fold ∉ `vn_common` | Restore raw | `text`→`text `, `is`→`is `, `saw`→`saw ` |
| 6e | fold ∈ `vn_common` (âm tiết Việt thông dụng) | **Giữ fold — tiếng Việt thắng cặp mơ hồ** | `cow`→`cơ`, `sex`→`sẽ`, `queen`→`quên`, `max`→`mã` |

## 3. Bảng xung đột có thể xảy ra và cách giải

| Xung đột | Ai thắng | Cách người dùng khác ý |
|---|---|---|
| Đang mode VN, gõ `cow` (muốn "cơ" hay English "cow"?) | **"cơ" (VN)** — mode đang bật | Muốn "cow": Escape, hoặc thêm `cow` vào Từ điển EN |
| Đang mode VN, gõ `text` (muốn "text" hay fold "tẽt"?) | **"text" (EN)** — "tẽt" không phải từ Việt thông dụng | Muốn "tẽt": Escape rồi gõ telex đúng "teext"? — `tẽt` không có nghĩa; không có thiệt hại |
| Đang mode VN, Tab sau "vn" khi có macro `vn` | **Macro** (cấp 2 trước cấp 5) | Xoá/đổi macro trong Gõ tắt… |
| Đang mode VN, Tab sau "dow" (muốn indent?) | **Gợi ý "download"** — từ đã transform → đây là từ tiếng Anh đang gõ dở | Bỏ chọn "Khôi phục từ tiếng Anh khi gõ sai" để tắt gợi ý + restore |
| Đang mode EN, gõ bất cứ gì | **EN thắng toàn bộ** — passthrough | Bật VN bằng Ctrl+Shift |
| `auto_restore_english=false` | **Engine giữ fold** — hành vi UniKey cổ điển; không restore, không gợi ý; Escape vẫn hoạt động | Bật lại trong Bảng điều khiển |
| Người dùng thêm `sẽ` vào Từ điển EN (từ Việt!) | **Người dùng** — engine restore "se"→…? TỐI THIỂU: file chỉ nhận chữ ASCII, "sẽ" bị loại khi lưu | UI chỉ nhận a–z nên không thể gây xung đột |
| Từ trong `en_common` trùng macro của người dùng | Macro thắng (trigger chạy trước khi từ kết thúc) | — |
| Caps Lock / HOA: `COW`, `Text` | Restore/gợi ý giữ đúng HOA-thường người dùng gõ; Tab gợi ý viết HOA chữ đầu nếu từ đang gõ HOA | — |
| VNI/VIQR method | Không có phím transform trùng tiếng Anh (VNI dùng số, VIQR dùng dấu câu) → các nhánh restore/gợi ý gần như không bắn; structural vẫn bảo vệ | — |
| Bảng mã TCVN3/VNI-Windows (charset encode đổi độ dài) | Restore/gợi ý dùng `owned` = số glyph đã phát theo charset → xoá đúng số ký tự | — |

## 4. Nguyên tắc điều chỉnh thiết kế (ghi để không lặp tranh luận)

1. **Mode là chủ**: cặp mơ hồ hai chiều luôn giải theo mode đang bật — mode VN
   thì tiếng Việt thắng, mode EN thì tiếng Anh thắng (passthrough).
2. **Tường minh > phỏng đoán**: macro và `english_words` của người dùng thắng
   mọi phỏng đoán của engine (chỉ dưới secure + phím hệ thống).
3. **Luôn có lối thoát**: Escape restore raw tại mọi thời điểm từ đang active.
4. **Đánh đổi ghi rõ**: bật từ điển dựng sẵn có nghĩa KHÔNG restore được những
   từ như `cow`/`sex`/`queen`/`max`/`quên` khi mode VN — đó là giá để gõ
   tiếng Việt không bị phá; người dùng đổi bằng Từ điển EN.
