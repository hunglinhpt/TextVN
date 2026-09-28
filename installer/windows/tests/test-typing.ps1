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
        SendInput(2, new[] { Key(vk, false), Key(vk, true) }, Marshal.SizeOf(typeof(INPUT)));
        Thread.Sleep(40);
    }

    public static void Down(ushort vk) { SendInput(1, new[] { Key(vk, false) }, Marshal.SizeOf(typeof(INPUT))); Thread.Sleep(30); }
    public static void Up(ushort vk) { SendInput(1, new[] { Key(vk, true) }, Marshal.SizeOf(typeof(INPUT))); Thread.Sleep(30); }

    // Go chuoi ASCII: chu hoa = Shift + phim (tru khi Caps Lock dang bat), ' ' = Space.
    public static void Type(string s) {
        bool caps = (GetKeyState(0x14) & 1) != 0;
        foreach (char c in s) {
            if (c == ' ') { Tap(0x20); continue; }
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

# Tray (IPC, trang thai bat/tat) - tu dang ky TSF per-user neu can.
if (-not (Get-Process -Name TextVN -ErrorAction SilentlyContinue)) {
    Start-Process -FilePath $tray -ArgumentList '--autostart' -WorkingDirectory $Dir | Out-Null
}
$ready = $false
for ($i = 0; $i -lt 30 -and -not $ready; $i++) {
    Start-Sleep -Milliseconds 500
    $ready = ((& $tray --status 2>&1 | Out-String) -match 'RUNNING')
}
if (-not $ready) { throw 'TextVN tray did not start' }

$np = Start-Process notepad.exe -PassThru
$main = [IntPtr]::Zero
for ($i = 0; $i -lt 40 -and $main -eq [IntPtr]::Zero; $i++) {
    Start-Sleep -Milliseconds 250
    $np.Refresh()
    $main = $np.MainWindowHandle
}
if ($main -eq [IntPtr]::Zero) { throw 'Notepad window not found' }
$edit = [TvKeys]::EditOf($main)
if ($edit -eq [IntPtr]::Zero) { throw 'Notepad edit control not found' }

# Kich hoat profile TextVN cho phien (giong nguoi dung chon TextVN o thanh ngon ngu).
& $cli register | Out-Host
$null = [TvKeys]::Focus($main)

$cases = @(
    @{ name = 'telex dduocj';      keys = 'dduocj ';       want = (U '\u0111\u01b0\u1ee3c ') },
    @{ name = 'Vieetj Nam';        keys = 'Vieetj Nam ';   want = (U 'Vi\u1ec7t Nam ') },
    @{ name = 'nguowif (uow)';     keys = 'nguowif ';      want = (U 'ng\u01b0\u1eddi ') },
    @{ name = 'cuar (tone rule)';  keys = 'cuar ';         want = (U 'c\u1ee7a ') },
    @{ name = 'english hello';     keys = 'hello ';        want = 'hello ' }
)

$fail = 0
function Check([string]$name, [string]$want) {
    Start-Sleep -Milliseconds 400
    $got = [TvKeys]::Text($edit)
    if ($got -ceq $want) {
        Write-Host ("PASS {0}" -f $name)
    } else {
        Write-Host ("FAIL {0}: want [{1}] got [{2}]" -f $name, (Codes $want), (Codes $got))
        $script:fail++
    }
    [TvKeys]::Clear($edit)
    $null = [TvKeys]::Focus($main)
}

foreach ($c in $cases) {
    $null = [TvKeys]::Focus($main)
    [TvKeys]::Type($c.keys)
    Check $c.name $c.want
}

# Ctrl+Shift: tat tieng Viet roi bat lai.
[TvKeys]::CtrlShiftTap()
[TvKeys]::Type('as ')
Check 'Ctrl+Shift -> EN' 'as '
[TvKeys]::CtrlShiftTap()
[TvKeys]::Type('as ')
Check 'Ctrl+Shift -> VN' (U '\u00e1 ')

# Caps Lock: phim dau viet hoa van la phim dau.
[TvKeys]::Tap(0x14)
[TvKeys]::Type('VIEETJ ')
[TvKeys]::Tap(0x14)
Check 'Caps Lock VIEETJ' (U 'VI\u1ec6T ')

if ($fail -gt 0) {
    Write-Host '--- diagnostics'
    $npPid = [uint32]0
    $npTid = [TvKeys]::GetWindowThreadProcessId($main, [ref]$npPid)
    Write-Host ('notepad HKL = 0x{0:X}' -f [TvKeys]::GetKeyboardLayout($npTid).ToInt64())
    & $cli doctor 2>&1 | Out-Host
}

Stop-Process -Id $np.Id -Force -ErrorAction SilentlyContinue
if ($fail -gt 0) { throw "$fail typing case(s) failed" }
Write-Host 'typing: OK'
