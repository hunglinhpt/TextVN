# SPDX-License-Identifier: GPL-3.0-or-later
# measure_assist_rss.ps1 - F61: measure the TextVN TIP DLL working set inside a REAL
# TSF client process (Notepad) while Assist is enabled and text is being typed.
#
# PREREQUISITES (done by the operator, NOT by this script):
#   1. Build: cargo build -p textvn-win-tsf --release
#   2. Per-user TIP registration pointing at the freshly built DLL:
#        textvn-cli.exe register --dll <abs path>\textvn_win_tsf.dll
#   3. Assist enabled in %APPDATA%\TextVN\config.json ("assist":{"mode":"highlight"})
#      and the tray state enabled (state.json global_enabled=true).
#   4. A FRESH Notepad process that loaded the DLL under test:
#        notepad.exe -m "<scratch file>"
#      (Windows 11 Notepad is single-instance by default; -m forces a new process.
#       Never reuse the user's Notepad window/process: it may already have another
#       TextVN build loaded and may contain the user's unsaved work.)
#   5. Cleanup afterwards is the operator's job (unregister the per-user entry,
#      restore config/state, close the scratch Notepad).
#
# WHAT IS MEASURED
#   * "DLL working set" = sum of RESIDENT pages (4 KiB each) that belong to image
#     regions mapped from the target DLL file (VirtualQueryEx -> GetMappedFileNameW
#     -> QueryWorkingSetEx Valid bit). This is the in-process cost attributable to
#     the DLL image: .text/.rdata/.data/.reloc plus copy-on-write pages.
#     LIMITATION: heap/private allocations made by code inside the DLL are charged
#     to the process heap, not to the mapped image, so they are NOT in this number.
#     Total process WorkingSet/PrivateBytes are reported alongside to bound that.
#   * Process WorkingSet + Private bytes (System.Diagnostics.Process).
#   * Percentiles (mean/p95/max) of the sampled series (200 ms period by default).
#
# S2 / privacy: the script types a FIXED synthetic Vietnamese phrase (hard-coded in
# this file). It never reads, prints or writes document content; the one verification
# step compares the editor text against the hard-coded expected string in memory and
# reports only "typing-verified=yes/no".
#
# Output is ASCII only (the repo convention for tools/win scripts).
#
# Usage:
#   powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\measure_assist_rss.ps1 `
#       -ProcessId 1234 -DurationSec 60 -IntervalMs 200 -IdleSec 10
#   Optional: -OutFile <path.json> to persist the summary.

param(
    [Parameter(Mandatory = $true)][int]$ProcessId,
    [int]$DurationSec = 60,
    [int]$IntervalMs = 200,
    [int]$IdleSec = 10,
    [switch]$NoTyping,
    [string]$DllName = 'textvn_win_tsf.dll',
    [string]$OutFile = ''
)

$ErrorActionPreference = 'Stop'

