# AGENTS.md — luật bắt buộc cho MỌI agent làm việc trong repo TextVN

> File này là **điểm vào tự nạp** cho agent (ZCode/Codex/Claude/…). Nội dung
> tóm tắt; **luật đầy đủ và có ưu tiên cao hơn**: `docs/00-WORKFLOW.md`
> (G1–G20) · `docs/01-AGENT-HANDBOOK.md` (DoD/security/review) ·
> `docs/release/release-process.md` (checklist phát hành). Mâu thuẫn → làm
> theo file đó, không tự diễn giải.

## 0. Đọc trước khi làm bất cứ việc gì (trình tự)

0. **`cargo xtask board status --agent <id>` + `cargo xtask board inbox`** — ai đang làm gì, thư gửi
   mình, khoá phát hành (G21, `coordination/README.md`). Nhận việc từ `docs/10-shared/P0-7-TASKS-ARM-SF.md`.
1. `docs/00-WORKFLOW.md` — ground rules G1–G23 + quy trình audit/fixbug/dọn dẹp/CI.
2. `PLAN.md` + `docs/01-AGENT-HANDBOOK.md` — mục tiêu, DoD 8 mục (kèm Dream RSI), security.
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
| G18 | **Commit & Release bắt buộc cập nhật tài liệu liên quan & README song ngữ VI-EN toàn diện** — Mọi thay đổi tính năng/hành vi/API và mọi bản release PHẢI cập nhật docs liên quan (`README.md`, specs, ADR, CHANGELOG); `README.md` phải chính xác, đầy đủ bằng hoặc hơn template hiện tại, không thiếu sót; nêu bật các **tính năng vượt trội** (Rust core 3 OS an toàn bộ nhớ, TSF thuần không hook/inject, dự đoán P7 trie/frecency/sub-word, trợ lý ngữ pháp câu/từ ngữ cảnh, zero keylogging offline); **bắt buộc song ngữ Anh - Việt 1:1 đồng nhất 100% về cấu trúc, độ sâu thông tin và số liệu**; trình bày khoa học mạch lạc |
| G19 | **Dream RSI (P10) — mọi agent code bộ gõ buộc phải review & implement Dream RSI** — Gợi ý phải luôn tiến hóa, đo được bằng số: (a) thêm đường accept prediction mới → gắn hook `DreamRsi` (tăng `accepts` + `chars_saved` đúng công thức — xem `docs/specs/dream-rsi.md`); (b) không phá `keys_typed` (chỉ phím gõ chữ thật `ch>0`, loại key-up/injected/modifier/chord/điều hướng); (c) bench/gate đo savings-rate trên corpus qua `dream_rsi_snapshot()` — **tính năng làm savings-rate giảm so baseline = không merge**; (d) audit 3× phải có 1 view Dream RSI (metrics tiến hóa tăng dần, G8 chỉ counter) |
| G20 | **Mọi thay đổi code/tính năng/sửa lỗi → BẮT BUỘC tự động chạy trọn quy trình phát hành (`docs/release/release-process.md`), KHÔNG hỏi lại, KHÔNG bỏ bước** — Bất kỳ phiên nào có sửa mã nguồn (Rust/C/C++/Swift/script cài đặt/UI/packaging) đều PHẢI tự đi đủ từ đầu tới cuối: `cargo xtask preflight` → bump đủ 15 chỗ version + `Cargo.lock` + bộ docs phát hành B4 (`CHANGELOG.md` VI+EN, `README.md` 1:1 VI+EN, AppStream, `debian/changelog`, `PKGBUILD`/`.SRCINFO`, `build-release-report.md`) → commit ký GPG + push → 7/7 CI xanh → tag ký GPG `vX.Y.Z` → `release-candidate` 10/10 job xanh → verify 34 asset + GPG `Good signature` → sync `approved/vX.Y.Z/` (HTTP 200) → `sync-cask.yml` Homebrew tap xanh → pin hash Homebrew + AUR (`PKGBUILD` & `.SRCINFO`) + cập nhật report → `graphify update .` + push + CI cuối xanh. **Cấm hỏi "bạn có muốn phát hành không?" và cấm skip bất kỳ bước nào** (chỉ ngoại lệ khi người dùng ra lệnh tường minh không phát hành hoặc phiên chỉ sửa tài liệu thuần không đổi hành vi/code) |
| G21 | **Nhiều agent cùng làm = giao nhận việc qua `coordination/board.jsonl`** (`cargo xtask board`, luật R1–R14 trong `coordination/README.md`): đầu phiên `board status` + `board inbox`; nhận task bằng `board claim` (task phải đạt DoR, commit riêng tệp board + push); `board reserve` tệp sắp sửa; trước mọi commit `board guard $(git diff --cached --name-only)`; xong `board release --status done --commit` + `notify '*'`; tệp phát hành (CHANGELOG/README/report/version) chỉ agent giữ `board rm-lock` sửa — agent task viết `coordination/changes/<TASK>.md`; cấp mã lỗi mới qua `board note "alloc E.."` |
| G22 | **Không duplicate code/nghiệp vụ — một framework**: mọi nghiệp vụ chung (luật gõ, composition, phím chuyển, trợ lý, dự đoán/chữ mờ, nhận diện ô, IPC…) và **nguyên lý UI** (chữ hiển thị, nhóm cài đặt, gợi ý, hành vi popup) chỉ có MỘT nguồn trong lớp Rust dùng chung (`core/`, `textvn-session` P0-6, dữ liệu sinh mã `cargo xtask gen-*`); lớp OS chỉ khác ở cách tận dụng sức mạnh hệ điều hành (TSF/IMK/IBus/Fcitx5, toolkit UI native, API OS). Thêm logic vào `adapters/`/`tray/`/Swift/C khi nó thuộc nghiệp vụ chung = vi phạm; gặp bản sao cũ ⇒ mở task hợp nhất (P0-7 SF-*), test parity chỉ là tạm |
| G23 | **Phát hành không re-work (một lần là xong)**: gộp mọi thay đổi chờ vào MỘT bản; trước khi tag BẮT BUỘC chạy thử `gh workflow run release.yml --ref main` trên đúng commit và chờ mọi job xanh (B4b); lỗi CI/release ⇒ sửa cả LỚP lỗi trên mọi OS/target + gate chống tái diễn rồi mới chạy lại; tag lại chỉ khi publish chưa chạy |
| — | Ngôn ngữ: chỉ Rust + C/C++ + Swift native, không thêm runtime mới cho user |

