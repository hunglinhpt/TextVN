# P4 — REVIEW LOG (Phần 4: Trợ lý ngữ pháp — Assist)

> Giao thức: `P4-0-MASTER-PLAN.md` — mỗi phần phải có review **2 lượt** (1 = *Đúng & Đủ*,
> 2 = *Nhất quán & Sẵn sàng*); sau implement lại 3 lượt (SA/QA → Audit → PO/Design).
> Phạm vi tổng hợp: `P4-0-MASTER-PLAN` (§11 audit) · `P4-1-harper-evaluation` ·
> `P4-2-CROSS-REVIEW-2026-10-20` (agent B) · `P4-3-SPIKES` (P4-1b) · `P4-5-P4-2A-REVIEW` ·
> `P4-7-REVIEW-AS-021-022` · `P4-8-QUALITY-BASELINE` · `P4-9-REVIEW-DICT` ·
> `P4-10-OS-PROVIDER-SPEC` + log theo ngày ở `docs/00-INDEX.md` (2026-10-20…2026-10-21).
> Mã finding: `Fassist-n` (1…110) + ID tạm `P4-10-Fn` của đặc tả provider OS.
> **Mã vòng R (2026-10-21):** `R-{n}` (1…14) — review **AS-050 (A3 tự sửa cả câu)** + **AS-060 (parity macOS/Linux)**,
> commit `c701600` + các fix sau (`1f5c7c6` · `dc4a1e4` · `4f7cd7d`); xem §"VÒNG REVIEW AS-050/AS-060" bên dưới.

---

## VÒNG 1 — Đánh giá Harper (vòng "Đúng & Đủ") — 2026-10-20

**Vai trò:** PO/BA/SA/QA/Audit/Design (6 vai trò) · **Phạm vi:** `P4-1-harper-evaluation.md`
**Số finding:** 6 (`Fassist-1..6`) — 1 major còn hiệu lực, 1 major bị triệt tiêu, 4 minor

| ID | Mức | Finding (tóm tắt) | Trạng thái |
|---|---|---|---|
| Fassist-1 | **major** | `harper-core` chỉ hỗ trợ **tiếng Anh ở mọi tầng** (lexer/dictionary/luật/POS) → không tạo tính năng tiếng Việt | ✅ **Đã xử lý** → **D1/D2**: không vendor ở v1, mượn kiến trúc |
| Fassist-2 | **major** | "crates.io stale ~27 tháng" | ⚠️ **Sai tiền đề → superseded** bởi Fassist-32 (bản crates.io thật = 2.13.0/2026-10-16); giữ ID theo G6, không xoá |
| Fassist-3 | minor | Không có từ điển tiếng Việt mở đóng gói sẵn với giấy phép đã kiểm | ✅ Đã xử lý → **D3** + P4-0 §5 (tự xây + audit từng nguồn) |
| Fassist-4 | minor | "Highlight lỗi" là giới hạn kỹ thuật (TSF/IMK/IBus không vẽ gạch chân vào edit control) | ✅ Đã xử lý → **D4** (hiển thị cạnh con trỏ) |
| Fassist-5 | minor | Áp sửa "cả câu" cần map offset → vị trí thật trong field (ITfRange/AX/AT-SPI) | ✅ Đã xử lý → D12 + giới hạn v1 (chỉ câu tự gõ, precondition caret) |
| Fassist-6 | minor | Trùng lặp với backlog S1/S2 của PLAN | ✅ Đã xử lý → P4-0 §1 (S1/S2 nâng cấp thành Phần 4) |

---

## VÒNG 2 — Master plan "Nhất quán & Sẵn sàng" — 2026-10-20 (`P4-0` §11.1)

**Vai trò:** Agent Core (audit 2/2 của phần doc) · **Phạm vi:** `P4-0-MASTER-PLAN` v1 + `00-INDEX` + `PLAN.md`
**Số finding:** 6 (`Fassist-7..12`) — toàn bộ minor

| ID | Mức | Finding | Trạng thái |
|---|---|---|---|
| Fassist-7 | minor | Thiếu task list có ID | ✅ Đóng — `P4-TASKS.md` (AS-001…068) |
| Fassist-8 | minor | Chưa có ADR "không vendor Harper" | ✅ Fixed — **ADR-012** |
| Fassist-9 | minor | `00-INDEX.md` §1 chưa liệt kê Phần 4 | ✅ Fixed @604434c |
| Fassist-10 | minor | `PLAN.md` §2.3 chưa trỏ Phần 4 | ✅ Fixed @604434c |
| Fassist-11 | minor | `antivirus-false-positive.md` §8 chưa có dòng Assist | ✅ **Đã xử lý** — thêm A14 vào §8.1 + §8.4 (0 network, 0 new process, 0 content log) |
| Fassist-12 | minor | Thiếu checklist manual (overlay/parity UI) 3 OS | 🟡 Một phần — `parity-checklist.md` đã có dòng `assist.*` (trạng thái 3 OS); còn chờ hoàn thiện Cài đặt 3 OS (P4-6) |

---

## VÒNG 3–5 — Review chuyên sâu (SA/QA → Audit → PO/BA/Design) — 2026-10-20 (`P4-0` §11.2)

