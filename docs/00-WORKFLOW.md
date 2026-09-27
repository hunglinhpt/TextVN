# VietIME — Workflow & Ground Rules thống nhất cho mọi agent

> **Phạm vi:** mọi agent (người hoặc AI) làm việc trong repo này — đây là *quy trình vận hành hằng ngày*.
> **Nguồn gốc:** yêu cầu thống nhất workflow (user, 2026-09-27).
> **Quan hệ:** BỔ SUNG `01-AGENT-HANDBOOK.md`. Handbook là gốc về DoD / security / review;
> file này là gốc về **thứ tự thao tác hằng ngày**. Nếu mâu thuẫn → ghi `FW-{n}` vào §7 rồi sửa cả hai
> (không tự xử lý lén, không sửa file đang có WIP của agent khác).
> **Trạng thái:** v1 · Review 1 (*Đúng & Đủ*) = tác giả tự rà 2026-09-27 ·
> Review 2 (*Nhất quán & Sẵn sàng*) **chờ agent thứ hai** — finding `FW-{n}`, không xóa.

## 1. Ground rules (G-IDs) — áp dụng mọi lúc

| ID | Rule | Bằng chứng / lý do |
|---|---|---|
| G1 | **Đọc đủ §2 trước khi chạm task** — mỗi phiên, kể cả agent quen việc (đọc phần delta) | tránh làm lại hoặc đi ngược design đã chốt |
| G2 | **Không `git add -A`** — chỉ `git add` list file tường minh; không commit WIP của agent khác | repo đa agent, WIP treo thường xuyên |
| G3 | **Lỗi → fix xong → ghi NGAY vào common-errors** (§3): hiện tượng = dán nguyên thông báo lỗi, nguyên nhân, fix; ID liền mạch, **không xóa entry** | `win-test-common-errors.md` A1–A10: nhiều lỗi đã lặp ≥2 lần vì trước đây không ghi |
| G4 | **Thao tác/tool chạy thành công → ghi NGAY vào `specs/verified-ops.md`** (§4) | để agent sau **không kiểm tra lại** mỗi phiên |
| G5 | **Xong việc → `git add <tường minh>` → verify → commit → push ngay → `graphify update .` → commit `graphify-out/` → push** | user rule 2026-09-27; message `loai(scope): nd` với `loai ∈ {spike, docs, fix, test, chore}` |
| G6 | **ID thống nhất:** review phần = `F{phần}-{n}` · spike = `S{k}-{n}` (S3 = TSF/registration, S4 = UIA) · common-error = `A{n}` (Windows) / `E{n}` (project-wide) · workflow này = `FW-{n}` | tránh đụng nhau khi grep |
| G7 | **PowerShell:** luôn viết file `.ps1` ASCII-only rồi `powershell -NoProfile -ExecutionPolicy Bypass -File <x.ps1>`; không `&&` (dùng `;`); không truyền `$var`/`$_` inline (shell nuốt); static member phải `[Type]::X` hoặc `$v::X` — **không** `$v.X` | lỗi A1–A10 trong common-errors |
| G8 | **S2 — không bao giờ log nội dung phím / text người dùng** (chỉ vk, HRESULT, đếm, tên element) | Handbook §security, handlog S2 |
| G9 | **Task Done = đủ DoD 7 mục** (`01-AGENT-HANDBOOK.md §4`) kèm evidence cho từng acceptance | … |
| G10 | **Tài liệu phần phải đạt 2/2 review** rồi mới đánh dấu ✅ tại `00-INDEX §1` | giao thức `00-INDEX §3` |
| G11 | **File đang có WIP chưa commit của agent khác → KHÔNG sửa, KHÔNG add** — cần thật sự thì báo user / đợi họ commit rồi làm | 00-INDEX, HANDOOK, P1-1, P1-5… đang treo WIP (2026-09-27) |
| G12 | **Trước khi commit: `git diff --cached --name-only`** — đối chiếu đúng list mong muốn, không có file lạ | sai lúc add nhầm WIP là mất công ngược |

## 2. Bắt buộc đọc — trình tự mỗi phiên

