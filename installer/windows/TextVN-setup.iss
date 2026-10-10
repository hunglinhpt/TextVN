; SPDX-License-Identifier: GPL-3.0-or-later
; Script dong goi bo cai dat TextVN cho Windows bang Inno Setup 6 (WIN-054 / P1-4 §4).
; Ho tro cai dat linh hoat:
; TSF RegisterProfile/RegisterCategory can HKLM: cai machine mot lan, sau do
; bat profile trong user session. CLI elevated chi duoc chay tu Program Files.

; Ten/publisher ARP: ghi de duoc bang /D khi build de khop TUNG CHU voi Partner
; Center (STO-03 / B13f). Mac dinh la gia tri da dung tu 0.2.18; neu ten san pham
; tren Partner Center khac (vi du "TextVN - Bo go tieng Viet") thi build lai:
;   ISCC.exe /DMyAppName="Ten that" /DMyAppPublisher="Nha phat hanh that" ...
#ifndef MyAppName
  #define MyAppName "TextVN"
#endif
; Version: build-release.ps1 / build_installer.ps1 truyen /DMyAppVersion=<ver>
; (doc tu Cargo.toml). #ifndef de /D cua ISCC thang - #define tinh se de tham so
; va ket version cu (review R3 blocker 1). Fallback duoi day duoc gate
; `cargo xtask check-version-sync` giu khop Cargo.toml.
#ifndef MyAppVersion
  #define MyAppVersion "0.2.27"
#endif
; Publisher ARP PHAI khop TUNG CHU voi "Publisher display name" tren Partner
; Center. Lich su 7 vong (B13f/STO-03/B18) — chu tai khoan TIM DUOC dung trang
; trong account va CHINH SUA ngay tai do 2026-10-05:
;   0.2.16-0.2.20: "LinhBH.CoM"  -> chuoi DUNG, nhung cac vong do thua vi ly do
;                  khac (0.2.16 exit 10, 0.2.18 admin=740, 0.2.19 bay B13g
;                  "no changes") — khong phai vi chuoi sai
;   0.2.21:        "Linh βùi"    -> SAI (doc nham font: B U+0042 nhu β U+03B2)
;   0.2.22:        "Linh Bui"    -> SAI (suy dien tu lan doc sai thu hai)
;   0.2.23:        "LinhBH.CoM"  -> DUNG, xac nhan tu nguon su that
; Bai hoc B18 them: gia tri Publisher display name nam o trang KHAC voi nhung
; gi hien o overview/certificate — phai vao Account settings -> Publisher info
; va dump codepoint truoc khi tin bat ky chuoi nao.
#ifndef MyAppPublisher
  #define MyAppPublisher "LinhBH.CoM"
#endif
#define MyAppURL "https://github.com/hunglinhpt/TextVN"
#define MyAppExeName "TextVN.exe"

