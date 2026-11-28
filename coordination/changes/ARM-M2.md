<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# ARM-M2 — E2E IMK gõ thật trên runner macOS arm64 (claude-opus-c4c5a567)

## VI

- **Job mới `imk-e2e`** (`ci-macos.yml`, `needs: package-candidate`, `macos-latest` = Apple Silicon):
  1. Kiểm `uname -m = arm64`; cài **đúng gói `.pkg` candidate** của commit (artifact
     `TextVN-macos-candidate-<sha>`) bằng `sudo installer` như người dùng ⇒ `/Library/Input Methods`,
     kiểm binary có lát `arm64` (`lipo -archs`), đăng ký LaunchServices (`lsregister -f`). Lần chạy
     đầu cài thẳng vào `~/Library/Input Methods`: TIS (318 nguồn) không bao giờ liệt kê TextVN dù
     `TISRegisterInputSource = noErr` — nên đổi sang đường gói thật.
  2. `tools/mac/tis-select.swift` (mới, Swift thuần + Carbon TIS): `TISRegisterInputSource` →
     `TISEnableInputSource` → `TISSelectInputSource`, rồi **xác minh** nguồn hiện hành thuộc bundle
     `vn.textvn.im`. TIS từ chối hoặc không thấy nguồn ⇒ exit 1 = **đỏ thật** (không được coi là SKIP).
  3. Mở TextEdit, chờ tiến trình `TextVN-IM`, chạy `tools/mac/smoke-imk.sh`: 6 case lấy từ `corpus/mac`
     (giá trị mong đợi là output engine đã verify, không bịa), undo ⌘Z, toggle EN/VN Ctrl+Shift+Space.
  4. Kết quả: exit 0 = PASS; exit 3 (runner thiếu quyền Accessibility/GUI) = `::warning::` +
     **GIỚI HẠN** ghi vào job summary kèm lý do, **không pass ngầm**; mã khác = đỏ.
- Lớp cầu (bridge) IMK vẫn gate ở job `swift` (`IMKLibTests`) và `replay --adapter mac`, như
  acceptance ARM-M2 yêu cầu khi runner không cho quyền.
- `tools/mac/README.md`: thêm `tis-select.swift` và chỗ chạy CI của `smoke-imk.sh`.
- G8: chỉ in ID nguồn nhập và chuỗi case cố định của corpus, không log nội dung người dùng.

## EN

- **New job `imk-e2e`** (`ci-macos.yml`, `needs: package-candidate`, `macos-latest` = Apple Silicon):
  1. Asserts `uname -m = arm64`; installs **the exact candidate `.pkg`** of the commit (artifact
     `TextVN-macos-candidate-<sha>`) with `sudo installer`, as a user would ⇒ `/Library/Input Methods`,
     checks the binary has an `arm64` slice (`lipo -archs`) and registers it with LaunchServices
     (`lsregister -f`). The first run installed straight into `~/Library/Input Methods`: TIS (318
     sources) never listed TextVN although `TISRegisterInputSource = noErr`, so the job now uses the
     real package path.
  2. `tools/mac/tis-select.swift` (new, pure Swift + Carbon TIS): `TISRegisterInputSource` →
     `TISEnableInputSource` → `TISSelectInputSource`, then **verifies** that the current input
     source belongs to bundle `vn.textvn.im`. If TIS refuses or finds no source ⇒ exit 1 = a
     **real failure** (never treated as SKIP).
  3. Opens TextEdit, waits for the `TextVN-IM` process, then runs `tools/mac/smoke-imk.sh`: 6 cases
     from `corpus/mac` (expected values are verified engine output, not invented), ⌘Z undo, and the
     Ctrl+Shift+Space EN/VN toggle.
  4. Outcome: exit 0 = PASS; exit 3 (runner lacks Accessibility/GUI) = `::warning::` plus a
     **LIMITATION** line with the reason in the job summary, **never a silent pass**; any other
     code = red.
- The IMK bridge layer stays gated by the `swift` job (`IMKLibTests`) and `replay --adapter mac`, as
  the ARM-M2 acceptance requires when the runner does not grant the permission.
- `tools/mac/README.md`: adds `tis-select.swift` and where CI runs `smoke-imk.sh`.
- G8: only input-source IDs and the fixed corpus case strings are printed; no user content is logged.