if (-not ('F61' -as [type])) {
    Add-Type @'
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

public static class F61
{
    // ---------- native interop ----------
    [StructLayout(LayoutKind.Sequential)]
    public struct MBI
    {
        public IntPtr BaseAddress;
        public IntPtr AllocationBase;
        public uint AllocationProtect;
        private uint _pad0;
        public IntPtr RegionSize;
        public uint State;
        public uint Protect;
        public uint Type;
        private uint _pad1;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct WSEx
    {
        public IntPtr VirtualAddress;
        public ulong Attributes;   // PSAPI_WORKING_SET_EX_BLOCK, bit 0 = Valid
    }

    [StructLayout(LayoutKind.Sequential)]
    struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Explicit, Size = 40)]
    struct INPUT { [FieldOffset(0)] public uint type; [FieldOffset(8)] public KEYBDINPUT ki; }

    [DllImport("kernel32.dll", SetLastError = true)] static extern IntPtr OpenProcess(uint access, bool inherit, int pid);
    [DllImport("kernel32.dll", SetLastError = true)] static extern bool CloseHandle(IntPtr h);
    [DllImport("kernel32.dll", SetLastError = true)] static extern IntPtr VirtualQueryEx(IntPtr h, IntPtr addr, out MBI mbi, IntPtr len);
    [DllImport("psapi.dll", CharSet = CharSet.Unicode, SetLastError = true)] static extern uint GetMappedFileNameW(IntPtr h, IntPtr addr, StringBuilder name, uint max);
    [DllImport("psapi.dll", SetLastError = true)] static extern bool QueryWorkingSetEx(IntPtr h, [In, Out] WSEx[] info, uint cb);

    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] p, int cb);
    [DllImport("user32.dll")] static extern uint MapVirtualKeyW(uint code, uint type);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool f);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr FindWindowExW(IntPtr parent, IntPtr after, string cls, string title);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, StringBuilder l);
    [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, IntPtr l);

    static IntPtr _hProc = IntPtr.Zero;
    static int _pid = 0;

    public static int Open(int pid)
    {
        _pid = pid;
        // PROCESS_QUERY_INFORMATION | PROCESS_VM_READ
        _hProc = OpenProcess(0x0410, false, pid);
        if (_hProc == IntPtr.Zero) return Marshal.GetLastWin32Error();
        return 0;
    }

    public static void Close()
    {
        if (_hProc != IntPtr.Zero) { CloseHandle(_hProc); _hProc = IntPtr.Zero; }
    }

    /// Sum of mapped image bytes and resident (Valid) 4 KiB pages for image regions
    /// whose mapped file name ends with <paramref name="dllName"/>.
    public static long[] DllWorkingSet(string dllName, out long mappedBytes)
    {
        mappedBytes = 0;
        long residentPages = 0;
        if (_hProc == IntPtr.Zero) return new long[] { -1, -1 };

        IntPtr addr = IntPtr.Zero;
        MBI mbi;
        int guard = 0;
        while (guard++ < 500000)
        {
            if (VirtualQueryEx(_hProc, addr, out mbi, (IntPtr)Marshal.SizeOf(typeof(MBI))) == IntPtr.Zero) break;
            long size = mbi.RegionSize.ToInt64();
            if (size <= 0) break;

            if (mbi.State == 0x1000 /*MEM_COMMIT*/ && mbi.Type == 0x1000000 /*MEM_IMAGE*/)
            {
                var name = new StringBuilder(600);
                if (GetMappedFileNameW(_hProc, mbi.BaseAddress, name, 600) > 0)
                {
                    string f = name.ToString();
                    if (f.EndsWith(dllName, StringComparison.OrdinalIgnoreCase))
                    {
                        mappedBytes += size;
                        int pageCount = (int)((size + 4095) / 4096);
                        var arr = new WSEx[pageCount];
                        long baseAddr = mbi.BaseAddress.ToInt64();
                        for (int i = 0; i < pageCount; i++) arr[i].VirtualAddress = (IntPtr)(baseAddr + (long)i * 4096);
                        uint cb = (uint)(pageCount * Marshal.SizeOf(typeof(WSEx)));
                        if (QueryWorkingSetEx(_hProc, arr, cb))
                        {
                            for (int i = 0; i < pageCount; i++)
                                if ((arr[i].Attributes & 1UL) != 0) residentPages++;
                        }
                    }
                }
            }

            long next = mbi.BaseAddress.ToInt64() + size;
            if (next <= addr.ToInt64()) break;
            addr = (IntPtr)next;
        }
        return new long[] { residentPages * 4096, mappedBytes };
    }

    // ---------- synthetic typing load (no document text is ever read back) ----------
    // Fixed Telex keystroke sequence for: "Toi toi di hoc." (deliberate duplicate word
    // so the L1 rule fires and the Mode 1 overlay has something to show). Kept ASCII:
    // the expected result is expressed with \u escapes below.
    const string TELEX = "tooi tooi ddii hocj.";

    const uint INPUT_KEYBOARD = 1, KEYEVENTF_KEYUP = 2;
    static volatile bool _stop = false;
    static Thread _thread;
    public static volatile int KeysTyped = 0;
    public static volatile bool Verified = false;
    public static volatile bool FocusLost = false;
    static IntPtr _hwnd = IntPtr.Zero;

    static INPUT Key(ushort vk, bool up)
    {
        var i = new INPUT();
        i.type = INPUT_KEYBOARD;
        i.ki.wVk = vk;
        i.ki.wScan = (ushort)MapVirtualKeyW(vk, 0);
        i.ki.dwFlags = up ? KEYEVENTF_KEYUP : 0;
        return i;
    }

    static void Tap(ushort vk, int ms)
    {
        SendInput(2, new[] { Key(vk, false), Key(vk, true) }, Marshal.SizeOf(typeof(INPUT)));
        if (ms > 0) Thread.Sleep(ms);
    }

    public static bool Focus(IntPtr h)
    {
        uint pid;
        uint fgThread = GetWindowThreadProcessId(GetForegroundWindow(), out pid);
        uint me = GetCurrentThreadId();
        AttachThreadInput(me, fgThread, true);
        ShowWindow(h, 5);
        BringWindowToTop(h);
        bool ok = SetForegroundWindow(h);
        AttachThreadInput(me, fgThread, false);
        Thread.Sleep(250);
        return ok && GetForegroundWindow() == h;
    }

    public static IntPtr EditOf(IntPtr main)
    {
        foreach (var cls in new[] { "Edit", "RichEditD2DPT", "RICHEDIT50W" })
        {
            IntPtr e = FindWindowExW(main, IntPtr.Zero, cls, null);
            if (e != IntPtr.Zero) return e;
        }
        IntPtr box = IntPtr.Zero;
        while ((box = FindWindowExW(main, box, "NotepadTextBox", null)) != IntPtr.Zero)
        {
            if (!IsWindowVisible(box)) continue;
            IntPtr e = FindWindowExW(box, IntPtr.Zero, "RichEditD2DPT", null);
            if (e != IntPtr.Zero) return e;
        }
        return IntPtr.Zero;
    }

    static string Text(IntPtr edit)
    {
        int n = (int)SendMessageW(edit, 0x000E /*WM_GETTEXTLENGTH*/, IntPtr.Zero, IntPtr.Zero);
        if (n <= 0) return string.Empty;
        var sb = new StringBuilder(n + 1);
        SendMessageW(edit, 0x000D /*WM_GETTEXT*/, (IntPtr)(n + 1), sb);
        return sb.ToString();
    }

    static void TypeTelex()
    {
        foreach (char c in TELEX)
        {
            if (_stop) return;
            if (c == ' ') { Tap(0x20, 25); }
            else if (c == '.') { Tap(0xBE, 25); }
            else { Tap((ushort)Char.ToUpperInvariant(c), 25); }
        }
    }

    public static void StartTyping(IntPtr hwnd)
    {
        _hwnd = hwnd;
        _stop = false;
        _thread = new Thread(() =>
        {
            try
            {
                if (!Focus(hwnd)) FocusLost = true;

                // One-shot functional check of the TIP pipeline (content stays in RAM).
                IntPtr edit = EditOf(hwnd);
                TypeTelex();
                Thread.Sleep(900);
                if (edit != IntPtr.Zero)
                {
                    // Expected: "T\u00f4i t\u00f4i \u0111i h\u1ecdc." (auto_capitalize on).
                    string want = "T\u00f4i t\u00f4i \u0111i h\u1ecdc.";
                    string got = Text(edit);
                    Verified = got.IndexOf(want, StringComparison.Ordinal) >= 0;
                }
                KeysTyped += TELEX.Length;

                int cycle = 0;
                while (!_stop)
                {
                    cycle++;
                    if (GetForegroundWindow() != hwnd)
                    {
                        FocusLost = true;
                        if (!Focus(hwnd)) { Thread.Sleep(500); continue; }
                    }
                    TypeTelex();
                    KeysTyped += TELEX.Length;
                    // Idle window so the D11 display point (idle_ms = 700) fires and the
                    // overlay is created/refreshed; longer pause every 4th sentence.
                    Thread.Sleep(cycle % 4 == 0 ? 1500 : 900);
                }
            }
            catch (Exception) { FocusLost = true; }
        });
        _thread.IsBackground = true;
        _thread.Start();
    }

    public static void StopTyping()
    {
        _stop = true;
        if (_thread != null) { _thread.Join(3000); _thread = null; }
    }
}
'@
}

