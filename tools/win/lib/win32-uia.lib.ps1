# SPDX-License-Identifier: GPL-3.0-or-later
# Shared Win32 + UIA helper lib (G15): dot-source tu spikes/* va tools/win/*.
# Chi tra ve ten element/hwnd (S2 - khong log text/nguoi dung).
#
# Usage:  . (Join-Path $PSScriptRoot '..\..\tools\win\lib\win32-uia.lib.ps1')
#         Initialize-TextVNUiA

# --- 1. UIA assemblies (idempotent) -----------------------------------------
function Initialize-TextVNUiA {
    if (-not ('System.Windows.Automation.AutomationElement' -as [type])) {
        Add-Type -AssemblyName UIAutomationClient
        Add-Type -AssemblyName UIAutomationTypes
    }
}

# --- 2. Win32 window enum (C#, chay nhanh hon PS) ---------------------------
# Replaces UW (uia_spike.ps1) + UW7 (probe_conditions.ps1) - trung lap truoc do.
if (-not ('VtWin' -as [type])) {
    Add-Type @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public class VtWin {
    public delegate bool EnumProc(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern IntPtr GetWindow(IntPtr h, uint cmd);
    // cua so so huu owner = popup/dialog (vd Chrome "Translate this page?") - bo qua khi chon cua so chinh
    public static IntPtr GetOwner(IntPtr h) { return GetWindow(h, 4); /* GW_OWNER */ }
    // format: hwnd|class|title
    public static List<string> WinsOf(uint pid) {
        var r = new List<string>();
        EnumWindows((h, l) => {
            uint p; GetWindowThreadProcessId(h, out p);
            if (p == pid && IsWindowVisible(h)) {
                var c = new StringBuilder(256); GetClassName(h, c, 256);
                var t = new StringBuilder(512); GetWindowText(h, t, 512);
                r.Add(h.ToInt64() + "|" + c + "|" + t);
            }
            return true;
        }, IntPtr.Zero);
        return r;
    }
    // format: hwnd|class|title
    public static List<string> WinsByClass(string cls) {
        var r = new List<string>();
        EnumWindows((h, l) => {
            if (!IsWindowVisible(h)) return true;
            var c = new StringBuilder(256); GetClassName(h, c, 256);
            if (c.ToString() == cls) {
                var t = new StringBuilder(512); GetWindowText(h, t, 512);
                r.Add(h.ToInt64() + "|" + c + "|" + t);
            }
            return true;
        }, IntPtr.Zero);
        return r;
    }
    // format: hwnd|pid|class|title  (tat ca cua so visible, loc ngoai)
    public static List<string> WinsAll() {
        var r = new List<string>();
        EnumWindows((h, l) => {
            if (IsWindowVisible(h)) {
                var c = new StringBuilder(256); GetClassName(h, c, 256);
                var t = new StringBuilder(512); GetWindowText(h, t, 512);
                uint p; GetWindowThreadProcessId(h, out p);
                r.Add(h.ToInt64() + "|" + p + "|" + c + "|" + t);
            }
            return true;
        }, IntPtr.Zero);
        return r;
    }
}
'@
}

# --- 3. UIA condition helpers -----------------------------------------------
# A10: goi [Type]::StaticProp bang ngoac () khi truyen vao function.
function New-UiACond($prop, $val) {
    New-Object System.Windows.Automation.PropertyCondition $prop, $val
}
function Get-UiAControlType([string]$name) {
    $t = [System.Windows.Automation.ControlType]
    # .NET Framework: ControlType statics la FIELD (khong phai property); 39 type,
    # khong co TextArea/Grid (S4: member = NULL) -> neu thieu thi fail ro rang.
    $f = $t.GetField($name)
    if (-not $f) { throw ("Unknown control_type (khong co trong .NET UIA): " + $name) }
    $v = $f.GetValue($t)
    if ($null -eq $v) { throw ("ControlType." + $name + " = NULL tren runtime nay") }
    return $v
}
function Wait-UiAWin([uint32]$pidWant, [string]$titleLike, [int]$sec = 30) {
    for ($i = 0; $i -lt $sec; $i++) {
        foreach ($w in [VtWin]::WinsOf($pidWant)) {
            $p = $w.Split('|')
            if ($titleLike -eq '' -or $p[2] -like $titleLike) { return [IntPtr][long]$p[0] }
        }
        Start-Sleep -Seconds 1
    }
    return [IntPtr]::Zero
}
function Get-UiAWindowByClass([string]$cls) {
    foreach ($cand in [VtWin]::WinsByClass($cls)) { return [IntPtr][long]$cand.Split('|')[0] }
    return [IntPtr]::Zero
}

# --- 5. Focus (AttachThreadInput pattern) -----------------------------------
# Truoc do trung lap trong hook_probe.ps1 [HK].Focus -> tach ve day (G15).
if (-not ('VtFocus' -as [type])) {
    Add-Type @'
using System;
using System.Runtime.InteropServices;
public class VtFocus {
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool f);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    public static bool Focus(IntPtr h) {
        IntPtr fg = GetForegroundWindow(); uint fpid;
        uint fgTid = GetWindowThreadProcessId(fg, out fpid);
        uint tpid; uint tTid = GetWindowThreadProcessId(h, out tpid);
        uint my = GetCurrentThreadId();
        AttachThreadInput(my, fgTid, true);
        AttachThreadInput(my, tTid, true);
        ShowWindow(h, 9); BringWindowToTop(h);
        bool ok = SetForegroundWindow(h);
        AttachThreadInput(my, fgTid, false);
        AttachThreadInput(my, tTid, false);
        return ok;
    }
}
'@
}
function Focus-UiAWindow([IntPtr]$hwnd) { [VtFocus]::Focus($hwnd) }

