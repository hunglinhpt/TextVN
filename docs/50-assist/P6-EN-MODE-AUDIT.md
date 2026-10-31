# P6-EN-MODE-AUDIT — Trợ lý & dự đoán tiếng Anh: audit + đề xuất (vòng 42)

> 2026-10-31 · Expert auditor + SA view · Bối cảnh: user báo "mode tiếng Anh
> prediction và auto sửa lỗi/gợi ý không chạy ổn định hoặc không chạy hẳn".

## 1. Bản đồ đường tiếng Anh (đã trace từng nhánh)

| Tính năng | Đường đi | Điều kiện chạy |
|---|---|---|
| Ghost text EN | adapter → `ime_peek_completion` → `peek_completion` → `probe.key(Tab)` | VI bật + `suggest.enabled` = true (opt-in) |
| Tab hoàn tất EN | `key(Tab)` → `try_english_complete` | VI bật + `suggest.enabled` (từ chưa fold) hoặc từ đang fold |
| Space hoàn tất EN | `try_english_complete(space)` | VI bật + `suggest.trigger = tab_and_space` |
| Trợ lý ngữ pháp EN (confusions) | trigger space/,/. → `ime_assist_check_v1` → `check_confusions` | **mọi ngôn ngữ** (vòng 42 — trước đây En-only) |
| Trợ lý SVA/giới từ EN | `check_en_agreement` | **mọi ngôn ngữ** (vòng 42 — trước đây En-only) |
| Correlative EN (Although…but) | `check_en` | En-dominant (giữ — cần ngữ cảnh mệnh đề EN trọn) |
| Đại từ "I" thường | `check_pronoun` | En-dominant (giữ — "i" là âm tiết Việt ASCII) |
| Chính tả EN (OS spell) | `en_spelling` + WinSpell/MacSpell | En-dominant + OS provider khoẻ (Windows: retry 30 s — vòng 42) |

## 2. Root cause đã fix (vòng 42)

- **RC-3** — câu mix Vi-dominant (≥2 từ dấu) bỏ qua TOÀN BỘ luật EN:
  "Anh should of done it sớm hơn", "He don't know nhé anh",
  "Em cần discuss about kế hoạch" đều SẠCH. Fix: `check_confusions` +
  `check_en_agreement` (cặp ASCII đóng, tự-guard) chạy mọi ngôn ngữ; giữ
  En-only cho pronoun "i" + correlative + en_spelling.
- **RC-4** — WinSpell probe quá hạn 150 ms MỘT lần = `OnceLock` cache `None`
  **cả phiên** (máy tải nặng là đủ): spell-assist tắt vĩnh viễn tới khi
  restart. Fix: retry mỗi 30 s (`SpellAttempt` + Mutex poison-safe).
- **PERF** — sau khi mở gate, vòng SVA O(n²) trên câu dài (10k từ = 10 s):
  hoist HashMap start→idx, về <3 s cả suite abnormal.
- **RC-1 (một phần)** — VI tắt = strategy Passthrough = **prediction/ghost/
  Tab EN chết hoàn toàn** (theo thiết kế: Tab là phím cấu trúc của app;
  buffer từ không được track trong Passthrough). Assist (auto-fix) VẪN chạy
  khi VI tắt ✓ (`check_sentence` không gate VI, `record()` vẫn feed câu).
  Phần prediction VI-off: xem đề xuất P1.

## 3. Đề xuất (cần quyết định PO — không tự ý đổi)

- **P1 — EN prediction khi VI tắt**: track buffer từ trong Passthrough
  (feed `word.raw` từ `note_pass`, không secure) + hoàn tất **chỉ qua
  Ctrl+Space** (Tab phải giữ nguyên cho indent/điều hướng app) + ghost preview
  theo `suggest.enabled`. Chi phí: config mới (`suggest.when_vi_off`?) —
  đồng bộ 15 site theo luật default. Khuyến nghị làm vòng 43 sau khi PO chốt.
- **P2 — modal + V2/V3** ("will sent" → "will send"): cần bảng động từ bất
  quy tắc đóng (~120 cặp) — recall thực tế cao trong email. Danh sách đóng,
  FPR thấp, làm được ngay khi có ngân sách test.
- **P3 — discoverability của `suggest.enabled`** (mặc định TẮT): phần lớn
  report "prediction không chạy" rơi vào đây. Đề xuất: bật ON cho cài mới
  (sync 15 site + test Swift/C) HOẶC hint trong tray settings lần đầu.
- **P4 — backlog Pass C vòng 41**: appStates cap parity Windows, pipe header
  deadline, SwiftUI duplicate-id.

## 4. Kiểm chứng

- 137 assist lib + 43 abnormal tests xanh; FPR corpus 0/2739 + tatoeba
  rule-touched 0 FP mới; recall 28/28 + 38/38 giữ nguyên.
- Probe CLI: "Anh should of done it sớm hơn." → word_confusion ✓;
  "He don't know nhé anh." → word_confusion ✓; "Tôi is học sinh." → SẠCH
  (đúng — "tôi" không thuộc cặp đại từ EN); "gửi cho i nhé" → không bêu.

