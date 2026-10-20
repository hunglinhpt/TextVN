# P4-4 — Tận dụng năng lực check ngôn ngữ **có sẵn của OS/app** (OS-first) — nghiên cứu & thiết kế

- **Ngày:** 2026-10-21 · **Trạng thái:** Nghiên cứu xong → **D32–D37** (bổ sung D1–D31) · **Phần:** 4 — P4
- **Động cơ (user 2026-10-21):** *"nghiên cứu tìm thêm phương pháp tận dụng năng lực hệ điều hành và app đang bật để gõ có sẵn chế độ check và rà soát ngôn ngữ, tối ưu sử dụng nó để TextVN đạt performance + chất lượng tốt nhất trên mọi nền tảng"*.
- **Cross-ref:** `P4-0-MASTER-PLAN.md` §3.1 (D1–D37) · §5.2 (từ điển) · `P4-3-SPIKES.md` (S3/S4a) · `P4-2-CROSS-REVIEW-2026-10-20.md` · `docs/adr/README.md` (ADR-012/013)

---

## 0. Kết luận 1 dòng

**Chiến lược từ điển đảo chiến:** từ *"tự mang từ điển + tự làm spellcheck"* → sang **"OS-first"**: dùng **spell-checker có sẵn của OS** (Windows Spell Checking API · macOS `NSSpellChecker` · Linux GNOME `libspelling`/enchant) làm **nguồn chính**; từ điển FST của ta chỉ là **fallback** khi OS không có checker cho `vi`; luật **ngữ pháp câu** vẫn là của ta (đồng nhất 3 OS).

---

## 1. API có sẵn — đã verify (2026-10-21)

| OS | API chính thức | Ngôn ngữ | Ngữ pháp? | Ghi chú quan trọng |
|---|---|---|---|---|
| **Windows** | **Spell Checking API** — `ISpellCheckerFactory` (`spellcheck.h`, Win8+): `IsSupported(tag)` · `CreateSpellChecker(tag)` · `get_SupportedLanguages`; `ISpellChecker::Check(text) → IEnumSpellingError` (`StartIndex`,`Length`,`CorrectiveAction` = DELETE/REPLACE/GET_SUGGESTIONS, `Replacement`) · `Suggest(word) → IEnumString` | theo **spell-checker đã cài** (mặc định `en-US`; **`vi-VN` chỉ khi user cài language pack / proofing tools**) | ❌ spelling (+ auto-replace khi chắc chắn) | COM **in-proc** (`CLSCTX_INPROC_SERVER`) → **không sinh tiến trình mới**; nhưng *provider* có thể chạy **out-of-proc** (restricted context) → gọi có thể block → phải **timeout + worker thread** |
| **macOS** | **`NSSpellChecker`** (AppKit): `checkSpelling(of:startingAt:language:)` · `guesses(forWordRange:in:language:inSpellDocumentWithTag:)` · `hasCorrection` · `language(for:)` · `availableLanguages` | `availableLanguages` (**có `vi`** nếu macOS cài từ điển) | doc ghi *"Checking Strings for Spelling **and Grammar**"* — nhưng **API ngữ pháp công khai chưa rõ** → v1 **không** hứa dùng ngữ pháp của OS | native, có cache; user dictionary của hệ thống dùng chung với app khác |
| **Linux** | **GNOME `libspelling`** (LGPLv2.1+, GTK4 — cùng hướng **GNOME Text Editor/Builder**): `spelling_checker_get_default()` · **`spelling_checker_check_ranges(snapshot UTF-8, ranges tính bằng ký tự Unicode) → DexFuture` → `SpellingMistakes`** (offset **ký tự Unicode**) | bất kỳ dict **cài trong hệ thống** (enchant-2 → `hunspell-*`) — `hunspell-vi` là gói sẵn của Debian/Ubuntu/Fedora | ❌ spelling | `check_ranges` **bất đồng bộ** + **offset Unicode** ⇒ **khớp** với `assist` (char offset) |

**Nguồn:** MS Learn "Spell Checking API" + `ISpellCheckerFactory`/`ISpellChecker`; Apple `NSSpellChecker` (doc JSON 2026-10-21); `GNOME/libspelling` README + blog GNOME 2023-07-12.

## 2. So sánh "OS-first" vs "tự mang từ điển"