# --- 6. Locator evaluation (targets JSON semantics) -------------------------
# Locator = object AND- cac key: control_type, automation_id, name, name_regex,
#                             class_name, class_regex (1 trong 2 *_regex thi filter client-side).
# Field.locators = mang uu tien: locator dau tien tim thay thang (fallback chain).
# Rust harness (WIN-060) + field-detect phai mo ta cung nghia nay (G15 - 1 semantics).
function Find-UiAElementByLocator($root, $locator) {
    Initialize-TextVNUiA
    $AE = [System.Windows.Automation.AutomationElement]
    $TS = [System.Windows.Automation.TreeScope]::Descendants
    $conds = New-Object System.Collections.Generic.List[object]
    if ($locator.control_type) { $conds.Add((New-UiACond $AE::ControlTypeProperty (Get-UiAControlType $locator.control_type))) }
    if ($locator.automation_id) { $conds.Add((New-UiACond $AE::AutomationIdProperty $locator.automation_id)) }
    if ($locator.class_name) { $conds.Add((New-UiACond $AE::ClassNameProperty $locator.class_name)) }
    if ($locator.name) { $conds.Add((New-UiACond $AE::NameProperty $locator.name)) }
    if ($conds.Count -eq 0 -and -not ($locator.name_regex -or $locator.class_regex)) { return $null }
    if ($conds.Count -eq 0) { $cond = [System.Windows.Automation.Condition]::TrueCondition }
    elseif ($conds.Count -eq 1) { $cond = $conds[0] }
    else { $cond = New-Object System.Windows.Automation.AndCondition $conds.ToArray() }
    $hit = $root.FindFirst($TS, $cond)
    if (-not $hit) { return $null }
    if (-not ($locator.name_regex -or $locator.class_regex)) { return $hit }
    # regex: PropertyCondition khong ho tro regex -> FindAll + filter client-side
    $all = $root.FindAll($TS, $cond)
    foreach ($e in $all) {
        if ($locator.name_regex -and $e.Current.Name -notmatch $locator.name_regex) { continue }
        if ($locator.class_regex -and $e.Current.ClassName -notmatch $locator.class_regex) { continue }
        return $e
    }
    return $null
}
