# P0 — REVIEW LOG (Phần 0: Nền tảng chung)

> Giao thức: `../00-INDEX.md §3`. Review 1 = *Đúng & Đủ*, Review 2 = *Nhất quán & Sẵn sàng*.
> Phạm vi review: `00-INDEX.md`, `01-AGENT-HANDBOOK.md`, `P0-1`, `P0-2`, `P0-3`, `P0-4`,
> `../adr/README.md`, `../specs/oracle-unikey.md` + độ khớp với `../../PLAN.md`.

---

## REVIEW 1 — *Đúng & Đủ* (2026-09-27)

**Phạm vi:** chính xác kỹ thuật (API/đường đi có thật?), đủ hạng mục, khả thi với agent mới, vi phạm S1–S9.

| ID | Mức | Finding | Trạng thái | Fix |
|---|---|---|---|---|
| F0-001 | **blocker** | Mâu thuẫn process model của hook: `P0-1` ghi `vietime-win-hook` là lib nhúng vào tray, `P0-3 §5` liệt kê `vietime-hook` là IPC client riêng → agent không biết viết exe nào | ✅ Fixed | Chốt **process riêng `vietime-hook.exe`** (crash-isolated, tray spawn + watchdog <500ms); sửa `P0-1 §1`, thêm `vietime-hook.exe` vào bảng naming `00-INDEX §4` |
| F0-002 | **major** | `ime_context_v1.hint` mập mờ ("engine tự resolve" nhưng engine không biết adapter có preedit/selection hay không) → resolve strategy sai cho adapter thiếu capability | ✅ Fixed | Thêm `uint32_t caps` + `IME_CAP_PREEDIT/SELECTION/FIELD_DETECT/INJECT_VK`, `hint` = strategy id 0..4 hoặc -1; `P0-3 §3.1` bước 5 gate theo `ctx.caps` |
| F0-003 | minor | Walkthrough TSF trong `P0-2 §7` trỏ nhầm `P1-2 §5` (file hook) | ✅ Fixed | Sửa thành `P1-1 §6` |
| F0-004 | **major** | `P0-3 §3.1` bước 3 trỏ "user override cho app_id" nhưng `config.app_overrides` chỉ chứa `method/diacritic_style`, **không chứa strategy** → nhầm nguồn override | ✅ Fixed | Sửa thành "User preset override (user `appdb.json` §2.1)" |
| F0-005 | **blocker** | Mọi tài liệu (`P0-1 §3`, Handbook §9, PLAN §4.3, task P1-5) trỏ tới corpus `.keys` nhưng **không có spec định dạng** → agent không biết gõ file corpus thế nào | ✅ Fixed | Tạo **`P0-4-test-and-corpus.md`**: bảng lệnh DSL, simulator semantics, exit code, quy ước tên file |
| F0-006 | minor | Handbook §2 đặt tên file `P{n}-{chủ-đề}.md` không khớp file thật (`P0-2-engine-ffi-contract.md`) | ✅ Fixed | Sửa mẫu thành `P{n}-{mã}-{chủ-đề}.md` |
| F0-007 | **major** | Thiếu mapping `field_role` chuỗi (JSON preset) ↔ `IME_FIELD_*` số (FFI) → hai bảng dùng tên khác nhau, adapter sẽ đoán | ✅ Fixed | Bảng 1-1 trong `P0-3 §2.1` |
| F0-008 | **blocker** | `PLAN.md §3.3` còn bản sketch FFI **cũ** (`ime_init`, action có `POPUP`, struct layout khác) mâu thuẫn `P0-2` → 2 nguồn sự thật | ✅ Fixed | `PLAN §3.3` chỉ còn tóm tắt + câu "P0-2 + test size thắng"; gỡ code block cũ |
| F0-009 | minor | Chưa enforce size `ime_key_v1` (chỉ `ime_result_v1`) | ✅ Fixed | `P0-2 §6` thêm assert `size_of::<ime_key_v1>() == 20` + `offset_of!` |
| F0-010 | minor | `00-INDEX §2` thiếu `P0-4` trong thứ tự đọc | ✅ Fixed | Thêm bước 6, đánh lại số |
| F0-017 | minor | 3 ký tự lỗi ngôn ngữ lẫn lộn khi soạn (`命名`/`документ`/`入参`) trong handbook & P0-1/P0-2 | ✅ Fixed | Thay bằng tiếng Việt tương ứng |

