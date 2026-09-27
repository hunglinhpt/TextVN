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
- 2026-09-27: **W1 implement (P1-6 core feature set)** — `core`: thêm 3 method gõ
  (`vni`, `viqr`, `simple_telex`; `telex` tách `fold_with` để tái dùng), `validate.rs`
  (check âm tiết onsets/nucleus/coda/tone — bảng hand-code, xem deviation dưới), `post/`
  gồm `restore_en` (fix B5), `caps` (tự viết hoa), `macro` + `emoji` (khớp đuôi text đã gõ
  theo `macro_trigger` tab/space, có điều kiện `vi_on`), engine theo dõi buffer `recent` cho
  trigger. `config`/`ffi`: parse `macros[]`/`emoji[]` + map sang `EngineOptions`.
  `cli`: thêm subcommand `sizes` (verify ABI 20/532), `config init|validate|default`, `doctor`
  (check ABI + config người dùng; không in nội dung config — S2). `config init` mới theo
  `P0-1 §2`; `doctor --export` (WIN-058) + `register` (WIN-003) **chưa có** — ghi rõ trong `--help`.
  `corpus/shared`: +13 case (`vni_*`, `viqr_*`, `simple_telex_*`, `telex_horn_*`, `restore_en_*`,
  `caps_*`, `macro_*`, `emoji_*`) → **22/22 pass**; `cargo test --workspace` 123 test xanh,
  `clippy --workspace --all-targets` 0 warning.
  **Deviation (ghi nhận):** `validate.rs` dùng bảng âm tiết/nucleus viết tay thay vì sinh từ
  corpus `data/*.txt` (chưa có pipeline sinh bảng — `P1-3` preset mới cấp data); khi có data
  thì sinh bảng rồi diff, nếu lệch thì giữ bảng hand-code làm fallback.
  **Rủi ro còn lại:** B5 dựa cấu trúc âm tiết nên `"texts" → "tết"` (hợp lệ về cấu trúc) **không**
  được restore — đã chốt bằng corpus `restore_en_dictionary_gap_01.keys`; cần từ điển EN (P1) để đóng.
- 2026-09-27: **W2 implement (bỡ lỗ hạ tầng P0 theo plan)** — 6 việc, tất cả có bằng chứng chạy thật:
  - **`cargo xtask` + `data/tables/*.toml`** (P0-1 §3 — bảng transform **phải là data**):
    `data/tables/{vowels,telex,simple_telex,vni,viqr}.toml` → `xtask gen-tables` sinh
    `core/src/transform/vowel_table_generated.rs` (bảng 72 âm) + `core/src/method/keys_generated.rs`
    (bảng phím 4 kiểu gõ). `mark_vowel`/`mark_horn`/`circum_pair`/`is_marker` của 4 method đọc
    bảng thay vì hardcode; `undo.rs` nhận `&[(key, gốc, đích)]`. `check-tables` là gate CI
    (lệch data ↔ code = fail). xtask **0 dependency** (parser TOML tối giản 300 dòng + 5 test)
    để không kéo thêm dep vào lockfile/`cargo deny`; output được `rustfmt` hoá nên
    `cargo fmt --all --check` vẫn sạch. Hành vi **không đổi**: corpus 22/22 + 86 test core xanh.
  - **L4 fuzz** (P0-4 §1): crate `fuzz/` riêng (không thuộc workspace) với 3 target
    `ffi_key` · `config_parse` · `appdb_parse`, assert **invariant thật** (không chỉ "no panic"):
    `insert_len ≤ IME_MAX_TEXT`, fail-open `rc≠OK ⇒ PASS`, phím injected luôn PASS, abi lệch ⇒
    `IME_ERR_ABI`, NULL ⇒ `IME_ERR_INVALID_ARG`, FFI và `vietime-config` cùng kết luận,
    `last_error` không echo nội dung config (S2), appdb verify fail-closed.
  - **Stress test chạy được trên stable** (thay cho "chờ nightly"): `ffi/tests/abi_invariants.rs`
    — 200 vòng × 64 phím + ~250 input config, cùng bộ invariant, chạy **mọi PR** 3 OS.
  - **`.github/workflows/ci-shared.yml`** (P0-1 §4): 9 job — fmt, clippy `-D warnings`,
    `xtask check-tables`, abi-sizes (20/532), cargo-deny, reuse, test ×3 OS, replay ×3 OS,
    fuzz smoke 60s/target. (Các workflow theo OS thuộc agent khác, không tạo ở đây.)
  - **REUSE lint xanh**: `.reuse/dep5` (deprecated ở REUSE 6.x) → `REUSE.toml` theo `P0-1 §1`;
    sửa 2 chỗ đọc nhầm chuỗi SPDX trong nội dung (`docs/01-AGENT-HANDBOOK.md`,
    `schemas/appdb.v1.schema.json`) + `.gitignore` bỏ track `target/` ở mọi cấp
    → `reuse lint` **554/554 file, exit 0**.
  - **`schemas/ffi.v1.md`** (deliverable `P0-1 §1` còn thiếu): copy **nguyên văn** header C
    (đã verify khớp 1-1) + bất biến, bảng áp action, bảng lỗi thường gặp B1–B13, điều kiện bump ABI.
  - `vietime-ffi` re-export `ACTION_*` từ core (1 nguồn sự thật, test/fuzz không chép số tay).
  **Ghi nhận:** `cargo fmt --all` cũng đã chỉnh 1 file của agent khác
  (`field-detect/src/rules_win.rs`, chỉ whitespace) — nếu không, gate `fmt --check` mới sẽ đỏ.
  **Còn thiếu (chưa làm, ghi rõ):** `cargo xtask cbindgen` (cần crate `cbindgen` + nightly —
    header FFI vẫn do người giữ tay, kiểm bằng `vietime verify` + `vietime sizes`),
    `docs/compat.md` đã tạo khung nhưng **chưa có kết quả chạy thật** (L7 cần 40 app × 3 OS).
