# AGENT HANDBOOK — Quy ước làm việc cho mọi agent/contributor

> Áp dụng cho **mọi** phần (0/1/2/3) và mọi người/agent tham gia.
> Tài liệu này là hợp đồng: đọc xong bạn phải biết *đặt file ở đâu, viết commit thế nào,
> thế nào là xong, thế nào là bị reject*.

---

## 1. Nguyên tắc bất di bất dịch (vi phạm = reject PR)

| # | Quy tắc | Lý do |
|---|---|---|
| **S1** | `vietime-core` **không có** network, file I/O, thread spawn, logging nội dung text | Engine chạy trong-process của app người dùng; auditable |
| **S2** | **Không bao giờ** ghi nội dung phím/từ ra log/telemetry/crash dump | IME reputation = an toàn (bài học EVKey) |
| **S3** | **Không bao giờ** xử lý chuỗi khi `context.secure == 1` (ô mật khẩu) | Tránh đọc mật khẩu |
| **S4** | Core **fail-open**: mọi lỗi/panic → trả `PASS` (phím đi thẳng), không bao giờ chặn phím | Crash IME = mất bàn phím người dùng |
| **S5** | Không thêm dependency network (http/ws/mqtt) vào crate trong `core/`, `strategy/`, `adapters/*` | Gate CI `cargo-deny` |
| **S6** | Mọi file có SPDX header (REUSE) <!-- REUSE-IgnoreStart -->: `// SPDX-License-Identifier: GPL-3.0-or-later`<!-- REUSE-IgnoreEnd --> | Tuân thủ license |
| **S7** | Không copy code từ EVKey/WinVNKey (đóng/ambiguous). Ý tưởng → ghi spec clean-room vào `docs/specs/` → viết mới | Tránh scandal license |
| **S8** | FFI: **không malloc chéo ranh giới**; struct POD, caller-allocated | Tránh double-free giữa Rust/C++/Swift |
| **S9** | Không đăng ký hotkey/hook global trừ khi adapter có justification trong ADR | Xung đột phím tắt hệ thống (bug B6) |

---

## 2. Cấu trúc thư mục & nơi đặt tài liệu

```
vietime/
├── PLAN.md                     # plan tổng (ngoài repo docs/)
├── docs/
│   ├── 00-INDEX.md             # chỉ mục + giao thức review
│   ├── 01-AGENT-HANDBOOK.md    # file này
│   ├── 10-shared/              # Phần 0 — hợp đồng chung
│   ├── 20-windows/             # Phần 1
│   ├── 30-macos/               # Phần 2
│   ├── 40-linux/               # Phần 3
│   ├── adr/                    # ADR-001, 002, ... (mỗi quyết định 1 file)
│   ├── specs/                  # spec clean-room (behavior lấy từ dự án khác, viết mới)
│   └── compat.md               # app compatibility matrix (public)
├── core/                       # Rust: vietime-core, vietime-ffi, vietime-strategy, ...
├── adapters/
│   ├── windows-tsf/            # vietime-win-tsf (cdylib)
│   ├── windows-hook/           # vietime-win-hook
│   ├── macos-imk/              # (Phần 2)
│   └── linux-ibus/, linux-fcitx5/  # (Phần 3)
├── tray/                       # vietime-tray (egui) — Win đầu, sau tái dùng ý tưởng cho mac/linux
├── cli/                        # vietime.exe (doctor / replay / verify)
├── data/                       # appdb.default.json, tables/*.toml, spelling/, emoji.tsv
├── schemas/                    # config.v1.schema.json, appdb.v1.schema.json, ffi.v1.md, ipc.v1.md
├── corpus/                     # golden test: *.keys → *.expect
├── tests/                      # integration + conformance
├── fuzz/                       # cargo-fuzz targets
└── .github/workflows/          # ci-windows.yml, ci-macos.yml, ci-linux.yml, release.yml
```

**Đặt tài liệu:** mọi doc mới nằm trong đúng thư mục phần; đặt tên file theo mẫu `P{n}-{mã}-{chủ-đề}.md`
(ví dụ: `P0-2-engine-ffi-contract.md`, `P1-1-tsf.md`). Các file đánh số `P{n}-0-...` là tài liệu tổng
của phần đó.
Mọi quyết định kiến trúc → **ADR** (template trong §7), không viết trôi nổi trong README.

---

## 3. Git & commit

- Branch: `feat/WIN-014-tsf-composition`, `fix/F1-003-preedit-underline`, `docs/P0-ffi`.
- Commit message (Conventional Commits): `feat(windows-tsf): end composition on focus loss (#WIN-014)`.
- **PR tối thiểu 1 file test hoặc 1 file corpus** (trừ PR docs thuần).
- DCO: `Signed-off-by:` bắt buộc. PR chạm `core/`, `schemas/`, `adapters/` cần **2 approve**.
- Không force-push nhánh đã có PR; không commit binary (release do CI tạo).

## 4. Definition of Done (DoD) cho một TASK

Một task (WIN-xxx / MAC-xxx / LNX-xxx / P0-xxx) được đóng khi **đủ 7 mục**:

