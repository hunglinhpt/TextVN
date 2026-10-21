# P4-0 — MASTER PLAN: Trợ lý ngữ pháp tiếng Việt (Assist) cho TextVN

- **Ngày:** 2026-10-20 (v1) · **v2:** sau **3 vòng review chuyên sâu** (vòng 3 SA/QA, vòng 4 Audit, vòng 5 PO/BA/Design) · **Trạng thái:** Kế hoạch (chưa triển khai sản phẩm)
- **Phần:** 4 — P4 · **Owner:** agent · **Review:** vòng 1 = `P4-1-harper-evaluation.md` · vòng 2 = §11.1 · vòng 3–5 = §11.2
- **Cross-ref:** `PLAN.md` §2.3 (S1 chính tả + gạch đỏ · S2 gợi ý từ · C1 AI local) · `PLAN.md` §5.5 (perf budget) · `PLAN.md` §6.1 (threat model STRIDE) · `docs/10-shared/P0-2-engine-ffi-contract.md` · `P0-3-config-preset-strategy.md` · `P0-4-test-and-corpus.md` · `docs/specs/antivirus-false-positive.md` §8 · `docs/01-AGENT-HANDBOOK.md` (DoD 7 mục) · ADR-012 (`docs/adr/README.md`)
- **Yêu cầu gốc (user 2026-10-20):** 2 chế độ — **highlight lỗi ngữ pháp** và **tự correct cả câu** sau khi bấm **dấu cách** hoặc **`,` / `.`**

---

## 0. Tóm tắt điều hành

| | |
|---|---|
| **Làm gì** | Kiểm tra **câu vừa gõ** bằng bộ luật tiếng Việt **objective** (lặp từ, khoảng trắng, hoa đầu câu, sai dấu…) → **gợi ý** (overlay/candidate) và (v2) **tự sửa cả câu** |
| **Không làm gì** | Không đọc nội dung app khác · không network · không log text · không gạch chân thật trong field · không ngữ pháp học thuật ở v1 |
| **Vì sao tự viết** | Harper **chỉ hỗ trợ tiếng Anh** ở mọi tầng (không có giá trị cho tiếng Việt) + API churn ~hàng tháng + chi phí resident khi nhúng trong TIP in-process → mượn **kiến trúc**, tự viết **luật + từ điển** (chi tiết: `P4-1`, `P4-2-CROSS-REVIEW-2026-10-20.md`) |
| **Rủi ro lớn nhất** | **Áp sửa text đã commit** → đã đổi cơ chế sang **Backspace+Type có precondition** (D12) + **fail-safe** |
| **Ước lượng** | 6 pha · ~14–18 tuần (tăng so với v1 sau vòng 3–5: thêm 3 spike chặn + runtime shadow buffer thật) |

---

## 1. Bối cảnh & mapping với PLAN hiện có

| Backlog | Nội dung hiện tại | Khoảng cách với yêu cầu mới |
|---|---|---|
| `S1` | "Chính tả offline (âm tiết…) + **gạch đỏ** trên mọi OS" | Chỉ bắt lỗi **từ**; chưa có **ngữ pháp câu**, chưa có **tự sửa cả câu** |
| `S2` | "Gợi ý từ (offline n-gram/trie)" | Chỉ gợi ý **từ**; chưa có **highlight lỗi câu** |
| `C1` | "Gợi ý AI **local** (Ollama) opt-in" | Để v2.x, **không** nằm trong đường chính (P2: không network) |

→ Phần 4 (P4) **nâng S1+S2** thành *offline grammar assistant cho bộ gõ*: 2 chế độ, 3 OS, không network, không tiến trình ẩn.

---

## 2. Mục tiêu & phiêu miêu

**Mục tiêu (v1):** khi người dùng gõ xong một câu, TextVN **phát hiện lỗi objective trong câu vừa gõ** và **gợi ý sửa** — 100% offline, không log text, không đọc nội dung app khác.

**Không phải mục tiêu (v1):** không phải "Grammarly đầy đủ" (POS, ngữ pháp học thuật, đọc cả tài liệu) · không sửa format trong field của app · không tự sửa khi luật không chắc chắn · không gửi text ra máy khác.

**Tiêu chí thành công (đo được):**

| # | Tiêu chí | Ngưỡng |
|---|---|---|
| 1 | Recall trên corpus lỗi objective | **≥ 80%** |
| 2 | FPR trên corpus cây sạch | **≤ 2%** |
| 3 | Độ thống nhất giữa 2 người đánh giá độc lập (100 câu) | **≥ 80%** |
| 4 | Độ trễ cảm nhận khi gõ | **không tăng > 5 ms** |
| 5 | Rò rỉ | 0 log/0 network/0 đọc nội dung app khác |

---

## 3. Kiến trúc (Solution Architect view)

```
┌─ Adapter (3 OS) ────────────────────────────────────────────────────────────────┐
│ windows-tsf (TIP, Rust)  macos-imk (Swift)   linux-ibus (C) / linux-fcitx5 (C++)│
│ + trigger space/,/.  + idle 700ms  + caret check (chỉ VỊ TRÍ, không đọc text)   │
│ + **không** tự giữ buffer câu (tránh 3 bản trôi lệch — D24)                     │
└──────────────┬──────────────────────┬──────────────────────┬──────────────────┘
               │ Rust: gọi crate `assist` trực tiếp (D23)                          │
               │ Swift/C/C++: qua C-ABI `ime_assist_*` trong crate `ffi`            │
               │ + header `textvn_ffi.h` + `textvn-cli verify` chống drift (D23)    │
       ┌───────▼──────────────────────────────────────────────────┐
       │  core/  (đã có)                                            │
       │  `Engine.sentence` — buffer CÂU (Unicode, ≤256, chỉ bật    │
       │  khi assist.mode != off — D26) + `Engine::glyph_len_of`    │
       │  (số ký tự glyph để xóa — D27)                            │
       │  ↕ refactor 13 call site `recent_replace` → 2 helper (D24)  │
       ├───────────────────────────────────────────────────────────┤
       │  assist/  (crate Rust MỚI — thuần, 0-dep, không I/O)      │
       │  tokenize → mask(URL/email) → rules/(1 file = 1 luật)       │
       │  → Issue{kind, span(char), message, suggestions, fix,       │
       │         confidence} → apply_auto_fixes (char-space)         │
       │  tài nguyên nặng: OnceLock<Arc<…>> (immutable, D25)        │
       └───────────────────────────────────────────────────────────┘
               │ kết quả (RAM only)
    ┌──────────▼─────────────┐   ┌────────────────────────────────────┐
    │ strategy crate          │   │ UI 3 OS (D13)                       │
    │ Backspace+Type (A2/A3)  │   │ • Windows: ITfCandidateListUI (mới) │
    │ + precondition caret     │   │ • macOS: IMK input controller (có)  │
    └──────────────────────────┘   │ • Linux: IBus/Fcitx5 candidate list │
                                   │ fallback: tray toast (P4-2)         │
                                   └────────────────────────────────────┘
```

