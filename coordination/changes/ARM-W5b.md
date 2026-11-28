<!-- TASK: ARM-W5b · commit: 28b3b593 + 3fc0f2f1 + 729ec914 + dacb5cfd · agent: claude-opus-7e41b2d9 -->
## VI
- **ARM-W5b — chứng minh TIP đã cài nạp ĐÚNG kiến trúc trên Windows ARM64 thật, không cần route input**
  (thay phần gõ phím của ARM-W5 mà runner `windows-11-arm` không làm được — E89).
- `tools/win/tip-load-probe.c` (C, không phụ thuộc): `CoCreateInstance(CLSID TIP TextVN, CLSCTX_INPROC_SERVER,
  IID_ITfTextInputProcessor)` đi đúng đường COM thật qua `InprocServer32` mà bộ cài ghi (64-bit view = forwarder
  ARM64X `textvn-tsf.dll`, WOW6432Node = `textvn-tsf-x86.dll`), rồi `GetModuleHandleW` xem DLL đích nào nằm trong
  tiến trình: ARM64 ⇒ `textvn-tsf-arm64.dll`, x64 giả lập ⇒ `textvn-tsf-x64.dll`, x86 ⇒ `textvn-tsf-x86.dll`.
  Exit 0 = đúng, 1 = CoCreateInstance lỗi, 2 = sai DLL (hoặc nạp thêm DLL khác). Không ghi registry, không gõ
  phím, không log chữ (G8).
- `ci-windows.yml` job ARM64: build 3 probe (arm64/x64 bắt buộc, x86 khi runner có toolset) bằng `/W4 /WX`;
  `test-installer.ps1` chạy probe sau khi cài máy thật, rồi **đột biến bắt buộc mặc định**: thay forwarder bằng
  DLL ARM64 thường ⇒ probe x64 phải đỏ; khôi phục ⇒ phải xanh lại.
- Hai sự cố trong lúc làm, đã ghi mã: `|` trong `%P%` bị cmd hiểu là pipe (đổi dấu tách thành `,`); **E93** —
  `echo X=1>>file` trong cmd là chuyển hướng handle 1 nên biến đột biến rỗng ⇒ bước đột biến bị bỏ qua im lặng
  (fail-open). Sửa: đột biến chạy mặc định (chỉ tắt bằng `=0`) + gate `xtask/src/hygiene.rs` test `e93_*`.

## EN
- **ARM-W5b — prove the installed TIP loads the RIGHT architecture on real Windows ARM64, without input routing**
  (replaces the typing part of ARM-W5, which the `windows-11-arm` runner cannot do — E89).
- `tools/win/tip-load-probe.c` (C, no dependencies): `CoCreateInstance(TextVN TIP CLSID, CLSCTX_INPROC_SERVER,
  IID_ITfTextInputProcessor)` goes through the real COM path via the `InprocServer32` the installer wrote (64-bit
  view = ARM64X forwarder `textvn-tsf.dll`, WOW6432Node = `textvn-tsf-x86.dll`), then `GetModuleHandleW` checks which
  target DLL is in the process: ARM64 ⇒ `textvn-tsf-arm64.dll`, emulated x64 ⇒ `textvn-tsf-x64.dll`, x86 ⇒
  `textvn-tsf-x86.dll`. Exit 0 = correct, 1 = CoCreateInstance failed, 2 = wrong DLL (or an extra DLL loaded). No
  registry writes, no typing, no text logging (G8).
- `ci-windows.yml` ARM64 job: builds the 3 probes (arm64/x64 required, x86 when the runner has the toolset) with
  `/W4 /WX`; `test-installer.ps1` runs them after the real machine install, then a **mutation that is on by
  default**: replace the forwarder with a plain ARM64 DLL ⇒ the x64 probe must go red; restore ⇒ it must be green
  again.
- Two incidents along the way, both coded: `|` inside `%P%` was parsed by cmd as a pipe (separator changed to `,`);
  **E93** — `echo X=1>>file` in cmd redirects handle 1, so the mutation variable was empty and the mutation step was
  silently skipped (fail-open). Fix: mutation runs by default (only `=0` disables it) + gate test `e93_*` in
  `xtask/src/hygiene.rs`.

## Bằng chứng / Evidence
- ci-windows run 46132150921 (718a7d70, gồm dacb5cfd) — 4/4 job xanh; job `Windows ARM64 native (windows-11-arm)`:
  ```
  tip-load-probe arch=arm64 CoCreateInstance=ok loaded: textvn-tsf-arm64.dll (mong doi textvn-tsf-arm64.dll) => PASS
  tip-load-probe arch=x64 CoCreateInstance=ok loaded: textvn-tsf-x64.dll (mong doi textvn-tsf-x64.dll) => PASS
  tip-load-probe arch=x86 CoCreateInstance=ok loaded: textvn-tsf-x86.dll (mong doi textvn-tsf-x86.dll) => PASS
  PASS ARM-W5b TIP nap dung kien truc trong 3 tien trinh (CoCreateInstance qua registry that)
  tip-load-probe arch=x64 CoCreateInstance hr=0x800700C1 => FAIL
  PASS ARM-W5b dot bien: thay forwarder bang DLL ARM64 => probe x64 do (exit 1)
  tip-load-probe arch=x64 CoCreateInstance=ok loaded: textvn-tsf-x64.dll (mong doi textvn-tsf-x64.dll) => PASS
  ```
  `0x800700C1` = `ERROR_BAD_EXE_FORMAT`: tiến trình x64 không nạp được DLL ARM64 thường — đúng lý do ADR-015 cần
  forwarder ARM64X. / `0x800700C1` = `ERROR_BAD_EXE_FORMAT`: an x64 process cannot load a plain ARM64 DLL — exactly
  why ADR-015 needs the ARM64X forwarder.

## Giới hạn / Limits
- Chứng minh NẠP đúng kiến trúc, không chứng minh GÕ được (runner không route input — E89). Gõ thật trên máy
  Windows ARM64 vẫn là bước của chủ repo. / Proves correct-architecture LOADING, not typing (the runner does not
  route input — E89). Real typing on a Windows ARM64 device remains an owner step.

## Tài liệu / Docs
- `docs/specs/project-common-errors.md` + `docs/specs/regression-index.md` (E93). Không sửa CHANGELOG/README (tệp
  phát hành — release manager gộp).