**Kết luận Review 1:** 3 blocker + 4 major đã fix hết → đạt điều kiện sang Review 2.
**Rủi ro còn lại (chấp nhận):** target CLI oracle UniKey (`docs/specs/oracle-unikey.md`) chưa tự build thử —
để mở, task **WIN-007** (`../20-windows/P1-6-TASKS.md`) xác nhận lại tên binary/flag rồi cập nhật spec.

---

## REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27)

**Phạm vi:** cross-check bằng `grep` toàn bộ symbol/path/GUID/tên file giữa `PLAN.md` ↔ `P0-*` ↔ `adr/` ↔ `specs/`; khả năng chạy thật.

| ID | Mức | Finding | Trạng thái | Fix |
|---|---|---|---|---|
| F0-011 | **major** | `PLAN §4.1` vẫn chốt "Windows TSF: **C++/WRL** (hoặc Rust…)" mâu thuẫn quyết định đã chốt **Rust + windows crate** | ✅ Fixed | Cập nhật bảng `PLAN §4.1`, ghi rõ chỉ quay lại C++ nếu spike `WIN-002` chặn (ADR-005) |
| F0-012 | **major** | `PLAN §3.6/§4.1` còn "Shell_NotifyIcon/WinUI 3 dialog" mâu thuẫn **egui** (ADR-004) | ✅ Fixed | 2 bảng sửa theo egui (tray-icon crate + egui settings) |
| F0-013 | minor | `PLAN §4.3` nói corpus `.keys → .expected` (file riêng) vs `P0-4` assertion inline | ✅ Fixed | Sửa `PLAN §4.3` theo `P0-4` |
| F0-014 | **major** | `P0-2` tham chiếu `P1-1` (§6 apply_replace) và `P1-2` (§5 injection modes) nhưng file Phần 1 chưa tồn tại → ràng buộc phải giữ đúng tên/số mục | ✅ Verified (Review tổng thể #3) | Ràng buộc đã thoả: `P1-1-tsf.md` có `## 6. apply_replace`, `P1-2-hook.md` có `## 5. Injection modes` (grep 2026-09-27) |
| F0-015 | **major** | `ime_instance_new` với config sai: `P0-2 §1` không nói gì, `§5` nói "engine fallback" → không rõ instance có được tạo không (adapter có thể deref NULL) | ✅ Fixed | Chốt **non-fatal**: vẫn tạo instance với config mặc định, trả `IME_ERR_CONFIG`, `*out` luôn set; ghi ngay trong header comment + `§5` |
| F0-016 | minor | `adr/README.md` và `specs/oracle-unikey.md` được `P0-4 §6`/`Handbook §7` refer nhưng chưa tồn tại | ✅ Fixed | Tạo 2 file (ADR index 10 mục; oracle UniKey 3 cách + quy tắc ghi nguồn) |
| F0-018 | minor | Cross-check bằng grep: `ime_init`/`ime_abi(`/`appdb.toml`/`.expected`/`C++/WRL` còn sót ở `PLAN` | ✅ Fixed | Đã sửa hết (grep lại = 0 match, trừ mô tả khách quan về Bamboo Viet) |

**Kết luận Review 2:** hết major → **Phần 0 đạt 2/2**.
**Rủi ro còn lại (chấp nhận, theo dõi):**
1. Oracle UniKey chưa build thử (F0-001 rủi ro carry-over) → task **WIN-007** (P1-6).
2. Namespace `VietIME` chưa kiểm tra trademark/name collision → task **WIN-008** (P1-6) trước khi public release.
3. `ime_suggest` chỉ khai báo (feature `suggest` tắt mặc định) — không block adapter v1.

---

## Cross-check checklist (Review 2) — kết quả

- [x] `grep "ime_init\|ime_abi(\|appdb.toml\|\.expected"` → 0 lỗi mâu thuẫn còn lại
- [x] Tên crate/binary khớp giữa `00-INDEX §4`, `P0-1 §1`, `P0-3 §5` (kể cả `vietime-hook.exe`)
- [x] `ime_result_v1`=532, `ime_key_v1`=20, `IME_MAX_TEXT`=64 — nhất quán `PLAN §3.3` ↔ `P0-2`
- [x] Strategy enum 5 giá trị — nhất quán `P0-2` ↔ `P0-3 §3` ↔ `PLAN §3.4` ↔ `appdb` JSON
- [x] Bảng role JSON↔FFI có đủ 11 role
- [x] Mọi lệnh corpus trong `P0-4 §2.2/§2.3b` đều có trong simulator semantics §3
- [x] DoD mỗi file: P0-1 §…, P0-2 §6, P0-3 §7, P0-4 §6 — đủ
- [x] `00-INDEX` trạng thái cập nhật: Phần 0 → ✅ 2/2

**Kết luận tổng:** Phần 0 **ĐẠT 2/2** → mở Phần 1 (Windows).

---

## Review tổng thể đợt 3 (full sweep 34 file docs + tiếp nhận scaffold) — 2026-09-27

**Phạm vi:** toàn bộ `docs/` (34 md) + `PLAN.md`; kiểm tự động bằng script (CJK, placeholder,
finding chưa-fix, path ref, task ID, §-ref, tên crate) + đối chiếu scaffold Rust thật của agent kia.

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F0-014 | (đã ghi Review 2) | Ràng buộc `P1-1 §6`/`P1-2 §5` chờ verify | ✅ Verified | Grep xác nhận: `P1-1-tsf.md` có `## 6. apply_replace`, `P1-2-hook.md` có `## 5. Injection modes` |
| F0-019 | minor | `00-INDEX` changelog ghi "Phần 1 … **66 task** `WIN-*`" — thực tế `P1-6` chỉ có **46** heading (khoảng trống đánh số cố ý: 009, 020–029, 036–039, 046–049, 059) | ✅ Fixed | Sửa thành "46 task (đánh số có khoảng trống cố ý)" |
| F0-020 | minor | `P0-REVIEW-LOG` header tham chiếu `../PLAN.md` (= `docs/PLAN.md` — không tồn tại); PLAN.md ở repo root | ✅ Fixed | → `../../PLAN.md` |

**Kết quả kiểm tự động (sweep 3):**
- [x] CJK: 0 mới (chỉ còn trích dẫn lịch sử có chủ đích trong REVIEW-LOG + `越南` test `P1-5 §2`).
- [x] Placeholder: 0 thật (toàn bộ match là mẫu protocol `F0-xxx`/`WIN-xxx`).
- [x] §-ref chéo `P0-P3`: 0 hỏng (file + số mục tồn tại).
- [x] Finding chưa-fix: chỉ F0-014 (đã verify).
- [x] Task ID: WIN=46, MAC=46, LNX=50 — không có ID nào được tham chiếu nhưng thiếu
      (WIN-020/MAC-045 chỉ nằm trong trích dẫn finding lịch sử).
- [x] Tên crate khớp `P0-1 §2` ↔ scaffold thật: `vietime-{core,strategy,config,ffi,cli}` ✓.
- [x] Path ref chưa tồn tại = **task deliverable** (spike specs, rc/parity checklists, `docs/compat.md`,
      `env-*.md`) — đúng quy tắc deliverable, không phải lỗi.

**Tiếp nhận scaffold (agent kia) — kết quả:**
- [x] `cargo test --workspace` → **58/58 pass** (core 30, strategy 9, ffi 6, config 5, cli 8); FFI size/offset test đúng 20/532 (P0-2 §6).
- [x] `cargo clippy --workspace --all-targets` → 0 warning.
- [x] fail-open/`catch_unwind`/config-sai-non-fatal đúng P0-2 §0/§5 (có test).
- [ ] `cargo run -p vietime-cli -- sizes` **chưa implement** (P0-1 §3, P0-2 §6 — F2-003 đã định nghĩa) → giao lại agent cli (đang active), **không đụng** để tránh conflict.
- [ ] `rust-toolchain.toml` để `channel = "stable"` chưa pin version (chính file có TODO(WIN-001)) — giữ theo TODO.

**Kết luận:** docs đạt; scaffold accepted (chờ `sizes`). Mở development Windows (task `P1-6`) theo phân chia:
agent kia giữ `core/ffi/cli/config/strategy`, tôi nhận `adapters/windows-*`, `tools/win/`, `corpus/win/`.
