# Nộp TextVN vào Debian / Ubuntu chính thức (RFP → ITP → Sponsor)

> Trạng thái 2026-10-18. Bổ sung cho lộ trình VỤ 2 — đưa TextVN vào kho
> chính thức. Kênh đã chuẩn bị: `debian/` (source package chạy được — QA gate
> `ci-deb.yml` build .deb thật trong container `debian:trixie`).

## 0. Nguyên tắc (khác gì PPA)

| | PPA (Launchpad) | Debian/Ubuntu official |
|---|---|---|
| Upload dạng | Source package, crate **vendored** | Source package, crate **không được vendor** — cấm theo Debian Policy; mỗi crate phải là gói `librust-*-dev` độc lập |
| Ai build | Launchpad build hộ | Debian buildd build hộ — nhưng người upload vào kho chính phải là **Debian Developer** (sponsor) |
| Bài toán Rust | `cargo vendor` (R2-119: `.cargo/config.toml` chỉ nằm trong source package, không commit vào repo) | **Không vendor được** — phụ thuộc ngoài phải có sẵn: serde, serde_json ✅ đã có `librust-serde-dev`/`librust-serde-json-dev` |

**Audit phụ thuộc (2026-10-18)**: cây Linux của TextVN (core, strategy,
config, appdb, ipc, ffi, cli + adapters C) chỉ phụ thuộc ngoài **serde +
serde_json** — cả hai đều có sẵn trong Debian Rust Team; `windows-*` chỉ là
dependency target `cfg(windows)` (không ảnh hưởng build Linux). Đây là tín
hiệu rất tốt cho maintainer: **không phải đóng gói thêm crate nào**.

## 1. Lộ trình ba bước

### Bước A — RFP (nhờ cộng đồng đóng gói) hoặc ITP (tự đóng gói rồi xin sponsor)

TextVN ĐÃ có sẵn `debian/` hoàn chỉnh (control/rules/changelog/copyright,
multiarch đúng chuẩn, QA bằng makepkg-style thật) ⇒ chọn **ITP** sẽ nhanh hơn
RFP: upload lên [mentors.debian.net](https://mentors.debian.net) rồi mở phiếu
RFS (Request For Sponsor).

Mẫu email ITP/RFP (gửi `submit@bugs.debian.org`, subject đúng mẫu):

```
To: submit@bugs.debian.org
Subject: ITP: textvn -- Vietnamese input method editor for IBus and Fcitx5

Package: wnpp
Severity: wishlist
Owner: LinhBH.CoM <hunglinhpt@gmail.com>
X-Debbugs-CC: debian-devel@lists.debian.org

* Package name    : textvn
  Version         : 0.2.34-1
  Upstream Author : LinhBH.CoM <hunglinhpt@gmail.com>
* URL             : https://github.com/hunglinhpt/TextVN
* License         : GPL-3.0-or-later
  Programming Lang: Rust, C, C++
  Description     : Vietnamese input method editor for IBus and Fcitx5

 TextVN converts keystrokes into Vietnamese with tone marks (Telex, VNI,
 VIQR, Simple Telex) using a shared Rust engine across Windows, macOS and
 Linux. The Linux package provides an IBus engine, an Fcitx5 addon
 (installed to the multiarch addon directory /usr/lib/<triplet>/fcitx5/)
 and a GTK4 settings application.

 Notable properties for packaging:
 - Linux runtime deps are minimal: serde and serde_json (both already in
   Debian as librust-*-dev); all other code is first-party or C/C++ against
   libibus/libfcitx5/gtk4.
 - Install layout already follows Debian multiarch conventions
   (/usr/share/ibus/component, /usr/lib/<triplet>/fcitx5) — the same layout
   used by the packaging files in the repository (debian/, QA'd by a CI job
   that runs dpkg-buildpackage inside a debian:trixie container).
 - Integration tests run against real ibus-daemon/fcitx5 in CI.

 Git: https://github.com/hunglinhpt/TextVN
```

(RFP = cùng nội dung, đổi Subject thành `RFP: textvn -- …` và bỏ Owner.)

### Bước B — mentors.debian.net upload (kèm ITP)

```bash
# Trên máy Debian/Ubuntu của owner (hoặc nhờ maintainer):
dpkg-buildpackage -S -us -uc          # source package từ debian/ hiện có
dput mentors ../textvn_0.2.34-1_source.changes
# Mở phiếu RFS (bug qua reportbug wnpp) kèm link mentors + lệnh sponsor-build
```

### Bước C — Liên hệ Debian Input Method Team (nhanh nhất)

Gửi email đến `debian-input-method@lists.debian.org`, CC
`debian-rust@lists.debian.org` (team phụ trách ibus-unikey, fcitx5-unikey,
ibus-bamboo…; các DD nhóm này quen với bộ gõ châu Á):

```
Subject: Upstream of TextVN (Vietnamese IME, Rust) — packaging help / ITP #NNNNNN

Hi Debian Input Method Team,

I'm the upstream author of TextVN, an open-source (GPL-3.0-or-later)
Vietnamese input method for IBus and Fcitx5 with a shared Rust engine
(also on Windows TSF and macOS IMK; published on the Microsoft Store and
Homebrew tap). Repository: https://github.com/hunglinhpt/TextVN

Key points for Debian packaging:
- deps are minimal: serde/serde_json only (already in Debian);
- the install layout follows multiarch exactly (/usr/share/ibus/component,
  /usr/lib/<triplet>/fcitx5) and is shared with the install.sh tarball and
  the Arch PKGBUILD;
- debian/ packaging files are in the repo and CI builds the .deb for real
  inside a debian:trixie container on every relevant change;
- ITP filed as #NNNNNN, source package on mentors.debian.net: <link>.

I'd be grateful if a DD could sponsor the upload — happy to maintain it
under the input-method team.

Thanks,
LinhBH.CoM <hunglinhpt@gmail.com>
```

Sau khi sponsor upload vào **unstable (sid)**: gói đi qua kiểm thử tự động →
testing → Ubuntu tự sync ở chu kỳ phát hành tiếp theo.

## 2. Việc upstream đã chuẩn bị (bằng chứng cho maintainer)

- `debian/` hoàn chỉnh: control (Build-Depends đúng tên gói Debian),
  rules (cargo --offline vendored cho PPA / thường cho mentors), changelog,
  copyright (dep5), source/format 3.0 (quilt).
- Multiarch: `/usr/lib/<DEB_HOST_MULTIARCH>/fcitx5/libtextvn-fcitx5.so` —
  cùng thư mục mà engine tự dò (`textvn-common.sh`: `/usr/lib/<triplet>/fcitx5`,
  `/usr/lib64/fcitx5`, libdir từ pkg-config Fcitx5Core).
- IBus component XML sinh ra với đường dẫn cuối `/usr/libexec/ibus-engine-textvn`.
- QA: `ci-deb.yml` — dpkg-buildpackage thật trong `debian:trixie`, kiểm nội
  dung .deb + lintian.
- PPA path: `tools/release/prepare-ppa.sh` (vendored source → debuild → dput).
