# SPDX-License-Identifier: GPL-3.0-or-later
# test-typing.ps1 -Dir <thu muc chua TextVN.exe, textvn-cli.exe, textvn-tsf.dll>
#
# Go tieng Viet THAT qua TSF: mo Notepad, gui phim bang SendInput (VK + scan code nhu ban
# phim that - KHONG dung KEYEVENTF_UNICODE vi TIP bo qua VK_PACKET), doc lai noi dung o
# soan thao va so voi ky vong. Dung chung cho kich ban portable va kich ban cai dat.
# Chi ghi ket qua dung/sai + ma ky tu (S2: khong ghi text nguoi dung that).

param(
    [Parameter(Mandatory = $true)][string]$Dir
)
$ErrorActionPreference = 'Stop'

if (-not ('TvKeys' -as [type])) {
    Add-Type @'
using System;
using System.Text;
using System.Threading;
using System.Runtime.InteropServices;
public static class TvKeys {
    [StructLayout(LayoutKind.Sequential)]
    struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Explicit, Size = 40)]
    struct INPUT { [FieldOffset(0)] public uint type; [FieldOffset(8)] public KEYBDINPUT ki; }
    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] p, int cb);
    [DllImport("user32.dll")] static extern uint MapVirtualKeyW(uint code, uint type);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool f);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] public static extern IntPtr GetKeyboardLayout(uint tid);
    [DllImport("user32.dll")] public static extern short GetKeyState(int vk);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowExW(IntPtr parent, IntPtr after, string cls, string title);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, StringBuilder l);
    [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, IntPtr l);

    const uint INPUT_KEYBOARD = 1, KEYEVENTF_KEYUP = 2, KEYEVENTF_EXTENDEDKEY = 1;

    static INPUT Key(ushort vk, bool up) {
        var i = new INPUT();
        i.type = INPUT_KEYBOARD;
        i.ki.wVk = vk;
        i.ki.wScan = (ushort)MapVirtualKeyW(vk, 0);
        i.ki.dwFlags = up ? KEYEVENTF_KEYUP : 0;
        return i;
    }

    public static void Tap(ushort vk) {
        if (SendInput(2, new[] { Key(vk, false), Key(vk, true) }, Marshal.SizeOf(typeof(INPUT))) != 2)
            throw new InvalidOperationException("SendInput failed");
        Thread.Sleep(40);
    }

    public static void Down(ushort vk) { if (SendInput(1, new[] { Key(vk, false) }, Marshal.SizeOf(typeof(INPUT))) != 1) throw new InvalidOperationException("SendInput failed"); Thread.Sleep(30); }
    public static void Up(ushort vk) { if (SendInput(1, new[] { Key(vk, true) }, Marshal.SizeOf(typeof(INPUT))) != 1) throw new InvalidOperationException("SendInput failed"); Thread.Sleep(30); }

    // Go chuoi ASCII: chu hoa = Shift + phim (tru khi Caps Lock dang bat), ' ' = Space,
    // '\n' = Enter, '\t' = Tab, ',' '.' = phim dau cau (layout US), '^' = Home.
    public static void Type(string s) {
        bool caps = (GetKeyState(0x14) & 1) != 0;
        foreach (char c in s) {
            if (c == ' ') { Tap(0x20); continue; }
            if (c == '\n') { Tap(0x0D); continue; }
            if (c == '\t') { Tap(0x09); continue; }
            if (c == ',') { Tap(0xBC); continue; }
            if (c == '.') { Tap(0xBE); continue; }
            if (c == '^') { Tap(0x24); continue; }  // Home
            ushort vk = (ushort)char.ToUpperInvariant(c);
            bool shift = char.IsUpper(c) != caps && char.IsLetter(c);
            if (shift) Down(0x10);
            Tap(vk);
            if (shift) Up(0x10);
        }
    }

    // Ctrl+Shift nhan roi nha (phim chuyen kieu UniKey).
    public static void CtrlShiftTap() { Down(0x11); Down(0x10); Up(0x10); Up(0x11); Thread.Sleep(80); }

    public static bool Focus(IntPtr h) {
        uint pid;
        uint fgThread = GetWindowThreadProcessId(GetForegroundWindow(), out pid);
        uint me = GetCurrentThreadId();
        AttachThreadInput(me, fgThread, true);
        ShowWindow(h, 5);
        BringWindowToTop(h);
        bool ok = SetForegroundWindow(h);
        AttachThreadInput(me, fgThread, false);
        Thread.Sleep(200);
        return ok && GetForegroundWindow() == h;
    }

    public static IntPtr EditOf(IntPtr main) {
        foreach (var cls in new[] { "Edit", "RichEditD2DPT", "RICHEDIT50W" }) {
            IntPtr e = FindWindowExW(main, IntPtr.Zero, cls, null);
            if (e != IntPtr.Zero) return e;
        }
        return IntPtr.Zero;
    }

    public static string Text(IntPtr edit) {
        int n = (int)SendMessageW(edit, 0x000E /*WM_GETTEXTLENGTH*/, IntPtr.Zero, IntPtr.Zero);
        var sb = new StringBuilder(n + 1);
        SendMessageW(edit, 0x000D /*WM_GETTEXT*/, (IntPtr)(n + 1), sb);
        return sb.ToString();
    }

    public static void Clear(IntPtr edit) {
        SendMessageW(edit, 0x000C /*WM_SETTEXT*/, IntPtr.Zero, new StringBuilder(""));
    }
}
'@
}