function Get-Percentile {
    param([double[]]$Values, [double]$Pct)
    if ($Values.Count -eq 0) { return 0 }
    $sorted = @($Values | Sort-Object)
    $rank = [int][Math]::Ceiling($Pct / 100.0 * $sorted.Count)
    if ($rank -lt 1) { $rank = 1 }
    if ($rank -gt $sorted.Count) { $rank = $sorted.Count }
    return $sorted[$rank - 1]
}

function Show-Stats {
    param([string]$Name, [double[]]$Values, [string]$Unit)
    if ($Values.Count -eq 0) { return [ordered]@{ name = $Name; n = 0 } }
    $mean = ($Values | Measure-Object -Average).Average
    return [ordered]@{
        name = $Name
        n = $Values.Count
        unit = $Unit
        mean = [Math]::Round($mean, 1)
        p95 = [Math]::Round((Get-Percentile -Values $Values -Pct 95), 1)
        max = [Math]::Round((@($Values | Measure-Object -Maximum).Maximum), 1)
    }
}

# ---------- preflight ----------
$proc = Get-Process -Id $ProcessId -ErrorAction Stop
if ($proc.ProcessName -ne 'Notepad') {
    Write-Warning "target pid $ProcessId is '$($proc.ProcessName)', not Notepad (continuing: any TSF client works)"
}
$mod = $proc.Modules | Where-Object { $_.ModuleName -eq $DllName } | Select-Object -First 1
if (-not $mod) {
    throw "pid $ProcessId does not have $DllName loaded - wrong registration/process (F61 requires the DLL under test in the TSF client)"
}
$hwnd = $proc.MainWindowHandle
if ($hwnd -eq [IntPtr]::Zero) { throw "pid $ProcessId has no main window" }

