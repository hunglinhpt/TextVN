# PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)

> Tên tạm định (working name): **TextVN** — repo `github.com/<org>/textvn`
> Tài liệu này tổng hợp góc nhìn của 6 vai trò: **Product Owner · Solution Architect · Engineer · Technical · Audit · Chuyên gia lĩnh vực**.
> Kỳ vọng: bản **v1.0 production** sau **~6–8 tháng** với team 4–6 người.

---

## 0. Executive Summary

**Mục tiêu:** một bộ gõ tiếng Việt **miễn phí, mã nguồn mở, auditable, chạy tốt trên Windows/macOS/Linux**, thừa hưởng tinh hoa và **sửa dứt điểm bug kinh niên** của 7 dự án tham chiếu, đồng thời **chống thoái hóa khi OS update** (Win11 24H2+, macOS Sequoia/Tahoe+, GNOME/KDE Wayland) và **tương thích env/browser tốt nhất**.

**4 nguyên tắc bất di bất dịch:**

| # | Nguyên tắc | Lý do |
|---|---|---|
| P1 | **Một lõi (core) duy nhất, mọi nền tảng chỉ là adapter mỏng** | Tránh 3 bộ hành vi khác nhau; test 1 lần, chạy 3 nơi (kế thừa mô hình gotiengviet/gonhanh/bamboo-viet) |
| P2 | **Không keylogger, không network trong engine, 100% offline** | IME là tiến trình quyền cao nhất về bàn phím — uy tín = an toàn (bài học EVKey vs OpenKey) |
| P3 | **Giấy phép GPL rõ ràng, nguồn sạch, truy vết từng dòng** | Tránh scandal "giảm bớt open source" + bị audit GPL (bài học EVKey) |
| P4 | **Compatibility là data, không hard-code** | App preset/strategy lives in signed data file → fix Chrome/Excel/LibreOffice **không cần release code** |

**Kiến trúc 1 dòng:** `Core (Rust, C-ABI) → Strategy Engine (preedit/backspace/selection/surrounding) → Adapter (TSF · InputMethodKit · IBus · Fcitx5) → UI/Tray + Config + Preset DB`.

---

## 1. Phân tích 7 nguồn tham chiếu (Audit of sources)

### 1.1 Bảng tổng hợp

| Dự án | Giấy phép | Kiến trúc | Điểm HOẠT HẠNG kế thừa | Bug/nhược điểm cần TRÁNH hoặc SỬA | Cái gì lấy vào TextVN |
|---|---|---|---|---|---|
| **UniKey / x-unikey** (unikey.org) | **GPL** | Lõi C thuần (engine trong gói `x-unikey-1.0.4`), Win32 hook gửi phím | Engine Telex/VNI thuần C đã battle-test 20+ năm; UVConverter (13 bảng mã); cấu hình file | Gửi phím kiểu *keyboard hook + SendInput* → lỗi **dính chữ thanh địa chỉ Chrome, gợi ý Excel**; UniKey **không mở source bản mới** nữa (bị lợi dụng GPL); GUI cũ | **Lấy core engine làm golden reference/test oracle** (GPL → hợp pháp), chuẩn behaviour khi migrate |
| **UniKey 3.62 source** | GPL | Win32, engine đời cũ | Mã tham chiếu cho free-marking, chuyển chế độ | Engine cũ (không phải bản mới nhất) | Chỉ dùng để đối chiếu hành vi, **không** copy blind |
| **EVKey** (lamquangminh) | **mập mờ** — GUI base UniKey, **engine source đóng** (chỉ `evkau` public) | Hook + có mode IME, tray UI | **Danh sách tính năng là "spec vàng"**: loại trừ ứng dụng (ignore list), sửa dấu gợi ý browser/Excel, hỗ trợ app Metro, chạy Admin, hỗ trợ game, nhiều phím tắt, "gõ tắt khi tắt tiếng Việt", ARM64 build | Không auditable → bị cảnh báo mã độc (true/false alarm đều gây hại), nghi vi phạm GPL | **Không lấy 1 dòng code.** Lấy **behavior spec** (mục 2.4) để implement clean-room |
| **GoTiengViet** (isthaison) | **MIT** | `engine/` C thuần + adapter `linux/`(IBus) `windows/`(TSF) `macos/`(IMK); `gtv.sh test` chạy cả 3 OS | Cấu trúc repo 3 nền tảng; packaging .deb/.exe/.dmg; CI 3 OS; test harness; macro/emoji bằng Tab; cấu hình 1 định dạng cho 3 OS; parity table làm backlog | macOS mới preview; gạch đỏ chính tả chỉ có Linux; AI suggestions qua overlay tray | **Mượn nguyên layout repo, script build/test, mô hình parity table**, engine C tham chiếu |
| **WinVNKey** (SourceForge) | Miễn phí (giấy phép riêng, cần soát lại điều khoản) | Windows keyboard driver kiểu cũ (Vista/7 era) | **Độ phủ bảng mã/kiểu gõ rộng nhất**: Telex/VNI/VIQR/Microsoft, 45+ bảng mã quốc gia, **Hán-Nôm** (gõ theo âm/Pinyin/góc bốn), **kiểm tra chính tả 1 từ & cụm từ**, đặt dấu thông minh | Cực kỳ legacy (target Win7, 32/64-bit, DEP/UAC issues), không có macOS/Linux, closed-ish | Lấy **spec tính năng nâng cao**: bảng mã mở rộng, spelling check, Hán-Nôm (backlog P2), multi-lingual |
| **Gõ Nhanh** (khaphanspace) | **BSD-3** | **Rust core (7-stage validation-first pipeline)** + FFI → Swift (CGEventTap) / C# (WH_KEYBOARD_LL) | **App Compatibility Matrix** + chiến lược thay thế text `Backspace` vs `Selection` dựa trên **AX role** (ComboBox/SearchField → Selection); **auto-restore tiếng Anh** (`text`≠`têt`), **ESC restore**, **tự nhớ ON/OFF theo app**, quy tắc dấu mới `hoà`, auto capitalize, cài qua Homebrew, latency <10ms, RSS ~20MB | CGEventTap = phải xin Accessibility; không hiện trong input menu của macOS; Linux beta | **Đây là nguồn IP chính cho "app-compat engine"** — BSD-3 ⇒ copy được có ghi credit |
| **Bamboo Viet** (ngkhhuy) | **GPL-3** (core `bamboo-core` MIT) | Go core → `c-shared` (`libvicore.so`) + IBus engine + **addon Fcitx5 (C++)** | **Đầy đủ bug-fix list đã kiểm chứng**: commit-before-hide (fix lặp từ khi Enter trong chat), tối ưu D-Bus 6× (fix nhảy chữ LibreOffice), **selection-aware + token boundary** (fix nuốt chữ ô tìm kiếm/URL), **App Preset Profiles** (Electron/Chromium hardening), filter phím tắt hệ thống (`isValidState`), 4 chế độ gõ `Preedit/SurroundingText/ForwardAsCommit/UsIM`, GUI UniKey-style + CLI config, `.deb`/snap | Go runtime nặng hơn cần cho IME in-process; X11 hacks (record/clipboard) là lối đi vòng | **Toàn bộ matrix lỗi Linux/Wayland** thành test case; mô hình 4 input mode; preset per-app |

