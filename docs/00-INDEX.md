# VietIME — Chỉ mục tài liệu & Giao thức Review

> **Mục tiêu dự án:** bộ gõ tiếng Việt mã nguồn mở, chạy Windows → macOS → Linux,
> kế thừa tinh hoa & fix bug của UniKey/x-unikey · EVKey · GoTiengViet · WinVNKey · Gõ Nhanh · Bamboo Viet.
> Plan tổng: `../PLAN.md` (đọc trước khi bắt tay vào việc).

## 1. Trạng thái các phần

| Phần | Tài liệu | Trạng thái | Review |
|---|---|---|---|
| **0 — Nền tảng chung** (repo, FFI, schema, strategy, test) | `10-shared/P0-*.md` | ✅ Reviewed | 2/2 → `10-shared/P0-REVIEW-LOG.md` |
| **1 — Windows** | `20-windows/P1-*.md` | ✅ Reviewed | 2/2 → `20-windows/P1-REVIEW-LOG.md` |
| **2 — macOS** | `30-macos/P2-*.md` | ✅ Reviewed | 2/2 → `30-macos/P2-REVIEW-LOG.md` |
| **3 — Linux** | `40-linux/P3-*.md` | ✅ Reviewed | 2/2 → `40-linux/P3-REVIEW-LOG.md` |

> Quy tắc: **không bắt đầu phần N+1 khi phần N chưa đạt 2/2 review.**

## 2. Thứ tự đọc cho agent mới (bắt buộc)

```
1. ../PLAN.md                     # tổng quan, nguồn kế thừa, 13 bug kinh niên (B1–B13)
2. 01-AGENT-HANDBOOK.md           # quy ước, Definition of Done, checklist review, security rules
3. 10-shared/P0-1-repo-and-workflow.md     # repo layout, crate names, lệnh build/test, CI
4. 10-shared/P0-2-engine-ffi-contract.md   # HỢP ĐỒNG FFI — nguồn sự thật duy nhất
5. 10-shared/P0-3-config-preset-strategy.md# schema config/appdb + chiến lược xuất chữ + IPC
6. 10-shared/P0-4-test-and-corpus.md       # định dạng corpus `.keys`, 7 lớp test, lệnh replay
7. 20-windows/P1-*.md             # nếu bạn làm Windows
   (30-macos/, 40-linux/ — tương tự khi tới phần đó)
8. 20-windows/P1-6-TASKS.md       # nhận task theo ID (WIN-xxx)
```

## 3. Giao thức Review (bắt buộc cho mọi phần)

Mỗi phần phải qua **tối thiểu 2 lượt review**, mỗi lượt có checklist riêng,
kết quả ghi vào `P{n}-REVIEW-LOG.md` của phần đó:

| Lượt | Tên | Trọng tâm | Ai làm |
|---|---|---|---|
| **Review 1** | *Đúng & Đủ* | Kiểm tra **chính xác kỹ thuật** (API/đường đi có thật không?), **đủ đầy** (mọi hạng mục phần đó có file chưa?), **khả thi** (agent làm theo có chạy được không?), security rules có bị vi phạm không | Author + reviewer thứ hai (agent khác nếu có) |
| **Review 2** | *Nhất quán & Sẵn sàng* | Cross-reference giữa các file (tên struct/crate/đường dẫn/task ID có khớp 100%?), không mâu thuẫn giữa các tài liệu, thuật ngữ ổn định, checklist DoD hoàn chỉnh, mọi task có *acceptance criteria* | Author + reviewer thứ hai |

**Quy trình:**
1. Viết hết tài liệu của phần.
2. Review 1 → ghi finding `F0-xxx`/`F1-xxx` (mức `blocker/major/minor`) → **fix hết blocker+major** → ghi `Fixed`.
3. Review 2 → chạy lại toàn bộ cross-check (dùng `grep` theo tên symbol) → fix → ghi log.
4. Cập nhật trạng thái ở mục 1 bảng trên → **mới** sang phần tiếp theo.

**Finding ID:** `F{phần}-{số}`. Ví dụ `F1-003`. Không xóa finding — luôn ghi trạng thái.

## 4. Quy ước đặt tên (không đổi sau khi đã viết docs)