### 3.1 Quyết định kiến trúc (đã chốt)

| # | Quyết định | Lý do | Nguồn |
|---|---|---|---|
| D1 | Không vendor `harper-core` **ở v1** | English-only ở mọi tầng (không có giá trị cho tiếng Việt) · API churn ~hàng tháng (2.13.0 mới 4 ngày) · chi phí resident khi nhúng trong TIP in-process (cần spike S4) · ed2024 → MSRV ≥ 1.85. ⚠️ lý do *"crates.io stale"* ở bản v1 là **SAI** → xem **D21** + `P4-2-CROSS-REVIEW-2026-10-20.md` Fassist-32 | `P4-1` §0, D21 |
| D2 | Mượn kiến trúc Harper, tự viết luật VN | Tầng tokenize/lint mượn được; từ điển+luật+POS là 100% tiếng Anh | `P4-1` §0 |
| D3 | Tự xây từ điển VN từ nguồn mở, **audit license từng nguồn** | Không có từ điển VN "đóng gói sẵn + license sạch" | `P4-1` §0 |
| D4 | Highlight = hiển thị **cạnh con trỏ** (không gạch chân thật trong field) | Giới hạn kỹ thuật: TSF/IMK/IBus không vẽ underline vào edit control mà không sửa format | `Fassist-4` |
| D5 | Thứ tự: **Mode 1 (gợi ý) → Mode 2 (tự sửa cả câu)** | Auto-sửa sai là mất niềm tin | user |
| D6 | Tên tính năng: **"Trợ lý ngữ pháp"** | user | user |
| D7 | Auto-fix **chỉ luật objective**; luật mơ hồ chỉ gợi ý | Giảm rủi ro sửa sai | expert |
| D8 | **Không đọc nội dung** app khác; chỉ dùng **shadow buffer** do ta tự commit | Không rò rỉ, không cần quyền đọc | expert (xem D12 về vị trí caret) |
| D9 | V1 **không hỗ trợ tiếng Anh** → giữ D1 tuyệt đối | Không có engine EN | expert |
| D10 | **Không sinh tiến trình mới**; không log; không network | `antivirus-false-positive.md` §8 (rủi ro "tiến trình ẩn giữ text") | expert |
| **D11** | Tách **điểm kiểm tra** (mọi trigger) vs **điểm hiển thị** (chỉ kết thúc câu / idle ≥ 700 ms / `Ctrl+Shift+G`) | Trigger space sẽ **spam UI** khi gõ thường | `Fassist-14` |
| **D12** | Áp sửa = **Backspace+Type trên phần ta vừa gõ**, có **precondition**: caret ở cuối câu (chỉ đọc *vị trí*) + câu do ta gõ liên tục; **không xác minh được → không sửa** (fail-safe) | Sửa text đã commit bằng ITfRange/AX/AT-SPI rủi ro sửa nhầm vị trí | `Fassist-13` |
| **D13** | UI = **candidate list chuẩn của IME** (IBus/Fcitx5 có sẵn, macOS IMK có sẵn, Windows TSF phải implement `ITfCandidateListUI`); overlay tự vẽ chỉ là fallback Windows | Overlay **không khả thi ở Linux/Wayland** (fcitx5 = C++ không GTK) | `Fassist-15` |
| **D14** | `assist` là crate **nội bộ**, expose qua crate `ffi` (`textvn_ffi.dll` sẵn có) → **không artifact phát hành mới**; nếu cần struct mới → `assist_result_v1` + bump `IME_ABI_VERSION` + `textvn-cli verify` + header | Tránh nhân bản packaging/ABI | `Fassist-16` |
| **D15** | Từ điển = **FST mmap** (không parse) — load < 10 ms, RSS dùng chung page cache | TIP nạp vào **mọi process** → parse 3 MB mỗi app là không chấp nhận được | `Fassist-18` |
| **D16** | Luật **L0**: bỏ qua câu không phải tiếng Việt (heuristic tỷ lệ từ có dấu) + **L6 bỏ qua khi `auto_capitalize=true`** | Không báo sai cho câu tiếng Anh; tránh trùng chức năng với config sẵn có | `Fassist-19`, `Fassist-20` |
| **D17** | **Quy tắc clean-room** bằng văn bản: chỉ đọc Harper để hiểu ý tưởng; mọi file viết từ đầu; lấy code → attribution Apache-2.0 + NOTICE/REUSE | "Mượn kiến trúc" không được trôi thành copy code | `Fassist-21` |
| **D18** | **3 spike chặn** trước P4-3/P4-4: (S1) TSF mở candidate list sau commit? (S2) đọc vị trí caret 3 OS? (S3) khảo sát nguồn từ điển VN | 3 giả định lớn chưa kiểm chứng | `Fassist-29`, `Fassist-22` |
| **D19** | Tiêu chí chất lượng: **recall ≥ 80%**, **FPR ≤ 2%**, **2 người đánh giá độc lập** 100 câu (thống nhất ≥ 80%) | "Cảm thấy ổn" không đo được | `Fassist-27` |
| **D20** | **Kill-switch 1 bước** (`assist.mode=off` + tray toggle) + **auto-disable streak** + **không thu thập metrics** nào | UI phiền → người dùng bỏ; privacy | `Fassist-24`, `Fassist-25`, `Fassist-30` |
| **D21** | **D1 viết lại đúng tiền đề**: (a) English-only ở mọi tầng; (b) API churn ~hàng tháng → vendor = pin + nợ nâng cấp; (c) chi phí resident khi nhúng TIP in-process (deps `zip`/`boxcar` + FST 790 KB) → cần **S4a/S4b** đo; (d) ed2024 → MSRV ≥ 1.85. **Không** dùng lý do "crates.io stale" (đã chứng minh sai) | `Fassist-32` (cross-review) |
| **D22** | v1 **không dùng `SelectionReplace`** để áp sửa — chỉ **Backspace+Type** (A2/A3) theo D12; `SelectionReplace` chỉ ghi chú là cơ chế có sẵn trong `strategy` | `Fassist-33` |
| **D23** | Biên gọi: `windows-tsf` (Rust) gọi crate `assist` **trực tiếp**; macOS/Linux (Swift/C/C++) qua **C-ABI `ime_assist_*`** trong crate `ffi` + header `textvn_ffi.h` + `textvn-cli verify` chống drift | `Fassist-34` |
| **D24** | Buffer câu nằm trong **core** (`Engine.sentence`, Unicode) — **refactor trước**: gom 13 call site `recent_replace` → 2 helper (`recent_push`/`recent_pop`), rồi mới thêm mirror; tách khỏi `recent` (glyph, 64) | `Fassist-35` |
| **D25** | Tài nguyên nặng dùng **`OnceLock<Arc<…>>`** (immutable-after-init → không vi phạm "no global **mutable** state"); **S4a** đo per-process RSS/binary/p95 (gate: RSS ≤ +5 MB, binary ≤ +3 MB, p95 ≤ 2 ms) **chặn P4-4**; **S4b** (harper-core/EN) = v2, **không** chặn v1 | `Fassist-36` |
| **D26** | Buffer câu chỉ tồn tại **khi `assist.mode != off`** (feature flag trong config) + cap **256** ký tự; không dùng chung buffer với `recent` | `Fassist-38` |
| **D27** | `assist` trả offset theo **char Unicode**; **số ký tự cần xóa do core tính** (`Engine::glyph_len_of` — `emit()` đã encode sang bảng mã xuất); `Fix` **không** mang số xóa | `Fassist-39` |
| **D28** | `Issue` có **`confidence: u8`** (0–100) ngay từ P4-1 (thêm sau = phá API) | `Fassist-41` |
| **D29** | Biên FFI chuẩn hoá **UTF-16 code unit** (khớp TSF/AX); crate `assist` giữ char + helper đổi | `Fassist-42` |
| **D30** | UI theo OS: **Windows = overlay tự vẽ** (bỏ `ITfCandidateListUI` ở v1) · **Linux = candidate list của engine** (IBus/Fcitx5 API có sẵn) · **macOS = toast** (IMK server vẽ candidate list, không phải app) | S1 (spike) + giới hạn Wayland + kiến trúc IMK |
| **D31** | Buffer câu trong `core` **luôn bật**, cap **256 ký tự** (≈ 1 KB/process) — **không** cần feature flag trong ABI | Đơn giản hơn D26 (config flag ⇒ đổi ABI `EngineOptions` + đồng bộ 3 OS); 1 KB không đáng kể |
| **D32** | Kiến trúc **Provider**: `assist` giữ trait + **luật của ta**; **glue OS** (Windows `ISpellCheckerFactory` · macOS `NSSpellChecker` · Linux `libspelling`) nằm ở biên FFI/adapter theo D23 | `assist` giữ 0-dep & portable; OS-specific code chỉ 1 nơi/OS |
| **D33** | **OS-first**: dùng checker OS khi hỗ trợ `vi`/`vi-VN`; **FST của ta = fallback** | Chất lượng + user dictionary + footprint tốt hơn (F67) |
| **D34** | **Không trùng lặp app**: app có spellcheck native → assist chỉ làm **ngữ pháp câu** (per-app `appdb`) | Ít nhiễu, ít CPU (F72) |
| **D35** | **L0 dùng OS** trước heuristic (`NLLanguageRecognizer` · ICU · locale/MUI) | Thay đoán bằng dữ liệu hệ thống (F68) |
| **D36** | Provider OS gọi **bất đồng bộ, timeout ≤ 50 ms, fail-open** (provider có thể out-of-proc) | Không chặn input path (F70) |
| **D37** | Từ điển FST **vẫn ship** (fallback) + attribution GPL đầy đủ | Windows mặc định không có `vi-VN` (F67) |
| **D38** | **Spec hiển thị overlay = `textvn-cli assist show "<câu>"`** (câu + vạch `^` đúng span char Unicode + thông điệp tiếng Việt + gợi ý) — UI lane (tray/overlay) lấy làm chuẩn hiển thị + kiểm thử thủ công không cần dựng UI | Đồng bộ 3 OS dễ hơn (P4-0 §6) |
| **D39** | **v1 KHÔNG phát hành từ điển `hunspell-vi`** (license gốc không nêu phiên bản / không có "or later" ⇒ GPL-2.0-only sẽ **không tương thích GPL-3.0**) → dùng **OS spellchecker** (D33) + luật objective; crate `dict` (FST/mmap) giữ làm **hạ tầng v2** chờ nguồn license sạch (permissive/CC0). Installer đã loại `data/assist/*.dic` khỏi gói (ADR-014) | `P4-9-REVIEW-DICT.md` (Fassist-98…103) |