**Vai trò:** Lượt 3 = SA + QA · Lượt 4 = Audit (license/security/privacy/ops) · Lượt 5 = PO/BA/Design
**Phạm vi:** thiết kế Mode 1/2, cơ chế áp sửa, UI, từ điển, DoD
**Số finding:** 19 (`Fassist-13..31`) — 6 major, 13 minor → sinh quyết định **D11–D20**

| ID | Mức | Finding (1 dòng) | Trạng thái |
|---|---|---|---|
| Fassist-13 | **major** | Sửa text đã commit (Mode 2) dễ sửa nhầm (không giữ range sau commit) | ✅ → **D12** Backspace+Type + precondition caret + fail-safe |
| Fassist-14 | **major** | Trigger space/,/. spam UI | ✅ → **D11** tách điểm kiểm tra / điểm hiển thị |
| Fassist-15 | **major** | Overlay cạnh con trỏ không khả thi Linux/Wayland | ✅ → **D13** candidate list của IME (sau D30 bổ sung theo OS) |
| Fassist-16 | minor | Crate mới → artifact/packaging mới | ✅ → **D14** crate nội bộ, expose qua `ffi` |
| Fassist-17 | minor | Shadow buffer chưa tồn tại ở runtime (TSF chỉ có mô phỏng test) | ✅ **Đã xong** ở **P4-2a** (`Engine.sentence`, AS-020) |
| Fassist-18 | minor | Parse từ điển trong mỗi process = 100 ms+/app | ✅ → **D15** FST mmap |
| Fassist-19 | minor | Câu tiếng Anh bị báo sai (v1 không hỗ trợ EN) | ✅ → **D16** luật L0 |
| Fassist-20 | minor | Xung đột `auto_capitalize` | ✅ → **D16** L6 bỏ qua khi auto_capitalize bật |
| Fassist-21 | **major** | "Mượn kiến trúc Harper" cần quy tắc clean-room bằng văn bản | ✅ → **D17** + ADR-012 |
| Fassist-22 | **major** | Nguồn từ điển VN chưa khảo sát = không cam kết lịch | ✅ → **D18** spike S3 (P4-1b xong) |
| Fassist-23 | minor | UI hiển thị text người dùng: không log/clipboard/a11y-tree | ✅ Ghi §6 + Audit lượt 8/17 xác nhận 0 log |
| Fassist-24 | minor | Không auto-disable khi UI phiền | ✅ → **D20** |
| Fassist-25 | minor | Không thu thập metrics ngầm | ✅ → **D20** |
| Fassist-26 | minor | Threat model (app độc hại đẩy text) phải ghi `PLAN.md` §6.1 | ✅ **Đã xử lý** — thêm kịch bản context tampering vào `PLAN.md` §6.1 STRIDE |
| Fassist-27 | **major** | Thiếu tiêu chí "đủ tốt" (recall) | ✅ → **D19** recall ≥ 80% · FPR ≤ 2% · 2 người đánh giá |
| Fassist-28 | minor | Thiếu bằng chứng cho từng task (DoD) | ✅ §9 + P4-TASKS (bằng chứng mỗi tick) |
| Fassist-29 | minor | 3 giả định lớn chưa kiểm chứng | ✅ → **D18** spike S1/S2/S3 |
| Fassist-30 | minor | Thiếu kill-switch 1 bước + default off + rollback | ✅ → **D20** + `mode: off` |
| Fassist-31 | minor | Quan hệ với spellcheck/auto_capitalize sẵn có | ✅ §5.1 (L6) + hồi quy |

---

## VÒNG 6 — CROSS-REVIEW (agent B) + phản hồi agent A — 2026-10-20 (`P4-0` §11.3, `P4-2-CROSS-REVIEW`)

**Vai trò:** agent B (cross-review độc lập) → agent A (verify + xử lý)
**Phạm vi:** mọi fact + thiết kế Phần 4
**Số finding:** 14 (`Fassist-32..45`) — 7 major, 7 minor → quyết định **D21–D29**, ADR-013, task P4-2a

| ID | Mức | Finding (1 dòng) | Trạng thái |
|---|---|---|---|
| Fassist-32 | **major** | **Tiền đề "crates.io stale 27 tháng" SAI** (thật 2.13.0/2026-10-16) — vi phạm G3 | ✅ Đã sửa 8 chỗ + viết lại **D21**; Fassist-2 superseded |
| Fassist-33 | **major** | Mâu thuẫn Mode 1 (SelectionReplace) vs D12 | ✅ → **D22** v1 bỏ SelectionReplace |
| Fassist-34 | minor | Swift/C/C++ không gọi crate Rust trực tiếp | ✅ → **D23** C-ABI `ime_assist_*` + header + `verify` |
| Fassist-35 | **major** | Buffer câu nên nằm ở **core** (recent = glyph space, 13 call site) | ✅ → **D24** + P4-2a (AS-020, xong) + ADR-013 |
| Fassist-36 | **major** | Chi phí resident per-process (TIP in-proc / IBus 1 instance/context) | ✅ → **D25** `OnceLock<Arc<…>>` + spike S4a (đạt gate) + S4b (v2) |
| Fassist-37 | minor | clippy full-parallel hết paging file | ✅ → `verified-ops.md` **B9/B10** |
| Fassist-38 | **major** | F35 tạo mâu thuẫn mới với F36 (buffer trong core × TIP in-proc) | ✅ → **D26** buffer chỉ khi `mode != off` (sau D31: luôn bật, cap 256) |
| Fassist-39 | **major** | Hợp đồng Unicode ↔ glyph chưa rõ | ✅ → **D27** `Engine::glyph_len_of`; Fix không mang số xoá |
| Fassist-40 | **major** | L0 đã chốt nhưng chưa có trong skeleton | ✅ Task P4-1 (`rules/language.rs`) — xong |
| Fassist-41 | minor | `Issue` thiếu `confidence` (thêm sau phá API) | ✅ → **D28** thêm ngay ở P4-1 |
| Fassist-42 | minor | Offset xuyên biên chưa chốt (char vs UTF-16) | ✅ → **D29** chuẩn hoá UTF-16 ở biên FFI |
| Fassist-43 | minor | Định dạng corpus `.ass` chưa đặc tả | ✅ TSV trong `P4-TASKS.md`/`P0-4` |
| Fassist-44 | minor | "6 luật" vs 6 module / 7 mã luật | ✅ Sửa wording (§5.1) |
| Fassist-45 | minor | Timeline "nếu EN pass" mâu thuẫn D9 | ✅ Giữ v1 không EN; S4b không tính timeline |

