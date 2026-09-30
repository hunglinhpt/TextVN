#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# stage-linux.sh <stage_dir> [adapters_build_dir]
#
# Gom binary đã build (target/release + CMake build của adapter) thành bố cục gói Linux —
# cùng một bố cục cho tarball (build-linux.sh) và cài từ mã nguồn (install_linux.sh):
#
#   bin/textvn  bin/textvn-settings
#   lib/textvn/textvn-ibus-engine  lib/textvn/fcitx5/libtextvn-fcitx5.so
#   share/ibus/component/textvn.xml  share/fcitx5/{addon,inputmethod}/textvn.conf
#   share/applications  share/icons  share/metainfo  share/doc/textvn
#   install.sh  uninstall.sh  textvn-portable.sh  textvn-common.sh  VERSION

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAGE="${1:?cần thư mục stage}"
ADAPTERS="${2:-$ROOT/target/linux-adapters}"
VERSION="$(sed -n 's/^version = "\([^"]*\)".*/\1/p' "$ROOT/Cargo.toml" | head -n1)"

[[ -x "$ROOT/target/release/textvn-cli" ]] || { echo "thiếu target/release/textvn-cli" >&2; exit 1; }
case "$(realpath -m "$STAGE")" in
    /|"$HOME"|"$ROOT") echo "thư mục stage không hợp lệ: $STAGE" >&2; exit 1 ;;
esac
rm -rf "$STAGE"
mkdir -p "$STAGE"
inst() { install -D -m "$1" "$2" "$STAGE/$3"; }

inst 0755 "$ROOT/target/release/textvn-cli" bin/textvn
[[ -x "$ADAPTERS/settings/textvn-settings" ]] &&
    inst 0755 "$ADAPTERS/settings/textvn-settings" bin/textvn-settings
if [[ -x "$ADAPTERS/ibus/textvn-ibus-engine" ]]; then
    inst 0755 "$ADAPTERS/ibus/textvn-ibus-engine" lib/textvn/textvn-ibus-engine
    inst 0644 "$ROOT/packaging/linux/ibus/textvn.xml" share/ibus/component/textvn.xml
fi
if [[ -f "$ADAPTERS/fcitx5/libtextvn-fcitx5.so" ]]; then
    inst 0755 "$ADAPTERS/fcitx5/libtextvn-fcitx5.so" lib/textvn/fcitx5/libtextvn-fcitx5.so
    inst 0644 "$ROOT/packaging/linux/fcitx5/addon/textvn.conf" share/fcitx5/addon/textvn.conf
    inst 0644 "$ROOT/packaging/linux/fcitx5/inputmethod/textvn.conf" share/fcitx5/inputmethod/textvn.conf
fi
for icon in textvn_v textvn_e; do
    inst 0644 "$ROOT/resources/icons/$icon.svg" "share/icons/hicolor/scalable/apps/$icon.svg"
    # PNG 128px cho panel/tray cũ không render SVG (sinh bằng generate_app_icons.py)
    inst 0644 "$ROOT/resources/icons/$icon.png" "share/icons/hicolor/128x128/apps/$icon.png"
done
inst 0644 "$ROOT/packaging/linux/desktop/textvn-settings.desktop" share/applications/textvn-settings.desktop
inst 0644 "$ROOT/packaging/linux/appstream/io.github.hunglinhpt.textvn.metainfo.xml" \
    share/metainfo/io.github.hunglinhpt.textvn.metainfo.xml
for doc in README.md LICENSE CHANGELOG.md docs/user-guide.md; do
    [[ -f "$ROOT/$doc" ]] && inst 0644 "$ROOT/$doc" "share/doc/textvn/$(basename "$doc")"
done
for s in install.sh uninstall.sh textvn-portable.sh; do
    inst 0755 "$ROOT/packaging/linux/$s" "$s"
done
inst 0644 "$ROOT/packaging/linux/textvn-common.sh" textvn-common.sh
echo "$VERSION" > "$STAGE/VERSION"
echo "staged TextVN $VERSION → $STAGE"