### 1.2 Kế thừa & from-scratch — quyết định rõ ràng

**Kế thừa (fork/copy, tôn trọng license):**
- `x-unikey` engine (GPL) → dùng làm **oracle test** và nguồn bàn ánh xạ dấu đã chuẩn.
- gonhanh (BSD-3) → **App-compat strategy matrix + AX detection rules + auto-restore algorithm**.
- gotiengviet (MIT) → **cấu trúc repo, script build/test, mô hình parity table**.
- bamboo-viet (GPL-3) → **bộ test case Linux/Wayland + 4 input mode** (chúng ta cũng GPL-3 nên tương thích).
- OpenKey (GPL, `tuyenvm/OpenKey`) → phương pháp "backspace technique" trên macOS.

**Clean-room (chỉ lấy IDEA, viết lại 100%):**
- Mọi thứ EVKey: ignore-list, fix suggestion bug, Metro support, game mode.
- WinVNKey: bảng mã mở rộng, Hán-Nôm, spelling check.

**Không lấy:** binary/compiled engine của EVKey, code đóng, file .exe của bất kỳ dự án nào.

### 1.3 Ma trận bug kinh niên → kế hoạch xử lý (Đây là "bài toán thật" của dự án)

| # | Bug kinh niên | Xuất hiện ở | Chiến lược fix trong TextVN | Nguồn học |
|---|---|---|---|---|
| B1 | Dính chữ / lặp dấu ở **thanh địa chỉ & ô gợi ý** (Chrome/Edge/Firefox/Safari, Excel, Word, JetBrains) | UniKey, EVKey, mọi bộ gõ backspace | **Strategy `SelectionReplace`** khi detect field là ComboBox/SearchField/autocomplete (AX trên macOS, UIA trên Windows, AT-SPI trên Linux) + preset theo bundle-id/exe | gonhanh |
| B2 | **Lặp từ cuối khi Enter** trong Messenger/Slack/Telegram/Zalo | IBus/Fcitx5 | **Commit-before-hide**: luôn commit preedit trước khi mất focus/ẩn | bamboo-viet |
| B3 | **Nuốt chữ / mất chữ** khi app có autocomplete gợi ý (ô tìm kiếm, URL) | mọi nơi | Selection-aware + chỉ thay đúng **token boundary** | bamboo-viet |
| B4 | **Nhảy con trỏ, trễ** trong LibreOffice/Google Docs | Linux (D-Bus 200ms) | SurroudingText native, giảm round-trip; strategy `ForwardAsCommit` cho editor nặng | bamboo-viet |
| B5 | **Gõ tiếng Anh ra dấu** (`text`→`têt`, `expect`→`ễpct`) | mọi Telex | **Auto-restore khi Space/đuôi từ** + **ESC restore** + từ điển từ Anh hợp lệ | gonhanh |
| B6 | **Enter/Space bị nuốt hoặc phím tắt hệ thống bị hiểu nhầm** (`Ctrl+C`, `Alt+Tab`, `Super`) | Linux | Bộ lọc state mở rộng (`isValidState`) + không nuốt modifier/keypad/combo | bamboo-viet |
| B7 | **Electron/Chromium hardening** (VS Code, Zalo PC, Slack, Notion) | mọi nơi | Preset per-app + strategy riêng cho Chromium render process | gonhanh/bamboo |
| B8 | **Terminal** gõ lỗi (kitty, alacritty, Windows Terminal, iTerm2) | mọi nơi | `ForwardAsCommit` (không preedit, không backspace thừa) | bamboo |
| B9 | **Game** (RawInput/DirectInput) & app chạy Admin | Windows | **Dual mode: TSF (chính) + Hook/IME fallback (game & elevated)** | EVKey/OpenKey |
| B10 | **Wayland** không cho grab phím | Linux | Không grab: đi qua IBus/Fcitx5 chuẩn (text-input-v3), X11 chỉ là fallback | bamboo |
| B11 | **macOS marked text gạch chân** khó chịu, lag | mọi IMK trên macOS | Kết hợp: IMK làm input chính (không cần Accessibility) + **commit strategy** (không giữ marked text lâu) + preset "backspace mode" per-app | OpenKey/gonhanh |
| B12 | Dấu **đặt位置 mới** (`hoà`, `khoẻ`, `thuỷ`) vs cũ (`hòa`) | mọi nơi | Config `diacritic_style = new|old`, engine xử lý khi `f` gõ lên đã có dấu | gotiengviet/gonhanh |
| B13 | Mất dữ liệu gõ dở khi **click sang ô khác** | mọi nơi | Giữ buffer theo context (TSF composition / IMK marked text / IBus preedit) | gotiengviet |