> Chi tiết nghiên cứu OS-first: `P4-4-OS-NATIVE-ASSIST.md`.

### 3.2 Ràng buộc bắt buộc
- Perf theo `PLAN.md` §5.5: `ime_key` < 0.5 ms p99 · adapter+strategy < 3 ms p99 · end-to-end < 10 ms p99 · RSS < 30 MB · không wake-idle spike > 5 ms.
- Check ngữ pháp **không chặn** input path: chạy sau khi trigger đã xử lý; vượt budget → **bỏ qua im lặng** (fail-open).
- Không ghi text người dùng ra log/disk/crash-dump (S2) — kể cả khi bật trace.
- Không đọc nội dung app khác (D8); chỉ đọc **vị trí con trỏ** khi cần precondition cho D12.

---

## 4. Hai chế độ — đặc tả chi tiết (sau vòng 3–5)

### 4.1 Điểm kiểm tra vs điểm hiển thị (D11)
| Mục | Hành vi |
|---|---|
| **Điểm kiểm tra** (silent) | Mỗi khi gõ `space` / `,` / `.` / `!` / `?` / `;` / `:` **sau khi text đã commit** → kiểm tra **câu vừa gõ** (từ dấu câu trước → dấu câu vừa gõ, ≤ 500 ký tự) từ **shadow buffer** |
| **Điểm hiển thị** (UI) | Chỉ khi: (a) dấu **kết thúc câu** `.` `!` `?`, **hoặc** (b) user **ngừng gõ ≥ 700 ms**, **hoặc** (c) `Ctrl+Shift+G` (gọi lại) → mới hiện candidate/overlay |
| **Kết quả im lặng** | Câu sạch → không hiện gì. Lỗi mơ hồ + Mode 1 → hiện. Lỗi objective + Mode 2 → sửa **im lặng** + toast nhỏ |
| **Bỏ qua** | App không phải ô nhập (terminal/IDE) · đang compose · `assist.mode=off` · câu < 3 từ · câu bị mask (URL/email) · câu không phải tiếng Việt (L0) |

### 4.2 Mode 1 — "Trợ lý ngữ pháp: gợi ý" (P4-1…P4-3)
- Hiển thị: **candidate list chuẩn của IME** (D13) — 1 dòng gợi ý đầu + danh sách lỗi; **overlay tự vẽ** chỉ là fallback ở Windows.
- Nội dung: câu vừa gõ + lỗi **gạch chân đỏ trong candidate panel** + thông điệp tiếng Việt ngắn ("Lặp từ: \"của của\"").
- Phím: `↑/↓` lỗi · `Enter` áp dụng gợi ý · `Alt+1..3` chọn gợi ý · `Esc` bỏ · `Ctrl+Shift+G` gọi lại.
- **Áp dụng gợi ý = A2** (xem §4.4) — Backspace+Type cho **từ** lỗi, có precondition.

