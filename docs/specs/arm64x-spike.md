# Spike SPK-ARM64X — một TIP cho tiến trình x64 giả lập và ARM64 native (`spikes/arm64x-forwarder`)

> Task gỡ rủi ro: **ARM-W2** (forwarder ARM64X), **ARM-W3** (`register` trên máy ARM64) — `../10-shared/P0-7-TASKS-ARM-SF.md`
> Luồng W, `../10-shared/P0-5-ARM-PLATFORMS-PLAN.md` §5. Spike code: `../../spikes/arm64x-forwarder/` (+ DLL mẫu
> `../../spikes/tsf-min/`) · CI: `../../.github/workflows/spikes.yml` jobs `arm64x-build` (windows-2022) → `arm64x-run` (windows-11-arm).
> Chủ repo: "dùng CI trước" (chưa có máy Snapdragon) ⇒ spike chỉ chạy trên CI.

## 1. Câu hỏi và tiêu chí đóng

Trên Windows 11 ARM64 chỉ có MỘT registry 64-bit cho cả tiến trình ARM64 native và x64 giả lập, nên một
CLSID TIP chỉ có một đường dẫn `InprocServer32`. Đề xuất: DLL **ARM64X pure forwarder** (công thức
"Arm64X pure forwarder" của Microsoft; tiền lệ Weasel `arm64x_wrapper`) — export phía native chuyển sang
`tsf_min_arm64.dll`, export phía EC (x64) chuyển sang `tsf_min_x64.dll`.

Rủi ro chưa ai trả lời cho TextVN: Weasel cài DLL vào **System32**; TextVN cài **per-user**
(`%LOCALAPPDATA%\Programs\TextVN`). Forwarded export có tìm thấy DLL đích nằm CẠNH forwarder khi tiến trình
(Notepad, Edge…) có thư mục làm việc khác không?

**Đóng khi** trên `windows-11-arm`, từ thư mục làm việc khác và thư mục cài có dấu cách:

| # | Kỳ vọng | Ý nghĩa |
|---|---|---|
| 1 | `probe-x64.exe` (giả lập) + `LoadLibraryExW(…, LOAD_WITH_ALTERED_SEARCH_PATH)` (cách COM nạp InprocServer32) ⇒ `load=ok factory=ok tip=ok target=tsf_min_x64.dll` | x64 app dùng được TIP qua forwarder |
| 2 | `probe-arm64.exe` (native), cùng cách nạp ⇒ `target=tsf_min_arm64.dll` | ARM64 app dùng bản native |
| 3 | Cả hai với `LoadLibraryW` không cờ | đối chứng đường nạp thường |
| 4 | `probe-x64.exe` nạp thẳng `tsf_min_arm64.dll` ⇒ **thất bại** | đối chứng âm: probe thật sự phân biệt kiến trúc |
| 5 | `dumpbin /headers tsf_min.dll` có `ARM64X` | đúng loại ảnh |

Đạt #1–#5 ⇒ ARM-W2 dùng forwarder + 2 DLL cạnh nhau trong thư mục cài per-user. #1/#2 thất bại do không
tìm thấy DLL đích ⇒ phương án B: đường dẫn tuyệt đối không được trong forwarder ⇒ cài 2 DLL đích vào thư
mục có trong đường tìm DLL, hoặc build ARM64X "thật" (ARM64 + ARM64EC trong cùng ảnh — Rust chưa hỗ trợ
trực tiếp; cần link thủ công `arm64ec-pc-windows-msvc` + `aarch64`). Kết quả ghi ở §3.

## 2. Thiết kế

- `build.ps1` (x64 host): `cargo build --target {x86_64,aarch64}-pc-windows-msvc` cho `tsf-min` + probe;
  `vswhere` → `vcvarsall x64_arm64` → `cl /c` (ARM64) + `cl /c /arm64EC` (tệp rỗng) → **import lib**
  `link /lib /machine:x64 /def:x64.def` + `link /lib /machine:arm64 /def:arm64.def` → `link /dll /noentry
  /machine:arm64x /defArm64Native:arm64.def /def:x64.def <2 obj> <2 lib>` → `tsf_min.dll`; kèm `rsi-oracle` ARM64 cho PERF-2.
- `run.ps1` (windows-11-arm): chép 3 DLL vào `…\TextVN Spike ARM64X\bin`, chạy probe từ thư mục khác.
- Probe không ghi registry, không đăng ký TIP; chỉ `DllGetClassObject` → `IClassFactory::CreateInstance
  (ITfTextInputProcessor)` trong chính tiến trình probe. Log HRESULT của DLL mẫu chuyển `LOCALAPPDATA` sang thư mục tạm.
- Smoke cục bộ (x64, không forwarder): `SPK_ARM64X_ALLOW_SELF=1 arm64x-probe <tsf_min.dll>` ⇒ PASS cả hai
  cách nạp; đường dẫn sai ⇒ `load=FAIL hr=0x8007007e`, exit 1 (2026-11-27).

## 3. Kết quả CI

- Run 1 [#46028585161](https://github.com/hunglinhpt/TextVN/actions/runs/46028585161): build chéo Rust `aarch64-pc-windows-msvc`
  trên `windows-2022` **OK**, `cl /arm64EC` **OK**; link forwarder **đỏ** `LNK2001 unresolved DllGetClassObject (EC Symbol /
  Arm64 Native Symbol)` — thiếu hai import lib (`link /lib`) mà công thức Microsoft truyền vào lệnh link cuối. Đã thêm.
- Run 2: _chờ kết quả — ghi dòng `SPK-ARM64X …` của từng probe vào đây._
