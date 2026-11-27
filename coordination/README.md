# coordination/ — giao nhận việc & thông báo giữa các agent (G21)

> Bắt buộc cho MỌI agent (Claude, ZCode, Codex, WorkBuddy…) và người cùng làm repo này.
> Danh sách việc: `docs/10-shared/P0-7-TASKS-ARM-SF.md`. Luật chung: `docs/00-WORKFLOW.md` (G1–G21).

## 1. Một nguồn sự thật: `coordination/board.jsonl`

- Tệp **append-only**, mỗi dòng một sự kiện JSON phẳng: ai nhận task nào, giữ chỗ tệp nào, gửi
  thông báo gì, ai đang giữ khoá phát hành. **Không bao giờ sửa/xoá dòng cũ** — sửa sai = ghi sự kiện mới.
- Git theo dõi với `merge=union` (`.gitattributes`): agent chung checkout thấy **ngay**, agent máy
  khác thấy sau `git pull`; hai bên cùng nối dòng không xung đột.
- Chỉ ghi qua công cụ (giữ khoá tệp của OS trong lúc đọc–kiểm–ghi ⇒ hai agent không thể cùng nhận
  một task):

```
cargo xtask board help
```

## 2. Định danh agent

`TEXTVN_AGENT=<công cụ>-<model>-<8 ký tự phiên>` — ví dụ `claude-opus-c4c5a567`,
`zcode-glm-1a2b3c4d`, `codex-gpt-5e6f7a8b`; người: `human-<github>`. Đặt biến môi trường một lần
mỗi phiên (hoặc truyền `--agent`). Không dùng chung định danh giữa hai phiên.

## 3. Quy tắc giao nhận việc (R1–R14)

| # | Khi | Bắt buộc làm |
|---|---|---|
| R1 | Bắt đầu phiên | `git pull --ff-only` → `cargo xtask board status --agent $TEXTVN_AGENT` → `board inbox` → xử lý thư → `board ack` |
| R2 | Nhận task | task phải đạt **DoR** (P0-7 §2) và mọi `Dep` đã `done`. `board claim <TASK> --msg "<kế hoạch 1 dòng>"` → commit RIÊNG `coordination/board.jsonl` (`board: claim <TASK> (<agent>)`) → push ngay. Tối đa **2 task** đang giữ mỗi agent |
| R3 | Trước khi sửa tệp | `board reserve <tệp/thư mục/> --task <TASK>` cho mọi tệp sẽ sửa. Tệp "nóng" chỉ **người giữ khoá phát hành** sửa: `CHANGELOG.md`, `README.md`, `docs/release/build-release-report.md`, `Cargo.toml` (version), `Cargo.lock`, `debian/changelog`, `packaging/**` — agent task viết **mảnh ghi chú** `coordination/changes/<TASK>.md` (VI + EN) thay vào đó |
| R4 | Đang làm | `board heartbeat <TASK>` ít nhất mỗi 4 giờ (lease task 8 h, giữ chỗ 4 h). Hết lease = người khác được nhận |
| R5 | Trước MỖI commit | `cargo xtask board guard $(git diff --cached --name-only)` phải OK; chỉ `git add` danh sách tệp tường minh (G2); không commit tệp agent khác đang giữ chỗ |
| R6 | Giao lại | `board handoff <TASK> --to <agent> --msg "<nhánh/commit, đã xong gì, bước kế, rủi ro>"` + ghi cùng nội dung vào mục *Ghi chú* của task trong P0-7 |
| R7 | Xong | đủ **DoD** (P0-7 §3) + **audit** (P0-7 §4) → `board release <TASK> --status done --commit <sha>` → `board unreserve` → `board notify --to '*' --msg "<TASK> xong: <tóm tắt>"` → `board review-request --to <agent review> --task <TASK> --msg "<điểm cần soát>"` |
| R8 | Bị chặn | `board release <TASK> --status blocked --msg "<chặn bởi gì, cần gì>"` + `board notify --to <chủ task chặn>` |
| R9 | Phát hành (G16/G20) | chỉ agent giữ `board rm-lock --msg vX.Y.Z` mới bump version, gộp `coordination/changes/*.md` vào CHANGELOG/README/report, tag, chạy B7–B9; xong `board rm-unlock` + notify `*`. Agent khác **không** tag/push tag |
| R10 | Lease hết hạn | được `claim` lại nhưng PHẢI `notify --to <chủ cũ>` và đọc ghi chú handoff/commit cuối của họ trước |
| R11 | Cấp mã lỗi/finding mới (E/R/B/R2-…) | trước khi dùng: `board note --msg "alloc E83 <tiêu đề>"`; `board log` để xem mã đã cấp — tránh hai agent cùng dùng một số |
| R12 | Đổi hợp đồng dùng chung (C ABI `ffi/include/textvn_ffi.h`, `textvn-session` API, schema config, định dạng corpus/vết) | `board notify --to '*'` TRƯỚC khi push, kèm mô tả tương thích; chỉ một agent giữ chỗ tệp hợp đồng tại một thời điểm |
| R13 | CI `main` đỏ (G13) | ai thấy trước: `board notify --to '*' --msg "CI đỏ <workflow> <run> — <agent> xử lý"` rồi claim việc sửa; không ai push thêm thay đổi không liên quan tới khi xanh |
| R14 | Nội dung thông báo | KHÔNG ghi chữ người dùng gõ, mật khẩu, token, đường dẫn có tên người dùng (G8) — repo public |

## 4. Mảnh ghi chú phát hành `coordination/changes/<TASK>.md`

Mỗi task đổi hành vi/tính năng viết một mảnh (người phát hành gộp — tránh xung đột CHANGELOG):

```markdown
<!-- TASK: SF-2 · commit: abc1234 · agent: claude-opus-c4c5a567 -->
## VI
- **SF-2 — …**: …
## EN
- **SF-2 — …**: …
## Tài liệu / Docs
- …
## Kiểm thử / Testing
- …
```

## 5. Ví dụ một vòng

```bash
export TEXTVN_AGENT=claude-opus-c4c5a567
git pull --ff-only && cargo xtask board status && cargo xtask board inbox && cargo xtask board ack
cargo xtask board claim SF-1a --msg "khung crate session + C ABI"
git add coordination/board.jsonl && git commit -m "board: claim SF-1a (claude-opus-c4c5a567)" && git push
cargo xtask board reserve session/ ffi/src/lib.rs ffi/include/textvn_ffi.h --task SF-1a
# … làm việc, heartbeat mỗi ≤ 4 h …
cargo xtask board guard $(git diff --cached --name-only)
git add <danh sách tệp> && git commit -m "feat(session): SF-1a …"
cargo xtask board release SF-1a --status done --commit "$(git rev-parse --short HEAD)"
cargo xtask board unreserve
cargo xtask board notify --to '*' --msg "SF-1a xong: crate session + ime_session_* (header đổi — R12)"
git add coordination/board.jsonl coordination/changes/SF-1a.md && git commit -m "board: release SF-1a" && git push
```
