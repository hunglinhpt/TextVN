# ARM-L1 + ARM-L2 — CI Linux aarch64 native + gate `char` không dấu (claude-opus-c4c5a567)

## VI

- **ARM-L1**: job `linux-arm64` trong `ci-linux.yml` chạy NATIVE trên `ubuntu-24.04-arm` (glibc 2.39, GTK 4.14, Fcitx5 5.1)
  và `ubuntu-22.04-arm` (glibc 2.35, GTK 4.6, Fcitx5 5.0.14) — Raspberry Pi 5, NVIDIA Jetson/DGX Spark, máy chủ
  Graviton/Ampere: chặn `uname -m = aarch64`, `cargo test --workspace --locked`, `verify` + `sizes`, replay 4 adapter,
  ctest linux-common ASan + UBSan, build IBus/Fcitx5/settings + e2e ibus-daemon/fcitx5 thật. Job x86_64 giữ nguyên.
  Xanh cả 2 cột ở run [46098023352](https://github.com/hunglinhpt/TextVN/actions/runs/46098023352).
- **ARM-L2**: lượt ctest thứ hai trên x86 với `-funsigned-char -Wsign-compare` (char như aarch64) cạnh lượt có dấu mặc
  định. Đột biến `(unsigned char)c < 0x20` → `c < 0x20` trong `assist_result.c`: lượt có dấu ĐỎ (chuỗi Việt bị escape
  từng byte `áº·`), lượt không dấu xanh ⇒ cần cả hai (kiểm độc lập bởi claude-opus-9d3a41c7).
- **Bắt được lỗi thật nhờ cột 22.04-arm**: Fcitx5 5.0.x giữ Ctrl+Shift (EnumerateForwardKeys) — chẩn đoán in
  `fcitx CurrentInputMethod=keyboard-us`; cấu hình phím của phiên test gom về `scripts/lib/fcitx5-test-env.sh` (G22) và
  người dùng được `doctor` cảnh báo (R2-175).
- **E86**: commit board không còn kích hoạt CI (`coordination/**` trong `paths-ignore`); push lên main dùng nhóm
  concurrency theo commit — run của commit code không còn bị thay thế khi nhiều agent cùng đẩy.

## EN

- **ARM-L1**: job `linux-arm64` in `ci-linux.yml` runs NATIVELY on `ubuntu-24.04-arm` (glibc 2.39, GTK 4.14, Fcitx5 5.1) and
  `ubuntu-22.04-arm` (glibc 2.35, GTK 4.6, Fcitx5 5.0.14) — Raspberry Pi 5, NVIDIA Jetson/DGX Spark, Graviton/Ampere servers:
  `uname -m = aarch64` gate, `cargo test --workspace --locked`, `verify` + `sizes`, replay of all 4 adapters, linux-common
  ctest ASan + UBSan, IBus/Fcitx5/settings build + real ibus-daemon/fcitx5 e2e. x86_64 jobs unchanged. Both columns green
  in run [46098023352](https://github.com/hunglinhpt/TextVN/actions/runs/46098023352).
- **ARM-L2**: a second ctest pass on x86 with `-funsigned-char -Wsign-compare` (aarch64 char semantics) next to the default
  signed pass. Mutation `(unsigned char)c < 0x20` → `c < 0x20` in `assist_result.c`: the signed pass goes RED (Vietnamese
  text escaped byte by byte `áº·`), the unsigned pass stays green ⇒ both are needed (independently verified
  by claude-opus-9d3a41c7).
- **Real bug caught by the 22.04-arm column**: Fcitx5 5.0.x holds Ctrl+Shift (EnumerateForwardKeys) — diagnostics printed
  `fcitx CurrentInputMethod=keyboard-us`; the test-session hotkey config moved to `scripts/lib/fcitx5-test-env.sh` (G22)
  and users get a `doctor` warning (R2-175).
- **E86**: board commits no longer trigger CI (`coordination/**` in `paths-ignore`); pushes to main use a per-commit
  concurrency group — code-commit runs are no longer replaced when several agents push.

## Kiểm thử / Testing

CI ci-linux run 46098023352 (aarch64 24.04 + 22.04 xanh; x86 + 6 distro E2E + 2 smoke giữ nguyên xanh ở 46095917848).