$configPath = Join-Path $env:APPDATA 'TextVN\config.json'
$assistMode = 'unknown'
if (Test-Path $configPath) {
    try {
        $cfg = Get-Content $configPath -Raw | ConvertFrom-Json
        if ($cfg.assist -and $cfg.assist.mode) { $assistMode = [string]$cfg.assist.mode }
    } catch { $assistMode = 'parse-error' }
}

$hash = (Get-FileHash $mod.FileName -Algorithm SHA256).Hash
Write-Host ("target       : pid={0} exe={1}" -f $ProcessId, $proc.Path)
Write-Host ("dll          : {0}" -f $mod.FileName)
Write-Host ("dll sha256   : {0}" -f $hash)
Write-Host ("dll version  : {0}" -f $mod.FileVersionInfo.FileVersion)
Write-Host ("assist.mode  : {0}" -f $assistMode)
Write-Host ("plan         : idle {0}s + typing {1}s, sample every {2} ms" -f $IdleSec, $DurationSec, $IntervalMs)

$err = [F61]::Open($ProcessId)
if ($err -ne 0) { throw "OpenProcess failed ($err)" }

$idleDll = New-Object System.Collections.Generic.List[double]
$idleWs = New-Object System.Collections.Generic.List[double]
$idlePriv = New-Object System.Collections.Generic.List[double]
$typeDll = New-Object System.Collections.Generic.List[double]
$typeWs = New-Object System.Collections.Generic.List[double]
$typePriv = New-Object System.Collections.Generic.List[double]
$mappedBytes = 0L

function Sample-Once {
    param($DllList, $WsList, $PrivList)
    $mapped = 0L
    $r = [F61]::DllWorkingSet($DllName, [ref]$mapped)
    $script:mappedBytes = $mapped
    $proc.Refresh()
    if ($r[0] -ge 0) {
        $DllList.Add([double]$r[0] / 1024.0)
        $script:lastDllBytes = $r[0]
    }
    $WsList.Add([double]$proc.WorkingSet64 / 1048576.0)
    $PrivList.Add([double]$proc.PrivateMemorySize64 / 1048576.0)
}