| # | File | Để làm gì |
|---|---|---|
| 0 | `docs/00-WORKFLOW.md` (file này) | ground rules + quy trình |
| 1 | `../PLAN.md` | mục tiêu tổng, 13 bug B1–B13, nguồn kế thừa |
| 2 | `01-AGENT-HANDBOOK.md` | DoD §4, security rules, quy ước chung |
| 3 | `specs/verified-ops.md` | cái gì **đã kiểm chứng** — tra trước khi cài/verify môi trường |
| 4 | `specs/win-test-common-errors.md` (Windows) · `specs/project-common-errors.md` (nếu có) | lỗi đã gặp — tránh lặp |
| 5 | `00-INDEX.md` §2 → tài liệu phần (`10-shared/`, `20-windows/` …) | design đã chốt, ID task |
| 6 | `20-windows/P1-6-TASKS.md` (hoặc `30-macos/`, `40-linux/`) | nhận task + acceptance criteria |

- **Agent mới (lần đầu trong repo):** đọc đủ 0 → 6 = onboarding bắt buộc.
- **Agent quen việc:** mỗi phiên đọc **0, 3, 4 (domain sắp làm), 6**; đọc lại 5 khi có thay đổi design.

## 3. Common errors — ghi ở đâu, ghi như thế nào

| Domain | File | ID |
|---|---|---|
| Windows test/spike: PowerShell, UIA, TSF, SendInput, registry… | `specs/win-test-common-errors.md` | `A{n}` |
| Project-wide: git, cargo, CI, graphify, doc… | `specs/project-common-errors.md` — **tạo file ngay khi gặp lỗi đầu tiên** | `E{n}` |

Format mỗi entry: `| ID | Hiện tượng (dán nguyên thông báo lỗi) | Nguyên nhân | Fix |`

- Ghi **ngay lúc vừa fix xong** — không để tích cuối ngày.
- Không xóa entry; nếu lỗi hết áp dụng thì thêm cột trạng thái, giữ nguyên dòng.

## 4. Verified ops — sổ thao tác đã kiểm chứng

→ **`specs/verified-ops.md`** (quy tắc G4).

Trước MỌI việc mới (cài tool, dựng app test, verify môi trường, push/CI…): **tra sổ trước**.
Thao tác chạy thành công lần đầu → **append dòng mới ngay** kèm ngày + link chi tiết.
Entry gồm: thao tác cụ thể (có lệnh/đường dẫn), kết quả (ngày), chi tiết.

## 5. Workflow 7 bước — mọi task

```text
[1] Đọc §2 (phần delta)     → ghi rõ task ID sẽ nhận
[2] Nhận task                 → acceptance criteria + spec liên quan
[3] Làm việc                  → lỗi mới: G3 · thao tác thành công mới: G4
[4] Self-review               → tick từng dòng acceptance; evidence = log / đường dẫn / số
[5] Stage + verify            → git add <list tường minh> → git diff --cached --name-only (G12)
[6] Commit + push ngay        → message `loai(scope): nd`
[7] graphify update .         → commit graphify-out/ → push → ghi status (INDEX/review log nếu thuộc phần)
```

## 6. Vùng sở hữu file (snapshot 2026-09-27 — cập nhật §7 khi đổi)

| Agent | Vùng |
|---|---|
| Agent Windows / spike | `adapters/windows-*`, `spikes/`, `tools/win/`, `docs/specs/*spike*.md`, `docs/specs/win-test-common-errors.md`, `docs/specs/verified-ops.md` |
| Agent core/FFI | `core/`, `ffi/`, `cli/`, `config/`, `strategy/`, `corpus/`, `appdb/`, `.github/` |
| Chung (vẫn áp G11 khi có WIP) | `PLAN.md`, `docs/00-*`, `docs/01-*`, `docs/20-windows/*` (ngoài spike specs), `docs/specs/*` còn lại |

## 7. Review & change log

| Ngày | Việc | Finding |
|---|---|---|
| 2026-09-27 | Tạo v1 (workflow + ground rules + sổ verified-ops) theo yêu cầu user | — |
| — | Review 2 chờ agent thứ hai | `FW-{n}` |