## 2. Chiến lược phát hành — các mốc bắt buộc theo thứ tự

Chi tiết từng bước: `docs/release/release-process.md` (A1–A11 local, B1–B9
repo/docs). Tóm tắt các mốc hay bị bỏ sót:

1. **Preflight**: `cargo xtask preflight` phải PASS đủ gate rồi mới commit/tag.
2. **Version**: bump đủ 15 site (`cargo xtask check-version-sync`) — nhớ
   `testDefaultConfig` (Swift) + assertion trong `test_settings_model.c` +
   `schemas/config.v1.schema.json` khi đổi default; **cask Homebrew cần cả
   `version` lẫn `sha256`** (B7b, R2-116).
3. **B4 — bộ docs phát hành bắt buộc**:
   - `CHANGELOG.md`: entry **song ngữ VI+EN** + link compare `[N.N.N]: ...` cuối file.
   - `README.md`: cập nhật "Bản mới nhất" + 1 dòng tóm tắt đắt giá + trạng thái chữ ký (GPG/Sigstore); **chuẩn template mẫu hoặc hơn**, không thiếu sót bất kỳ bảng ma trận, kiến trúc, hay hướng dẫn nào; **nêu bật các tính năng vượt trội** so với IME truyền thống; **song ngữ VI và EN (mục `# English` cuối file) phải đồng nhất 100%** về nội dung, cấu trúc và độ sâu kỹ thuật (không tóm tắt lược bỏ).
   - `packaging/linux/appstream/*.metainfo.xml`: `<release version="N.N.N">` đứng đầu.
   - `build-release-report.md`: mục bản mới với bảng gate A1–A4, perf, release run.
   - Ghi mã sự cố mới (`B{n}`/`E{n}`/`R{n}`). Thiếu mục nào = CHƯA ĐƯỢC TAG.
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
