# P4-3 — SPIKES chặn (S1–S4): kết luận trước khi làm P4-3/P4-4

- **Ngày:** 2026-10-20 · **Trạng thái:** 4/4 spike có kết luận · **Phần:** 4 — P4
- **Mục đích:** D18 (3 spike) + D25 (S4a/S4b) — **không có kết luận spike thì không bắt đầu P4-3/P4-4**.
- **Cross-ref:** `P4-0-MASTER-PLAN.md` §3.1 (D11–D31) · §8 (lộ trình) · `P4-2-CROSS-REVIEW-2026-10-20.md` (Fassist-35/36) · `P4-TASKS.md` (AS-007…AS-011)

---

## 0. Tóm tắt kết luận

| Spike | Câu hỏi | Kết luận | Ảnh hưởng |
|---|---|---|---|
| **S1** | TSF có mở được candidate list **sau khi commit** không? | **Chưa có** trong adapter (không có `ITfCandidateListUI`); nhưng **overlay tự vẽ** rẻ hơn nhiều và đủ cho Windows | **D30**: Windows dùng **overlay tự vẽ** (như D4), **không** implement `ITfCandidateListUI` ở v1 |
| **S2** | Đọc **vị trí caret** 3 OS? | **Windows: khả thi ngay** — adapter đã có `selection_range(ctx, ec)` + `GetText` (`edit_session.rs:414/484`). macOS/Linux: chưa có, cần làm ở P4-6 | Precondition D12 khả thi trên Windows từ P4-3 |
| **S3** | Nguồn từ điển VN? | **`1ec5/hunspell-vi`** (gốc GNU Aspell 0.60 VI — Hồ Ngọc Đức), **2 biến thể kiểu dấu** (`vi-DauCu` / `vi-DauMoi`), **6.642 mục**, **29 KB/biến thể** | Khớp `config.diacritic_style` (cũ/mới) của TextVN; FST dự kiến **< 100 KB** |
| **S4a** | Chi phí per-process của tài nguyên assist? | **Đạt gate**: peak RSS **8.86 MB** (cả tiến trình CLI), exe **993 KB**, rlib assist **241 KB**, **0.52 ms/câu** | **Không chặn P4-4** → có thể làm từ điển FST |
| **S4b** | Có nên dùng `harper-core` cho tiếng Anh? | **Không ở v1** (D9): English-only + churn ~hàng tháng + deps nặng (`zip`, `boxcar`) + FST 790 KB embed + ed2024 | Giữ D9; S4b chỉ là khảo sát v2, **không** cộng timeline v1 |

---

## 1. S1 — TSF candidate list sau commit

**Verify trong code** (`adapters/windows-tsf/src/`):

| Mục | Kết quả |
|---|---|
| `ITfCandidateListUI` / candidate list | ❌ **không có** trong adapter (grep `CandidateList` chỉ trúng `candidate_pipe_names` của IPC) |
| `ITfRange` / `ITfContext` / `GetText` | ✅ có (`edit_session.rs:199/210/414`) |
| Helper đọc vùng chọn | ✅ `selection_range(ctx, ec)` (`edit_session.rs:484`) |
| `ITfDisplayAttributeProvider` | ✅ có (`display_attr.rs`) — TIP còn **chủ động** trả `TF_LS_NONE` để app không gạch chân chữ đang gõ |

**Kết luận:**
1. Implement `ITfCandidateListUI` là **mảng UI lớn** (window class, vòng lặp message, tương thích app) — không đáng cho v1.
2. Windows dùng **overlay tự vẽ** (như D4) → khớp đúng ý user ("hiển thị cạnh con trỏ"), rẻ hơn, kiểm soát được (no-activate, không sửa format trong app).
3. Linux: overlay tự vẽ **không khả thi** (Wayland, adapter fcitx5 = C++) → dùng **candidate list chuẩn của engine** (`ibus_text_set_candidate_list` / Fcitx5 `candidateList`) — đã xác nhận API tồn tại, chưa dùng trong repo → task P4-6.
4. macOS: IMK — **candidate list do IMK server vẽ**, không phải app → cần spike riêng ở P4-6; tạm dùng toast.

