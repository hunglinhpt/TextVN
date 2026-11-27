# spikes/ — mã thăm dò (không phát hành)

> Spike = mã nháp trả lời **một câu hỏi rủi ro** trước khi làm task thật. Không vào bộ cài, không vào
> `cargo test --workspace` (mỗi crate là workspace TÁCH RIÊNG), không bị gate `no_injection_apis`
> (copy sang `adapters/` vẫn bị chặn tại đích). CI: `.github/workflows/spikes.yml` — **chỉ dispatch**
> (`gh workflow run spikes.yml`), không gate PR. Cập nhật 2026-11-27 theo kế hoạch P0-5…P0-8.

## Danh mục

| Spike | Câu hỏi | Task được gỡ rủi ro | Trạng thái | Kết quả |
|---|---|---|---|---|
| [`tsf-min/`](tsf-min/) (WIN-002) | TSF viết bằng Rust thuần được không? | P1-1 TSF | **Đóng** 10/10 (2026-09-27) — nay là DLL TIP mẫu cho SPK-ARM64X | [`tsf-spike.md`](../docs/specs/tsf-spike.md), [`tsf-registration-spike.md`](../docs/specs/tsf-registration-spike.md) |
| [`uia-probe/`](uia-probe/) (WIN-004) | Độ trễ UIA + `IsPassword` xuyên tiến trình | nhận diện ô (field-detect) | **Đóng** (2026-09-27) | [`uia-spike.md`](../docs/specs/uia-spike.md) |
| [`hook-probe/`](hook-probe/) (WIN-005 → **SPK-RUNNER**) | Runner GHA có phiên desktop, SendInput, UIA không? Nay hỏi lại cho **`windows-2025` và `windows-11-arm`** | COMPAT-1, ARM-W5 (e2e Windows ARM64) | **Mở lại** 2026-11-27 — job `runner-probe` (ma trận 3 runner) | [`hook-spike.md`](../docs/specs/hook-spike.md) |
| [`arm64x-forwarder/`](arm64x-forwarder/) (**SPK-ARM64X**, mới) | MỘT CLSID TIP nạp được từ tiến trình **x64 giả lập** và **ARM64 native** qua DLL ARM64X pure forwarder đặt ở thư mục per-user (không System32)? | ARM-W2, ARM-W3 | **Mở** — chỉ chạy được trên CI (`arm64x-build` → `arm64x-run`) | [`arm64x-spike.md`](../docs/specs/arm64x-spike.md) |
| [`rsi-oracle/`](rsi-oracle/) (**SPK-RSI**, mới) | Số Dream RSI 22% có thật không? KSR-oracle, precision@ghost, nhiễu là bao nhiêu? | RSI-1, RSI-2, RSI-3, PERF-2 | **Có kết quả** 2026-11-27 (bắt được R2-174) — chạy lại trên 5 kiến trúc/OS ở CI | [`rsi-oracle-spike.md`](../docs/specs/rsi-oracle-spike.md) |

## Luật spike (bổ sung G1–G23)

1. **Một câu hỏi, một tiêu chí đóng** ghi trong doc `docs/specs/*-spike.md`; kết quả có số + link run CI.
2. **Workspace tách riêng** (`[workspace]` rỗng trong `Cargo.toml`, `Cargo.lock` riêng) — không đổi
   `Cargo.lock` gốc; build vào `target/spikes` (`CARGO_TARGET_DIR=target/spikes`).
3. **G8**: spike không ghi phím/chữ người dùng — chỉ chạy trên corpus của chính nó, in counter/HRESULT.
4. **Không đổi máy người dùng**: không ghi registry/không cài đặt; script có bước hệ thống chỉ chạy trên
   runner CI (tự huỷ). Log của DLL mẫu chuyển `LOCALAPPDATA` sang thư mục tạm.
5. **G22**: dùng lại mã dùng chung (crate `textvn-core`, CLSID của `tsf-min`) — không chép logic.
   Spike xong ⇒ **hấp thụ** vào task thật (ví dụ SPK-RSI → `tools/bench` ở RSI-1) rồi **xoá** spike.
6. Script PowerShell ASCII-only (gate `ascii-ps1`); Rust qua `cargo fmt` + `clippy -D warnings`.

## Chạy

```bash
# SPK-RSI (máy dev, ~10 s)
CARGO_TARGET_DIR=target/spikes cargo run --release --manifest-path spikes/rsi-oracle/Cargo.toml
SPK_RSI_DEBUG=1 …            # in thêm từng Tab nhận sai / câu lệch đích (chữ của corpus spike)

# Toàn bộ trên CI (SPK-RUNNER ×3 runner, SPK-ARM64X, SPK-RSI ×5 + so tất định)
gh workflow run spikes.yml && gh run watch --exit-status
```

---

# English

`spikes/` holds **non-shipping probe code**: each spike answers one risk question before the real task.
Every crate is a detached workspace (never part of `cargo test --workspace` or the installers); CI is the
dispatch-only `spikes.yml`. Current spikes: `tsf-min` (WIN-002, closed — now the sample TIP DLL),
`uia-probe` (WIN-004, closed), `hook-probe` (WIN-005, re-opened as **SPK-RUNNER** for `windows-2025` and
`windows-11-arm`), **SPK-ARM64X** (`arm64x-forwarder`: one TIP CLSID loaded by emulated x64 and native
ARM64 processes through an ARM64X pure forwarder in a per-user folder — ARM-W2/W3), **SPK-RSI**
(`rsi-oracle`: real KSR-oracle / precision / noise versus the 22% bench figure — RSI-1, found bug
R2-174). Rules: one question and exit criterion per spike, detached workspace, no user keystrokes (G8),
no changes to the user's machine, reuse shared code (G22) and delete the spike once absorbed.
