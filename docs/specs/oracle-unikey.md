# Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)

> Mục đích: corpus `corpus/shared/*.keys` cần một **kết quả mong đợi khách quan**.
> UniKey/x-unikey (GPL) là engine được dùng hàng 20 năm → dùng nó làm oracle.
> **Chỉ dùng để sinh dữ liệu kỳ vọng, KHÔNG copy code engine vào repo.**

## 1. Cách lấy oracle

### Cách A (khuyến nghị): x-unikey CLI (Linux, có thể chạy trong CI container)

```bash
# 1. Tải source (link chính thức: https://www.unikey.org/linux.html#download-x-unikey)
curl -LO http://prdownloads.sourceforge.net/unikey/x-unikey-1.0.4.tar.bz2
tar xf x-unikey-1.0.4.tar.bz2
cd x-unikey-1.0.4
# 2. Build phần engine/CLI (không cần X11 nếu chỉ dùng lib transform)
make -C src ukconv   # tên target thay đổi theo bản — nếu không có, build target CLI tương ứng
# 3. Sinh kỳ vọng cho một case
printf 'duocj' | ./ukconv -m telex -u > /tmp/expect.txt    # flag thay đổi theo binary thật
```

> **Lưu ý triển khai:** task **WIN-007** (`docs/20-windows/P1-6-TASKS.md`) sẽ ghi lại **đúng tên binary +
> flag** sau khi tự build thử, cập nhật mục này. Nếu không build được headless, dùng Cách B.

### Cách B: UniKey 3.62 GUI trên Wine/VM (manual, chỉ cho case hiếm)

1. Download `Uk362src.zip`/bản binary từ trang source của UniKey.
2. Chạy trên VM Windows sạch → chọn Telex/VNI, Unicode.
3. Gõ chuỗi test, copy kết quả vào corpus kèm `:note "oracle=unikey362 manual YYYY-MM-DD"`.

### Cách C (dự phòng): bảng Unicode chuẩn + quy tắc chính tả tiếng Việt
Chỉ dùng khi A/B không khả thi cho case cụ thể → ghi `:note "oracle=spec <tên quy tắc>"`.

## 2. Quy tắc chung

1. **Mỗi case corpus phải ghi nguồn kỳ vọng** trong comment đầu file:
   `# oracle: x-unikey-1.0.4 telex | unikey362 manual | spec quy-tắc-đặt-dấu-2024`
2. Nếu VietIME khác oracle → **phải** có ADR/spec giải thích (ví dụ: kiểu dấu mới `hoà` là chủ đích,
   UniKey mặc định kiểu cũ) — không được "im lặng sửa expect".
3. Không commit binary UniKey vào repo. Script tải+build để trong `tools/oracle/` (dùng URL chính thức,
   verify checksum ghi trong script).
4. License: oracle là GPL — dữ liệu kỳ vọng (input→output) là thông tin, không phải copy code; vẫn ghi
   nguồn để audit (REUSE không áp cho corpus, có header comment riêng).

## 3. Coverage yêu cầu (DoD của oracle)

- [ ] ≥ 200 case `corpus/shared/` có nguồn oracle ghi rõ (mục 2.1).
- [ ] Telex, VNI, VIQR đều có bộ case oracle.
- [ ] Các trường hợp mark reposition (`hoaf`), undo (`ass`), w-vowel (`nhw`), đ (`dd`) đều có.
- [ ] Bảng "khác biệt chủ đích" được duy trì tại `docs/specs/deliberate-differences.md`.