### 4.3 Mode 2 — "tự sửa cả câu" (P4-5)
- Cùng điểm kiểm tra; **chỉ luật objective** (L1/L2/L11, L3, L4, L5, L6, L8) được sửa tự động → áp dụng **cả câu** (nhiều lỗi cùng lúc).
- Lỗi mơ hồ (L7/L9/L10/L12) → vẫn hiện candidate như Mode 1, **không** tự sửa.
- Toast "Đã sửa 2 lỗi" + `Ctrl+Z` (giữ 5 s) · `assist.auto_fix` mặc định `off`, bật `safe` = chỉ objective.
- Precondition D12 áp dụng cho **cả câu**: chỉ sửa khi caret xác minh được ở cuối câu.

### 4.4 Ba mức áp dụng (thay cho "sửa bằng selection replace" — rủi ro map vị trí)
| Mức | Phạm vi | Cơ chế | Precondition | Pha |
|---|---|---|---|---|
| **A1** | Không áp dụng | Chỉ gợi ý | — | P4-1 |
| **A2** | 1 từ lỗi | **Backspace+Type** (xoá đúng số ký tự ta vừa gõ rồi gõ lại từ đúng) | caret ở cuối từ + từ do ta gõ | P4-3 |
| **A3** | Cả câu | **Backspace+Type** cho cả câu | caret ở cuối câu + câu do ta gõ liên tục | P4-5 |

> Vì sao không dùng "selection replace" (đã có sẵn trong `strategy`): nó cần **biết vị trí text đã commit trong app** — mà app có thể tự format/auto-correct → lệch vị trí → sửa nhầm. Backspace+Type chỉ dựa vào **số ký tự ta tự gõ** → an toàn hơn, và nếu app đã can thiệp thì `Ctrl+Z` + fail-safe.
>
> **D22 (cross-review):** v1 **không dùng `SelectionReplace`** — nó chỉ là ghi chú "cơ chế có sẵn trong `strategy`", không đưa vào kế hoạch. **D27:** số ký tự cần xóa **do core tính** (`Engine::glyph_len_of` — `emit()` đã encode sang bảng mã xuất: 1 chữ Việt = 2 ký tự ở VNI-Windows/Unicode tổ hợp), `Fix` chỉ mang vùng lỗi trong **Unicode**.

### 4.5 Precondition (D12) — chi tiết
| Kiểm tra | Nguồn (chỉ *vị trí*, không đọc nội dung) | Nếu không kiểm được |
|---|---|---|
| Caret ở cuối câu vừa gõ? | Windows: `ITfContext::GetSelection` / `GetGUIThreadInfo`; macOS: `AXRange`; Linux: AT-SPI `TextCaret` | **Không sửa** (chỉ gợi ý) |
| Câu vừa gõ là do ta gõ liên tục? | Shadow buffer của adapter (đếm chữ ta commit) | **Không sửa** |
| Có gì khác được gõ xen kẽ không? | Shadow buffer (ký tự lạ → app tự chèn → không khớp) | **Không sửa** |

---

## 5. Bộ luật tiếng Việt

### 5.1 V1 — objective (đều auto-fix được)
> **Hợp đồng với config (D16):** crate `assist` **không** đọc config (0-dep) → nó **luôn** trả về issue L6; **lớp adapter** (chỗ có config) loại L6 khi `auto_capitalize=true` để không trùng chức năng.
> **Điểm tin cậy (D28):** mọi `Issue` có `confidence: u8` (0–100) để v2 lọc luật mơ hồ mà **không phá API**.
> **Đếm (Fassist-44):** v1 = **6 module luật / 7 mã luật** (L1+L2+L11 gộp trong `duplicate_word`).
| Mã | Luật | Module | Auto-fix | V1 |
|---|---|---|---|---|
| **L0** | Bỏ qua câu **không phải tiếng Việt** (heuristic tỷ lệ từ có dấu) | `language.rs` | — (bỏ qua) | ✓ (D16) |
| L1/L2/L11 | Lặp từ ("tôi tôi", "của của", "rất rất") | `duplicate_word.rs` | ✅ | ✓ |
| L3 | Thừa khoảng trắng trước dấu câu | `space_before_punct.rs` | ✅ | ✓ |
| L4 | Thiếu khoảng trắng sau dấu câu | `space_after_punct.rs` | ✅ | ✓ |
| L5 | Khoảng trắng kép | `double_space.rs` | ✅ | ✓ |
| L6 | Chữ thường đầu câu — **bỏ qua nếu `auto_capitalize=true`** | `capitalize_sentence.rs` | ✅ | ✓ |
| L8 | Sai vị trí dấu ("nghiã"→"nghĩa") — bảng curated ~12 mục | `diacritic_typo.rs` | ✅ | ✓ |
| **L9** | Lặp từ ở **ranh giới câu** — chỉ khi từ lặp là **từ chức năng** (F91) | `sentence_repeat.rs` | ❌ gợi ý | ✓ (F90) |
| **L10** | Phủ định kép "không có … không" | `double_negative.rs` | ❌ gợi ý | ✓ (F90) |
| **L12** | Dấu phẩy ở cuối câu | `comma_at_end.rs` | ❌ gợi ý | ✓ (F90) |
| L7 | Thiếu dấu chấm cuối câu khi gõ Enter | — | ❌ gợi ý | v2 |
| L9a/L10a/L12a | Ngữ pháp học thuật (POS, cụm từ) | — | ❌ | v3 |
| L7/L9/L10/L12 | Thiếu dấu chấm cuối câu · lặp từ ở ranh giới câu · phủ định kép · dấu câu sai | — | ❌ gợi ý | v2 |
| — | Từ điển: sai chính tả từ + gợi ý từ (edit distance âm tiết) | `spell.rs` | gợi ý | **P4-4** |

### 5.2 Từ điển (P4-4) — chỉ bắt đầu sau spike S3 (D18)
- **Nguồn**: ≥ 3 nguồn mở khảo sát (license + kích thước + chất lượng) → chọn nguồn **permissive/ODbL**; **không** dùng nguồn license không rõ; ghi nguồn vào `docs/compliance/assist-dictionary.md` + `REUSE.toml`.
- **Dạng lưu**: FST (mmap, D15) + tần suất âm tiết; gợi ý bằng edit distance trên **âm tiết**.
- **Kích thước**: ≤ 3 MB · load (mmap) < 10 ms · RSS tăng ≤ 20 MB (dùng chung page cache).
- **Báo động giả**: chỉ báo khi từ **không có** trong từ điển **và** không khớp từ nào (edit distance ≤ 1 âm tiết) **và** không phải tên riêng/viết tắt/số.

