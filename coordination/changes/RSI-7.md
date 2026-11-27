# RSI-7 — Cổng ngôn ngữ cho chữ mờ / Ghost language gate

> Mảnh ghi chú cho agent giữ khoá phát hành (R3) — gộp vào CHANGELOG/README/report khi phát hành 0.2.96.

## VI

**Cổng ngôn ngữ cho chữ mờ (RSI-7, P0-7 Luồng R)** — không gợi ý từ tiếng Anh cho tiền tố là âm
tiết/tiền tố Việt hợp lệ khi không có ngữ cảnh tiếng Anh (từ ngay trước là từ tiếng Anh thật mở
cổng lại — code-switch `gửi report` không vỡ). Kèm E84: lọc âm tiết "Việt" nhiễm từ tiếng Anh
ngoài từ điển Việt (`guy/king/thing/my/by/kong/on/po/them` từ corpus OpenSubtitles) tại điểm đọc
và trong `cargo xtask gen-vocab`. Số đo bench RSI-1: KSR-oracle VI giữ nguyên 267/2636 phím
(10,1%), EN giữ nguyên 32/725 (4,4%); nhiễu VI 45,2 → 33,7/100 phím (−25%; cổng chặn 100% nguồn
EN tại tiền tố Việt — phần nhiễu còn lại là nguồn VN, phạm vi RSI-4). Test: `rsi_7_*` trong
`core/tests/text_prediction.rs`, `e84_*` trong `core/tests/invariants.rs`.

Thay đổi tệp: `core/src/post/predict.rs` (cổng `en_context` + `vn_prefix_possible` + lọc E84),
`core/src/lib.rs` (tính ngữ cảnh EN từ `sentence_tail`), `core/src/post/restore_en.rs` (chặn
Tier-2/allow_full_en cho tiền tố Việt), `xtask/src/gen_vocab.rs` (luật gen-vocab E84), test.

## EN

**Ghost language gate (RSI-7, P0-7 lane R)** — no static English suggestions while the typed
prefix can still grow into a valid Vietnamese syllable and there is no English context (the
previous word being a real English word re-opens the gate — code-switching like `gửi report`
keeps working). Includes E84: English-infested "Vietnamese" syllables from the OpenSubtitles
frequency corpus (`guy/king/thing/my/by/kong/on/po/them`) are filtered out at read time and in
`cargo xtask gen-vocab`. Bench numbers (RSI-1): Vietnamese KSR-oracle unchanged at 267/2636 keys
(10.1%), English unchanged at 32/725 (4.4%); Vietnamese noise 45.2 → 33.7/100 keys (−25%; the
gate removes 100% of the EN source at Vietnamese prefixes — the remaining noise is VN-source,
RSI-4 scope). Tests: `rsi_7_*` in `core/tests/text_prediction.rs`, `e84_*` in `core/tests/invariants.rs`.

Files changed: `core/src/post/predict.rs` (`en_context` gate + `vn_prefix_possible` + E84 filter),
`core/src/lib.rs` (EN context from `sentence_tail`), `core/src/post/restore_en.rs` (block Tier-2/
allow_full_en for Vietnamese prefixes), `xtask/src/gen_vocab.rs` (E84 gen rule), tests.