| Tiêu chí | OS-first (mới) | Tự mang từ điển (cũ) |
|---|---|---|
| Chất lượng từ điển | Tốt hơn: từ điển của hệ thống + **user dictionary** dùng chung với app khác | Tĩnh, không cập nhật, chỉ từ đơn (6.642 mục — S3) |
| License/attribution | Không phải ship dữ liệu của ta (trừ fallback) | Phải attribution GPL + `REUSE.toml` |
| Kích thước/footprint | ≈ 0 (dùng thư viện OS) | FST ~< 100 KB (2 bản) + build pipeline |
| Phụ thuộc nền tảng | Có (spellcheck.h / AppKit / libspelling) → mỗi OS 1 lớp glue | Không |
| Tốc độ | Provider OS cache sẵn, có thể out-of-proc → cần async | Tuyệt đối nhanh, offline định |
| Độ phủ `vi` | Windows: **mặc định KHÔNG có** `vi-VN`; Linux: cần user cài `hunspell-vi`; macOS: có sẵn | Luôn có (nếu ta ship) |
| Rủi ro AV/privacy | **Provider là tiến trình của OS** → ta không giữ text trong tiến trình lạ (rất tốt cho `antivirus-false-positive.md` §8) | Ta giữ text trong tiến trình của mình (TIP in-proc) |

→ **Kết luận: OS-first là chiến lược chính; FST của ta = fallback bắt buộc cho Windows (và cho máy không cài gì).**

## 3. Kiến trúc provider (đề xuất)

```
assist/ (crate Rust thuần, 0-dep — KHÔNG gọi OS)
  ├─ GrammarProvider  (luật câu của ta: L0–L12)         ← luôn có
  └─ trait SpellProvider { fn check_word(&self, w) -> Option<Vec<String>> }
        impl: BuiltinSpellProvider (FST fallback) | OsSpellProvider (adapter/OS glue)
```

| Mục | Chỗ đặt | Lý do |
|---|---|---|
| `assist` crate | Giữ **0-dep, portable** — chỉ luật + trait | test được 3 OS, không phụ thuộc OS trong lõi (D23 đã chốt biên FFI) |
| Glue OS (`ISpellCheckerFactory` · `NSSpellChecker` · `libspelling`) | crate `ffi` (Windows) / `adapters/macos-imk` (Swift) / `adapters/linux-*` (C) | mỗi OS 1 chỗ, theo D23 |
| Fallback FST | crate `assist` (chỉ đọc file FST khi được cấu hình) | dùng khi OS không có `vi` |

## 4. Quyết định mới (bổ sung D32–D37)

| # | Quyết định | Lý do |
|---|---|---|
| **D32** | Kiến trúc **Provider**: `assist` giữ trait + luật của ta; **glue OS** nằm ở biên FFI/adapter theo D23 | `assist` giữ 0-dep & portable; OS-specific code chỉ 1 nơi/OS |
| **D33** | **OS-first**: dùng checker OS khi `IsSupported("vi"/"vi-VN")`; **fallback** FST của ta khi OS không có | Chất lượng + user dictionary + footprint tốt hơn (F67) |
| **D34** | **Không trùng lặp app**: app đã có spellcheck native (VSCode/Word/Chrome/gedit/GNOME Text Editor) → assist chỉ làm **ngữ pháp câu** (per-app trong `appdb`) | Ít nhiễu, ít CPU, UX đúng (F72) |
| **D35** | **L0 dùng OS** trước heuristic: macOS `NLLanguageRecognizer` · Linux ICU (`uloc_getLanguage`) · Windows locale/MUI | Thay đoán bằng dữ liệu hệ thống (F68) |
| **D36** | Provider OS gọi **bất đồng bộ, timeout ≤ 50 ms, fail-open** (provider có thể out-of-proc) | Không chặn input path (`PLAN.md` §5.5) |
| **D37** | Từ điển FST của ta **vẫn ship** (fallback) + attribution GPL đầy đủ | Windows mặc định không có `vi-VN` ⇒ không thể bỏ (F67) |

## 5. Finding mới (Fassist-67..72) — rút ra từ nghiên cứu này

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-67** | major | "OS-first ⇒ không cần từ điển riêng" là **sai**: Windows mặc định **không có** spellchecker `vi-VN`; Linux cần user cài `hunspell-vi` | **D33/D37**: OS-first là *đường chính*, FST là *fallback bắt buộc* (P4-4 giữ nguyên, đổi vai trò) |
| **Fassist-68** | major | macOS: doc ghi *"spelling and grammar"* nhưng **API ngữ pháp công khai chưa rõ** | **Không** hứa "ngữ pháp của OS" ở v1 → ngữ pháp luôn là luật của ta ⇒ **đồng nhất 3 OS** |
| **Fassist-69** | minor | `libspelling`/enchant là **LGPL** ⇒ ràng buộc license + phải có mặt trên máy | Link động OK; cần graceful fallback (thiếu thư viện → dùng FST của ta) + ghi `deny.toml`/`REUSE.toml` |
| **Fassist-70** | minor | Provider Windows có thể chạy **out-of-proc** → lời gọi COM có thể **block** | **D36**: gọi từ worker thread + timeout 50 ms + fail-open (bỏ qua câu đó) |
| **Fassist-71** | minor | OS có **user dictionary** riêng → từ người dùng tự thêm có thể bị báo sai | Đọc danh sách user words qua API khi có → bỏ qua (Windows: `%AppData%\Microsoft\Spelling\default.dic`; enchant: user hunspell) |
| **Fassist-72** | minor | Trùng lặp với spellcheck của app (VSCode/Word/Chrome/gedit) → double underline + tốn CPU | **D34**: per-app trong `appdb`: `assist.scope = "grammar" \| "full" \| "off"` (mặc định `grammar` cho app có spellcheck) |