| Hạng mục | Giá trị |
|---|---|
| Tên sản phẩm / brand | **VietIME** (tên public cuối cùng chốt trước khi release, xem ADR) |
| Tên kho | `vietime` (monorepo, Git) |
| Crate Rust | `vietime-core`, `vietime-ffi`, `vietime-strategy`, `vietime-config`, `vietime-appdb`, `vietime-ipc`, `vietime-field-detect`, `vietime-win-tsf`, `vietime-win-hook`, `vietime-tray`, `vietime-updater`, `vietime-cli` (+ tool: `vietime-appcomptest`, `vietime-bench`) |
| Binary Windows | `vietime-tsf.dll`, `vietime-tray.exe`, `vietime-hook.exe`, `vietime.exe` (CLI), `vietime-setup.exe` |
| Pipe IPC | `\\.\pipe\vietime-ipc-v1` |
| Config người dùng | `%APPDATA%\VietIME\config.json` |
| Preset người dùng | `%APPDATA%\VietIME\appdb.json`; mặc định cài: `<install>\data\appdb.default.json` |
| Log | `%LOCALAPPDATA%\VietIME\logs\` — **không bao giờ ghi nội dung phím** |
| Kiểu gõ | `telex`, `vni`, `viqr`, `simple_telex` |
| Bundle macOS | `~/Library/Input Methods/VietIME-IM.app` (IMK) · `/Applications/VietIME.app` (settings/menu bar) · bundle id `vn.vietime.im` |
| Config/Socket macOS | `~/Library/Application Support/VietIME/{config.json, state.json, appdb.json, ipc.sock}` · log `~/Library/Logs/VietIME/` |
| Config/Socket Linux | `~/.config/VietIME/{config.json, state.json, appdb.json, ipc.sock}` · log `~/.local/state/VietIME/log/` (chốt `40-linux/P3-0 §2`) |
| Binary Linux | `vietime` (CLI), `vietime-tray`, `vietime-x11`, `vietime-ibus-engine`, `libvietime-fcitx5.so` |

## 5. Changelog của chỉ mục

- 2026-09-27: Phần 0 hoàn thành, Review 1 (11 finding) + Review 2 (7 finding) → **đạt 2/2** → bắt đầu Phần 1 (Windows).
- 2026-09-27: Phần 1 (Windows) hoàn thành — 7 file solution + 46 task `WIN-*` (P1-0…P1-6, đánh số có khoảng trống cố ý);
  Review 1 (10 finding) + Review 2 (14 finding) → **đạt 2/2** → `20-windows/P1-REVIEW-LOG.md`.
  Sửa bổ sung P0 theo yêu cầu cross-part: `P0-1` (3 crate mới + CLI subcommand + `tools/win`),
  `P0-4` (`--adapter` values), `P0-REVIEW-LOG`/`specs/oracle-unikey` (task ID → WIN-007/008).
- 2026-09-27: Phần 2 (macOS) hoàn thành — 7 file solution + 46 task `MAC-*` (P2-0…P2-6);
  Review 1 (14 finding) + Review 2 (4 finding) → **đạt 2/2** → `30-macos/P2-REVIEW-LOG.md`.
  Sửa bổ sung P0/P1 cross-part: `P0-1` (layout macos-tap/tools-mac/homebrew/docs/perf + `vietime sizes`/`uninstall`),
  `P0-2` (C-ABI `ime_appdb_verify`/`ime_strategy_resolve` cho adapter không phải Rust),
  `P0-3` (đường dẫn per-OS + `engine_owner` + IPC transport per-OS), `P1-5` (tên rc-checklist-win),
  `adr/README` (ADR-006 Accepted, thêm ADR-011). ADR-006 chốt: IMK primary + CGEventTap opt-in.
- 2026-09-27: Phần 3 (Linux) hoàn thành — 8 file solution + 50 task `LNX-*` (P3-0…P3-7,
  nhóm task `T0–T6`); Review 1 (12 finding) + Review 2 (3 finding) → **đạt 2/2** → `40-linux/P3-REVIEW-LOG.md`.
  Sửa bổ sung cross-part: `P0-1` (layout linux-common/linux-x11/tools-linux/packaging-linux + CLI `purge`),
  `P0-3` (`engine_owner` += `x11`, `inject_mode` += `keycode_ascii`, bỏ hedge path Linux),
  `adr/README` (**ADR-007 Accepted**: IBus + Fcitx5 dual, không grab Wayland, X11 opt-in).
  **Hoàn tất roadmap 3 phần** (0 → Windows → macOS → Linux); sang giai đoạn implement theo task ID.
- 2026-09-27: **W0 implement (nền P0)** — workspace Rust `strategy`/`config`/`core`/`ffi`/`cli`:
  58 unit test xanh (cli 6 · config 5 · core 30 · ffi 8 · strategy 9), `clippy --workspace -- -D warnings`
  + `fmt --check` sạch; `vietime replay` (parser `.keys` + simulator, P0-4) chạy `corpus/shared` +
  `corpus/win` (9 case, exit 0) với `--adapter headless|win|mac`, `--json`, `--filter`.
  Deviation nhỏ so với `P0-1 §1`: `vietime-ffi` thêm `rlib` vào crate-type (để CLI/integration test
  Rust link được) — `P0-1 §1` đã cập nhật tương ứng.
