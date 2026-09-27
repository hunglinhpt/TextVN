# Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)

> Mục đích: corpus `corpus/shared/*.keys` và `corpus/win/*.keys` cần một **kết quả mong đợi khách quan**.
> UniKey/x-unikey (GPL) là engine được dùng hàng 20 năm → dùng nó làm oracle.
> **Chỉ dùng để sinh dữ liệu kỳ vọng, KHÔNG copy code engine vào repo.**

---

## 1. Kết quả kiểm tra môi trường Windows (WIN-007)

Theo task `WIN-007` (`docs/20-windows/P1-6-TASKS.md`), bảng thông tin chi tiết về các bản build UniKey trên Windows:

| Phiên bản | Tên tập tin thực thi (Binary) | GUI / Headless | Lưu ý & Cách trích xuất kết quả |
|---|---|---|---|
| **UniKey 4.3 RC5 (Official Windows)** | `UniKeyNT.exe` (x64 / x86) | Chỉ có GUI Win32 hook | Không hỗ trợ CLI headless trực tiếp qua stdin/stdout. Khi chạy, icon xuất hiện ở khay hệ thống (System Tray). Cấu hình lưu tại registry `HKCU\Software\UniKey`. Kết quả test được trích xuất qua kịch bản tương tác tự động hóa UI hoặc lấy mẫu thủ công trên VM chuẩn. |
| **UniKey 3.62 (Classic GPL Reference)** | `UniKey362.exe` / `uk362src` | Win32 GUI / Core C | Cung cấp mã nguồn tham chiếu giải thuật Telex/VNI kinh điển và xử lý phím lặp, free-marking. |
| **x-unikey 1.0.4 (Linux engine wrapper)** | `ukconv` / `x-unikey` | Headless CLI / X11 | Bản port GPL có công cụ dòng lệnh `ukconv` hỗ trợ biến đổi bảng mã và phân tích engine qua pipe. |

### Xác nhận Oracle Case trên Windows (Task Acceptance)
- **Chuỗi gõ vào:** `d u o c j` (phương pháp gõ Telex)
- **Cấu hình:** Bảng mã Unicode dựng sẵn (Unicode Precomposed), kiểu gõ Telex, bật kiểm tra chính tả tự do.
- **Kết quả thực tế từ UniKey trên Windows:** `được`
- **Ghi nhận nguồn:** `corpus/win/oracle_unikey_reference_01.keys` (kèm comment chứng minh nguồn gốc `:note "oracle=unikey-windows-4.3-rc5"`).

---

## 2. Cách lấy oracle

### Cách A (khuyến nghị cho CI/Linux): x-unikey CLI
```bash
# 1. Tải source (link chính thức: https://www.unikey.org/linux.html#download-x-unikey)
curl -LO http://prdownloads.sourceforge.net/unikey/x-unikey-1.0.4.tar.bz2
tar xf x-unikey-1.0.4.tar.bz2
cd x-unikey-1.0.4
# 2. Build phần engine/CLI (không cần X11 nếu chỉ dùng lib transform)
make -C src ukconv
# 3. Sinh kỳ vọng cho một case
printf 'duocj' | ./ukconv -m telex -u > /tmp/expect.txt
```

### Cách B: UniKey Windows trên VM/Desktop (Áp dụng cho WIN-007)
1. Chạy `UniKeyNT.exe` trên môi trường Windows sạch (Unicode, kiểu gõ Telex).
2. Gõ chuỗi ký tự kiểm thử vào ứng dụng mục tiêu (Notepad/Word).
3. Ghi lại kết quả và chú thích rõ ràng trong corpus:
   ```text
   # oracle: UniKeyNT.exe 4.3 RC5 (Windows 11 x64, 2026-09-27)
   :note "oracle=unikey-windows-4.3-rc5"
   ```

### Cách C (dự phòng): Bảng Unicode chuẩn + quy tắc chính tả tiếng Việt
Chỉ dùng khi A/B không khả thi cho case cụ thể → ghi `:note "oracle=spec <tên quy tắc>"`.

---

## 3. Quy tắc chung

1. **Mỗi case corpus phải ghi nguồn kỳ vọng** trong comment đầu file:
   `# oracle: unikey-windows-4.3-rc5 | x-unikey-1.0.4 telex | unikey362 manual | spec quy-tắc-đặt-dấu-2024`
2. Nếu TextVN khác oracle → **phải** có ADR/spec giải thích (ví dụ: kiểu dấu mới `hoà` là chủ đích,
   UniKey mặc định kiểu cũ) — không được "im lặng sửa expect".
3. Không commit binary UniKey vào repo. Script tải+build để trong `tools/oracle/` (dùng URL chính thức,
   verify checksum ghi trong script).
4. License: oracle là GPL — dữ liệu kỳ vọng (input→output) là thông tin, không phải copy code; vẫn ghi
   nguồn để audit (REUSE không áp cho corpus, có header comment riêng).

---

## 4. Coverage yêu cầu & Tiến độ

- [x] Xác nhận tên binary `UniKeyNT.exe` và cơ chế tương tác trên Windows (WIN-007).
- [x] Tạo case corpus Windows mẫu xác thực với UniKey oracle (`corpus/win/oracle_unikey_reference_01.keys`).
- [ ] Mở rộng ≥ 200 case `corpus/shared/` có nguồn oracle ghi rõ.
- [ ] Duy trì bảng deliberate differences tại `docs/specs/deliberate-differences.md`.

