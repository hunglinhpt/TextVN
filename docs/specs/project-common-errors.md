<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# Project Common Errors — Sổ lỗi chung toàn dự án

> **Mục đích (G3 — `../00-WORKFLOW.md`):** Ghi nhận các lỗi lập trình, công cụ, cú pháp, CI, git đã gặp và fix thành công.
> ID dạng `E{n}` liền mạch, không xóa entry.

## 1. Danh sách lỗi (E-IDs)

| ID | Hiện tượng (dán nguyên thông báo lỗi) | Nguyên nhân | Fix |
|---|---|---|---|
| E1 | `UnicodeEncodeError: 'charmap' codec can't encode character...` khi chạy `reuse lint` trên Windows console | Môi trường Windows cmd/powershell mặc định dùng code page ANSI (cp1252), Python stdout ném ngoại lệ khi in ký tự tiếng Việt | Set biến môi trường `$env:PYTHONUTF8 = "1"` và `$env:PYTHONIOENCODING = "utf-8"` trước khi gọi `reuse lint` qua script `.ps1` |
| E2 | `reuse lint` báo lỗi `Missing licenses: ...` hoặc `Invalid SPDX Expression` do nhầm lẫn scanner | Viết literal chuỗi định dạng SPDX header trong phần doc-comment hoặc prose markdown khiến scanner của REUSE quét nhầm thành header thật bị hỏng | Diễn đạt gián tiếp, không viết nguyên văn chuỗi nhận dạng SPDX trong comment; hoặc bọc `<!-- REUSE-IgnoreStart -->` |
| E3 | `error: unexpected closing delimiter: '}'` ở `adapters\windows-hook\src\lib.rs:118:1` | Dư dấu ngoặc nhọn `}` ở hàm `empty_result()` (dòng 117) | Xóa dấu `}` thừa, chạy `cargo fmt` để chuẩn hoá thụt lề |
| E4 | `error[E0432]: unresolved import 'windows::Win32::Foundation::BOOL'` | Trong crate `windows 0.61`, `BOOL` được đặt trong `windows::core::BOOL` (re-export qua `windows_core`), không nằm ở `Win32::Foundation` | Sử dụng `windows::core::BOOL` hoặc `use windows::core::*;` |
| E5 | `error[E0599]: no method named 'query' found for struct 'Com::IClassFactory' in the current scope` | Trait method `query` của COM interface thuộc trait `windows::core::Interface`, cần đưa trait vào scope | Thêm `use windows::core::Interface;` hoặc `use windows::core::*;` |
| E6 | `error[E0308]: mismatched types: expected Result<&ITfContext, Error>, found Option<_>` | Helper method `.ok()` trên wrapper `Ref<'_, T>` của `windows 0.61` trả về kiểu `Result<&T, windows_core::Error>`, không phải `Option` | Pattern match `Ok(c)` / `Err(e)` thay vì `Some(c)` / `None`, hoặc dùng toán tử `?` |