## 5. Pass EN-1 (agent audit độc lập) — finding + fix (vòng 42)

- **F1 NẶNG (regression của RC-3, đã fix)**: `do`/`go`/`say` là ÂM TIẾT VIỆT
  deaccent (do=vì, go=gõ, say=sảy) — câu mix "Máy nó it do lỗi nhé" bị tự sửa
  "does" phá chữ (conf 100). Fix: 3 động từ đụng chỉ fire khi En-dominant;
  probe đối chứng "It do the job." En vẫn fix ✓.
- **F2 TRUNG (đã fix)**: `after_adj_is_particle` tính vô điều kiện mỗi window
  ⇒ O(n²) (100k = 4.3s) → lazy trong điều kiện copula+to+adj.
- **F5 THẤP (đã fix)**: lỗi EN trong ngoặc kép ("He said \"we should of
  known better\"") không được sửa — quote-parity vector O(n) hoist cho cả
  check_confusions + check_en_agreement (in_quotes per-window chính nó là
  O(n²) — hoist xong abnormal suite 19.7s → 1.38s).
- **F6 (đã fix)**: comment stale "CHỈ En-dominant", dead statement,
  assert-comment sai.
- **F3 TRUNG (BACKLOG — macOS EN spelling chết cấu trúc)**:
  MacSpellProvider chỉ đăng ký "vi" (EN từ sai đi vào checker vi → fail-open
  im lặng ⇒ recall en_spelling = 0 trên macOS, Windows có dual-language);
  `isAvailable` probe-once không retry (cùng class RC-4 Windows vừa fix).
  Cần: provider "en" thứ hai dispatch theo ASCII + retry cooldown — sửa Swift
  chỉ compile trên CI, làm vòng sau khi có ngân sách verify macOS thật.
- **F4 THẤP (ghi nhận)**: os_spell TrySendError::Full vẫn stall 50ms —
  breaker 1 lần/10s che, chấp nhận.

Verdict EN-1 sau fix: các fix a16ca0b đạt chất lượng ship; corpus 4551 cases
0 mismatch.

## 6. Pass EN-2 (đối kháng 2 commit vòng 42) — finding + fix (vòng 42, 2026-10-31)

- **N1+N2 NẶNG (đã fix)**: văn bản Việt **KHÔNG DẤU** ("noi it do met",
  "gui cho i nhe") classify sang **En** (it+do = 2 từ chức năng) — F1 gate
  lang không che được → auto-fix "does"/"I" phá chữ. Fix: (a) `do/go/say`
  thêm gate **câu có tín hiệu Việt** (`vn_sentence_hint`: ≥1 từ âm-tiết
  Việt-only `is_vietnamese_ascii_word` — loại giao điểm do/to — hoặc chữ Việt
  có dấu); (b) `check_pronoun` skip khi từ kề "i" là tín hiệu Việt.
  Đối chứng: EN thuần ("It do the job perfectly.", "and then i went home.")
  vẫn fix ✓.
- **R1 TRUNG (đđ fix)**: quote MỞ KHÔNG ĐÓNG / dấu inch `24"` lật parity nuốt
  MỌI lỗi EN phía sau. Fix: `compute_quote_parity` — parity mở mà NHÁY CUỐI
  nằm trước vị trí đó ⇒ coi như ngoài quote (parity vector nâng lên **Ctx
  dùng chung** — 1 lần/câu).
- **R2 TRUNG (đã fix)**: parity guard chỉ có ở windows(3) — đồng bộ cho
  windows(2) ("there own"), then/than backscan, check_pronoun. Còn lại
  (article_an — đã có 3 tầng guard riêng; check_en correlative — câu mở đầu
  EN quote hiếm) ghi nhận.
- **P1 ghi nhận (backlog)**: F2 lazy vẫn O(n²) worst ("was to slow"×80k =
  18s) + 2.5/2.6 `position()` — IME window 256 chars nên không đau; hoist
  HashMap khi có ngân sách.
- Corpus đối chứng EN-2: FPR 0/4334 (4 corpus + tatoeba), hard recall 100%,
  15/15 probe đối kháng sau fix.

**Đo vòng 42**: 135 lib + 48 abnormal = 183 test assist · FPR 0/2739 ·
recall 28/28 + 38/38 · clippy 0 · preflight exit=0.

## 7. Vòng 43 — audit 3 pass "mọi ngõ ngách" (D1/D2/D3) — 2026-10-31

- **Self-audit các crate chưa từng soi** (strategy/config/appdb/field-detect/
  ipc): 0 panic production, `config validate` chặn sạch mọi abnormal (type
  sai/macro loop/macro rỗng/macro 100k/emoji trùng/per-app key 400 ký tự/
  JSON hỏng), IPC codec error taxonomy đầy đủ. PASS.