## 6. Tác động lộ trình

| Pha | Thay đổi |
|---|---|
| P4-1 (xong) | không đổi (luật + corpus + CLI) |
| P4-2a/P4-2 | không đổi (buffer câu + FFI + trigger) — **thêm** cổng provider trong FFI (`ime_assist_check_v1` → `{language, provider_kind}`) |
| P4-3 | overlay + A2 (như cũ) — **dùng** provider OS cho gợi ý từ nếu có |
| **P4-4 (đổi vai trò)** | (a) **FST fallback** từ `hunspell-vi` (như đã định) + (b) **glue provider OS** (Windows `ISpellCheckerFactory`; macOS `NSSpellChecker`; Linux `libspelling`) — theo thứ tự: **glue OS trước** (rẻ hơn, không cần data), FST fallback sau |
| P4-5 | Mode 2 dùng provider OS cho **auto-fix** khi nó trả `CORRECTIVE_ACTION_REPLACE` (chắc chắn) |
| P4-6 | parity: cả 3 OS dùng cùng trait + cùng corpus; ghi `ui-spec`/`parity-checklist` |

## 7. Rủi ro mới

| # | Rủi ro | Mức | Giảm thiểu |
|---|---|---|---|
| R14 | OS provider **chậm/không có** → assist im lặng | Trung bình | timeout + fail-open + fallback FST |
| R15 | Ngữ pháp OS khác nhau giữa 3 OS → trải nghiệm lệch | Trung bình | **ngữ pháp luôn của ta** (F68) → chỉ *gợi ý từ* là của OS |
| R16 | Phụ thuộc thư viện hệ thống (libspelling) vắng mặt | Thấp | capability-detect + fallback |
| R17 | Provider OS có thể là **tiến trình của bên thứ ba** (Windows) → rủi ro quyền riêng tư | Thấp | provider OS đã là hạ tầng của hệ thống (ngoài phạm vi `antivirus-false-positive.md` §8 vì không phải tiến trình của TextVN) |

---

## 8. Review 3 lượt (SA/QA → Audit → PO/Design) — sau khi nghiên cứu

**Lượt 12 — SA/QA**
| Mức | Finding | Xử lý |
|---|---|---|
| major | OS-first làm **thay đổi vai trò** của P4-4 (từ "nguồn chính" → "fallback") mà plan chưa cập nhật | D33/D37 + cập nhật §6 lộ trình + `P4-TASKS` AS-040…049 |
| minor | `assist` (crate 0-dep) không thể gọi OS ⇒ provider phải nằm ở biên | D32 (glue ở `ffi`/adapter theo D23) — giữ `assist` portable |
| minor | Provider OS có thể **out-of-proc** ⇒ lời gọi COM block ⇒ phá budget input | D36 (async + timeout 50 ms + fail-open) |

**Lượt 13 — Audit**
| Mức | Finding | Xử lý |
|---|---|---|
| major | `libspelling`/enchant = **LGPL**; Windows provider có thể là tiến trình bên thứ ba | F69/R16/R17: link động + fallback + ghi `deny.toml`/`REUSE.toml`; provider OS **không phải** tiến trình của TextVN ⇒ ngoài phạm vi §8 AV |
| minor | Attribution GPL của `hunspell-vi` vẫn bắt buộc cho fallback | D37 + `docs/compliance/assist-dictionary.md` (task P4-4) |

**Lượt 14 — PO/Design**
| Mức | Finding | Xử lý |
|---|---|---|
| major | Ngữ pháp **của OS khác nhau** giữa 3 OS (đặc biệt macOS) ⇒ trải nghiệm lệch, khó test | F68/R15: **ngữ pháp luôn là luật của ta**; chỉ *gợi ý từ* là của OS ⇒ 1 hành vi giống nhau 3 OS |
| minor | Trùng lặp với spellcheck của app (VSCode/Word/Chrome/gedit) | F72/D34: `assist.scope` per-app (`grammar`/`full`/`off`) |
| minor | Người dùng thêm từ riêng vào OS → bị báo sai | F71: đọc user dictionary khi API cho phép |
