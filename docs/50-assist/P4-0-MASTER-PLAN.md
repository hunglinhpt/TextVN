# P4-0 — MASTER PLAN: Trợ lý ngữ pháp tiếng Việt (Assist) cho TextVN

- **Ngày:** 2026-10-20 · **Trạng thái:** Kế hoạch (chưa triển khai) · **Phần:** 4 — P4
- **Owner:** agent · **Review:** vòng 1 = `P4-1-harper-evaluation.md` · vòng 2 = §11 (dưới đây)
- **Đánh giá nguồn:** `P4-1-harper-evaluation.md` (Harper, Harper so sánh phương án, quyết định D1–D5)
- **Cross-ref:** `PLAN.md` §2.3 (S1 chính tả + gạch đỏ · S2 gợi ý từ · C1 AI local) · `PLAN.md` §5.5 (perf budget) · `docs/10-shared/P0-2-engine-ffi-contract.md` · `docs/10-shared/P0-3-config-preset-strategy.md` · `docs/10-shared/P0-4-test-and-corpus.md` · `docs/specs/antivirus-false-positive.md` §8 · `docs/specs/reference-parity.md` · `docs/01-AGENT-HANDBOOK.md` (DoD 7 mục)
- **Yêu cầu gốc (user 2026-10-20):** 2 chế độ — **highlight lỗi ngữ pháp** và **tự correct cả câu** sau khi bấm **dấu cách** hoặc **`,` / `.`**

---

## 1. Bối cảnh & mapping với PLAN hiện có

PLAN đã có 2 mục backlog liên quan nhưng **mức thấp hơn** yêu cầu mới:

| Backlog | Nội dung hiện tại | Khoảng cách với yêu cầu mới |
|---|---|---|
| `S1` | "Chính tả offline (âm tiết: âm đầu–vần–phụ âm cuối–than) + **gạch đỏ** trên mọi OS" | Chỉ bắt lỗi **từ**; chưa có **ngữ pháp câu**, chưa có **tự sửa cả câu** |
| `S2` | "Gợi ý từ (offline n-gram/trie) — không AI bắt buộc" | Chỉ gợi ý **từ**; chưa có **highlight lỗi câu** |
| `C1` | "Gợi ý AI **local** (Ollama) opt-in" | Để v2.x, **không** nằm trong đường chính (P2: không network) |

→ Phần 4 (P4) **nâng S1+S2** thành: *offline grammar assistant cho bộ gõ*, 2 chế độ, 3 OS, không network, không sinh tiến trình ẩn.

**Bản chất sản phẩm:** "Trợ lý ngữ pháp" = **kiểm tra câu vừa gõ** + **gợi ý sửa** + (v2) **tự sửa câu**. Không phải "spellcheck từ" (đó là `spellcheck` đã có trong config).

---

## 2. Mục tiêu & phiêu miêu

**Mục tiêu (v1, P4-1…P4-3):** khi người dùng gõ xong một câu (dấu cách / `,` / `.`), TextVN **phát hiện lỗi objective trong câu vừa gõ** và **gợi ý sửa** bằng overlay nhỏ cạnh con trỏ — **100% offline**, không log text, không đọc process khác.

**Không phải mục tiêu (v1):**
- Không phải "Grammarly đầy đủ" (ngữ pháp học thuật, POS, đọc cả tài liệu).
- Không sửa format/gạch chân bên trong field của app (xem `Fassist-4`).
- Không tự sửa khi luật không chắc chắn.
- Không gửi text ra ngoài máy (P2).

**Tiêu chí thành công (DoD sản phẩm):**
1. Câu có lỗi objective → overlay hiện đúng lỗi, đúng vị trí, gợi ý sửa đúng.
2. Câu sạch → **không** hiện gì (0 false positive trên corpus ≥ 500 câu sạch).
3. Không ảnh hưởng cảm giác gõ: không thêm độ trễ cảm nhận > 5 ms vào đường gõ.
4. Không rò rỉ: không log, không network, không file tạm chứa text.

---

## 3. Kiến trúc (Solution Architect view)

