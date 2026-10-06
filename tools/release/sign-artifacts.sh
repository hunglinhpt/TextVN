#!/usr/bin/env bash
# sign-artifacts.sh — Ký số tự động cho release assets (3 platform).
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Hai lớp chữ ký cho MỌI artifact của một release (exe/zip/msix/tar.gz/pkg):
#   1. GPG detached (ASCII .sig)  — key từ secret RELEASE_GPG_PRIVATE_KEY
#      (TextVN Release Signing, FPR 3921595ABC961199F15303B6C45B84D0C7F4A822,
#      public key: docs/release/signing/gpg-release-key.asc).
#   2. Sigstore keyless (cosign sign-blob, OIDC GitHub Actions) — chữ ký +
#      chứng thư chứng minh artifact được build BỞI workflow này (provenance).
#
# LƯU Ý: hai lớp này là integrity/provenance — KHÔNG thay thế Authenticode
# (policy 10.2.9 của Microsoft Store đòi hỏi Authenticode/Trusted Signing hoặc
# MSIX mà Store tự ký — xem docs/release/store-policy-10-2-9.md).
#
# Cách dùng: sign-artifacts.sh <thư-mục-chứa-assets>
# Env: RELEASE_GPG_PRIVATE_KEY (armor, tuỳ chọn — thiếu thì bỏ qua GPG),
#      COSIGN_YES=1 (cosign keyless, tuỳ chọn — thiếu cosign binary thì bỏ qua).
set -euo pipefail

DIR="${1:?dung: sign-artifacts.sh <assets-dir>}"
cd "$DIR"

# 1. GPG: import key từ secret (nếu có) + ký từng file.
if [ -n "${RELEASE_GPG_PRIVATE_KEY:-}" ]; then
    echo "$RELEASE_GPG_PRIVATE_KEY" | gpg --batch --import 2>/dev/null || true
    for f in *; do
        case "$f" in
            *.sig|*.asc|*.cert|SHA256SUMS.txt) continue ;;
        esac
        if [ -f "$f" ]; then
            gpg --batch --yes --armor --detach-sign "$f"
            echo "GPG signed: $f.sig"
        fi
    done
    if [ -f SHA256SUMS.txt ]; then
        gpg --batch --yes --clearsign SHA256SUMS.txt
        mv SHA256SUMS.txt.asc SHA256SUMS.txt
        echo "GPG clearsigned: SHA256SUMS.txt"
    fi
else
    echo "SKIP GPG: RELEASE_GPG_PRIVATE_KEY not set"
fi

# 2. Sigstore keyless (cosign): chữ ký + chứng thư OIDC của workflow.
if command -v cosign >/dev/null 2>&1; then
    export COSIGN_YES="${COSIGN_YES:-true}"
    for f in *; do
        case "$f" in
            *.sig|*.asc|*.cert|SHA256SUMS.txt) continue ;;
        esac
        if [ -f "$f" ]; then
            cosign sign-blob --yes \
                --output-signature "$f.cosign.sig" \
                --output-certificate "$f.cosign.cert" \
                "$f" >/dev/null
            echo "COSIGN signed: $f.cosign.sig"
        fi
    done
else
    echo "SKIP cosign: binary not found"
fi

echo "Signing done: $(ls | wc -l) files trong $DIR"