---

## 6. Giao diện người dùng (Design view)

| Thuộc tính | Yêu cầu |
|---|---|
| Nơi hiển thị | **Candidate list chuẩn của IME** (D13): Windows `ITfCandidateListUI` (phải implement), macOS IMK input controller (đã có), Linux IBus/Fcitx5 candidate list (đã có API) — panel hiện **cạnh con trỏ** → đúng ý D4 |
| Fallback | Toast/tray notification (P4-2, và mặc định nếu IME không hỗ trợ candidate) |
| Không chiếm focus | Windows: cửa sổ không activate; IBus/Fcitx5: panel chuẩn không chiếm focus; IMK: input controller chuẩn |
| Tự tắt | Click ra ngoài · 8 s không tương tác · app đổi focus · gõ tiếp · `Esc` |
| Nội dung | Câu vừa gõ + lỗi gạch chân đỏ + 1–3 gợi ý + thông điệp tiếng Việt ngắn |
| A11y | Nhãn tiếng Việt · bàn phím đầy đủ · dark mode · không đưa nội dung câu vào accessibility tree (tránh app khác đọc được) |
| Kill-switch | `assist.mode=off` + toggle trong tray/Settings + auto-disable streak (D20) |

---

## 7. Cấu hình (schema)

```jsonc
"assist": {
  "mode": "off",              // off | highlight | autocorrect
  "auto_fix": "off",          // off | safe      (chỉ luật objective)
  "language": "vi",           // vi | en | auto  (v1: chỉ vi)
  "trigger_keys": ["space", ",", "."],
  "display": {                // điểm hiển thị (D11)
    "show_on_sentence_end": true,
    "idle_ms": 700,
    "show_on_comma": false
  },
  "max_sentence_chars": 500,
  "per_app": { "notepad": true, "code.exe": false }
}
```

**Đồng bộ bắt buộc (AGENTS.md):** `schemas/config.v1.schema.json` + crate `config` (Rust) + `ConfigModel.swift` (macOS) + `settings_model.c` (Linux) + test Swift/C liên quan + `docs/release/ui-spec.md` + `docs/release/parity-checklist.md`.

---

## 8. Kế hoạch theo pha (sau vòng 3–5)

| Pha | Task | Nội dung & file | Acceptance | Ước lượng |
|---|---|---|---|---|
| **P4-0** | AS-000 | Tài liệu: `P4-0` (plan), `P4-1` (đánh giá), ADR-012, INDEX, PLAN §2.3 | Review 3 vòng; CI xanh; graphify | ✅ xong |
| **P4-1** | AS-001…006 | crate `assist/` (tokenize/mask/rules/fix) + **L0** (F40) + **`confidence`** (F41/D28) + `corpus/assist/*.ass` (đặc tả TSV — F43) + test tích hợp + `textvn-cli assist check` | `cargo test -p textvn-assist` xanh · corpus ≥ 60 câu · clippy+fmt sạch · **0 dep, 0 network, 0 I/O** · recall ≥ 80% / FPR ≤ 2% trên corpus | 2 tuần |
| **P4-1b** | AS-007…010 | **3 spike chặn (D18)**: S1 TSF candidate list sau commit · S2 đọc caret 3 OS · S3 khảo sát từ điển VN | Mỗi spike có kết luận **viết ra** + bằng chứng (log/screenshot) | ✅ **xong** → `P4-3-SPIKES.md` |
| **P4-1b+** | AS-011 | **S4a** đo per-process RSS/binary/p95 của tài nguyên assist (chặn P4-4) + **S4b** khảo sát `harper-core`/EN (**v2, không chặn**) | S4a: RSS ≤ +5 MB, binary ≤ +3 MB, p95 ≤ 2 ms (đo trên Windows thật) | ✅ **xong** (S4a: RSS 8,86 MB · exe 993 KB · 0,52 ms/câu) |
| **P4-1c** | AS-012…014 | Patch docs từ cross-review: Fassist-32 (tiền đề) · Fassist-33/34 (wording) · Fassist-37 (verified-ops) + ADR-013 | File này + `P4-0`/`P4-1`/`ADR-012`/`verified-ops` cập nhật, CI xanh | ✅ xong |
| **P4-2a** | AS-020 | **Refactor core** (D24): gom 13 call site `recent_replace` → 2 helper `recent_push`/`recent_pop` + thêm `Engine.sentence` (Unicode ≤256, chỉ bật khi `assist.mode != off` — D26) + `Engine::glyph_len_of` (D27) | Không còn chỗ sửa buffer trực tiếp; `core` test xanh; không đổi hành vi gõ | 1 tuần |
| **P4-2** | AS-021…025 | FFI `ime_assist_*` (D23) + trigger + **toast tray** + kill-switch + `OnceLock` share (D25) | Gõ thật: câu lỗi → toast; câu sạch → im lặng; không tăng trễ > 5 ms; tắt được 1 bước | 2–3 tuần |
| **P4-3** | AS-030…036 | **Candidate list Windows** (`ITfCandidateListUI`) + **A2** (sửa 1 từ) + precondition | Sửa đúng vị trí ở Notepad/Word/Chrome/VSCode · `Ctrl+Z` hoàn tác · không chiếm focus | 3 tuần |
| **P4-4** | AS-040…046 | Từ điển VN (FST mmap) + gợi ý từ + L0 | ≤ 3 MB · mmap < 10 ms · FPR ≤ 2% · license audit đầy đủ | 2–3 tuần |
| **P4-5** | AS-050…056 | **Mode 2 / A3** (tự sửa cả câu) + toast + undo + `assist.auto_fix` UI | Chỉ luật objective bị sửa · precondition bắt buộc · mọi thay đổi undo được | 2 tuần |
| **P4-6** | AS-060…068 | **Parity 3 OS**: macOS (IMK) + Linux (IBus/Fcitx5) + config/UI 3 OS | Cùng corpus pass 3 OS · `ui-spec`/`parity-checklist` cập nhật | 3–4 tuần |

Mỗi pha: commit list tường minh → push → CI xanh → `graphify update .` → commit → push (G5). Không bắt đầu P4-3/P4-4 khi **chưa có kết luận spike** (D18).

---

## 9. Kiểm thử & DoD (QA)

