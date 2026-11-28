<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- TASK: SF-9 · initial WIP: claude-opus-7e41b2d9 · completed by: antigravity-gemini-37c7c6bc -->
# SF-9 (Phần 1 / Part 1) — Một nguồn chữ UI dùng chung 3 nền tảng

## VI

- **Tiếp quản theo chỉ đạo chủ repo**: `claude-opus-7e41b2d9` đã thiết kế và viết phần lõi WIP
  (`data/ui/strings.toml`, `xtask/src/gen_ui.rs`) trước khi gặp giới hạn quota;
  `antigravity-gemini-37c7c6bc` đã sửa lỗi parser danh sách đa phần tử (`out` placeholder trong
  `gen_ui::parse`), chuẩn hoá `\r\n`→`\n` khi băm FNV-1a 64, chỉnh `render_rust` theo chuẩn
  `rustfmt`, thêm bộ lọc bỏ qua chú thích (`strip_comments`) và nhãn < 4 ký tự không khoảng trắng,
  nối vào `xtask`/`preflight`/`ci-shared` và thay toàn bộ bản chép tay trên 3 hệ điều hành.
- **Nguồn duy nhất (`data/ui/strings.toml`)**: 8 mục / 17 chuỗi dùng chung Windows, Linux, macOS:
  `about_tagline`, `about_details` (0..2), `url_discussions`, `url_issues`, `url_math_guide`,
  `method_labels` (0..3), `assist_mode_labels` (0..2), `assist_mode_hints` (0..2, giới hạn ≤ 55 ký
  tự 1 dòng).
- **Generator & gate (`xtask/src/gen_ui.rs`)**:
  - `cargo xtask gen-ui` sinh 3 tệp:
    - Windows: `tray/src/ui_strings_generated.rs` (`tray/src/lib.rs`, `settings_dialog.rs`, `menu.rs`)
    - Linux: `adapters/linux-settings/src/ui_strings_generated.h` (`settings_model.c`)
    - macOS: `adapters/macos-app/Sources/TextVNAppLib/UiStringsGenerated.swift` (`AppInfo.swift`, `SettingsView.swift`, `AppDelegate.swift`)
  - `cargo xtask check-ui` (bước mới trong `xtask/src/preflight.rs` và job `gen-tables` của
    `.github/workflows/ci-shared.yml`): fail nếu bất kỳ bản sinh nào lệch nguồn hoặc mã UI viết tay
    chứa literal trùng nguồn (đã thay toàn bộ 43 chỗ chép tay ban đầu thành 0).
- **Test parity (`tray/src/settings_dialog.rs`)**: giữ nguyên tên 4 test trong `regression-index.md`
  (`labels_match_linux_settings_panel`, `assist_mode_hints_match_every_platform`,
  `about_text_matches_every_platform`, `help_window_links_match_every_platform`), xác minh cả 3 OS
  tham chiếu đúng hằng/macro sinh mã mà không lặp literal.
- **Kiểm chứng & đột biến**:
  - `cargo xtask check-ui`: `8 mục / 17 chuỗi, 3 bản sinh khớp nguồn, 0 bản chép tay` (exit 0).
  - Đột biến D3: sửa `Telex` → `Telex_MUTATED` trong `tray/src/ui_strings_generated.rs` ⇒ `check-ui`
    đỏ (`exit 1`); `cargo xtask gen-ui` khôi phục ⇒ xanh.
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (Windows) và
    `--target x86_64-unknown-linux-gnu` (Linux): xanh.
  - `cargo test -p xtask`: **128 passed** (gồm 6 test `sf9_*`); `cargo test -p textvn-tray` (với
    `TEXTVN_SKIP_APPCONTAINER_E2E=1` do ACL ổ `D:` local; CI chạy AppContainer thật): **99 + 3 passed**
    (gồm 4 test parity).
  - `regression-index --todo-max 0`, `eval-leak`, `hygiene-docs/secrets/apis`, `check-version-sync`:
    đều exit 0.
- **Phần 2 (kế tiếp)**: `data/ui/settings_model.toml` + nội dung hướng dẫn theo OS và đồng bộ phần
  còn lại ngoài 8 mục trên.

## EN

- **Coordinated completion per repo owner directive**: `claude-opus-7e41b2d9` authored the initial
  WIP (`data/ui/strings.toml`, `xtask/src/gen_ui.rs`) before hitting a quota limit;
  `antigravity-gemini-37c7c6bc` fixed a multi-item list placeholder bug in `gen_ui::parse`,
  normalized `\r\n`→`\n` before FNV-1a 64 hashing, aligned `render_rust` formatting with `rustfmt`,
  added comment-aware hand-copy scanning (`strip_comments` + skipping `< 4`-char single-word labels),
  wired `gen-ui`/`check-ui` into `xtask`/`preflight`/`ci-shared`, and replaced all 43 hand-copied UI
  strings across all 3 platforms.
- **Single source (`data/ui/strings.toml`)**: 8 entries / 17 strings shared across Windows, Linux, and
  macOS: `about_tagline`, `about_details` (0..2), `url_discussions`, `url_issues`,
  `url_math_guide`, `method_labels` (0..3), `assist_mode_labels` (0..2), `assist_mode_hints` (0..2,
  enforced `<= 55` chars single line).
- **Generator & gate (`xtask/src/gen_ui.rs`)**:
  - `cargo xtask gen-ui` emits 3 generated files:
    - Windows: `tray/src/ui_strings_generated.rs` (`tray/src/lib.rs`, `settings_dialog.rs`, `menu.rs`)
    - Linux: `adapters/linux-settings/src/ui_strings_generated.h` (`settings_model.c`)
    - macOS: `adapters/macos-app/Sources/TextVNAppLib/UiStringsGenerated.swift` (`AppInfo.swift`, `SettingsView.swift`, `AppDelegate.swift`)
  - `cargo xtask check-ui` (added to `xtask/src/preflight.rs` and the `gen-tables` job in
    `.github/workflows/ci-shared.yml`): fails on any generated-file drift or any hand-written UI
    string literal duplicating the source (all 43 initial occurrences replaced, now 0).
- **Parity tests (`tray/src/settings_dialog.rs`)**: preserved all 4 `regression-index.md` test names
  (`labels_match_linux_settings_panel`, `assist_mode_hints_match_every_platform`,
  `about_text_matches_every_platform`, `help_window_links_match_every_platform`), verifying all 3 OSes
  reference the generated symbols rather than repeating literals.
- **Verification & mutation check**:
  - `cargo xtask check-ui`: `8 entries / 17 strings, 3 generated outputs match source, 0 hand copies`
    (exit 0).
  - D3 mutation: changing `Telex` → `Telex_MUTATED` in `tray/src/ui_strings_generated.rs` made
    `check-ui` fail (`exit 1`); `cargo xtask gen-ui` restored exit 0.
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (Windows), and
    `--target x86_64-unknown-linux-gnu` (Linux): green.
  - `cargo test -p xtask`: **128 passed** (including 6 `sf9_*` tests); `cargo test -p textvn-tray`
    (with `TEXTVN_SKIP_APPCONTAINER_E2E=1` for local `D:` drive AppContainer ACL; CI runs full
    AppContainer): **99 + 3 passed** (including all 4 parity tests).
  - `regression-index --todo-max 0`, `eval-leak`, `hygiene-docs/secrets/apis`, `check-version-sync`:
    all exit 0.
- **Part 2 (follow-up)**: `data/ui/settings_model.toml` + per-OS help text and remaining UI strings.
