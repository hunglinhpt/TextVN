#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# TextVN - Master Linux Build & Packaging Script

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
VERSION="0.1.0"

echo "=== TextVN Linux Build v${VERSION} ==="

mkdir -p "${DIST_DIR}"

echo "--> 1. Kiểm tra môi trường..."
rustc --version
cargo --version

echo "--> 2. Test engine + adapter (unit + e2e với ibus-daemon/fcitx5 thật)..."
cargo test --workspace
"${SCRIPT_DIR}/e2e-linux.sh" "${ROOT_DIR}/target/linux-adapters"

echo "--> 3. Biên dịch release CLI..."
cargo build --release -p textvn-cli

echo "--> 4. Đóng gói Tarball Portable cho Linux..."
PKG_NAME="TextVN-linux-x86_64-v${VERSION}"
STAGE_DIR="${DIST_DIR}/${PKG_NAME}"
ADAPTERS="${ROOT_DIR}/target/linux-adapters"
rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}/bin" "${STAGE_DIR}/lib/textvn" "${STAGE_DIR}/lib/fcitx5" \
         "${STAGE_DIR}/share/icons" "${STAGE_DIR}/share/applications"

# Binaries: CLI + IBus engine + Fcitx5 addon (tray là thành phần Windows — không đóng gói).
cp "${ROOT_DIR}/target/release/textvn-cli" "${STAGE_DIR}/bin/textvn"
cp "${ADAPTERS}/ibus/textvn-ibus-engine" "${STAGE_DIR}/lib/textvn/"
cp "${ADAPTERS}/fcitx5/libtextvn-fcitx5.so" "${STAGE_DIR}/lib/fcitx5/"
if [[ -f "${ADAPTERS}/settings/textvn-settings" ]]; then
    cp "${ADAPTERS}/settings/textvn-settings" "${STAGE_DIR}/bin/"
fi

# Copy resources & icons
if [[ -d "${ROOT_DIR}/resources/icons" ]]; then
    cp -r "${ROOT_DIR}/resources/icons/"* "${STAGE_DIR}/share/icons/"
fi

# Copy packaging metadata
if [[ -d "${ROOT_DIR}/packaging/linux" ]]; then
    mkdir -p "${STAGE_DIR}/packaging"
    cp -r "${ROOT_DIR}/packaging/linux" "${STAGE_DIR}/packaging/"
fi

# Copy scripts & documentation
cp "${ROOT_DIR}/scripts/install_linux.sh" "${STAGE_DIR}/"
cp "${ROOT_DIR}/scripts/uninstall_linux.sh" "${STAGE_DIR}/"
chmod +x "${STAGE_DIR}/install_linux.sh" "${STAGE_DIR}/uninstall_linux.sh"
cp "${ROOT_DIR}/README.md" "${STAGE_DIR}/"
cp "${ROOT_DIR}/LICENSE" "${STAGE_DIR}/"

cd "${DIST_DIR}"
tar -czvf "${PKG_NAME}.tar.gz" "${PKG_NAME}"
rm -rf "${PKG_NAME}"

echo "--> 5. Sinh mã băm SHA256..."
sha256sum "${PKG_NAME}.tar.gz" > "${DIST_DIR}/SHA256SUMS-linux.txt"
cat "${DIST_DIR}/SHA256SUMS-linux.txt"

echo "=== Build Linux Thành Công! ==="
echo "Artifact: ${DIST_DIR}/${PKG_NAME}.tar.gz"