```
┌─ Adapter (3 OS) ───────────────────────────────────────────────────────────┐
│  windows-tsf (TIP)      macos-imk (IMK)        linux-ibus / linux-fcitx5   │
│  compose.rs: KeyKind    commit string         commit string               │
│  + shadow buffer (câu)  + shadow buffer        + shadow buffer             │
└───────────────┬──────────────────────┬──────────────────────┬────────────┘
                │ gọi trực tiếp (không IPC ở v1)                 │
        ┌───────▼──────────────────────────────────────────┐
        │  core::assist  (crate Rust mới — thuần, không I/O)│
        │  Document → Token(âm tiết/từ) → Rule → Issue      │
        │  + Suggestion (đã sửa) + Confidence + auto_fixable │
        └───────┬──────────────────────────────────────────┘
                │ kết quả (chỉ giữ trong RAM)
     ┌──────────▼───────────┐        ┌──────────────────────────┐
     │ strategy crate       │        │ Overlay UI (3 OS)         │
     │ ActionKind::Replace  │        │ Windows: HWND no-activate │
     │ SelectionReplace     │        │ macOS: NSPanel            │
     │ (tái dùng fix B1)    │        │ Linux: GTK overlay        │
     └──────────────────────┘        └──────────────────────────┘
```

**Quyết định kiến trúc (đã chốt ở P4-1):**

| # | Quyết định | Lý do |
|---|---|---|
| A-1 | `assist` là **crate Rust thuần** trong workspace (`core`-level, không I/O, không network, không `unsafe`) | test được, dùng lại 3 OS, không vi phạm P2/S2 |
| A-2 | **Không mở rộng IPC ở v1** — adapter gọi thẳng crate qua FFI hiện có | giữ nguyên hợp đồng FFI (`P0-2`); IPC chỉ khi cần gửi kết quả ra tray/overlay |
| A-3 | **Không sinh tiến trình mới** (không daemon, không service) | `docs/specs/antivirus-false-positive.md` §8: tiến trình ẩn giữ text = rủi ro heuristic AV; engine phải nằm trong adapter/TIP |
| A-4 | **Shadow buffer** = text ta tự commit trong câu hiện tại, chỉ trong RAM, xoá khi đổi focus | không cần đọc text app khác (không rò rỉ, không cần quyền) |
| A-5 | **Áp sửa = tái dùng `strategy`** (`ActionKind::Replace` + `SelectionReplace`) | đã có sẵn từ fix bug B1; không phát minh cơ chế mới |
| A-6 | **Overlay không chiếm focus**, tự tắt khi mất focus | không cướp focus khi người dùng đang gõ |

**Ràng buộc bắt buộc:**
- Độ trễ đường gõ giữ nguyên `PLAN.md` §5.5: `ime_key` < 0.5 ms p99, adapter+strategy < 3 ms p99, end-to-end < 10 ms p99, RSS < 30 MB, không wake-idle spike > 5 ms.
- Check ngữ pháp chạy **sau** khi trigger key đã xử lý xong → **không chặn** input path; nếu check vượt budget → **bỏ qua** (fail-open, im lặng).
- Không bao giờ ghi text người dùng ra log/disk/crash-dump (S2) — kể cả khi bật trace.

---

## 4. Hai chế độ (chi tiết hành vi)

### Mode 1 — Gợi ý / Highlight (P4-1…P4-3) ✅ *ưu tiên*

| Mục | Hành vi |
|---|---|
| **Trigger** | Người dùng gõ **dấu cách** hoặc `,` / `.` (v1: thêm `!` `?` `;` `:`), **khi không còn text đang compose** và app đang ở trạng thái VN |
| **Đơn vị kiểm tra** | **Câu vừa gõ xong** (từ dấu câu trước → dấu câu vừa gõ), tối đa 1 câu, ≤ 500 ký tự |
| **Kết quả** | Nếu có lỗi → **overlay nhỏ cạnh con trỏ**: câu vừa gõ, lỗi **gạch chân đỏ**, bên dưới là gợi ý |
| **Phím** | `↑/↓` lỗi trước/sau · `Enter` áp dụng gợi ý đang chọn · `Alt+1..3` chọn gợi ý thứ n · `Esc` bỏ · `Ctrl+Shift+G` gọi lại overlay cho câu vừa gõ |
| **Áp dụng gợi ý** | Tái dùng `SelectionReplace`: chọn đúng range lỗi trong field → thay bằng text sửa → **undo được** (Ctrl+Z) |
| **Không làm** | Không tự sửa; không hiện gì khi câu sạch; không hiện khi app không phải ô nhập (terminal/editor code) |
| **Cấu hình** | `assist.mode = "highlight" \| "off"`, `assist.language = "vi" \| "auto"` |

### Mode 2 — Tự sửa cả câu (P4-5) 🔜 *sau Mode 1*