try {
    $sw = [Diagnostics.Stopwatch]::StartNew()

    # Phase 1: idle (process loaded, no typing) - baseline attributable to the image.
    while ($sw.Elapsed.TotalSeconds -lt $IdleSec) {
        Sample-Once $idleDll $idleWs $idlePriv
        Start-Sleep -Milliseconds $IntervalMs
    }

    # Phase 2: typing with Assist enabled (overlay fires on sentence end / idle 700 ms).
    if (-not $NoTyping) {
        [F61]::StartTyping($hwnd) | Out-Null
    }
    while ($sw.Elapsed.TotalSeconds -lt ($IdleSec + $DurationSec)) {
        Sample-Once $typeDll $typeWs $typePriv
        Start-Sleep -Milliseconds $IntervalMs
    }
    if (-not $NoTyping) { [F61]::StopTyping() }
}
finally {
    try { [F61]::StopTyping() } catch { }
    [F61]::Close()
}

$summary = [ordered]@{
    timestamp = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ssK')
    pid = $ProcessId
    process = $proc.ProcessName
    process_path = $proc.Path
    dll_path = $mod.FileName
    dll_sha256 = $hash
    dll_version = [string]$mod.FileVersionInfo.FileVersion
    dll_image_mapped_kb = [Math]::Round([double]$mappedBytes / 1024.0, 1)
    assist_mode = $assistMode
    interval_ms = $IntervalMs
    idle_sec = $IdleSec
    typing_sec = $DurationSec
    no_typing = [bool]$NoTyping
    keys_typed = [F61]::KeysTyped
    typing_verified = [bool][F61]::Verified
    focus_lost = [bool][F61]::FocusLost
    idle_dll_ws = (Show-Stats 'dll_ws' $idleDll.ToArray() 'KB')
    idle_process_ws = (Show-Stats 'process_ws' $idleWs.ToArray() 'MB')
    idle_private = (Show-Stats 'process_private' $idlePriv.ToArray() 'MB')
    typing_dll_ws = (Show-Stats 'dll_ws' $typeDll.ToArray() 'KB')
    typing_process_ws = (Show-Stats 'process_ws' $typeWs.ToArray() 'MB')
    typing_private = (Show-Stats 'process_private' $typePriv.ToArray() 'MB')
    notes = @(
        'dll_ws counts resident 4 KiB pages of image regions mapped from the DLL file (VirtualQueryEx + GetMappedFileNameW + QueryWorkingSetEx).',
        'Heap/private allocations made by DLL code are charged to the process heap and are NOT included in dll_ws.',
        'p95 = nearest-rank percentile of the 200 ms sample series, not a per-key latency.'
    )
}

Write-Host ''
Write-Host '--- DLL working set (image-attributable, KiB) ---'
$summary.idle_dll_ws, $summary.typing_dll_ws | Format-Table -AutoSize | Out-String | Write-Host
Write-Host '--- Process working set (MiB) ---'
$summary.idle_process_ws, $summary.typing_process_ws | Format-Table -AutoSize | Out-String | Write-Host
Write-Host '--- Process private bytes (MiB) ---'
$summary.idle_private, $summary.typing_private | Format-Table -AutoSize | Out-String | Write-Host
Write-Host ("keys typed   : {0}; typing-verified={1}; focus-lost={2}" -f $summary.keys_typed, $summary.typing_verified, $summary.focus_lost)
Write-Host ("image mapped : {0} KiB" -f $summary.dll_image_mapped_kb)

if ($OutFile -ne '') {
    $json = $summary | ConvertTo-Json -Depth 6
    [System.IO.File]::WriteAllText($OutFile, $json, (New-Object System.Text.UTF8Encoding $false))
    Write-Host ("summary json : {0}" -f $OutFile)
}
