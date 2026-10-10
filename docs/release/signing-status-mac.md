# Trạng thái ký số macOS — Developer ID, notarization, stapling

> Cập nhật 2026-10-09 · task MAC-055 / rủi ro RM3 (`docs/30-macos/P2-0-MASTER-PLAN.md`).
> File này được `scripts/build-macos.sh`, `scripts/package-macos-pkg.sh`,
> `scripts/notarize-macos.sh` và `.github/workflows/release.yml` trỏ tới — chỉ ghi
> **trạng thái thật** của bản phát hành. Lớp GPG/Sigstore chung cho 3 nền tảng: `signing.md`.

## 1. Tóm tắt

| Lớp ký | Bản phát hành macOS hiện tại | Ghi chú |
|---|---|---|
| GPG `.asc` + Sigstore `.cosign.sig`/`.cosign.cert`, `SHA256SUMS.txt` clearsign | ✅ mọi file (`.pkg`, ZIP, tar.gz) từ v0.2.27 | Bắt buộc trong CI: thiếu khoá/cosign thì job publish dừng, chữ ký vừa tạo được tự verify (`tools/release/sign-artifacts.sh`) |
| **Developer ID Application** (`codesign` từng bundle, hardened runtime, secure timestamp) | ❌ chưa — bundle ký **ad-hoc** (`-`) | Script đã sẵn, chạy khi có secret Apple |
| **Developer ID Installer** (`productsign` cho `.pkg`) | ❌ chưa — `.pkg` **chưa ký** | Script đã sẵn, chạy khi có secret Apple |
| **Notarization + stapling** (`notarytool` + `stapler`) | ❌ chưa chạy | Script đã sẵn, chạy khi có cert + xác thực notarytool |

Chưa có tài khoản Apple Developer Program/chứng chỉ Developer ID, và job `macos` của
`release.yml` chưa truyền secret Apple nào — nên mọi bản phát hành cho tới nay đều ký
ad-hoc. Đây là release candidate, không phải bản production.

## 2. Ảnh hưởng tới người dùng

- Gatekeeper chặn lần mở đầu `TextVN-mac-v<ver>.pkg` và `TextVN.app` tải từ Internet.
  macOS 15 Sequoia trở lên **không còn** cách "click phải → Open": vào **System Settings ›
  Privacy & Security** → **Open Anyway** (cuộn xuống phần Security), rồi mở lại.
- Bản nén `TextVN-macos-universal-v<ver>.zip`/`.tar.gz`: sau khi chép
  `TextVN-IM.app` vào `~/Library/Input Methods/` và `TextVN.app` vào `Applications`, gỡ cờ
  quarantine: `xattr -dr com.apple.quarantine "<đường dẫn tới từng .app>"`.
- Muốn chắc file không bị sửa: kiểm chữ ký GPG/Sigstore theo `signing.md` §3 trước khi mở.

Hướng dẫn cài đầy đủ: `docs/user-guide.md`, mục macOS.

## 3. Tooling đã nối (chỉ chạy khi có secret)

| Script | Biến môi trường | Việc làm |
|---|---|---|
| `scripts/build-macos.sh` | `APPLE_DEVELOPER_ID_APP` = `Developer ID Application: <Tên> (<TEAMID>)` (tên cũ `DEVELOPER_ID`); `APPLE_KEYCHAIN` (tuỳ chọn) | `codesign --options runtime --timestamp` từng bundle với `packaging/macos/TextVN.entitlements` rồi `codesign --verify --strict`. Có thêm xác thực notarytool → notarize + staple từng `.app` **trước** khi đóng ZIP/tar.gz. Không có biến → ký ad-hoc và in cảnh báo. |
| `scripts/notarize-macos.sh` | Một trong ba: `APPLE_NOTARY_PROFILE` · `APPLE_NOTARY_KEY_ID` + `APPLE_NOTARY_ISSUER_ID` + `APPLE_NOTARY_KEY` (nội dung `.p8`) hoặc `APPLE_NOTARY_KEY_PATH` · `APPLE_NOTARY_APPLE_ID` + `APPLE_NOTARY_TEAM_ID` + `APPLE_NOTARY_PASSWORD` (app-specific password). Tên cũ vẫn nhận: `NOTARY_PROFILE`, `APPLE_ID`/`APPLE_TEAM_ID`/`APPLE_APP_PASSWORD` | Từ chối artifact chưa ký Developer ID; `notarytool submit --wait` → `stapler staple` + `validate` → `spctl -a` (gate) |
| `scripts/package-macos-pkg.sh` | `DEVELOPER_ID_INSTALLER` = `Developer ID Installer: <Tên> (<TEAMID>)`; cờ `--notarize` | `productsign` + `pkgutil --check-signature`; `--notarize` gọi `notarize-macos.sh` cho `.pkg` |

## 4. Việc còn lại để bật (chủ repo)

1. Đăng ký Apple Developer Program; tạo hai chứng chỉ **Developer ID Application** và
   **Developer ID Installer** (khác nhau — bundle dùng cái thứ nhất, `.pkg` dùng cái thứ hai).
2. Tạo xác thực notarytool (App Store Connect API key khuyến nghị cho CI).
3. Lưu cert (`.p12` + mật khẩu) và xác thực notarytool thành GitHub secrets; thêm vào job
   `macos` của `release.yml` bước nạp cert vào keychain tạm của runner và truyền các biến ở
   §3, rồi gọi `scripts/package-macos-pkg.sh --notarize` (bước này **chưa có** trong workflow).
4. Kiểm trên bản phát hành: `spctl -a -vv -t exec TextVN.app`, `spctl -a -vv -t install
   TextVN-mac-v<ver>.pkg`, `xcrun stapler validate`. Đạt thì cập nhật bảng §1 và bỏ lưu ý
   Gatekeeper trong `README.md`, `docs/user-guide.md`.

Cho tới khi xong bước 4: mọi tài liệu và release notes phải ghi rõ bản macOS **chưa** ký
Developer ID/notarize.
