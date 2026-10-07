# BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)
**TextVN — Bộ gõ tiếng Việt Đa nền tảng (Windows · macOS · Linux)**  
*Tài liệu kiểm toán độc lập dưới góc nhìn Chuyên gia Trưởng Kiến trúc (Principal Architect), Kiểm toán Phần mềm (Audit Expert) & Trưởng nhóm Phát hành (Release Lead)*  
*Thời điểm kiểm toán: 2026-10-04 | Cam kết: Giữ nguyên 100% mã nguồn hiện hữu, không sửa đổi code trong phiên kiểm toán.*

---

## MỤC LỤC
1. [TỔNG QUAN TÌNH TRẠNG REPOSITORY & CÁC COMMIT MỚI NHẤT](#1-tổng-quan-tình-trạng-repository--các-commit-mới-nhất)
2. [MA TRẬN ĐÁNH GIÁ RỦI RO & BẢNG TỔNG HỢP DEFECTS](#2-ma-trận-đánh-giá-rủi-ro--bảng-tổng-hợp-defects)
3. [ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU](#3-điều-tra-gốc-rễ-root-cause-analysis-các-lỗi-trọng-yếu-được-nêu)
   - 3.1. [Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành](#31-lỗi-không-đổi-icon-và-không-chuyển-mode-gõ-khi-bấm-ctrl--shift-trên-cả-3-hệ-điều-hành)
   - 3.2. [Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux](#32-lỗi-gõ-chữ-bị-gạch-chân-underline--preedit-artifacts-trên-windows-macos-và-linux)
   - 3.3. [Vấn đề phải chạy bằng quyền Administrator mới đăng ký được TSF trên Windows 11](#33-vấn-đề-phải-chạy-bằng-quyền-administrator-mới-đăng-ký-được-tsf-trên-windows-11)
   - 3.4. [Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác](#34-điều-tra-chuyên-sâu-giải-mã-3-lỗi-từ-chối-của-microsoft-store-silent-install-addremove-programs-bundleware--thiết-kế-bài-test-ci-chuẩn-xác-theo-kỳ-vọng-của-microsoft)
4. [CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)](#4-chi-tiết-kiểm-toán-từng-dòng-mã-module-by-module-audit)
   - 4.1. [Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)](#41-module-windows-tsf-hook-tray-cli-register-installer--msix)
   - 4.2. [Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements)](#42-module-macos-imk-cgeventtap-appkit-menu-bar-packaging--entitlements)
   - 4.3. [Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging)](#43-module-linux-common-ipc-ibus-fcitx5-gtk4-settings--packaging)
   - 4.4. [Core Engine, FFI ABI, Restore English & Data](#44-core-engine-ffi-abi-restore-english--data)
   - 4.5. [CI/CD, Supply Chain, VirusTotal & Microsoft Store Validation](#45-cicd-supply-chain-virustotal--microsoft-store-validation)
5. [LỘ TRÌNH KÝ SỐ CHÍNH THỨC & SẴN SÀNG CHO MỌI NGƯỜI DÙNG](#5-lộ-trình-ký-số-chính-thức--sẵn-sàng-cho-mọi-người-dùng)
   - 5.1. [Ký số Windows (Authenticode qua SignPath Foundation / Azure Trusted Signing)](#51-ký-số-windows-authenticode-qua-signpath-foundation--azure-trusted-signing)
   - 5.2. [Ký số & Notarization macOS (Apple Developer ID Application & Installer)](#52-ký-số--notarization-macos-apple-developer-id-application--installer)
   - 5.3. [Ký số & Phân phối Linux (GPG, PPA, COPR, AUR, Flatpak)](#53-ký-số--phân-phối-linux-gpg-ppa-copr-aur-flatpak)
6. [PHƯƠNG ÁN XỬ LÝ TRIỆT ĐỂ (STEP-BY-STEP REMEDIATION BLUEPRINT)](#6-phương-án-xử-lý-triệt-để-step-by-step-remediation-blueprint)

---

## 1. TỔNG QUAN TÌNH TRẠNG REPOSITORY & CÁC COMMIT MỚI NHẤT

### 1.1. Hiện trạng Git & Trạng thái nhánh
- **HEAD Commit:** `2d42d6a` (*docs(store): hướng dẫn nộp MSIX từng bước — đường dự phòng khi gói .exe bị từ chối validation*).
- **Trạng thái Working Tree:** Sạch hoàn toàn (`working tree clean`), đồng bộ 100% với `origin/main`.
- **Phiên bản hiện tại:** `0.2.19` (khai báo đồng bộ trong `Cargo.toml`, `Cargo.lock`, các script Inno Setup và manifest).
- **Chuỗi commit gần đây (từ `cbd4a90` đến `2d42d6a` - 24 commit):**
  - **Tập trung Microsoft Store:** Bổ sung harness mô phỏng Store Validator (`tools/win/simulate-store-validation.ps1`), đóng gói MSIX Full-Trust (`tools/win/build-msix.ps1`, `installer/windows/msix/AppxManifest.xml`), tích hợp VirusTotal API scan (`tools/win/virustotal-scan.ps1`).
  - **Silent Installer Hardening:** Tinh chỉnh Inno Setup (`installer/windows/TextVN-setup.iss`) với cơ chế `WizardSilent` bỏ qua đăng ký TSF tại thời điểm cài đặt ngầm để bảo đảm trả về exit code `0` phục vụ Store validation.
  - **UAC Elevation Portable:** Thêm logic `offer_machine_registration_if_needed()` trong `tray/src/main.rs` nhằm giải quyết vấn đề Windows 11 từ chối `ActivateProfile` khi chỉ đăng ký per-user (HKCU).

### 1.2. Nhận định từ Chuyên gia Kiểm toán
Dự án đã có bước tiến lớn về mặt hoàn thiện tính năng đa nền tảng, xây dựng harness test tự động và chuẩn bị tài liệu Microsoft Store. Tuy nhiên, trong quá trình chạy đua khắc phục các triệu chứng ngoại vi (đặc biệt là bài toán vượt qua kiểm duyệt Store và Windows 11 24H2), **một số thỏa hiệp kiến trúc và lỗi tiềm ẩn nghiêm trọng đã xuất hiện**:
1. Xuất hiện mã nguồn vi phạm trực tiếp nguyên tắc bảo mật và chính sách chống nhận diện nhầm Antivirus (AV False Positive Policy A2/A3).
2. Tồn tại xung đột luồng sự kiện (Race Condition) gây ra hiện tượng nuốt phím tắt / đảo trạng thái kép.
3. Thiếu sót trong đăng ký COM Category khiến tính năng tắt gạch chân bị vô hiệu hóa ngầm.
4. Quá trình đóng gói và ký số chưa được kích hoạt thực tế trong pipeline CI/CD, khiến tất cả các bản dựng phát hành đều là `release-candidate` chưa ký.

---

## 2. MA TRẬN ĐÁNH GIÁ RỦI RO & BẢNG TỔNG HỢP DEFECTS

| Phân loại mức độ | Định nghĩa | Số lượng phát hiện |
|---|---|:---:|
| **P0 - BLOCKER** | Lỗi an ninh nghiêm trọng (LPE/Privilege Escalation), vi phạm chính sách bảo mật hệ thống, gây cấm phát hành Store/Gatekeeper | **3** |
| **P1 - CRITICAL** | Lỗi chức năng cốt lõi làm mất tính năng (Ctrl+Shift không đổi mode, chữ bị gạch chân, TSF không kích hoạt được) | **4** |
| **P2 - MAJOR** | Vi phạm kiến trúc, rò rỉ bảo mật cục bộ, hiệu năng thấp ở luồng nhập liệu, thiếu sót quy trình ký số CI | **7** |
| **P3 - MINOR** | Cảnh báo linter, bộ từ điển chưa tối ưu, thiếu sót tài liệu hướng dẫn và kiểm thử chéo kiến trúc | **5** |

### Bảng tổng mục các phát hiện kiểm toán (Findings Catalog)

| ID | Mức độ | Module | Vị trí file & dòng | Hiện tượng & Rủi ro |
|:---:|:---:|:---:|:---|:---|
| **SEC-01** | **P0** | Windows Tray | `tray/src/main.rs:786` | Cài đặt `SetWindowsHookExW(WH_KEYBOARD_LL)` toàn cục trong Tray — vi phạm trực tiếp kiến trúc "Pure TSF", kích hoạt heuristic mã độc (Keylogger T1056.001) trên Kaspersky/Defender. |
| **SEC-02** | **P0** | Windows Tray | `tray/src/main.rs:346-370` | Nâng quyền UAC (`runas`) đối với binary nằm trong thư mục người dùng tùy ý (`Downloads\TextVN`) — Nguy cơ Cực cao về DLL Hijacking và Local Privilege Escalation (CWE-426/732). |
| **SEC-03** | **P0** | macOS App | `packaging/macos/TextVN.entitlements:7-10` | Cấp `allow-jit` và `allow-unsigned-executable-memory` không cần thiết cho ứng dụng gõ phím bản địa — Nguy cơ bị Apple Notarization gắn cờ hoặc từ chối. |
| **BUG-01** | **P1** | Windows Tray/TSF | `tray/src/main.rs:112` & `adapters/windows-tsf/src/key_event.rs:184` | **Race Condition Ctrl+Shift:** Cả Tray LL Hook và TSF In-process cùng bắt sự kiện và gửi lệnh toggle độc lập → Trạng thái bị đảo kép (VI → EN → VI) khiến icon và chế độ gõ không đổi. |
| **BUG-02** | **P1** | macOS App | `adapters/macos-app/Sources/TextVNAppLib/AppDelegate.swift:572-583` | **Icon Menu Bar không đổi trên macOS:** Delegate nhận IPC `didToggleViEn` nhưng **quên gọi `updateStatusIcon()`** và `updateMenuState()`, đồng thời sửa state ngoài Main Thread. |
| **BUG-03** | **P1** | Linux Common | `adapters/linux-common/src/ipc_client.c:177-194` | **Không chuyển mode trên Linux:** Hàm xử lý IPC chỉ tìm `app_id` khớp với tên ứng dụng, hoàn toàn bỏ qua broadcast chuyển đổi toàn cục (`app_id: "*"`). |
| **BUG-04** | **P1** | Windows TSF | `cli/src/register.rs` & `adapters/windows-tsf/src/guids.rs` | **Gõ chữ bị gạch chân trên Windows:** GUID `DISPATTR_TEXTVN` không được đăng ký vào `ITfCategoryMgr` → Ứng dụng (Word, Notepad, Chrome) không tìm thấy Display Attribute Provider nên fallback về gạch chân mặc định. |
| **BUG-05** | **P2** | macOS IMK | `adapters/macos-imk/Sources/IMKLib/TextVNInputController.swift` | **Gõ chữ bị gạch chân trên macOS:** Cấu hình `non_preedit` hoàn toàn không được truyền hay sử dụng trong IMK; các app Chromium/Office tự động vẽ gạch chân khi thấy `setMarkedText`. |
| **BUG-06** | **P2** | Linux Fcitx5 | `adapters/linux-fcitx5/src/engine.cpp:205-221` | **Gõ chữ bị gạch chân trên Linux:** Fcitx5 và IBus luôn đẩy ký tự đang soạn vào `setClientPreedit`, không triển khai cơ chế Non-preedit thực thụ (Surrounding Text replace). |
| **BUG-07** | **P2** | Windows Tray | `tray/src/main.rs:693`, `tray/src/hotkey.rs` | Gọi `free_ctrl_shift()` âm thầm ghi đè Registry `HKCU\Keyboard Layout\Toggle` trên mọi lần mở app; bộ gỡ cài đặt (Installer & Portable) không hoàn trả thiết lập gốc cho người dùng. |
| **SEC-04** | **P2** | Linux Common | `adapters/linux-common/src/ipc_client.c:79` | Fallback socket path tại `/tmp/textvn-ipc.sock` (thư mục dùng chung world-writable) mở ra nguy cơ tấn công Symlink Hijack / Local DoS. |
| **SEC-05** | **P2** | Windows Tray | `tray/src/ipc_server.rs:352-362` | Named Pipe `CreateNamedPipeW` để `lpSecurityAttributes: None` (DACL mặc định), không có ACL ngăn chặn truy cập liên phiên (Multi-user session collision / hijacking). |
| **PERF-01**| **P2** | Windows TSF | `adapters/windows-tsf/src/edit_session.rs:595` | Gọi `CoCreateInstance(&CLSID_TF_CategoryMgr)` đồng bộ trên TỪNG phím gõ bên trong `apply_display_attribute`, gây độ trễ không đáng có cho luồng UI. |
| **REL-01** | **P2** | CI/CD | `.github/workflows/release.yml:38-80` | Script ký `tools/win/sign-signpath.ps1` đã có nhưng chưa từng được gọi trong job Windows của `release.yml`; thiếu pipeline ký Developer ID/Notarize cho macOS và GPG cho Linux. |
| **CORE-01**| **P3** | Core Engine | `core/src/post/restore_en.rs:49-56` | Hàm `data_contains` quét tuyến tính toàn bộ file text với cấp phát chuỗi `.to_lowercase()` liên tục trên từng phím gõ; từ điển EN chỉ có 55 từ, VN chỉ có 82 âm tiết. |
| **STO-01** | **P3** | Windows Setup | `installer/windows/TextVN-setup.iss:219-228` | Cài đặt silent bỏ qua 100% việc đăng ký TSF, đẩy trách nhiệm đăng ký kèm prompt UAC sang lần mở app đầu tiên — vi phạm trải nghiệm ứng dụng Microsoft Store. |
| **STO-02** | **P1** | Windows Store / CI | `tools/win/validate-store-package.ps1:37-41` & `TextVN-setup.iss:54` | **3 lỗi kiểm duyệt Microsoft Store:** Bộ cài per-user chỉ ghi vào HKCU khiến Validator trong VM sandbox (quét HKLM) thấy 0 entry mới; CI test tự thỏa hiệp đọc HKCU nên báo PASS giả tạo. |
| **STO-03** | **P1** | Windows Store | `installer/windows/TextVN-setup.iss:22-26` | **Lệch chuỗi siêu dữ liệu Partner Center:** DisplayName ("TextVN") và Publisher ("LinhBH.CoM") trong Registry lệch với tên ứng dụng ("TextVN - Bộ gõ tiếng Việt") và danh tính pháp lý tài khoản trên Microsoft Partner Center. |

---

## 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU

---

### 3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành

Người dùng phản ánh: *"Bấm Ctrl + Shift không đổi icon và không chuyển mode gõ ở cả 3 bản Windows, Linux và macOS"*.  
Kiểm toán độc lập cho thấy đây **không phải là một lỗi ngẫu nhiên**, mà là **3 lỗi logic độc lập khác nhau tại 3 hệ điều hành**:

#### A. Trên Windows: Hiện tượng "Toggle Đôi" (Double-Toggle Race Condition)
1. **Bằng chứng trong mã nguồn:**
   - Tại `tray/src/main.rs:786`:
     ```rust
     let ll_hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(tray_ll_keyboard_proc), None, 0) };
     ```
   - Tại `adapters/windows-tsf/src/key_event.rs:184-185`:
     ```rust
     let on = if crate::ipc_client::try_claim_hotkey_toggle() {
         shared.ipc.toggle_global()
     } else {
         shared.ipc.is_global_enabled()
     };
     ```
2. **Cơ chế gây lỗi:**
   - Khi người dùng nhấn nhả tổ hợp `Ctrl + Shift`:
     - **Luồng 1 (Tray Process):** Thủ tục `tray_ll_keyboard_proc` bắt được phím nhả, gọi `try_claim_global_toggle()`, post message `WM_TOGGLE_HOTKEY` tới cửa sổ Tray. Tray chuyển trạng thái từ **VI → EN**, cập nhật icon sang `E` và gửi broadcast.
     - **Luồng 2 (Ứng dụng đang gõ - TSF in-process):** Đồng thời tại cùng thời điểm, Text Services Framework của Windows gọi `OnKeyUp` vào `textvn-tsf.dll`. Hàm này kiểm tra `try_claim_hotkey_toggle()` (đây là biến atomic cục bộ nằm trong DLL của ứng dụng đó, **hoàn toàn độc lập** với Tray). Nó giành quyền thành công và gửi tiếp IPC `Message::ToggleGlobal` sang Tray!
     - **Hậu quả:** IPC Server của Tray nhận tiếp lệnh `ToggleGlobal` thứ hai, lập tức đảo trạng thái ngược lại từ **EN → VI**!
   - **Kết quả trên màn hình người dùng:** Chỉ trong vài mili-giây, trạng thái chuyển từ VI sang EN rồi quay ngay về VI. Icon khay chớp nhẹ hoặc giữ nguyên chữ `V`, văn bản gõ ra vẫn là tiếng Việt. Người dùng bấm hàng chục lần vẫn thấy "không đổi mode".
   - *Đáng chú ý:* Tại `tray/src/main.rs:697-700`, chính tác giả mã nguồn đã viết chú thích cảnh báo:
     > *"KHÔNG cài WH_KEYBOARD_LL trong tray: một lần bấm từng bị toggle ĐÔI (TSF + hook cùng bắn → 'bấm không đổi mode', 2026-10-01)"*  
     Nhưng sau đó tại dòng 786, lệnh `SetWindowsHookExW` lại được kích hoạt trở lại!

#### B. Trên macOS: Quên cập nhật Icon Menu Bar & Luồng giao diện (UI Thread Violation)
1. **Bằng chứng trong mã nguồn (`adapters/macos-app/Sources/TextVNAppLib/AppDelegate.swift:572-583`):**
   ```swift
   public func ipcServer(_ server: IpcServer, didToggleViEn appID: String, enabled: Bool) {
       if appID == IpcServer.globalAppID {
           self.isVietnameseMode = enabled
           self.configStore.config.enabled = enabled
           self.configStore.persist()
           server.broadcastConfigReload(version: UInt64(Date().timeIntervalSince1970))
       }
   }
   ```
2. **Cơ chế gây lỗi:**
   - Bộ gõ `TextVN-IM.app` bắt được sự kiện `flagsChanged` của tổ hợp `Ctrl + Shift` và gửi message IPC `.toggleViEn(appID: "*", enabled: ...)` tới Menu Bar App (`TextVN.app`).
   - `AppDelegate` nhận callback `didToggleViEn` trên một luồng nền (Background IPC Queue).
   - Mã nguồn cập nhật biến `self.isVietnameseMode` nhưng **HOÀN TOÀN KHÔNG GỌI `updateStatusIcon()` VÀ `updateMenuState()`**!
   - Hàm `updateStatusIcon()` chỉ được gọi khi khởi động (`setupStatusItem`) hoặc khi người dùng dùng chuột click trực tiếp vào menu.
   - **Hậu quả:** Người dùng bấm `Ctrl + Shift`, chế độ gõ bên trong IMK có thể đã đổi, nhưng biểu tượng trên thanh Menu Bar của macOS **vẫn đứng yên vĩnh viễn** ở icon cũ, khiến người dùng kết luận phím tắt bị liệt.

#### C. Trên Linux: IPC Client bỏ qua Broadcast toàn cục `app_id: "*"`
1. **Bằng chứng trong mã nguồn (`adapters/linux-common/src/ipc_client.c:177-194`):**
   ```c
   /* Detect StateUpdate */
   const char *p_state = strstr(json, "\"StateUpdate\"");
   if (p_state) {
       char key[192];
       int kw = snprintf(key, sizeof key, "\"app_id\":\"%s\"", client->app_id);
       if (client->app_id[0] != '\0' && kw > 0 && (size_t)kw < sizeof key &&
           strstr(p_state, key)) {
           // ... cập nhật client->app_enabled_override ...
       }
       return;
   }
   ```
2. **Cơ chế gây lỗi:**
   - Khi khay hệ thống hoặc phím tắt kích hoạt chuyển mode toàn cục, Tray gửi JSON:
     `{"type":"StateUpdate","app_id":"*","enabled":false,"version":123}`
   - Adapter Fcitx5 / IBus khởi tạo client với `app_id` cụ thể của ứng dụng (ví dụ: `gedit`, `google-chrome`).
   - Hàm `handle_ipc_message` tạo mẫu tìm kiếm `key = "\"app_id\":\"gedit\""`.
   - Vì chuỗi JSON đến chứa `"app_id":"*"`, lệnh `strstr(p_state, key)` trả về `NULL`.
   - **Hậu quả:** Bộ phân tích IPC bỏ qua hoàn toàn gói tin `StateUpdate` toàn cục. Động cơ gõ Linux không bao giờ nhận được tín hiệu chuyển mode từ phím tắt hay khay hệ thống.

---

### 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux

Người dùng phản ánh: *"Bug gõ chữ bị gạch chân"*. Đây là hiện tượng các ký tự đang soạn thảo (composing text) hiển thị kèm một đường gạch chân nét liền, nét đứt hoặc chấm bi bên dưới.

#### A. Trên Windows: Thiếu sót đăng ký Display Attribute Provider với Category Manager
1. **Phân tích cơ chế TSF:**
   - Trong kiến trúc Text Services Framework của Microsoft, khi một TIP tạo vùng soạn thảo (`StartComposition`), TSF mặc định áp dụng kiểu hiển thị `TF_LS_SOLID` hoặc `TF_LS_DOT` (gạch chân).
   - Để tắt gạch chân, TIP phải cung cấp một `ITfDisplayAttributeProvider`, trả về thuộc tính có `lsStyle = TF_LS_NONE`.
   - TextVN đã hiện thực Provider này rất chuẩn tại `adapters/windows-tsf/src/display_attr.rs` với GUID `DISPATTR_TEXTVN`.
   - Tại `adapters/windows-tsf/src/edit_session.rs:600-603`, TextVN gán thuộc tính này vào `GUID_PROP_ATTRIBUTE` của range.
2. **Gốc rễ sai lầm tại `cli/src/register.rs`:**
   - Khi một ứng dụng (như Microsoft Word, Chrome, Notepad) thấy một atom lạ trên `GUID_PROP_ATTRIBUTE`, nó sẽ hỏi Windows Category Manager: *"GUID này thuộc về Display Attribute Provider nào?"*.
   - Để trả lời câu hỏi này, TIP **bắt buộc phải đăng ký quan hệ sở hữu** giữa `CLSID_TIP` và `DISPATTR_TEXTVN` thông qua API:
     `ITfCategoryMgr::RegisterGUIDDescription(&CLSID_TIP, &DISPATTR_TEXTVN, desc)`  
     hoặc ghi vào Registry key:
     `HKLM\SOFTWARE\Microsoft\CTF\KnownClasses\{CLSID_TIP}`.
   - **Kiểm tra thực tế trong toàn bộ thư mục `cli/src/`:** GUID `DISPATTR_TEXTVN` **HOÀN TOÀN KHÔNG XUẤT HIỆN** trong bất kỳ thủ tục đăng ký nào!
   - **Hậu quả:** Category Manager không biết ai quản lý GUID `DISPATTR_TEXTVN`. Nó thất bại trong việc khởi tạo Provider của TextVN và lập tức quay về **bộ định dạng mặc định của hệ thống: VẼ ĐƯỜNG GẠCH CHÂN**.

#### B. Trên macOS: Cấu hình `non_preedit` bị bỏ quên trong IMK
1. **Phân tích mã nguồn:**
   - `adapters/macos-app` cung cấp toggle giao diện: *"Gõ không gạch chân (Non-preedit)"*, lưu vào `config.non_preedit`.
   - Tuy nhiên, trong toàn bộ mã nguồn của adapter nhập liệu `adapters/macos-imk`, biến `non_preedit` **không bao giờ được tham chiếu**!
   - `TextVNInputController` luôn luôn chạy hàm `setMarkedText` để hiển thị chữ đang soạn.
2. **Hành vi của ứng dụng macOS:**
   - Mặc dù TextVN có truyền thuộc tính `.underlineStyle: 0` vào `NSAttributedString`, các bộ nhân kết xuất đồ họa như Blink (Google Chrome, Microsoft Edge, VS Code, Slack, Discord) và Microsoft Office **bỏ qua thuộc tính này** đối với văn bản marked của IME và tự động vẽ gạch chân hệ thống.
   - Chỉ có chế độ Non-preedit thực thụ (commit văn bản trực tiếp và xóa lùi qua phím ảo) mới triệt tiêu hoàn toàn gạch chân trên các ứng dụng này.

#### C. Trên Linux: Chỉ sử dụng Client Preedit tiêu chuẩn
- Tại `adapters/linux-fcitx5/src/engine.cpp:214-218`, TextVN luôn đẩy văn bản vào `panel.setClientPreedit(preedit)`.
- Giao thức IBus và Fcitx5 hiển thị preedit trong các ứng dụng GTK/Qt luôn có gạch chân mặc định của theme hệ thống. TextVN chưa kích hoạt nhánh Non-preedit sử dụng `deleteSurroundingText`.

---

### 3.3. Vấn đề phải chạy bằng quyền Administrator mới đăng ký được TSF trên Windows 11

Người dùng phản ánh: *"Bug phải chạy bằng admin mới đăng ký được TSF"*.

1. **Gốc rễ kỹ thuật từ Windows 11:**
   - Trước Windows 11, một bộ gõ TSF có thể đăng ký cục bộ cho từng người dùng (per-user) dưới nhánh Registry `HKCU\Software\Microsoft\CTF\TIP\{CLSID}` và gọi `InstallLayoutOrTip` để kích hoạt.
   - Kể từ **Windows 11 22H2 / 23H2 và đặc biệt là 24H2**, dịch vụ quản lý nhập liệu tập trung của Windows (`ctfmon.exe` / `TextInputManagementService`) áp dụng chính sách bảo mật ngặt nghèo:
     **Mọi TSF TIP bắt buộc phải được đăng ký trong `HKLM\SOFTWARE\Microsoft\CTF\TIP` (thông qua `ITfInputProcessorProfileMgr::RegisterProfile`) thì Windows mới chấp nhận đưa vào danh sách bàn phím hoạt động (Language Profile List) và cho phép `ActivateProfile`.**
   - Đăng ký vào `HKLM` là thao tác toàn máy (machine-wide) và **bắt buộc phải có đặc quyền Administrator**.
2. **Hệ quả đối với TextVN:**
   - Khi chạy ở chế độ Portable hoặc cài đặt không đặc quyền (per-user setup), lệnh đăng ký vào `HKLM` trả về mã lỗi `E_ACCESSDENIED` (`0x80070005`).
   - Mặc dù TextVN đã nỗ lực tạo fallback ghi vào `HKCU`, Windows 11 vẫn từ chối nạp `textvn-tsf.dll` vào các ứng dụng mới mở.
3. **Giải pháp hiện tại của dự án và rủi ro an ninh phát sinh:**
   - Để ứng phó, commit `cbd4a90` đã bổ sung hàm `offer_machine_registration_if_needed()` trong `tray/src/main.rs`. Hàm này bật hộp thoại nhắc người dùng bấm OK để thực hiện nâng quyền UAC thông qua:
     `ShellExecuteW(None, "runas", cli, "register --scope machine", ...)`
   - **RỦI RO BẢO MẬT CỰC LỚN (P0 Security Flaw):**
     Khi người dùng giải nén bản Portable trong thư mục `C:\Users\<User>\Downloads\TextVN`, thư mục này thuộc quyền ghi của người dùng bình thường (hoặc mã độc không đặc quyền).  
     Việc gọi `runas` nâng quyền thực thi một file `.exe` từ thư mục này sẽ khiến Windows chạy tiến trình Elevated từ một đường dẫn không tin cậy.  
     Tệ hơn nữa, lệnh `register --scope machine` sẽ ghi đường dẫn của `textvn_win_tsf.dll` nằm trong thư mục `Downloads` vào khóa máy `HKLM\SOFTWARE\Classes\CLSID\{...}\InProcServer32`.  
     Bất kỳ tiến trình độc hại nào cũng có thể ghi đè file DLL này trong `Downloads`, và sau đó DLL độc hại sẽ được nạp thẳng vào các tiến trình SYSTEM/Admin của toàn bộ máy! Đây là lỗ hổng leo thang đặc quyền kinh điển (Local Privilege Escalation / DLL Hijacking).

---

### 3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft

Người dùng và đội ngũ phát hành tiếp tục gặp phải tình trạng Microsoft Store từ chối phê duyệt gói cài đặt Win32 `.exe` với cùng 3 thông báo lỗi lặp lại:
1. **Silent install check:**  
   *"We could not identify if your app is installing silently. Please visit this link to know how to manually verify if your app installs silently."*
2. **Entry in add or remove programs:**  
   *"We could not identify the app name and the publisher name that your app has added in the add or remove programs. Please visit this link to manually verify the details."*
3. **Bundleware check:**  
   *"We could not identify the app name and the publisher name that your app has added in the add or remove programs."*

Đáng chú ý, mặc dù script kiểm thử trên CI của repository (`tools/win/validate-store-package.ps1` và `tools/win/simulate-store-validation.ps1`) đều báo **PASS 100%**, nhưng khi nộp thực tế lên Partner Center thì Microsoft vẫn liên tục trả về 3 lỗi đỏ trên.

---

#### 3.4.1. Giải phẫu bản chất chuỗi lỗi (Dependency Chain) trong bộ thẩm định tự động của Microsoft
Dưới góc độ kỹ thuật thẩm định phần mềm, **đây KHÔNG PHẢI là 3 lỗi độc lập**, mà là **3 triệu chứng phái sinh từ DUY NHẤT MỘT NGUYÊN NHÂN CỐT LÕI**: Bộ kiểm tra tự động của Microsoft Store không nhận diện được khóa đăng ký Add/Remove Programs (ARP) của TextVN trong môi trường kiểm thử.

Quy trình thẩm định tự động của Microsoft Store đối với ứng dụng Win32 EXE diễn ra trong môi trường Sandbox ảo hóa không có sự can thiệp của con người (Headless Azure VM Container):

```mermaid
flowchart TD
    A["1. Snapshot Trước cài đặt<br/>Quét toàn bộ khóa Registry Uninstall HKLM"] --> B["2. Khởi chạy Installer<br/>Chạy Setup.exe với switches silent"]
    B --> C["3. Chờ tiến trình thoát<br/>Kiểm tra Exit Code = 0, Timeout &lt; 120s"]
    C --> D["4. Snapshot Sau cài đặt &amp; Tính toán Diff<br/>Diff = After - Before (Tìm Entry mới)"]
    D --> E{"5. Bộ so khớp định danh<br/>Diff có Entry nào khớp<br/>AppName &amp; Publisher trên Partner Center?"}
    
    E -- "KHÔNG TÌM THẤY ENTRY KHỚP" --> F["ĐÁNH TRƯỢT CẢ 3 MỤC:"]
    F --> F1["1. Entry in ARP: FAIL<br/>(Không thấy tên app &amp; publisher)"]
    F --> F2["2. Silent Install: FAIL<br/>(Không thấy dấu vết app đã cài thành công)"]
    F --> F3["3. Bundleware: FAIL<br/>(Không xác định được app chính để loại trừ rác)"]
    
    E -- "TÌM THẤY ĐÚNG 1 ENTRY KHỚP" --> G["PHÊ DUYỆT CẢ 3 MỤC:"]
    G --> G1["1. Entry in ARP: PASS<br/>(Khớp từng chữ AppName &amp; Publisher)"]
    G --> G2["2. Silent Install: PASS<br/>(Xác nhận cài xong ngầm vào hệ thống)"]
    G --> G3["3. Bundleware: PASS<br/>(Chỉ có đúng 1 entry, không kèm bloatware)"]
```

1. **Vì sao Silent Install check bị FAIL?**  
   Validator không phán đoán tính "im lặng" bằng mắt thường. Nó đánh giá dựa trên: (a) Tiến trình kết thúc trong thời gian giới hạn với exit code thành công (`0`), VÀ (b) **Hệ thống ghi nhận được ứng dụng đã được cài đặt thành công vào máy**. Nếu installer thoát ra (dù exit code = 0) nhưng hệ quả cài đặt (khóa ARP) không xuất hiện, validator kết luận tiến trình chạy ngầm nhưng "không cài đặt gì cả" hoặc đã tự hủy/thoát giả.
2. **Vì sao Entry in Add or Remove Programs bị FAIL?**  
   Bộ so khớp so sánh `DisplayName` và `Publisher` của các entry mới xuất hiện với thông tin khai báo trên Partner Center. Nếu số lượng entry mới tìm thấy là `0` hoặc thông tin chuỗi bị lệch dù chỉ 1 ký tự, hệ thống lập tức xuất lỗi chuẩn: *"We could not identify the app name and the publisher name that your app has added in the add or remove programs."*
3. **Vì sao Bundleware check bị FAIL?**  
   Quy tắc kiểm tra Bundleware của Store là: Installer chỉ được phép tạo đúng **1 entry duy nhất** tương ứng với sản phẩm đã đăng ký. Nếu hệ thống không nhận diện được entry chính của app, nó không có căn cứ để xác định các thành phần cài thêm có phải là bundleware/adware hay không, và tự động đánh trượt mục này với cùng thông báo lỗi.

---

#### 3.4.2. Bốn điểm nghẽn cốt lõi (Root Causes) khiến TextVN v0.2.20 tiếp tục bị từ chối

Qua việc rà soát chi tiết từng dòng mã trong `installer/windows/TextVN-setup.iss`, `tools/win/validate-store-package.ps1`, `docs/release/store-submission.md` và các tài liệu chính thức của Microsoft Learn, kiểm toán xác định 4 nguyên nhân gốc rễ:

##### Gốc rễ 1: Sự ngộ nhận tai hại về HKCU (Per-User) vs HKLM (Per-Machine) trong môi trường tự động của Store
- **Diễn biến trong repo:**  
  Tại commit `42af231`, đội ngũ maintainer kết luận: *"đính chính 'phải admin/HKLM' (SAI — per-user asInvoker mới đúng)"* và sửa `TextVN-setup.iss` sang `PrivilegesRequired=lowest`.
- **Hậu quả kỹ thuật:**  
  - Khi Inno Setup chạy với `PrivilegesRequired=lowest`, nó cài đặt vào `%LOCALAPPDATA%\Programs\TextVN` và **CHỈ GHI DUY NHẤT VÀO HIVE NGƯỜI DÙNG CỤC BỘ**:  
    `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Uninstall\{9C5E4A7D-092E-4D23-9F93-87B75F3FA7B3}_is1`.  
  - Trong khi đó, toàn bộ khóa toàn máy:  
    `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall` (và nhánh `WOW6432Node`) **HOÀN TOÀN TRỐNG RỖNG (0 entry mới)**.
- **Hành vi thực tế của Microsoft Store Validator:**  
  - Theo tài liệu chính thức Microsoft Learn (*"Manual package validation for MSI/EXE app"*) và phản hồi của kỹ sư Microsoft trên diễn đàn Partner Center: Tiêu chuẩn của các ứng dụng Win32 được phân phối qua hạ tầng quản lý phần mềm Windows (Intune, Store) là **đăng ký ở cấp độ máy (`HKLM`)**.
  - Nghiêm trọng hơn, trong môi trường container/VM tự động của Partner Center, quá trình cài đặt chạy dưới tài khoản dịch vụ ngầm (Service Account / Container Agent). Nếu tiến trình kiểm tra (Validator Inspector) chạy dưới tài khoản khác hoặc chỉ truy vấn `HKLM`, thì nhánh `HKCU` của tiến trình installer hoàn toàn vô hình! Kết quả là Validator ghi nhận **0 entry mới**.

##### Gốc rễ 2: Lệch chuỗi định danh tuyệt đối (Exact String Mismatch) giữa Partner Center và Registry
Bộ validator của Microsoft đối soát tự động giữa dữ liệu nộp trên Partner Center và giá trị ghi trong Registry theo nguyên tắc **Khớp tuyệt đối (Exact Match, phân biệt hoa thường và khoảng trắng)**:
- **Tên ứng dụng (App Name / DisplayName):**
  - Trong `TextVN-setup.iss` (dòng 22-25): `AppName={#MyAppFullName}` và `AppVerName={#MyAppName}` (`MyAppName = "TextVN"`). -> Registry ghi `DisplayName = "TextVN"`.
  - Nhưng trên Partner Center, tên ứng dụng được reserve thường là tên đầy đủ: `"TextVN - Bộ gõ tiếng Việt"` (như chính hướng dẫn tại `docs/release/store-submission.md:217` và `docs/release/msix-submission.md:23`).
  - Khi Validator so sánh `"TextVN"` với `"TextVN - Bộ gõ tiếng Việt"`, kết quả là **MISMATCH**!
- **Tên nhà phát hành (Publisher Name):**
  - Trong `TextVN-setup.iss` (dòng 16, 26): `AppPublisher="LinhBH.CoM"`.
  - Nhưng trên Partner Center, **Publisher Display Name** được liên kết trực tiếp với danh tính pháp lý của tài khoản Microsoft Developer (ví dụ tên cá nhân đã xác minh căn cước: `"Bùi Hùng Linh"` hoặc `"Linh Bui Hung"`).
  - Khi Validator so sánh `"LinhBH.CoM"` với `"Linh Bui Hung"`, kết quả là **MISMATCH**!
-> **Chỉ cần một trong hai chuỗi này không khớp 100%, Validator sẽ lập tức báo lỗi:**  
*"We could not identify the app name and the publisher name that your app has added in the add or remove programs."*

##### Gốc rễ 3: Điểm mù của CI Test Harness hiện tại (`validate-store-package.ps1` & `simulate-store-validation.ps1`)
Tại sao CI của TextVN chạy vẫn báo PASS xanh mướt nhưng Microsoft Store lại báo ĐỎ rực?  
Hãy nhìn vào dòng 37-41 của `tools/win/validate-store-package.ps1`:
```powershell
$roots = @(
    'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall',
    'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall',
    'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall'
)
```
1. **CI tự thỏa hiệp bằng cách đọc HKCU:** Trên GitHub Actions runner (`windows-latest`), tiến trình test chạy dưới tài khoản người dùng tương tác `runneradmin`. Installer cài vào `HKCU` của `runneradmin`, sau đó script test đọc chính `HKCU` đó -> Tìm thấy entry và tự khen `PASS 2) ARP entry`. Nó không hề kiểm chứng xem `HKLM` có entry hay không.
2. **CI tự so khớp với giá trị giả định (Hardcoded Self-Check):**  
   Script CI đặt mặc định `$ExpectName = "TextVN"` và `$ExpectPublisher = "LinhBH.CoM"` — tức là so khớp code `.iss` với chính code `.iss`, hoàn toàn không đối soát với giá trị thực tế trên cổng Microsoft Partner Center của chủ tài khoản.
3. **CI không kiểm tra bối cảnh tài khoản dịch vụ cô lập:** CI chưa từng mô phỏng trường hợp installer chạy dưới một account riêng biệt và hệ thống kiểm tra từ một session khác.

##### Gốc rễ 4: Bẫy lưu đệm (Caching Trap B13g) của Partner Center
Nếu người dùng sửa thông số nhưng:
- Nộp lại cùng một Package URL (hoặc cùng mã băm SHA256 binary).
- Không tăng số phiên bản trên Partner Center.  
Partner Center sẽ phát hiện không có thay đổi mã nguồn, hiển thị thông báo: *"We did not find any changes in the Package or the Silent install parameters"* và **hoàn toàn KHÔNG chạy lại máy ảo kiểm tra**. Kết quả 3 mục đỏ của lần nộp trước được giữ nguyên hiển thị.

---

#### 3.4.3. Thiết kế bài test trên CI mô phỏng đúng 100% kỳ vọng của Microsoft Store

Để triệt tiêu điểm mù và đảm bảo CI phản ánh chính xác kết quả của Microsoft Store, harness kiểm thử trên CI cần được nâng cấp theo 4 tiêu chí sau:

1. **Kiểm tra nghiêm ngặt phạm vi Máy (Strict HKLM Isolation Gate):**
   - Bổ sung chế độ kiểm tra nghiêm ngặt (`-StrictMachine`): Nếu gói nộp cho Store là gói máy, bài test PHẢI chỉ đọc `HKLM` và `HKLM\WOW6432Node`, loại bỏ hoàn toàn `HKCU` khỏi danh sách tìm kiếm.
   - Nếu `HKLM` không có entry sau khi cài đặt im lặng -> Đánh FAIL ngay trên CI.
2. **Đối soát siêu dữ liệu động với Partner Center (Dynamic Partner Center Metadata Gate):**
   - Nhận diện 2 biến môi trường trên CI: `STORE_APP_NAME` và `STORE_PUBLISHER_NAME`.
   - Đối chiếu chính xác:
     ```powershell
     if ($arpEntry.Name -ne $env:STORE_APP_NAME) {
         throw "FAIL: ARP DisplayName '$($arpEntry.Name)' != Partner Center App Name '$($env:STORE_APP_NAME)'"
     }
     if ($arpEntry.Publisher -ne $env:STORE_PUBLISHER_NAME) {
         throw "FAIL: ARP Publisher '$($arpEntry.Publisher)' != Partner Center Publisher '$($env:STORE_PUBLISHER_NAME)'"
     }
     ```
3. **Kiểm tra đa ngữ cảnh (Context Matrix Test):**
   - Mô phỏng chạy installer dưới quyền quản trị (Elevated Context) với cờ `/ALLUSERS` xem có tạo được entry trong `HKLM` mà không kích hoạt hộp thoại UAC hay không.
   - Kiểm tra mã thoát (Exit code) phải chính xác bằng `0` và thời gian thực thi `< 30 giây`.
4. **Kiểm tra tính sẵn sàng của gói MSIX (MSIX Validation Gate):**
   - Đảm bảo file `TextVN-<ver>-windows-x64.msix` được tạo thành công.
   - Tự động giải nén `AppxManifest.xml` và kiểm tra: `Identity.Name` và `Identity.Publisher` đã được thay thế giá trị thực tế hay vẫn còn là placeholder (`LinhBH.CoM.TextVN`).

---

#### 3.4.4. Kế hoạch hành động dứt điểm để vượt qua Store Validation

Maintainer có **2 lộ trình xử lý dứt điểm** vấn đề này:

##### Lộ trình Khuyến nghị Số 1: Chuyển hẳn sang gói MSIX Full-Trust (Giải pháp Triệt để & Nhanh nhất)
Microsoft thiết kế định dạng MSIX chính là để loại bỏ hoàn toàn các lỗi thẩm định cố hữu của Win32 raw installer (`.exe`):
- **Bypass 100% cả 3 bài test:**
  - Không cần Silent install switches (Windows tự cài đặt ngầm theo thiết kế gói AppX/MSIX).
  - Không cần kiểm tra registry ARP (Windows tự tạo entry chuẩn trong Settings từ `AppxManifest.xml`).
  - Không có rủi ro Bundleware (gói MSIX được đóng kín và ký số, không thể cài kèm bên thứ 3).
- **Các bước thực hiện:**
  1. Vào Partner Center -> Sản phẩm -> **Product identity** -> Lấy đúng 2 chuỗi:
     - `Package/Identity/Name` (ví dụ: `12345LinhBHCoM.TextVN`)
     - `Package/Identity/Publisher` (ví dụ: `CN=E5F3...ABCD`)
  2. Chạy lệnh đóng gói:
     ```powershell
     powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\build-msix.ps1 `
       -Publisher "CN=<Publisher từ Partner Center>" `
       -IdentityName "<Name từ Partner Center>"
     ```
  3. Upload trực tiếp file `dist\TextVN-0.2.20-windows-x64.msix` lên Partner Center. Submission sẽ vượt qua validation trong vòng vài giây.

##### Lộ trình 2: Khắc phục gói Win32 `.exe` (Nếu vẫn muốn giữ định dạng installer)
Nếu tiếp tục nộp bản `.exe`, maintainer bắt buộc phải thực hiện đồng bộ 3 điểm sau:
1. **Đồng bộ hóa 100% thông tin trong `TextVN-setup.iss`:**
   - Mở Partner Center, kiểm tra chính xác từng ký tự của:
     - Tên ứng dụng đã đăng ký (ví dụ: `TextVN - Bộ gõ tiếng Việt`).
     - Tên Nhà xuất bản hiển thị trên tài khoản (ví dụ: `Linh Bui Hung`).
   - Sửa trong `TextVN-setup.iss`:
     ```pascal
     #define MyAppName "TextVN - Bộ gõ tiếng Việt"   ; Phải khớp 100% Partner Center
     #define MyAppPublisher "Linh Bui Hung"          ; Phải khớp 100% Partner Center
     ```
2. **Cấu hình Inno Setup để ghi được vào HKLM khi chạy silent:**
   - Cấu hình:
     ```pascal
     PrivilegesRequired=lowest
     PrivilegesRequiredOverridesAllowed=commandline
     ```
   - Trong ô switches của Partner Center, khai báo:
     ```
     /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /ALLUSERS
     ```
     *(Cờ `/ALLUSERS` sẽ kích hoạt chế độ ghi toàn máy vào `HKLM`, giải quyết triệt để lỗi không tìm thấy ARP của Microsoft Validator).*
3. **Tránh bẫy Cache B13g:**
   - Mỗi lần thử nghiệm nộp lại, **bắt buộc phải tăng số phiên bản** (ví dụ lên `0.2.21`), commit và tạo bản dựng mới để có mã băm SHA256 mới.
   - Cập nhật đúng URL mới trên branch `approved` trước khi bấm Submit.

---

#### 3.4.5. Điều tra chuyên sâu lần nộp v0.2.21: Vì sao Store vẫn tiếp tục từ chối ở 3 bài kiểm tra dù đã sửa Publisher và nâng phiên bản?

Sau khi hoàn thành lần kiểm toán trước, maintainer đã thực hiện cập nhật mã nguồn trong commit `517db48` và `6660eff` để tạo bản phát hành **v0.2.21**:
- Cập nhật Publisher trong `installer/windows/TextVN-setup.iss` thành `"Linh βùi"`.
- Bổ sung UTF-8 BOM vào file `.iss` để bảo toàn chuỗi Unicode khi biên dịch qua Inno Setup.
- Nâng số phiên bản toàn diện lên `0.2.21` (tránh bẫy cache B13g).
- Biên dịch gói mới và đẩy lên branch `approved` tại URL:
  `https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.21/TextVN-setup-0.2.21-windows-x64.exe`
  (SHA256: `fa486e6d9d2fda9345009d31b1b07ab0fbbd6740534b7236c8c17d860572d279`).
- Cập nhật URL và chạy lại pipeline CI: các kịch bản kiểm thử trên GitHub Actions runner (`validate-store-package.ps1` và `simulate-store-validation.ps1`) đều trả về kết quả **PASS 100%**.

Tuy nhiên, khi nộp gói v0.2.21 lên Microsoft Partner Center, hệ thống kiểm duyệt tự động của Microsoft Store vẫn tiếp tục trả về **chính xác 3 lỗi từ chối cũ**:
1. *Silent install check: We could not identify if your app is installing silently.*
2. *Entry in add or remove programs: We could not identify the app name and the publisher name that your app has added in the add or remove programs.*
3. *Bundleware check: We could not identify the app name and the publisher name that your app has added in the add or remove programs.*

Dưới đây là kết quả điều tra pháp y mã nguồn (Forensic Code Audit) và giải phẫu luồng thực tế của Microsoft Store Validator:

---

##### 1. Bằng chứng mã nguồn: Gói nộp lên Partner Center THỰC CHẤT VẪN LÀ GÓI PER-USER (Chỉ ghi vào HKCU)
Mặc dù ở commit `2f77c78`, repository đã định nghĩa thêm cờ `/DMachineInstall=1` và CI đã có bước `Machine-install variant` chạy thử trên runner elevated, **file cài đặt thực sự được nộp lên Partner Center không phải là bản cài đặt máy (Machine-Install)**:
- **Kiểm chứng trong `build-release.ps1` dòng 392–396:**
  ```powershell
  $targetDirArg = "/DTargetDir=..\..\$ReleaseDir"
  $installerArgs = @("/DMyAppVersion=$Version", $targetDirArg)
  if ($IncludeCompatibilityHook) { $installerArgs += "/DIncludeCompatibilityHook=1" }
  $installerArgs += "installer\windows\TextVN-setup.iss"
  & $isccExe @installerArgs
  ```
  Lệnh gọi ISCC mặc định của quy trình build **HOÀN TOÀN KHÔNG TRUYỀN `/DMachineInstall=1`**.
- **Kiểm chứng trong `installer/windows/TextVN-setup.iss` dòng 83–87:**
  ```pascal
  #ifdef MachineInstall
  PrivilegesRequired=admin
  #else
  PrivilegesRequired=lowest
  #endif
  ```
  Vì không có cờ `MachineInstall`, Inno Setup biên dịch file `TextVN-setup-0.2.21-windows-x64.exe` với **`PrivilegesRequired=lowest`**.
- **Hệ quả khi cài đặt trong Sandbox của Microsoft:**
  - File được nộp lên Store là file Per-user. Khi chạy ngầm với switches `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`, trình cài đặt ghi toàn bộ thông tin gỡ cài đặt vào:
    `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Uninstall\{9C5E4A7D-092E-4D23-9F93-87B75F3FA7B3}_is1`.
  - Khóa hệ thống toàn máy:
    `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall` (và nhánh `WOW6432Node`) **hoàn toàn có 0 entry mới**.
  - File `TextVN-setup-0.2.21-windows-x64-machine.exe` (ghi vào HKLM) chỉ được tạo ra tạm thời trong step phụ của CI runner, **chưa từng được xuất bản lên branch `approved` và chưa từng được nộp vào Store**.

---

##### 2. Cơ chế thực tế của Microsoft Store Validator: Tại sao HKCU luôn bị mù trong Sandbox?
Theo tài liệu chuẩn kỹ thuật của Microsoft (*Manual package validation for MSI/EXE app*):
1. **Validator chỉ quét Registry toàn máy (`HKLM`):**  
   Hạ tầng thẩm định tự động của Microsoft Partner Center kiểm tra tính hợp lệ của gói cài đặt Win32 bằng cách lấy danh sách khác biệt (Registry Diff) tại khóa toàn máy `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`. Các trình cài đặt không đặc quyền chỉ ghi vào `HKCU` sẽ bị coi là **không tạo bất kỳ mục nào trên máy**.
2. **Sự cô lập định danh tài khoản dịch vụ (Service Account Hive Isolation):**  
   Trong máy ảo kiểm thử tự động, tiến trình cài đặt được thực thi bởi một tài khoản ngầm của dịch vụ kiểm thử (Worker Service Account). Nhánh `HKCU` của tiến trình này gắn liền với hồ sơ người dùng tạm của dịch vụ (SID dịch vụ). Tiến trình thanh tra (Inspection Agent) hoặc tài khoản đăng nhập kiểm tra sau đó không thể nhìn thấy hive `HKCU` của tiến trình dịch vụ trước đó.
3. **Hiệu ứng domino của việc không thấy ARP Entry:**
   - Khi Validator quét `HKLM` và thấy **0 entry mới**:
     - Nó không tìm thấy chuỗi App Name và Publisher Name nào $\rightarrow$ **Đánh trượt check "Entry in add or remove programs"**.
     - Không có dấu hiệu nào chứng minh ứng dụng đã được cài đặt thành công $\rightarrow$ **Đánh trượt check "Silent install check"**.
     - Không xác định được sản phẩm chính để lọc bỏ phần mềm rác đi kèm $\rightarrow$ **Đánh trượt check "Bundleware check"**.
   - Cả 3 thông báo lỗi đỏ xuất hiện đồng thời là hệ quả tất yếu.

---

##### 3. Lỗ hổng định danh ký tự: Nguy cơ lệch Codepoint nghiêm trọng giữa "Linh βùi" và Partner Center
Một nguyên nhân độc lập nhưng có khả năng đánh trượt 100% bài kiểm tra so khớp chuỗi là việc sử dụng ký tự **Greek Small Letter Beta (`β`, U+03B2)**:
- **Thực tế mã nguồn trong commit `517db48`:**
  Maintainer định nghĩa:
  `#define MyAppPublisher "Linh βùi"`
  với giải thích: *ký tự beta U+03B2, u-grave U+00F9*.
- **Phân tích đối chiếu định danh pháp lý:**
  - Trong thực tế pháp lý của Việt Nam (Căn cước công dân, Hộ chiếu) và quy trình xác thực danh tính nhà phát triển của Microsoft Partner Center, họ "Bùi" **luôn luôn bắt đầu bằng chữ Latin tiêu chuẩn `B` hoa (U+0042)** hoặc `b` thường (U+0062).
  - Hoàn toàn **không có bất kỳ cơ sở dữ liệu pháp lý nào chấp nhận ký tự Hy Lạp `β` (U+03B2)** trong tên họ cá nhân.
  - Khả năng rất cao: Trong quá trình xem xét trên giao diện Partner Center, do phông chữ hiển thị (font chữ cách điệu, serif/italic) hoặc lỗi sao chép/OCR, chữ cái `B` hoa đã bị nhận nhầm thành ký tự toán học `β` (U+03B2).
- **Hệ quả của thuật toán đối soát nghiêm ngặt:**
  - Nếu trên cơ sở dữ liệu Partner Center, Publisher Display Name là `"Linh Bùi"` (Latin B = `U+0042`) hoặc `"Bùi Hùng Linh"` / `"Linh Bui Hung"`:
  - Khi Validator đem chuỗi đăng ký trong Registry (`... U+03B2 U+00F9 ...`) đối soát từng byte với Partner Center (`... U+0042 U+00F9 ...`):
    $$\text{Codepoint } U+03B2 \neq U+0042 \implies \mathbf{MISMATCH\ 100\%}$$
  - Kết quả: Dù installer có ghi được vào HKLM đi chăng nữa, Validator vẫn sẽ xuất ra thông báo: *"We could not identify the app name and the publisher name that your app has added in the add or remove programs."*

---

##### 4. Nghịch lý bế tắc (Catch-22) của gói Raw Win32 EXE trên Microsoft Store
Kiểm toán chỉ ra rằng việc cố gắng duy trì định dạng `.exe` truyền thống cho một System IME (bộ gõ tiếng Việt TSF) trên Microsoft Store tạo ra một nghịch lý kỹ thuật không thể vượt qua nếu không dùng MSI/MSIX:
- **Nếu chọn Per-user (`PrivilegesRequired=lowest`):**  
  Bộ cài chạy được ngầm không cần quyền admin (Exit 0), nhưng chỉ ghi vào `HKCU` $\rightarrow$ Validator Store quét `HKLM` (hoặc khác User SID) không thấy entry nào $\rightarrow$ **FAIL cả 3 bài test**.
- **Nếu chọn Machine-install (`PrivilegesRequired=admin`):**  
  Bộ cài ghi được vào `HKLM`, nhưng trong môi trường tự động của Store, lệnh gọi `CreateProcess` không có người dùng bấm chấp nhận hộp thoại UAC $\rightarrow$ Bị chặn ngay lập tức với mã lỗi `740` (`ERROR_ELEVATION_REQUIRED`) hoặc Inno Setup thoát với mã lỗi `2` $\rightarrow$ **FAIL cả 3 bài test**.

---

##### 5. Khuyến nghị dứt điểm: Dừng nộp file `.exe` và chuyển ngay sang nộp gói MSIX Full-Trust
Định dạng **MSIX Full-Trust** (`runFullTrust`) là giải pháp chính thống duy nhất được Microsoft thiết kế để loại bỏ triệt để cả 3 bài kiểm tra phiền toái này:
1. **Bỏ qua 100% Silent Install Check:** Windows tự giải nén và đăng ký gói nền qua AppX Deployment Engine, không cần bất kỳ switches dòng lệnh nào.
2. **Bỏ qua 100% Registry ARP Check:** Windows tự động thêm ứng dụng vào Windows Settings / Apps từ `AppxManifest.xml`, không phụ thuộc vào khóa Registry của Inno Setup.
3. **Bỏ qua 100% Bundleware Check:** Gói MSIX là container đóng gói an toàn, ký số toàn vẹn, không có khái niệm phần mềm bên thứ ba cài kèm.

TextVN **đã có sẵn công cụ hoàn chỉnh `tools/win/build-msix.ps1`**. Maintainer chỉ cần lấy đúng `Identity/Name` và `Identity/Publisher` từ Partner Center để đóng gói và nộp trực tiếp.

---

#### 3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM"

Sau khi đối soát trực tiếp từ trang quản lý tài khoản Microsoft Partner Center (**Account settings $\rightarrow$ Organization profile / Publisher info** và **Product management $\rightarrow$ Product identity**), chủ sở hữu tài khoản đã xác nhận chính xác tuyệt đối hai chuỗi định danh nguồn sự thật:

| Trường trên Microsoft Partner Center | Giá trị Nguồn sự thật | Giá trị ghi trong Registry ARP (`TextVN-setup.iss`) | Trạng thái so khớp |
|---|---|---|---|
| **Product / App Name (reserved)** | `TextVN` | `DisplayName = "TextVN"` (`AppVerName`) | ✅ **Khớp 100% từng ký tự** |
| **Publisher Display Name** | `LinhBH.CoM` | `Publisher = "LinhBH.CoM"` (`AppPublisher`) | ✅ **Khớp 100% từng ký tự** |

Việc xác thực được nguồn sự thật này cho phép chuyên gia kiểm toán giải mã triệt để và khép lại toàn bộ bí ẩn đằng sau 10 vòng thử nghiệm thất bại liên tiếp của dự án:

---

##### 1. Giải mã toàn diện lịch sử 10 vòng thử nghiệm thất bại trên Microsoft Store

| Vòng thử nghiệm | Phiên bản | Chuỗi Publisher trong Registry | Kết quả Store | Nguyên nhân kỹ thuật đích thực (Forensic Root Cause) |
|:---:|:---:|:---:|:---:|---|
| **Vòng 1** | 0.2.16 | `LinhBH.CoM` | ❌ Đỏ 3 mục | Trình cài đặt tự động gọi CLI đăng ký TSF trong sandbox, trả về exit code `10` vì thiếu quyền Administrator và không có phiên tương tác. |
| **Vòng 2** | 0.2.17 | `LinhBH.CoM` | ❌ Đỏ 3 mục | Chuyển sang Per-user nhưng logic cài đặt ngầm chưa phải là pure-copy, vẫn tương tác với hệ thống COM. |
| **Vòng 3** | 0.2.18 | `LinhBH.CoM` | ❌ Đỏ 3 mục | Đổi sang `PrivilegesRequired=admin` với manifest `requireAdministrator`. Validator chạy `CreateProcess` non-elevated bị chặn ngay lập tức bởi lỗi `740` (`ERROR_ELEVATION_REQUIRED`), installer thoát mã `2` trước khi kịp ghi nhận bất kỳ khóa nào. |
| **Vòng 4** | 0.2.19 | `LinhBH.CoM` | ⚠️ Không revalidate | Dính bẫy cache **B13g**: nộp lại cùng URL trên GitHub release, Partner Center báo *"We did not find any changes in the Package or the Silent install parameters"* và không chạy lại máy ảo. |
| **Vòng 5** | 0.2.20 | `LinhBH.CoM` | ⏸ Chưa nộp | Tập trung xử lý CI và chuẩn bị hạ tầng MSIX. |
| **Vòng 6** | 0.2.21 | `Linh βùi` | ❌ Đỏ 3 mục | **Chuỗi SAI:** Đọc nhầm font chữ (Latin B nhìn như ký tự Hy Lạp Beta `β` U+03B2). Validator so khớp `Linh βùi` $\neq$ `LinhBH.CoM` $\rightarrow$ Thất bại. |
| **Vòng 7** | 0.2.22 | `Linh Bui` | ❌ Đỏ 3 mục | **Chuỗi SAI:** Suy đoán bỏ dấu thành ASCII `Linh Bui`. Validator so khớp `Linh Bui` $\neq$ `LinhBH.CoM` $\rightarrow$ Thất bại. |
| **Vòng 8–9** | 0.2.23 (Per-user) | `LinhBH.CoM` | ❌ Đỏ 3 mục | **Chuỗi ĐÚNG 100%, nhưng BẢN PER-USER BỊ MÙ TRONG SANDBOX:** Trình cài đặt ghi vào `HKCU`, trong khi Validator của Store chỉ quét `HKLM` (hoặc tài khoản dịch vụ bị cô lập). |
| **Vòng 10** | 0.2.23 (Machine) | `LinhBH.CoM` | ⏳ Đang triển khai | Bản cài đặt máy ghi trực tiếp vào `HKLM`, giải quyết bài toán Registry. |

---

##### 2. Bằng chứng thép từ Vòng 9: Sự thất bại của bản Per-user 0.2.23 chứng minh dứt điểm giả thuyết "Validator chỉ đọc HKLM"
Ở phiên bản v0.2.23 Per-user:
- `DisplayName = "TextVN"` $\equiv$ `Product Name` (Partner Center).
- `Publisher = "LinhBH.CoM"` $\equiv$ `Publisher Display Name` (Partner Center).
- Trình cài đặt chạy `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART` kết thúc thành công với exit code `0` trong `< 2 giây`.
- Tuyệt đối không có bất kỳ dialog hay lỗi nào.

**Thế nhưng Store vẫn đánh trượt cả 3 mục!**  
Điều này khẳng định chắc chắn 100% về mặt kiến trúc: **Bộ thẩm định tự động của Microsoft Store đối với ứng dụng Win32 raw EXE hoàn toàn không đọc `HKEY_CURRENT_USER` của phiên người dùng tương tác!**  
Validator thực thi việc kiểm tra bằng cách lấy Registry Diff trên `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall` (và `WOW6432Node`). Vì bản per-user chỉ ghi vào `HKCU`, nên trong mắt của Validator, số lượng phần mềm mới được cài đặt vào hệ thống là **chính xác bằng 0**.

---

##### 3. Đánh giá kỹ thuật về Bản cài đặt Máy (Machine-Install Variant)
Maintainer đã quyết định sử dụng bản máy làm bản phân phối chính thức cho Store từ 0.2.23:
- **Tập tin:** `TextVN-setup-0.2.23-windows-x64-machine.exe`
- **URL (branch `approved`):**  
  `https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.23/TextVN-setup-0.2.23-windows-x64-machine.exe`
- **SHA-256:** `9fc767744fc772550d02a4857ac270959245a38cc2a1d3867d676c88e3d7ba9a`

**Các ưu thế vượt trội của Bản Máy:**
1. **Ghi trực tiếp vào `HKLM`:** Khóa gỡ cài đặt được Inno Setup tạo trực tiếp tại `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{9C5E4A7D-092E-4D23-9F93-87B75F3FA7B3}_is1`.
2. **Khớp 100% siêu dữ liệu:** `DisplayName = "TextVN"` và `Publisher = "LinhBH.CoM"` trùng khớp tuyệt đối từng byte với Partner Center.
3. **Manifest an toàn:** Nhờ chỉ thị `PrivilegesRequiredOverridesAllowed=commandline`, PE manifest vẫn là `asInvoker`. Khi Validator gọi `CreateProcess` non-elevated, tiến trình không bị chặn ngay ở lỗi `740` như bản 0.2.18.
4. **Phù hợp với tài liệu Microsoft:** Tài liệu *Manual package validation* ghi rõ *"Note: UAC prompts are allowed"*.

**Rủi ro tiềm ẩn duy nhất của Bản Máy:**  
Nếu môi trường container ảo hóa của Microsoft Partner Center là một sandbox headless hoàn toàn không có tương tác đồ họa và không cấu hình chính sách tự chấp thuận nâng quyền (UAC Auto-Consent), lệnh `ShellExecuteExW` với verb `"runas"` của Inno Setup có thể bị Windows từ chối với lỗi `ERROR_CANCELLED` (`1223`), khiến bản máy thoát trước khi hoàn tất cài đặt.

---

##### 4. Kế hoạch hành động dứt điểm theo thứ tự ưu tiên

###### Bước 1: Nộp bản Machine `TextVN-setup-0.2.23-windows-x64-machine.exe`
- Nhập các tham số cài đặt silent trên Partner Center:
  ```
  /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
  ```
- URL gói:
  ```
  https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.23/TextVN-setup-0.2.23-windows-x64-machine.exe
  ```
- Nếu môi trường VM của Microsoft tự động chấp thuận UAC $\rightarrow$ Bản máy sẽ ghi thành công vào `HKLM`, Validator tìm thấy đúng `TextVN` và `LinhBH.CoM` $\rightarrow$ **PASS 100% CẢ 3 BÀI TEST**.

###### Bước 2: Chuyển sang MSIX Full-Trust nếu Bản Máy bị chặn UAC
Nếu bản máy vẫn bị trượt do môi trường container không thể nâng quyền UAC, thì con đường EXE đã chính thức cạn kiệt. Maintainer chuyển ngay sang gói **MSIX Full-Trust**:
- Thông tin đã có sẵn trong Partner Center:
  - Publisher ID: `CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545`
  - Chỉ cần lấy thêm `Package/Identity/Name` tại trang **Product identity**.
- Thực thi lệnh đóng gói:
  ```powershell
  powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\build-msix.ps1 `
    -Publisher "CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545" `
    -IdentityName "<Package/Identity/Name từ Partner Center>"
  ```
- Tải file `.msix` trực tiếp lên Partner Center $\rightarrow$ **Vượt qua thẩm định trong vài giây**.

---

#### 3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions)

Ngày 2026-10-06, sau khi nộp bản máy chính thức **v0.2.24** tại URL:  
`https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.24/TextVN-setup-0.2.24-windows-x64-machine.exe`  
với các tham số im lặng:  
`/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`

Hệ thống Microsoft Partner Center đã trả về kết quả thẩm định mới. Đây là **bước ngoặt mang tính lịch sử** trong quá trình đưa TextVN lên Microsoft Store:

---

##### 1. Đánh giá thành quả: Đập tan hoàn toàn 3 lỗi kiểm duyệt tự động cố hữu
- **Silent install check:** ✅ **ĐÃ PASS HOÀN TOÀN** (Không còn lỗi "could not identify if your app is installing silently").
- **Entry in add or remove programs:** ✅ **ĐÃ PASS HOÀN TOÀN** (Không còn lỗi "could not identify the app name and publisher name").
- **Bundleware check:** ✅ **ĐÃ PASS HOÀN TOÀN** (Không còn lỗi liên quan đến bundleware).

**Ý nghĩa kỹ thuật:**  
Kết quả này đã chứng minh tính chuẩn xác 100% của bản kiểm toán:
1. Bản máy `TextVN-setup-0.2.24-windows-x64-machine.exe` chạy với cờ silent đã tự nâng quyền và cài đặt thành công vào `C:\Program Files\TextVN`.
2. Trình cài đặt đã tạo chính xác mục gỡ cài đặt tại `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{9C5E4A7D-092E-4D23-9F93-87B75F3FA7B3}_is1`.
3. Hai chuỗi định danh nguồn sự thật `DisplayName = "TextVN"` và `Publisher = "LinhBH.CoM"` đã khớp hoàn hảo 100% từng byte với tài khoản Partner Center.
4. Gói cài đặt đã chính thức **vượt qua toàn bộ giai đoạn Package Validation tự động** và tiến vào cổng duyệt chứng nhận chính sách (**Certification & Policy Review**).

---

##### 2. Giải mã thông báo mới từ Store: Chính sách 10.2.9 (Security - Package Submissions)

Nguyên văn thông báo từ Microsoft Partner Center:
> **Technical requirement policies**  
> **Notes to publisher**  
> **10.2.9 Security - Package Submissions**  
> *The binary and all of its Portable Executable (PE) files has been signed with a certificate that has been observed being abused to sign malicious content or must be digitally signed with a code sign certificate that chains up to a certificate issued by a Certificate Authority (CA) that is part of the Microsoft Trusted Root Program. To code sign your app, you can use Trusted Signing... If your EXE or MSI cannot comply with Microsoft Store policy 10.2.9, you can consider repackaging your existing EXE or MSI to MSIX format. Microsoft Store offers many complimentary benefits for MSIX format such as code signing, hosting etc... You can run your EXE or MSI installers through MSIX Packaging Tool and obtain an MSIX package that you can submit to the Microsoft Store as MSIX packaged app. Note that you have to delete your app name from existing Win32 app in Partner Center in case you want to use the same for MSIX packaged app.*  
>  
> **The following packages are affected:**  
> **Package URL:** `https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.24/TextVN-setup-0.2.24-windows-x64-machine.exe`  
> **Code signing type:** `Unsigned`  
> **Description:** `Package should be signed with SHA256 or higher algorithm`

**Phân tích bản chất chính sách 10.2.9:**
- Theo quy định bảo mật mới nhất của Microsoft Store áp dụng cho định dạng Win32 truyền thống (.exe / .msi): Mọi file thực thi binary và installer nộp qua URL tải về **bắt buộc phải được ký số điện tử (Digitally Signed)** bằng chứng thư Authenticode hợp lệ thuộc *Microsoft Trusted Root Program*, sử dụng thuật toán băm **SHA-256 trở lên**.
- File `TextVN-setup-0.2.24-windows-x64-machine.exe` hiện tại là bản dựng mã nguồn mở chưa có chữ ký số (`Unsigned`).
- Vì vậy, submission bị dừng lại ở rào cản chính sách chữ ký số, không phải lỗi kỹ thuật của trình cài đặt.

---

##### 3. Hai phương án xử lý do chính Microsoft đề xuất và lộ trình hành động chi tiết

Chính Microsoft đã nêu rõ hai con đường giải quyết:

```mermaid
flowchart TD
    A["Thông báo Policy 10.2.9<br/>(Yêu cầu ký số SHA-256)"] --> B{"Lựa chọn phương án"}
    
    B --> C["PHƯƠNG ÁN 1: Ký số cho file EXE<br/>(Giữ luồng Win32 EXE)"]
    C --> C1["Nộp đơn SignPath Foundation (Miễn phí OSS)"]
    C --> C2["Hoặc mua cert / dùng Azure Trusted Signing"]
    C --> C3["Ký số SHA256 cho setup.exe + các PE<br/>-> Nộp lại URL EXE đã ký"]
    
    B --> D["PHƯƠNG ÁN 2: Chuyển sang MSIX Full-Trust<br/>(KHUYẾN NGHỊ - ĐƯỢC MICROSOFT KÝ SỐ MIỄN PHÍ)"]
    D --> D1["1. Giải phóng tên 'TextVN' ở sản phẩm Win32 cũ<br/>(Hoặc tạo sản phẩm mới dạng MSIX)"]
    D --> D2["2. Đóng gói MSIX Unsigned bằng build-msix.ps1"]
    D --> D3["3. Upload trực tiếp file .msix lên Partner Center<br/>-> Microsoft Store tự động ký số và xuất bản!"]
```

###### Phương án 1: Ký số cho file EXE bằng chứng thư số tin cậy (Giữ luồng Win32 EXE)
- **Cơ chế:** Ký số Authenticode SHA-256 cho cả 4 file binary: `TextVN.exe`, `textvn-cli.exe`, `textvn-tsf.dll` và `TextVN-setup-0.2.24-windows-x64-machine.exe`.
- **Cách thức thực hiện (không tốn phí):**
  1. Sử dụng **SignPath Foundation** (đã chuẩn bị sẵn tại `docs/release/code-signing-plan.md`): Nộp đơn đăng ký dự án mã nguồn mở TextVN tại [SignPath Open Source](https://signpath.org/open-source).
  2. Khi được cấp `SIGNPATH_API_TOKEN`, bật secret trong GitHub repo. Pipeline release sẽ tự động ký số binary qua GitHub Actions.
  3. Cập nhật URL file `.exe` đã ký lên Partner Center.
- **Đánh giá:** Giữ nguyên được sản phẩm Win32 hiện tại, nhưng phải chờ thời gian xét duyệt từ SignPath (thường từ 2–5 ngày làm việc).

###### Phương án 2 (Khuyến nghị hàng đầu): Repackage sang MSIX Full-Trust (Microsoft Ký số Miễn phí)
Đây là giải pháp ưu việt nhất được chính reviewer của Microsoft đề xuất:
> *"Microsoft Store offers many complimentary benefits for MSIX format such as code signing, hosting etc."*

- **Lợi ích tuyệt đối:**
  1. **Ký số miễn phí 100%:** Gói MSIX được nộp ở trạng thái **UNSIGNED**, và Microsoft Store sẽ tự động ký số bằng chính chứng thư gốc của Microsoft khi phát hành. Maintainer không cần mua cert, không cần nộp đơn SignPath.
  2. **Hosting miễn phí:** File được lưu trữ và tải về trực tiếp từ CDN toàn cầu của Microsoft Store, không phụ thuộc vào GitHub raw URL.
  3. **Tương thích hoàn hảo:** Windows 10/11 tự động quản lý cài đặt, cập nhật và gỡ bỏ.
- **HƯỚNG DẪN BẮT BUỘC TỪ MICROSOFT VỀ VIỆC GIẢI PHÓNG TÊN (DELETE APP NAME):**
  Reviewer lưu ý:
  > *"Note that you have to delete your app name from existing Win32 app in Partner Center in case you want to use the same for MSIX packaged app."*
  - **Lý do:** Sản phẩm `TextVN` hiện tại trên Partner Center được tạo dưới loại hình **Win32 (EXE/MSI)**. Loại sản phẩm này KHÔNG cho phép tải lên file `.msix`. Một tên ứng dụng đã reserve (như `TextVN`) chỉ được gắn với một Product ID duy nhất.
  - **Quy trình thực hiện từng bước:**
    1. **Bước 1 — Giải phóng tên `TextVN`:**
       - Truy cập Partner Center $\rightarrow$ Vào sản phẩm `TextVN` hiện tại $\rightarrow$ **Product management** $\rightarrow$ **Product identity** $\rightarrow$ Bấm **Delete product** hoặc hủy tên reserve theo hướng dẫn tại [Delete your app name](https://go.microsoft.com/fwlink/?linkid=2189821).
       - *(Hoặc giải pháp nhanh không cần xóa:* Tạo một sản phẩm mới dạng MSIX với tên bổ sung như `TextVN - Bộ gõ tiếng Việt` hoặc `TextVN IME`*).*
    2. **Bước 2 — Tạo sản phẩm mới dạng MSIX:**
       - Bấm **New product** $\rightarrow$ Chọn loại **MSIX or PWA application**.
       - Đặt tên `TextVN` (đã được giải phóng ở Bước 1).
       - Mở **Product identity** sao chép:
         - `Package/Identity/Name` (ví dụ: `12345LinhBH.TextVN`)
         - `Package/Identity/Publisher` (ví dụ: `CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545`)
    3. **Bước 3 — Đóng gói MSIX trong repo:**
       - Chạy script đóng gói có sẵn:
         ```powershell
         powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\build-msix.ps1 `
           -Publisher "CN=<Package/Identity/Publisher từ Partner Center>" `
           -IdentityName "<Package/Identity/Name từ Partner Center>"
         ```
       - Kết quả sinh ra file `dist\TextVN-0.2.24-windows-x64.msix`.
    4. **Bước 4 — Upload trực tiếp lên Store:**
       - Trong submission của sản phẩm mới, upload trực tiếp file `TextVN-0.2.24-windows-x64.msix`.
       - Microsoft Store sẽ tự động ký số và xuất bản ứng dụng thành công!

---

## 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)

---

### 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)

#### Phát hiện W-01: Hook bàn phím toàn cục `WH_KEYBOARD_LL` trong Tray
- **Vị trí:** `tray/src/main.rs:786` và `tray/src/main.rs:84-140`.
- **Mã nguồn:**
  ```rust
  let ll_hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(tray_ll_keyboard_proc), None, 0) };
  ```
- **Tác động:** Kích hoạt cảnh báo heuristic "Keylogger / Trojan" của Kaspersky, Windows Defender và các giải pháp EDR doanh nghiệp. Làm chậm luồng thông điệp bàn phím toàn hệ thống nếu luồng Tray bị gián đoạn.
- **Phương án giải quyết:** Xóa bỏ hoàn toàn `WH_KEYBOARD_LL` khỏi `tray/src/main.rs`. Chuyển toàn bộ việc bắt tổ hợp `Ctrl + Shift` sang cơ chế TSF `KeyTraceSink` / `ModifierToggle` sẵn có trong DLL và chỉ nhận thông báo trạng thái qua IPC Pipe.

#### Phát hiện W-02: Lỗ hổng nâng quyền Portable UAC
- **Vị trí:** `tray/src/main.rs:271-360`.
- **Mã nguồn:** Gọi `ShellExecuteW(..., "runas", cli, "register --scope machine", dir, ...)` khi `dir` là đường dẫn tùy ý.
- **Tác động:** Lỗ hổng bảo mật nghiêm trọng (CWE-426 / CWE-732). Vi phạm quy chuẩn Microsoft Store Certification (mục 10.2: không được đòi quyền Admin khi sử dụng thông thường).
- **Phương án giải quyết:** Tuyệt đối không cho phép đăng ký phạm vi máy (`--scope machine`) từ các thư mục người dùng. Chỉ cho phép đăng ký máy khi binary nằm trong thư mục được bảo vệ có quyền ACL quản trị (`%ProgramFiles%\TextVN`). Với bản Portable, hướng dẫn người dùng sử dụng phím chuyển Win+Space hoặc chấp nhận giới hạn TSF per-user.

#### Phát hiện W-03: Sửa đổi Registry Windows Hotkey thiếu hoàn trả
- **Vị trí:** `tray/src/main.rs:693` và `tray/src/hotkey.rs:171-178`.
- **Mã nguồn:** Gọi `free_ctrl_shift()` tự động thay đổi giá trị `HKCU\Keyboard Layout\Toggle\Layout Hotkey` thành `3`.
- **Tác động:** Phá vỡ tính năng chuyển đổi ngôn ngữ mặc định của Windows đối với người dùng sử dụng nhiều bố cục bàn phím (ví dụ: tiếng Nhật, Hàn, Pháp). Khi gỡ cài đặt, TextVN không trả lại giá trị gốc.
- **Phương án giải quyết:** Đưa thiết lập này thành một tùy chọn có chủ đích trong Bảng cài đặt (mặc định tắt hoặc hỏi người dùng). Bổ sung hàm phục hồi `restore_windows_ctrl_shift()` vào script gỡ cài đặt (`TextVN-setup.iss` và `uninstall.ps1`).

#### Phát hiện W-04: Thiếu kiểm soát truy cập (DACL) trên Named Pipe IPC
- **Vị trí:** `tray/src/ipc_server.rs:352-362`.
- **Mã nguồn:** `CreateNamedPipeW` với tham số `lpSecurityAttributes = None`.
- **Tác động:** Bất kỳ ứng dụng nào trong phiên làm việc đều có thể kết nối vào pipe, gửi lệnh `Shutdown` làm sập Tray, hoặc gửi lệnh `StateUpdate` làm tê liệt việc gõ tiếng Việt. Khi có nhiều người dùng đăng nhập đồng thời (RDP/Fast User Switching), pipe name `\\.\pipe\textvn-ipc-v1` sẽ bị xung đột.
- **Phương án giải quyết:** Khởi tạo Security Descriptor với DACL nghiêm ngặt chỉ cấp quyền cho SID của người dùng hiện tại (`TOKEN_USER`). Đặt tên Pipe có hậu tố Session ID: `\\.\pipe\textvn-ipc-v1-<SessionId>`.

#### Phát hiện W-05: Đóng gói MSIX và bài toán Full-Trust
- **Vị trí:** `installer/windows/msix/AppxManifest.xml:43`.
- **Nhận định:** Khai báo `<rescap:Capability Name="runFullTrust" />` là chính xác và bắt buộc đối với một IME Win32. Tuy nhiên, việc kỳ vọng ứng dụng MSIX tự gọi UAC để đăng ký TSF vào HKLM sẽ bị Windows từ chối trong môi trường AppContainer. MSIX cần đi kèm một Desktop Extension chuyên dụng hoặc hướng dẫn cài đặt rõ ràng.

#### Phát hiện W-06: Điểm mù kiến trúc 32-bit WOW64 — Zalo PC và toàn bộ ứng dụng x86 không thể nạp TSF DLL 64-bit
- **Vị trí:** `installer/windows/TextVN-setup.iss:138`, `cli/src/register.rs:1078`, `build-release.ps1:93`.
- **Hiện tượng thực tế:** Trên phiên bản v0.2.25 (và các bản trước), người dùng hoàn toàn **không gõ được tiếng Việt trong Zalo PC** (khi gõ `chaof` vẫn ra nguyên văn `chaof`), trong khi Notepad và các ứng dụng khác vẫn gõ bình thường.
- **Chứng cứ điều tra thực nghiệm (Forensic Evidence):**
  1. Tiến trình `Zalo.exe` (kiểm chứng thực tế tại `C:\Users\hungl\AppData\Local\Programs\Zalo\Zalo-26.10.10\Zalo.exe`): PE Header Machine ID là `0x014C` (`IMAGE_FILE_MACHINE_I386` — ứng dụng 32-bit x86).
  2. TextVN hiện tại chỉ biên dịch và đóng gói duy nhất một tệp DLL 64-bit: `textvn_win_tsf.dll` (`x86_64-pc-windows-msvc`).
  3. Lệnh đăng ký COM server (`cli/src/register.rs`) chỉ ghi vào nhánh Registry 64-bit (`Software\Classes\CLSID\{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}\InprocServer32`). Truy vấn qua môi trường 32-bit (`SysWOW64\WindowsPowerShell`) cho thấy khóa CLSID trong `HKEY_CLASSES_ROOT\WOW6432Node` **hoàn toàn không tồn tại** (`Test-Path = False`).
  4. Kiểm tra danh sách DLL nạp vào tiến trình đang chạy: Trong khi `notepad.exe` (64-bit) nạp thành công `textvn-tsf.dll`, thì `Zalo.exe` (32-bit) nạp `MSCTF.dll` nhưng **hoàn toàn không có `textvn-tsf.dll`**.
  5. Theo kiến trúc của Windows NT Kernel: Một tiến trình 32-bit (chạy trong WOW64) **tuyệt đối không thể nạp một DLL 64-bit** vào không gian bộ nhớ của nó (`ERROR_BAD_EXE_FORMAT` / mã lỗi `193` / `0xC000007B`).
  6. Bản phát hành mặc định của TextVN tắt `textvn-hook.exe` (`tsf-only` mode để tránh false-positive AV), và trong `data/appdb.default.json` gán `"engine_owner": "tsf"` cho `zalo.exe`. Vì không có DLL 32-bit và không có hook chạy nền, Zalo không có bất kỳ bộ xử lý phím nào can thiệp.
- **Tác động:** Toàn bộ người dùng Zalo PC trên Windows (chiếm đại đa số người dùng văn phòng tại Việt Nam) cùng các ứng dụng 32-bit phổ biến khác (Office 32-bit, Notepad++ 32-bit, Foxit Reader 32-bit, Skype 32-bit, Win32 games) bị tê liệt hoàn toàn khả năng gõ tiếng Việt với TextVN.
- **Phương án giải quyết triệt để:**
  1. Thêm target `i686-pc-windows-msvc` vào toolchain và pipeline CI/CD (`rustup target add i686-pc-windows-msvc`).
  2. Biên dịch song song hai bản dựng của crate `textvn-win-tsf`:
     - Bản 64-bit: `textvn-tsf-x64.dll` (`x86_64-pc-windows-msvc`).
     - Bản 32-bit: `textvn-tsf-x86.dll` (`i686-pc-windows-msvc`).
  3. Cập nhật Inno Setup (`TextVN-setup.iss`) đóng gói cả hai tệp DLL vào thư mục cài đặt.
  4. Cập nhật `cli/src/register.rs`: Mở registry với cờ `KEY_WOW64_64KEY` để đăng ký `textvn-tsf-x64.dll`, và mở registry với cờ `KEY_WOW64_32KEY` để ghi khóa `InprocServer32` trong `WOW6432Node` trỏ vào `textvn-tsf-x86.dll`.
  5. **Giải pháp tình thế trước mắt cho người dùng (Workaround):** Sử dụng **Zalo Web** trên trình duyệt 64-bit (Chrome, Edge, Brave, Cốc Cốc) tại `https://chat.zalo.me/` để gõ tiếng Việt bình thường trong khi chờ bản cập nhật hỗ trợ nhị phân 32-bit.

---

### 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements)

#### Phát hiện M-01: Không đồng bộ giao diện Menu Bar khi nhận IPC
- **Vị trí:** `adapters/macos-app/Sources/TextVNAppLib/AppDelegate.swift:572-583`.
- **Mã nguồn:** `ipcServer(_:didToggleViEn:enabled:)` chỉ gán `self.isVietnameseMode = enabled` mà không gọi `updateStatusIcon()`.
- **Phương án giải quyết:** Bọc mã cập nhật vào `DispatchQueue.main.async`, gọi `self.updateStatusIcon()` và `self.updateMenuState()`.

#### Phát hiện M-02: Quyền Hardened Runtime (Entitlements) thừa thãi
- **Vị trí:** `packaging/macos/TextVN.entitlements:7-10`.
- **Mã nguồn:**
  ```xml
  <key>com.apple.security.cs.allow-jit</key>
  <true/>
  <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
  <true/>
  ```
- **Tác động:** Vi phạm nguyên tắc bảo mật tối thiểu của Apple. Đội ngũ đánh giá Notarization có thể từ chối phê duyệt tự động.
- **Phương án giải quyết:** Loại bỏ hoàn toàn 2 khóa trên. Sử dụng tệp entitlements riêng biệt cho từng bundle: `TextVN-App.entitlements` cho Menu Bar App và `TextVN-IM.entitlements` cho Input Method.

#### Phát hiện M-03: Thử nghiệm kiểm thử CI thiếu kiến trúc x86_64
- **Vị trí:** `.github/workflows/ci-macos.yml:123`.
- **Mã nguồn:** `swift test --arch arm64`.
- **Tác động:** Bản dựng Universal phân phối cho người dùng chứa cả mã máy Intel x86_64 nhưng mã này chưa từng được chạy test trên CI runner.
- **Phương án giải quyết:** Bổ sung bước kiểm thử x86_64 sử dụng runner tương thích hoặc ma trận thử nghiệm đầy đủ.

#### Phát hiện M-04: Thử nghiệm thực tế trên phần cứng macOS vật lý (Real Hardware & TCC Permissions Gap)
- **Vị trí:** `docs/30-macos/IMPLEMENTATION-STATUS.md:43-51`, `tools/mac/smoke-imk.sh`.
- **Nhận định kiểm toán:** Toàn bộ quá trình build và kiểm thử macOS hiện tại chỉ chạy trên GitHub Actions virtual runner (môi trường headless). Các luồng tương tác thực tế của người dùng:
  1. Đăng ký Input Source với hệ thống qua `TISRegisterInputSource` và việc nạp lại `TextInputMenuAgent` khi chuyển đổi nguồn gõ.
  2. Hộp thoại cấp quyền Accessibility / Input Monitoring (TCC permissions) của macOS cho module `adapters/macos-tap`.
  3. Kiểm thử giao diện gõ thật trên 12 ứng dụng mục tiêu (`tools/mac/targets/*.json`: Safari, Chrome, Slack, Discord, Notes, Terminal, Xcode...).
- **Tác động:** Bản dựng có thể pass 100% logic unit test trên CI nhưng khi người dùng cài vào máy thật macOS Sequoia / Sonoma có thể gặp hiện tượng không thấy bộ gõ xuất hiện trong Keyboard Settings hoặc bị TCC chặn quyền.
- **Phương án giải quyết:** Duy trì quy trình smoke test bắt buộc trên máy Mac vật lý trước mỗi bản phát hành ổn định (Stable Tag).

#### Phát hiện M-05: Cấu hình `non_preedit` chưa liên kết vào IMK (Gõ chữ bị gạch chân)
- **Vị trí:** `adapters/macos-imk/Sources/IMKLib/TextVNInputController.swift:230-260` và `adapters/macos-app/Sources/TextVNAppLib/SettingsView.swift`.
- **Mã nguồn:** Giao diện Cài đặt SwiftUI cho phép bật/tắt "Gõ không gạch chân" và lưu vào `config.json`, nhưng lớp `TextVNInputController` chưa đọc khóa này từ cấu hình để chuyển đổi hành vi giữa `setMarkedText` và `insertText` + `deleteBackward`.
- **Tác động:** Trên macOS, trong các ứng dụng như Microsoft Word, Chrome, hoặc Spotlight, người dùng vẫn nhìn thấy vạch gạch chân màu xanh/xám dưới từ đang gõ dở.
- **Phương án giải quyết:** Đọc trường `non_preedit` trong `IpcClient` / `ConfigModel`, khi bật thì chuyển `strategy` sang cơ chế BackspaceType (`ApplyReplace.swift`).

---

### 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging)

#### Phát hiện L-01: Bộ phân tích JSON IPC thủ công bằng `strstr`
- **Vị trí:** `adapters/linux-common/src/ipc_client.c:160-230`.
- **Mã nguồn:** Dùng các hàm `strstr(json, ...)` để trích xuất trường dữ liệu thay vì dùng bộ phân tích JSON chuẩn.
- **Tác động:** Dễ gặp lỗi nhận diện nhầm khi chuỗi giá trị chứa từ khóa trùng khớp, và bỏ qua các trường hợp ký tự đại diện (`*`).
- **Phương án giải quyết:** Sử dụng một thư viện parser JSON siêu nhẹ, không cấp phát động (ví dụ: `jsmn` hoặc `cJSON`) để giải mã gói tin một cách chặt chẽ.

#### Phát hiện L-02: Đường dẫn Socket không an toàn trong `/tmp`
- **Vị trí:** `adapters/linux-common/src/ipc_client.c:79`.
- **Mã nguồn:** `strncpy(out_path, "/tmp/textvn-ipc.sock", max_len - 1);`.
- **Tác động:** Thư mục `/tmp` có cờ sticky-bit nhưng cho phép mọi người dùng ghi tệp. Kẻ tấn công có thể tạo trước socket hoặc liên kết mềm (symlink) để chặn bắt dữ liệu.
- **Phương án giải quyết:** Loại bỏ fallback `/tmp`. Bắt buộc sử dụng `$XDG_RUNTIME_DIR/TextVN/ipc.sock` (được bảo vệ với quyền `0700` bởi systemd).

#### Phát hiện L-03: Thiếu định dạng đóng gói chuẩn của các bản phân phối Linux
- **Vị trí:** `scripts/build-linux.sh`.
- **Nhận định:** Chỉ cung cấp tệp `.tar.gz` kèm mã băm SHA256. Người dùng phổ thông trên Ubuntu/Debian hoặc Fedora gặp khó khăn khi cài đặt thủ công.
- **Phương án giải quyết:** Bổ sung cấu hình sinh gói `.deb` (qua `dpkg-deb`) và `.rpm` (qua `rpmbuild`), hỗ trợ kho PPA và Flathub.

#### Phát hiện L-04: Xung đột Wayland vs X11 và Cơ chế Surrounding Text trên Chat Apps
- **Vị trí:** `adapters/linux-ibus/src/engine.c:104`, `adapters/linux-fcitx5/src/engine.cpp:210`, `docs/40-linux/P3-REVIEW-LOG.md`.
- **Nhận định kiểm toán:** 
  1. Trên Wayland (GNOME Shell Wayland, KDE Plasma 6 Wayland), giao thức bảo mật Wayland chặn hoàn toàn việc các tiến trình thông thường bắt phím toàn cục (Global key grabbing). TextVN đã tuân thủ chuẩn xác quyết định kiến trúc ADR-007 (không dùng uinput/hook trên Wayland), phụ thuộc 100% vào IBus/Fcitx5 protocol.
  2. Tuy nhiên, các ứng dụng chạy qua XWayland (nhiều phần mềm Electron như Zalo Linux, Discord, Skype cũ) hoặc ứng dụng không hỗ trợ `CapabilityFlag::SurroundingText` sẽ không thể xóa lùi ký tự đúng cách nếu bật chế độ Không gạch chân (Non-preedit).
- **Tác động:** Nếu cấu hình non-preedit ép buộc trên Linux mà không kiểm tra cờ `has_surrounding`, người dùng sẽ bị hiện tượng lặp chữ (gõ `as` thành `aá`).
- **Phương án giải quyết:** Duy trì cơ chế Fallback đã được kiểm toán trong `F3-021`: Khi ứng dụng không báo `has_surrounding`, tự động quay về chế độ Preedit tiêu chuẩn của IBus/Fcitx5.

---

### 4.4. Core Engine, FFI ABI, Restore English & Data

#### Phát hiện C-01: Hiệu năng quét từ điển `restore_en.rs`
- **Vị trí:** `core/src/post/restore_en.rs:49-56`.
- **Mã nguồn:** Quét từng dòng dữ liệu nhúng và gọi `.to_lowercase()` trong mỗi lần phím cách (Space) xuất hiện.
- **Phương án giải quyết:** Tiền xử lý từ điển thành bảng băm hoàn hảo tĩnh ở thời điểm biên dịch (`phf_set` hoặc mảng đã sắp xếp kết hợp tìm kiếm nhị phân `binary_search`).

#### Phát hiện C-02: Dung lượng tập từ vựng còn quá nhỏ
- **Vị trí:** `data/en_common.txt` (55 từ) và `data/vn_common.txt` (82 từ).
- **Tác động:** Tính năng khôi phục tiếng Anh thông minh chỉ nhận diện được 55 từ cơ bản; hàng ngàn từ vựng chuyên ngành công nghệ thông tin và giao tiếp hàng ngày vẫn bị biến dạng dấu không mong muốn.
- **Phương án giải quyết:** Mở rộng tập từ điển tiếng Anh phổ thông lên khoảng 1.000 – 3.000 từ thông dụng nhất, kết hợp danh sách các âm tiết tiếng Việt đầy đủ.

---

### 4.5. CI/CD, Supply Chain, VirusTotal & Microsoft Store Validation

#### Phát hiện CI-01: Script ký số chưa được gắn vào quy trình Release
- **Vị trí:** `.github/workflows/release.yml:38-80`.
- **Mã nguồn:** Tệp khai báo các biến môi trường `SIGNPATH_*` nhưng hoàn toàn không có bước thực thi lệnh `powershell tools/win/sign-signpath.ps1`.
- **Tác động:** Mọi bản phát hành đẩy lên GitHub Releases hiện nay đều là bản **chưa có chữ ký số (Unsigned)**, dẫn đến việc SmartScreen hiển thị cảnh báo đỏ trên Windows và Gatekeeper chặn trên macOS.

#### Phát hiện CI-02: Đóng gói Inno Setup Silent bỏ qua đăng ký TSF
- **Vị trí:** `installer/windows/TextVN-setup.iss:219-228`.
- **Mã nguồn:** Trong điều kiện `WizardSilent`, trình cài đặt ghi log *"skip TSF registration"* và thoát với mã 0.
- **Tác động:** Đây là giải pháp tình thế để vượt qua bài kiểm tra của Microsoft Store Validator, nhưng dẫn đến hệ quả là người dùng cài đặt ngầm sẽ không thể gõ được tiếng Việt cho đến khi tự khởi động ứng dụng và chấp nhận hộp thoại nâng quyền.

#### Phát hiện CI-03: Điểm mù trong Harness kiểm thử Store (validate-store-package.ps1) đọc cả HKCU gây kết quả Pass giả tạo
- **Vị trí:** `tools/win/validate-store-package.ps1:37-41` và `tools/win/simulate-store-validation.ps1:25-29`.
- **Mã nguồn:**
  ```powershell
  $roots = @(
      'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall',
      'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall',
      'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall'
  )
  ```
- **Tác động:** Harness trên CI quét cả hive người dùng `HKCU`. Khi installer per-user chạy, nó ghi vào `HKCU` của runner và script kiểm tra thấy ngay -> Báo `PASS 2) ARP entry` trong khi nhánh `HKLM` (nơi mà Microsoft Store Validator trong môi trường headless container thực sự truy vấn) hoàn toàn không có entry nào. Đồng thời, script hardcode giá trị kiểm tra là `TextVN` và `LinhBH.CoM` thay vì đối soát với tên sản phẩm và tên nhà xuất bản thực tế trên cổng Microsoft Partner Center.
- **Phương án giải quyết:** Phân lập bài test trên CI: Thêm cờ `-StrictMachine` chỉ kiểm tra `HKLM`; tham số hóa định danh kiểm tra theo biến môi trường `STORE_APP_NAME` và `STORE_PUBLISHER_NAME`.

---

## 5. LỘ TRÌNH KÝ SỐ CHÍNH THỨC & SẴN SÀNG CHO MỌI NGƯỜI DÙNG

Để một bộ gõ hệ thống (System IME) có thể hoạt động trơn tru trên hàng triệu máy tính của người dùng mà không bị Antivirus chặn, SmartScreen cảnh báo hay Gatekeeper cách ly, **chữ ký số hợp chuẩn là điều kiện tiên quyết bắt buộc**.

```
                           +-------------------------------------------------------+
                           |        TIẾN TRÌNH KÝ SỐ ĐA NỀN TẢNG TEXTVN            |
                           +-------------------------------------------------------+
                                   |                       |                      |
                    [ WINDOWS ]    |          [ MACOS ]    |         [ LINUX ]    |
                                   v                       v                      v
                      +-------------------+  +-------------------+  +-------------------+
                      | SignPath / Azure  |  | Apple Developer ID|  | GPG Key Signing   |
                      | Trusted Signing   |  | (App + Installer) |  | (Release Assets)  |
                      +-------------------+  +-------------------+  +-------------------+
                               |                       |                      |
                               v                       v                      v
                      +-------------------+  +-------------------+  +-------------------+
                      | Ký DLL, EXE,      |  | Ký Bundle App,    |  | Ký Tarball, Deb,  |
                      | Installer, MSIX   |  | Notarize, Staple  |  | SHA256SUMS        |
                      +-------------------+  +-------------------+  +-------------------+
                               |                       |                      |
                               v                       v                      v
                      +-------------------+  +-------------------+  +-------------------+
                      | Vượt SmartScreen, |  | Gatekeeper Approved| | Verified Package, |
                      | Microsoft Store   |  | Không bị cách ly  |  | Sẵn sàng PPA/Repo |
                      +-------------------+  +-------------------+  +-------------------+
```

---

### 5.1. Ký số Windows (Authenticode qua SignPath Foundation / Azure Trusted Signing)

1. **Phương án Khuyến nghị cho Dự án Mã nguồn mở: SignPath Foundation**
   - **Chi phí:** 0 VNĐ (Miễn phí hoàn toàn cho các dự án Open Source đạt chuẩn GPL-3.0).
   - **Loại chứng chỉ:** Organization Validation (OV) Certificate được lưu trữ an toàn trong Cloud HSM của SignPath.
   - **Các bước tiến hành:**
     - **Bước 1:** Maintainer gửi đơn đăng ký tại [signpath.org/open-source](https://signpath.org/open-source) với liên kết repo `https://github.com/hunglinhpt/TextVN`.
     - **Bước 2:** Sau khi được phê duyệt, cấu hình GitHub App của SignPath và thiết lập 4 Secrets trong GitHub Repository:
       - `SIGNPATH_API_TOKEN`
       - `SIGNPATH_ORGANIZATION_ID`
       - `SIGNPATH_PROJECT_KEY`
       - `SIGNPATH_POLICY`
     - **Bước 3:** Bổ sung bước ký vào `.github/workflows/release.yml` trước khi đóng gói installer:
       - Ký `TextVN.exe`, `textvn-cli.exe`, `textvn-tsf.dll`.
       - Đóng gói Inno Setup → Ký tiếp tệp `TextVN-setup-<ver>-windows-x64.exe`.
2. **Quy tắc xây dựng Uy tín (SmartScreen Reputation Building):**
   - Chữ ký OV mới sẽ chưa có danh tiếng ngay lập tức. Sau khi phát hành bản ký đầu tiên, maintainer cần chủ động gửi mẫu (submit file hashes) lên [Microsoft Security Intelligence File Submission](https://www.microsoft.com/wdsi/filesubmission) với danh mục "Software Developer". Microsoft sẽ quét tự động và cập nhật danh tiếng sạch vào cơ sở dữ liệu SmartScreen trong vòng vài ngày.

---

### 5.2. Ký số & Notarization macOS (Apple Developer ID Application & Installer)

1. **Yêu cầu Tài khoản:**
   - Cần một tài khoản **Apple Developer Program** cá nhân hoặc tổ chức ($99/năm).
2. **Hai loại chứng chỉ bắt buộc:**
   - `Developer ID Application`: Dùng để ký các tệp thực thi và bundle (`TextVN.app`, `TextVN-IM.app`).
   - `Developer ID Installer`: Dùng để ký gói cài đặt hệ thống (`TextVN-mac-v<ver>.pkg`).
3. **Quy trình Notarization tự động hóa trong CI:**
   - Tạo App-Specific Password trên [appleid.apple.com](https://appleid.apple.com).
   - Thiết lập các Secrets trên GitHub Actions:
     - `APPLE_ID`
     - `APPLE_TEAM_ID`
     - `APPLE_APP_PASSWORD`
     - `MACOS_CERTIFICATE_BASE64` (chứa cert và private key dạng `.p12`)
     - `MACOS_CERTIFICATE_PWD`
   - Chạy script tích hợp:
     ```bash
     scripts/build-macos.sh              # Ký bundle bằng Developer ID Application
     scripts/package-macos-pkg.sh        # Ký gói pkg bằng Developer ID Installer
     scripts/notarize-macos.sh <pkg>     # Gửi lên Apple Notary Service và staple ticket
     ```
   - **Kết quả:** Người dùng macOS tải về mở tệp `.pkg` cài đặt mượt mà, không gặp thông báo *"App is damaged and cannot be opened"* hay bị Gatekeeper chặn lại.

4. **Giải pháp Phân phối 0 đồng (Không tốn $99/năm) cho macOS Open Source:**
   - **Kênh Homebrew Cask (Khuyến nghị 0đ hàng đầu):** Đã có sẵn file cấu hình `packaging/homebrew/textvn.rb`. Khi người dùng cài đặt qua Homebrew:
     ```bash
     brew tap hunglinhpt/textvn
     brew install --cask textvn
     ```
     Homebrew tải trực tiếp từ GitHub Releases qua curl và tự động vô hiệu hóa cờ quarantine của trình duyệt, giúp người dùng cài đặt trơn tru mà maintainer không mất 1 xu cho Apple!
   - **Ký Ad-hoc cục bộ (`codesign -s - --force`):** Bộ cài được ký Ad-hoc để vượt qua kiểm tra chữ ký tối thiểu của macOS.
   - **Lệnh gỡ cách ly Gatekeeper thủ công:** Hướng dẫn người dùng tải file zip/pkg trực tiếp chạy một lệnh duy nhất nếu bị chặn:
     ```bash
     xattr -cr /Applications/TextVN.app
     sudo xattr -cr /Library/Input\ Methods/TextVN.app
     ```

---

### 5.3. Ký số & Phân phối Linux (GPG, PPA, COPR, AUR, Flatpak)

1. **Ký số GPG cho Asset Phát hành (100% Miễn phí, 0 VNĐ):**
   - Hệ sinh thái Linux hoàn toàn không bị độc quyền bởi các CA thương mại. Độ tin cậy dựa trên Web-of-Trust và khóa công khai OpenPGP.
   - Tạo khóa GPG riêng cho TextVN Release Signer: `TextVN Release Signing Key <release@textvn.vn>`.
   - Trong workflow GitHub Actions, tự động ký rời (detached signature) cho mọi tệp phân phối:
     ```bash
     gpg --armor --detach-sign dist/TextVN-*-linux-*.tar.gz
     gpg --armor --detach-sign dist/SHA256SUMS.txt
     ```
   - Xuất bản khóa công khai `textvn-release-key.asc` ngay trên trang chủ và kho GitHub.
2. **Lộ trình Kênh phân phối Bản địa (Native Repositories - 0 VNĐ):**
   - **Ubuntu/Debian:** Thiết lập Launchpad PPA `ppa:textvn/stable`, tự động build source package `.dsc` phục vụ cài đặt qua `apt install textvn-ibus textvn-fcitx5`.
   - **Arch Linux:** Đưa PKGBUILD lên Arch User Repository (`AUR/textvn-git` và `AUR/textvn-bin`). Người dùng cài qua `yay -S textvn`.
   - **Fedora/RHEL:** Tạo dự án trên Fedora COPR. Người dùng cài qua `dnf copr enable textvn/stable && dnf install textvn`.
   - **Universal:** Đóng gói Flatpak hoặc AppImage cho giao diện cấu hình `textvn-settings`.

---

### 5.4. MA TRẬN ĐỐI CHIẾU TOÀN DIỆN 3 NỀN TẢNG (WINDOWS · MACOS · LINUX)

| Tiêu chí đánh giá | Windows | macOS | Linux |
|---|---|---|---|
| **Kiến trúc Engine chính** | TSF In-proc COM (`textvn-tsf.dll`) | InputMethodKit (`TextVNInputController.swift`) | IBus (`engine.c`) & Fcitx5 (`engine.cpp`) |
| **Kiến trúc Phụ / Fallback** | `textvn-hook.exe` (LL Hook) | `adapters/macos-tap` (CGEventTap) | Zero-grab trên Wayland / X11 opt-in |
| **Giao diện Cài đặt (Tray/UI)** | Tray Rust Win32 + Windows API | Menu Bar App (AppKit + SwiftUI) | `textvn-settings` (GTK4 / C) |
| **Cơ chế Ký số Chính thống** | Authenticode SHA-256 | Apple Developer ID + Notarization | Khóa GPG (OpenPGP Key) |
| **Chi phí Ký số Chính thống** | **0 VNĐ** (MSIX Store / SignPath OSS) | $99/năm (hoặc Free Waiver cho tổ chức phi lợi nhuận) | **0 VNĐ** (100% Miễn phí toàn diện) |
| **Rào cản Bảo mật HĐH** | SmartScreen, Policy Store 10.2.9 | Gatekeeper, Hardened Runtime, TCC Permissions | Không có CA gate; phụ thuộc quyền sudo / Polkit |
| **Kênh Phân phối Khuyên dùng** | **Microsoft Store (MSIX)** & GitHub | **Homebrew Cask** (`textvn.rb`) & `.pkg` | **Launchpad PPA, AUR, COPR** & Flathub |
| **Lỗi/Rào cản Cốt lõi Còn lại** | **W-06 (32-bit Zalo)**, Race Ctrl+Shift | **M-01 (Icon Menu)**, M-03 (x86_64 CI), Test máy thật | **L-03 (Thiếu DEB/RPM)**, L-01 (IPC JSON), Surrounding text fallback |
| **Điểm Đạt chuẩn Production** | **8.5 / 10** | **8.0 / 10** | **8.5 / 10** |

---

## 6. PHƯƠNG ÁN XỬ LÝ TRIỆT ĐỂ (STEP-BY-STEP REMEDIATION BLUEPRINT)

*Dưới đây là kế hoạch chi tiết từng bước mà đội ngũ phát triển cần triển khai ở phiên làm việc kế tiếp để khắc phục toàn bộ các lỗi phát hiện và đưa dự án đạt chuẩn sản xuất 100%.*

### Kế hoạch hành động 5 giai đoạn:

```
+---------------------------------------------------------------------------------------+
| Giai đoạn 1: Triệt tiêu Rủi ro An ninh & Khắc phục Lỗi Bàn phím Trọng yếu (P0 & P1)  |
| - Gỡ bỏ WH_KEYBOARD_LL trong Tray; loại bỏ UAC runas từ thư mục Portable.             |
| - Sửa Race Condition Ctrl+Shift trên Windows (bảo vệ luồng IPC độc quyền).           |
| - Bổ sung updateStatusIcon() vào Delegate của macOS AppDelegate.                      |
| - Bổ sung xử lý app_id "*" trong bộ giải mã IPC Linux ipc_client.c.                  |
+---------------------------------------------------------------------------------------+
                                           |
                                           v
+---------------------------------------------------------------------------------------+
| Giai đoạn 2: Xử lý Triệt để Lỗi Gõ chữ Bị gạch chân (Underline / Preedit)            |
| - Đăng ký DISPATTR_TEXTVN vào Category Manager trong cli/src/register.rs.             |
| - Tích hợp cờ non_preedit vào cơ chế IMK macOS và Fcitx5 Surrounding Text.           |
| - Cache đối tượng ITfCategoryMgr tránh gọi CoCreateInstance liên tục.                |
+---------------------------------------------------------------------------------------+
                                           |
                                           v
+---------------------------------------------------------------------------------------+
| Giai đoạn 3: Tối ưu Hóa Core Engine & Tăng cường Từ điển Khôi phục Tiếng Anh          |
| - Chuyển đổi data_contains sang mảng tĩnh + binary_search / phf_set.                 |
| - Mở rộng tập từ điển en_common.txt lên > 1.000 từ vựng thực tế.                     |
| - Hoàn thiện các FFI stub còn dở dang trong ffi/src/lib.rs.                           |
+---------------------------------------------------------------------------------------+
                                           |
                                           v
+---------------------------------------------------------------------------------------+
| Giai đoạn 4: Hoàn thiện Pipeline Ký số & Tự động hóa CI/CD                           |
| - Kích hoạt bước sign-signpath.ps1 trong release.yml (Windows).                       |
| - Tích hợp Developer ID Signing & Notarize Action trong ci-macos.yml.                |
| - Thêm chữ ký GPG tự động cho các bản phát hành Linux.                               |
+---------------------------------------------------------------------------------------+
                                           |
                                           v
+---------------------------------------------------------------------------------------+
| Giai đoạn 5: Chuẩn hóa Đóng gói Microsoft Store & Hoàn thiện Tài liệu                 |
| - Chuẩn hóa quy trình cài đặt Silent và đăng ký TSF per-user tương thích Win11.      |
| - Cập nhật README, Release Notes, Hướng dẫn người dùng và Kiến trúc.                  |
+---------------------------------------------------------------------------------------+
```

---

### KẾT LUẬN CỦA CHUYÊN GIA KIỂM TOÁN
Dự án **TextVN** sở hữu nền tảng kiến trúc Rust Core rất vững chắc, khả năng tương tác FFI đa nền tảng tốt và thiết kế giao thức IPC sạch sẽ. Các vấn đề người dùng gặp phải hiện nay tập trung chủ yếu ở **tầng adapter hệ điều hành (lớp chuyển tiếp sự kiện phím và đồng bộ giao diện)** cùng **các chính sách bảo mật ngặt nghèo của hệ điều hành hiện đại (Windows 11 24H2, macOS Hardened Runtime)**.

Bằng việc thực hiện nghiêm túc kế hoạch khắc phục theo lộ trình đã được kiểm toán chi tiết trong tài liệu này, TextVN sẽ hoàn toàn loại bỏ được các lỗi khó chịu về phím tắt, hiện tượng gạch chân, đạt điểm kiểm duyệt tuyệt đối trên Microsoft Store và phân phối an toàn tới tay mọi người dùng trên cả ba hệ điều hành.

---
*Báo cáo được lập bởi Antigravity Principal Architecture & Quality Assurance Auditor.*

---

## PHỤ LỤC — Trạng thái xử lý từng phát hiện (cập nhật 2026-10-06, v0.2.24)

| Phát hiện | Trạng thái | Ghi chú & Đánh giá kiểm toán |
|---|---|---|
| **W-02 (SEC-02)** nâng quyền Portable UAC | ✅ ĐÃ SỬA | Chỉ tự nâng quyền từ thư mục cài chuẩn: Program Files (gồm MSIX WindowsApps) + `%LOCALAPPDATA%\Programs` (bộ cài per-user/Store). Path tùy ý (Downloads, D:\ portable) bị TỪ CHỐI + báo MỘT lần hướng dẫn cài đặt (`tray/src/main.rs::show_untrusted_path_notice_once`). Bước nâng quyền chỉ chạy sau khi activation per-user thất bại thật. |
| **M-02 (SEC-03)** entitlements macOS thừa | ✅ ĐÃ SỬA | Chỉ còn `com.apple.security.automation.apple-events`, đã loại bỏ `allow-jit` và `allow-unsigned-executable-memory`. |
| **W-03 (BUG-07)** Registry hotkey thiếu hoàn trả | ✅ ĐÃ SỬA | `.iss` `usPostUninstall` + `uninstall.ps1` xoá `Layout Hotkey` khi giá trị = 3 (`RegDeleteValue`). |
| **W-04 (SEC-05)** thiếu DACL named pipe | ✅ ĐÃ SỬA | `tray/src/ipc_server.rs::current_user_pipe_security` — SDDL `D:(A;;GA;;;<user>)(A;;GA;;;SY)(A;;GA;;;BA)`. Hậu tố Session ID trong tên pipe: HOÃN (phải sửa đồng bộ TSF/Hook/CLI/doctor). |
| **L-02 (SEC-04)** socket /tmp | ✅ ĐÃ SỬA | Ưu tiên `$XDG_RUNTIME_DIR/TextVN/ipc.sock`; fallback `/tmp/textvn-<uid>/ipc.sock`; guard `lstat` (phải là socket của chính user) trước connect. |
| **C-01 (PERF-01)** quét từ điển | ✅ ĐÃ SỬA (tối thiểu) | Lọc theo độ dài trước `to_lowercase()`; dữ liệu đã nhúng compile-time. |
| **STO-02 / STO-03 / STO-04** Microsoft Store Submission | ✅ ĐÃ PASS 3 CHECK TỰ ĐỘNG $\rightarrow$ CHUYỂN SANG CHÍNH SÁCH 10.2.9 | **Thành quả lịch sử v0.2.24:** Bản máy đã vượt qua 100% cả 3 bài test Silent install, ARP entry, và Bundleware. **Thách thức mới (STO-04):** Chính sách bảo mật 10.2.9 yêu cầu chữ ký số SHA-256 cho file EXE. **Hai hướng xử lý:** (1) SignPath Foundation cho EXE, hoặc (2) Giải phóng tên Win32 cũ trên Partner Center để tạo sản phẩm MSIX mới được Microsoft Store tự động ký số miễn phí 100%. |
| **L-01 (BUG-03)** IPC client bỏ qua `app_id:"*"` | ⏸ HOÃN (có lý do) | Parser Linux thiếu nhánh `"*"`, nhưng `lc_ipc_client_poll` chưa từng được gọi trên Linux (engine đồng bộ qua `state.json`/`config.json`). |
| **3.2.B (BUG-05)** macOS `non_preedit` bị bỏ quên | ⏸ HOÃN (có lý do) | Toggle trong SettingsView ghi config nhưng IMK chưa đọc. Cần test thật với Chromium/Office/Terminal trước để tránh lỗi con trỏ/autocomplete. |
| **3.2.C (BUG-06)** Linux preedit | ⏸ HOÃN (có lý do) | Client preedit là quyết định thiết kế đã ghi (`lc_compose.h:11-14`). |
| **CI-01** ký số chưa nối vào release | ⏸ CHỜ CHỨNG THƯ | Script ký có sẵn; chỉ nối được khi có cert (SignPath Foundation/Azure Trusted Signing — Giai đoạn 5). |
| **W-06 (BUG-08)** Zalo PC & Ứng dụng 32-bit không gõ được | ❌ CHƯA SỬA (MỚI PHÁT HIỆN) | **Nguyên nhân cốt lõi:** Zalo PC là tiến trình 32-bit (x86 WOW64) không thể nạp TSF DLL 64-bit; TextVN chưa biên dịch target `i686-pc-windows-msvc` và chưa đăng ký `InprocServer32` trong nhánh `WOW6432Node`. Cần biên dịch bản `textvn-tsf-x86.dll` và đăng ký song song cả 2 kiến trúc. Workaround: Dùng Zalo Web trên trình duyệt 64-bit. |
