# SPDX-License-Identifier: GPL-3.0-or-later
# run-distro-container.sh — MỘT nguồn cho khối docker-run E2E distro (G22, review ARM-L4:
# bản arm64 y nguyên bản x86_64 chỉ khác 1 dòng assert arch ⇒ sửa phụ thuộc distro ở một
# bên sẽ trôi bên kia). Dùng chung cho job linux-distro-matrix (x86_64) và phần arm64
# trong release.yml; luật deps dnf/zypper/apt + machine-id + user tester chỉ nằm ở đây.
#
# Cú pháp:
#   run-distro-container.sh <image> <tag> <mode> <tarball> <helpers-dir> <expect-arch|->
#     image       ảnh container (almalinux:8, debian:12, …)
#     tag         DISTRO_TAG (almalinux-8, ubuntu-22-04, …)
#     mode        el8 | matrix
#     tarball     đường dẫn gói TextVN-*-linux-*.tar.gz cần kiểm
#     helpers-dir thư mục chứa e2e_ibus / e2e_fcitx5 (đã tải từ artifact)
#     expect-arch "-" (không kiểm) hoặc aarch64 (container + runner phải đúng arch)

set -euo pipefail

IMAGE="$1"; TAG="$2"; MODE="$3"; TARBALL="$4"; HELPERS="$5"; EXPECT_ARCH="$6"

if [[ ! -s "$TARBALL" ]]; then
  echo "run-distro-container: không thấy gói $TARBALL" >&2
  exit 1
fi

chmod +x "$HELPERS/e2e_ibus" "$HELPERS/e2e_fcitx5" 2>/dev/null || true
TARBALL_NAME="$(basename "$TARBALL")"
HOST_UID="$(id -u)"
HOST_GID="$(id -g)"

docker run --rm \
  -v "$PWD:/src:ro" \
  -e DISTRO_TAG="$TAG" \
  -e TEST_MODE="$MODE" \
  -e TARBALL_NAME="$TARBALL_NAME" \
  -e HOST_UID="$HOST_UID" \
  -e HOST_GID="$HOST_GID" \
  -e EXPECT_ARCH="$EXPECT_ARCH" \
  "$IMAGE" bash -euo pipefail -c '
    if [[ "$EXPECT_ARCH" != "-" ]]; then
      test "$(uname -m)" = "$EXPECT_ARCH"
    fi
    case "$DISTRO_TAG" in
      almalinux-8|almalinux-9)
        dnf install -y --setopt=install_weak_deps=False \
          ibus gtk3 dbus-daemon dbus-tools procps-ng gawk dconf xkeyboard-config diffutils findutils tar gzip shadow-utils util-linux >/dev/null
        ;;
      almalinux-10)
        # EL10: không có fcitx5 ở kho gốc (EPEL) — chỉ IBus như EL9; GTK4 có sẵn.
        dnf install -y --setopt=install_weak_deps=False \
          ibus gtk4 dbus-daemon dbus-tools procps-ng gawk dconf xkeyboard-config diffutils findutils tar gzip shadow-utils util-linux >/dev/null
        ;;
      fedora-41|fedora-42|fedora-43)
        dnf install -y --setopt=install_weak_deps=False \
          ibus fcitx5 gtk4 dbus-daemon dbus-tools procps-ng gawk dconf xkeyboard-config diffutils findutils tar gzip shadow-utils util-linux >/dev/null
        if [[ ! -s /etc/machine-id ]]; then
          dbus-uuidgen > /etc/machine-id
          chmod 0444 /etc/machine-id
        fi
        ;;
      opensuse-leap-15-6)
        zypper --non-interactive install --no-recommends \
          ibus libgtk-3-0 dbus-1 procps gawk dconf xkeyboard-config diffutils findutils tar gzip glib2-tools shadow util-linux >/dev/null
        ;;
      opensuse-tumbleweed)
        zypper --non-interactive install --no-recommends \
          ibus fcitx5 libgtk-4-1 dbus-1 dbus-1-daemon procps gawk dconf xkeyboard-config diffutils findutils tar gzip glib2-tools shadow util-linux >/dev/null
        if [[ ! -s /etc/machine-id ]]; then
          dbus-uuidgen > /etc/machine-id
          chmod 0444 /etc/machine-id
        fi
        ;;
      ubuntu-22-04|ubuntu-24-04|ubuntu-26-04|mint-22|debian-12|debian-13)
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -qq
        apt-get install -y -qq --no-install-recommends \
          ibus fcitx5 fcitx5-modules libgtk-4-1 dbus dbus-x11 procps gawk dconf-cli xkb-data diffutils findutils tar gzip libdbus-1-3 >/dev/null
        ;;
      *)
        echo "run-distro-container: DISTRO_TAG lạ: $DISTRO_TAG" >&2
        exit 1
        ;;
    esac
    groupadd -f -g "$HOST_GID" tester || true
    useradd -m -u "$HOST_UID" -g "$HOST_GID" -o tester
    if [[ "$TEST_MODE" == el8 ]]; then
      su -s /bin/bash tester -c "LC_ALL=C.UTF-8 LANG=C.UTF-8 bash /src/scripts/test-linux-el8-package.sh /src/dist/$TARBALL_NAME /src/e2e-helpers/e2e_ibus"
      LC_ALL=C.UTF-8 LANG=C.UTF-8 bash /src/scripts/test-linux-el8-package.sh --system-only /src/dist/$TARBALL_NAME /src/e2e-helpers/e2e_ibus
    else
      su -s /bin/bash tester -c "LC_ALL=C.UTF-8 LANG=C.UTF-8 bash /src/scripts/test-linux-distro-matrix.sh $DISTRO_TAG /src/dist/$TARBALL_NAME /src/e2e-helpers/e2e_ibus /src/e2e-helpers/e2e_fcitx5"
      LC_ALL=C.UTF-8 LANG=C.UTF-8 bash /src/scripts/test-linux-distro-matrix.sh --system-only "$DISTRO_TAG" "/src/dist/$TARBALL_NAME" /src/e2e-helpers/e2e_ibus /src/e2e-helpers/e2e_fcitx5
    fi
  '
