# P4-11 — ĐO TÀI NGUYÊN ASSIST (F61) — TIP THẬT

> Mục tiêu **F61** (`P4-TASKS.md`): đo **RSS / binary / p95** của tài nguyên "Trợ lý
> ngữ pháp" **trong tiến trình IME/TIP thật** — không đo bằng CLI độc lập (S4a chỉ
> đo tiến trình riêng, không đại diện in-proc).
> Script: `tools/win/measure_assist_rss.ps1` (viết 2026-10-22, ASCII-only).

---

## 1. Phương pháp

| Thành phần | Cách đo |
|---|---|
| **DLL working set** (`dll_ws`) | `VirtualQueryEx` liệt kê vùng `MEM_IMAGE`, `GetMappedFileNameW` lọc đúng file DLL, `QueryWorkingSetEx` đếm **page 4 KiB có bit Valid** ⇒ tổng byte **thường trú thuộc ảnh DLL** (`.text/.rdata/.data/.reloc` + copy-on-write) |
| **Process WorkingSet / Private** | `System.Diagnostics.Process` (đo song song để **giới hạn trên**) |
| **Tải gõ** | `SendInput` chuỗi Telex **tổng hợp cố định** (`tooi tooi ddii hocj.`) — có **L1 lặp từ** để Mode 1 (`highlight`) phải chạy; không đọc/nội dung tài liệu ra ngoài (S2) |
| **Mẫu** | 200 ms/mẫu; pha **idle** (không gõ) và pha **gõ** riêng; p95 = nearest-rank |
| **Định danh build** | SHA-256 + version DLL + `assist_mode` lấy từ `%APPDATA%\TextVN\config.json` |

**Giới hạn đã biết (không được bỏ qua khi đọc số):**
1. `dll_ws` **không** gồm heap/private do code trong DLL cấp phát (được tính vào heap tiến trình) ⇒ dùng `process_ws/private` làm **giới hạn trên**, nhưng phần lớn mức tăng của Notepad là **nội dung tài liệu + undo + render**, **không** quy cho DLL.
2. Đây là RSS **thường trú**, không phải "RSS ảo/danh nghĩa"; page bị trim sẽ giảm số.
3. Chưa có `p99` độ trễ phím trong phép đo này (F61 yêu cầu **RSS/binary/p95**; p95 độ trễ phím đo bằng bộ bench riêng — xem §5).

## 2. Môi trường đo

| | |
|---|---|
| Máy | Windows host (dev), Notepad **Store** `Microsoft.WindowsNotepad 11.2607.14.0` |
| Tiến trình | Notepad **mới** (`notepad.exe -m`), không dùng cửa sổ/phiên của người dùng |
| TIP | Đăng ký **per-user** trỏ DLL **build mới** (`textvn-cli register --scope user --dll …`), khôi phục đăng ký gốc sau khi đo |
| DLL | `textvn_win_tsf.dll` **0.2.34.0**, SHA-256 `6D21832E842BF3D9B5EFE1E7F3DE5315DB808B13550008085B66CD0C40FD6366` (build từ commit `765e2e7`, profile **release**) |
| Assist | `assist.mode = "highlight"`, `auto_fix = "off"`, `auto_capitalize = true` (config của máy — **không** sửa trong phép đo) |
| Thời lượng | idle 8 s (30 mẫu) + gõ 40 s (**145 mẫu** hợp lệ) |

## 3. Kết quả (2026-10-22)

### 3.1 RSS ảnh DLL (phần **quy được** cho DLL)

| Pha | mean | p95 | max |
|---|---|---|---|
| idle (không gõ) | **560 KiB** | 560 KiB | 560 KiB |
| gõ + assist highlight | **737,4 KiB** | **744 KiB** | 744 KiB |

⇒ Tải gõ + assist làm ảnh DLL tăng **~184 KiB** so với idle; tổng **~0,73 MiB**.
Ảnh DLL **mapped** (kể cả trang chưa thường trú): **1.264 KiB**.

### 3.2 Toàn tiến trình (giới hạn trên — **không** quy hết cho DLL)

| Chỉ số | idle | gõ (mean) | p95 | max |
|---|---|---|---|---|
| Process WorkingSet | 153,0 MB | 188,2 MB | 188,9 MB | 204,7 MB |
| Process Private | 99,5 MB | 126,3 MB | 127,1 MB | 127,2 MB |

Mức tăng ~35 MB WS / ~27 MB private là của **Notepad** (nội dung gõ nhiều vòng + undo + render); số `dll_ws` là phần **duy nhất** quy được cho TextVN.

## 4. So với budget (`P4-0-MASTER-PLAN.md` D25 · P4-0 §"Perf")

| Gate | Ngân sách | Đo được | Kết luận |
|---|---|---|---|
| RSS tăng do assist (in-proc, mode highlight, **chưa** nạp từ điển) | **≤ +20 MB** (P4-0 §Perf) · **≤ +5 MB** (S4a) | **+0,18 MB** (idle→gõ) · **0,73 MB** tổng ảnh | ✅ **Đạt** (dư rất nhiều) |
| Binary DLL | ~1,3 MB mapped | 1,3 MB | ✅ |
| `ime_key` < 0,5 ms p99 · adapter < 3 ms · end-to-end < 10 ms | — | **chưa đo** trong phép này | ⏳ (bench riêng) |

**Kết luận F61 (phần RSS in-proc):** ✅ có số thật trong TIP; **chưa** đóng hoàn toàn F61 vì (a) chưa đo **p99 độ trễ phím** trong TIP, (b) chưa đo **khi từ điển FST/OS provider được nạp** (Fassist-101).

## 5. Việc tiếp theo

1. **Đo p99 độ trễ** end-to-end trong TIP (không chỉ RSS): dùng ETW/`QueryPerformanceCounter` trong adapter hoặc bộ bench `core` + báo cáo riêng (đừng trộn với số RSS).
2. **Đo lại khi bật luật cần từ điển** (`spelling` / OS provider) ⇒ đóng **Fassist-101**.
3. Làm **kiểm chứng gõ** của script chắc hơn: hiện `typing_verified=false` do so khớp **một lần** quá sớm (900 ms sau vòng gõ đầu) — bằng chứng thay thế: tiêu đề cửa sổ Notepad hiện **"tôi tôi đi học…"** (TIP biên dịch Telex + L1 lặp từ ⇒ Mode 1 phải hiện). Sửa: so khớp **sau khi có ít nhất 1 chu kỳ hoàn chỉnh**, bỏ qua khác biệt hoa/thường đầu câu, và chỉ báo `typing-verified` khi `EditOf()` tìm được control.
4. Đo trên **máy sạch** (không có `TextVN.exe` tray chạy) để tách nhiễu.

## 6. Dọn dẹp sau đo (đã làm)

- Đóng **đúng** Notepad mới (pid đo) — **không** đụng Notepad/phiên khác của người dùng.
- `textvn-cli register --scope user --dll D:\AppAI\TextVN\target\release\textvn_win_tsf.dll` ⇒ **khôi phục** đăng ký dev ban đầu (đã kiểm `InprocServer32` = đường dẫn gốc).
- `%APPDATA%\TextVN\config.json` **không** bị sửa (hash trước/sau giữ nguyên).
- File scratch trong `%TEMP%\opencode\` (nội dung tổng hợp, không phải dữ liệu người dùng).