---

## VÒNG 7–9 — Review sau khi implement P4-1 — 2026-10-20 (`P4-0` §11.4)

**Vai trò:** Lượt 7 SA+QA · Lượt 8 Audit · Lượt 9 PO/BA/Design
**Phạm vi:** crate `textvn-assist` + corpus + CLI
**Số finding:** 13 (`Fassist-46..58`) — 2 major (đã sửa), còn lại minor

| ID | Mức | Finding (1 dòng) | Trạng thái |
|---|---|---|---|
| Fassist-50 | **major** | **Panic slice byte bằng offset char** (tiếng Việt có dấu; 19 test đỏ) | ✅ Sửa: tách `Vec<char>`; luật "đọc token trong char-space" |
| Fassist-49 | **major** | CLI dispatch ăn mất subcommand `check` | ✅ Sửa + 5 case smoke test |
| Fassist-46 | minor | Message người dùng thiếu dấu tiếng Việt | ✅ Sửa toàn bộ |
| Fassist-47 | minor | `--file` check cả file → L0 bỏ file song ngữ | ✅ Sửa: per-line + số dòng |
| Fassist-48 | minor | Corpus 30 câu < ngưỡng 60 | ✅ Mở lên 69 câu |
| Fassist-51 | minor | Thiếu test `check_sentence` đầu câu | ✅ Bổ sung |
| Fassist-52 | minor | Audit: dependency mới? | ✅ Không (0-dep; deny OK) |
| Fassist-53 | minor | Audit: rò rỉ text qua log/disk? | ✅ Không (không log/file/network) |
| Fassist-54 | minor | REUSE/Spdx file mới | ✅ OK |
| Fassist-55 | minor | DoD #1–#4 | ✅ 34 unit + 3 integration pass · clippy 0 · deny OK |
| Fassist-56 | minor→**major** | CLI không giới hạn số lỗi in ra (spike lộ: 8.000 dòng) | ✅ Sửa ở P4-1b: min 50 lỗi mặc định + `--max <n>` |
| Fassist-57 | minor | `textvn-cli assist` chưa vào `docs/developer-guide.md` | ✅ **Đã xử lý** — tài liệu hoá đầy đủ trong `docs/user-guide.md` và `docs/developer-guide.md` (@ce6c3e2) |
| Fassist-58 | minor | Chưa đo recall/FPR thật (cần ≥ 300 câu có lỗi) | 🟡 Một phần — P4-8 đã đo 1.500+ câu thật (FPR thực ≈ 0%); còn thiếu **2 người đánh giá độc lập** trước khi thành gate CI/bật tính năng |

---

## VÒNG 8–10 + 11 — Review sau 4 spike (P4-1b) + bản đồ 13 call site — 2026-10-20 (`P4-0` §11.6–11.7, `P4-3-SPIKES`)

**Vai trò:** Lượt 8 SA · Lượt 9 Audit · Lượt 10 PO/QA/Design · Vòng 11 = review bản đồ call site
**Phạm vi:** S1 (TSF candidate) · S2 (caret 3 OS) · S3 (từ điển VN) · S4a/S4b (RSS/EN) · chuẩn bị P4-2a
**Số finding:** 8 (`Fassist-59..66`) — 2 major → quyết định **D30/D31**

