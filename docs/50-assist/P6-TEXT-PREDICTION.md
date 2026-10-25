# P6 — Dự đoán văn bản song ngữ (Tab / Ctrl + Space): nghiệp vụ & audit

- **Ngày:** 2026-10-25 · **Trạng thái:** đã code trên `main` (`43fbe788`, `c6c29912`, `a3f983fb`), **chưa phát hành** — bản đầu tiên chứa P6 là bản sau 0.2.45.
- **Phạm vi:** `core/src/post/predict.rs`, `core/src/lib.rs` (`try_vn_complete`, `try_predict_next_phrase`, `try_cycle_predict`), dữ liệu `data/vn_compounds.txt` · `data/vn_phrases.tsv` · `data/en_phrases.tsv`, adapter Ctrl+Space ở `adapters/windows-tsf/`, `adapters/macos-imk/`, `adapters/linux-common/` + `linux-ibus/` + `linux-fcitx5/`.
- **Tham chiếu:** `../00-WORKFLOW.md` (G1–G17) · `../01-AGENT-HANDBOOK.md` (S1–S9) · `P4-0-MASTER-PLAN.md` (trợ lý ngữ pháp — Ctrl+Space dùng chung) · `P5-OFFICE-PRODUCTIVITY-PLAN.md`.

## 1. Nghiệp vụ (hành vi chốt)

Bật bằng `suggest.enabled = true` (P0-3 §1.1). **Mặc định tắt**: khi tắt, Tab/Ctrl+Space giữ nguyên hành vi trước P6 (Tab tới app; Ctrl+Space là phím lệnh của app).

| # | Quy tắc | Ví dụ |
|---|---|---|
| P6-1 | **Thứ tự khi nhấn Tab / Ctrl+Space** (không Shift): (a) đang duyệt vòng → ứng viên kế (ưu tiên hơn cả gõ tắt); (b) đuôi câu khớp **khoá nhiều từ** → cụm nối tiếp; (c) hoàn tất tiền tố tiếng Việt; (d) hoàn tất từ tiếng Anh; (e) khoá **một từ** → cụm nối tiếp; (f) không gì khớp → Tab tới app / Ctrl+Space sang trợ lý on-demand | `phát triển`+Tab → `phát triển phần mềm` (b), `phát`+Tab → `phát triển` (c), `xin`+Tab → `xin chào` (e) |
| P6-2 | Cụm nối tiếp chạy được **ngay sau từ vừa gõ** (chưa có dấu cách): từ đang mở được chốt nguyên trạng và nối `" " + cụm` trong cùng một `Replace` | `Kính gửi`+Tab → `Kính gửi quý khách hàng` |
| P6-3 | Duyệt vòng: Tab/Ctrl+Space liên tiếp thay **chỉ phần ứng viên** (giữ từ ngữ cảnh), vòng lại ứng viên đầu; phím khác bất kỳ huỷ vòng | `Kính gửi`+Tab+Tab → `Kính gửi ban giám đốc` |
| P6-4 | Chữ hoa — **hoàn tất từ đang gõ**: viết hoa đầu → hoa đầu; HOA TOÀN BỘ (≥ 2 chữ cái) → hoa toàn bộ | `Phát`+Tab → `Phát triển`, `PHÁT`+Tab → `PHÁT TRIỂN` |
| P6-5 | Chữ hoa — **cụm nối tiếp**: KHÔNG thừa kế hoa đầu của từ trước; chỉ HOA TOÀN BỘ (≥ 2 chữ cái) kéo theo; dữ liệu giữ chữ hoa chuẩn (tên riêng, tiêu ngữ) | `Xin`+Tab → `Xin chào`; `CỘNG HÒA`+Tab → `CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM`; `Độc lập`+Tab → `Độc lập - Tự do - Hạnh phúc` |
| P6-6 | Kiểu dấu: dữ liệu viết kiểu cũ; ngữ cảnh và tiền tố được so ở cả hai kiểu; ứng viên chèn theo `diacritic_style` của người dùng | kiểu mới: `Cộng hoà`+Tab → `Cộng hoà xã hội chủ nghĩa Việt Nam`, `hoà`+Tab → `hoài` |
| P6-7 | Xuống dòng = ngữ cảnh mới: đuôi câu kết thúc bằng `\n`/`\r` (bỏ qua space/tab sau đó) không dự đoán | `Kính gửi⏎`+Tab → Tab tới app |
| P6-8 | Ctrl+Space không khớp dự đoán: engine trả `PASS` + `WORD_END`, **nhả quyền sở hữu từ** nhưng **giữ buffer câu** cho trợ lý on-demand; adapter chốt composition/preedit, chạy trợ lý; popup hiện → ăn phím; không → phím tới app và adapter **reset engine** (như chord B6) | xem R2-121 |
| P6-9 | VN tắt: Ctrl+Space như chord (quên đuôi text), không ghi dấu cách ảo | — |
| P6-10 | Mọi kết quả ≤ `MAX_TEXT` (64 glyph sau encode) — vượt thì fail-open (không dự đoán), không cắt cụt | — |

