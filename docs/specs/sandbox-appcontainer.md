# R2-26 — TextVN trong tiến trình sandbox Windows (AppContainer / LPAC)

> Trạng thái: **đã sửa ở 0.2.93** (2026-11-27). Mã: `ipc/src/sandbox.rs`,
> `config/src/sandbox.rs`, `tray/src/sandbox_mirror.rs`, `tray/src/ipc_server.rs`
> (`sandbox_policy`, `Visibility`, `FrameBudget`), `adapters/windows-tsf/src/ipc_client.rs`,
> `adapters/windows-tsf/src/lib.rs` (`reload_config_from_file`), `cli/src/doctor.rs`
> (`W_SANDBOX_MIRROR`). Kiểm thử đầu-cuối: `tray/src/sandbox_e2e.rs`.

## 1. Vấn đề

Ô tìm kiếm Start, Calculator, Cài đặt, ứng dụng Store chạy trong **AppContainer** (một số
thành phần hệ thống dùng **LPAC**). DLL TSF nạp được vào đó (`register` cấp ACL đọc DLL cho
`S-1-15-2-1`/`S-1-15-2-2`, category `IMMERSIVESUPPORT`), nhưng:

- không đọc được `%APPDATA%\TextVN\config.json` / `state.json` ⇒ luôn Telex mặc định, bỏ qua
  VNI, kiểu bỏ dấu, tự viết hoa, Math…;
- không mở được pipe khay (DACL chỉ user/SYSTEM/Admins) ⇒ nút V/E và Ctrl+Shift ở app khác
  không tới, Ctrl+Shift trong ô tìm kiếm không tới các app khác.

Bộ gõ khác cùng gặp: EVKey #52/#85, OpenKey #121/#227; VietType chưa giải.

## 2. Số đo thật (spike 2026-11-27, Windows 11 26300, tiến trình con AppContainer + LPAC)

| Đo trong sandbox | AppContainer | LPAC | Hệ quả thiết kế |
|---|---|---|---|
| Đọc `config.json` thật | lỗi 5 | lỗi 5 | cần bản sao |
| `%LOCALAPPDATA%`, known-folder LocalAppData | **chuyển hướng** `…\Packages\<tên>\AC` | như AC | KHÔNG đặt bản sao ở LocalAppData |
| `%APPDATA%`, `%USERPROFILE%` | đường thật | đường thật | bản sao ở `%APPDATA%\TextVN\sandbox` |
| `HKLM\…\ProfileList\<SID>` | đọc được | lỗi 5 | chỉ là dự phòng |
| Tệp trong thư mục có ACE `FRFX` cho `S-1-15-2-1`/`S-1-15-2-2` | đọc được | đọc được (ACE `-2-2`) | không cần quyền thư mục cha |
| Pipe DACL cũ | lỗi 5 | lỗi 5 | cần pipe riêng |
| Pipe có ACE `GRGW` sandbox + nhãn `S:(ML;;NW;;;LW)` | nối + đọc/ghi được | như AC | pipe sandbox |
| `OpenProcess(QUERY_LIMITED)` lên tray | **bị từ chối** | bị từ chối | client không kiểm được tên exe server |
| Tạo pipe tên toàn cục `\\.\pipe\x` | **lỗi 5** (chỉ `LOCAL\…`) | lỗi 5 | sandbox không giả được pipe của tray |
| Tray `OpenProcess` + image name + token của client | được | được | tray biết app nào đang nối |

## 3. Thiết kế

### 3.1 Bản sao cấu hình (`%APPDATA%\TextVN\sandbox\`)

- Tray ghi khi khởi động, sau MỖI lần lưu `config.json` (trước khi broadcast `ConfigReload`) và
  `state.json`; bỏ qua khi nội dung không đổi; lỗi bản sao không bao giờ làm hỏng lần lưu thật.
- DACL **bảo vệ** `D:P(A;OICI;FA;;;<user>)(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;FRFX;;;S-1-15-2-1)(A;OICI;FRFX;;;S-1-15-2-2)` — sandbox chỉ đọc.
- `config.json` bản sao = **danh sách cho phép** (`SANDBOX_KEEP_KEYS`): tuỳ chọn gõ, gợi ý, Math,
  trợ lý, phím tắt. **Bỏ**: `macros`, `emoji`, `english_words` (`SANDBOX_STRIP_KEYS`);
  `style_profile` chỉ giữ `enabled`/`decided`; khoá lạ bị bỏ. Test gate
  `sandbox_keys_cover_schema_and_code` buộc mọi khoá schema/`Config` mới được phân loại.
- `state.json` chép nguyên (công tắc V/E, trợ lý, override theo **tên exe** — không phải chữ gõ).
- Nguồn cấu hình chọn theo **loại tiến trình** (`current_process_is_app_container`, token, cache
  một lần): AppContainer/LPAC chỉ đọc bản sao (`read_sandbox_file`: `%APPDATA%` → ProfileList →
  `%USERPROFILE%`, thử lại một lần khi tray đang thay tệp); tiến trình thường chỉ đọc tệp thật như
  trước — đọc lỗi thoáng qua không bao giờ rơi về bản sao thiếu gõ tắt (E81).

### 3.2 Pipe sandbox `\\.\pipe\textvn-ipc-v1-s<phiên>-sandbox`

- Pipe đầy đủ giữ nguyên DACL cũ — sandbox không mở được nó; kernel quyết định ai vào pipe nào
  (không cần tray đoán token của client).