- 2026-09-27: **W3 implement (gate chất lượng + tài liệu)** — 3 task, đều có bằng chứng chạy thật:
  - **`vietime verify`** (P0-1 §2 liệt kê `verify` nhưng chưa có): kiểm header C
    (`ffi/include/vietime_ffi.h`) khớp code Rust theo **3 tầng** — `sizeof`, **43 hằng
    `IME_*`** (giá trị), và **tên/thứ tự trường** của 4 struct. Danh sách trường lấy từ
    `stringify!` của struct Rust nên thêm/bớt trường mà quên header → **hỏng build**.
    Đã thử bằng chứng: sửa header (đổi `IME_FLAG_ERROR` + thêm trường) → exit 1 + liệt kê
    sai lệch; hoàn nguyên → exit 0. `vietime-ffi` re-export thêm `IME_MOD_*`/`IME_FIELD_*`/
    `IME_CAP_*`/`IME_STRATEGY_*` (1 nguồn sự thật, không chép số tay). + 5 unit test cho parser.
  - **`tools/bench`** (`vietime-bench`, P1-5 §5): đo `ime_key` (chuỗi gõ thật) /
    `parse_config` / `ime_strategy_resolve`, **0 dependency** (không criterion) → sinh
    `perf/baseline-win.json` và gate hồi quy. **Thiết kế sửa 3 lần sau khi đo thật**: (1) so
    `p99` với ngưỡng 10% → báo hồi quy 84% ngay sau khi ghi baseline (p99 ở thang ns bị nhiễu
    timer/scheduler); (2) chuyển sang so **`p50`**; (3) thêm **best-of-3 + đo theo lô +
    ngưỡng nhiễu 100 ns** sau khi thấy còn lệch ±13% do tải máy. Kết quả: 3 lần đo liên tiếp
    lệch ±3.7% (gate pass), baseline giả vẫn bị bắt (`+120.7%` → exit 1). `p99` chỉ so
    **ngân sách tuyệt đối** (0.5 ms / 2 ms). +4 unit test.
  - **`docs/compat.md`** (L7 `P0-4 §1`): khung ma trận 60 app (20 Windows + 20 macOS +
    20 Linux, lấy đúng từ `P1-5/P2-5/P3-6 §3`) + 8 bước kiểm bắt buộc + quy tắc ghi chú
    (chuỗi phím lỗi phải tái hiện được) + bảng tổng kết release candidate.
  - `ci-shared.yml` giờ **11 job** (thêm `abi-header` = `vietime verify`, `perf` = bench gate).
    Job `perf` để `continue-on-error` vì runner GHA dùng chung; gate cứng để dành runner
    self-hosted (risk RW5) — ghi rõ trong `P1-5 §5` thay vì giả vờ gate đã chạy.