1. Code + unit test xanh trên máy bạn (`cargo test --workspace`).
2. Golden corpus thêm/truyền case tương ứng (nếu đổi hành vi gõ) → `vietime replay corpus/` pass.
3. `cargo clippy --all-targets -- -D warnings` và `cargo fmt --check` pass.
4. `cargo deny check` + REUSE lint pass.
5. Doc liên quan cập nhật (spec/ADR/compat.md) **hoặc** ghi rõ "không cần" trong PR.
6. Acceptance criteria của task trong `P{n}-TASKS.md` được tick (kèm bằng chứng: output CLI, screenshot, log).
7. CI 3 OS xanh (hoặc OS liên quan xanh + 2 OS còn lại không đổi).

## 5. Checklist Review (dùng cho cả 2 lượt — chi tiết hóa theo lượt)

**Review 1 — Đúng & Đủ**
- [ ] Mọi API/COM interface/đường dẫn file nhắc tới có **thật** và đúng phiên bản?
- [ ] Có đoạn nào mô tả thứ không tồn tại trong repo chưa (crate/namespace/chỉ mục sai)?
- [ ] Task nào thiếu acceptance criteria? Thiếu dependency/deadline?
- [ ] Có vi phạm S1–S9 không?
- [ ] Failure mode đã nêu? (crash, permission denied, OS update đổi API)

**Review 2 — Nhất quán & Sẵn sàng**
- [ ] `grep` toàn bộ tên struct (`ime_result_v1`…), crate (`vietime-*`), GUID, pipe, path → khớp 100% giữa các file?
- [ ] Không mâu thuẫn giữa PLAN.md ↔ P0 ↔ P1?
- [ ] Thứ tự bước thực hiện có tự chặn (agent làm theo từng bước không bị chờ task chưa có)?
- [ ] Corpus/test command chạy được ngay (không thiếu flag)?
- [ ] Trạng thái index (`00-INDEX.md`) đúng sau review.

## 6. Security rules khi dev & test

- Không chạy test với app thật đang nhập mật khẩu; test secure-field dùng placeholder.
- Log level `debug` **không** được in tham số key/char. Validator CI: grep `log::.*key|log::.*char` trong `core/`.
- Preset/appdb update: chỉ nhận file có chữ ký Ed25519 khớp key trong `data/preset.pub` (xem P0-3).
- Fuzz tối thiểu 60s trên PR chạm parser (config/appdb/FFI), 10 phút nightly.
- Báo lỗ hổng: `SECURITY.md` (90 ngày responsible disclosure), không public issue.

## 7. Template ADR

```markdown
# ADR-0XX: Tiêu đề
- Status: Proposed | Accepted | Deprecated | Superseded by ADR-0YY
- Date: YYYY-MM-DD
- Context: (vấn đề, ràng buộc, nguồn tham chiếu)
- Decision: (quyết định cụ thể, có thể test được)
- Alternatives considered: (a) ... tại sao loại
- Consequences: (điều gì thay đổi, rủi ro, việc phải làm tiếp)
- Revisit trigger: (khi nào mở lại quyết định này — ví dụ "macOS spike xong")
```

**Danh sách ADR bắt buộc có** (viết trong phần tương ứng): 001 core Rust+C-ABI · 002 strategy engine ·
003 license GPL-3.0-or-later + clean-room · 004 UI egui · 005 Windows TSF-primary + hook-fallback ·
006 macOS IMK-primary · 007 Linux IBus+Fcitx5 · 008 updater & signing · 009 config schema v1 ·
010 test pyramid + app-compat automation.

## 8. Thuật ngữ (giữ đúng, không dịch lung tung)

| Term | Nghĩa |
|---|---|
| **Engine/ core** | `vietime-core` — biến đổi phím → chữ, không biết OS |
| **Adapter** | phần OS-specific nhận sự kiện phím & đẩy chữ ra app |
| **Strategy** | cách đẩy chữ ra app: `Preedit`, `BackspaceType`, `SelectionReplace`, `ForwardAsCommit`, `Passthrough` |
| **Preset / appdb** | dữ liệu cấu hình theo từng ứng dụng (chứa strategy) |
| **Composition** | chuỗi đang gõ dở (TSF `ITfComposition`) |
| **Preedit** | text hiển thị gạch chân trước khi commit |
| **Commit** | đẩy preedit vào document vĩnh viễn |
| **Pass-through** | để phím đi thẳng cho app (không biến đổi) |
| **Fail-open** | khi lỗi → phím đi thẳng, không chặn |

## 9. Quy trình khi phát hiện bug kinh niên (B1–B13)

1. Ghi issue mẫu: OS + app + version + **chuỗi phím gõ lại được** (user tự dán) + ảnh/video.
2. Xác định nó thuộc `B{n}` trong `PLAN.md §1.3` hay mới → nếu mới ghi `B14+`.
3. Viết **reproduction thành corpus** (`corpus/win/bug_B1_xxx.keys`) TRƯỚC khi fix.
4. Fix → corpus pass trên 3 CI OS (core) + app-compat test nếu liên quan app cụ thể.
