#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# prepare-ppa.sh — chuẩn bị + upload source package lên PPA Launchpad
# (ppa:hunglinhpt/textvn). Chạy trên Ubuntu/WSL (cần devscripts + GPG key
# đã đăng ký Launchpad — các bước tài khoản là của CHỦ REPO, xem
# docs/release/linux-distribution.md §4).
#
#   ./tools/release/prepare-ppa.sh              # version từ Cargo.toml
#   ./tools/release/prepare-ppa.sh --upload     # sau khi build xong, dput luôn
#
# Nguyên tắc Launchpad: CHỈ nhận SOURCE package (mã nguồn + debian/) — Canonical
# build hộ trên máy không có Internet ⇒ crate phải VENDORED trong source
# (`cargo vendor`, .cargo/config.toml ép offline). Script làm trong bản SAO để
# không đụng worktree của repo.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
UPLOAD=0
[[ "${1:-}" == "--upload" ]] && UPLOAD=1

VERSION="$(sed -n 's/^version = "\([^"]*\)".*/\1/p' "$ROOT/Cargo.toml" | head -n1)"
DEB_VER="$(head -n1 "$ROOT/debian/changelog" | sed -n 's/^textvn (\([^-]*\)-.*$/\1/p')"
if [[ "$VERSION" != "$DEB_VER" ]]; then
    echo "✗ debian/changelog ($DEB_VER) ≠ Cargo.toml ($VERSION) — bump cả hai" >&2
    exit 1
fi
# CI ci-deb phải xanh trước khi upload (QA gate).
echo "==> Version $VERSION (khớp changelog)."

WORK="$(mktemp -d "/tmp/textvn-ppa-$VERSION-XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

# 1. Bản sao nguồn sạch (git archive — loại .git, dist/, target/)
git -C "$ROOT" archive --format=tar.gz --prefix="textvn-$VERSION/" HEAD \
    -o "$WORK/textvn-$VERSION.tar.gz"
mkdir -p "$WORK/src"
tar -xzf "$WORK/textvn-$VERSION.tar.gz" -C "$WORK/src"
SRC="$WORK/src/textvn-$VERSION"

# 2. Vendor crates (offline build cho Launchpad) — R2-119
echo "==> cargo vendor (chỉ lần đầu mỗi bản, có thể vài phút)…"
(cd "$SRC" && cargo vendor vendor > /tmp/vendor-config.toml)
mkdir -p "$SRC/.cargo"
cat /tmp/vendor-config.toml > "$SRC/.cargo/config.toml"
echo "==> vendor/ kích thước: $(du -sh "$SRC/vendor" | cut -f1)"

# 3. orig.tar.gz chuẩn (bản sao đã vendor)
(cd "$WORK/src" && tar -czf "$WORK/textvn_${VERSION}.orig.tar.gz" "textvn-$VERSION")

# 4. dpkg-buildpackage -S (source package, ký bằng GPG của owner)
cp -r "$ROOT/debian" "$SRC/debian"
cd "$SRC"
KEY_ARG=""
[[ -n "${DEBSIGN_KEYID:-}" ]] && KEY_ARG="-k$DEBSIGN_KEYID"
debuild -S -sa $KEY_ARG

# 5. dput lên PPA
CHANGES="$(ls "$SRC"/../textvn_"$VERSION"-*_source.changes | head -n1)"
echo "==> Source package: $CHANGES"
if [[ "$UPLOAD" == 1 ]]; then
    dput ppa:hunglinhpt/textvn "$CHANGES"
    echo "✓ Đã upload. Launchpad build amd64 + arm64 (~10-30 phút)."
    echo "  Sau khi build xong, người dùng cài:"
    echo "    sudo add-apt-repository ppa:hunglinhpt/textvn && sudo apt install textvn"
else
    echo "✓ Đã tạo source package — kiểm tra rồi chạy lại với --upload để dput."
    echo "  $CHANGES"
fi