| Mục | Hành vi |
|---|---|
| **Trigger** | Giống Mode 1 |
| **Tự động sửa** | Chỉ lỗi có `auto_fix = true` (objective, xem §6): lặp từ liên tiếp, khoảng trắng trước/sau dấu câu, khoảng trắng kép, chữ hoa đầu câu, sai dấu tiếng Việt kiểu `nghiã→nghĩa` |
| **Phần còn lại** | Lỗi mơ hồ → hiện overlay như Mode 1 (không tự sửa) |
| **Tin nhắn** | Toast nhỏ: "Đã sửa 2 lỗi" + `Ctrl+Z` để hoàn tác (giữ 5 s) |
| **An toàn** | Mặc định `assist.auto_fix = "off"`; bật "safe" chỉ gồm luật objective; **không bao giờ** tự sửa khi luật có điểm tin cậy < ngưỡng |
| **Giới hạn v1** | Chỉ sửa lỗi **trong câu cuối** (offset gần con trỏ) → tránh map sai vị trí (`Fassist-5`) |

### Ánh xạ sang hành vi hiện có

| Hành vi TextVN sẵn có | Liên quan |
|---|---|
| `auto_capitalize` (config) | trùng lớp với "hoa đầu câu" → **gộp**: assist lo phần *sau dấu câu*, engine lo phần *đầu câu đầu tiên* |
| `suggest` / `english_words` (config) | gợi ý từ hiện có (S2) → assist **mở rộng** sang gợi ý *câu* |
| `spellcheck` (config, mặc định `Should`) | trạng thái "nên báo lỗi" vs "báo sai" → assist dùng cùng ngữ nghĩa cho lỗi câu |
| Strategy `SelectionReplace` | dùng để áp sửa (fix B1) |
| `appdb` per-app | thêm "tắt assist cho app này" (terminal, IDE) |

---

## 5. Bộ luật tiếng Việt & từ điển

### 5.1 Luật v1 (objective — auto-fix được hoặc không)

| # | Luật | Ví dụ | `auto_fix` | Ghi chú |
|---|---|---|---|---|
| L1 | Lặp từ liên tiếp | "tôi tôi", "của của" | ✅ | bỏ 1 từ |
| L2 | Lặp giới từ liên tiếp | "đã đã", "sẽ sẽ" | ✅ | |
| L3 | Thừa khoảng trắng trước dấu câu | "xin chào ," | ✅ | |
| L4 | Thiếu khoảng trắng sau dấu câu | "xin chào,bạn" | ✅ | trừ `) ] } ' "` |
| L5 | Khoảng trắng kép | "a  b" | ✅ | |
| L6 | Chữ thường đầu câu | sau `. ! ?` + khoảng trắng | ✅ | bỏ qua nếu là mã/viết tắt |
| L7 | Thiếu dấu chấm cuối câu khi gõ Enter | "tôi đi học⏎" | ❌ (gợi ý) | cần biết người dùng kết thúc đoạn |
| L8 | Sai dấu tiếng Việt | "nghiã"→"nghĩa", "cuả"→"của" | ✅ | **tái dùng** bộ luật dấu sẵn có (`core/src/transform/diacritic_style.rs`) |
| L9 | Trùng từ ở ranh giới câu | "…đi. Đi tới…" | ❌ (gợi ý) | |
| L10 | Phủ định kép | "không có … không" | ❌ (gợi ý) | độ tin cậy trung bình |
| L11 | "rất" + tính từ lặp | "rất rất" | ✅ | |
| L12 | Dấu câu sau chữ cái | "tôi đi học," cuối câu | ❌ (gợi ý) | |

> **Nguyên tắc QA:** chỉ luật **objective** mới được `auto_fix`. Luật mơ hồ → gợi ý, không bao giờ tự sửa (giảm rủi ro mất niềm tin).

### 5.2 Từ điển tiếng Việt (P4-4)

