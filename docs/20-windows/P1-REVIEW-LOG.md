# P1 — REVIEW LOG (Phần 1: Windows)

> Giao thức: `../00-INDEX.md §3`. Review 1 = *Đúng & Đủ*, Review 2 = *Nhất quán & Sẵn sàng*.
> Phạm vi: `P1-0-MASTER-PLAN`, `P1-1-tsf`, `P1-2-hook`, `P1-3-strategy-appdb`,
> `P1-4-ui-packaging-release`, `P1-5-test-plan`, `P1-6-TASKS` + cross-check với `P0-*`, `PLAN.md`, `adr/`.

---

## REVIEW 1 — *Đúng & Đủ* (2026-09-27)

| ID | Mức | Finding | Trạng thái | Fix |
|---|---|---|---|---|
| F1-001 | **major** | Task ID ma `P0-005`/`P0-006` được refer từ `specs/oracle-unikey.md` + `P0-REVIEW-LOG` nhưng không tồn tại ở đâu → agent không nhận việc được | ✅ Fixed | Định nghĩa task thật **WIN-007/WIN-008** trong `P1-6`, sửa mọi tham chiếu (2 file P0) trỏ về `P1-6` |
| F1-002 | **major** | `P1-5/P1-6` dùng `replay … --adapter win` nhưng spec CLI `P0-4 §4` chỉ có `headless` → CLI không biết giá trị `win` | ✅ Fixed | `P0-4 §4`: `[--adapter headless\|win\|mac\|linux]` + định nghĩa "profile mô phỏng capability", giá trị lạ → exit 2 |
| F1-003 | minor | `P1-1 §11` trỏ `P1-5 §4.2` — không tồn tại (smoke script nằm §4.1) | ✅ Fixed | Sửa → `P1-5 §4.1` |
| F1-004 | **major** | Subcommand mới (`config init\|validate`, `ipc probe`, `tray --stop`, `doctor --stats/--export`) không có trong responsibility `textvn-cli` (`P0-1 §2`) → CLI viết tự phát, lệch naming | ✅ Fixed | `P0-1 §2` liệt kê đủ subcommand + bổ sung `src/{config.rs, ipc.rs, tray.rs}` vào layout |
| F1-005 | **major** | Crate mới `textvn-field-detect`, `textvn-appcomptest`, `textvn-bench` không có trong repo layout/responsibility `P0-1` (nguồn sự thật về tên crate) | ✅ Fixed | Thêm `field-detect/`, `tools/bench/`, đặt tên crate `textvn-appcomptest` + bảng responsibility `textvn-field-detect` |
| F1-006 | minor | `P1-0 §3` WS7 ghi task "WIN-001…WIN-04x" nhưng `P1-6` có tới WIN-066 | ✅ Fixed | Sửa → "WIN-001…WIN-066" |
| F1-007 | minor | `P1-0` W5 exit criterion trỏ `P1-5 §7` (manual checklist) thay vì §6 (release gate) | ✅ Fixed | Sửa → `P1-5 §6` |
| F1-008 | minor | Ký tự Trung Quốc lọt vào `P1-4 §6` (`10s一次`) | ✅ Fixed | → "10s một lần" |
| F1-009 | minor | `P1-3 §2` rule R5 mập mờ (có `?`, tên class liệt kê ngẫu nhiên); `P1-3 §3` id `win.canva/figma` chứa `/` (vi phạm format id) | ✅ Fixed | R5 viết lại theo class `Excel7/XLGRID`; id → `win.figma`, match `figma.exe, canva.exe` |
| F1-010 | **major** | `P1-3 §3` không ghi quy tắc **thứ tự file** khi 2 entry cùng exe (`win.chrome.url` hẹp vs `win.chrome.body` rộng) → resolve sai strategy (P0-3 §2.1 "entry đầu khớp thắng") | ✅ Fixed | Thêm nguyên tắc "entry hẹp đứng trước" vào `P1-3 §3` |

**Kết luận Review 1:** 0 blocker + 4 major (F1-001/002/004/005/010) đã fix hết → sang Review 2.