---

## 2. PRODUCT OWNER VIEW

### 2.1 Tầm nhìn & USP

> "Bộ gõ tiếng Việt **duy nhất** mã nguồn mở, chạy **đủ 3 OS**, **không bao giờ** dính bug thanh địa chỉ, và **chứng minh được** là sạch sẽ bằng audit công khai."

**Khác biệt hóa so với đối thủ:**

| Đối thủ | Họ làm tốt | TextVN thắng bằng gì |
|---|---|---|
| UniKey | Phổ biến, engine chuẩn | Cross-platform + browser fix + mở source + auto-update |
| EVKey | Nhiều tính năng Windows | **Auditable & GPL-compliant** (điểm yếu chí mạng của họ) |
| Gõ Nhanh | App-compat macOS/Win xuất sắc | Đủ 3 OS + Linux sâu (IBus/Fcitx5/Wayland) + test harness public |
| GoTiengViet | Cross-platform MIT | UI/UX hoàn chỉnh parity 3 OS + app-compat engine |
| Bamboo Viet | Linux fix triệt để | Đưa matrix fix Linux lên Windows/macOS |
| WinVNKey | Bảng mã/Hán-Nôm | Modern UX + 3 OS (Hán-Nôm để backlog) |
| Bộ gõ mặc định của OS | Không cần cài | Telex/VNI chuẩn + linh hoạt + miễn phí |

### 2.2 Personas & Jobs-to-be-done

| Persona | Bối cảnh | JTBD | Nỗi đau ưu tiên |
|---|---|---|---|
| **P1 Nhân viên văn phòng** | Word, Excel, Chrome, Zalo | Gõ nhanh, không mất dấu, gõ tắt | B1 (dính gợi ý), B5 (gõ Anh ra dấu) |
| **P2 Lập trình viên** | VS Code, Terminal, ChatGPT/Claude, browser | Tắt/bật theo app tự động, không phá phím tắt | B7, B9, ignore-list |
| **P3 Sinh viên** | Google Docs, Facebook, học online | Dễ dùng, có chính tả, gõ tắt tiếng Việt | B4, spelling |
| **P4 Người song ngữ VN/EN** | Gõ xen kẽ cả ngày | `text` ra `text`, `duocj` ra `được` | B5 |
| **P5 Game thủ / Admin** | Game, app cần Admin | Bộ gõ không cản game, không cần admin thường trực | B9 |
| **P6 Người dùng Linux** | Ubuntu/Fedora/Arch, Wayland | Cài 1 lệnh, chạy mọi distro, Wayland sống khỏe | B6, B8, B10 |

### 2.3 Backlog ưu tiên (MoSCoW)

**Must (v1.0):**
- M1. Core engine: Telex, VNI, VIQR (+ Simple Telex), undo (`ww`→`w`), free-marking, kiểu dấu mới/cũ, word-boundary rõ ràng.
- M2. Chiến lược xuất chữ theo app (4 strategy: `Preedit`, `Backspace+Type`, `SelectionReplace`, `ForwardAsCommit`) + **App Preset DB** ký số.
- M3. Chế độ Việt/Anh + **per-app remembered state** + **ignore list** (theo proc/bundle-id/class).
- M4. Auto-restore tiếng Anh + ESC restore + auto capitalization sau `. ! ?`.
- M5. Gõ tắt/emoji expansion (Tab hoặc Space, có Trie + prefix boundary).
- M6. Tray/Settings UI 1 bộ, parity 3 OS (Telex/VNI, bảng mã Unicode/TCVN3/VNI-Windows, style dấu, ignore list, presets, phím tắt).
- M7. Cấu hình 1 định dạng JSON+schema, hot-reload, migration có version.
- M8. Cài đặt & update: Win (MSIX/Inno per-user, winget), macOS (notarized .pkg + Homebrew cask), Linux (.deb/.rpm/AUR).
- M9. Bộ test conformance chạy cả 3 OS + fuzzing + CI/CD + signed release.
- M10. Docs: cài đặt, góp lỗi, compatibility matrix public.

**Should (v1.x):**
- S1. Chính tả offline (âm tiết: âm đầu–vần–phụ âm cuối–than) + gạch đỏ trên mọi OS.
- S2. Gợi ý từ (offline n-gram/trie) — không AI bắt buộc.
- S3. Hỗ trợ bàn phím vật lý đa quốc gia (Telex gõ được trên layout Pháp/Đức…), CapsLock as Fn.
- S4. Preset update channel (data-only release).
- S5. Game mode Windows (hook) + UIAccess cho app elevated.

**Could (v2):**
- C1. Gợi ý AI **local** (Ollama) opt-in.
- C2. Hán-Nôm (theo âm Việt/Pinyin/góc bốn) kế thừa ý tưởng WinVNKey.
- C3. Đa ngôn ngữ (Pháp, Nga, Pinyin…) như WinVNKey.
- C4. Learning từ điển người dùng (local, opt-in).

**Won't (v1):** Cloud sync, thu thập telemetry mặc định, quảng cáo, plugin marketplace.

### 2.4 "EVKey Spec" — danh sách hành vi phải implement clean-room

1. Ignore/exclude list theo ứng dụng (gõ English khi code trong IDE/terminal).
2. Fix dấu cho ô gợi ý browser + Excel (chiến lược Selection).
3. Chạy được với app Metro/UWP (TSF).
4. Nhiều phím tắt chuyển EN/VN (không giới hạn 2).
5. Gõ tắt được bật cả khi tắt tiếng Việt (tùy chọn).
6. Hỗ trợ 32/64/ARM64 Windows, chế độ portable + installer.
7. Không đơ chương trình (watchdog restart engine, không block UI thread).

