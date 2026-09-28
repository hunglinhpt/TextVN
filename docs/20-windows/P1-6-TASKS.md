# P1-6 — TASKS Windows (WBS) — Nhận việc từng task

> Quy ước: mỗi task = 1 record. `Done` khi đủ **DoD 7 mục** (`../01-AGENT-HANDBOOK.md §4`).
> Estimate: S ≤ 0.5 ngày · M = 1–2 ngày · L = 3–5 ngày. `Dep` = task ID chặn.
> Finding review ghi vào `P1-REVIEW-LOG.md`, không sửa task đã Done mà không ghi chú.

## M0 — Spike & nền (tuần 1–2)

### WIN-001 · Chuẩn bị môi trường build (S, dep: —)
Build host: `rustup target add x86_64-pc-windows-msvc`, VS Build Tools (SDK 10.0.22621+), Inno Setup 6.
- **Acceptance:** `cargo build --workspace --target x86_64-pc-windows-msvc` pass trên máy sạch theo `P0-1 §4`.
- Docs: ghi version SDK/Inno vào `docs/20-windows/env.md`.

### WIN-002 · SPIKE: TSF-in-Rust (L, dep: WIN-001) — **ADR-005, chặn mọi task WS2**
TIP tối thiểu trong `spikes/tsf-min/`: đăng ký → Notepad nhận 1 `OnKeyDown` → `StartComposition/SetText/EndComposition` hiện "được".
- **Acceptance:** đủ 10 dòng checklist `P1-1 §9` có kết quả ✅/❌ trong `docs/specs/tsf-spike.md`.
- Nếu #2/#4 ❌ → mở ADR-005b (C++/WRL glue) ngay, ghi lý do.

### WIN-003 · SPIKE: đăng ký TIP per-user (M, dep: WIN-002)
`textvn register --scope user` trên VM không admin; xác nhận API ghi ở đâu (HKCU/HKLM).
- **Acceptance:** `docs/specs/tsf-registration-spike.md` ghi registry paths thực tế + Win+Space thấy "TextVN".

### WIN-004 · SPIKE: UIA latency & IsPassword (M, dep: WIN-001)
Query role/IsPassword trên 10 phần tử (Chrome, Word, Excel, Terminal, Explorer, KeePass…).
- **Acceptance:** `docs/specs/uia-spike.md` có bảng ms/query + kết luận cache strategy (P1-3 §2).

### WIN-005 · SPIKE: hook + GHA runner session (M, dep: WIN-001)
WH_KEYBOARD_LL + SendInput + UIA trên `windows-latest` (kiểm chứng RW5) và local.
- **Acceptance:** `docs/specs/hook-spike.md`: GHA có session desktop? SendInput có bị chặn? → quyết định nightly ở đâu.

### WIN-006 · Corpus `corpus/win/` đầu tiên (M, dep: —, song song)
Viết ≥ 60 case: `bug_B1/B2/B3/B6/B7/B8`, `secure_field_passthrough`, `owner_no_double` (theo `P1-5 §2`).
- **Acceptance:** `textvn replay corpus/win --adapter win` chạy được (kể cả khi fail — reproduct trước khi fix).

### WIN-007 · Xác nhận oracle UniKey (rủi ro carry-over từ Phần 0) (S, dep: —)
Xác nhận tên binary/flag UniKey trên Windows, cập nhật `../specs/oracle-unikey.md`.
- **Acceptance:** 1 corpus case lấy output thật từ UniKey Windows, ghi nguồn.

### WIN-008 · Kiểm tra tên/tên miền TextVN (rủi ro carry-over từ Phần 0) (S, dep: —)
Kiểm tra `TextVN` (GitHub repo đã có), trademark/namespace, đăng ký tên cho release.
- **Acceptance:** ghi `docs/compliance/naming-check.md`.

## M1 — TSF core (tuần 3–6) — dep: WIN-002, WIN-003

### WIN-010 · TIP skeleton + COM lifecycle (L)
`class.rs`/`tip.rs`/`thread.rs` theo `P1-1 §3`: DllGetClassObject, ActivateEx/Deactivate (advise/unadvise đủ).
- **Acceptance:** đăng ký → Notepad không crash; `textvn doctor` thấy TIP; test `activate_deactivate ×100` không rò cookie (debug build đếm object).

### WIN-011 · Key sink + engine PASS (M)
`key_event.rs`: system/chord check → `ime_key`; toàn bộ PASS trả `*eaten=FALSE`.
- **Acceptance:** corpus `combo_pass` (shared) pass trên Notepad; tiếng Anh không đổi hành vi 1 chữ.

### WIN-012 · ReplaceEditSession `BackspaceType` (L) — `P1-1 §6.1`
- **Acceptance:** Notepad: `dduocj` → `được`, `muownf` → `muốn`; corpus `win/*_bs_type` ≥ 20 case.