---

## REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27)

Cross-check bằng `grep`: IPC messages ↔ P0-3 §5 · priority ↔ P0-3 §3.1 · task ID ↔ P1-6 ·
section refs ↔ heading thật · PLAN section numbers ↔ heading thật · symbol FFI ↔ P0-2 · ký tự lạ.

| ID | Mức | Finding | Trạng thái | Fix |
|---|---|---|---|---|
| F1-011 | **major** | `P1-4` bịa message IPC: `SetMode`, `SetConfig`, `Hello{pid,ver,caps}` (sai payload), `CrashReport{module,…}` (P0-3 = `{code,count}`), `Pong` thêm field lạ; reply `GetSnapshot` trong khi P0-3 reply là `Snapshot` | ✅ Fixed | `P1-4 §2` viết lại đúng bảng `P0-3 §5` (Hello/GetSnapshot/Subscribe/ToggleViEn/CrashReport/Ping + push); menu & Settings đi qua **svc layer in-process** → ghi file → broadcast; thêm quy tắc "thêm message = sửa P0-3 + `ipc.v1.md` trước" |
| F1-012 | **major** | `P1-3 §4` bảng priority mâu thuẫn `P0-3 §3.1`: để `appdb user` (1) thắng `config.ignore_apps` (2), trong khi P0-3 để `!enabled` (bước 2) thắng user preset (bước 3) | ✅ Fixed | Viết lại `P1-3 §4` thành **ánh xạ nguồn → đúng bước P0-3 §3.1**; ghi rõ `enabled_default` = mặc định (không đấu bước 2); `app_overrides` chỉ là lớp phủ method/style |
| F1-013 | **major** | `P1-0 §5` dependency graph trỏ task **không tồn tại**: `WIN-020..024`, `WIN-035..038`, range `WIN-050..056` sai | ✅ Fixed | Graph viết lại theo đúng range P1-6 (010→019, 030..033 → 034..035, 040..045, 050..058, 060..066) |
| F1-014 | minor | `P1-0` L48 trỏ `P1-2 §4` (focus/UIA) cho giới hạn 2ms — đúng phải §3 (callback timebox) | ✅ Fixed | Sửa → `P1-2 §3` |
| F1-015 | minor | Tên file health lệch nhau: `P1-4 §6` = `hook-heartbeat.json`, `P1-2 §9` = `hook-stats.json` | ✅ Fixed | Thống nhất **`hook-stats.json`** (P1-4 §6), `Pong` không chứa health của hook |
| F1-016 | **major** | `P1-4` bịa thêm binary `textvn-win-updater.exe` mâu thuẫn `P0-1` (`textvn-updater` là **lib**) — 2 process updater thừa, routing Apply mơ hồ | ✅ Fixed | Updater = lib chạy trong tray; apply qua `textvn-setup.exe /SILENT /UPDATE`; sửa title §0/§5/§7 + viết lại flow Apply/Rollback (§7 bước 4) |
| F1-017 | minor | `P1-5 §1` dùng `cargo run -p appcomptest` — sai package name (đã đặt `textvn-appcomptest`) | ✅ Fixed | Sửa → `-p textvn-appcomptest` |
| F1-018 | minor | `tools/win/{smoke-tsf,soak,mem-check}.ps1` chưa có trong layout `P0-1` | ✅ Fixed | Thêm dòng `tools/win/` |
| F1-019 | minor | `P1-6 WIN-016` trỏ `P1-1 §3.6` — mục con không tồn tại | ✅ Fixed | Sửa → `P1-1 §3` (bước 5–6) |
| F1-020 | minor | Ký tự CJK còn sót: `adr/README` (`暴露`), `P1-6` (`接手`), `P1-4` (`申請` — sửa 1 lần vẫn còn) | ✅ Fixed | Thay hết bằng tiếng Việt/Anh; grep lại = 0 (trừ `越南` **có chủ đích** trong test UTF-8 của `P1-5 §2`) |
| F1-021 | minor | Title `WIN-007/008` vẫn ghi "Task P0-005/P0-006" (ID ma đã bị gỡ ở F1-001) | ✅ Fixed | Title → "rủi ro carry-over từ Phần 0" |
| F1-022 | **major** | Tham chiếu `PLAN.md` sai mục: parity checklist → "PLAN §4.2" (thực ra = Engine pipeline); app matrix → "PLAN §2.4" (thực ra = EVKey spec); perf → "PLAN §3.3" (thực ra = FFI); per-user install → "PLAN §5.2" | ✅ Fixed | Sửa: parity = `PLAN §2.3 (M6)` + `§8`; matrix = `PLAN §5.4` (Windows subset 20/40); perf = `PLAN §5.5`; packaging = `PLAN §3.7` |
| F1-023 | minor | Số bịa chưa kiểm chứng: "menu **10** mục" (bảng chỉ 9), "**42** control" parity (không có nguồn) | ✅ Fixed | Sửa → "menu 9 mục / 9/9"; parity = checklist điền thật vào `docs/release/parity-checklist.md` |
| F1-024 | minor | `P1-4 §7` bước Apply mơ hồ ("tsf-loaded??", `.new` + `MoveFileEx` + fallback chạy song song không rõ thứ tự) | ✅ Fixed | Viết lại 1 flow tuần tự: setup spawn → chờ hết khoá → thay → marker `last-good` → rollback bằng `staging\<old>` |

