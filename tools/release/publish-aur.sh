#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# publish-aur.sh — đẩy PKGBUILD + .SRCINFO lên AUR (aur.archlinux.org/textvn).
#
# Dùng sau khi release được publish (checklist B7e):
#   ./tools/release/publish-aur.sh                 # lấy version từ Cargo.toml
#   ./tools/release/publish-aur.sh 0.2.33          # hoặc chỉ định version
#
# Điều kiện:
#   1. CI `ci-aur` đã XANH trên commit của PKGBUILD này (makepkg thật đã pass).
#   2. SSH key có quyền write trên AUR:
#      - Cục bộ: ssh-agent sẵn (ssh-add ~/.ssh/id_ed25519).
#      - CI: secret `AUR_SSH_KEY` (private key OpenSSH) — script tự nạp vào agent.
#   3. Account AUR của owner (hunglinhpt) đã add key trong Profile → SSH Keys.
#
# Lần đầu tiên: `git clone ssh://aur@aur.archlinux.org/textvn.git` tự tạo repo
# gói (AUR tạo khi push đầu tiên; account phải có ít nhất 1 gói? — AUR chấp
# nhận tạo gói qua push đầu tiên từ account đã đăng ký ≥30 ngày? Nếu bị từ
# chối, dùng giao diện web: Submit → dán .SRCINFO).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ARCH_DIR="$ROOT/packaging/arch"

# --- Version nguồn sự thật = Cargo.toml -------------------------------------
VERSION="$(sed -n 's/^version = "\([^"]*\)".*/\1/p' "$ROOT/Cargo.toml" | head -n1)"
if [[ $# -ge 1 ]]; then
    VERSION="$1"
fi
if [[ ! -f "$ARCH_DIR/PKGBUILD" || ! -f "$ARCH_DIR/.SRCINFO" ]]; then
    echo "✗ thiếu packaging/arch/PKGBUILD hoặc .SRCINFO" >&2
    exit 1
fi
if ! grep -q "^	pkgver = $VERSION\$" "$ARCH_DIR/.SRCINFO"; then
    echo "✗ .SRCINFO pkgver ≠ $VERSION — bump .SRCINFO trước khi publish" >&2
    exit 1
fi
if ! grep -q "^pkgver=$VERSION$" "$ARCH_DIR/PKGBUILD"; then
    echo "✗ PKGBUILD pkgver ≠ $VERSION" >&2
    exit 1
fi

# --- SSH key cho CI (cục bộ: dùng agent sẵn có) ------------------------------
if [[ -n "${AUR_SSH_KEY:-}" ]]; then
    eval "$(ssh-agent -s)" >/dev/null
    KEYFILE="$(mktemp)"
    trap 'rm -f "$KEYFILE"; ssh-agent -k >/dev/null 2>&1 || true' EXIT
    printf '%s\n' "$AUR_SSH_KEY" > "$KEYFILE"
    chmod 600 "$KEYFILE"
    ssh-add "$KEYFILE"
fi

# --- Clone AUR, chép, commit, push -------------------------------------------
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
git clone ssh://aur@aur.archlinux.org/textvn.git "$TMP/textvn"
cp -- "$ARCH_DIR/PKGBUILD" "$TMP/textvn/PKGBUILD"
cp -- "$ARCH_DIR/.SRCINFO" "$TMP/textvn/.SRCINFO"
cd "$TMP/textvn"
if git diff --quiet && ! git ls-files --error-unmatch PKGBUILD >/dev/null 2>&1; then
    git add PKGBUILD .SRCINFO
fi
if git diff --cached --quiet && git diff --quiet; then
    echo "✓ AUR đã đúng version $VERSION — không cần push"
    exit 0
fi
git add PKGBUILD .SRCINFO
git commit -m "Update to $VERSION

Xem: https://github.com/hunglinhpt/TextVN/releases/tag/v$VERSION
Build được kiểm bởi ci-aur (makepkg thật trên archlinux:base-devel)."
git push origin master
echo "✓ Đã đẩy textvn $VERSION lên AUR (hiển thị sau ~1 phút trên aur.archlinux.org)."
