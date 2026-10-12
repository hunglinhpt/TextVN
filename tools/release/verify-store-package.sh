#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# verify-store-package.sh — kiểm gói .msix của MỘT GitHub Release trước khi nộp Microsoft
# Store tự động (.github/workflows/store-publish.yml, job verify). Chạy tay cũng được:
#
#   tools/release/verify-store-package.sh <thư-mục-asset> <version A.B.C> [<identity-name>]
#
# Thư mục phải có đúng các asset tải từ release: TextVN-<ver>-windows-x64.msix kèm .asc,
# .cosign.sig, .cosign.cert và SHA256SUMS.txt (clearsign). Kiểm:
#   1. chữ ký GPG tách rời của .msix VÀ chữ ký clearsign của SHA256SUMS.txt đều do khoá phát
#      hành (FPR cố định bên dưới). Public key lấy từ checkout của repo
#      (docs/release/signing/gpg-release-key.asc) — KHÔNG dùng gpg-release-key.asc đính kèm
#      release, vì đó cũng chỉ là một file tải về cùng gói;
#   2. SHA-256 của .msix khớp đúng một dòng của nó trong SHA256SUMS.txt;
#   3. Sigstore: chứng thư phải là của ĐÚNG workflow release.yml chạy trên ĐÚNG tag v<version>
#      của repo này (gói của tag khác, nhánh khác hay repo khác đều bị từ chối). Bắt buộc
#      trong CI; chạy tay mà không có cosign thì bỏ qua kèm cảnh báo;
#   4. tools/win/verify-msix.py --require-store-identity: luật Partner Center, Version
#      (A+1).B.C.0 map từ <version>, và Identity/Name = <identity-name> nếu có truyền.
# Dòng cuối stdout: "sha256=<hex>" (store-publish.yml chuyển sang job nộp để kiểm lại).
set -euo pipefail

DIR="${1:?dung: verify-store-package.sh <assets-dir> <version A.B.C> [<identity-name>]}"
VERSION="${2:?thieu version A.B.C}"
IDENTITY="${3:-}"
RELEASE_FPR="3921595ABC961199F15303B6C45B84D0C7F4A822"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
KEY="${RELEASE_PUBKEY:-$ROOT/docs/release/signing/gpg-release-key.asc}"
PY="${PYTHON:-python3}"
command -v "$PY" >/dev/null 2>&1 || PY=python
COSIGN="${COSIGN:-cosign}"
REPO="${GITHUB_REPOSITORY:-hunglinhpt/TextVN}"

die() { echo "FATAL: $*" >&2; exit 1; }

[[ "$VERSION" =~ ^[0-9]{1,5}\.[0-9]{1,5}\.[0-9]{1,5}$ ]] || die "version '$VERSION' không phải A.B.C"
NAME="TextVN-${VERSION}-windows-x64.msix"
MSIX="$DIR/$NAME"
for f in "$MSIX" "$MSIX.asc" "$DIR/SHA256SUMS.txt" "$KEY"; do
    [ -f "$f" ] || die "thiếu $f"
done

WORK="$(mktemp -d)"
export GNUPGHOME="$WORK/gnupg"
mkdir "$GNUPGHOME"
chmod 700 "$GNUPGHOME" 2>/dev/null || true   # Git Bash (Windows) không đổi được mode
trap 'rm -rf "$WORK"' EXIT

gpg --batch --quiet --import "$KEY" 2>/dev/null || die "không import được public key $KEY"
gpg --batch --with-colons --fingerprint "$RELEASE_FPR" 2>/dev/null | grep -q "^fpr:*${RELEASE_FPR}:" \
    || die "$KEY không chứa khoá phát hành $RELEASE_FPR"

# VALIDSIG <fpr khoá ký> ... <fpr khoá chính> (trường cuối): nhận chữ ký của khoá chính hoặc
# subkey của nó. gpg chỉ in VALIDSIG cho chữ ký đúng.
valid_sig() {
    awk -v f="$RELEASE_FPR" '$1 == "[GNUPG:]" && $2 == "VALIDSIG" && ($3 == f || $NF == f) { ok = 1 }
        END { exit !ok }' "$1"
}

gpg --batch --status-file "$WORK/msix.status" --verify "$MSIX.asc" "$MSIX" 2>/dev/null \
    || die "chữ ký GPG của $NAME không hợp lệ"
valid_sig "$WORK/msix.status" || die "$NAME.asc không do khoá phát hành $RELEASE_FPR ký"
echo "OK   GPG $NAME.asc ($RELEASE_FPR)"

gpg --batch --status-file "$WORK/sums.status" --output "$WORK/sums.txt" \
    --decrypt "$DIR/SHA256SUMS.txt" 2>/dev/null || die "SHA256SUMS.txt không có chữ ký clearsign hợp lệ"
valid_sig "$WORK/sums.status" || die "SHA256SUMS.txt không do khoá phát hành $RELEASE_FPR ký"
echo "OK   GPG SHA256SUMS.txt (clearsign)"

want="$(awk -v n="$NAME" '$2 == n || $2 == "*" n { print $1 }' "$WORK/sums.txt")"
[ "$(printf '%s\n' "$want" | grep -c .)" -eq 1 ] || die "SHA256SUMS.txt cần đúng một dòng cho $NAME"
have="$(sha256sum "$MSIX" | cut -d' ' -f1)"
[ "$have" = "$want" ] || die "SHA-256 $NAME = $have ≠ SHA256SUMS.txt ($want)"
echo "OK   SHA-256 $NAME khớp SHA256SUMS.txt"

IDENT="https://github.com/${REPO}/.github/workflows/release.yml@refs/tags/v${VERSION}"
if command -v "$COSIGN" >/dev/null 2>&1; then
    for f in "$MSIX.cosign.sig" "$MSIX.cosign.cert"; do
        [ -f "$f" ] || die "thiếu $f"
    done
    "$COSIGN" verify-blob --signature "$MSIX.cosign.sig" --certificate "$MSIX.cosign.cert" \
        --certificate-identity "$IDENT" \
        --certificate-oidc-issuer https://token.actions.githubusercontent.com "$MSIX" >/dev/null 2>&1 \
        || die "Sigstore: $NAME không phải do $IDENT build"
    echo "OK   Sigstore $NAME ← $IDENT"
elif [ "${GITHUB_ACTIONS:-false}" = "true" ]; then
    die "không có cosign trong CI (sigstore/cosign-installer)"
else
    echo "WARN cosign không có trên máy — bỏ qua kiểm provenance Sigstore"
fi

args=("$MSIX" --expect-version "$VERSION" --require-store-identity)
[ -n "$IDENTITY" ] && args+=(--expect-name "$IDENTITY")
"$PY" "$ROOT/tools/win/verify-msix.py" "${args[@]}" || die "gói không đạt luật Store (verify-msix.py)"

echo "sha256=$have"
