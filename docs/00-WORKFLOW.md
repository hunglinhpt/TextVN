# TextVN — Workflow & Ground Rules thống nhất cho mọi agent

> **Phạm vi:** mọi agent (người hoặc AI) làm việc trong repo này — đây là *quy trình vận hành hằng ngày*.
> **Nguồn gốc:** yêu cầu thống nhất workflow (user, 2026-09-27).
> **Quan hệ:** BỔ SUNG `01-AGENT-HANDBOOK.md`. Handbook là gốc về DoD / security / review;
> file này là gốc về **thứ tự thao tác hằng ngày**. Nếu mâu thuẫn → ghi `FW-{n}` vào §7 rồi sửa cả hai
> (không tự xử lý lén, không sửa file đang có WIP của agent khác).
> **Trạng thái:** v2 (2026-09-27: thêm §8 audit 5W1H · §9 fixbug · §10 dọn dẹp · §11 docs/nghiệp vụ · §12 GitHub Actions) ·
> Review 1 (*Đúng & Đủ*) = tác giả tự rà · Review 2 (*Nhất quán & Sẵn sàng*) **chờ agent thứ hai** — finding `FW-{n}`, không xóa.

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
| G13 | **Push đỏ CI → xử lý ngay trong phiên** (`repo-hygiene` + `ci-shared`, §12) — chưa xanh thì chưa Done, không nhận task mới | giữ repo sạch sau mỗi lần code (yêu cầu user 2026-09-27) |
| G14 | **Kết thúc phiên/task → qua checklist dọn dẹp §10** (kill process test, dồn evidence, `git status` sạch, CI xanh) | tránh rò process/file tạm sang phiên sau |
| G15 | **Phần trùng lặp → bắt buộc viết thành lib/module dùng chung, KHÔNG viết riêng từng nơi** — code sạch, reuse tối đa: logic nào dùng ở ≥2 chỗ thì tách thành module chung (crate chung Rust / script lib chung) ngay từ đầu | user rule 2026-09-27; chống copy-paste (vd locator/rules UIA: field-detect ↔ appcomptest phải dùng chung 1 lib) |

## 2. Bắt buộc đọc — trình tự mỗi phiên

| # | File | Để làm gì |
|---|---|---|
| 0 | `docs/00-WORKFLOW.md` (file này) | ground rules + quy trình: audit 5W1H §8 · fixbug §9 · dọn dẹp §10 · docs/nghiệp vụ §11 · CI §12 |
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

## 5. Workflow 8 bước — mọi task

```text
[1] Đọc §2 (phần delta)        → ghi rõ task ID sẽ nhận
[2] Nhận task                    → acceptance criteria + spec liên quan
[3] Làm việc                     → lỗi: G3 · thao tác thành công: G4 · bug: quy trình §9
[4] Audit 2 vòng 5W1H (§8)       → vòng 1 Đúng&Đủ → fix hết blocker/major → vòng 2 Nhất quán&Sẵn sàng
[5] Stage + verify               → git add <list tường minh> → git diff --cached --name-only (G12)
[6] Commit + push ngay           → message `loai(scope): nd` → CI xanh (G13, §12)
[7] graphify update .            → commit graphify-out/ → push
[8] Đóng task                    → dọn dẹp §10 → cập nhật docs/nghiệp vụ §11 → ghi status
```

## 6. Vùng sở hữu file (snapshot 2026-09-27 — cập nhật §7 khi đổi)

| Agent | Vùng |
|---|---|
| Agent Windows / spike | `adapters/windows-*`, `spikes/`, `tools/win/`, `docs/specs/*spike*.md`, `docs/specs/win-test-common-errors.md`, `docs/specs/verified-ops.md`, `.github/workflows/repo-hygiene.yml`, `.github/workflows/hook-spike.yml`, `.github/scripts/` |
| Agent core/FFI | `core/`, `ffi/`, `cli/`, `config/`, `strategy/`, `corpus/`, `appdb/`, `.github/workflows/ci-shared.yml` |
| Chung (vẫn áp G11 khi có WIP) | `PLAN.md`, `docs/00-*`, `docs/01-*`, `docs/20-windows/*` (ngoài spike specs), `docs/specs/*` còn lại |

