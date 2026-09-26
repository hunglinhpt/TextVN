# P1-0 — WINDOWS MASTER PLAN (Phần 1)

> Điều kiện tiên quyết: Phần 0 đã đạt 2/2 review (`../10-shared/P0-REVIEW-LOG.md`).
> Tài liệu này = **kế hoạch chi tiết phần Windows**. Các file `P1-1 … P1-6` là solution chi tiết
> cho từng workstream; `P1-6-TASKS.md` là danh sách task nhận việc.

## 1. Mục tiêu phần Windows (exit condition của Phần 1)

**Phần 1 xong khi:**
1. Gõ Telex/VNI/VIQR chuẩn (corpus `shared/` + `win/` ≥ 300 case pass).
2. **App matrix Windows 20 app đầu tiên ≥ 95% pass** (bug B1/B2/B3/B6/B7/B8 không còn open) —
   bảng đầy đủ ở `P1-5 §3`.
3. Cài per-user (không admin), gỡ sạch, ký số, update có rollback.
4. CI `ci-windows.yml` + `ci-shared.yml` xanh; nightly fuzz/soak chạy định kỳ.
5. `docs/compat.md` có số liệu thật; `vietime doctor` chẩn đoán được env.
6. Review 2/2 của Phần 1 hoàn tất.

**Không nằm trong Phần 1:** gợi ý AI, Hán-Nôm, chính tả gạch đỏ (mặc định Should),
macOS/Linux adapter (Phần 2/3), nhưng **không được phá vỡ FFI/schema** để 2 phần sau cắm vào được.

## 2. Kiến trúc 3 process (đã chốt với P0-1/P0-3)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Process của app người dùng (Notepad, Chrome, Word, VS Code…)             │
│   └── vietime-tsf.dll   (in-proc COM TIP, Rust + windows crate)          │
│         • ITfKeystrokeMgr sink → ime_key() → composition/replace          │
│         • ipc client → pipe vietime-ipc-v1 (nhận config/state)            │
└──────────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────────┐
│ vietime-hook.exe   (process RIÊNG — WH_KEYBOARD_LL, game/elevated mode)   │
│   • hook thread timeboxed <2ms → ime_key() → SendInput                    │
│   • UIA focus cache → field_role + IsPassword → ime_set_context           │
│   • fail: crash → tray restart <500ms; quá thời gian → fail-open          │
└──────────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────────┐
│ vietime-tray.exe   (autostart, egui) — SOURCE OF TRUTH                    │
│   • tray menu + Settings (egui) • IPC server • config/appdb watcher        │
│   • spawn + watchdog hook • updater (Ed25519) • diagnostics export         │
└──────────────────────────────────────────────────────────────────────────┘
         Core engine (vietime-core + vietime-ffi) được link STATIC
         vào từng process — không có engine chạy chung (không IPC trong hot path).
```

**Quy tắc bất di bất dịch của Phần 1:**
- Hot path (phím → chữ) **không đi qua IPC**; IPC chỉ sync config/state (P0-3 §4).
- TSF in-proc = crash kéo sập app người dùng → **fail-open + fuzz là nghĩa vụ**, không phải option (S4).
- Hook **không bao giờ** block > 2 ms trong callback (xem `P1-2 §3`).

## 3. Workstream & file solution tương ứng

| WS | Tên | File solution | Sản phẩm chính |
|---|---|---|---|
| **WS1** | Core readiness cho Win | `P1-5` (test) + `P0-*` | corpus `win/` chạy được trên headless |
| **WS2** | TSF adapter | **`P1-1-tsf.md`** | `vietime-tsf.dll` gõ được Telex trong 5 app chuẩn |
| **WS3** | Hook fallback | **`P1-2-hook.md`** | `vietime-hook.exe` cho game/app elevated |
| **WS4** | Strategy + AppDB Win | **`P1-3-strategy-appdb.md`** | UIA field detect + 20 preset đầu |
| **WS5** | Tray/Settings/IPC/Update | **`P1-4-ui-packaging-release.md`** | `vietime-tray.exe` + installer + updater |
| **WS6** | Test & automation | **`P1-5-test-plan.md`** | `tools/appcomptest` + matrix + CI |
| **WS7** | Task & điều phối | **`P1-6-TASKS.md`** | WIN-001…WIN-066, dependencies |

## 4. Lộ trình (Windows slice của PLAN §2.6)

| Milestone Windows | Tuần | Nội dung | Exit criteria (kiểm chứng bằng ...) |
|---|---|---|---|
| **W0 – Spike & skeleton** | 1–2 | `WIN-001..008`: ADR-005 spike TSF-in-Rust (TIP nhỏ nhất đăng ký được + gõ 1 ký tự trong Notepad), xác nhận registry/UIA/SoX paths | Demo 1 ký tự; `docs/specs/tsf-spike.md` ghi kết quả từng API |
| **W1 – Text correctness** | 3–6 | TSF composition + engine full + corpus `win/` ≥ 150 case | `replay corpus/win` xanh; Notepad/Word/VS Code gõ đúng |
| **W2 – Compat layer** | 7–10 | WS4 (UIA detect + preset) + WS3 (hook) + fix B1/B2/B6 | 12 app matrix ≥ 90%; bug B1/B2/B6 corpus có test |
| **W3 – UX & packaging** | 11–14 | WS5: tray/egui/IPC/autostart/installer/unsigned beta | Cài/gỡ sạch trên VM sạch; settings parity checklist |
| **W4 – Hardening** | 15–18 | Fuzz, soak 24h, perf budget, signing, winget | KPI `PLAN §2.5` (Win) đạt; 20 app ≥ 95% |
| **W5 – RC Windows** | 19–20 | Security review nội bộ + license audit + docs | Cổng ra: `P1-5 §6` |

## 5. Dependency graph (thứ tự kéo việc)

```
WIN-002 (spike TSF) ──┐
WIN-003 (spike đăng ký TIP per-user) ──┤
                      └─► WIN-010 → 019 (TSF core, đúng thứ tự P1-6 M1)