### WIN-013 · Composition + preedit + display attribute (L) — `P1-1 §7`
- **Acceptance:** gạch chân preedit trong Notepad/Word; Space/Enter → commit; corpus `tsf_preedit_*` ≥ 40 case.

### WIN-014 · Focus/commit-before-hide + `ime_reset` (M) — `P1-1 §3/§4`
- **Acceptance:** corpus `bug_B2_chat_enter` pass; manual: Alt-Tab khỏi Word khi đang preedit → text giữ nguyên (không mất không nhân).

### WIN-015 · Hotkey preserve: toggle EN/VN `Ctrl+Shift+Space` + mặc định app EN (M)
- **Acceptance:** toggle hoạt động mọi app; corpus `restore_en_*` pass (B5).

### WIN-016 · ipc_client trong TSF (M) — `P1-1 §3` (bước 5–6)
Hello/GetSnapshot/Subscribe/ConfigReload; offline-tolerant.
- **Acceptance:** đổi method trong Settings → gõ đổi ngay (<1s) ở Notepad đang mở; kill tray → gõ vẫn được (config cũ).

### WIN-017 · Secure field → `secure=1` (M, dep: WIN-004)
- **Acceptance:** corpus `secure_field_passthrough` pass với KeePassXC (matrix #17); không inject ký tự nào.

### WIN-018 · RESTORE + SelectionReplace (L) — `P1-1 §6.2`
- **Acceptance:** corpus `bug_B1_urlbar` (Chrome/Edge/Explorer) pass ≥ 90%; `restore_en_*` pass.

### WIN-019 · Self-heal `OnEndEdit` lệch + ForwardAsCommit (M)
- **Acceptance:** manual: click giữa preedit → không crash, state về Idle; corpus `bug_B8_terminal` pass (WT).

## M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006

### WIN-030 · crate `field-detect` + rules R1–R10 (M) — `P1-3 §2`
- **Acceptance:** `cargo test -p textvn-field-detect` ≥ 30 case mock element pass.

### WIN-031 · Cache + invalidation + budget 2ms (M)
- **Acceptance:** benchmark ghi vào `perf/`: 200 switch app → p99 resolve < 2ms; 0 query đồng bộ trong hook cb (assert qua log counter).

### WIN-032 · `engine_owner` + override chain (M) — `P1-3 §4`, `P1-2 §6`
- **Acceptance:** unit test thứ tự ưu tiên 6 tầng; corpus `owner_no_double` pass.

### WIN-033 · appdb loader + Ed25519 verify (M) — `P0-3 §2.2`
- **Acceptance:** file sai chữ ký → từ chối + warning trong doctor, không crash; file hỏng → fallback default.

### WIN-034 · Preset 20 app + corpus ≥ 40 case (M) — `P1-3 §3`
- **Acceptance:** bảng preset đầy đủ 20 dòng; `replay corpus/win` pass 100%; notes trỏ bug `Bn`.

### WIN-035 · Settings: "thêm preset từ app đang chạy" (S, dep: WIN-052)
- **Acceptance:** UI lưu được appdb user hợp lệ (validate qua `textvn config validate`).

## M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005

### WIN-040 · `textvn-hook.exe` skeleton + IPC + heartbeat (M) — `P1-2 §2`
- **Acceptance:** spawn/kill/restart từ tray < 500ms (test tự động trong `hook_smoke`); doctor thấy PID/uptime.

### WIN-041 · Callback theo `P1-2 §3` (L)
- **Acceptance:** corpus hook `combo_pass` + `loop_guard` pass; 0 loop (test: inject 1000 keys → không nhân đôi); callback p99 < 2ms (hook-stats).

### WIN-042 · Inject engine 3 modes + modifier restore (L) — `P1-2 §5`
- **Acceptance:** corpus `win/hook_*` ≥ 40 case pass (unicode + vk_then_unicode + selection); không phím modifier bị kẹt sau 500 lần inject (manual + automated key-state check).

### WIN-043 · Focus + UIA worker (M) — `P1-2 §4`
- **Acceptance:** role đúng trên 8 app mẫu (test script); UIA treo 1s → vẫn gõ được (fault injection test).

### WIN-044 · Rule `engine_owner` phía hook (S, dep: WIN-032)
- **Acceptance:** foreground app `owner=tsf` → hook 100% PASS-thru (counter = 0 inject trong 30s gõ).

### WIN-045 · Game/legacy presets + blocklist (M) — `P1-2 §8`
- **Acceptance:** 10 game/app vào preset; corpus `bug_B9_game_chat` pass; manual 1 game có chat.

## M4 — Tray / Settings / Packaging (tuần 11–14) — dep: WIN-016 (IPC contract đã chốt từ P0)

### WIN-050 · Tray skeleton + menu 9 mục (M) — `P1-4 §1`
- **Acceptance:** 9/9 mục hoạt động; single-instance (mở 2 lần → 1 process, lần 2 focus Settings).

### WIN-051 · IPC server + state/watch + watchdog hook (L) — `P1-4 §2/§6`
- **Acceptance:** `textvn ipc probe` thấy 2 client; kill hook → tự restart < 500ms × 5 lần liên tiếp đều OK; kill tray → hook exit sau ≤ 30s.

### WIN-052 · Settings egui 6 tab + parity checklist (L) — `P1-4 §3`
- **Acceptance:** `docs/release/parity-checklist.md` đủ mục theo `PLAN §2.3 (M6)` + `PLAN §8`; debounce 300ms (unit test counter).

### WIN-053 · Autostart + config init + hot-reload (M)
- **Acceptance:** `config.json` sửa tay → hiệu lực < 1s; file sai schema → giữ config cũ + warning trong Settings.

### WIN-054 · Installer 2 mode + uninstall sạch (L) — `P1-4 §4`
- **Acceptance:** VM sạch (Win10 + Win11): cài per-user & system → gỡ → `reg query` + file scan = 0 residual; giữ config theo lựa chọn.

### WIN-055 · uiAccess + ký số SignPath (M) — `P1-4 §5/§7`, `P1-2 §7`
- **Acceptance:** binary `Get-AuthenticodeSignature` = Valid; Notepad admin gõ được khi system-install (uiAccess ON).

### WIN-056 · Updater + rollback (L) — `P1-4 §7`
- **Acceptance:** test trên pre-release: update OK; giả lập fail (sai hash) → giữ bản cũ, có log; kill giữa update → lần mở sau rollback tự động.

### WIN-057 · winget manifest + submit (S, dep: WIN-056, release đầu)
- **Acceptance:** `winget install TextVN` trên VM sạch pass; manifest PR mở trong `packaging/winget/`.

### WIN-058 · `textvn doctor --export` (M) — `P1-4 §9`
- **Acceptance:** zip export không chứa nội dung text (test grep chuỗi đã gõ trong corpus) + có đủ 8 hạng mục chẩn đoán.

## M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC)

### WIN-060 · `appcomptest` harness (L) — `P1-5 §4`
- **Acceptance:** chạy `--suite ci` 12 app × 5 case = 60 case trên 1 lệnh, report JSON + markdown; 0 case treo > 15s.

### WIN-061 · Targets JSON cho 12 app CI (M)
- **Acceptance:** mỗi app ≥ 2 locator; chạy lại sau khi upgrade app version không vỡ (test với 1 app upgrade).

### WIN-062 · Perf bench + baseline (M) — `P1-5 §5`
- **Acceptance:** `perf/baseline-win.json` commit; CI so sánh, regression >10% → fail.

### WIN-063 · Soak 24h script (M)
- **Acceptance:** `soak.ps1` gõ xen kẽ 24h, 0 crash, RSS delta < 10MB, report vào `docs/release/` (chạy thử 2h trước).

### WIN-064 · CI jobs (M) — `P1-5 §8`
- **Acceptance:** 4 job xanh 3 lần liên tiếp; nightly report tạo Issue tự động khi fail.

### WIN-065 · Fuzz + ASan/TSan job Windows (M)
- **Acceptance:** fuzz 15' PR / 60' nightly; 0 crash; TSan job trên core+strategy pass.

### WIN-066 · Security + license checklist Phần 1 (M) — Handbook §8
- **Acceptance:** checklist S1–S9 có chứng cứ (đường dẫn file/log); `cargo deny` pass.

## Thứ tự khuyến nghị cho agent nhận việc

```
Tuần 1: WIN-001 → WIN-002 (khối chính) ∥ WIN-006 ∥ WIN-007/008
Tuần 2: WIN-002 kết thúc → WIN-003, WIN-004, WIN-005
Tuần 3–6: WIN-010 → 011 → 012 → 013 → 014 → 015 → 016 → 017 → 018 → 019
Tuần 7–11: M2 (030→035) và M3 (040→045) song song (2 stream, khác file)
Tuần 11–14: WIN-050 → 051 → 052 → 053 → 054 → 055 → 056 → (057) → 058
Tuần 15–18: WIN-060 → 061 → 062 → 063 → 064 → 065 → 066 → release gate `P1-5 §6`
```

**Checkpoint sau mỗi milestone:** chạy full `P1-5 §1` pyramid → ghi kết quả vào `P1-REVIEW-LOG.md`
(mục "Trạng thái task") → nếu có blocker/major finding mở → không kéo milestone kế tiếp.