**Verify ràng buộc của Phần 0 (F0-014):**
- [x] `grep "## 6. \`apply_replace\`"` → `P1-1-tsf.md:110` ✅ (P0-2 §7 trỏ tới có mục thật)
- [x] `grep "## 5. Injection modes"` → `P1-2-hook.md:73` ✅ (P0-2 §4 trỏ tới có mục thật)

**Kết luận Review 2:** 0 blocker, 6 major (F1-011/012/013/016/022 + carry F1-010) đã fix hết → **Phần 1 đạt 2/2**.

---

## Cross-check checklist (Review 2) — kết quả

- [x] Task ID: mọi `WIN-xxx` trong P1-0…P1-5 đều tồn tại trong `P1-6` (grep range 001–066, 0 sót)
- [x] Section refs: mọi `P1-x §n` trỏ đúng heading thật (đã audit từng mục)
- [x] IPC message set = hệt `P0-3 §5` (7 message + 2 push)
- [x] Priority resolve: `P1-3 §4` ánh xạ đúng 6 bước `P0-3 §3.1`
- [x] Symbol FFI dùng trong P1 (`ime_key/ime_set_context/ime_reset/ime_reload_config/ime_instance_*`,
      `IME_FIELD_*`, `IME_CAP_*`, `ime_context_v1.{enabled,secure,field_role,caps,hint}`) đều có trong `P0-2`
- [x] Tên crate/binary/pipe/path khớp `00-INDEX §4` + `P0-1 §1` (kể cả sau khi bổ sung 3 crate mới)
- [x] `--adapter` values có trong `P0-4 §4`
- [x] 0 ký tự lạ (CJK/Cyrillic) còn lại
- [x] `PLAN.md` section refs audit lại đúng heading

**Rủi ro còn lại (chấp nhận, theo dõi — không block):**
1. **RW1**: spike `WIN-002` (TSF-in-Rust) chưa chạy — toàn bộ WS2 phụ thuộc kết quả; fallback ADR-005b đã ghi.
2. **RW5**: GHA `windows-latest` có chạy được UIA/session thật chưa → `WIN-005` tuần 1 quyết định nightly ở đâu.
3. GHA không có Windows license VM cho app matrix 12 app → nếu RW5 fail, matrix chuyển self-hosted/manual (đã ghi `P1-5 §1/§4`).
4. Win-007 (oracle UniKey) + Win-008 (tên) là carry-over từ P0, chưa làm.

**Kết luận tổng:** Phần 1 **ĐẠT 2/2** → `00-INDEX` cập nhật; Phần 2 (macOS) được phép bắt đầu.
