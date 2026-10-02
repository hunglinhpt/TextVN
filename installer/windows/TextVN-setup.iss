; SPDX-License-Identifier: GPL-3.0-or-later
; Script dong goi bo cai dat TextVN cho Windows bang Inno Setup 6 (WIN-054 / P1-4 §4).
; Ho tro cai dat linh hoat:
; TSF RegisterProfile/RegisterCategory can HKLM: cai machine mot lan, sau do
; bat profile trong user session. CLI elevated chi duoc chay tu Program Files.

#define MyAppName "TextVN"
#define MyAppFullName "TextVN"
; Version: build-release.ps1 / build_installer.ps1 truyen /DMyAppVersion=<ver>
; (doc tu Cargo.toml). #ifndef de /D cua ISCC thang - #define tinh se de tham so
; va ket version cu (review R3 blocker 1). Fallback duoi day duoc gate
; `cargo xtask check-version-sync` giu khop Cargo.toml.
#ifndef MyAppVersion
  #define MyAppVersion "0.2.7"
#endif
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
; Icon wizard cài đặt/uninstaller (trước đây dùng icon mặc định của Inno Setup)
SetupIconFile=..\..\tray\resources\textvn.ico
DefaultDirName={autopf}\{#MyAppName}
DisableDirPage=yes
UsePreviousAppDir=no
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

PrivilegesRequired=admin
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
Filename: "{app}\textvn-cli.exe"; Parameters: "config init"; Flags: runhidden runasoriginaluser
Filename: "{app}\{#MyAppExeName}"; Parameters: "--free-ctrl-shift"; Flags: runhidden runasoriginaluser; Tasks: freectrlshift
; Dang ky TSF TIP chay trong [Code] (CurStepChanged/ssPostInstall) de dung thu tu:
; machine elevated truoc, user session sau. Khong nang quyen binary o thu muc
; user-writable: path installer duoc khoa o {autopf}\TextVN.
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppFullName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent; Check: RegistrationSucceeded

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

var
  RegistrationOK: Boolean;

// Exec CLI elevated chi tu thu muc Program Files co ACL admin. /DIR custom hoac
// previous app dir khong duoc phep doi dich do an toan nay.
function RegisterTextServices(): Boolean;
var
  ResultCode: Integer;
begin
  if CompareText(ExpandConstant('{app}'), ExpandConstant('{autopf}\TextVN')) <> 0 then
  begin
    Log('TextVN refused elevated TSF registration outside Program Files.');
    Result := False;
    Exit;
  end;
  // Cai PER-USER ({autopf} = %LOCALAPPDATA%\Programs, khong elevation): dang ky
  // per-user la du va dung — khong goi --scope machine (CLI tra exit 3 khi
  // khong co quyen, truoc day lam toan bo nhanh per-user bao "Windows tu choi
  // dang ky" va ep nguoi dung phai chay bang admin).
  if not IsAdminInstallMode then
  begin
    ResultCode := -1;
    Result := ExecAsOriginalUser(CliPath(), 'register', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) and
      (ResultCode = 0);
    if not Result then
      Log(Format('TextVN per-user TSF registration failed (exit code %d).', [ResultCode]));
    Exit;
  end;
  ResultCode := -1;
  Result := Exec(CliPath(), 'register --scope machine', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) and
    (ResultCode = 0);
  if not Result then
  begin
    Log(Format('TextVN machine TSF registration failed (exit code %d).', [ResultCode]));
    Exit;
  end;
  ResultCode := -1;
  Result := ExecAsOriginalUser(CliPath(), 'register', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) and
    (ResultCode = 0);
  if not Result then
    Log(Format('TextVN user TSF activation failed (exit code %d).', [ResultCode]));
end;

function RegistrationSucceeded(): Boolean;
begin
  Result := RegistrationOK;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    RegistrationOK := RegisterTextServices();
    if not RegistrationOK then
    begin
      Log('TextVN was installed but not started because TSF registration did not complete.');
      // Cai im lang: bao loi qua exit code (GetCustomSetupExitCode ben duoi).
      // Inno Setup khong co 'SetErrorFlag' - ban truoc khong compile duoc.
      if not WizardSilent then
        SuppressibleMsgBox('TextVN da duoc chep, nhung Windows tu choi dang ky bo go cho tai khoan nay.' + #13#10#13#10 +
          'TextVN chua duoc mo de tranh hien trang da cai nhung khong go duoc tieng Viet.' + #13#10 +
          'Xem %LOCALAPPDATA%\TextVN\logs\register.log de biet buoc loi. ' +
          'Neu TSF profile bi chan boi chinh sach may, lien he quan tri vien.',
          mbError, MB_OK, IDOK);
    end;
  end;
end;

// Event function Inno Setup 6: chi duoc goi khi Setup chay xong va exit code se
// la 0. Tra khac 0 khi dang ky TSF that bai de script/CI cai im lang nhan ra
// "da chep file nhung khong go duoc". 10 nam ngoai dai ma Inno dung san (0..8).
function GetCustomSetupExitCode(): Integer;
begin
  if RegistrationOK then
    Result := 0
  else
    Result := 10;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    // Bao toan du lieu cau hinh nguoi dung trong %APPDATA%\TextVN
  end;
end;
