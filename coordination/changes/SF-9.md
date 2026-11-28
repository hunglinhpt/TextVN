<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- TASK: SF-9 · initial WIP: claude-opus-7e41b2d9 · completed (Part 1 + Part 2) by: antigravity-gemini-37c7c6bc -->
# SF-9 (Phần 1 + Phần 2 / Part 1 + Part 2) — Một nguồn chữ UI & mô hình cài đặt dùng chung 3 nền tảng

## VI

- **Tiếp quản theo chỉ đạo chủ repo**: `claude-opus-7e41b2d9` đã thiết kế và viết phần lõi WIP
  (`data/ui/strings.toml`, `xtask/src/gen_ui.rs`) trước khi gặp giới hạn quota;
  `antigravity-gemini-37c7c6bc` đã sửa lỗi parser danh sách đa phần tử (`out` placeholder trong
  `gen_ui::parse`), chuẩn hoá `\r\n`→`\n` khi băm FNV-1a 64, chỉnh `render_rust` theo chuẩn
  `rustfmt`, thêm bộ lọc bỏ qua chú thích (`strip_comments`) và nhãn < 4 ký tự không khoảng trắng,
  nối vào `xtask`/`preflight`/`ci-shared` và thay toàn bộ bản chép tay trên 3 hệ điều hành ở Phần 1,
  sau đó hoàn thiện **Phần 2** (`data/ui/settings_model.toml` + `charset_labels` + đối chiếu 1:1 schema).
- **Nguồn duy nhất Phần 1 (`data/ui/strings.toml`)**: 9 mục / 21 chuỗi dùng chung Windows, Linux, macOS:
  `about_tagline`, `about_details` (0..2), `url_discussions`, `url_issues`, `url_math_guide`,
  `method_labels` (0..3), **`charset_labels` (0..3)**, `assist_mode_labels` (0..2), `assist_mode_hints`
  (0..2, giới hạn ≤ 55 ký tự 1 dòng).
- **Nguồn duy nhất Phần 2 (`data/ui/settings_model.toml`)**: 5 nhóm (`typing`, `options`, `assist`,
  `math`, `system`) + 18 mục `[[setting]]` kèm `group`, `config_key`, `kind` (`bool`/`enum`/`u32`),
  `values` (cho enum), `default` và `depends_on`:
  - `check-ui` dùng parser JSON chuẩn của repo (`check_mac_targets::parse_json`, G22) để đối chiếu
    1:1 mọi `[[setting]]` với `schemas/config.v1.schema.json` (type, enum cùng thứ tự, default).
  - Sinh bổ sung các danh sách giá trị enum (`METHOD_VALUES`, `OUTPUT_CHARSET_VALUES`,
    `DIACRITIC_STYLE_VALUES`, `ASSIST_MODE_VALUES`, `ASSIST_AUTO_FIX_VALUES`), `ASSIST_IDLE_MS_DEFAULT`,
    và bảng `SETTINGS_MODEL` / `UiSettingsModel.settingsModel` cho Rust/C/Swift.
- **Generator & gate (`xtask/src/gen_ui.rs`)**:
  - `cargo xtask gen-ui` sinh 3 tệp:
    - Windows: `tray/src/ui_strings_generated.rs` (`tray/src/lib.rs`, `settings_dialog.rs`, `menu.rs`)
    - Linux: `adapters/linux-settings/src/ui_strings_generated.h` (`settings_model.c`)
    - macOS: `adapters/macos-app/Sources/TextVNAppLib/UiStringsGenerated.swift` (`AppInfo.swift`, `SettingsView.swift`, `AppDelegate.swift`)
  - `cargo xtask check-ui` (bước mới trong `xtask/src/preflight.rs` và job `gen-tables` của
    `.github/workflows/ci-shared.yml`): fail nếu bất kỳ bản sinh nào lệch nguồn, lệch
    `schemas/config.v1.schema.json`, hoặc mã UI viết tay chứa literal trùng nguồn (0 bản chép tay).
- **Test parity (`tray/src/settings_dialog.rs`)**: giữ nguyên tên 4 test trong `regression-index.md`
  (`labels_match_linux_settings_panel`, `assist_mode_hints_match_every_platform`,
  `about_text_matches_every_platform`, `help_window_links_match_every_platform`), xác minh cả 3 OS
  tham chiếu đúng hằng/macro sinh mã (`CHARSET_LABELS`, `METHOD_LABELS`, `ASSIST_MODE_LABELS`,
  `TV_UI_*`, `UiStrings.*`).
- **Kiểm chứng & đột biến (Phần 2)**:
  - `cargo xtask check-ui`: `9 mục / 21 chuỗi + 5 nhóm / 18 cài đặt khớp schema, 3 bản sinh khớp nguồn, 0 bản chép tay` (exit 0).
  - Đột biến D3 Phần 1: sửa `Telex` → `Telex_MUTATED` trong `tray/src/ui_strings_generated.rs` ⇒ `check-ui` đỏ (`exit 1`).
  - Đột biến D3 Phần 2: đổi `free_marking` `default = false` trong `data/ui/settings_model.toml` ⇒ `check-ui` đỏ (`exit 1`: `default model 'false' != schema 'true'`); hoàn nguyên ⇒ xanh.
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (Windows) và
    `--target x86_64-unknown-linux-gnu` (Linux): xanh.
  - `cargo test -p xtask`: **130 passed** (gồm 8 test `sf9_*` và `sf9_part2_*`); `cargo test -p textvn-tray` (với
    `TEXTVN_SKIP_APPCONTAINER_E2E=1` do ACL ổ `D:` local; CI chạy AppContainer thật): **99 + 3 passed**
    (gồm 4 test parity).
  - `regression-index --todo-max 0`, `hygiene-docs`, `hygiene-secrets`, `hygiene-apis`: đều exit 0.

