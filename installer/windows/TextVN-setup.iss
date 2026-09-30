; SPDX-License-Identifier: GPL-3.0-or-later
; Script dong goi bo cai dat TextVN cho Windows bang Inno Setup 6 (WIN-054 / P1-4 §4).
; Ho tro cai dat linh hoat:
; - Mac dinh per-user (khong yeu cau quyen Administrator).
; - Che do All Users khi chay elevated.

#define MyAppName "TextVN"
#define MyAppFullName "TextVN"
; Version: build-release.ps1 / build_installer.ps1 truyen /DMyAppVersion=<ver>
; (doc tu Cargo.toml). #ifndef de /D cua ISCC thang - #define tinh se de tham so
; va ket version cu (review R3 blocker 1). Fallback duoi day duoc gate
; `cargo xtask check-version-sync` giu khop Cargo.toml.
#ifndef MyAppVersion
  #define MyAppVersion "0.2.0"
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
; per-user sau. Khong nang quyen binary trong {app}: voi cai per-user, day la thu muc
; nguoi dung ghi duoc va ShellExec('runas') se mo lo hong leo thang dac quyen.
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

// Dang ky per-user khong can UAC. Khong tu dong ShellExec('runas') mot executable
// nam trong {app}, vi {app} cua cai per-user co the bi user/process khac thay doi.
// Machine scope chi nen duoc cap boi mot installer/helper da ky, o thu muc tin cay.
function RegisterTextServices(): Boolean;
var
  ResultCode: Integer;
begin
  ResultCode := -1;
  Result := Exec(CliPath(), 'register', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) and
    (ResultCode = 0);
  if not Result then
    Log(Format('TextVN per-user TSF registration failed (exit code %d).', [ResultCode]));
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
          'Mo "TextVN Doctor" sau khi sua chinh sach/quyen registry roi chon "Cai & bat TSF".',
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