| Tầng | Nội dung |
|---|---|
| **Unit** | Mỗi luật 1 file test (kể cả case "gần đúng" phải **không** báo) + test `apply_auto_fixes` (char-space) |
| **Corpus** | `corpus/assist/*.ass` (câu + kind mong đợi) — chạy CI 3 OS; ≥ 60 câu ở v1, ≥ 200 ở P4-4 |
| **Đo chất lượng** | recall ≥ 80% · FPR ≤ 2% · 2 người đánh giá độc lập 100 câu (thống nhất ≥ 80%) (D19) |
| **Fuzz** | Input rác/byte lạ vào `assist::check` → **không panic** (crate `fuzz` sẵn có) |
| **Perf** | check câu ≤ 1 ms (không từ điển) / ≤ 3 ms (có từ điển) · RSS ≤ +20 MB · không wake-idle spike > 5 ms |
| **Manual** | 8 app: Notepad, Word, Chrome (textarea + contenteditable), VSCode, Teams/Slack, Terminal (phải tắt), app elevated (phải tắt) · 3 OS |
| **Bằng chứng DoD** | Mỗi task kèm: output CLI + screenshot + log **không chứa text** (D20) |
| **Hồi quy** | Không phá: Telex/VNI, undo, macro, emoji, per-app state, `auto_capitalize` (đã có), `spellcheck` (UI 3 OS) |

---

## 10. Rủi ro & giảm thiểu (cập nhật sau vòng 3–5)

| # | Rủi ro | Mức | Giảm thiểu |
|---|---|---|---|
| R1 | Sửa nhầm vị trí | **Cao** | D12 (Backspace+Type + precondition + fail-safe) · A2 trước A3 |
| R2 | Auto-fix sai | **Cao** | D7 (chỉ objective) · mặc định tắt · toast + undo · auto-disable streak |
| R3 | **UI spam** khi gõ thường | Cao | D11 (điểm kiểm tra ≠ điểm hiển thị) |
| R4 | Overlay **không khả thi ở Linux/Wayland** | Cao | D13 (candidate list chuẩn của IME) |
| R5 | **Shadow buffer chưa tồn tại ở runtime** | Trung bình | P4-2 tạo mới; ước lượng đã tăng |
| R6 | Tăng độ trễ gõ / RSS | Trung bình | Budget §3.2 · fail-open · từ điển mmap (D15) |
| R7 | False positive từ điển | Trung bình | Ngưỡng edit distance + heuristic · đo FPR · tắt được từng luật |
| R8 | Rủi ro AV/privacy | Trung bình | D10 (không tiến trình mới, không log, không network) · ghi vào `antivirus-false-positive.md` §8 khi có code |
| R9 | Lệch parity 3 OS | Trung bình | Sync 4 chỗ + test Swift/C + `ui-spec` |
| R10 | Bản quyền từ điển | Trung bình | Spike S3 trước · audit từng nguồn · `docs/compliance/` |
| R11 | **Trôi thành copy code Harper** | Trung bình | D17 (clean-room rule) + ADR-012 |
| R12 | Phình phạm vi | Trung bình | Roadmap chỉ luật objective ở v1 |
| R13 | ABI/đóng gói | Thấp | D14 (không artifact mới) · nếu cần struct mới thì bump ABI + `verify` |

---

## 11. Audit

### 11.1 Vòng 2 — "Nhất quán & Sẵn sàng" (đã xong ở v1)
| ID | Mức độ | Finding | Trạng thái |
|---|---|---|---|
| Fassist-7 | minor | Cần task list có ID (`P4-TASKS.md`) | còn mở → tạo ở P4-1 |
| Fassist-8 | minor | Chưa có ADR cho quyết định "không vendor Harper" | **fixed** → ADR-012 (`docs/adr/README.md`) |
| Fassist-9 | minor | `00-INDEX.md` §1 chưa liệt kê Phần 4 | fixed @604434c |
| Fassist-10 | minor | `PLAN.md` §2.3 chưa trỏ Phần 4 | fixed @604434c |
| Fassist-11 | minor | `antivirus-false-positive.md` §8 chưa có dòng về Assist | còn mở → bổ sung khi có code (P4-2) |
| Fassist-12 | minor | Thiếu checklist manual overlay 3 OS | còn mở → P4-6 |

### 11.2 Vòng 3–5 — review chuyên sâu (2026-10-20, 3 lượt: SA/QA → Audit → PO/BA/Design)

**Lượt 3 — Solution Architect + QA**

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| Fassist-13 | major | "Sửa text đã commit" (giả định của Mode 2) là rủi ro lớn nhất: sau commit TIP không giữ range → sửa cần map offset→vị trí thật (ITfRange/AX/AT-SPI) → dễ sửa nhầm | **D12**: Backspace+Type + precondition caret + fail-safe (§4.4–4.5) |
| Fassist-14 | major | Trigger space/,/. sẽ **spam UI** khi gõ thường | **D11**: tách điểm kiểm tra / điểm hiển thị (§4.1) |
| Fassist-15 | major | "Overlay cạnh con trỏ" **không khả thi ở Linux/Wayland** (fcitx5 = C++ không GTK) | **D13**: candidate list chuẩn của IME; overlay chỉ fallback Windows (§6) |
| Fassist-16 | minor | Crate mới nếu expose riêng → thêm artifact + sửa packaging/ABI | **D14**: crate nội bộ, expose qua `ffi` |
| Fassist-17 | minor | **Shadow buffer chưa tồn tại ở runtime** (`compose.rs` chỉ là mô phỏng test) | P4-2 tạo mới; ước lượng tăng (§8) |
| Fassist-18 | minor | Từ điển parse trong mỗi process (TIP nạp vào mọi app) = 100 ms+ mỗi app | **D15**: FST mmap |
| Fassist-19 | minor | Câu tiếng Anh sẽ bị báo sai khi v1 không hỗ trợ EN | **D16**: luật L0 bỏ qua câu không phải tiếng Việt |
| Fassist-20 | minor | Xung đột với `auto_capitalize` (mặc định `true`) | **D16**: L6 bỏ qua khi `auto_capitalize=true` |

**Lượt 4 — Audit (license / security / privacy / ops)**

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| Fassist-21 | major | "Mượn kiến trúc Harper" cần quy tắc văn bản, nếu không dễ trôi thành copy code (Apache-2.0 → attribution bắt buộc) | **D17** clean-room rule + ADR-012 |
| Fassist-22 | major | Nguồn từ điển VN **chưa khảo sát** → không cam kết lịch P4-4 | **D18** spike S3 chặn P4-4 |
| Fassist-23 | minor | UI hiển thị text người dùng → không log, không clipboard, tự xoá khi mất focus, không đưa vào accessibility tree | §6 (A11y) |
| Fassist-24 | minor | Không có auto-disable khi UI phiền | **D20** auto-disable streak (pattern đã có trong repo) |
| Fassist-25 | minor | Rủi ro "thu thập số liệu ngầm" | **D20**: không thu thập metrics nào, kể cả đếm |
| Fassist-26 | minor | Threat model: app độc hại có thể đẩy text vào ô nhập để khiến assist gợi ý/sửa → không nguy hiểm (không gửi đi đâu) nhưng phải ghi | Ghi vào `PLAN.md` §6.1 (STRIDE) ở P4-2 |