## EN

- **Coordinated completion per repo owner directive**: `claude-opus-7e41b2d9` authored the initial
  WIP (`data/ui/strings.toml`, `xtask/src/gen_ui.rs`) before hitting a quota limit;
  `antigravity-gemini-37c7c6bc` fixed a multi-item list placeholder bug in `gen_ui::parse`,
  normalized `\r\n`→`\n` before FNV-1a 64 hashing, aligned `render_rust` formatting with `rustfmt`,
  added comment-aware hand-copy scanning (`strip_comments` + skipping `< 4`-char single-word labels),
  wired `gen-ui`/`check-ui` into `xtask`/`preflight`/`ci-shared`, replaced all 43 hand-copied UI
  strings across all 3 platforms in Part 1, and completed **Part 2** (`data/ui/settings_model.toml`
  + `charset_labels` + 1:1 schema cross-check).
- **Single UI-string source (`data/ui/strings.toml`)**: 9 entries / 21 strings shared across Windows,
  Linux, and macOS: `about_tagline`, `about_details` (0..2), `url_discussions`, `url_issues`,
  `url_math_guide`, `method_labels` (0..3), **`charset_labels` (0..3)**, `assist_mode_labels` (0..2),
  `assist_mode_hints` (0..2, enforced `<= 55` chars single line).
- **Single settings-model source (`data/ui/settings_model.toml`)**: 5 groups (`typing`, `options`,
  `assist`, `math`, `system`) + 18 `[[setting]]` entries with `group`, `config_key`, `kind`
  (`bool`/`enum`/`u32`), `values` (for enums), `default`, and `depends_on`:
  - `check-ui` reuses the repo JSON parser (`check_mac_targets::parse_json`, G22) to cross-check
    every `[[setting]]` 1:1 against `schemas/config.v1.schema.json` (type, exact enum order, default).
  - Emits generated enum value lists (`METHOD_VALUES`, `OUTPUT_CHARSET_VALUES`,
    `DIACRITIC_STYLE_VALUES`, `ASSIST_MODE_VALUES`, `ASSIST_AUTO_FIX_VALUES`), `ASSIST_IDLE_MS_DEFAULT`,
    and the `SETTINGS_MODEL` / `UiSettingsModel.settingsModel` metadata table for Rust/C/Swift.
- **Generator & gate (`xtask/src/gen_ui.rs`)**:
  - `cargo xtask gen-ui` emits 3 generated files:
    - Windows: `tray/src/ui_strings_generated.rs` (`tray/src/lib.rs`, `settings_dialog.rs`, `menu.rs`)
    - Linux: `adapters/linux-settings/src/ui_strings_generated.h` (`settings_model.c`)
    - macOS: `adapters/macos-app/Sources/TextVNAppLib/UiStringsGenerated.swift` (`AppInfo.swift`, `SettingsView.swift`, `AppDelegate.swift`)
  - `cargo xtask check-ui` (wired into `xtask/src/preflight.rs` and the `gen-tables` job in
    `.github/workflows/ci-shared.yml`): fails on any generated-file drift, any mismatch against
    `schemas/config.v1.schema.json`, or any hand-written UI literal duplicating the source (0 hand copies).
- **Parity tests (`tray/src/settings_dialog.rs`)**: preserved all 4 `regression-index.md` test names
  (`labels_match_linux_settings_panel`, `assist_mode_hints_match_every_platform`,
  `about_text_matches_every_platform`, `help_window_links_match_every_platform`), verifying all 3 OSes
  reference the generated symbols (`CHARSET_LABELS`, `METHOD_LABELS`, `ASSIST_MODE_LABELS`,
  `TV_UI_*`, `UiStrings.*`) rather than repeating literals.
- **Verification & mutation checks (Part 2)**:
  - `cargo xtask check-ui`: `9 entries / 21 strings + 5 groups / 18 settings match schema, 3 generated outputs match source, 0 hand copies` (exit 0).
  - D3 mutation (Part 1): changing `Telex` → `Telex_MUTATED` in `tray/src/ui_strings_generated.rs` made `check-ui` fail (`exit 1`).
  - D3 mutation (Part 2): flipping `free_marking` `default = false` in `data/ui/settings_model.toml` made `check-ui` fail (`exit 1`: `default model 'false' != schema 'true'`); reverted → exit 0.
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (Windows), and
    `--target x86_64-unknown-linux-gnu` (Linux): green.
  - `cargo test -p xtask`: **130 passed** (including 8 `sf9_*` / `sf9_part2_*` tests); `cargo test -p textvn-tray`
    (with `TEXTVN_SKIP_APPCONTAINER_E2E=1` for local `D:` drive AppContainer ACL; CI runs full
    AppContainer): **99 + 3 passed** (including all 4 parity tests).
  - `regression-index --todo-max 0`, `hygiene-docs`, `hygiene-secrets`, `hygiene-apis`: all exit 0.