Popup trợ lý mở bằng Ctrl+Space (Windows): ↑/↓ **hoặc Tab/Shift+Tab** chuyển lỗi. Popup **tự hiện** sau dấu câu (Mode 1) không dùng Tab — Tab đóng popup rồi tới app (R2-122).

## 2. Audit 2026-10-25 — finding

Audit theo yêu cầu chủ repo ("audit sâu nghiệp vụ, technical, test end to end, fix toàn bộ bug còn sót"). Mỗi lỗi được tái hiện bằng test trước khi sửa (`00-WORKFLOW.md` §9); test hồi quy FAIL trên mã cũ, PASS trên mã mới.

| ID | Mức | Hiện tượng (tái hiện) | Nguyên nhân gốc | Sửa · guard |
|---|---|---|---|---|
| R2-120 | major | `phát triển`+Tab → "phát triển **khai**", `hợp đồng`+Tab → "hợp đồng **ý**"; `Kính gửi`+Tab / `Thank you`+Tab (chưa có dấu cách) → chèn Tab; `Xin `+Tab → "Xin **Chào**"; `Cộng hòa `+Tab → "… **việt nam**"; `Độc lập `+Tab → "Độc lập tự do hạnh phúc"; `Kính gửi⏎`+Tab → " quý khách hàng" ở đầu dòng mới; kiểu dấu mới `Cộng hoà` không bao giờ khớp; Tab thứ hai có thể bung gõ tắt thay vì duyệt vòng | Hoàn tất tiền tố của **âm tiết cuối** chạy trước, bỏ qua ngữ cảnh nhiều từ; cụm nối tiếp chỉ chạy khi từ đã đóng; `adapt_capitalization` dùng chung cho cả hoàn tất lẫn cụm nối tiếp; dữ liệu viết thường tên riêng; `trim()` nuốt cả `\n`; khoá chỉ viết kiểu dấu cũ; duyệt vòng chạy sau `try_macro` | P6-1…P6-7 ở §1 · `core/tests/text_prediction.rs` `r2_120_*` + unit test `predict.rs` |
| R2-121 | major | Ctrl+Space không khớp dự đoán: Windows/Linux giữ composition/preedit **mở** và engine giữ quyền sở hữu từ trong khi phím tới app (Win32 Edit chèn dấu cách cho Ctrl+Space, IDE mở gợi ý…) → buffer câu/gõ tắt lệch document (lớp lỗi B7); macOS trả `false` kể cả khi đã hiện bảng gợi ý (app nhận thêm Ctrl+Space) và Mode 2 **tự sửa im lặng** thay vì hiện gợi ý; VN tắt thì ghi một dấu cách ảo vào đuôi text | Commit `c6c29912`/`a3f983fb` miễn Ctrl+Space khỏi nhánh chord nhưng không thay ngữ nghĩa chord (chốt từ + quên đuôi) bằng gì | P6-8/P6-9 · `compose.rs` (plan `pass_through`), `edit_session.rs` (`reset_engine` khi không ăn phím, chỉ ăn khi popup thật sự hiện), `compose.c` (`reset_engine`), `TextVNInputController.swift` · test `r2_121_*`, `ctrl_space_plan_never_inserts_space_on_pass`, `test_compose.c` |
| R2-122 | major | Popup trợ lý **tự hiện** sau dấu câu nuốt phím Tab (chuyển ô form/Excel, thụt lề) để chuyển lỗi | `c6c29912` thêm Tab → `Next` vào bản đồ phím chung, trái P4-0 §4.2 ("phím khác: đóng overlay rồi đi tiếp") | `ActiveAssist.on_demand` + `overlay_action_for` (Tab chỉ chuyển lỗi ở popup Ctrl+Space; Shift+Tab = lỗi trước) · unit test trong `assist.rs` |
| R2-123 | minor | App IMM32 qua CUAS (Notepad cổ điển, WinForms): Tab hoàn tất/dự đoán hay gõ tắt bằng Tab, Ctrl+Space mở trợ lý → app nhận **thêm** Tab/Ctrl+Space | `OnKeyDown` replay phím đã hoãn **kể cả khi TIP đã ăn phím** | Không replay khi `eaten` (`key_event.rs`) — replay chỉ để trả phím ranh giới app phải nhận (`replay.rs`) |
| R2-124 | major | Máy chủ repo có **hai bản cài**: per-user 0.2.33 + máy 0.2.45; `HKCU\…\Run\TextVN` trỏ exe 0.2.33, `HKLM\…\Run\TextVN` trỏ exe 0.2.45 → tray nào lên trước thắng, tray cũ tự đăng ký lại COM HKCU (đè HKLM) → người dùng chạy TIP cũ, không có trợ lý/toast | Inno chỉ thấy bản cùng hive; `.iss` không phát hiện bản ở phạm vi còn lại (cùng AppId) | `HandleOtherScopeInstall` trong `PrepareToInstall`: cài máy → chạy uninstaller per-user (token gốc, im lặng) + dọn Run HKCU cũ; cài per-user khi đã có bản máy → dừng và hướng dẫn · CI `test-installer.ps1` kịch bản R2-124 |
| — | hygiene | `c6c29912` đổi `core/src/keymap.rs` sang CRLF (diff cả file) và làm `verified-ops.md` lẫn CRLF/LF | Ghi file ở text mode trên Windows | Trả về LF + `.gitattributes` (`*.rs *.c *.h *.cpp *.swift *.tsv text eol=lf`) · E31 |

**Chưa làm (ghi nhận, không phải lỗi hồi quy):** Linux chưa có trợ lý **on-demand** qua Ctrl+Space (chỉ có dự đoán; Ctrl+Space không khớp thì tới app như chord) · chưa có **ghost text** (xem trước ứng viên mờ trước khi nhấn Tab, kiểu Grammarly/Copilot) — cần UI overlay riêng từng OS · macOS: Ctrl+Space mặc định là phím tắt hệ thống *Chọn nguồn nhập trước* (IMK không nhận được nếu người dùng chưa tắt).

## 3. Kiểm chứng

- `cargo test -p textvn-core` (gồm `text_prediction` 16 test), `-p textvn-win-tsf` (96), `-p textvn-ffi` — xanh trên Windows host.
- Test mới chạy trên mã cũ (stash phần sửa nguồn): 9/16 `text_prediction` FAIL — chứng minh test bắt đúng lỗi.
- Swift (macOS) và C (Linux) chỉ xác thực được trên CI (`ci-macos`, `ci-linux`) — E16 / verified-ops B18.
- Installer: ISCC compile cả biến thể per-user lẫn `-machine`; hành vi cài thật kiểm trên runner `ci-windows` (kịch bản R2-124).