## 7. Review & change log

| Ngày | Việc | Finding |
|---|---|---|
| 2026-09-27 | Tạo v1 (workflow + ground rules + sổ verified-ops) theo yêu cầu user | — |
| 2026-09-27 | v2: G13/G14 + §8 audit 2 vòng 5W1H · §9 fixbug · §10 dọn dẹp · §11 docs/nghiệp vụ · §12 GitHub Actions (`repo-hygiene` + tham chiếu Rust/SLSA/CMake) | — |
| 2026-09-27 | v2.1: thêm `hook-spike` workflow (WIN-005, dispatch-only) vào §6 ownership + §12.1 | — |
| 2026-09-27 | v2.2: thêm **G15** — phần trùng lặp bắt buộc là lib dùng chung, code sạch + reuse tối đa (user rule) | — |
| — | Review 2 chờ agent thứ hai | `FW-{n}` |

## 8. Audit 2 vòng — 5W1H

Áp dụng cho: mỗi tài liệu phần (P0–P3), mỗi task lớn (spike/adapter), trước khi tick ✅ `00-INDEX §1`.
**Vòng 1** chạy trước khi push lần đầu của phần · **Vòng 2** chạy sau khi fix hết blocker/major vòng 1.
Finding theo G6 (`F{phần}-{n}` / `FW-{n}`) — **không xóa finding**, chỉ thêm trạng thái.

### Vòng 1 — "Đúng & Đủ" (audit *nội dung*)

| 5W1H | Câu hỏi | Check cụ thể |
|---|---|---|
| **What** | Đối tượng audit gồm file / task / section nào? | Liệt kê đủ theo task ID (grep); bỏ sót file = fail vòng 1 |
| **Why** | Vì sao file này tồn tại? Mục đích/acceptance có đạt? | Đối chiếu **từng dòng** acceptance vs evidence (log / đường dẫn / số) |
| **Who** | Ai viết, ai tiếp nhận sau này? | Tác giả + agent tiếp nhận ghi rõ; không để "ai cũng được" |
| **When** | Thứ tự / thời điểm có đúng? | Dependency, staging: spike trước khi code task, review trước khi ✅ |
| **Where** | Ở file/đoạn nào? Đúng vùng sở hữu? | Grep path → file thật tồn tại; không đụng WIP (G11) |
| **How** | Agent khác copy-paste lệnh có chạy được không? | Chạy thật lệnh build/test ghi trong doc; reproduce được evidence |

**Output vòng 1:** finding mới, phân mức `blocker / major / minor`. Fix hết **blocker + major** rồi mới sang vòng 2.

### Vòng 2 — "Nhất quán & Sẵn sàng" (audit *tinh thần đồng đội*)

| 5W1H | Câu hỏi | Check cụ thể |
|---|---|---|
| **What** | Thuật ngữ / ID / symbol có thống nhất toàn repo? | Grep mọi ID (`R*`, `G*`, `A*`, `FW*`, `WIN-*`, `S*`…) — không trùng, không đổi tên giữa 2 file |
| **Why** | Còn thiếu gì so với DoD (Handbook §4)? | Tick đủ 7 mục DoD; mọi tham chiếu `§x` trỏ đúng mục |
| **Who** | Agent tiếp nhận cần gì để bắt đầu ngay? | Mục "đọc trước" (§2) đủ; không có kiến thức ngầm (implicit) |
| **When** | Điều kiện chạy / ghép đã rõ? | OS, trigger CI, thời điểm cleanup — ghi trong doc |
| **Where** | Link / tham chiếu có hỏng? | CI `repo-hygiene` check #5 mỗi push (§12.2) |
| **How** | Sẵn sàng merge/publish chưa? | CI xanh (G13), graphify xanh, `00-INDEX §1` cập nhật, review 2/2 (G10) |