| ID | Mức | Finding (1 dòng) | Trạng thái |
|---|---|---|---|
| Fassist-59 | major | S1 kết luận "Windows overlay tự vẽ" mâu thuẫn D13 | ✅ → **D30** UI theo OS (Win overlay · Linux candidate list · macOS toast) |
| Fassist-60 | minor | Từ điển GPLv2 → phái sinh FST phải attribution + REUSE | ✅ Đã thực hiện (thuộc P4-4) + chốt lại ở Fassist-98/ADR-014 |
| Fassist-61 | minor | S4a đo trên tiến trình CLI ≠ TIP in-process | ❌ **Còn mở** — phải đo lại trong TIP/IME thật ở P4-2 |
| Fassist-62 | minor | Từ điển 6.642 từ đơn → recall có thể không đạt | 🟡 Đã đo (AS-041/Damerau: recall 100% trên corpus lỗi); con số recall/FPR "từ điển thật" vẫn phụ thuộc v2 (Fassist-100) |
| Fassist-63 | minor | macOS dùng toast ≠ trải nghiệm 2 OS kia | ❌ **Còn mở** → ghi parity-checklist; macOS adapter chưa nối assist ✗ |
| Fassist-64 | minor | S4b giữ D9 (không sửa) | ✅ Không thay đổi |
| Fassist-65 | **major** | 2 call site có hậu kỳ hậu-encode (site 523 cắt `MAX_TEXT` sau `emit`) | ✅ Đã xử lý qua recipe P4-2a sạch (`record()` giữ semantics, oracle = test `core` không đổi — AS-020 143/143 pass) |
| Fassist-66 | minor | `recent.pop()` xoá 1 glyph vs `sentence` 1 char Unicode | ✅ Xác nhận 2 buffer tách bạch (D24/D27) trong P4-2a |

---

## VÒNG 12–14 — Nghiên cứu OS-first (P4-4-OS-NATIVE-ASSIST) — 2026-10-21

**Vai trò:** SA/QA (API 3 OS) → Audit (license/AV) → PO/UX
**Phạm vi:** tận dụng spellchecker có sẵn của OS/app
**Số finding:** 6 (`Fassist-67..72`) — 2 major → quyết định **D32–D37**

| ID | Mức | Finding (1 dòng) | Trạng thái |
|---|---|---|---|
| Fassist-67 | **major** | "OS-first ⇒ không cần từ điển riêng" là SAI (Windows không có `vi-VN`) | ✅ → **D33/D37** OS-first là đường chính + FST là fallback bắt buộc |
| Fassist-68 | **major** | macOS doc "grammar check" — API ngữ pháp không chắc | ✅ Không hứa ngữ pháp OS ở v1 (sau P4-10-F6 xác nhận `checkGrammar` CÓ nhưng vẫn không dùng — đồng nhất 3 OS) |
| Fassist-69 | minor | `libspelling`/enchant là LGPL | ✅ Ghi: link động + graceful fallback; compliance còn mở (P4-10-F8) |
| Fassist-70 | minor | Provider Windows out-of-proc có thể block | ✅ → **D36** worker thread + timeout 50 ms + fail-open (đã implement AS-042) |
| Fassist-71 | minor | OS user dictionary có thể bị báo sai | ✅ Thiết kế: đọc user words khi có (verify con lại theo P4-10 §9.9) |
| Fassist-72 | minor | Trùng lặp spellcheck của app | ✅ → **D34** per-app `assist.scope` |

---

## VÒNG FOLLOW-UP P4-2a + UI/TSF (AS-020…022) — 2026-10-21 (`P4-5-P4-2A-REVIEW`, `P4-7-REVIEW-AS-021-022`)

**Vai trò:** Review độc lập code (SA/QA → Audit → PO/UX, vòng 15–18)
**Phạm vi:** `P4-2a` (buffer `Engine.sentence`) và FFI assist của agent song song (commit `dba598c`)
**Số finding:** 7 (`Fassist-73..79`) — 1 major

| ID | Mức | Finding (1 dòng) | Trạng thái |
|---|---|---|---|
| Fassist-73 | **major** | Thiếu bất biến 3 buffer (`sentence` ↔ `sentence_widths`) lệch độ dài | ✅ Sửa: `debug_assert_eq!` + test sync |
| Fassist-74 | minor | `Tail::Pop` xoá 1 char Unicode dù app xoá 1 glyph | ✅ Chấp nhận (hướng bảo toàn — conservative) + doc-comment + test |
| Fassist-75 | minor | ADR-013 "chỉ bật khi mode ≠ off" lệch code (D31 luôn bật) | ✅ Cập nhật ADR-013 |
| Fassist-76 | minor | `sentence_tail()` trả `String` (copy) per-trigger | ✅ Đóng — `sentence_tail_chars() -> &[char]` (0-alloc) |
| Fassist-77 | minor | `Tail::Replace` 3 tham số dễ lệch cặp | ✅ Ghi doc-comment + checklist "call site mới phải kèm test" |
| Fassist-84 | **major** | **Mirror header Swift stale** → job macOS đỏ từ `dba598c` | ✅ Đã sửa (`dd1afeb`) — 7/7 workflow xanh; bài học: sửa header = sửa cả 2 bản cùng commit |
| Fassist-85 | minor | Header không do cbindgen sinh → sửa tay được nhưng phải cả 2 bản | ✅ Đã ghi rõ quy trình |
| Fassist-79 | minor | Điểm hiển thị (D11) chưa có trong diff AS-022 | ✅ Đúng phạm vi — thuộc UI (AS-023/AS-030), không chặn |
| Fassist-78 | minor | Header chưa tài liệu hoá cap + ownership của `ime_assist_result_v1` | ✅ **Đã xử lý** — tài liệu hoá cap `IME_MAX_ASSIST_ISSUES` + caller-owned struct trong `ffi/include/` và CTextVNFFI |

---

## VÒNG DICT — Review 3 lượt + CI (Agent Dictionary) — 2026-10-21 (`P4-9-REVIEW-DICT`)

