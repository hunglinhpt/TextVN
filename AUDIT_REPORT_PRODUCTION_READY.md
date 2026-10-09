# BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)

> **TÀI LIỆU LỊCH SỬ — ảnh chụp 2026-10-04 (v0.2.19), KHÔNG phản ánh hiện trạng.** Bản đầy
> đủ + bảng trạng thái từng phát hiện: [`docs/release/AUDIT_AND_PRODUCTION_ROADMAP_v0.2.19.md`](docs/release/AUDIT_AND_PRODUCTION_ROADMAP_v0.2.19.md)
> (PHỤ LỤC). Hiện trạng ký số (2026-10-09): từ v0.2.27 mọi asset release được ký GPG (`.asc`,
> FPR `3921595ABC961199F15303B6C45B84D0C7F4A822`, UID `TextVN Release Signing
> <hunglinhpt@users.noreply.github.com>`, public key `docs/release/signing/gpg-release-key.asc`)
> và Sigstore keyless, `SHA256SUMS.txt` clearsign — trong CI là **bắt buộc** (thiếu khoá/cosign
> thì dừng phát hành) và tự verify (`docs/release/signing.md`). Còn thiếu: Authenticode cho
> Windows (chờ SignPath Foundation duyệt) và Apple Developer ID/notarization cho macOS
> (`docs/release/signing-status-mac.md`). Khoá `release@textvn.vn` / `textvn-release-key.asc`
> nêu ở §5.3 dưới đây là đề xuất cũ, không phải khoá đang dùng.

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

---

### 5.3. Ký số & Phân phối Linux (GPG, PPA, COPR, AUR, Flatpak)

1. **Ký số GPG cho Asset Phát hành:**
   - Tạo khóa GPG riêng cho TextVN Release Signer: `TextVN Release Signing Key <release@textvn.vn>`.
   - Trong workflow GitHub Actions, tự động ký rời (detached signature) cho mọi tệp phân phối:
     ```bash
     gpg --armor --detach-sign dist/TextVN-*-linux-*.tar.gz
     gpg --armor --detach-sign dist/SHA256SUMS.txt
     ```
   - Xuất bản khóa công khai `textvn-release-key.asc` ngay trên trang chủ và kho GitHub.
2. **Lộ trình Kênh phân phối Bản địa (Native Repositories):**
   - **Ubuntu/Debian:** Thiết lập Launchpad PPA `ppa:textvn/stable`, tự động build source package `.dsc` phục vụ cài đặt qua `apt install textvn-ibus textvn-fcitx5`.
   - **Arch Linux:** Đưa PKGBUILD lên Arch User Repository (`AUR/textvn-git` và `AUR/textvn-bin`).
   - **Fedora:** Tạo dự án trên Fedora COPR.
   - **Universal:** Đóng gói Flatpak hoặc AppImage cho giao diện cấu hình `textvn-settings`.

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