WIN-004 (spike UIA) ──► WIN-030..033 (field detect) ──► WIN-034..035 (preset 20 app + UI thêm preset)
WIN-005 (spike hook) ──► WIN-040..045 (hook full)
corpus win (WIN-006) ──► mọi task WS2/WS3/WS4 (reproduce trước khi fix)
WIN-050..058 (tray/IPC/updater) song song được với WS3 sau khi IPC schema chốt (P0-3 đã chốt)
WIN-060..066 (appcomptest/perf/CI) bắt đầu tuần 3, chạy song song, chặn release
```

## 6. Rủi ro & mitigation (riêng Windows)

| # | Rủi ro | P/I | Mitigation |
|---|---|---|---|
| RW1 | **TSF bằng Rust thiếu sample/docs** → chậm | Cao/Cao | Spike `WIN-002` tuần 1 với checklist API trong `P1-1 §9`; **fallback ADR-005b**: chuyển C++/WRL cho phần COM glue, core vẫn Rust (giữ FFI) |
| RW2 | Đăng ký TIP per-user bị bắt buộc admin | Trung/Cao | Spike `WIN-003`: thử `ITfInputProcessorProfiles::Register` trong HKCU; nếu cần HKLM → ghi registry trực tiếp theo layout TIP (documented) + test VM sạch; installer ghi rõ trong UI |
| RW3 | **Antivirus false positive** (hook + SendInput = hành vi keyinject) | Cao/Trung | Ký EV/OV (SignPath OSS), Submit to Microsoft Security Intelligence + VirusTotal whitelist, đóng gói có manifest rõ, minimal hook scope, không tự tải code |
| RW4 | UIA query chậm/lệch (Chrome/Electron) | Trung/Trung | Async + cache theo hwnd + invalidate theo focus event; budget <2ms; fallback theo preset (không block gõ) |
| RW5 | GHA `windows-latest` không chạy được UIA (session khác) | Trung/Trung | Verify tuần 1 (`WIN-007`); fallback: UIA test chạy trên self-hosted runner/VM trong nightly, PR chỉ chạy corpus headless |
| RW6 | Game anti-cheat coi hook là cheat | Trung/Trung | Hook **tắt mặc định**, chỉ bật khi user chọn "Game mode"; list game known trong preset; document |
| RW7 | uiAccess cần cert + Program Files | Trung/Trung | Installer detect: nếu cài ở Program Files + đã ký → bật manifest uiAccess (task `WIN-055`), nếu portable → hướng dẫn |
| RW8 | TSF crash kéo sập app (in-proc) | Trung/Cao | `catch_unwind` mọi entry (P0-2 §5), crash counter qua `CrashReport` IPC, fuzz gate, tối giản code trong DLL |

## 7. Definition of Done cho PHẦN 1

- [ ] Mọi task `WIN-*` trong `P1-6-TASKS.md` ở trạng thái `Done` (DoD 7 mục của Handbook §4).
- [ ] `P1-5 §6` release gate pass (corpus, matrix, perf, soak, security checklist).
- [ ] `docs/compat.md` cập nhật số liệu thật.
- [ ] `P1-REVIEW-LOG.md` đạt 2/2 → `00-INDEX` cập nhật.
- [ ] Không còn finding `blocker/major` mở; rủi ro RW1–RW8 đều có owner + trạng thái.
- [ ] Bàn giao cho Phần 2 (macOS): FFI/schema **không đổi** hoặc đã bump version đúng quy trình P0-2 §6.