**Vai trò:** Vòng 1 SA/QA · Vòng 2 Audit (license — phát hiện nghiêm trọng) · Vòng 3 PO/QA · CI (G13)
**Phạm vi:** commit `1b2fc48` (trait `Dictionary` + crate `dict` FST/mmap + `.dic` + REUSE)
**Số finding:** 7 (`Fassist-98..104`) — 2 major → **ADR-014 / D39**

| ID | Mức | Finding (1 dòng) | Trạng thái |
|---|---|---|---|
| Fassist-98 | **major** | `.dic` khai "GPL-2.0-or-later" **chưa chứng minh** (gốc chỉ "GPL", không "or later" → GPL-2.0-only không tương thích GPL-3.0 một chiều) | ✅ Fix triệt để: REUSE id `LicenseRef-hunspell-vi` + LICENSES file + **`.iss` loại `.dic` khỏi gói** + ADR-014; reuse lint 789/789 ✓ |
| Fassist-99 | minor | Điều khoản "sửa thì gửi thay đổi" — ta phái sinh FST ⇒ nên thông báo upstream / ghi `docs/compliance/` | ❌ **Còn mở** (LICENSES đã có, compliance chưa) |
| Fassist-100 | minor | Chưa có luật nào dùng trait `Dictionary` ⇒ chưa đo recall/FPR có từ điển | ✅ Đóng theo **D33** — v1 = luật objective + OS provider; luật spelling + đo = **v2** |
| Fassist-101 | minor | Chưa đo RSS/p95 per-process cho crate `dict` | ❌ **Còn mở** → đo ở P4-2 tích hợp thật (gộp với Fassist-61) |
| Fassist-102 | minor | Hạ tầng có nhưng chưa dùng được → tránh hứa "đã có chính tả" | ✅ Đã ghi rõ trong plan: v1 = luật objective + OS spellcheck |
| Fassist-103 | minor | Chưa có test FST/mmap (tra cứu có/không, gợi ý) | ✅ Đóng — `dict` có unit test (lib.rs/mmap.rs) |
| Fassist-104 | **major** | **CI đỏ**: self-declared symbol `open` trùng std runtime (clippy ≥ 0.1.99) | ✅ Đã sửa `dict/src/mmap.rs`: `std::fs::File` + `AsRawFd`, chỉ còn extern `mmap/munmap` (bài học → P4-10 W5) |

---

## VÒNG OS PROVIDER SPEC (Agent B) — 2026-10-21 (`P4-10-OS-PROVIDER-SPEC` + INDEX)

**Vai trò:** Agent B đặc tả + đo thực nghiệm Win11; Agent A review + đáp số; chủ P4-TASKS cấp số
**Phạm vi:** provider OS 3 nền tảng (Windows Spell Checking API · macOS `NSSpellChecker` · Linux libspelling/enchant)
**Số finding:** 6 cấp số `Fassist-105..110` + 8 ID tạm `P4-10-F1..F8` — 5 major

| ID | Mức | Finding (1 dòng) | Trạng thái |
|---|---|---|---|
| Fassist-105 | **major** | Luật spelling gọi `suggest()` cho MỌI từ lạ ⇒ vượt budget 50 ms (Suggest đắt 50–100× Check) | ✅ Đã implement — **D40** `Budgeted<D>` + `DEFAULT_SUGGEST_BUDGET = 2`/câu + `assist/tests/dictionary_budget.rs` |
| Fassist-106 | minor | Offset UTF-16 ở biên chưa có test astral | ✅ Thuộc D29 (P4-10 §2.4 đo được; phần convert + test ở adapter) |
| Fassist-107 | minor | Windows mặc định không có `vi-VN` ⇒ KHÔNG hứa chính tả v1 | ✅ Ghi vào plan/parity-checklist |
| Fassist-108 | minor | Fact SAI "hunspell-vi có sẵn Debian/Ubuntu" (chỉ Fedora/RHEL) | ✅ Đã sửa P4-3-SPIKES |
| Fassist-109 | minor | `check_ranges` KHÔNG có libspelling 0.4.x (chỉ `main`) | ✅ Đã sửa P4-4 §1 (dùng `check_word` + `list_corrections`) |
| Fassist-110 | minor | Test agent A sai ngân (không phải mọi issue đều có gợi ý) | ✅ Đã sửa |
| P4-10-F1 | **critical** (*) | = Fassist-109 (spec P4-4 ghi `check_ranges` mà không dùng được ở v1) | ✅ Đóng |
| P4-10-F2 | major | = Fassist-108 + giữ ADR-014 | ✅ Đóng |
| P4-10-F3 | major | = Fassist-105 (budget) | ✅ Đóng (D40) |
| P4-10-F4 | major | Offset Windows = UTF-16, MS không nói ⇒ bug ẩn | ✅ Convert + test astral ghi ở AC-6 (P4-10 §7) |
| P4-10-F5 | minor | `hasCorrection` không public trên `NSSpellChecker` | ✅ Đã sửa spec |
| P4-10-F6 | minor | `checkGrammar` có nhưng dùng sẽ lệch hành vi 3 OS | ✅ Giữ F68 (không dùng v1) |
| P4-10-F7 | minor | libspelling kéo GTK4 vào engine ⇒ enchant-2 trực tiếp làm mặc định Linux | ✅ Chọn enchant-2 (AS-042) |
| P4-10-F8 | minor | `cargo-deny` không kiểm soát license thư viện C ⇒ thêm `docs/compliance/` | ❌ **Còn mở** (compliance hiện chỉ có `naming-check.md`) |

