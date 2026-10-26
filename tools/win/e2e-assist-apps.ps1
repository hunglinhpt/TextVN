# SPDX-License-Identifier: GPL-3.0-or-later
# tools/win/e2e-assist-apps.ps1 -DevDir <dir with textvn_win_tsf.dll + textvn-cli.exe> [-Apps notepad,edge,npp,vscode,x86rich,x86edit] [-X86Probe <x86probe.exe>] [-Only name] [-Shots]
#
# E2E on REAL apps for the assistant hotkeys + ghost text (R2-131..R2-134):
#   notepad = Notepad Win11 (TSF-aware RichEdit), edge = Edge --app textarea (Chromium TSF =
#   the engine of Teams/WebView2/Electron), npp = Notepad++ (Scintilla, IMM32 via CUAS),
#   vscode = a separate VS Code instance (Electron; own profile under %TEMP%, extensions off),
#   x86rich / x86edit = tools/win/x86probe (32-bit RichEdit / EDIT host, needs -X86Probe <exe>;
#   DevDir must also hold textvn-tsf-x86.dll = the i686 build of textvn-win-tsf).
# Cases: live popup, click "Sua tat ca", Alt+1, Tab prediction, Ctrl+Space fixes the whole
#   sentence, fix then predict, Alt+Space never taken by TextVN (R2-136: window menu opens),
#   Ctrl+Shift+G interactive popup keeps Tab, ghost text before Tab, plain typing.
# Registers the dev TSF DLL as a TEMPORARY HKCU override and always restores the Program Files
# registration in finally (needs the installed build at -ProdCli). Takes the keyboard and mouse
# for ~4 minutes (SendInput). Edge runs with its own --user-data-dir under %TEMP% (the user's
# Edge is not touched). Only test strings are logged (S2). Trace: %TEMP%\textvn-e2e-apps-trace.log
# Result 2026-10-26: 44/44 PASS (P6-TEXT-PREDICTION.md section 2b).
param(
    [Parameter(Mandatory = $true)][string]$DevDir,
    [string]$ProdCli = 'C:\Program Files\TextVN\textvn-cli.exe',
    [string[]]$Apps = @('notepad', 'edge', 'npp', 'vscode'),
    [string]$VSCodeExe = '',
    [string]$X86Probe = '',
    [string]$Only = '',
    [switch]$Shots,
    [switch]$NoRegister
)
$ErrorActionPreference = 'Stop'
$here = Join-Path $env:TEMP 'textvn-e2e-apps'
New-Item -ItemType Directory -Force $here | Out-Null
Add-Type -ReferencedAssemblies System.Drawing @'
using System;
using System.Text;
using System.Threading;
using System.Runtime.InteropServices;
public static class D {
    [StructLayout(LayoutKind.Sequential)]
    struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Explicit, Size = 40)]
    struct INPUT { [FieldOffset(0)] public uint type; [FieldOffset(8)] public KEYBDINPUT ki; }
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] p, int cb);
    [DllImport("user32.dll")] static extern uint MapVirtualKeyW(uint code, uint type);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool f);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowExW(IntPtr parent, IntPtr after, string cls, string title);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, uint d, IntPtr e);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, StringBuilder l);
    [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] static extern bool SetProcessDPIAware();
    public static void DpiAware() { SetProcessDPIAware(); }
    const uint INPUT_KEYBOARD = 1, KEYEVENTF_KEYUP = 2;
    static INPUT Key(ushort vk, bool up) {
        var i = new INPUT(); i.type = INPUT_KEYBOARD; i.ki.wVk = vk;
        i.ki.wScan = (ushort)MapVirtualKeyW(vk, 0); i.ki.dwFlags = up ? KEYEVENTF_KEYUP : 0; return i;
    }
    public static void Tap(ushort vk) { SendInput(2, new[] { Key(vk, false), Key(vk, true) }, Marshal.SizeOf(typeof(INPUT))); Thread.Sleep(45); }
    public static void Down(ushort vk) { SendInput(1, new[] { Key(vk, false) }, Marshal.SizeOf(typeof(INPUT))); Thread.Sleep(30); }
    public static void Up(ushort vk) { SendInput(1, new[] { Key(vk, true) }, Marshal.SizeOf(typeof(INPUT))); Thread.Sleep(30); }
    public static void Chord(ushort mod, ushort vk) { Down(mod); Tap(vk); Up(mod); Thread.Sleep(150); }
    public static void Type(string s) {
        foreach (char c in s) {
            if (c == ' ') { Tap(0x20); continue; }
            if (c == '\n') { Tap(0x0D); continue; }
            if (c == '\t') { Tap(0x09); continue; }
            if (c == ',') { Tap(0xBC); continue; }
            if (c == '.') { Tap(0xBE); continue; }
            if (c == '=') { Tap(0xBB); continue; }
            if (c == '<') { Tap(0x1B); continue; }
            if (c == '\b') { Tap(0x08); continue; }
            if (c == '>') { Tap(0x23); Thread.Sleep(100); continue; }
            if (c == '~') { Chord(0x11, 0x20); continue; }
            if (char.IsDigit(c)) { Tap((ushort)c); continue; }
            ushort vk = (ushort)char.ToUpperInvariant(c);
            bool shift = char.IsUpper(c);
            if (shift) Down(0x10);
            Tap(vk);
            if (shift) Up(0x10);
        }
    }
    public static bool Focus(IntPtr h) {
        uint pid; uint fg = GetWindowThreadProcessId(GetForegroundWindow(), out pid); uint me = GetCurrentThreadId();
        AttachThreadInput(me, fg, true); ShowWindow(h, 5); BringWindowToTop(h); bool ok = SetForegroundWindow(h); AttachThreadInput(me, fg, false);
        Thread.Sleep(250); return ok && GetForegroundWindow() == h;
    }
    delegate bool EnumProc(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] static extern bool EnumChildWindows(IntPtr p, EnumProc f, IntPtr l);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
    public static IntPtr FindVisibleDescendant(IntPtr parent, string cls) {
        IntPtr found = IntPtr.Zero;
        EnumChildWindows(parent, (h, l) => {
            var sb = new StringBuilder(256); GetClassNameW(h, sb, 256);
            if (sb.ToString() == cls && IsWindowVisible(h)) { found = h; return false; }
            return true;
        }, IntPtr.Zero);
        return found;
    }
    public static IntPtr Ghost() {
        IntPtr h = IntPtr.Zero;
        while ((h = FindWindowExW(IntPtr.Zero, h, "TextVNGhostText", null)) != IntPtr.Zero) { if (IsWindowVisible(h)) return h; }
        return IntPtr.Zero;
    }
    public static string Shot(IntPtr around, string path) {
        RECT r; GetWindowRect(around, out r);
        int x = Math.Max(0, r.Left - 500), y = Math.Max(0, r.Top - 60), w = 1300, h = 220;
        using (var bmp = new System.Drawing.Bitmap(w, h)) {
            using (var g = System.Drawing.Graphics.FromImage(bmp)) { g.CopyFromScreen(x, y, 0, 0, new System.Drawing.Size(w, h)); }
            bmp.Save(path, System.Drawing.Imaging.ImageFormat.Png);
        }
        return string.Format("ghost={0},{1},{2},{3}", r.Left, r.Top, r.Right, r.Bottom);
    }
    public static bool SysMenuVisible() {
        IntPtr h = IntPtr.Zero;
        while ((h = FindWindowExW(IntPtr.Zero, h, "#32768", null)) != IntPtr.Zero) { if (IsWindowVisible(h)) return true; }
        return false;
    }
    public static string Title(IntPtr h) { var sb = new StringBuilder(4096); GetWindowTextW(h, sb, 4096); return sb.ToString(); }
    public static string WmText(IntPtr edit) {
        int n = (int)SendMessageW(edit, 0x000E, IntPtr.Zero, IntPtr.Zero);
        var sb = new StringBuilder(n + 1); SendMessageW(edit, 0x000D, (IntPtr)(n + 1), sb); return sb.ToString();
    }
    public static void WmClear(IntPtr edit) { SendMessageW(edit, 0x000C, IntPtr.Zero, new StringBuilder("")); }
    public static IntPtr Overlay() {
        IntPtr h = IntPtr.Zero;
        while ((h = FindWindowExW(IntPtr.Zero, h, "TextVNAssistOverlay", null)) != IntPtr.Zero) { if (IsWindowVisible(h)) return h; }
        return IntPtr.Zero;
    }
    // Click the "Sua tat ca" button: header row, right edge (pad 14, header 30 @96dpi).
    public static string ClickFixAll() {
        IntPtr o = Overlay(); if (o == IntPtr.Zero) return "no-overlay";
        RECT r; GetWindowRect(o, out r); double s = GetDpiForWindow(o) / 96.0; if (s <= 0) s = 1;
        int x = r.Right - (int)(39 * s), y = r.Top + (int)(25 * s);
        SetCursorPos(x, y); Thread.Sleep(120);
        mouse_event(0x0002, 0, 0, 0, IntPtr.Zero); Thread.Sleep(60); mouse_event(0x0004, 0, 0, 0, IntPtr.Zero);
        Thread.Sleep(100);
        return string.Format("rect={0},{1},{2},{3} dpi={4} click={5},{6}", r.Left, r.Top, r.Right, r.Bottom, GetDpiForWindow(o), x, y);
    }
}
'@
[D]::DpiAware()
function U([string]$s) { [regex]::Unescape($s.Replace('{u}', [string][char]92 + 'u')) }
$script:pass = 0; $script:fail = 0
function Report([string]$app, [string]$name, [bool]$ok, [string]$want, [string]$got, [string]$extra) {
    if ($ok) { $script:pass++; Write-Host "PASS [$app] $name $extra" }
    else { $script:fail++; Write-Host "FAIL [$app] $name want [$want] got [$got] $extra" }
}
$devDll = Join-Path $DevDir 'textvn_win_tsf.dll'
$devCli = Join-Path $DevDir 'textvn-cli.exe'
$hkcuInproc = 'HKCU:\Software\Classes\CLSID\{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}\InprocServer32'
$owned = New-Object System.Collections.Generic.List[int]
$trace = Join-Path $env:TEMP 'textvn-e2e-apps-trace.log'
Remove-Item $trace -ErrorAction SilentlyContinue
$env:TEXTVN_TSF_TRACE = $trace
function Get-ViEnabled { try { return [bool]((Get-Content (Join-Path $env:APPDATA 'TextVN\state.json') -Raw | ConvertFrom-Json).global_enabled) } catch { return $true } }
function Set-ViEnabled([bool]$want, [IntPtr]$focusWin) {
    for ($attempt = 0; $attempt -lt 3; $attempt++) {
        if ((Get-ViEnabled) -eq $want) { return $true }
        if ($focusWin -ne [IntPtr]::Zero) { [D]::Focus($focusWin) | Out-Null }
        [D]::Down(0x11); [D]::Down(0x10); [D]::Up(0x10); [D]::Up(0x11)
        for ($i = 0; $i -lt 10; $i++) { Start-Sleep -Milliseconds 200; if ((Get-ViEnabled) -eq $want) { return $true } }
    }
    return ((Get-ViEnabled) -eq $want)
}
$viOriginal = Get-ViEnabled