**Output vòng 2:** finding tiếp tục `F{k}-{n}`; hết major → đánh ✅.

## 9. Fixbug chuẩn hoá — 5 bước

Mọi bug (code / doc / script) đi đúng 5 bước, bỏ bước nào cũng không được:

1. **Repro trước** — tạo case fail nhỏ nhất (test/script/log dán được). Không repro được → chưa phải bug → hỏi, **không đoán**.
2. **Root cause** — viết 1 dòng "vì sao" **trước** khi sửa. Chỉ ghi triệu chứng = chưa qua bước này.
3. **Fix tối thiểu** — sửa đúng root cause, không kèm thay đổi vô liên quan.
4. **Regression guard** — bug hay lặp: thêm test case / corpus entry / check CI / dòng common-error (G3). Không có guard = chưa Done.
5. **Verify + ghi nhận** — chạy lại repro + relevant DoD (fmt/clippy/test); commit `fix(scope): …`; finding đã có → giữ nguyên + thêm `fixed @<commit>` (**không xóa**).

## 10. Dọn dẹp — checklist trước khi đóng phiên / tick Done

- [ ] **Process:** kill đúng app test **MÌNH mở trong phiên** (`taskkill /PID <id> /T /F` — mẫu `verified-ops` B3); **không** kill app người dùng thật.
- [ ] **Chrome fixture:** kill theo `--user-data-dir` của profile test; xóa profile `%TEMP%` đã tạo.
- [ ] **Office COM:** `.Close(0)` / `.Quit(0)` — không để WINWORD/EXCEL mồ côi.
- [ ] **File tạm `%TEMP%`:** evidence cần giữ → copy vào repo (spec / `spikes/`) trước khi xóa.
- [ ] **Repo:** `git status` không còn file debug/tạm (`dbg_*`, `*.tmp`…) trong vùng mình; file cần giữ → commit hoặc `.gitignore` có lý do.
- [ ] **CI:** push cuối phiên → `repo-hygiene` + `ci-shared` xanh (G13); đỏ → xử lý ngay.
- [ ] **Đăng ký hệ thống:** spike còn TIP "TextVN" registered (đang là default input) → **hỏi user trước khi unregister**.

## 11. Cập nhật docs & nghiệp vụ — checklist khi "xong"

- [ ] **Evidence vào đúng nơi:** số liệu / log / đường dẫn ghi vào spec-task file (không để sót trong hội thoại).
- [ ] **Lỗi mới → G3** common-errors (§3) · **thao tác thành công mới → G4** verified-ops (§4).
- [ ] **Nghiệp vụ lặp lại** (làm đi làm lại vẫn đúng) → nâng cấp: entry verified-ops **hoặc** script hoá trong `tools/` — chạy đúng 1 lần = không kiểm tra lại.
- [ ] **Task status** tại task list của phần mình; `00-INDEX §1` chỉ khi đủ 2/2 review (G10).
- [ ] **graphify** update + commit `graphify-out/` (G5).
- [ ] **Finding:** đủ ID + trạng thái (`fixed @commit` / còn mở), không xóa (G6).

## 12. GitHub Actions — giữ repo sạch sau mỗi lần code

### 12.1 Workflow hiện có