> (*) ID tạm **chờ chủ P4-TASKS cấp số** — bảng này ghi nhận ánh xạ sang `Fassist-*` đã cấp số; không tự gán mã mới (G3).

---

## VÒNG REVIEW AS-050 / AS-060 — 3 lượt (SA/QA → Audit → PO/UX) — 2026-10-21 (`R-1…R-14`)

**Phạm vi:** commit `c701600` (*feat(assist): AS-050 whole-sentence auto-fix + AS-060 macOS/Linux parity*) + các commit fix sau
(`1f5c7c6` · `dc4a1e4` · `a9b105b` · `4f7cd7d` · `feb5f64` · `63d8ef0`).
**Số finding:** 14 (`R-1…R-14`) — 10 major, 4 minor.
**Bàn giao:** sổ bàn giao của **Phần 4** — mỗi dòng có **bằng chứng kiểm chéo** (lệnh grep / commit / link CI) để agent sau tự verify.

> **Nguồn:** `R-1` (tick Linux parity sai) và `R-8` (push khi CI đỏ nhưng tick ✅) là finding gốc của vòng review.
> `R-2…R-7`, `R-9…R-14` được **verify lại trực tiếp trong code/CI** khi chuyển sổ bàn giao (2026-10-21) — mỗi dòng đều có cột *Bằng chứng* để đối chiếu.

| ID | Mức | Finding (1 dòng) | Bằng chứng | Trạng thái |
|---|---|---|---|---|
| R-1 | **major** | Tick **"Linux (IBus/Fcitx5) parity ✅" sai** — IBus engine **không có một dòng assist nào** | `git grep -n assist -- adapters/linux-ibus` ⇒ **0 dòng** (chỉ `adapters/linux-fcitx5` có) | ✅ **Đã sửa** — `P4-TASKS.md`: hàng P4-6 + dòng AS-060 → **⏳** (IBus: chưa làm · Fcitx5: đã nối, còn lỗi đang sửa) + ghi chú nguyên nhân |
| R-2 | **major** | Tick ✅ cho AS-050/AS-060 **trong khi CI của commit đó đỏ** ⇒ task bị coi là xong oan | `gh run list --commit c701600…` ⇒ `ci-macos` FAILURE · `ci-linux` FAILURE · `ci-shared` FAILURE | ✅ **Đã sửa** — AS-050/AS-060 → ⏳ kèm ghi chú (G13 + G17) |
| R-3 | **major** | `parity-checklist.md` dòng `assist.*` **stale**: ghi "macOS ✗ · Linux ✗ adapter chưa nối" trong khi code **đã nối** | macOS `AssistModel.swift` + `TextVNInputController.processAssist` · Fcitx5 `engine.cpp::checkAndApplyAssist` | ✅ **Đã sửa** — dòng `assist.*` viết lại theo trạng thật (**⏳**, không ghi ✓) + chú thích ⏳ trong legend |
| R-4 | **major** | Fcitx5 Mode 2 **không có precondition D12** — gọi `deleteSurroundingText` + `commitString` thẳng ⇒ rủi ro sửa nhầm vùng text của app | `adapters/linux-fcitx5/src/engine.cpp` (`checkAndApplyAssist`) — **không** có `FixPrecondition` (Windows có 4 điều kiện: selection rỗng · caret không đổi · buffer là của ta · lùi đúng glyph); `63d8ef0` **chưa** sửa mục này | 🔄 **Đang sửa** (lane Linux) |
| R-5 | **major** | Fcitx5 **thiếu undo A3** ⇒ lệch parity Windows (`AssistUndoState` 5 s) và macOS (`Cmd+Z`) | `grep -i undo adapters/linux-fcitx5/src/engine.cpp` ⇒ **0 dòng** (kể cả sau `63d8ef0`) | 🔄 **Đang sửa** (lane Linux) |
| R-6 | **major** | **IBus chưa nối** ⇒ "parity Linux" chưa đạt (mới đạt Fcitx5) | `adapters/linux-ibus/` không có mã assist (`engine.c`/`main.c` không gọi `ime_assist_*`) | ✅ **Đã xử lý** (@f630ff6) — hoàn thiện `engine.c` + `tests/e2e_ibus.c` (Mode 1 lookup table, Mode 2 Backspace+Type, undo 5s, `ime_reset`) |
| R-7 | **major** | **3 bản sao** logic "plan sửa câu" (Rust / Swift / C) ⇒ vi phạm **G15** + dễ lệch hành vi 3 OS | `windows-tsf/src/assist.rs::plan_sentence_auto_fix` · `macos-imk/…/AssistModel.swift::planSentenceAutoFix` · `linux-common/src/compose.c::lc_plan_sentence_auto_fix` | ❌ **Còn mở** — đề xuất: đưa vào crate `assist` + FFI `ime_assist_*` (1 nguồn sự thật) |
| R-8 | **major** *(quy trình)* | Push **khi CI đỏ** + task **đã tick ✅** ⇒ vi phạm **G13** ⇒ thiếu luật "tick ✅ phải kèm CI xanh" | commit `c701600` (`ci-macos`/`ci-linux`/`ci-shared` FAILURE) + tick ✅ trong `P4-TASKS.md` | ✅ **Đã sửa** — thêm luật **G17** vào `docs/00-WORKFLOW.md` (§1 + §7 change log + §11 checklist) |
| R-9 | **major** | Lệch **semantics xoá lặp từ** giữa 3 OS: Windows/macOS so `sameWord && prevIsSpace`; Linux so `kind == IME_ASSIST_KIND_DUPLICATE_WORD` | `assist.rs` (plan_sentence_auto_fix) · `AssistModel.swift` (planSentenceAutoFix) · `compose.c` (`lc_plan_sentence_auto_fix`) | 🔄 **Đang sửa** — chọn 1 semantics chung (đề xuất: theo `kind`) + test hồi quy 3 OS |
| R-10 | **major** | AS-030/031/050 tick ✅ khi **chưa có bằng chứng verify tay** (DoD yêu cầu output CLI/screenshot + log sạch) | `docs/specs/verified-ops.md` **không có** entry assist · `P4-7-REVIEW-AS-021-022.md` §"Còn bằng chứng chạy thật" | ✅ **Đã sửa** — AS-030/031/050 → ⏳ + ghi chú "tick ✅ sau khi verify tay" |
| R-11 | minor | `lc_plan_sentence_auto_fix` so sánh span trong `span_buf[256]` ⇒ span > 255 byte bị **cắt âm thầm** ⇒ so sánh sai ⇒ có thể đề xuất sửa nhầm | `adapters/linux-common/src/compose.c` (`span_buf[256]`) + `lc_utf32_to_utf8` (`adapters/linux-common/src/utf.c` — `break` khi hết buffer) | 🔄 **Đang sửa** — cấp phát theo độ dài span hoặc so sánh trực tiếp trên UTF-32 |
| R-12 | minor | Fcitx5 Mode 1 dùng **candidate list chuẩn của IME** để hiện gợi ý lỗi (D13) nhưng `select()` xoá `delete_glyphs` rồi `commitString` ⇒ lệch cơ chế A2 (Backspace+Type) của Windows/macOS | `AssistCandidateWord::select` trong `adapters/linux-fcitx5/src/engine.cpp` | 🔄 **Đang sửa** — `63d8ef0` đã thêm fail-open (thiếu surrounding text ⇒ **không** xoá) + phím `Esc`/`1..3`; **còn lệch cơ chế A2** (vẫn xoá+commit thay vì Backspace+Type) |
| R-13 | minor | `AssistSettings`/`AssistDisplay` **định nghĩa 2 lần** trong Swift (app vs IMK) ⇒ 2 nguồn sự thật cho `assist.*` | `adapters/macos-app/Sources/TextVNAppLib/ConfigModel.swift` **và** `adapters/macos-imk/Sources/IMKLib/AssistModel.swift` | ❌ **Còn mở** (AGENTS.md §3: đổi mặc định config phải đồng bộ mọi bản) |
| R-14 | minor | `docs/release/ui-spec.md` **chưa có mục Trợ lý ngữ pháp** ⇒ UI overlay không có đặc tả 3 OS (Fassist-12) | `ui-spec.md` trước 2026-10-21 chỉ có §1–§7 | ✅ **Đã sửa** — thêm **§8 "Trợ lý ngữ pháp"** (nội dung panel + bảng phím + hành vi chung 3 OS) |