function U([string]$s) { [regex]::Unescape($s) }
function Codes([string]$s) { ($s.ToCharArray() | ForEach-Object { 'U+{0:X4}' -f [int]$_ }) -join ' ' }

$cli = Join-Path $Dir 'textvn-cli.exe'
$tray = Join-Path $Dir 'TextVN.exe'
foreach ($f in @($cli, $tray)) { if (-not (Test-Path $f)) { throw "missing $f" } }

# Smoke test thay doi input source va mode; khong dung instance cua nguoi dung.
if (Get-Process -Name TextVN,notepad,wordpad -ErrorAction SilentlyContinue) {
    throw 'Close existing TextVN/Notepad/WordPad before running the destructive typing smoke test'
}
$ownedTray = Start-Process -FilePath $tray -ArgumentList '--autostart' -WorkingDirectory $Dir -WindowStyle Hidden -PassThru
$script:ownedEditor = $null
try {
$ready = $false
for ($i = 0; $i -lt 30 -and -not $ready; $i++) {
    Start-Sleep -Milliseconds 500
    $ready = ((& $tray --status 2>&1 | Out-String) -match 'RUNNING')
}
if (-not $ready) { throw 'TextVN tray did not start' }

# Kich hoat profile TextVN cho phien (giong nguoi dung chon TextVN o thanh ngon ngu).
& $cli register | Out-Host
if ($LASTEXITCODE -ne 0) { throw "TextVN TSF registration failed ($LASTEXITCODE)" }
# "Danh Ctrl + Shift cho TextVN" (installer mac dinh chon; ban portable: o trong Bang dieu
# khien): phim tat doi bo cuc cua Windows khong con nuot Ctrl+Shift.
& $tray --free-ctrl-shift | Out-Host
if ($LASTEXITCODE -ne 0) { throw "TextVN Ctrl+Shift configuration failed ($LASTEXITCODE)" }

$cases = @(
    @{ name = 'telex dduocj';      keys = 'dduocj ';       want = (U '\u0111\u01b0\u1ee3c ') },
    @{ name = 'Vieetj Nam';        keys = 'Vieetj Nam ';   want = (U 'Vi\u1ec7t Nam ') },
    @{ name = 'nguowif (uow)';     keys = 'nguowif ';      want = (U 'ng\u01b0\u1eddi ') },
    @{ name = 'cuar (tone rule)';  keys = 'cuar ';         want = (U 'c\u1ee7a ') },
    @{ name = 'english hello';     keys = 'hello ';        want = 'hello ' },
    @{ name = 'comma boundary';    keys = 'Vieetj, Nam ';  want = (U 'Vi\u1ec7t, Nam ') },
    # Enter phai toi app nhu phim that; sau Enter chu dau cau tu viet hoa (mac dinh).
    @{ name = 'Enter boundary';    keys = "chaof`nbanj ";  want = (U "ch\u00e0o`nB\u1ea1n ") },
    @{ name = 'Tab boundary';      keys = "tieengs`tx ";   want = (U "ti\u1ebfng`tx ") },
    # Phim dieu huong khi dang go: tu duoc chot tai cho, Home di dung ve dau dong.
    @{ name = 'Home mid-word';     keys = 'chaof^x ';      want = (U 'x ch\u00e0o') }
)

$script:fail = 0
$script:results = New-Object System.Collections.Generic.List[string]
# Edit tra "\r\n", RichEdit tra "\r": quy ve "\n" truoc khi so.
function Norm([string]$s) { ($s -replace "`r`n", "`n") -replace "`r", "`n" }
function Check($app, [string]$name, [string]$want) {
    Start-Sleep -Milliseconds 400
    $got = Norm ([TvKeys]::Text($app.Edit))
    if ($got -ceq $want) {
        $line = "PASS [{0}] {1}" -f $app.Name, $name
    } else {
        $line = "FAIL [{0}] {1}: want [{2}] got [{3}]" -f $app.Name, $name, (Codes $want), (Codes $got)
        $script:fail++
    }
    Write-Host $line
    $script:results.Add($line)
    [TvKeys]::Clear($app.Edit)
    $null = [TvKeys]::Focus($app.Main)
}

function Open-App([string]$name, [string]$exe) {
    $p = Start-Process $exe -PassThru
    $script:ownedEditor = $p
    $main = [IntPtr]::Zero
    # Never attach to another user's document if the newly started process
    # hands off to an existing editor instance.
    for ($i = 0; $i -lt 60 -and $main -eq [IntPtr]::Zero; $i++) {
        Start-Sleep -Milliseconds 500
        $p.Refresh()
        $main = $p.MainWindowHandle
    }
    if ($main -eq [IntPtr]::Zero) { throw "$name window not found after 30 s" }
    $edit = [IntPtr]::Zero
    for ($i = 0; $i -lt 20 -and $edit -eq [IntPtr]::Zero; $i++) {
        $edit = [TvKeys]::EditOf($main)
        if ($edit -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 250 }
    }
    if ($edit -eq [IntPtr]::Zero) { throw "$name edit control not found" }
    @{ Name = $name; Proc = $p; Main = $main; Edit = $edit }
}

# Notepad (Win32 Edit, IMM32 qua CUAS) va WordPad (RichEdit, TSF-aware) neu co.
$apps = @(@{ Name = 'notepad'; Exe = 'notepad.exe' })
$wordpad = Join-Path $env:ProgramFiles 'Windows NT\Accessories\wordpad.exe'
if (Test-Path $wordpad) { $apps += @{ Name = 'wordpad'; Exe = $wordpad } }

# Nhat ky chan doan cua key sink (khong chua noi dung go) - app ke thua bien moi truong.
$trace = Join-Path $env:TEMP 'textvn-tsf-trace.log'
Remove-Item $trace -ErrorAction SilentlyContinue
$env:TEXTVN_TSF_TRACE = $trace

foreach ($a in $apps) {
    Add-Content -Path $trace -Value ("=== " + $a.Name)
    $app = Open-App $a.Name $a.Exe
    $null = [TvKeys]::Focus($app.Main)
    [TvKeys]::Clear($app.Edit)

    foreach ($c in $cases) {
        $null = [TvKeys]::Focus($app.Main)
        [TvKeys]::Type($c.keys)
        Check $app $c.name $c.want
    }

    # Ctrl+Shift: tat tieng Viet roi bat lai.
    [TvKeys]::CtrlShiftTap()
    [TvKeys]::Type('as ')
    Check $app 'Ctrl+Shift -> EN' 'as '
    [TvKeys]::CtrlShiftTap()
    [TvKeys]::Type('as ')
    Check $app 'Ctrl+Shift -> VN' (U '\u00e1 ')

    # Caps Lock: phim dau viet hoa van la phim dau.
    [TvKeys]::Tap(0x14)
    [TvKeys]::Type('VIEETJ ')
    [TvKeys]::Tap(0x14)
    Check $app 'Caps Lock VIEETJ' (U 'VI\u1ec6T ')

    if ($script:fail -gt 0) {
        $npPid = [uint32]0
        $npTid = [TvKeys]::GetWindowThreadProcessId($app.Main, [ref]$npPid)
        Write-Host ('{0} HKL = 0x{1:X}' -f $app.Name, [TvKeys]::GetKeyboardLayout($npTid).ToInt64())
    }
    Stop-Process -Id $app.Proc.Id -Force -ErrorAction SilentlyContinue
    $script:ownedEditor = $null
}

if ($script:fail -gt 0) {
    Write-Host '--- diagnostics'
    & $cli doctor 2>&1 | Out-Host
    if (Test-Path $trace) {
        Write-Host '--- TSF key trace (TEXTVN_TSF_TRACE, chi phim dieu khien)'
        Get-Content $trace | Where-Object { $_ -notmatch ' chr ' } | Select-Object -Last 150 | Out-Host
    }
    Write-Host '--- summary'
    $script:results | Out-Host
    throw "$($script:fail) typing case(s) failed"
}
Write-Host 'typing: OK'
} finally {
    if ($script:ownedEditor) {
        Stop-Process -Id $script:ownedEditor.Id -Force -ErrorAction SilentlyContinue
    }
    & $tray --stop | Out-Null
}
