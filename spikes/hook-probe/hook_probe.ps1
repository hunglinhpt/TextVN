# SPDX-License-Identifier: GPL-3.0-or-later
# WIN-005 - hook spike (local + GHA windows-latest): WH_KEYBOARD_LL + SendInput + UIA, kiem tra session (RW5).
# Log chi ghi vk + so lan (S2: khong ghi text/nguoi dung).
$ErrorActionPreference = 'Stop'
$out = Join-Path $env:TEMP 'hook_probe_out.txt'
if (Test-Path $out) { Remove-Item $out -Force }
function Log([string]$m) { Write-Output $m; Add-Content -Path $out -Value $m }

# G15: dot-source lib dung chung (Add-Type UIA + focus helper - truoc do trung lap trong HK.Focus)
. (Join-Path $PSScriptRoot '..\..\tools\win\lib\win32-uia.lib.ps1')
Initialize-TextVNUiA
Add-Type @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public class HK {
    public delegate IntPtr LowLevelProc(int nCode, IntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] public static extern IntPtr SetWindowsHookExW(int idHook, LowLevelProc lpfn, IntPtr hMod, uint dwThreadId);
    [DllImport("user32.dll")] public static extern bool UnhookWindowsHookEx(IntPtr hhk);
    [DllImport("user32.dll")] public static extern IntPtr CallNextHookEx(IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);
    [StructLayout(LayoutKind.Sequential)]
    public struct KBDLLHOOKSTRUCT { public uint vkCode; public uint scanCode; public uint flags; public uint time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Sequential)]
    public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Sequential)]
    public struct INPUT { public uint type; public KEYBDINPUT ki; public IntPtr _unionTail; }
    [DllImport("user32.dll", SetLastError = true)] public static extern uint SendInput(uint n, INPUT[] p, int cb);
    [DllImport("kernel32.dll")] public static extern uint WTSGetActiveConsoleSessionId();

    public static List<uint> Recv = new List<uint>();
    public static int HookCount = 0;
    public static IntPtr Hook = IntPtr.Zero;
    public static LowLevelProc Proc;   // giu ref - khong bi GC

    public static IntPtr Callback(int nCode, IntPtr wParam, IntPtr lParam) {
        if (nCode >= 0) {
            var k = Marshal.PtrToStructure<KBDLLHOOKSTRUCT>(lParam);
            lock (Recv) { Recv.Add(k.vkCode); HookCount++; }
        }
        return CallNextHookEx(Hook, nCode, wParam, lParam);
    }

}
'@