---

## Tổng hợp finding theo trạng thái

| Nhóm | Finding (`Fassist-n` · `R-n`) | Ghi chú |
|---|---|---|
| **Đã đóng** (xử lý trong plan/code/docs đã sửa) | 1, 3–11, 13–27, 28–45, 46–57, 59, 60, 64–79, 84–97, 103–110 | Thể hiện qua D1–D40, ADR-012/013/014, P4-TASKS ✅ (AS-001…060), hoặc docs/code đã sửa |
| **Superseded / đổi phân loại, giữ ID theo G6** | 2 (sai tiền đề → superseded bởi 32) · 56 (minor → major sau spike, rồi đã sửa) | ID không xoá |
| **Chấp nhận có ghi lý do (hết trách nhiệm v1)** | 74 (pop 1 char = hướng bảo toàn) · 80–83 (số đo "kỳ vọng" trên corpus mẫu — cần corpus thật + 2 người đánh giá) · 100, 102 (từ điển FST → v2) | Không còn hành động bắt buộc v1 |
| **Còn MỞ** | 12 (một phần), 58, 61, 63, 99, 101 | Xem danh sách bàn giao bên dưới |
| **Vòng R (AS-050/AS-060) — ĐÃ SỬA** | R-1, R-2, R-3, R-6, R-8, R-10, R-14 | Tick sai + tick khi CI đỏ + docs stale + IBus nối (@f630ff6) → đã sửa: `P4-TASKS.md` · `docs/00-WORKFLOW.md` (**G17**) · `docs/release/parity-checklist.md` · `docs/release/ui-spec.md` (§8) |
| **Vòng R — ĐANG SỬA** | R-4, R-5, R-9, R-11, R-12 | Lane Linux/Fcitx5: precondition D12 · undo A3 · semantics lặp từ 3 OS · cắt span UTF-8 · cơ chế A2 qua candidate list |
| **Vòng R — CÒN MỞ** | R-7, R-13 | gộp 3 bản sao logic plan sửa câu (G15) · `assist.*` 2 nguồn sự thật trong Swift |