- **Pass D1 — static typing engine** (method/transform/validate, 4.1k dòng):
  256 test core + 24 test tạm pass; perf key() = **0.13µs/phím** (dư 3800×
  ngân sách). Fix: **F1 TRUNG** VIQR thiếu freeze E28 (`hoa...` toggle vô tận,
  `dddd` gạch lại) → undone_tone/mark/stroke state per-word; **F4** VIQR `DD`
  hoa không gạch → eq_ignore_ascii_case; **F3** xoá dead code
  `is_valid_syllable_prefix` (115 dòng — whitelist chỉ đúng Telex, hazard nếu
  cắm VNI). **F2 THẤP ghi spec**: từ >63 phím spam-literal co display bị ăn
  1 phím khi close_word (không lệch sổ, không panic).
- **Pass D2 — hành trình phím 3 adapter**: fix vòng 40-42 nguyên 6/6 trên
  đường phím. Fix: **F1 NẶNG** macOS toggle VI giữa từ KHÔNG chốt marked +
  reset (2 nền tảng kia có — parity; phím kế thay thế marked = MẤT TỪ) →
  commitBeforeHide(sender) trước toggle; **F2 TRUNG** nhánh `.pass` vi phạm
  bất biến "phím pass ⇒ composition đã đóng" → chốt marked cuối nhánh
  (parity Windows pass_through/composing); **F3 TRUNG** ConfigReload giữa từ
  clear buffer (core set_options) → defer qua `pendingConfigReload` tới
  marked rỗng. F4-F7 THẤP ghi nhận (dead-key/OSK, replay keyup, toggle tray
  trễ 1 từ, Fcitx5 flag per-engine).
- **Pass D3 — đối kháng** VIQR freeze (11/11 probe: đa âm, interleave,
  free_marking=false, ESC/Backspace rebuild, hoa-thường, sừng cụm) + Swift
  fixes (call site sender, .pass các đường tránh đúng, defer không nuốt
  message) + góc cuối (register.rs WOW64 nguyên, viqr.toml đủ, 11 YAML OK)
  + corpus **0/4334** + hard recall 100%. Follow-up đã vá: guard F9
  `markedRange == NSNotFound` trong commitBeforeHide (app tự commit marked →
  chèn thêm = nhân đôi) + nit comment.

## 8. Vòng 44 — persona 7 profile + parity VI/EN (2026-10-31)

**Parity OS × tính năng (tự audit từ code):** đủ — ghost/kinds 13/transcode/
suggest UI/Mode 2/per-app/EN-words UI trên cả 3 OS. Gap duy nhất: macOS EN
spelling (F3 backlog vòng 42); Linux không có OS spell (by design — không có
API hệ thống nhất quán); Linux tray-toggle trễ 1 từ (F6); macOS dead-key
Option+chữ (F4). **Parity VI/EN engine:** phẩy-sau-"but" (EN) thiếu đối xứng
"nhưng" (VI) → helper chung `check_comma_after_conjunction`; L6 parity ✓
sentence mode.

**P-2 persona battery (agent, 140 câu + 25 thăm dò — 7 profile):** 6 persona
công việc 78/78 CLEAN · nhóm NGƯỜI DÙNG BÌNH THƯỜNG (chat không dấu) lộ 2
FPR auto-fix conf 100:
- **FPR-1** `article_an` bêu "an com" (ăn cơm) → "a com" — instance-check cũ
  dùng bản loại-giao-điểm ("com" có trong en_freq3000). Fix: instance-level
  dùng `is_vn_ascii_syllable` (không loại giao điểm) cho **từ kề ngay** a/an.
- **FPR-2** `lowercase_pronoun` bêu "i di" (= i đi — "di" cũng giao điểm) →
  công thức `has_vn_lazy_hint`: **≥1 âm tiết VN ∧ 0 từ EN thật** (không âm
  tiết VN, không EN-function — went/home/brother là EN thật) ⇒ "i di an" im,
  "and then i went home"/"My brother told me that i could" vẫn chạy.
- EN_FUNC +2: "am" (ẩm — "therefore i am" bị nuốt), "then" (thến — "and then
  i" bị nuốt).
- **R-1 (đã fix)**: trợ động từ EN lạc sau từ VI ("Chưa been đối chiếu",
  "cần been reviewed") → +5 cặp (chưa/cần/đang/sẽ/vẫn, been) suggestion-only;
  thu hẹp guard T5 (từ VI hoa ĐẦU CÂU = vị trí, không phải tên riêng — từ EN
  hoa mới là title).
- **R-2 (backlog)**: phẩy sai giữa câu chung ("Cần chốt scope, trước thứ
  Sáu") — cần rule mới, FPR khó; **R-3**: typo biến dạng ("chỉnhin") — cần
  spell-check VI (ngoài scope).

**Đề xuất tính năng (từ persona — chưa làm):** (1) cảnh báo ngày tháng vô lý
(32/13, 31/02 — kế toán/HR viết deadline); (2) cảnh báo format số tiền phân
cách hỗn loạn ("1.250.00đ"); (3) spell-check tiếng Việt (R-3) — cần nguồn
từ điển VN offline. **Đo:** FPR 0/2739 · recall 28/28 + 38/38 · 135 lib +
52 abnormal tests · 15/15 probe parity.
