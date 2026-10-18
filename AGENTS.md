# AGENTS.md — luật bắt buộc cho MỌI agent làm việc trong repo TextVN

> File này là **điểm vào tự nạp** cho agent (ZCode/Codex/Claude/…). Nội dung
> tóm tắt; **luật đầy đủ và có ưu tiên cao hơn**: `docs/00-WORKFLOW.md`
> (G1–G16) · `docs/01-AGENT-HANDBOOK.md` (DoD/security/review) ·
> `docs/release/release-process.md` (checklist phát hành). Mâu thuẫn → làm
> theo file đó, không tự diễn giải.

## 0. Đọc trước khi làm bất cứ việc gì (trình tự)

1. `docs/00-WORKFLOW.md` — ground rules G1–G16 + quy trình audit/fixbug/dọn dẹp/CI.
2. `PLAN.md` + `docs/01-AGENT-HANDBOOK.md` — mục tiêu, DoD 7 mục, security.
3. `specs/verified-ops.md` + `specs/win-test-common-errors.md` +
   `specs/project-common-errors.md` — cái đã kiểm chứng + lỗi đã gặp (đừng lặp).

## 1. Luật không được phá (vi phạm = task chưa xong)

| ID | Luật |
|---|---|
| G2 | Không `git add -A` — chỉ add list file tường minh; không đụng WIP của agent khác |
| G3/G4 | Lỗi mới → ghi NGAY thành mã (`A{n}`/`E{n}`/`R{n}`/`F{n}`) kèm nguyên nhân + chặn tái diễn; tool chạy được → ghi vào `specs/verified-ops.md` |
| G5 | Xong việc → commit → push → `graphify update .` → commit `graphify-out/` → push |
| G8 | Không bao giờ log nội dung phím/text người dùng |
| G13 | Push đỏ CI → xử lý ngay trong phiên, chưa xanh chưa Done |
| G16 | **Phát hành version mới = đi ĐỦ Checklist A (local) + Checklist B (repo/docs) trong `docs/release/release-process.md`** — không tag khi CI của commit gắn tag chưa xanh |
| — | Ngôn ngữ: chỉ Rust + C/C++ + Swift native, không thêm runtime mới cho user |

## 2. Chiến lược phát hành — các mốc bắt buộc theo thứ tự

Chi tiết từng bước: `docs/release/release-process.md` (A1–A11 local, B1–B9
repo/docs). Tóm tắt các mốc hay bị bỏ sót:

1. **Preflight**: `cargo xtask preflight` phải PASS đủ gate rồi mới commit/tag.
2. **Version**: bump đủ 14 site (`cargo xtask check-version-sync`) — nhớ
   `testDefaultConfig` (Swift) + assertion trong `test_settings_model.c` +
   `schemas/config.v1.schema.json` khi đổi default; **cask Homebrew cần cả
   `version` lẫn `sha256`** (B7b, R2-116).
3. **B4 — bộ docs phát hành**: CHANGELOG entry + link compare cuối file ·
   README "Bản mới nhất" + tóm tắt · appstream `<release>` đứng đầu ·
   `build-release-report.md` mục bản mới. Thiếu mục nào = chưa được tag.
4. **Tag** git-signed (commit.gpgsign/tag.gpgsign đã bật) → push → `release-candidate`
   4/4 xanh → verify 34 asset + chữ ký (B7).
5. **approved**: sync `vX.Y.Z/` byte-identical lên branch `approved` (B7c, raw URL 200).
6. **Homebrew tap** (B7d): `gh workflow run sync-cask.yml -R hunglinhpt/homebrew-tap`
   → run xanh + `Casks/textvn.rb` trên tap có đúng version/hash bản mới. Tap tự
   sync mỗi 6 giờ — không cần PAT, không thêm bước trong release.yml.
7. **Memory dự án** cập nhật nếu có bài học mới (B9).

## 3. Kiến trúc & ràng buộc kỹ thuật cốt lõi

- Một engine Rust (`core/`) dùng chung cho Windows TSF, macOS IMK, Linux
  IBus/Fcitx5 — sửa engine phải nghĩ 3 nền tảng; UI/settings 3 nền tảng buộc
  giống nhau (spec `docs/release/ui-spec.md`).
- `core/` không I/O, không panic qua FFI (`catch_unwind`), config qua
  `config.v1.json` (spec P0-3); đổi mặc định config phải đồng bộ: serde
  `Config::default` + `EngineOptions::default` + `ConfigModel.swift` + Linux
  `settings_model.c` + schema + P0-3 + test Swift/C liên quan.
- Mọi thay đổi engine phải có test hồi quy trong `core/tests/` (corpus thật,
  không test đơn lẻ vô nghĩa).