→ **D30**: UI theo OS — Windows **overlay tự vẽ**; Linux **candidate list của engine**; macOS **toast** (chuyển sang IMK server ở P4-6 nếu khả thi). (Sửa bổ sung D13.)

## 2. S2 — Đọc vị trí caret (precondition D12)

| OS | Đường đọc vị trí (không đọc nội dung) | Trạng thái trong repo |
|---|---|---|
| Windows | `ITfContext::GetSelection` → `ITfRange` → `GetText` (chỉ lấy **độ dài/offset**) | ✅ **có sẵn**: `selection_range()` + `GetText` trong `edit_session.rs` |
| macOS | `AXSelectedTextRange` (`AXRange`) | ⚠️ `adapters/macos-imk/IMKTextTarget.swift` có abstraction nhưng chưa có đọc selection → P4-6 |
| Linux | AT-SPI `TextCaret`/`Text` interface | ⚠️ chưa có → P4-6 |

**Kết luận:** precondition D12 **khả thi ngay trên Windows** (giảm rủi ro lớn nhất của Mode 2). Đọc **vị trí**, không đọc nội dung → vẫn đúng D8.

## 3. S3 — Nguồn từ điển tiếng Việt

| Nguồn | Nội dung | License | Kích thước | Đánh giá |
|---|---|---|---|---|
| **`1ec5/hunspell-vi`** | `vi-DauCu.dic` / `vi-DauMoi.dic` — gốc **GNU Aspell 0.60 VI** (Hồ Ngọc Đức), bổ sung bởi Ivan Garcia / Nguyễn Xuân Minh | **GPLv2** theo `README-en.txt` (bản đóng gói OOo ghi GPLv3; `elastic/hunspell` khai `vi_VN: GPL3`) | **39,9 KB** mỗi file (29,1 KB text) · **6.642 mục** | ✅ **Nguồn chính** — tương thích GPL-3 của repo; **2 biến thể kiểu dấu** khớp `config.diacritic_style` (cũ/mới) |
| `wooorm/dictionaries` (npm `dictionary-vi`) | bản chuẩn hoá từ `hunspell-vi` | MIT (repo) nhưng `dic`/`aff` = **GPL-2.0** | ~40 KB | dùng như nguồn thứ 2 để đối chiếu |
| Wiktionary dumps (vi) | từ điển lớn hơn | **CC BY-SA 4.0** | lớn | ⚠️ **tránh ở v1**: nghĩa vụ attribution + share-alike cho dữ liệu phái sinh |

**Kết luận & việc cần làm (P4-4):**
1. Sinh **FST compact** từ `vi-DauMoi.dic` + `vi-DauCu.dic` (2 bản, hoặc 1 bản + cờ kiểu dấu) — dự kiến **< 100 KB** mỗi bản (6.642 mục).
2. **Attribution bắt buộc**: giữ `README` + tên tác giả gốc (Hồ Ngọc Đức) + link `1ec5/hunspell-vi`; cập nhật `REUSE.toml` (`LicenseRef-GPL-2.0-or-later` cho file sinh ra) + `docs/compliance/assist-dictionary.md`.
3. **Giới hạn đã biết**: 6.642 mục chỉ gồm **từ đơn** → đủ bắt lỗi chính tả cơ bản, **không** đủ cho gợi ý ngữ pháp nâng cao (D19 recall ≥ 80% có thể không đạt với từ điển này → cần đo thật ở P4-4).
4. Không dùng `.aff` (affix) của hunspell trong v1 → chỉ dùng danh sách từ.

## 4. S4a — Chi phí per-process (đo thật, Windows x64)

Đo bằng `textvn-cli assist check --file` (release build, 2.000 dòng tiếng Việt, 8.000 lỗi):