### 2.5 KPI & Definition of Success

| KPI | Target v1.0 |
|---|---|
| Latency gõ (p99, key→hiện chữ) | < **10 ms** (core < 0.5 ms) |
| RSS của engine | < **30 MB** (tray+UI < 60 MB) |
| Crash-free session | > 99.9% |
| App-compat matrix pass | > 95% trên 40 app top (bảng ở mục 5.4) |
| Thời gian cài < 2 phút, không reboot | ✓ |
| Test suite xanh cả 3 OS | 100% gate |
| external audit security | trước v1.0 |

### 2.6 Release train

| Milestone | Thời gian | Nội dung | Exit criteria |
|---|---|---|---|
| **M0 – Foundations** | Tuần 1–3 | Repo, license, ADR, core skeleton, FFI contract v1, CI 3 OS | `core` build + 100 test xanh 3 OS |
| **M1 – Tech demo** | Tuần 4–8 | Adapter thô: TSF (Win), IMK (mac), IBus (Linux) gõ được Telex trong Notepad/TextEdit/gedit | Gõ được `được`, không crash 24h soak |
| **M2 – Compat alpha** | Tuần 9–14 | Strategy engine + preset DB + auto-restore + ignore list; matrix 20 app | Matrix pass ≥ 80% |
| **M3 – Feature beta** | Tuần 15–20 | Tray UI đủ 3 OS, gõ tắt, config schema, Fcitx5, packaging | Beta public, 100 người thử |
| **M4 – Hardening** | Tuần 21–26 | Fuzz, perf, updater signed, chính tả, 40-app matrix ≥ 95% | Feature freeze |
| **M5 – Audit & RC** | Tuần 27–30 | External security+license audit, notarization, winget/brew | v1.0.0 |

---

## 3. SOLUTION ARCHITECT VIEW

### 3.1 C4 — Context

```
       ┌────────────── OS / Apps ──────────────┐
       │ Browser │ Office │ IDE │ Terminal │ Game │ Chat │
       └─────┬───────────┬───────────┬─────────┘
             │ key events / text slots
   ┌─────────▼───────────────────────────────┐
   │                TextVN                  │
   │  Core Engine · Strategy · PresetDB      │
   │  Tray/Settings · Updater · Diagnostics  │
   └────┬───────────────┬───────────────┬────┘
        │               │               │
   Local config     Signed preset    (opt-in) Local
   JSON + schema    & update feed    AI suggestion
   (không network trong engine)      — tắt mặc định
```

### 3.2 C4 — Containers (Monorepo) — *bản layout chính thức đã chốt ở `docs/10-shared/P0-1-repo-and-workflow.md`; dưới đây là bản rút gọn*

```
textvn/
├── core/                    # Rust — engine thuần, KHÔNG std I/O, KHÔNG network, no_std-friendly
├── ffi/                     # crate textvn-ffi — C-ABI duy nhất (header include/textvn_ffi.h)
│   └── src/{normalize,buffer,validate,transform,marking,word,macro,spelling}/ ← nằm trong core/
├── strategy/                # Rust — output strategy state machine (per context)
├── config/, appdb/, ipc/    # crate load config · preset ký số · codec IPC
├── schemas/                 # JSON Schema: config.v1, appdb.v1, ffi.v1, ipc.v1
├── data/                    # appdb.default.json, tables/*.toml, spelling/, emoji.tsv
├── adapters/
│   ├── windows-tsf/         # Rust + windows crate — TSF Text Service (đã chốt: ADR-005)
│   ├── windows-hook/        # WH_KEYBOARD_LL fallback (game/elevated) — Rust, process riêng
│   ├── macos-imk/           # Swift — InputMethodKit .app          (Phần 2)
│   ├── macos-tap/           # Swift — CGEventTap optional           (Phần 2)
│   ├── linux-ibus/          # C — ibus engine                      (Phần 3)
│   └── linux-fcitx5/        # C++ — fcitx5 addon                   (Phần 3)
├── tray/                    # textvn-tray (egui) + updater + CLI   (đã chốt: ADR-004)
├── tools/appcomptest/       # driver UIA/AX/AT-SPI cho app-compat test
├── fuzz/, xtask/, docs/, packaging/, corpus/
└── .github/workflows/       # ci-shared, ci-{linux,windows,macos}, release.yml (signed)
```

### 3.3 Hợp đồng FFI v1 — **bản chính thức đã chốt: `docs/10-shared/P0-2-engine-ffi-contract.md`**

Tóm tắt (khi mâu thuẫn, **P0-2 + test size trong CI thắng**):

- API theo **instance**: `ime_abi_version()` · `ime_instance_new(config)` · `ime_set_context(ctx)` ·
  `ime_key(inst, key, out)` · `ime_reset` · `ime_reload_config` · `ime_suggest` · `ime_instance_free`.
- Struct POD caller-allocated: `ime_key_v1` = **20 bytes**, `ime_result_v1` = **532 bytes** (CI enforce),
  `ime_context_v1` chứa con trỏ nên kích thước theo platform.
- Action: `0 PASS | 1 REPLACE | 2 COMMIT | 3 RESTORE` (không có POPUP trong v1 — gợi ý tách API riêng).
- **Không `malloc` qua FFI** → không double-free giữa Rust/C++/Swift/C.
- **Không callback** engine→adapter trong v1; **thread-confinement**: 1 instance = 1 thread.
- Fuzz `ime_key` trực tiếp trên ABI (mọi adapter hưởng lợi); panic được `catch_unwind` → fail-open.

### 3.4 Output Strategy Engine (trái tim của app-compat)