**Lượt 5 — PO / BA / Design (giá trị & đo lường)**

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| Fassist-27 | major | Thiếu tiêu chí "đủ tốt" (chỉ có FPR) → không chứng minh được giá trị | **D19**: recall ≥ 80% + FPR ≤ 2% + 2 người đánh giá (§2, §9) |
| Fassist-28 | minor | Thiếu yêu cầu **bằng chứng** cho từng task (DoD §6) | §9 (CLI output + screenshot + log sạch) |
| Fassist-29 | minor | 3 giả định kỹ thuật lớn chưa kiểm chứng (candidate list, caret, từ điển) | **D18**: 3 spike chặn (P4-1b) |
| Fassist-30 | minor | Thiếu kill-switch 1 bước + default off + rollback | **D20** + §7 (`mode: off`) |
| Fassist-31 | minor | Chưa nêu quan hệ với `spellcheck`/`auto_capitalize` sẵn có (tránh trùng chức năng) | §5.1 (L6) + §9 (hồi quy) |

### 11.3 Vòng 6 — CROSS-REVIEW (agent B, 2026-10-20) + phản hồi agent A
> Chi tiết đầy đủ + disposition từng finding: `P4-2-CROSS-REVIEW-2026-10-20.md`.

| Nguồn | ID | Mức | Tóm tắt | Xử lý |
|---|---|---|---|---|
| B | Fassist-32 | major | **Tiền đề "crates.io stale ~27 tháng" SAI** (thật: 2.13.0 / 2026-10-16) → 8 chỗ docs sai | **D21** + sửa 8 chỗ; `Fassist-2` đánh dấu *sai tiền đề → superseded* (không xóa, G6) |
| B | Fassist-33 | major | Mâu thuẫn Mode 1 (SelectionReplace) vs D12 | **D22**: v1 bỏ hẳn SelectionReplace; sweep wording |
| B | Fassist-34 | minor | Biên gọi: chỉ Rust adapter gọi crate được; Swift/C/C++ cần C-ABI | **D23**: `ime_assist_*` qua crate `ffi` + header + `verify` |
| B | Fassist-35 | major | Buffer câu nên ở **core** (glyph vs Unicode, 13 call site `recent_replace`) | **D24** + task **P4-2a** (refactor 13 → 2 helper trước) + ADR-013 |
| B | Fassist-36 | major | Chi phí resident per-process (IBus 1 instance/context, TIP in-proc) | **D25**: `OnceLock<Arc<…>>` + spike **S4a** (chặn P4-4) + **S4b** (v2) |
| B | Fassist-37 | minor | `cargo clippy --workspace` full parallel → hết paging file | `verified-ops.md` **B9/B10** |
| A | Fassist-38 | major | Đề xuất F35 tạo mâu thuẫn mới với F36 (buffer câu trong core × TIP in-proc) | **D26**: buffer chỉ bật khi `assist.mode != off`, cap 256 |
| A | Fassist-39 | major | Hợp đồng Unicode ↔ glyph chưa rõ (app nhận glyph, assist trả char) | **D27**: `Engine::glyph_len_of` tính số xóa; `Fix` không mang số xóa |
| A | Fassist-40 | major | L0 chốt ở D16 nhưng **chưa có trong skeleton** | Task P4-1: `rules/language.rs` + test |
| A | Fassist-41 | minor | `Issue` thiếu `confidence` → thêm sau = phá API | **D28**: thêm `confidence: u8` ngay ở P4-1 |
| A | Fassist-42 | minor | Quy ước offset xuyên biên chưa chốt (char vs UTF-16) | **D29**: chuẩn hoá UTF-16 ở biên FFI |
| A | Fassist-43 | minor | Định dạng corpus `.ass` chưa đặc tả | Task P4-1: TSV + mô tả trong `P4-TASKS.md`/`P0-4` |
| A | Fassist-44 | minor | "6 luật" vs 6 module / **7 mã luật** | Sửa wording (§5.1) |
| A | Fassist-45 | minor | Timeline "15–19 tuần nếu EN pass" mâu thuẫn D9 | **Giữ scope v1 = không EN**; S4b không tính timeline v1 |

### 11.4 Vòng 7–9 — review **sau khi implement P4-1** (2026-10-20)

> Quy trình bắt buộc: implement xong → review 3 lượt (SA/QA → Audit → PO/Design) → fix → mới commit.

**Lượt 7 — SA + QA (sau khi viết `assist/` + corpus + CLI)**

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-50** | major | **Panic khi slice byte bằng offset char**: L0 dùng `text[token.start..token.end]` → `panic` với tiếng Việt có dấu (đã tái hiện 19 test đỏ) | ✅ Sửa: tách `Vec<char>` trước khi slice. **Đây là bằng chứng thực tế cho Fassist-39/D27** → bổ sung luật: *mọi code đọc token phải dùng char-space, không slice byte* |
| **Fassist-49** | major | CLI dispatch ăn mất subcommand: `args[2..]` → `rest[0]` = `"check"` bị hiểu là nội dung câu → mọi lệnh đều fail | ✅ Sửa: `cmd_assist` tách subcommand `check` + 5 case smoke test (0/1 exit code đúng) |
| **Fassist-46** | minor | Thông điệp người dùng thiếu dấu tiếng Việt (chuẩn UI repo) — 6 message luat + CLI | ✅ Sửa toàn bộ ("Lặp từ", "Thừa khoảng trắng trước dấu câu", …) |
| **Fassist-47** | minor | CLI `--file` check **cả file** → L0 áp cho cả file ⇒ file song ngữ bị bỏ qua toàn bộ | ✅ Sửa: check **từng dòng** + báo số dòng |
| **Fassist-48** | minor | Corpus chỉ 30 câu < ngưỡng 60 trong plan | ✅ Mở rộng lên **69 câu** (đủ 6 loại lỗi) + nâng ngưỡng test |
| **Fassist-51** | minor | Thiếu test cho `check_sentence` (hoa đầu câu ở index 0) | ✅ Bổ sung test + 1 case corpus |

**Lượt 8 — Audit (license / security / privacy / DoD)**