[Setup]
AppId={{9C5E4A7D-092E-4D23-9F93-87B75F3FA7B3}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
; ARP DisplayName = ĐÚNG tên sản phẩm trên Partner Center (validator đối chiếu từng chữ).
AppVerName={#MyAppName}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
; Vòng 11 (rà từng dòng lần 3) — 3 khe hở còn lại phía gói, đóng hết:
; 1) UninstallDisplayIcon: trước đây entry ARP THIẾU value `DisplayIcon` —
;    checker nào tra DisplayIcon (icon trong Programs and Features) sẽ thấy
;    entry "không hoàn chỉnh". Giờ trỏ vào exe chính.
UninstallDisplayIcon={app}\{#MyAppExeName}
; 2) VersionInfoTextVersion: trước đây chuỗi FileVersion của setup.exe TRỐNG
;    (chỉ có số) — checker đọc metadata file sẽ thấy FileVersion rỗng. Giờ đầy đủ.
VersionInfoTextVersion={#MyAppVersion}
; 3) SetupLogging: ghi log cài đặt vào %TEMP% — nếu validation lại fail và chủ
;    repo mở ticket với Microsoft, log trên VM của họ là bằng chứng duy nhất
;    ta có thể yêu cầu. Không ảnh hưởng thời gian cài (file text vài chục KB).
SetupLogging=yes
; Icon wizard cài đặt/uninstaller (trước đây dùng icon mặc định của Inno Setup)
SetupIconFile=..\..\tray\resources\textvn.ico
DefaultDirName={autopf}\{#MyAppName}
DisableDirPage=yes
UsePreviousAppDir=no
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=..\..\LICENSE
OutputDir=..\..\dist
; OutputSuffix cho phep build BIEN THE canh nhau ma khong ghi de (vi du
; ISCC /DOutputSuffix="-machine" -> TextVN-setup-<ver>-windows-x64-machine.exe).
#ifndef OutputSuffix
  #define OutputSuffix ""
#endif
OutputBaseFilename=TextVN-setup-{#MyAppVersion}-windows-x64{#OutputSuffix}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

; MAC DINH = per-user (KHONG elevation): validator cua Store tu dong chay
; installer qua CreateProcess — exe co manifest requireAdministrator fail NGAY
; voi ERROR_ELEVATION_REQUIRED (740) truoc khi cai gi ca → ca 3 check (silent/
; ARP/bundleware) cung do vi KHONG CO LAN CAI NAO dien ra (bat that: exit=2
; trong shell non-elevated 2026-10-04; B13). Per-user chay duoc o moi ngu canh,
; ARP o HKCU — Programs and Features hien thi gop ca hai hive.
;
; BIEN THE MAY (lo trinh 2 cua audit vong 4 / STO-02 — chi dung khi gia thuyet
; "validator chi doc HKLM" duoc xac nhan):
;   ISCC.exe /DMyAppVersion=<ver> /DMachineInstall=1 /DOutputSuffix="-machine" installer\windows\TextVN-setup.iss
; → che do cai MAY: ARP + file o Program Files/HKLM (Inno lo het, khong tu ghi tay).
; ⚠️ DO THAT 2026-10-04: vi PrivilegesRequiredOverridesAllowed=commandline, manifest
; cua CA HAI ban deu la `asInvoker` (Inno tu relaunch elevated bang UAC khi che do
; can quyen admin) — KHONG the phan biet bang cach doc manifest, va trong moi
; truong khong tra loi duoc UAC thi ban may se fail y nhu 0.2.18 (exit=2).
; Vi vay: chi dung ban may khi VM cua Store duoc phep elevate; kiem bang
; `validate-store-package.ps1 -MachineOnly` tren runner elevated (co buoc CI).
#ifdef MachineInstall
PrivilegesRequired=admin
#else
PrivilegesRequired=lowest
#endif
PrivilegesRequiredOverridesAllowed=commandline
; /VERYSILENT van co the bi chan boi dialog chon ngon ngu khi locale cua may
; validator khong khop [Languages] (auto -> hien dialog khi khong tim duoc ngon
; ngu he thong) -> cai khong hoan tat = ca 3 check cua Store do (B13). Cam han:
; khong co dialog nao trong moi truong silent.
ShowLanguageDialog=no
DisableWelcomePage=no

[Languages]
Name: "vietnamese"; MessagesFile: "languages\Vietnamese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

#ifndef TargetDir
#define TargetDir "..\..\target\x86_64-pc-windows-msvc\release"
#endif

[UninstallDelete]
; DLL cũ bị rename khi nâng cấp (RenameLockedTsfDll) — dọn cùng uninstaller;
; file còn bị nạp Windows sẽ tự xoá sau khi các tiến trình nhả (sau đăng xuất).
Type: files; Name: "{app}\textvn-tsf.dll.old-*"
Type: files; Name: "{app}\textvn-tsf-x86.dll.old-*"
Type: files; Name: "{app}\TextVN.exe.old-*"

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
; Vong 14 (Zalo 32-bit): TIP DLL x86 cho app WOW64 — build-release copy vao
; TargetDir voi ten textvn-tsf-x86.dll; register ghi mirror COM/CTF 32-bit view.
Source: "{#TargetDir}\textvn-tsf-x86.dll"; DestDir: "{app}"; Flags: ignoreversion; DestName: "textvn-tsf-x86.dll"
Source: "..\..\tray\resources\*"; DestDir: "{app}\resources"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\data\*"; DestDir: "{app}\data"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Kiem tra he thong (TextVN Doctor)"; Filename: "{app}\textvn-cli.exe"; Parameters: "doctor --pause"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
Root: HKA; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#MyAppName}"; ValueData: """{app}\{#MyAppExeName}"" --autostart"; Flags: uninsdeletevalue; Tasks: autostart

[Run]
Filename: "{app}\textvn-cli.exe"; Parameters: "config init"; Flags: runhidden runasoriginaluser skipifsilent
; KHONG skipifsilent: tu CR-17 tray khong tu "danh Ctrl + Shift" khi la ban Inno
; (de bo cai quyet dinh) -> cai im lang (Store /VERYSILENT, nang cap im lang) ma
; bo qua dong nay thi Ctrl + Shift trung phim doi bo cuc cua Windows. Task
; checkedonce: cai im lang lan dau = chon; user bo chon o lan cai tay duoc giu.
Filename: "{app}\{#MyAppExeName}"; Parameters: "--free-ctrl-shift"; Flags: runhidden runasoriginaluser; Tasks: freectrlshift
; Dang ky TSF TIP chay trong [Code] (CurStepChanged/ssPostInstall) de dung thu tu:
; machine elevated truoc, user session sau. Khong nang quyen binary o thu muc
; user-writable: path installer duoc khoa o {autopf}\TextVN.
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent; Check: RegistrationSucceeded

[UninstallRun]
; R2-93: chi dung tray / go dang ky TSF thuoc {app} (hoac mo coi) - ban portable,
; Store hay ban cai o thu muc khac dang dung thi giu nguyen. {app} khong co '\' cuoi.
Filename: "{app}\{#MyAppExeName}"; Parameters: "--stop --if-image-under ""{app}"""; Flags: runhidden; RunOnceId: "StopTray"
Filename: "{app}\textvn-cli.exe"; Parameters: "unregister --if-owned-by ""{app}"""; Flags: runhidden; RunOnceId: "UnregisterUser"
; Vong 13: bo Check IsAdminInstallMode — may tung co dang ky may (portable
; self-heal B7) thi go per-user cung phai don not HKLM. Khong elevation thi CLI
; fail em (exit 1, log) - vo hai; chay elevated (go ban may, hoac chuot phai
; "Run as administrator") thi don sach. Ghost da bat that: go 0.2.19 con sot
; 27 path chua GUID TIP (Assemblies Default + InputMethodOverride + HKLM).
Filename: "{app}\textvn-cli.exe"; Parameters: "unregister --scope machine --if-owned-by ""{app}"""; Flags: runhidden; RunOnceId: "UnregisterMachine"

[Code]
const
  TipKey = 'SOFTWARE\Microsoft\CTF\TIP\{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}';

function CliPath(): String;
begin
  Result := ExpandConstant('{app}\textvn-cli.exe');
end;

var
  RegistrationOK: Boolean;

// textvn-tsf.dll dang duoc TSF nap trong cac tien trinh dang chay (explorer,
// Notepad...) — DeleteFile tra code 5 (Access denied) ke ca khi elevated vi
// Windows KHONG cho xoa file co image section, nhung CHO PHÉP RENAME. Doi ten
// file cu thanh .old-<timestamp> truoc khi copy de cho cho file moi (Bao cao
// 0.2.19: "khong replace duoc profile cu textvn-tsf.dll trong Program Files").
// Cac file .old-* duoc don khi uninstall ([UninstallDelete]) va lan nang cap ke.
// Ap dung cho CA textvn-tsf-x86.dll: app 32-bit (Zalo...) nap ban x86, nen
// nang cap khi app do dang mo cung bi file-in-use nhu ban x64.
procedure RenameLockedFile(const FileName: String);
var
  AppDir, OldFile, Backup: String;
  FindRec: TFindRec;
begin
  AppDir := ExpandConstant('{app}');
  OldFile := AppDir + '\' + FileName;
  // Don backup cu truoc (best-effort — co the van bi nap thi de lai, khong fail)
  if FindFirst(OldFile + '.old-*', FindRec) then
  begin
    try
      repeat
        DeleteFile(AppDir + '\' + FindRec.Name);
      until not FindNext(FindRec);
    finally
      FindClose(FindRec);
    end;
  end;
  if FileExists(OldFile) then
  begin
    Backup := OldFile + '.old-' + GetDateTimeString('yyyymmddhhnnss', '-', '-');
    if RenameFile(OldFile, Backup) then
      Log('Renamed locked ' + FileName + ' to ' + Backup)
    else
      Log('Could not rename locked ' + FileName + '; Setup will show the file-in-use dialog if replace fails.');
  end;
end;

procedure RenameLockedTsfDll();
begin
  RenameLockedFile('textvn-tsf.dll');
  RenameLockedFile('textvn-tsf-x86.dll');
end;

var
  WaitIdx: Integer;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  // Vong 13: neu mot uninstaller Inno van dang chay (no copy minh ra %TEMP%
  // va tiep tuc xoa file {app} SAU khi tien trinh goc thoat — B16) thi cho
  // toi da 10s truoc khi cai. Cai ngay sau khi go thi delete-in-flight dua
  // nhau voi [Files] → silent cancel → exit 2 (repro 2026-10-05).
  for WaitIdx := 0 to 19 do
  begin
    if FindWindowByClassName('TUninstallProgressForm') = 0 then
      Break;
    Log('Waiting for a running Inno uninstaller to finish...');
    Sleep(500);
  end;
  RenameLockedTsfDll();
  Result := '';
end;

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
    // R2-98: tray ghi %APPDATA%\TextVN\autostart_disabled khi tai khoan bo chon "Khoi
    // dong cung Windows" ma muc Run nam o HKLM. Cai lai va chon task autostart = muon
    // bat lai -> bo marker cua tai khoan dang cai (tai khoan khac giu lua chon rieng).
    if WizardIsTaskSelected('autostart') then
      DeleteFile(ExpandConstant('{userappdata}\TextVN\autostart_disabled'));
    if WizardSilent then
    begin
      // Silent (luong Store) = PURE FILE COPY: khong go API TSF trong phien
      // khong-interactive cua validator (co the tre -> timeout -> ca 3 check
      // cua Store do cung luc, B13). App tu dang ky o lan chay dau — 0.2.16:
      // tray tu de nghi UAC mot lan neu Windows tu choi per-user.
      Log('Silent install: skip TSF registration; app self-registers on first run.');
      RegistrationOK := True;
    end
    else
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
end;

// Event function Inno Setup 6: chi duoc goi khi Setup chay xong va exit code se
// la 0. Tra khac 0 khi dang ky TSF that bai de script/CI cai im lang nhan ra
// "da chep file nhung khong go duoc". 10 nam ngoai dai ma Inno dung san (0..8).
function GetCustomSetupExitCode(): Integer;
begin
  if RegistrationOK then
    Result := 0
  // Silent (luồng Store): LUÔN 0 khi đã chép đủ file — validator của Store coi
  // exit != 0 là "cài thất bại", trong khi app sẽ TỰ đăng ký TSF ở lần chạy
  // đầu (kèm đề nghị UAC một lần nếu Windows từ chối per-user — 0.2.19). Đây
  // là lý do bỏ exit 10 cho silent (sự cố B13).
  else if WizardSilent then
  begin
    Log('TSF registration did not complete; app self-heals on first run.');
    Result := 0;
  end
  else
    Result := 10;
end;

// DLL bị TSF nạp (image-mapped) trong mọi tiến trình đang mở nên Windows từ chối
// DeleteFile — không thể "tắt TSF" của hệ điều hành (B12). Ở usUninstall (CLI còn
// trên đĩa — R2-29): đổi tên hai DLL thành .old-<ts> rồi gọi CLI (đang elevated) hẹn
// xoá các .old-* qua MoveFileEx(DELAY_UNTIL_REBOOT). KHÔNG hẹn xoá theo tên gốc
// textvn-tsf.dll: gỡ rồi cài lại trước khi khởi động lại thì lần reboot sẽ xoá mất
// DLL của bản vừa cài.
// — CLI dùng API thật với lpNewFileName=NULL, tránh cả vấn đề Pascal Script
// không có kiểu Pointer lẫn RegWriteMultiStringValue (trả String, không phải
// TArrayOfString).
procedure ScheduleCleanupViaCli();
var
  AppDir, Params: String;
  FindRec: TFindRec;
  ResultCode: Integer;
begin
  AppDir := ExpandConstant('{app}');
  if not DirExists(AppDir) then
    Exit;
  RenameLockedTsfDll();
  Params := 'schedule-delete';
  // Thứ tự: các .old-* (gồm hai DLL vừa đổi tên) → thư mục (PFO xử lý tuần tự).
  if FindFirst(AppDir + '\*.old-*', FindRec) then
  begin
    try
      repeat
        Params := Params + ' "' + AppDir + '\' + FindRec.Name + '"';
      until not FindNext(FindRec);
    finally
      FindClose(FindRec);
    end;
  end;
  Params := Params + ' "' + AppDir + '"';
  if Exec(ExpandConstant('{app}\textvn-cli.exe'), Params, AppDir, SW_HIDE,
          ewWaitUntilTerminated, ResultCode) then
    Log('schedule-delete via CLI, exit=' + IntToStr(ResultCode))
  else
    Log('Cannot run schedule-delete (CLI)');
end;

// R2-40: tra lai Ctrl + Shift theo marker %APPDATA%\TextVN\ctrl_shift_default_applied
// (cung dinh dang tray/src/lib.rs + portable\uninstall.ps1): dong dau 'freed', moi
// dong sau '<ten>=<gia tri cu>' (rong = truoc do khong co); '1' = ban <= 0.2.27, chi
// Layout Hotkey. Khong co marker / 'declined' = TextVN khong doi gi -> giu nguyen lua
// chon "Not assigned" cua nguoi dung. Chi tra muc VAN la '3' (ca Language Hotkey).
procedure RestoreToggleValue(Name, Prev: String);
var
  Cur: String;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Keyboard Layout\Toggle', Name, Cur) then
    Exit;
  if Cur <> '3' then
    Exit;
  if Prev = '' then
    RegDeleteValue(HKEY_CURRENT_USER, 'Keyboard Layout\Toggle', Name)
  else
    RegWriteStringValue(HKEY_CURRENT_USER, 'Keyboard Layout\Toggle', Name, Prev);
  Log('Ctrl+Shift tra lai cho Windows: ' + Name);
end;

procedure RestoreCtrlShift();
var
  Marker, Line, Name, Prev: String;
  Lines: TArrayOfString;
  I, P: Integer;
begin
  Marker := ExpandConstant('{userappdata}\TextVN\ctrl_shift_default_applied');
  if not LoadStringsFromFile(Marker, Lines) then
    Exit;
  if GetArrayLength(Lines) > 0 then
  begin
    if Trim(Lines[0]) = '1' then
      RestoreToggleValue('Layout Hotkey', '')
    else if Trim(Lines[0]) = 'freed' then
      for I := 1 to GetArrayLength(Lines) - 1 do
      begin
        Line := Trim(Lines[I]);
        P := Pos('=', Line);
        if P > 1 then
        begin
          Name := Trim(Copy(Line, 1, P - 1));
          Prev := Trim(Copy(Line, P + 1, Length(Line)));
          if ((Name = 'Layout Hotkey') or (Name = 'Language Hotkey') or (Name = 'Hotkey'))
             and ((Prev = '') or (Prev = '1') or (Prev = '2') or (Prev = '3') or (Prev = '4')) then
            RestoreToggleValue(Name, Prev);
        end;
      end;
  end;
  DeleteFile(Marker);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  RunValue: String;
begin
  // R2-29: hen xoa DLL bi khoa PHAI chay o usUninstall — luc usPostUninstall Inno da
  // xoa textvn-cli.exe nen Exec that bai im lang va DLL/thu muc con lai mai mai.
  if CurUninstallStep = usUninstall then
    ScheduleCleanupViaCli();
  if CurUninstallStep = usPostUninstall then
  begin
    // R2-27: tray tu ghi HKCU Run 'TextVN' (o "Khoi dong cung Windows") - [Registry]
    // chi xoa muc cua chinh bo cai. Muc tro vao thu muc dang go thi xoa; muc cua ban
    // khac (portable/Store) giu nguyen.
    if RegQueryStringValue(HKEY_CURRENT_USER, 'Software\Microsoft\Windows\CurrentVersion\Run', 'TextVN', RunValue)
       and (Pos(Lowercase(ExpandConstant('{app}') + '\'), Lowercase(RunValue)) > 0) then
      RegDeleteValue(HKEY_CURRENT_USER, 'Software\Microsoft\Windows\CurrentVersion\Run', 'TextVN');
    // BUG-07 (audit 2026-10-04): tra lai Ctrl + Shift cho Windows khi go cai dat —
    // theo marker (R2-40, RestoreCtrlShift). Xoa VALUE qua RegDeleteValue: Pascal
    // Script cua Inno KHONG co RegDeleteKeyValue (CI do 2026-10-04: "Unknown
    // identifier 'RegDeleteKeyValue'"; xem win-test-common-errors B14).
    RestoreCtrlShift();
    // Bao toan du lieu cau hinh nguoi dung trong %APPDATA%\TextVN (khong dung toi).
  end;
end;
