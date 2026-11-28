# SPDX-License-Identifier: GPL-3.0-or-later
# scripts/lib/fcitx5-test-env.sh — MỘT nguồn (G22) cấu hình phím tắt của Fcitx5 cho phiên TEST.
# `source` từ scripts/e2e-linux.sh, scripts/test-linux-distro-matrix.sh, scripts/test-linux-package.sh.
#
# E68 / R2-175: Fcitx5 5.0.x (Ubuntu 22.04 / Mint 21 / Debian 12) mặc định gắn Ctrl+Shift vào
# `EnumerateForwardKeys` — Fcitx5 đổi input method (textvn ⇄ keyboard-us) TRƯỚC khi phím tới TextVN,
# nên công tắc V/E của TextVN không chạy và state.json không đổi (bằng chứng ARM-L1: ci-linux
# 46097314825, ubuntu-22.04-arm, `[diag] fcitx CurrentInputMethod=keyboard-us`). Phiên test tắt các
# phím đổi IM mặc định để e2e kiểm đúng Ctrl+Shift của TextVN; máy người dùng được `textvn doctor`
# cảnh báo (W_FCITX5_CTRL_SHIFT) — không ghi đè cấu hình Fcitx5 của người dùng.

# $1 = thư mục cấu hình Fcitx5 của phiên test ($XDG_CONFIG_HOME/fcitx5).
textvn_fcitx5_test_hotkeys() {
    local dir="$1"
    mkdir -p "$dir"
    cat >"$dir/config" <<'C'
[Hotkey/EnumerateForwardKeys]

[Hotkey/EnumerateBackwardKeys]

[Hotkey/EnumerateGroupForwardKeys]

[Hotkey/EnumerateGroupBackwardKeys]
C
}