| Mục | Kế hoạch |
|---|---|
| **Nguồn (đề xuất, phải audit license trước khi dùng)** | (a) từ điển âm tiết mở (kiểm tra giấy phép từng nguồn), (b) ngữ liệu có sẵn trong repo (`data/`, `corpus/`, `docs/`), (c) sinh từ từ điển Wikibooks/Wiktionary (**CC BY-SA → có nghĩa vụ attribution + chia sẻ** → ưu tiên nguồn permissive) |
| **Dạng lưu** | FST/bloom + tần suất (âm tiết → tần suất) để gợi ý bằng edit distance trên **âm tiết** |
| **Kích thước mục tiêu** | ≤ 3 MB (đóng gói trong bộ cài), load < 150 ms (cold), RSS tăng ≤ 20 MB |
| **Báo động giả** | Chỉ báo "sai chính tả" khi: từ **không** có trong từ điển **và** không khớp từ nào trong từ điển (edit distance ≤ 1 âm tiết) **và** từ không phải tên riêng/viết tắt (heuristic: viết hoa giữa câu, có số, ký hiệu) |
| **Audit bắt buộc** | Ghi nguồn + giấy phép từng nguồn vào `docs/compliance/` + `REUSE.toml`; **không** dùng nguồn license không rõ |

### 5.3 Che (mask) nội dung không nên kiểm tra

URL, email, đường dẫn, mã hex/base64, mã định danh (`@`, `_`, `#`), số thứ tự, văn bản trong ngoặc kép kèm ký tự lạ. (Mượn nguyên tắc `mask/` của Harper.)

---

## 6. Giao diện người dùng (Design view)

**Overlay (3 OS, cùng spec `docs/release/ui-spec.md`):**

| Thuộc tính | Yêu cầu |
|---|---|
| Vị trí | Ngay dưới/song song con trỏ, bám theo vị trí con trỏ (TSF caret rect / AX / AT-SPI) |
| Kích thước | ≤ 320×220 px, tự co theo nội dung |
| Hành vi focus | **Không chiếm focus** (Windows: `WS_EX_NOACTIVATE`; macOS: `NSPanel` non-activating; Linux: cửa sổ GTK không focus) |
| Tự tắt | Click ra ngoài · 8 s không tương tác · app đổi focus · gõ tiếp (theo lựa chọn) |
| Hiển thị lỗi | Gạch chân đỏ **trong overlay** (không sửa field) + nhãn lỗi tiếng Việt ngắn |
| Gợi ý | Danh sách 1–3 gợi ý, gợi ý đầu được chọn sẵn |
| Dark mode | Bắt buộc (theo theme OS) |
| A11y | Nhãn tiếng Việt, hỗ trợ bàn phím đầy đủ, không chỉ chuột |

**Vòng 1 (giảm rủi ro):** trước khi làm overlay thật, dùng **thông báo tray** (đã có sẵn) để khép vòng giá trị: "Câu vừa gõ có 1 lỗi: lặp từ" → sau đó mới xây overlay.

---

## 7. Cấu hình (schema)

```jsonc
"assist": {
  "mode": "off",              // off | highlight | autocorrect
  "language": "vi",           // vi | en | auto
  "auto_fix": "off",          // off | safe
  "trigger_keys": ["space", ",", "."],
  "min_confidence": 0.9,
  "max_sentence_chars": 500,
  "per_app": { "notepad": true, "code.exe": false }
}
```

**Đồng bộ bắt buộc (AGENTS.md):** `schemas/config.v1.schema.json` + crate `config` (Rust) + `ConfigModel.swift` (macOS) + `settings_model.c` (Linux) + test Swift/C liên quan + `docs/release/ui-spec.md` + `docs/release/parity-checklist.md`.

---

## 8. Kế hoạch theo pha (ước lượng 1 người)

| Pha | Nội dung | Xuất xưởng | Tiêu chí nghiệm thu (acceptance) |
|---|---|---|---|
| **P4-1** | Crate `assist/` skeleton + 6 luật objective (L1–L6, L8) + corpus câu tiếng Việt + CLI debug `textvn-cli assist check "<câu>"` | 2 tuần | `cargo test -p assist` xanh; corpus ≥ 200 câu (≥ 50 lỗi, ≥ 150 sạch) pass; clippy/fmt sạch; **0 network, 0 I/O** (audit bằng mắt + `cargo deny`) |
| **P4-2** | Shadow buffer trong adapter + trigger (space/`,`/`.`) + **thông báo tray** khi có lỗi (chưa overlay) | 2 tuần | Gõ thật trên Notepad/Chrome: câu lỗi → có thông báo; câu sạch → im lặng; không tăng độ trễ cảm nhận (> 5 ms) |
| **P4-3** | **Overlay Windows** (HWND no-activate) + **áp dụng gợi ý** qua `SelectionReplace` + undo | 2–3 tuần | Sửa đúng vị trí trong Notepad/Word/Chrome/VSCode; Ctrl+Z hoàn tác được; overlay không chiếm focus; perf p95 theo §3 |
| **P4-4** | Từ điển tiếng Việt (nguồn + audit license + pipeline build) + gợi ý từ (edit distance âm tiết) | 2–3 tuần | ≤ 3 MB; load < 150 ms; false-positive ≤ 2% trên corpus 500 câu sạch; `docs/compliance/` có nguồn + license |
| **P4-5** | **Mode 2 — tự sửa câu** (chỉ luật objective) + toast + undo + `assist.auto_fix` trong UI | 2 tuần | Bật "safe": chỉ luật objective bị sửa; mọi thay đổi undo được; không sửa khi luật mơ hồ |
| **P4-6** | **Parity 3 OS**: macOS (IMK + NSPanel) + Linux (IBus/Fcitx5 + overlay) + UI 3 OS + corpus 3 OS | 3–4 tuần | Cùng corpus pass 3 OS; UI giống nhau; `ui-spec`/`parity-checklist` cập nhật |