```
ime_key() result
   └─ action
        ├─ PASS      → adapter để hệ thống xử lý (prefix, phím tắt, modifier)
        ├─ REPLACE   → strategy theo context (đúng 5 loại trong enum v1 — P0-3 §3):
        │              • Preedit           (TSF composition / IBus preedit / IMK marked text)
        │              • BackspaceType     (mặc định, ~85% app)
        │              • SelectionReplace  (field có autocomplete — B1)
        │              • ForwardAsCommit   (terminal — B8)
        │              • Passthrough       (secure / ignore list)
        │     (SurroundingText là cơ chế adapter Linux, PasteViaClipboard là fallback hiếm
        │      của hook — không nằm trong enum strategy v1)
        ├─ COMMIT    → commit ngay preedit (chat Enter — B2, commit-before-hide)
        └─ RESTORE   → khôi phục chuỗi gốc (ESC auto-restore — B5)
```

**Chọn strategy = hàm 3 tầng (data-driven):**

```toml
# Minh hoạ dữ liệu. Bản CHÍNH THỨC là JSON: data/appdb.default.json (schema: docs/10-shared/P0-3 §2)
[preset."com.google.Chrome"]
strategy_address_bar = "SelectionReplace"
strategy_body        = "BackspaceType"
notes = "B1: autocomplete dính chữ"

[preset."org.libreoffice.LibreOffice"]
strategy_body = "Preedit"        # tránh D-Bus lag B4
delay_budget_ms = 40

[preset."com.microsoft.VSCode"]
strategy_body = "BackspaceType"
ignore_default = true            # dev muốn English mặc định (P2)
```

**Quyền ưu tiên:** `field_role (OS API) > app preset > global default`.
- macOS: AX role (`AXComboBox`, `AXTextField@AXSearchField`) → Selection.
- Windows: UIA `ControlType.Edit` + `IsLegacyIAccessiblePatternAvailable`/autocomplete caret info; app exe-name.
- Linux: AT-SPI role + env hints.

### 3.5 Config & state

- `config.v1.json` — **một schema, ba OS**, validate bằng JSON Schema khi load; lỗi schema → fallback default + log (không crash).
- Hot-reload qua file watcher (bên ngoài process để không cần restart IME).
- Migration: `config_version`, auto upgrade + backup `config.v1.json.bak`.
- State per-app lưu cục bộ: `{enabled, method}` → file nhỏ, **không ghi nội dung text người dùng**.
- Môi trường/IPC: nếu cần tách tray ↔ engine → **Unix domain socket / named pipe localhost-only** với schema-validated messages; **không TCP, không localhost HTTP**.

### 3.6 UI/Tray (1 codebase, 3 native shell)

| OS | Tray | Settings |
|---|---|---|
| Windows | egui status-item (crate `tray-icon`) + icon theo theme | egui (đã chốt ADR-004) |
| macOS | `NSStatusItem` SwiftUI menu | SwiftUI Settings window |
| Linux | StatusNotifier/AppIndicator + fallback | GTK4 hoặc webview nhẹ (tuỳ chọn) |

> Quyết định ADR-004: **không** dùng Electron/Qt cho tray (tránh +80MB RAM, giữ P2 "nhẹ"). Dùng native + render chung một `ui-model` crate/library ở giữa.

### 3.7 Packaging & Distribution

- **Windows**: MSIX hoặc Inno Setup **per-user, không cần admin**, `uiAccess=true` (yêu cầu ký cert và cài ở Program Files có ký) cho app elevated; winget manifest; portable zip (32/64/ARM64).
- **macOS**: `.pkg` notarized → `~/Library/Input Methods/`; Homebrew cask; updater dùng Sparkle-style (ed25519 key).
- **Linux**: `.deb`/`.rpm`/AUR, systemd user service cho tray, `ibus` component xml + `fcitx5` addon; **không** cài global hook.
- **Release flow**: tag → CI build (matrix 3 OS × 2 arch) → reproducible (`SOURCE_DATE_EPOCH`) → sign (Authenticode / notarize / debsign + sigstore) → SBOM → GitHub Release.

---

## 4. ENGINEER VIEW

### 4.1 Ngôn ngữ & технологies (quyết định)

| Thành phần | Chọn | Lý do | Alt bị loại |
|---|---|---|---|
| Core engine | **Rust** | Memory-safe (với P2), fuzz dễ, C-ABI `cbindgen`, perf | C (gotiengviet) — dễ UB, audit khó hơn; Go (bamboo) — runtime + GC không hợp in-process IME |
| Windows TSF | **Rust + `windows` crate** (đã chốt 2026-09-27) | Cùng toolchain với core/hook/updater, chia sẻ thẳng types strategy, ít thành phần | C++/WRL (mẫu MS) — chỉ quay lại nếu spike `WIN-002` chặn (xem ADR-005); C# — COM interop chậm/khó đăng ký per-user |
| Windows hook fallback | Rust | Dùng lại crate input-event | — |
| macOS IMK | **Swift** | Native, đúng chuẩn IME menu | Obj-C |
| Linux IBus | C | chuẩn IBus | Rust bindings chưa ổn định |
| Linux Fcitx5 | C++ | addon API C++ | — |
| Tray/UI | **egui (Windows)** · SwiftUI (macOS) · GTK4 (Linux) — cùng ui-model JSON-driven | nhẹ, không Electron | Electron — loại |
| Build | `cargo` + `xtask` + CMake (adapter C/C++) | | |

### 4.2 Engine pipeline (kế thừa + mở rộng 7-stage của gonhanh)