| Chỉ số | Kết quả | Gate (D25) | Kết luận |
|---|---|---|---|
| Peak working set (cả tiến trình) | **8,86 MB** | ≤ +5 MB so với nền (nền ≈ 8 MB) | ✅ đạt |
| `textvn-cli.exe` (release) | **993 KB** | binary ≤ +3 MB | ✅ đạt |
| `libtextvn_assist.rlib` | **240,7 KB** | — | nhỏ |
| Tốc độ check | **≈ 0,52 ms/câu** (1.039 ms / 2.000 dòng) | p95 ≤ 2 ms | ✅ đạt |
| Output | 8.000 lỗi → **đã giới hạn 50 dòng in** (Fassist-56 → major, đã sửa) | — | ✅ |

**Kết luận:** engine `assist` (không từ điển) **không tạo áp lực RSS** cho TIP in-process. Thêm từ điển FST ~< 100 KB (mmap, D15) → vẫn dưới gate. **Không chặn P4-4.**

## 5. S4b — `harper-core` cho tiếng Anh (v2, không chặn v1)

| Yếu tố | Số liệu (2026-10-20) | Ảnh hưởng |
|---|---|---|
| Bản mới nhất | 2.13.0 (2026-10-16), 123 bản, ~hàng tháng phát hành | nợ nâng cấp nếu vendor |
| Edition | 2024 → MSRV ≥ 1.85 | CI dùng stable ✓ nhưng tăng ràng buộc |
| Dependencies | `hashbrown`, `ordered-float`, `smallvec`, `levenshtein_automata`, `harper-brill`, `harper-thesaurus`, `bitflags`, **`zip`**, **`boxcar`** | bề mặt phụ thuộc + binary size |
| Từ điển | `dictionary.dict` = **789.666 B** FST **embed** trong crate | RSS per-process (TIP in-proc) → phải đo (gate S4a mở rộng) |
| Giá trị cho v1 | **0** — không có tiếng Việt | không dùng ở v1 (D9) |

**Kết luận:** giữ **D9**. Nếu v2 muốn hỗ trợ tiếng Anh → spike riêng (đo per-process RSS với `harper-core` thật, pin commit, attribution Apache-2.0 + NOTICE/REUSE).

## 6. Quyết định mới từ spike

| # | Quyết định | Lý do |
|---|---|---|
| **D30** | UI theo OS: **Windows = overlay tự vẽ** (bỏ `ITfCandidateListUI` ở v1) · **Linux = candidate list của engine** (IBus/Fcitx5 API có sẵn) · **macOS = toast** (IMK server vẽ candidate list, không phải app) | S1 + giới hạn Wayland + kiến trúc IMK |
| **D31** | Buffer câu trong `core` **luôn bật**, cap **256 ký tự** (≈ 1 KB/process) — không cần feature flag trong ABI | Đơn giản hơn D26 (config flag ⇒ đổi ABI `EngineOptions` + đồng bộ 3 OS); 1 KB là không đáng kể |

## 7. Việc còn lại sau 4 spike

- **P4-2a** (buffer câu ở core — D24/D31) → **P4-2** (FFI + trigger + toast) → **P4-3** (overlay + A2) → **P4-4** (từ điển FST + gợi ý từ) → **P4-5** (Mode 2) → **P4-6** (parity 3 OS).
- macOS/Linux vị trí caret (S2) → P4-6.
- Nguồn từ điển: **ship `1ec5/hunspell-vi`** + attribution; chưa dùng Wiktionary (CC BY-SA) ở v1.

> **C?p nh?t 2026-10-21 (Fassist-108):** hunspell-vi **kh?ng c?** trong Debian/Ubuntu (ch? Fedora/RHEL - Repology) v? license g?i = **GPL-2.0-only** => xem P4-9-REVIEW-DICT.md (Fassist-98) v? P4-10-OS-PROVIDER-SPEC.md.