**Định nghĩa "xong phần":** theo `docs/01-AGENT-HANDBOOK.md` DoD 7 mục (test workspace xanh, corpus pass, clippy+fmt, `cargo deny`+REUSE, docs cập nhật, acceptance tick, CI 3 OS xanh) + 2/2 review trong `P4-REVIEW-LOG.md`.

---

## 9. Kiểm thử (QA)

| Tầng | Nội dung |
|---|---|
| **Unit** | Mỗi luật 1 file test riêng (`assist/tests/rules/*.rs`): positive + negative + case "gần đúng" phải **không** báo |
| **Corpus** | `corpus/assist/*.txt` (câu + expected issue), chạy trong CI 3 OS (như `textvn replay corpus/`) |
| **Fuzz** | Fuzz target mới: input ngẫu nhiên/byte lạ vào `assist::check` → **không panic** (crate `fuzz` đã có sẵn) |
| **Perf** | Gate p95: check 1 câu ≤ 1 ms (không từ điển) / ≤ 3 ms (có từ điển); RSS ≤ +20 MB; không wake-idle spike > 5 ms |
| **Manual** | Checklist app: Notepad, Word, Chrome (textarea + contenteditable), VSCode, Teams/Slack, Terminal (phải **tắt**), app elevated (phải **tắt**) |
| **Hồi quy** | Không được phá: gõ Telex/VNI, undo, macro, emoji, per-app state, `auto_capitalize` (đã có) — có corpus hồi quy riêng |

---

## 10. Rủi ro & giảm thiểu

| # | Rủi ro | Mức | Giảm thiểu |
|---|---|---|---|
| R1 | Sửa nhầm vị trí trong app (offset ≠ vị trí thật) | **Cao** | v1 chỉ sửa lỗi **cuối câu**; app không hỗ trợ range → bỏ qua; luôn có undo; test trên 8 app |
| R2 | Auto-fix sai → mất niềm tin | **Cao** | chỉ luật objective; mặc định **tắt**; toast + undo; thống kê số lỗi đã sửa |
| R3 | Overlay che nội dung / cướm focus | Trung bình | không chiếm focus; tự tắt; không hiện khi đang compose |
| R4 | False positive từ điển | Trung bình | ngưỡng edit distance + heuristic tên riêng; đo FPR trên corpus sạch; tắt được từng luật |
| R5 | Tăng độ trễ gõ / RSS | Trung bình | budget §3; check fail-open; từ điển lazy-load; gate perf trong CI |
| R6 | Rủi ro AV/privacy (text người dùng trong bộ nhớ tiến trình) | Trung bình | không sinh tiến trình mới; không log; không network; không đọc app khác; ghi trong `docs/specs/antivirus-false-positive.md` §8 (bổ sung mục Assist) |
| R7 | Lệch parity 3 OS (config/UI) | Trung bình | bắt buộc sync 4 chỗ + test Swift/C (AGENTS.md) |
| R8 | Phình phạm vi (làm luôn ngữ pháp học thuật) | Trung bình | roadmap chỉ 6 luật objective ở v1; phần mơ hồ để sau, có backlog riêng |
| R9 | Bản quyền từ điển/ngữ liệu | Trung bình | audit từng nguồn trước khi đóng gói; ghi `docs/compliance/` + `REUSE.toml` |
| R10 | Phụ thuộc ngoài (crates.io) làm tăng bề mặt | Thấp | `assist` chỉ dùng crate đã có sẵn trong workspace (`serde`?) — ưu tiên 0 dep |

---

## 11. Audit vòng 2 — "Nhất quán & Sẵn sàng" (checklist + findings)