Log ("hook spike local - " + (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'))

# === 1) Session check (RW5 context) ===
$fg = [VtFocus]::GetForegroundWindow()
$fgpid = 0; [void][VtFocus]::GetWindowThreadProcessId($fg, [ref]$fgpid)
$session = (Get-Process -Id $PID).SessionId
Log ("interactive session: GetForegroundWindow=" + $fg.ToInt64() + " fgpid=$fgpid (pid=$PID session=$session)")
Log ("WTSGetActiveConsoleSessionId=" + [HK]::WTSGetActiveConsoleSessionId() + " UserInteractive=" + [Environment]::UserInteractive + " user=" + ([Security.Principal.WindowsIdentity]::GetCurrent().Name))
Log ("desktop window class (neu 0/blocked -> session 0 khac user): " + $(if ($fg -eq [IntPtr]::Zero) { 'KHONG CO FOREGROUND' } else { 'co' }))

# === 2) Install WH_KEYBOARD_LL (khong can dll rieng - callback trong process) ===
# (G15) bo dong chet $mod = [HK]::GetCurrentThreadId() - khong ai dung, member da chuyen sang lib
[HK]::Proc = [HK+LowLevelProc]{ param($nc, $wp, $lp) [HK]::Callback($nc, $wp, $lp) }
$sw = [System.Diagnostics.Stopwatch]::StartNew()
[HK]::Hook = [HK]::SetWindowsHookExW(13, [HK]::Proc, [IntPtr]::Zero, 0)   # 13 = WH_KEYBOARD_LL
$sw.Stop()
Log ("SetWindowsHookExW(WH_KEYBOARD_LL) = " + [HK]::Hook.ToInt64() + " (" + [math]::Round($sw.Elapsed.TotalMilliseconds,1) + "ms) lastErr=" + [Runtime.InteropServices.Marshal]::GetLastWin32Error())
if ([HK]::Hook -eq [IntPtr]::Zero) { Log 'FAIL: khong cai duoc hook'; exit 1 }

# LL hook dua vao message queue cua thread cai -> PHAI pump message (DoEvents) moi khi cho,
# khong he thong se gom hook am tham (LowLevelHooksTimeout ~300ms)
Add-Type -AssemblyName System.Windows.Forms
function PumpSleep([int]$ms) {
    $end = [DateTime]::UtcNow.AddMilliseconds($ms)
    while ([DateTime]::UtcNow -lt $end) {
        [System.Windows.Forms.Application]::DoEvents()
        Start-Sleep -Milliseconds 10
    }
}
# doc do dai text qua UIA (chi tra ve LENGTH - khong log noi dung, S2)
# -1 = element null; -2 = khong co pattern nao doc duoc
function ReadLen($el) {
    if (-not $el) { return -1 }
    try {
        $vp = $el.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
        if ($vp) { $v = $vp.Current.Value; if ($null -ne $v) { return $v.Length }; return -1 }
    } catch { }
    try {
        $tp = $el.GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern)
        if ($tp) { $t = $tp.DocumentRange.GetText(256); if ($null -ne $t) { return $t.Length }; return -1 }
    } catch { }
    return -2
}

# === 3) SendInput -> hook nhan? (target: cua so minh dang focus - khong phim vao app cua user) ===
$kiD = New-Object HK+KEYBDINPUT; $kiD.wVk = 0x41   # 'A'
$kiU = New-Object HK+KEYBDINPUT; $kiU.wVk = 0x41; $kiU.dwFlags = 2
$d = New-Object HK+INPUT; $d.type = 1; $d.ki = $kiD
$u = New-Object HK+INPUT; $u.type = 1; $u.ki = $kiU
$sz = [Runtime.InteropServices.Marshal]::SizeOf([type][HK+INPUT])

# tao cua so test rieng (message-only + 1 form nho) de phim khong vao app cua user
$f = New-Object System.Windows.Forms.Form
$f.Text = 'hook-spike-target'
$f.Width = 300; $f.Height = 100
$f.Show()
PumpSleep 500
$fh = $f.Handle
[void](Focus-UiAWindow $fh)
PumpSleep 400
Log ("focus truoc send: hwnd=" + $fh.ToInt64() + " fg=" + [VtFocus]::GetForegroundWindow().ToInt64())

$t0 = [HK]::HookCount
$ret = [HK]::SendInput(2, @($d, $u), $sz)
$err = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
PumpSleep 800
Log ("SendInput=$ret/2 err=$err -> hook nhan them: " + ([HK]::HookCount - $t0) + " (mong doi 2)")
$vks = [HK]::Recv | Select-Object -Last 6   # callback cung thread (DoEvents pump) -> khong can lock
Log ("vk gan nhat: " + (($vks | ForEach-Object { '0x{0:X2}' -f $_ }) -join ' '))

# === 4) UIA tu trong hook process (do latency giong WIN-004 nhung cung process) ===
$UAE = [System.Windows.Automation.AutomationElement]
$sw2 = [System.Diagnostics.Stopwatch]::StartNew()
$fe = $UAE.FocusedElement
$sw2.Stop()
$ms1 = [math]::Round($sw2.Elapsed.TotalMilliseconds, 1)
$ct = 'null'; $pw = 'null'
if ($fe) {
    $ct = $fe.Current.ControlType.ToString()
    $pw = $fe.Current.IsPassword
}
Log ("UIA FocusedElement (cung process): " + $ms1 + "ms controlType=$ct isPassword=$pw")

# === 4b) cross-process: notepad + SendInput + doc lai bang UIA (mau nightly harness) ===
# Win11 notepad = packaged: launcher pid khong co window (window thuoc pid khac) va co the
# exit som -> khong duoc lay MainWindowHandle cua Start-Process -PassThru; quet Get-Process.
$before = @(); Get-Process -Name notepad -ErrorAction SilentlyContinue | ForEach-Object { $before += $_.Id }
[void](Start-Process -FilePath 'notepad.exe')
$hn = [IntPtr]::Zero; $wpid = 0
# 15s (truoc do 6s): Win11 notepad packaged cold-start co the > 6s luc may dang tai ->
# luc do notepad "thua" con song, lan sau $before loai mat pid co window -> false negative (f6-12)
$deadline = [DateTime]::UtcNow.AddSeconds(15)
while ([DateTime]::UtcNow -lt $deadline -and $hn -eq [IntPtr]::Zero) {
    Start-Sleep -Milliseconds 250
    foreach ($q in @(Get-Process -Name notepad -ErrorAction SilentlyContinue)) {
        if ($before -notcontains $q.Id -and -not $q.HasExited) {
            $q.Refresh()
            $mh = $q.MainWindowHandle
            if ($mh -is [IntPtr] -and $mh -ne [IntPtr]::Zero) { $hn = $mh; $wpid = $q.Id; break }
        }
    }
}
Log ("notepad window: hwnd=" + $hn.ToInt64() + " wpid=$wpid (runner session=$session)")
if ($hn -eq [IntPtr]::Zero) {
    Log "notepad: KHONG tim thay window trong 15s (khong desktop session?) -> RW5 chan nightly tren GHA"
} else {
    PumpSleep 300   # cho notepad khoi tao control truoc khi query
    $sw3 = [System.Diagnostics.Stopwatch]::StartNew()
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($hn)
    $sw3.Stop()
    $ms2 = [math]::Round($sw3.Elapsed.TotalMilliseconds, 1)
    $ctp = [System.Windows.Automation.AutomationElement]::ControlTypeProperty
    # chain tim element: Edit(50004) -> Document(50030) -> ClassName 'Edit'
    # (Win11 packaged Notepad = Document; runner Server 2022 = Pane(50033) cls='Edit')
    $editCond = New-Object System.Windows.Automation.PropertyCondition -ArgumentList @($ctp, [System.Windows.Automation.ControlType]::Edit)
    $docCond = New-Object System.Windows.Automation.PropertyCondition -ArgumentList @($ctp, [System.Windows.Automation.ControlType]::Document)
    $sw4 = [System.Diagnostics.Stopwatch]::StartNew()
    $ed = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $editCond)
    $which = 'Edit'
    if (-not $ed) { $ed = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $docCond); $which = 'Document' }
    if (-not $ed) {
        # dump tree (chi cls/aId, khong log name de an toan S2) + tim theo ClassName
        $all2 = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
        $ids = @(); $exs = @()
        foreach ($e in $all2) {
            $cid = $e.Current.ControlType.Id
            if ($ids -notcontains $cid) { $ids += $cid }
            if ($exs.Count -lt 8) { $exs += ("ctId=" + $cid + " cls='" + $e.Current.ClassName + "' aId='" + $e.Current.AutomationId + "'") }
            if ((-not $ed) -and $e.Current.ClassName -eq 'Edit') { $ed = $e; $which = 'ClassName=Edit(ctId=' + $cid + ')' }
        }
        Log ("tree dump: count=" + $all2.Count + " ctIds=[" + ($ids -join ',') + "]")
        foreach ($x in $exs) { Log ("  " + $x) }
    }
    $sw4.Stop()
    $ms3 = [math]::Round($sw4.Elapsed.TotalMilliseconds, 1)
    $found = 'KHONG'; if ($ed) { $found = $which }
    Log ("notepad UIA: FromHandle=" + $ms2 + "ms Find(chain)=" + $ms3 + "ms -> $found")
    # doc truoc khi inject (Notepad co session restore -> chi so sanh DELTA)
    $len0 = ReadLen $ed
    Log ("value truoc inject: len=$len0 (-2 = khong co pattern doc duoc)")
    [void](Focus-UiAWindow $hn)
    PumpSleep 300
    Log ("focus notepad: fg=" + [VtFocus]::GetForegroundWindow().ToInt64() + " (mong doi $hn)")
    $t1 = [HK]::HookCount
    $r2 = [HK]::SendInput(2, @($d, $u), $sz)
    PumpSleep 600
    Log ("SendInput vao notepad=$r2/2 -> hook nhan them: " + ([HK]::HookCount - $t1))
    $len1 = ReadLen $ed
    $nhan = ($len0 -ge 0 -and $len1 -eq ($len0 + 1))
    Log ("doc lai tu notepad qua UIA: len0=$len0 len1=$len1 injectNhan=$nhan")
    # cleanup: chi kill notepad MOI (khong dot cua nguoi dung)
    Get-Process -Name notepad -ErrorAction SilentlyContinue | ForEach-Object {
        if ($before -notcontains $_.Id) { try { Stop-Process -Id $_.Id -Force } catch { } }
    }
}

# === 5) hook reinstall x10 (leak check) ===
$ok = 0
for ($i = 0; $i -lt 10; $i++) {
    [void][HK]::UnhookWindowsHookEx([HK]::Hook)
    [HK]::Hook = [HK]::SetWindowsHookExW(13, [HK]::Proc, [IntPtr]::Zero, 0)
    if ([HK]::Hook -ne [IntPtr]::Zero) { $ok++ }
}
Log ("reinstall x10 thanh cong: $ok/10")

# === cleanup ===
[void][HK]::UnhookWindowsHookEx([HK]::Hook)
$f.Close(); $f.Dispose()
# GHA: copy output ra RUNNER_TEMP de upload artifact (khong de file trong repo)
if ($env:RUNNER_TEMP -and ($env:RUNNER_TEMP -ne $env:TEMP)) {
    try { Copy-Item $out (Join-Path $env:RUNNER_TEMP 'hook_probe_out.txt') -Force } catch { }
}
Log 'DONE'