- ACE sandbox chỉ cấp **quyền dữ liệu** `0x12008B` (`READ_DATA|WRITE_DATA|READ_EA|READ_ATTRIBUTES|
  READ_CONTROL|SYNCHRONIZE`), client xin `0x100083`. **Không** `GW`: `GENERIC_WRITE` của pipe gồm
  `FILE_CREATE_PIPE_INSTANCE` ⇒ ứng dụng sandbox tạo được instance server cùng tên (E80 — audit
  trước phát hành, đột biến xác nhận trên tiến trình AppContainer/LPAC thật).
- Client (`open_tray_pipe`): pipe phiên → tên cũ → pipe sandbox **chỉ khi** pipe đầy đủ trả lỗi 5.
  Tiến trình thường bận/không thấy pipe không bao giờ rơi xuống pipe sandbox.
- Xác thực server trên pipe sandbox: cùng phiên; nếu `OpenProcess` bị từ chối thì chấp nhận
  (kẻ chiếm tên chỉ có thể là tiến trình thường của CHÍNH user — vốn đã sửa được config).
- `sandbox_policy` (liệt kê đủ mọi `Message`, thêm biến thể mới phải phân loại mới biên dịch):

| Lệnh | Pipe sandbox |
|---|---|
| `Hello`, `Subscribe`, `GetSnapshot`, `Ping`, `Pong`, `ToggleGlobal`, `ToggleViEn{"*"}` | cho phép |
| `StyleUpdate` | chỉ bộ đếm Dream RSI (trần 100 000/bộ đếm/lần), bỏ từ & cặp sửa |
| `Shutdown`, `ToggleViEn{app khác}`, `ToggleAssist`, `ToggleMath`, `CycleMathMode`, `OpenEmojiPicker`, `AssistNotification`, `AssistReport`, `CrashReport`, … | từ chối (trả `Ack`, đếm `sandbox_denied`) |

- Snapshot/`StateUpdate` cho client sandbox chỉ có `*`, `assist` và override của **chính app**
  (tên exe tray đọc từ tiến trình client).
- Chống làm nghẽn: tối đa 32 kết nối sandbox cùng lúc, 40 frame/giây/kết nối (vượt ⇒ đóng).
- Client sandbox không gửi nội dung người dùng: drain tự học chỉ còn bộ đếm; không toast/lệnh.

## 4. Mô hình đe doạ (mọi ứng dụng Store của người dùng nối được pipe sandbox và đọc bản sao)

| Ứng dụng sandbox độc hại làm được | Đánh giá |
|---|---|
| Đọc tuỳ chọn gõ (kiểu gõ, bảng mã, Math…) và danh sách tên exe có override V/E | chấp nhận — không phải nội dung gõ |
| Bật/tắt tiếng Việt toàn cục (như bấm Ctrl+Shift) | phiền toái, đã có debounce 250 ms + hạn mức frame |
| Đọc gõ tắt, emoji riêng, từ tiếng Anh tự thêm, từ đã học | **không** — không có trong bản sao |
| Thoát tray, đổi override app khác, bơm từ vào hồ sơ học, spam toast | **không** — `sandbox_policy` |
| Giả pipe tray | **không** — AppContainer không tạo được pipe tên toàn cục, cũng không tạo được instance của pipe sandbox (ACE chỉ quyền dữ liệu — E80) |
| Làm nghẽn tray bằng kết nối/frame | giới hạn 32 kết nối, 40 frame/s |

## 5. Giới hạn còn lại

- Trong ô tìm kiếm Start / ứng dụng Store **không** bung gõ tắt, emoji riêng, từ tiếng Anh tự
  thêm và không dùng từ đã học (thiết kế — quyền riêng tư); không hiện toast trợ lý.
- `%APPDATA%` chuyển hướng ra thư mục mạng (Folder Redirection): sandbox không có quyền mạng ⇒
  vẫn Telex mặc định.
- Tray ≤ 0.2.92 không ghi bản sao/pipe sandbox: cập nhật cả gói (TIP mới + tray mới) rồi mở lại
  tray. `textvn-cli doctor` báo `W_SANDBOX_MIRROR` khi bản sao thiếu/thiếu ACL/lệch config.

## 6. Kiểm thử

- `tray/src/sandbox_e2e.rs` — chạy binary test trong **AppContainer thật** rồi **LPAC thật**
  (`CreateAppContainerProfile` + `PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES`): config thật lỗi 5,
  bản sao đúng VNI và không chứa gõ tắt, pipe đầy đủ lỗi 5 → nối pipe sandbox, snapshot không lộ
  app khác, 4 lệnh nguy hiểm bị từ chối, Ctrl+Shift tới tray, override app khác giữ nguyên, từ
  học từ sandbox không vào hồ sơ, KHÔNG tạo được instance pipe sandbox, nhận ra mình là AppContainer.
  Đột biến kiểm chứng: pipe sandbox dùng DACL cũ ⇒ test đỏ; ACE `GRGW` ⇒ test đỏ ("mạo danh tray").
- `config/src/sandbox.rs`, `ipc/src/sandbox.rs`, `tray/src/sandbox_mirror.rs`,
  `tray/src/ipc_server.rs` (`sandbox_policy_allows_only_minimal_commands`,
  `sandbox_visibility_hides_other_apps`, `sandbox_frame_budget_resets_every_second`),
  `cli/src/doctor.rs` (`r2_26_sandbox_mirror_verdict`).
- Kiểm tay mỗi bản (A-release): gõ VNI trong ô tìm kiếm Start sau khi chọn VNI ở khay; Ctrl+Shift
  trong ô tìm kiếm đổi icon khay.