- 2026-09-27: **W4 — BUG config do chính gate mới phát hiện** (ghi lại vì nó là bằng chứng
  gate có tác dụng, không phải để khoe):
  - **Lỗ hổng schema**: `Config` có `#[serde(default)]` nên serde lấp cả `config_version` →
    **mọi** JSON đều "hợp lệ". Phát hiện khi thử `vietime config validate perf/baseline-win.json`
    → báo **OK** cho một file không phải config. Đã sửa: `config_version` nay là trường
    **bắt buộc** (probe `VersionProbe` riêng), còn các trường khác vẫn tuỳ chọn (P0-3 §1).
    +3 test (thiếu version / JSON không phải config / version sai). Sau khi sửa: file bench
    bị từ chối đúng (exit 1), `config default` vẫn validate OK.
  - **Gate perf bắt hồi quy thật**: sau khi sửa trên, `parse_config` parse 2 lần → p50
    512 → 731 ns (**+42.8%**) → gate đỏ đúng như mong đợi. Đây là thay đổi **có chủ đích**,
    nằm ngoài đường gõ nóng (chạy khi user bấm "Áp dụng" trong Settings) và vẫn rất xa
    ngân sách → đã ghi lại `perf/baseline-win.json` kèm lý do, thay vì nới ngưỡng.
- 2026-09-27: **W5 — đóng nhánh TỪ ĐIỂN của bug B5** (`data/stop_en.txt`, khoảng trống
  đã ghi trong `core/src/post/restore_en.rs` từ W0):
  - `EngineOptions.english_words` + `config.english_words[]` (mặc định **rỗng**) → khi chủ
    gõ khai báo `text` là từ EN của họ, Space trả lại `text` dù fold (`tễt`) hợp lệ về cấu trúc.
  - `data/stop_en.txt` (**36 từ**) **sinh bằng chính engine làm oracle** (Telex fold → kiểm
    âm tiết hợp lệ), không gõ tay cảm tính. Test `data_stop_en_file_is_self_consistent` kiểm
    lại **từng dòng** mỗi lần build → thêm bừa một từ vô nghĩa là test đỏ.
  - **Quyết định thiết kế: opt-in, KHÔNG bật sẵn.** Lý do ghi rõ trong file data và trong
    code: `test` → `tết` và `list` → `lít` là ca mơ hồ **ngược lại** — bật sẵn sẽ phá người
    đang gõ tiếng Việt. Đây là lựa chọn của người dùng, không phải suy đoán của engine
    (`PLAN §1.3` yêu cầu engine **không** tự suy đoán ngôn ngữ).
  - Corpus +2 case (`restore_en_dictionary_optin_01`, `restore_en_dictionary_no_damage_01`)
    → **24/24 pass**; test core 86 → 94, `cli` +2.
- 2026-09-27: **W6 — `schemas/config.v1.schema.json`** (deliverable `P0-1 §1` còn thiếu,
  và cần thiết sau khi siết validation ở W4):
  - Schema JSON 2020-12 khớp 1-1 với struct `Config` + `english_words[]` mới; ghi rõ
    `config_version` bắt buộc và lý do `english_words` mặc định rỗng.
  - **4 test chống lệch schema ↔ code** (không thêm crate `jsonschema` — chỉ kiểm 3 điểm
    dễ lệch: tên trường, enum, trường bắt buộc + mặc định): `schema_va_code_co_cung_bo_truong`,
    `schema_enum_khop_ham_as_str`, `schema_yeu_cau_config_version_dung_nhu_parser`,
    `schema_ghi_dung_mac_dinh_cua_code`. Test config 9 → **13**.
  - **Đã thử bằng chứng**: xoá 2 giá trị enum `method` → test đỏ; bỏ `config_version` khỏi
    `required` → test đỏ; khôi phục → xanh.