| Mục 5W1H | Kết quả kiểm tra |
|---|---|
| **What** | Đối chiếu §1–§10 với yêu cầu user (2 chế độ · trigger space/,/. · highlight · correct cả câu) → **đủ**; với `PLAN.md` S1/S2 → **không trùng lặp** (nâng cấp, có mapping ở §1) |
| **Why** | Mỗi hạng mục đều có lý do + tiêu chí nghiệm thu đo được (acceptance ở §8) |
| **Who** | Owner = agent; 4 quyết định phạm vi do user chốt 2026-10-20; **5 câu hỏi mở** ở `P4-1` §9 (Q1–Q5) cần user/PO trả lời trước P4-1 |
| **When** | 6 pha, ~13–16 tuần; bắt đầu sau khi Q1–Q5 được trả lời; mỗi pha 1 commit + CI + graphify (G5) |
| **Where** | Tài liệu mới: `docs/50-assist/P4-0-MASTER-PLAN.md` (file này), `docs/50-assist/P4-1-harper-evaluation.md`; code mới (P4-1 trở đi): crate `assist/`, `corpus/assist/`, `docs/compliance/assist-dictionary.md`; **chưa** sửa code runtime ở phiên này |
| **How** | Verify bằng gì: corpus + `cargo test -p assist` + fuzz + perf gate + manual checklist 8 app; CI 3 OS + `repo-hygiene` (8 check) + `graphify update .` trước khi tick Done |

**Findings vòng 2:**

| ID | Mức độ | Finding | Trạng thái |
|---|---|---|---|
| **Fassist-7** | minor | Cần **task list có ID** theo convention `P{n}-TASKS.md` (WIN/MAC/LNX/P0) để agent sau nhận việc — hiện phần 4 mới chỉ có plan | còn mở → tạo `docs/50-assist/P4-TASKS.md` ở pha P4-1 (task đầu tiên) |
| **Fassist-8** | minor | Chưa có **ADR** cho quyết định "không vendor Harper" — quyết định kiến trúc nên có bản ghi riêng (`docs/adr/`) | còn mở → tạo `docs/adr/ADR-00X-assist-engine.md` ở P4-1 |
| **Fassist-9** | minor | `docs/00-INDEX.md` §1 (trạng thái các phần) chưa liệt kê Phần 4 → agent mới không thấy phần này | fixed @604434c |
| **Fassist-10** | minor | `PLAN.md` §2.3 (S1/S2/C1) chưa trỏ sang Phần 4 → backlog và plan lệch nhau | fixed @604434c |
| **Fassist-11** | minor | `docs/specs/antivirus-false-positive.md` §8 chưa có dòng nói rõ Assist **không** sinh tiến trình mới / không log text | còn mở → bổ sung ở P4-1 (khi có code) |
| **Fassist-12** | minor | Chưa có bước kiểm thử **2 OS khác** cho overlay (overlay là UI native khác nhau) — DoD yêu cầu CI 3 OS nhưng overlay không tự test được | còn mở → bổ sung checklist manual 3 OS trong `P4-6` |

---

## 12. Việc cần làm ngay (nhỏ, để phần 4 "có mặt" trong repo)

1. `docs/50-assist/P4-0-MASTER-PLAN.md` (file này) + `P4-1-harper-evaluation.md` — xong ở phiên này.
2. `docs/00-INDEX.md` §1 — thêm Phần 4 (trạng thái "Kế hoạch", review 0/2) + mục trong §0/§2.
3. `PLAN.md` §2.3 — S1/S2/C1 trỏ sang Phần 4.
4. Trả lời Q1–Q5 (`P4-1` §9) → chốt `P4-TASKS.md` + ADR → bắt đầu P4-1 (crate `assist/`).

---

## 13. Liên kết

- `P4-1-harper-evaluation.md` (cùng thư mục) — đánh giá Harper + so sánh phương án + findings vòng 1
- `PLAN.md` §2.3 · §5.5 — backlog nguồn + perf budget
- `docs/01-AGENT-HANDBOOK.md` — DoD 7 mục + checklist review
- `docs/specs/antivirus-false-positive.md` §8 — chính sách kiến trúc hành vi (không giống malware)
- `docs/10-shared/P0-2-engine-ffi-contract.md` · `P0-3-config-preset-strategy.md` — hợp đồng FFI + config
- `docs/release/ui-spec.md` · `parity-checklist.md` — spec UI 3 OS