| Workflow | File | Chủ | Chạy khi | Việc làm |
|---|---|---|---|---|
| `repo-hygiene` | `.github/workflows/repo-hygiene.yml` | Agent Windows/spike | push `main` + PR + dispatch | 8 check "sạch" (§12.2) — **không** build Rust |
| `hook-spike` | `.github/workflows/hook-spike.yml` | Agent Windows/spike | **chỉ dispatch** | WIN-005: probe WH_KEYBOARD_LL + SendInput + UIA trên `windows-latest` (verify RW5, `P1-5 §1`) — không gate PR |
| `ci-shared` | `.github/workflows/ci-shared.yml` | Agent core/FFI | push `main` + PR + dispatch | Rust CI: fmt · clippy · abi-sizes · check-tables · verify header · perf · cargo-deny · reuse · **test 3 OS** · replay 3 OS · fuzz |
| `ci-{windows,macos,linux}` | chưa tạo | phân công sau | — | adapter theo OS (P1-5) |

**G13:** push đỏ → xử lý trong phiên, chưa xanh thì chưa Done, không nhận task mới.

### 12.2 8 check của `repo-hygiene`

| # | Check | Chi tiết |
|---|---|---|
| 1 | Conflict marker còn sót | `^<<<<<<< ` / `^>>>>>>> ` (loại `graphify-out/`) |
| 2 | File tạm/debug không được commit | `*.log *.bak *.tmp *.swp *.orig *.rej`, `.DS_Store`, `Thumbs.db`, `desktop.ini`, prefix `dbg_/debug_/tmp_/temp_` |
| 3 | `.ps1` ASCII-only | G7 / common-errors A5 |
| 4 | Secret / token / private key | `ghp_…`/`gho_…`, `github_pat_…`, `AKIA…`, `-----BEGIN … PRIVATE KEY-----` |
| 5 | Markdown link tương đối không hỏng | `docs/**/*.md` + `PLAN.md` → `.github/scripts/check_doc_links.py` (chạy local được: `python .github/scripts/check_doc_links.py`) |
| 6 | File quy trình bắt buộc tồn tại | `00-WORKFLOW`, `00-INDEX`, `01-AGENT-HANDBOOK`, `verified-ops`, `win-test-common-errors`, `PLAN.md` |
| 7 | File > 1.5MB không được commit | loại `graphify-out/` (graph auto) |
| 8 | API inject / hook non-LL bị cấm | `.github/scripts/check_no_injection_apis.py` — chính sách `docs/specs/antivirus-false-positive.md` §8 (**Farch-3**); chạy local: `python .github/scripts/check_no_injection_apis.py` |

> **Không** kiểm tra tham chiếu dạng `` `code` `` (ví dụ `` `docs/specs/uia-spike.md` ``): hiện đa số là file "kế hoạch chưa tạo" → false positive. Audit dạng này để lại bước sau (check tùy chọn qua `workflow_dispatch`).

### 12.3 Tham chiếu Marketplace (yêu cầu user 2026-09-27)

| Action / mẫu Marketplace | Nhà phát hành | Quyết định áp dụng |
|---|---|---|
| **Rust** — "Build and test a Rust project with Cargo" | GitHub Actions | ✅ **Đã có** qua `ci-shared.yml` (`dtolnay/rust-toolchain@stable` + cargo fmt/clippy/test matrix 3 OS/deny/reuse/fuzz) — **không** tạo thêm workflow trùng |
| **SLSA Generic generator** — "Generate SLSA3 provenance for your existing release workflows" | OpenSSF | 📅 **Kế hoạch** cho release workflow (P1-4 packaging): job `provenance` gọi `slsa-framework/slsa-github-generator/.github/workflows/generator_generic_slsa3.yml@v2.1.0` với input `base64-subjects` + `upload-assets: true`, permissions `actions: read` / `id-token: write` / `contents: write` → đính kèm `provenance.intoto.jsonl` cho `slsa-verifier` verify `textvn-setup.exe`. **Bắt buộc tag `@vX.Y.Z`** (SHA pin sẽ fail) |
| **CMake based, multi-platform projects** | GitHub Actions | ❌ **Không áp dụng** — repo là Cargo workspace, không dùng CMake. Ghi rõ ở đây để khỏi cân nhắc lại |
