; SPDX-License-Identifier: GPL-3.0-or-later
; Script đóng gói bộ cài đặt VietIME cho Windows bằng Inno Setup 6 (WIN-054 / P1-4 §4).
; AV/FP (RW3): hanh vi cai nay (register hidden + Run key + hook exe) bi antivirus hieu nham
;   - phan tich + ke hoach: docs/specs/antivirus-false-positive.md
; Hỗ trợ cài đặt linh hoạt:
; - Mặc định per-user (không yêu cầu quyền Administrator — tuân thủ S5 và PLAN §3.7/§8).
; - Chế độ All Users (System mode) khi chạy elevated hoặc chỉ định /ALLUSERS.

#define MyAppName "VietIME"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "VietIME Authors"
#define MyAppURL "https://github.com/hunglinhpt/TextVN"
#define MyAppExeName "vietime-tray.exe"

[Setup]
AppId={{9C5E4A7D-092E-4D23-9F93-87B75F3FA7B3}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=..\..\LICENSE
OutputDir=..\..\target\installer
OutputBaseFilename=vietime-setup-{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

; Cho phép người dùng chọn cài đặt cho bản thân (per-user) hoặc toàn bộ máy (admin)
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

; Không adware, không cài kèm phần mềm lạ, không đổi homepage
DisableWelcomePage=no

[Languages]
Name: "vietnamese"; MessagesFile: "compiler:Languages\Vietnamese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

#ifndef TargetDir
#define TargetDir "..\..\target\x86_64-pc-windows-msvc\release"
#endif

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "autostart"; Description: "Tự động khởi động VietIME cùng Windows"; GroupDescription: "Tùy chọn khởi động:"; Flags: checkedonce

[Files]
; Artifacts binary đã build từ target/release hoặc target/x86_64-pc-windows-msvc/release
Source: "{#TargetDir}\vietime-tray.exe"; DestDir: "{app}"; Flags: ignoreversion; DestName: "vietime-tray.exe"
Source: "{#TargetDir}\vietime-hook.exe"; DestDir: "{app}"; Flags: ignoreversion; DestName: "vietime-hook.exe"
Source: "{#TargetDir}\vietime_win_tsf.dll"; DestDir: "{app}"; Flags: ignoreversion; DestName: "vietime-tsf.dll"
Source: "{#TargetDir}\vietime.exe"; DestDir: "{app}"; Flags: ignoreversion; DestName: "vietime.exe"
Source: "..\..\data\*"; DestDir: "{app}\data"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Kiểm tra hệ thống (VietIME Doctor)"; Filename: "{app}\vietime.exe"; Parameters: "doctor"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; Tự khởi động cùng Windows khi đăng nhập (per-user)
Root: HKA; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#MyAppName}"; ValueData: """{app}\{#MyAppExeName}"" --autostart"; Flags: uninsdeletevalue; Tasks: autostart

[Run]
; 1. Khởi tạo cấu hình mặc định (không ghi đè nếu đã tồn tại)
Filename: "{app}\vietime.exe"; Parameters: "config init"; Flags: runhidden
; 2. Đăng ký Text Services Framework (TSF TIP)
Filename: "{app}\vietime.exe"; Parameters: "register"; Flags: runhidden
; 3. Chạy khay hệ thống VietIME Tray
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; 1. Yêu cầu dừng tiến trình Tray và Hook đang chạy
Filename: "{app}\{#MyAppExeName}"; Parameters: "--stop"; Flags: runhidden
; 2. Huỷ đăng ký TIP sạch sẽ khỏi hệ thống
Filename: "{app}\vietime.exe"; Parameters: "unregister"; Flags: runhidden

[Code]
// Tuân thủ Rule S9: Không tự ý xoá dữ liệu cấu hình cá nhân %APPDATA%\VietIME khi gỡ cài đặt
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    // Chỉ hỏi người dùng nếu có nhu cầu dọn dẹp sạch toàn bộ
    // Mặc định bảo toàn dữ liệu cấu hình người dùng
  end;
end;