## VIỆC CỒN MỞ — bàn giao người đọc sau

| ID | Mô tả | Nơi xử lý dự kiến |
|---|---|---|
| Fassist-58 | **2 người đánh giá độc lập** trên corpus thật (≥ 300 câu có lỗi, thống nhất ≥ 80%) trước khi recall/FPR thành gate CI / bật tính năng từng luật (D19) | Gate trước khi mở `mode≠off` cho user |
| Fassist-61 + Fassist-101 | **Đo lại RSS/binary/p95 của tài nguyên assist (kể cả `dict` FST) trong tiến trình IME/TIP thật** (S4a đo bằng CLI độc lập — không đại diện in-proc) | P4-2/AS-042 tích hợp → đo lại |
| Fassist-63 | Parity macOS: toast ≠ quy hoạch 2 OS kia; parity-checklist ghi macOS ✗ (adapter chưa nối assist) | P4-6/AS-060 lane sau |
| Fassist-12 (một phần) | Checklist manual UI/overlay 3 OS đầy đủ (khi Settings 3 OS nối assist) | P4-6 (Settings) + `parity-checklist.md` |
| Fassist-99 | Thông báo/patch upstream `hunspell-vi` + ghi `docs/compliance/` (phái sinh FST) | `docs/compliance/` khi HLS v2 |
| P4-10-F8 | `docs/compliance/assist-os-provider.md` + `LICENSES/` cho libspelling/enchant (LGPL, link động) + privacy notice "text rời tiến trình" (P4-10 §7.3.4) | `docs/compliance/` + `antivirus-false-positive.md` |
| P4-10 §9 (10 mục) | ⚠️ Chưa verify được (không có máy thật tương ứng): `availableLanguages` có `vi`; độ trễ NSSpellChecker/block main; provider Windows có gửi mạng không; `RPC_E_CANTCALLOUT` trong edit session; user dictionary đọc được; Δsize/ΔRSS `Win32_Globalization`… | Verify khi code/probe OS provider tương ứng |
| v2 (Fassist-100/102) | Luật spelling dùng trait `Dictionary` + đo recall/FPR có từ điển + nguồn từ điển **permissive/CC0** | Backlog v2 Phần 4 |
| **R-7** | **Gộp 3 bản sao** logic "plan sửa câu" (Rust `assist.rs` · Swift `AssistModel.swift` · C `compose.c`) thành **1 nguồn sự thật** (crate `assist` + FFI) — hiện tại vi phạm **G15** và đã sinh lệch hành vi (R-9) | P4-6 / refactor chung sau khi lane Linux ổn định |
| **R-13** | `AssistSettings`/`AssistDisplay` **định nghĩa 2 lần** trong Swift (`ConfigModel.swift` + `AssistModel.swift`) → gộp 1 chỗ + test đồng bộ mặc định (AGENTS.md §3) | P4-6 (lane macOS) |
| **R-4/R-5/R-9/R-11/R-12** | Lane Fcitx5: thêm **precondition D12** + **undo A3** · thống nhất **semantics lặp từ** 3 OS · bỏ **cắt span âm thầm** · cơ chế A2 qua candidate list | Đang sửa — **xanh CI + verify tay** thì tick ✅ AS-060 (theo **G17**) |

**Kết luận tổng:** Phần 4 đã trải qua ~18 lượt review (kế hoạch → cross-review → implement → spike → platform → dict) với **110 finding cấp số + 8 ID tạm**; còn mở **10 việc có ID** (đa số doc/compliance/đo lường, không blocker kiến trúc v1) + 10 mục "⚠️ chưa verify" của đặc tả provider OS. Trạng thái v1: luật objective + kill-switch + FFI + Windows TSF (overlay/sửa 1 từ/tự sửa câu) + glue OS provider 3 OS đã xong (AS-001…060); **macOS/Linux parity assist + Settings 3 OS chưa nối** (parity-checklist `assist.*` ✗).

**Cập nhật 2026-10-21 (vòng R, AS-050/AS-060 — 14 finding `R-1…R-14`):** Phần 4 nay có **124 finding** (110 `Fassist` + 8 ID tạm + 14 `R`) + **8 việc còn mở** (`Fassist-*` 10 → 10 việc cũ vẫn nguyên) + **3 việc mới** (R-6 IBus chưa nối · R-7 gộp 3 bản sao logic · R-13 config Swift 2 nguồn sự thật) và **5 việc đang sửa** ở lane Fcitx5 (R-4/R-5/R-9/R-11/R-12).
**Trạng thái thật sau vòng R:** Windows TSF — **code xong** (overlay + A2 + A3) nhưng **chưa verify tay** ⇒ ⏳ · macOS IMK — **đã nối**, còn lỗi ⇒ ⏳ · Linux — **Fcitx5 đã nối** (còn lỗi) / **IBus chưa làm** ⇒ ⏳ · **Settings cả 3 OS chưa có UI cho assist** ⇒ ✗.
**Quy trình:** đã bổ sung luật **G17** (`docs/00-WORKFLOW.md`) — *không tick ✅ khi CI của commit đó còn đỏ; tick ✅ phải kèm bằng chứng CI xanh (+ verify tay)* — do sự cố commit `c701600` push khi CI đỏ mà task đã tick ✅.
