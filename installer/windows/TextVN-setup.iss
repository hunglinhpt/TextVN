; SPDX-License-Identifier: GPL-3.0-or-later
; Script dong goi bo cai dat TextVN cho Windows bang Inno Setup 6 (WIN-054 / P1-4 §4).
; Ho tro cai dat linh hoat:
; - Mac dinh per-user (khong yeu cau quyen Administrator).
; - Che do All Users khi chay elevated.

#define MyAppName "TextVN"
#define MyAppFullName "TextVN"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "hunglinhpt"
#define MyAppURL "https://github.com/hunglinhpt/TextVN"
#define MyAppExeName "TextVN.exe"

[Setup]
AppId={{9C5E4A7D-092E-4D23-9F93-87B75F3FA7B3}
AppName={#MyAppFullName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=..\..\LICENSE
OutputDir=..\..\dist
OutputBaseFilename=TextVN-setup-{#MyAppVersion}-windows-x64
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
DisableWelcomePage=no

[Languages]
Name: "vietnamese"; MessagesFile: "languages\Vietnamese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

#ifndef TargetDir
#define TargetDir "..\..\target\x86_64-pc-windows-msvc\release"
#endif

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "autostart"; Description: "Tu dong khoi dong TextVN cung Windows"; GroupDescription: "Tuy chon khoi dong:"; Flags: checkedonce
; Windows mac dinh dung Ctrl+Shift de doi bo cuc ban phim - trung phim chuyen V/E (tray/src/hotkey.rs).
Name: "freectrlshift"; Description: "Danh Ctrl + Shift cho TextVN (tat phim Ctrl + Shift doi ban phim cua Windows)"; GroupDescription: "Phim chuyen tieng Viet:"; Flags: checkedonce

[Files]
Source: "{#TargetDir}\TextVN.exe"; DestDir: "{app}"; Flags: ignoreversion; DestName: "TextVN.exe"
#ifdef IncludeCompatibilityHook
Source: "{#TargetDir}\textvn-hook.exe"; DestDir: "{app}"; Flags: ignoreversion; DestName: "textvn-hook.exe"
#endif
Source: "{#TargetDir}\textvn-cli.exe"; DestDir: "{app}"; Flags: ignoreversion; DestName: "textvn-cli.exe"
Source: "{#TargetDir}\textvn_win_tsf.dll"; DestDir: "{app}"; Flags: ignoreversion; DestName: "textvn-tsf.dll"
Source: "..\..\tray\resources\*"; DestDir: "{app}\resources"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\data\*"; DestDir: "{app}\data"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppFullName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Kiem tra he thong (TextVN Doctor)"; Filename: "{app}\textvn-cli.exe"; Parameters: "doctor"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppFullName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
Root: HKA; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#MyAppName}"; ValueData: """{app}\{#MyAppExeName}"" --autostart"; Flags: uninsdeletevalue; Tasks: autostart

[Run]
Filename: "{app}\textvn-cli.exe"; Parameters: "config init"; Flags: runhidden
Filename: "{app}\{#MyAppExeName}"; Parameters: "--free-ctrl-shift"; Flags: runhidden; Tasks: freectrlshift
; Dang ky TSF TIP chay trong [Code] (CurStepChanged/ssPostInstall) de dung thu tu:
; HKLM (neu co quyen) truoc, per-user sau.
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppFullName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{app}\{#MyAppExeName}"; Parameters: "--stop"; Flags: runhidden; RunOnceId: "StopTray"
Filename: "{app}\textvn-cli.exe"; Parameters: "unregister"; Flags: runhidden; RunOnceId: "UnregisterUser"
Filename: "{app}\textvn-cli.exe"; Parameters: "unregister --scope machine"; Flags: runhidden; Check: IsAdminInstallMode; RunOnceId: "UnregisterMachine"

[Code]
const
  TipKey = 'SOFTWARE\Microsoft\CTF\TIP\{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}';

function CliPath(): String;
begin
  Result := ExpandConstant('{app}\textvn-cli.exe');
end;

// Profile TSF chuan duoc Windows luu o HKLM (ITfInputProcessorProfileMgr).
// Cai per-user khong co quyen do: hoi nguoi dung cap quyen MOT lan. Tu choi thi
// CLI van dung fallback per-user (HKCU) - bo go van cai dat day du.
// Cai im lang (/SUPPRESSMSGBOXES) mac dinh KHONG hien UAC: chi dang ky per-user.
procedure RegisterTextServices();
var
  ResultCode: Integer;
begin
  if IsAdminInstallMode then
    Exec(CliPath(), 'register --scope machine', '', SW_HIDE, ewWaitUntilTerminated, ResultCode)
  else if not RegKeyExists(HKLM, TipKey) then
  begin
    if SuppressibleMsgBox('TextVN can dang ky bo go voi Windows (Text Services Framework).' + #13#10 +
        'Buoc nay can quyen quan tri MOT lan de TextVN go duoc trong moi ung dung.' + #13#10#13#10 +
        'Tiep tuc?', mbConfirmation, MB_YESNO, IDNO) = IDYES then
      ShellExec('runas', CliPath(), 'register --scope machine', '', SW_HIDE,
        ewWaitUntilTerminated, ResultCode);
  end;
  Exec(CliPath(), 'register', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    RegisterTextServices();
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    // Bao toan du lieu cau hinh nguoi dung trong %APPDATA%\TextVN
  end;
end;
