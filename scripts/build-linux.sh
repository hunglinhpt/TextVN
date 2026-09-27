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

echo "--> 2. Chạy test workspace..."
cargo test --workspace

echo "--> 3. Biên dịch release..."
cargo build --release --workspace

echo "--> 4. Đóng gói Tarball Portable cho Linux..."
PKG_NAME="TextVN-linux-x86_64-v${VERSION}"
STAGE_DIR="${DIST_DIR}/${PKG_NAME}"
rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}/bin" "${STAGE_DIR}/share/icons" "${STAGE_DIR}/share/applications" "${STAGE_DIR}/scripts"

# Copy binaries
if [[ -f "${ROOT_DIR}/target/release/textvn-cli" ]]; then
    cp "${ROOT_DIR}/target/release/textvn-cli" "${STAGE_DIR}/bin/textvn"
fi
if [[ -f "${ROOT_DIR}/target/release/TextVN" ]]; then
    cp "${ROOT_DIR}/target/release/TextVN" "${STAGE_DIR}/bin/textvn-tray"
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