$taHtml = @'
<!doctype html>
<html><head><meta charset="utf-8"><title>[]</title></head>
<body style="font:20px Segoe UI;margin:24px">
<textarea id="ta" autofocus style="width:900px;height:300px;font:20px Segoe UI"></textarea>
<script>
const ta = document.getElementById('ta');
const upd = () => { document.title = '[' + ta.value + ']'; };
ta.addEventListener('input', upd);
setInterval(upd, 100);
ta.focus();
</script>
</body></html>
'@
Set-Content -Path (Join-Path $here 'ta.html') -Value $taHtml -Encoding ascii
# --- app adapters: each returns @{ Win; Clear; Text } ---
function Start-Notepad {
    $before = @(Get-Process -Name notepad -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
    Start-Process notepad.exe | Out-Null
    $p = $null
    for ($i = 0; $i -lt 60 -and -not $p; $i++) { Start-Sleep -Milliseconds 500; $p = Get-Process -Name notepad -ErrorAction SilentlyContinue | Where-Object { $before -notcontains $_.Id -and $_.MainWindowHandle -ne [IntPtr]::Zero } | Select-Object -First 1 }
    $owned.Add($p.Id)
    $main = $p.MainWindowHandle
    $edit = [IntPtr]::Zero
    for ($i = 0; $i -lt 20 -and $edit -eq [IntPtr]::Zero; $i++) {
        $box = [D]::FindWindowExW($main, [IntPtr]::Zero, 'NotepadTextBox', $null)
        if ($box -ne [IntPtr]::Zero) { $edit = [D]::FindWindowExW($box, [IntPtr]::Zero, 'RichEditD2DPT', $null) }
        if ($edit -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 250 }
    }
    $e = $edit
    return @{ Name = 'notepad'; Win = $main; Clear = { [D]::WmClear($e); Start-Sleep -Milliseconds 150 }.GetNewClosure(); Text = { [D]::WmText($e) }.GetNewClosure() }
}
function Start-Edge {
    $exe = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
    $prof = Join-Path $here 'edge-profile'
    $url = 'file:///' + ((Join-Path $here 'ta.html') -replace '\\', '/')
    $p = Start-Process $exe -ArgumentList @("--user-data-dir=$prof", '--no-first-run', '--no-default-browser-check', "--app=$url") -PassThru
    $win = [IntPtr]::Zero
    for ($i = 0; $i -lt 60 -and $win -eq [IntPtr]::Zero; $i++) {
        Start-Sleep -Milliseconds 500
        $cand = Get-Process -Name msedge -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero -and $_.MainWindowTitle -like '`[*' }
        if ($cand) { $win = ($cand | Select-Object -First 1).MainWindowHandle; $owned.Add(($cand | Select-Object -First 1).Id) }
    }
    $w = $win
    return @{ Name = 'edge'; Win = $win; Clear = { [D]::Chord(0x11, 0x41); [D]::Tap(0x2E); Start-Sleep -Milliseconds 250 }.GetNewClosure(); Text = { $t = [D]::Title($w); if ($t -match '^\[(.*)\]') { $Matches[1] } else { $t } }.GetNewClosure() }
}
function Start-VSCode {
    # Electron: a SEPARATE VS Code instance (own --user-data-dir/--extensions-dir under %TEMP%,
    # extensions disabled) so the user's own VS Code session is never touched. Text is read back
    # from the auto-saved file.
    $exe = $VSCodeExe
    if (-not $exe -or -not (Test-Path $exe)) { $c = Get-Command code -ErrorAction SilentlyContinue; if ($c) { $exe = Join-Path (Split-Path (Split-Path $c.Source)) 'Code.exe' } }
    if (-not $exe -or -not (Test-Path $exe)) { return $null }
    $file = Join-Path $here 'vscode.txt'
    Set-Content -Path $file -Value '' -NoNewline -Encoding ascii
    # Auto-save so the text is read from the file WITHOUT a keystroke (Ctrl+S is a chord:
    # TextVN forgets its sentence context on chords, which would break "fix then predict").
    $userDir = Join-Path (Join-Path $here 'vscode-profile') 'User'
    New-Item -ItemType Directory -Force $userDir | Out-Null
    Set-Content -Path (Join-Path $userDir 'settings.json') -Encoding ascii -Value '{ "files.autoSave": "afterDelay", "files.autoSaveDelay": 50, "editor.quickSuggestions": { "other": false, "comments": false, "strings": false }, "editor.wordBasedSuggestions": "off", "workbench.startupEditor": "none" }'
    $args2 = @("--user-data-dir=$(Join-Path $here 'vscode-profile')", "--extensions-dir=$(Join-Path $here 'vscode-ext')", '--disable-extensions', '--disable-workspace-trust', '--skip-welcome', '--skip-release-notes', '--new-window', $file)
    # A shell started from an Electron host (VS Code terminal / extension) inherits
    # ELECTRON_RUN_AS_NODE + VSCODE_*: Code.exe would then run as plain Node and exit at once.
    $saved = @{}
    Get-ChildItem env: | Where-Object { $_.Name -eq 'ELECTRON_RUN_AS_NODE' -or $_.Name -like 'VSCODE_*' } | ForEach-Object { $saved[$_.Name] = $_.Value; Remove-Item ("env:" + $_.Name) }
    try { Start-Process $exe -ArgumentList $args2 | Out-Null }
    finally { foreach ($k in $saved.Keys) { Set-Item ("env:" + $k) $saved[$k] } }
    $win = [IntPtr]::Zero
    for ($i = 0; $i -lt 80 -and $win -eq [IntPtr]::Zero; $i++) {
        Start-Sleep -Milliseconds 500
        $cand = Get-Process -Name Code -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero -and $_.MainWindowTitle -like 'vscode.txt*' } | Select-Object -First 1
        if ($cand) { $win = $cand.MainWindowHandle }
    }
    Start-Sleep -Milliseconds 1500
    $f = $file
    return @{ Name = 'vscode'; Win = $win; Clear = { [D]::Chord(0x11, 0x41); [D]::Tap(0x2E); Start-Sleep -Milliseconds 250 }.GetNewClosure(); Text = { [System.IO.File]::ReadAllText($f, [System.Text.Encoding]::UTF8) }.GetNewClosure() }
}
function Start-Probe([string]$mode) {
    # 32-bit host (tools/win/x86probe): exercises textvn-tsf-x86.dll (Zalo/Office 32-bit).
    # rich = RichEdit50W (TSF-aware), edit = EDIT (IMM32 via CUAS - WinForms/Delphi family).
    if (-not $X86Probe -or -not (Test-Path $X86Probe)) { return $null }
    $title = if ($mode -eq 'edit') { 'TextVN x86 probe EDIT' } else { 'TextVN x86 probe RICH' }
    $cls = if ($mode -eq 'edit') { 'Edit' } else { 'RICHEDIT50W' }
    $pr = Start-Process $X86Probe -ArgumentList $mode -PassThru
    $owned.Add($pr.Id)
    $main = [IntPtr]::Zero
    for ($i = 0; $i -lt 30 -and $main -eq [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 300; $pr.Refresh(); $main = $pr.MainWindowHandle }
    $ed = [D]::FindWindowExW($main, [IntPtr]::Zero, $cls, $null)
    $e = $ed
    return @{ Name = ('x86' + $mode); Win = $main; Clear = { [D]::WmClear($e); Start-Sleep -Milliseconds 150 }.GetNewClosure(); Text = { [D]::WmText($e) }.GetNewClosure() }
}
function Start-Npp {
    $exe = 'C:\Program Files\Notepad++\notepad++.exe'
    $p = Start-Process $exe -ArgumentList @('-multiInst', '-nosession', '-notabbar') -PassThru
    $owned.Add($p.Id)
    $main = [IntPtr]::Zero
    for ($i = 0; $i -lt 40 -and $main -eq [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 500; $p.Refresh(); $main = $p.MainWindowHandle }
    $sci = [IntPtr]::Zero
    for ($i = 0; $i -lt 20 -and $sci -eq [IntPtr]::Zero; $i++) { $sci = [D]::FindVisibleDescendant($main, 'Scintilla'); if ($sci -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 250 } }
    Write-Host ("npp scintilla=" + $sci)
    $s = $sci
    return @{ Name = 'npp'; Win = $main; Clear = { [D]::Chord(0x11, 0x41); [D]::Tap(0x2E); Start-Sleep -Milliseconds 200 }.GetNewClosure(); Text = { [D]::WmText($s) }.GetNewClosure() }
}

function Run-Case($app, [string]$name, [scriptblock]$body) {
    if ($Only -and $name -notlike "*$Only*") { return }
    for ($i = 0; $i -lt 10 -and -not [D]::Focus($app.Win); $i++) { Start-Sleep -Milliseconds 300 }
    & $app.Clear
    [D]::Type('>')
    Add-Content -Path $trace -Value "=== CASE [$($app.Name)] $name" -ErrorAction SilentlyContinue
    & $body
    [D]::Type('<'); Start-Sleep -Milliseconds 200
}
function WaitText($app, [string]$want, [int]$ms = 2500) {
    $got = $null
    for ($t = 0; $t -lt $ms; $t += 200) { Start-Sleep -Milliseconds 200; $got = (& $app.Text) -replace "`r`n", "`n" -replace "`r", "`n"; if ($got -ceq $want) { return @($true, $got) } }
    return @($false, $got)
}

try {
    if (-not $NoRegister) {
        & $devCli register --scope user --dll $devDll | Out-Host
        $reg = (Get-ItemProperty $hkcuInproc).'(default)'
        if ($reg -ne $devDll) { throw "HKCU override not pointing to dev DLL (got '$reg')" }
    }
    foreach ($appName in $Apps) {
        $app = switch ($appName) { 'notepad' { Start-Notepad } 'edge' { Start-Edge } 'npp' { Start-Npp } 'vscode' { Start-VSCode } 'x86rich' { Start-Probe 'rich' } 'x86edit' { Start-Probe 'edit' } }
        if (-not $app -or $app.Win -eq [IntPtr]::Zero) { Write-Host "SKIP $appName (no window)"; continue }
        [D]::Focus($app.Win) | Out-Null; [D]::Type('a'); Start-Sleep -Milliseconds 1200; & $app.Clear
        # The cases need Vietnamese mode (V); the user's own mode is restored after each app.
        if (-not (Set-ViEnabled $true $app.Win)) { throw 'could not switch TextVN to Vietnamese mode for the test' }
        $n = $app.Name

        $fixed = U 't{u}00f4i {u}0111i h{u}1ecdc '
        Run-Case $app 'live popup visible' {
            [D]::Type('tooi tooi ddi hocj '); Start-Sleep -Milliseconds 700
            $ov = [D]::Overlay() -ne [IntPtr]::Zero
            Report $n 'live popup visible on error' $ov 'overlay' "overlay=$ov" ''
        }
        Run-Case $app 'click fix-all' {
            [D]::Type('tooi tooi ddi hocj '); Start-Sleep -Milliseconds 700
            $info = [D]::ClickFixAll()
            $r = WaitText $app $fixed
            Report $n 'click Sua tat ca' $r[0] $fixed $r[1] $info
        }
        Run-Case $app 'alt+1' {
            [D]::Type('tooi tooi ddi hocj '); Start-Sleep -Milliseconds 700
            $ov = [D]::Overlay() -ne [IntPtr]::Zero
            [D]::Chord(0x12, 0x31)
            $r = WaitText $app $fixed
            Report $n 'Alt+1 applies suggestion' $r[0] $fixed $r[1] "overlay=$ov"
        }
        Run-Case $app 'tab prediction' {
            [D]::Type("phats trieenr`t")
            $w = U 'ph{u}00e1t tri{u}1ec3n ph{u}1ea7n m{u}1ec1m'
            $r = WaitText $app $w
            Report $n 'Tab prediction phat trien' $r[0] $w $r[1] ''
        }
        $fixedNoSp = U 't{u}00f4i {u}0111i h{u}1ecdc'
        Run-Case $app 'hotkey ctrl+space' {
            [D]::Type('tooi tooi ddi hocj'); Start-Sleep -Milliseconds 300
            [D]::Type('~')
            $r = WaitText $app $fixedNoSp
            Report $n 'Ctrl+Space fixes whole sentence' $r[0] $fixedNoSp $r[1] ''
        }
        Run-Case $app 'alt+space not taken' {
            # R2-136 (owner decision): Alt+Space is NOT a TextVN key - even with errors pending it
            # must reach Windows (window menu) and the text must stay as typed.
            [D]::Type('tooi tooi ddi hocj '); Start-Sleep -Milliseconds 500
            [D]::Chord(0x12, 0x20)
            $menu = $false
            for ($t = 0; $t -lt 1500 -and -not $menu; $t += 100) { Start-Sleep -Milliseconds 100; $menu = [D]::SysMenuVisible() }
            if ($menu) { [D]::Tap(0x1B); Start-Sleep -Milliseconds 300 }
            $w = U 't{u}00f4i t{u}00f4i {u}0111i h{u}1ecdc '
            $r = WaitText $app $w
            Report $n 'Alt+Space with errors pending opens the window menu, text unchanged (R2-136)' ($menu -and $r[0]) "sysmenu + $w" ("sysmenu=$menu text=" + $r[1]) ''
        }
        Run-Case $app 'hotkey fix then predict' {
            [D]::Type('tooi tooi phats trieenr'); Start-Sleep -Milliseconds 300
            [D]::Type('~')
            $w1 = U 't{u}00f4i ph{u}00e1t tri{u}1ec3n'
            $r1 = WaitText $app $w1
            [D]::Type('~')
            $w2 = U 't{u}00f4i ph{u}00e1t tri{u}1ec3n ph{u}1ea7n m{u}1ec1m'
            $r2 = WaitText $app $w2
            Report $n 'hotkey: fix first, then predict' ($r1[0] -and $r2[0]) "$w1 => $w2" ($r1[1] + ' => ' + $r2[1]) ''
        }
        Run-Case $app 'alt+space idle sysmenu' {
            Start-Sleep -Milliseconds 300
            [D]::Chord(0x12, 0x20)
            $menu = $false
            for ($t = 0; $t -lt 1500 -and -not $menu; $t += 100) { Start-Sleep -Milliseconds 100; $menu = [D]::SysMenuVisible() }
            if ($menu) { [D]::Tap(0x1B); Start-Sleep -Milliseconds 200 }
            Report $n 'Alt+Space with nothing to do opens the system menu' $menu 'sysmenu' "sysmenu=$menu" ''
        }
        Run-Case $app 'ctrl+shift+g interactive' {
            [D]::Type('tooi tooi ddi hocj '); Start-Sleep -Milliseconds 500
            [D]::Down(0x11); [D]::Down(0x10); [D]::Tap(0x47); [D]::Up(0x10); [D]::Up(0x11); Start-Sleep -Milliseconds 500
            $ov = [D]::Overlay() -ne [IntPtr]::Zero
            [D]::Type("`t"); Start-Sleep -Milliseconds 300
            $ov2 = [D]::Overlay() -ne [IntPtr]::Zero
            [D]::Type('<'); Start-Sleep -Milliseconds 300
            $w = U 't{u}00f4i t{u}00f4i {u}0111i h{u}1ecdc '
            $r = WaitText $app $w
            Report $n 'Ctrl+Shift+G popup: Tab stays in popup (R2-122)' ($ov -and $ov2 -and $r[0]) $w $r[1] "overlay=$ov afterTab=$ov2"
        }
        Run-Case $app 'ghost text' {
            [D]::Type('phats trieenr'); Start-Sleep -Milliseconds 400
            $g = [D]::Ghost(); $vis = $g -ne [IntPtr]::Zero
            $info = ''
            if ($vis -and $Shots) { $info = [D]::Shot($g, (Join-Path $here ("ghost-" + $n + ".png"))) }
            [D]::Type("`t"); Start-Sleep -Milliseconds 300
            $after = [D]::Ghost() -ne [IntPtr]::Zero
            $w = U 'ph{u}00e1t tri{u}1ec3n ph{u}1ea7n m{u}1ec1m'
            $r = WaitText $app $w
            # VS Code/Monaco: by design no ghost (hidden textarea does not reflect the text after the
            # caret; the editor has its own inline suggestions) - Tab prediction must still work.
            $wantGhost = $n -ne 'vscode'
            Report $n 'ghost text before Tab (not in VS Code), gone after Tab' (($vis -eq $wantGhost) -and -not $after -and $r[0]) "ghost=$wantGhost then $w" ("ghost=$vis after=$after text=" + $r[1]) $info
        }
        Run-Case $app 'plain typing' {
            [D]::Type('Vieetj Nam ')
            $w = U 'Vi{u}1ec7t Nam '
            $r = WaitText $app $w
            Report $n 'plain Vietnamese typing' $r[0] $w $r[1] ''
        }
        if (-not $viOriginal) { Set-ViEnabled $false $app.Win | Out-Null }
    }
}
finally {
    foreach ($id in $owned) { Stop-Process -Id $id -Force -ErrorAction SilentlyContinue }
    foreach ($exeName in @('msedge.exe', 'Code.exe')) {
        Get-CimInstance Win32_Process -Filter "Name='$exeName'" -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -like '*textvn-e2e-apps*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
    }
    Start-Sleep -Milliseconds 500
    if (-not $NoRegister) {
        & $ProdCli register | Out-Host
        $after = (Get-ItemProperty $hkcuInproc -ErrorAction SilentlyContinue).'(default)'
        Write-Host ("restore: HKCU override after = '" + $after + "' (expected empty)")
    }
    if (Test-Path $trace) { Write-Host '--- trace (assist/preserved lines) ---'; Get-Content $trace | Where-Object { $_ -match 'assist|preserved|CASE' } | Select-Object -Last 40 }
    Write-Host ("SUMMARY pass=" + $script:pass + " fail=" + $script:fail)
}