| ID | Mức | Finding | Kết quả |
|---|---|---|---|
| **Fassist-52** | minor | Dependency mới? | ✅ **Không** — `assist` 0-dep; `cli` dùng `serde_json` đã có sẵn; `cargo deny check` **không lỗi** (chỉ warning allow-list thừa + `syn` trùng, có sẵn từ trước) |
| **Fassist-53** | minor | Rò rỉ text qua log/disk? | ✅ Không log, không ghi file, không network; `--fix` chỉ in ra stdout (ý đồ người dùng); corpus chỉ chứa câu mẫu, **không** có text người dùng |
| **Fassist-54** | minor | REUSE/Spdx cho file mới | ✅ Mọi `.rs` có header SPDX dòng 1; `corpus/*.ass` được `REUSE.toml` (`path = ["**"]`) cover |
| **Fassist-55** | minor | DoD #1–#4 (test · corpus · clippy/fmt · deny) | ✅ 34 unit + 3 integration test xanh · corpus 69 câu pass · clippy workspace 0 warning · fmt sạch · deny OK |

**Lượt 9 — PO / BA / Design (giá trị & UX)**

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-56** | minor | CLI chưa giới hạn số lỗi in ra (file rất dài → flood) | Chấp nhận v1 (in theo dòng); thêm `--max` ở v2 |
| **Fassist-57** | minor | `textvn-cli assist` chưa có trong `docs/developer-guide.md` | Cập nhật ở P4-2 (khi CLI có thêm subcommand) |
| **Fassist-58** | minor | Chưa đo recall/FPR thật (cần corpus ≥ 500 câu sạch) | Gate P4-4 (D19) — v1 chỉ đo trên 69 câu |

### 11.5 Kết luận các lượt review
- **Blocker/major đã xử lý trong plan:** 13/13 (vòng 3–5) + 6/6 (cross-review B) + 8/8 (A) + 2/2 (vòng 7) → **D11–D29** hoặc task chặn (spike).
- **Điểm yếu còn lại (chấp nhận có điều kiện):** chất lượng luật tiếng Việt phụ thuộc nguồn từ điển (P4-4) và độ chính xác của L0 (heuristic) → cần đo (D19) và tắt được từng luật.
- **Điều kiện để bắt đầu code:** D11–D29 được chấp nhận + Q1–Q5 đã trả lời (`P4-1` §9).
- **Sau P4-1:** đã có engine + corpus + CLI; còn 3 spike chặn (S1–S3) + S4a trước khi làm P4-3/P4-4.

### 11.6 Vòng 8–10 — review **sau khi làm 4 spike** (2026-10-20) → `P4-3-SPIKES.md`

**Lượt 8 — SA**: S1 kết luận "Windows dùng overlay tự vẽ" **mâu thuẫn D13** (candidate list 3 OS) → **D30** (bổ sung theo OS) · ước lượng P4-3 **giảm** (không phải implement `ITfCandidateListUI`).
**Lượt 9 — Audit**: nguồn từ điển **GPLv2** → FST sinh từ dữ liệu GPL là phái sinh → phải **attribution + `REUSE.toml`** (task P4-4) · S4b giữ D9, nếu v2 dùng `harper-core` → Apache-2.0 + NOTICE/REUSE.
**Lượt 10 — PO/QA/Design**: S4a đo trên **tiến trình CLI độc lập** ≠ TIP in-process → phải **đo lại trong TIP** ở P4-2 (giới hạn đo ghi rõ) · từ điển chỉ **6.642 từ đơn** → recall ≥ 80% (D19) **có thể không đạt** → cần đo thật + fallback (không từ điển → chỉ luật objective) · macOS dùng toast ≠ trải nghiệm 2 OS kia → ghi vào `parity-checklist` (P4-6).

→ **Fassist-59..64** (6 finding): 59 mâu thuẫn D13↔S1 (→D30) · 60 attribution GPL bắt buộc (P4-4) · 61 S4a đo trong CLI ≠ TIP (đo lại P4-2) · 62 từ điển từ đơn → recall có thể không đạt (đo + fallback) · 63 macOS toast lệch parity (P4-6) · 64 S4b giữ D9 (không sửa).

### 11.7 Vòng 11 — review **bản đồ 13 call site** (chuẩn bị P4-2a, 2026-10-20)
| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-65** | major | 2 call site có **hậu kỳ hậu-encode** (site 523 cắt `MAX_TEXT` **sau** `emit`) → đổi tham số sang Unicode dễ **lệch hành vi `recent`** (và hành vi gõ) | `record()` phải tái tạo đúng semantics (giữ `MAX_TEXT` trong `record`) · oracle = test `core` không đổi · recipe ở `P4-TASKS.md` |
| **Fassist-66** | minor | `recent.pop()` xoá 1 **glyph**; nếu glyph cuối là 1 chữ 2 byte (TCVN3/VNI-Windows) thì `sentence` phải xoá 1 **char Unicode** | Xác nhận **2 buffer tách bạch** (D24/D27) — không dùng chung |


---

## 12. Việc cần làm ngay

1. ✅ `P4-0` (file này) + `P4-1-harper-evaluation.md` + `P4-2-CROSS-REVIEW-2026-10-20.md` — xong.
2. ✅ `docs/adr/README.md` — ADR-012 (tiền đề đã sửa) + **ADR-013** (buffer câu ở core + share immutable).
3. ✅ `docs/00-INDEX.md` §1 + log; `PLAN.md` §2.3 (S1/S2/C1 → Phần 4); `verified-ops.md` B9/B10.
4. ✅ **P4-1** — crate `assist/`: 6 module luật objective (7 mã) + L0 + `confidence` + `apply_auto_fixes` (char-space) + corpus **69 câu** + test tích hợp + `textvn-cli assist check` (`--json/--fix/--sentence/--file`) → **34 unit + 3 integration test xanh**, clippy 0 warning, `cargo deny` OK. Bằng chứng + task: `P4-TASKS.md` (AS-001…006).
5. ⏳ **P4-1b** — spike S1 (TSF candidate list) · S2 (caret 3 OS) · S3 (từ điển VN) · **S4a** (per-process RSS — chặn P4-4) · S4b (EN, v2).
6. ⏳ **P4-2a** — refactor core: 13 call site `recent_replace` → 2 helper + `Engine.sentence` + `Engine::glyph_len_of` (ADR-013).
7. ⏳ **P4-2** — FFI `ime_assist_*` + trigger + toast + kill-switch.

---

## 13. Liên kết

- `P4-1-harper-evaluation.md` — đánh giá Harper + so sánh phương án + findings vòng 1
- `PLAN.md` §2.3 · §5.5 · §6.1 — backlog nguồn · perf budget · threat model
- `docs/01-AGENT-HANDBOOK.md` — DoD 7 mục + checklist review
- `docs/specs/antivirus-false-positive.md` §8 — chính sách kiến trúc hành vi (không giống malware)
- `docs/10-shared/P0-2-engine-ffi-contract.md` · `P0-3-config-preset-strategy.md` · `P0-4-test-and-corpus.md`
- `docs/release/ui-spec.md` · `parity-checklist.md` — spec UI 3 OS
