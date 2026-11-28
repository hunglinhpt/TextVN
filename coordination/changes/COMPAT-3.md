<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# COMPAT-3 — E2E gõ tiếng Việt THẬT trong phiên desktop Linux (claude-opus-c4c5a567)

## VI

- **Mới**: `scripts/e2e-desktop.sh <de> <tarball>` và job `e2e-desktop` (`ci-linux.yml`, ma trận DE).
  Khác `e2e_ibus`/`e2e_fcitx5` (chỉ điều khiển input context qua D-Bus, không có app GUI): ở đây phím
  đi đúng đường của người dùng, từ X server/compositor → IM module của toolkit → IBus/Fcitx5 →
  TextVN, rồi chữ được cam kết vào ô soạn thảo của **một app thật** (Mousepad) và đọc lại.
  - Gói được cài bằng **chính `install.sh` của tarball** (per-user, HOME cách ly), nên kiểm luôn
    đường cài thật.
  - **Oracle gate**: trước khi gõ GUI, danh sách case được replay qua engine của gói vừa cài
    (`textvn replay`, cấu hình mặc định). Case sai sẽ đỏ ở bước này, không bị đổ lỗi cho desktop.
  - Bộ case **dùng chung** với harness macOS (`tools/e2e/typing-cases.txt`, `smoke-imk.sh` đọc cùng
    tệp — G22). Lần chạy đầu bắt được **2 dòng sai có từ bản macOS** (chưa từng chạy):
    `hoaf=hoà` (đó là kiểu `new`; mặc định `old` cho `hòa`) và `vietj=việt` (Telex cần `ee`).
- **xfce-x11 — ĐẠT**: Xvfb + xfwm4 + IBus + Mousepad (GTK3 `im-ibus`), **6/6 PASS**, oracle 6/6
  (run 46125806626 và các run sau). Đây là bài E2E gõ tiếng Việt thật đầu tiên trong một phiên
  desktop Linux. Các bài học đã sửa trong script:
  - phải đặt engine **sau** khi app có input context đang focus;
  - mã thoát của `ibus engine NAME` không đáng tin ⇒ kiểm bằng engine hiện hành;
  - một lệnh lỗi im lặng dưới `set -e` ⇒ thêm bẫy `ERR` in dòng lỗi, lệnh lỗi và các log.
- **sway-wayland — GIỚI HẠN MÔI TRƯỜNG (đạt một phần)** (Sway headless + Fcitx5 qua input-method-v2 + Mousepad Wayland gốc).
  Các lớp đã khoanh vùng:
  - Fcitx5 thứ hai do D-Bus tự kích hoạt (thiếu `WAYLAND_DISPLAY`) ⇒ đã sửa bằng
    `dbus-update-activation-environment` + `NameHasOwner`;
  - `wl-paste` trong Sway headless luôn “Nothing is copied” ⇒ đổi sang đọc lại bằng cách lưu tệp;
  - **ngay cả khi chưa có IM**, phím `wtype` không tới Mousepad (tệp 0 byte) ⇒ lỗi thuộc lớp môi
    trường. Đã thử ép focus bằng `swaymsg` và `wtype -s 300` (chờ keymap; gợi ý của
    claude-opus-7e41b2d9), giữ MỘT bàn phím ảo sống suốt phiên (seat có `capabilities: 2`,
    thiết bị `wlr_virtual_keyboard_v1`) — phím VẪN không tới Mousepad (job 46136096814).
  - Kết luận: bước kiểm môi trường thất bại ⇒ script thoát **mã 3 = GIỚI HẠN** (job ghi
    `::warning::` + job summary); lỗi SAU bước đó (khi TextVN tham gia) vẫn đỏ. KHÔNG được ghi
    “gõ thật trên Wayland” trong README/CHANGELOG.
- **Phase B (chưa làm)**: GNOME Wayland (IBus) và KDE Plasma Wayland (Fcitx5) cần compositor đầy đủ
  cùng cách bơm phím riêng.
- **Cho release manager (G18)**: README VI/EN có thể nêu “E2E gõ tiếng Việt thật trong phiên Xfce/X11
  (IBus)”. KHÔNG được ghi Wayland cho tới khi `sway-wayland` xanh.

## EN

- **New**: `scripts/e2e-desktop.sh <de> <tarball>` and the `e2e-desktop` job (`ci-linux.yml`, DE
  matrix). Unlike `e2e_ibus`/`e2e_fcitx5` (D-Bus input-context only, no GUI app), keys follow the
  real user path: X server/compositor → toolkit IM module → IBus/Fcitx5 → TextVN, and the text is
  committed into **a real app** (Mousepad) and read back.
  - The package is installed with **the tarball's own `install.sh`** (per-user, isolated HOME), so
    the real install path is covered too.
  - **Oracle gate**: before GUI typing, the case list is replayed through the installed package's
    engine (`textvn replay`, default config). A wrong case fails at that step instead of being
    blamed on the desktop.
  - The case list is **shared** with the macOS harness (`tools/e2e/typing-cases.txt`, also read by
    `smoke-imk.sh`, G22). The first run caught **2 wrong lines inherited from the macOS list** (which
    had never run): `hoaf=hoà` (that is the `new` style; the default `old` gives `hòa`) and
    `vietj=việt` (Telex needs `ee`).
- **xfce-x11 — PASS**: Xvfb + xfwm4 + IBus + Mousepad (GTK3 `im-ibus`), **6/6 PASS**, oracle 6/6
  (run 46125806626 and later). This is the first real Vietnamese-typing E2E inside a Linux desktop
  session. Lessons fixed in the script:
  - the engine must be set **after** the app has a focused input context;
  - the exit code of `ibus engine NAME` cannot be trusted ⇒ check the current engine instead;
  - one command failed silently under `set -e` ⇒ an `ERR` trap now prints the failing line, the
    command and the logs.
- **sway-wayland — ENVIRONMENT LIMITATION (partial)** (headless Sway + Fcitx5 via input-method-v2 + native Wayland
  Mousepad). Layers isolated so far:
  - a second, D-Bus-activated Fcitx5 without `WAYLAND_DISPLAY` ⇒ fixed with
    `dbus-update-activation-environment` + `NameHasOwner`;
  - `wl-paste` in headless Sway always reports “Nothing is copied” ⇒ read back by saving to a file;
  - **even with no IM**, `wtype` keys do not reach Mousepad (0-byte file) ⇒ an environment-layer
    issue. Tried forcing focus with `swaymsg` and `wtype -s 300` (wait for the keymap; suggested by
    claude-opus-7e41b2d9), and keeping ONE virtual keyboard alive for the whole session (seat
    `capabilities: 2`, device `wlr_virtual_keyboard_v1`) — keys STILL never reach Mousepad (job
    46136096814).
  - Conclusion: a failed environment check exits with **code 3 = LIMITATION** (the job emits
    `::warning::` plus a job summary line); a failure AFTER that check (TextVN involved) stays red.
    Do NOT claim “real typing on Wayland” in README/CHANGELOG.
- **Phase B (not done)**: GNOME Wayland (IBus) and KDE Plasma Wayland (Fcitx5) need full compositors
  and their own key-injection paths.
- **For the release manager (G18)**: README VI/EN may state “real Vietnamese typing E2E in an
  Xfce/X11 session (IBus)”. Do NOT mention Wayland until `sway-wayland` is green.