- 2026-09-27: **W7 — `vietime verify` tầng 4: drift hàm export** (lỗ hổng thật của
  header giữ tay — size/offset/hằng/trường struct đều không bắt được đổi tên
  hay đảo thứ tự hàm):
  - `cli/src/verify.rs`: `abi_exports()` (11 hàm chuẩn, khớp P0-2 §1) +
    `parse_header_exports()` (quét khối `/* ---- API ---- */`, gom theo `;`,
    chịu prototype xuống dòng, bỏ tên type `ime_*` trong tham số) +
    `parse_rust_exports()` (test-only, quét `pub extern "C" fn ime_*` trong
    `ffi/src/lib.rs` thật). `run()` so thứ tự header ↔ `abi_exports()`; JSON
    thêm `"exports":11`; report text thêm dòng `exports : 11 hàm…`.
  - `ffi/src/lib.rs`: đảo `ime_last_error` về cuối để khớp header + P0-2 §1
    (pure move, 0 đổi logic — `cargo test` vẫn xanh mới dám move).
  - **Bằng chứng gate có tác dụng**: chèn `ime_drift_probe` vào header →
    `verify` exit 1 đúng (`thứ tự hàm export · header=…drift_probe…`);
    khôi phục header → exit 0. +7 test (`cli` 14 → **21**):
    parse giữ thứ tự/bỏ type/chịu xuống dòng, đảo thứ tự bị phát hiện
    (cả header-mẫu lẫn Rust-mẫu), thiếu hàm bị phát hiện, header thật +
    `ffi/src/lib.rs` thật khớp `abi_exports()`.
  - `schemas/ffi.v1.md` + `P0-2 §6`: ghi gate mới (bump ABI khi đổi export).
- 2026-09-27: **W8 — Windows TSF Core Adapter (WIN-010..014) & Fix lỗi syntax/clippy**:
  - `adapters/windows-hook/src/lib.rs`: Sửa lỗi cú pháp dấu `}` thừa ở `empty_result()`, chuẩn hoá fmt.
  - `adapters/windows-tsf`: Triển khai đầy đủ COM server theo chuẩn Windows TSF P1-1:
    - `guids.rs`: Khai báo `CLSID_VIETIME_TIP`, `PROFILE_VIETIME`, `DISPATTR_VIETIME`, `LANGID_VI`.
    - `class.rs`: `ClassFactory` (IClassFactory) + `ObjGuard` đếm object COM toàn cục.
    - `edit_session.rs`: `ReplaceEditSession` (ITfEditSession) & `CompSink` (ITfCompositionSink), tuân thủ finding S3-1 (GetSelection) & S3-2 (null terminator).
    - `key_event.rs`: `KeySink` (ITfKeyEventSink) lọc system chords (B6) và forward key vào `ThreadState`.
    - `tip.rs`: `Tip` implementing `ITfTextInputProcessorEx`, `ITfTextInputProcessor`, `ITfThreadMgrEventSink` xử lý focus change và commit-before-hide (B2).
    - `lib.rs`: Kết nối 5 module, export `DllGetClassObject` và `DllCanUnloadNow` chuẩn C-ABI Windows COM.
    - `Cargo.toml`: Cấu hình `crate-type = ["cdylib", "rlib"]`, kéo `windows 0.61.3` + `windows-core 0.61.2` dưới `[target.'cfg(windows)'.dependencies]` để bảo toàn CI cross-platform.
    - Biên dịch thành công `target/debug/vietime_win_tsf.dll` (~1MB).
  - Hoàn thiện sổ lỗi `docs/specs/project-common-errors.md` (E1–E6) và cập nhật `docs/specs/verified-ops.md` (B6–B11).
  - Tạo `tools/win/check_reuse.ps1` chạy `reuse lint` an toàn với UTF-8 trên Windows console (E1).
  - Vượt qua 10/10 Quality Gates: 196 tests PASS, clippy 0 warning, fmt clean, reuse lint 971/971 compliant, deny ok, bench pass.