```
1. normalize     keycode+layout+modifier → char (hỗ trợ layout FR/DE/US, Shift/Caps, Keypad)
2. filter        bỏ modifier, combo hệ thống (Ctrl+C…), state-check mở rộng  → B6
3. context       secure-field? password? → PASS-through tuyệt đối (không bao giờ đụng ô mật khẩu)
4. classify      app/field/enable state (ignore list, per-app mode)          → P2/EVKey spec
5. validate      5 quy tắc âm tiết (có nguyên âm, âm đầu hợp lệ, vần đúng…)  → gonhanh
6. transform     bảng 72 vowel + đ/Đ + tone/vowel marks + free-marking + undo
7. post          style dấu mới/cũ · word boundary · macro/emoji Trie · auto-restore EN · capitalize
8. emit          điền ime_result_v1 (REPLACE/COMMIT/RESTORE/PASS)
```

**Bảng ánh xạ kiểu gõ** là **data** (không hard-code): `data/tables/{telex,vni,viqr,simpletelex}.toml` → build thành static table; thêm kiểu gõ = thêm file, không sửa code.

### 4.3 Test strategy (không thể thiếu — đây là chỗ 7 dự án yếu)

```
Lớp 1  Unit + property test (proptest) trong core            — nhanh
Lớp 2  Golden corpus: file .keys có assertion inline (:expect) — spec: docs/10-shared/P0-4  — đối chiếu UNIKEY ENGINE
        (dùng x-unikey/UniKey 3.6 build CLI để sinh oracle)
Lớp 3  Cross-platform conformance: cùng corpus chạy trên 3 adapter qua CLI harness
        `textvn replay --keys corpus/*.keys --adapter headless`
Lớp 4  Fuzz: cargo-fuzz + libFuzzer trên ime_key/FFI/JSON config parser (sanitizers ASan/UBSan)
Lớp 5  App-compat automation (tools/appcomptest):
        - Windows: UIAutomation type vào Chrome address bar / Excel cell / VS Code
        - macOS: AX API gõ vào Safari address bar / Spotlight
        - Linux: AT-SPI2 gõ vào Firefox / LibreOffice / GNOME Search
        → assert đúng output text, không lặp/không mất ký tự  ← regression B1–B4
Lớp 6  Soak: 24h key-stream replay, memory leak check (Valgrind/ASan/Instruments)
Lớp 7  Manual matrix trên 40 app × 3 OS trước mỗi release (checklist trong docs/)
```

### 4.4 Quy trình

- Trunk-based, PR bắt buộc: test xanh 3 CI, `clippy -D warnings`, formatter, DCO/CLA, Conventional Commits → changelog tự động.
- ADR cho mọi quyết định kiến trúc (ADR-001 FFI, 002 strategy, 003 license, 004 UI stack…).
- Issue template bắt buộc: OS version + app + cách gõ lại được (keys log ẩn danh do user tự dán).

---

## 5. TECHNICAL EXPERT VIEW

### 5.1 Chiến lược "sống sót qua OS update" (không sniff version, **capability detection**)

| OS | Rủi ro update | Kỹ thuật chống thoái hóa |
|---|---|---|
| Windows 11 24H2+ | TSF/profile registration đổi, UIAccess policy, Recall/IME interactions | Đăng ký ngôn ngữ qua `ITfInputProcessorProfileManager` (API chuẩn) thay vì registry hack; thử API → fallback chain; **contract test** CI trên windows-latest + insider; không dùng deprecated `keybd_event` cho mode chính |
| macOS Sequoia/Tahoe+ | TCC/Accessibility siết, Input menu, CapsLock switch, SFSymbol/permission prompt | **IMK là primary** (không cần Accessibility permission) → CGEventTap chỉ là opt-in; feature-detect từng loại tap (HID→Session→Annotated) đúng 3 bước gonhanh; khôngarcanve private API (bị App Review/TCC đứt) |
| Linux (GNOME/KDE/Sway) | Wayland siết grab, ibus/fcitx5 versioning, GTK4 IM module | **Không grab phím** — đi chuẩn IM protocol; link `libibus-1.0`/fcitx5 theo **weak feature detection**; X11 path tách module riêng để đứt không ảnh hưởng Wayland; matrix distro CI (Ubuntu LTS, Fedora, Arch) |
| Browser | Chromium/Safari/Firefox đổi behavior autocomplete | Preset data-update channel → fix trong 48h mà không cần user update binary |
| App mới (Electron, Zed, Cursor…) | — | Preset cộng đồng + **default fallback an toàn** (`BackspaceType` + token boundary) |

**Chính sách tương thích:** hỗ trợ **N và N−1** của 3 OS; `canary channel` cho người dùng muốn test sớm; **rollback 1-click** trong updater.

### 5.2 Env compatibility (được yêu cầu đặc biệt)

- **Linux env vars** phải được detect & hướng dẫn: `GTK_IM_MODULE`, `QT_IM_MODULE`, `XMODIFIERS`; document matrix app-by-app (Chromium dùng IM riêng,某些 GTK app cần `gtk-im-module=fcitx`…); có `textvn doctor` CLI chẩn đoán env và đề nghị fix.
- **Windows**: UAC/Secure Desktop không gõ được (đúng — báo rõ cho user), app elevated cần uiAccess, RDP/Conhost không hỗ trợ TSF → fallback hook.
- **macOS**: Secure Input (mật khẩu) → IMK tự pause; App Sandbox/TCC không ảnh hưởng vì IMK hợp lệ.
- **Terminal**: mọi terminal dùng `ForwardAsCommit`.

### 5.3 Browser compatibility matrix (mục tiêu v1.0)

| Browser/field | Chiến lược | Test tự động |
|---|---|---|
| Chrome/Edge — address bar | SelectionReplace | UIA/AX/AT-SPI |
| Chrome — ô search có gợi ý | SelectionReplace | ✓ |
| Chrome — body contenteditable | BackspaceType | ✓ |
| Firefox — URL bar (autocomplete) | SelectionReplace | ✓ |
| Safari — smart search field | SelectionReplace (AXSearchField) | ✓ |
| Google Docs / Word online (canvas) | BackspaceType, debounce ngắn | ✓ |
| Office desktop (Excel cell, Word suggest) | SelectionReplace theo preset | ✓ |

### 5.4 App compat matrix (40 app, giữ trong `docs/compat.md`, CI chạy subset)

Chrome · Edge · Firefox · Safari · Word · Excel · PowerPoint · Outlook · Google Docs · Notion · Slack · Discord · Zalo · Telegram · Messenger · VS Code · JetBrains全家 · Terminal/iTerm2/kitty/alacritty · Windows Terminal · GNOME Terminal · Blender · Photoshop · Figma · Xcode · LibreOffice · Explorer/Finder search · Spotlight/Windows Search · Steam/game (mode hook) · RDP …

### 5.5 Performance budget

| Hạng mục | Budget |
|---|---|
| `ime_key` core | < 0.5 ms p99 |
| Adapter + strategy | < 3 ms p99 |
| End-to-end key→glyph | < 10 ms p99 |
| Preset lookup (HashMap + role) | < 0.05 ms |
| RSS engine | < 30 MB (không chứa corpus lớn — load theo trang nếu cần) |
| Startup cold | < 150 ms |
| Wake từ idle | không spike > 5 ms |

---

## 6. AUDIT EXPERT VIEW

### 6.1 Threat model (tóm tắt STRIDE cho IME)

| Threat | Scenario | Control |
|---|---|---|
| **Keylogger trojan** | Bản lậu bị chèn code đọc phím | GPL toàn phần + reproducible build + signer key offline + publish hash/SBOM; encourage build-from-source; `textvn verify` |
| **Supply chain** | Dependency/npm/cargo bị compromise | `cargo-deny`, lockfile, minimal deps, vendoring option, CodeQL, Dependabot, provenance attestation (SLSA L2+) |
| **Updater giả mạo** | MITM push binary độc | Ed25519 signature (minisign/Sparkle-style), HTTPS only, HTTPS + hash kép, **rollback không nhận bản ký sai**, auto-update off default cho enterprise |
| **Installer privilege escalation** | Yêu cầu admin không cần thiết | Per-user install mặc định; uiAccess chỉ khi ký + Program Files |
| **Leak dữ liệu nhạy cảm** | Buffer text vào log/crash dump | **Không log nội dung text**; crash dump opt-in + scrub; `secure` flag cho ô mật khẩu → PASS tuyệt đối |
| **Ngụy trang open source** | Như tranh cãi EVKey | License header từng file, `THIRD_PARTY.yml` truy vết, CI check license, clean-room cho mọi code liên quan EVKey/WinVNKey |
| **DLL injection / hijacking** | Windows load DLL sai thư mục | SafeDllSearchMode, absolute path, `SetDefaultDllDirectories`, ký mọi binary |
| **Denial of typing** | Crash treo IME → mất bàn phím | Watchdog auto-restart < 500 ms; adapter fail-open (PASS through) — **không bao giờ chặn phím khi engine chết** |

### 6.2 License & compliance plan

- **Chọn `GPL-3.0-or-later` cho toàn dự án** (khớp với x-unikey GPL, bamboo-viet GPL-3; compatible với BSD-3 gonhanh + MIT gotiengviet khi trích dẫn có credit).
- `THIRD_PARTY.yml`: nguồn, license, commit hash, phần dùng.
- **Clean-room rule**: mọi feature lấy ý tưởng từ EVKey/WinVNKey → ghi spec trong `docs/specs/` → code mới 100% → reviewer khác kiểm tra không tham chiếu binary.
- CLA + DCO; `REUSE.toml`/reuse lint; CI block merge nếu thiếu SPDX header.
- Trước release: **license audit** (ScanCode + người thật) + công khai `docs/compliance.md`.

### 6.3 Security/quality gates (mọi PR phải qua)

```
□ unit+property+golden tests xanh (3 OS CI)
□ clippy -D warnings · clang-tidy · govet (nếu có Go)
□ cargo-fuzz smoke (60s) + corpus regression
□ cargo-deny / dependency audit / secret scanning
□ SBOM (CycloneDX) + provenance attestation
□ license header check (REUSE)
□ perf budget check (benchmark regression < 10%)
□ no-new-network-crate trong core (static check)
□ docs: matrix/ADR/CHANGELOG cập nhật nếu đổi hành vi
□ 2 reviewers cho core/ và adapters/ (1 người có quyền security)
```

### 6.4 External audit (trước v1.0)

1. **Security audit** bên thứ 3: engine, updater, installer, FFI.
2. **License/compliance audit** (GPL clean-room, trích dẫn gonhanh/bamboo/gotv).
3. **Penetration test** bản Windows (signing, uiAccess, hook mode).
4. Public `SECURITY.md`, quy trình 90-day disclosure, **bug bounty nhẹ** sau RC.
5. Kiểm chứng độc lập KPI (latency, matrix) → công bố báo cáo.

---

## 7. TEAM, RISK & WORK BREAKDOWN

### 7.1 Team (tối thiểu → lý tưởng)

| Vai trò | Số | Trách nhiệm |
|---|---|---|
| Tech Lead / Core engine (Rust) | 1 | FFI, pipeline, perf, fuzz |
| Windows engineer (TSF + hook) | 1 | Adapter, packaging, UIA tests |
| macOS engineer (Swift/IMK) | 1 | IMK, AX, notarization, Homebrew |
| Linux engineer (IBus/Fcitx5) | 1 | Wayland, distro, AT-SPI |
| Product/UX + QA automation | 1 | Matrix, appcomptest, docs, release |
| (part-time) Security/Audit advisor | 0.2 | Gates, audit, disclosure |

### 7.2 Risk register

| # | Risk | P/I | Mitigation |
|---|---|---|---|
| R1 | TSF/IMK phức tạp → chậm hơn plan | Cao/Cao | M1 demo tối giản trước; adapter theo spec FFI chặt; reuse pattern gotiengviet |
| R2 | App matrix không đóng được 100% (Chrome đổi behavior) | Cao/Trung | Preset data-update channel trong 48h; strategy fallback an toàn |
| R3 | Apple reject/permission UX (nếu dùng tap) | Trung/Cao | IMK primary, tap opt-in |
| R4 | Team mỏng, 3 OS song song | Cao/Cao | Đúng thứ tự: core → 1 OS → nhân bản; conformance test giữ parity |
| R5 | Vướng license (trích nhầm code EVKey/WinVNKey) | Trung/Cao | Clean-room process §6.2, gate CI REUSE, audit trước 1.0 |
| R6 | Cách biệt perf với bộ gõ hệ thống | Trung/Trung | Budget CI benchmark, profile early |
| R7 | Người dùng cũ cần TCVN3/VNI-Windows | Trung/Trung | Preset bảng mã có sẵn từ M3 |

### 7.3 WBS cấp 2 (sprint-ready)

1. **Repo & Governance**: license, SECURITY, CONTRIBUTING, ADR, CODEOWNERS, CI 3 OS.
2. **Core**: keymap, buffer, validator, transform, marking, word/macro, FFI, golden corpus.
3. **Strategy layer**: 6 strategy impl + context resolver + preset loader + hot-reload.
4. **Windows**: TSF registration/IME lifecycle, composition, hook fallback, tray, UIA test driver, installer/winget.
5. **macOS**: IMK app lifecycle, preedit/commit, AX detector, tray SwiftUI, notarize, brew cask.
6. **Linux**: IBus engine, Fcitx5 addon, Wayland verify, distro packages, AT-SPI driver, `doctor` CLI.
7. **Data**: appdb presets (40 app × 3 OS), spelling rules, English stopwords, emoji/macro seeds.
8. **UX**: Settings parity, onboarding (chọn Telex/VNI, phím tắt), i18n (vi/en).
9. **Quality**: fuzz, soak, perf bench, matrix CI, release signing, updater+rollback.
10. **Security/Compliance**: threat model doc, SBOM, third-party audit, compliance doc.

---

## 8. ACCEPTANCE CRITERIA — "hoàn hảo" nghĩa là gì (v1.0)

**Gõ & engine**
- [ ] Telex/VNI/VIQR đúng 100% golden corpus (≥ 2.000 cases, đối chiếu oracle UniKey).
- [ ] `duocjwd` → `được`; `hoaf` → `hoà` (style mới) / `hòa` (cũ); `ass` → `as`; `ww`→`w`; `text ` → `text ` (không `têt`); ESC khôi phục.
- [ ] Không bao giờ xử lý chuỗi trong ô mật khẩu/secure input.
- [ ] Gõ tắt & emoji expansion có Trie, không nuốt Space sai chỗ.

**Tương thích**
- [ ] 40-app matrix ≥ 95% pass, **0 lỗi B1/B2/B3** còn mở (lặp/gõ không được/dính chữ).
- [ ] Terminal mọi nền tảng: không lỗi preedit thừa.
- [ ] Wayland + X11 + macOS + Windows 10/11 (incl. ARM64) pass.

**Vận hành**
- [ ] p99 end-to-end < 10 ms; RSS engine < 30 MB; idle CPU ≈ 0.
- [ ] Crash → fail-open (phím vẫn gõ) + tự restart < 500 ms.
- [ ] Update có chữ ký, rollback 1-click, không cần reboot/logout ngoài lần cài IME đầu macOS.
- [ ] Cài per-user, không cần admin (trừ Linux package).

**Audit & tuân thủ**
- [ ] Reproducible build khớp hash; SBOM công bố; audit external PASSED.
- [ ] 100% file SPDX + THIRD_PARTY truy vết; clean-room evidence lưu trong `docs/specs/`.
- [ ] 0 network call trong core (được chứng minh bằng test network-sandbox).

---

## 9. APPENDIX

### 9.1 Nguồn tham chiếu
- UniKey source: https://www.unikey.org/source.html · x-unikey 1.0.4 (engine GPL)
- x-unikey/Linux: https://www.unikey.org/linux.html#download-x-unikey
- EVKey: https://github.com/lamquangminh/EVKey (chỉ `evkau` public; engine đóng → clean-room only)
- GoTiengViet (MIT): https://github.com/isthaison/gotiengviet — `docs/parity.md`
- WinVNKey: https://winvnkey.sourceforge.net/ — bảng mã/Hán-Nôm/spelling
- Gõ Nhanh (BSD-3): https://github.com/khaphanspace/gonhanh.org — `docs/system-architecture.md`, `docs/core-engine-algorithm.md`, `docs/validation-algorithm.md`
- Bamboo Viet (GPL-3): https://github.com/ngkhhuy/bamboo-viet — matrix lỗi Linux/Wayland
- Tham chiếu thêm: OpenKey (GPL) https://github.com/tuyenvm/OpenKey · ibus-bamboo/ibus-lotus · NAKL

### 9.2 ADR cần viết trong tuần 1
- ADR-001 Ngôn ngữ core: Rust + C-ABI (vs C vs Go)
- ADR-002 Output strategy engine & preset schema
- ADR-003 License: GPL-3.0-or-later + clean-room policy
- ADR-004 UI stack native (không Electron)
- ADR-005 Windows: TSF primary + hook fallback (game/elevated)
- ADR-006 macOS: IMK primary + tap opt-in
- ADR-007 Linux: IBus + Fcitx5 dual-adapter
- ADR-008 Updater & signing model
- ADR-009 Config schema & migration
- ADR-010 Test pyramid + app-compat automation approach

### 9.3 Definition of Done (mỗi feature)
Code + test (unit/golden/matrix nếu liên quan) + docs + preset/data tương ứng + security review nếu chạm FFI/IPC/updater + CHANGELOG + 3 CI xanh.
