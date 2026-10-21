# P4-9 — REVIEW (3 vòng) kết quả Agent Dict: từ điển tiếng Việt (AS-040/AS-041)

- **Ngày:** 2026-10-21 · **Đối tượng:** commit `1b2fc48` "P4-8: hoàn thành baseline chất lượng từ điển VN" (+ `abe0bdc` graphify) · **Reviewer:** Agent Core
- **Kết luận:** **Chấp nhận HẠ TẦNG** (trait `Dictionary` + crate `dict` FST/mmap + share global) — với **3 điều kiện**: (1) license phải sửa, (2) **không phát hành** file `.dic`, (3) **chưa bật** tính năng cho người dùng (v1 dùng OS provider — D33).
- **Cross-ref:** `P4-0-MASTER-PLAN.md` §3.1 (D33/D37) · `P4-8-QUALITY-BASELINE.md` (chất lượng hiện tại) · `docs/adr/README.md` (ADR-012/013/**014**)

---

## 0. Scope thực tế của agent Dict

| Hạng mục | Nội dung | Đánh giá |
|---|---|---|
| `assist/src/dict.rs` | trait `Dictionary` (`contains`/`suggest`) + `NoDictionary` (fallback "mọi từ đều đúng") | ✅ **đúng hướng**: crate `assist` giữ **0-dep, portable** (D32); trait chỉ là giao diện, không I/O |
| `dict/` (crate mới) | `build.rs` (sinh FST từ `.dic`), `fst.rs`, `mmap.rs` (mmap), `reader.rs`, `lib.rs` (`load`/`set_global`/`global`) | ✅ FST **mmap** đúng D15 (không parse 6.643 từ mỗi process); `Arc<Dictionary>` + global = share **immutable** đúng D25 |
| `data/assist/vi-DauCu.dic`, `vi-DauMoi.dic` | 2 biến thể kiểu dấu (cũ/mới) khớp `config.diacritic_style` | ⚠️ **license phải xem lại** (Fassist-98) |
| `REUSE.toml` | khai license cho `.dic` | ⚠️ sai id → đã sửa (Fassist-98) |
| `corpus/assist/vi_regressions.ass` | +20 dòng case hồi quy | ✅ tốt (khóa hành vi) |

## 1. Vòng 1 — SA / QA

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-99** | minor | Điều khoản gốc *"If you modify it, please send me the modifications"* — ta **phái sinh FST** (derivative) ⇒ nên gửi thông báo/patch upstream hoặc ghi rõ trong `docs/compliance/` | Ghi trong `LICENSES/LicenseRef-hunspell-vi.txt` + `docs/compliance/` (task còn mở) |
| **Fassist-100** | minor | **Chưa có luật nào dùng trait `Dictionary`** → `assist::check` vẫn chỉ có luật objective ⇒ **chưa** đo được recall/FPR *có từ điển*; scope "baseline chất lượng từ điển" **chưa đạt** | Giữ v1 = luật objective + OS provider (D33); tách task "luật spelling + đo recall/FPR" sang v2 (nguồn license sạch) |
| **Fassist-101** | minor | Chưa đo **RSS/p95 theo gate S4a** (per-process) cho crate `dict` | Đo ở P4-2 khi có tích hợp thật (F61) |

## 2. Vòng 2 — Audit (license) — **phát hiện nghiêm trọng**

**Fassist-98 (major):** `REUSE.toml` khai `data/assist/*.dic` là **"GPL-2.0-or-later"** nhưng điều đó **chưa được chứng minh**:

| Bằng chứng | Nội dung |
|---|---|
| Nguồn | `1ec5/hunspell-vi` → chuyển từ **GNU Aspell 0.60 VI** → `Viet39K.txt` của **Hồ Ngọc Đức** (Đại học Leipzig) |
| File `Copyright` gốc (tải `aspell6-vi-0.01.1-1.tar.bz2` từ ftp.gnu.org, **346 byte**) | *"This word list was created using Viet39K.txt … The word list is distributed under GNU General Public License. If you modify it, please send me the modifications."* |
| ⇒ | **Không nêu phiên bản**, **không có** "or any later version" ⇒ nếu hiểu là **GPL-2.0-only** thì **không tương thích** GPL-3.0-or-later (tương thích **một chiều**) ⇒ file `.dic` **và FST phái sinh** không được phát hành trong dự án GPL-3.0 |

**Fix đã thực hiện (triệt để):**
1. `REUSE.toml`: id → **`LicenseRef-hunspell-vi`** (không khẳng định "or-later" khi chưa chứng minh).
2. `LICENSES/LicenseRef-hunspell-vi.txt`: trích **nguyên văn** thông báo gốc + link nguồn + **cảnh báo tương thích** + quyết định "không phát hành".
3. `installer/windows/TextVN-setup.iss`: `Source: "..\..\data\*"` → thêm `Excludes: "assist\*.dic"` ⇒ **từ điển không vào gói phát hành** (trước đó nó sẽ **được** ship do `recursesubdirs`).
4. `docs/adr/README.md`: **ADR-014** ghi quyết định + điều kiện.
5. Xoá `LICENSES/GPL-2.0-or-later.txt` (đã thêm ở commit trước) vì không còn id nào tham chiếu → `reuse lint` **789/789 ✓**.

## 3. Vòng 3 — PO / QA (giá trị & phạm vi)

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-102** | minor | Hạ tầng đã có nhưng **chưa dùng được** cho người dùng (chưa có luật) ⇒ dễ hiểu nhầm là "đã có chính tả tiếng Việt" | Ghi rõ trong plan: v1 = **luật objective + OS spellcheck**; từ điển FST = **hạ tầng v2** (chờ nguồn license sạch) |
| **Fassist-103** | minor | Chưa có **test** chứng minh FST/mmap đúng (ví dụ: tra cứu từ có/không, gợi ý) | Thêm test ở task P4-2 (khi tích hợp) |

## 4. Kết luận & trạng thái

| Quyết định | Trạng thái |
|---|---|
| Giữ hạ tầng `dict` + trait `Dictionary` | ✅ giữ (đúng D32/D15/D25) |
| Dùng `.dic` hunspell-vi trong **sản phẩm** | ❌ **không** (ADR-014) |
| Bật luật spelling cho người dùng ở v1 | ❌ chưa (D33: OS provider) |
| Cần thêm | task v2: nguồn từ điển **permissive/CC0** + luật spelling + đo recall/FPR + test FST |
