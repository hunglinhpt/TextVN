# SPDX-License-Identifier: GPL-3.0-or-later
# WIN-004 diag 5 (cuoi): IsPassword condition co flaky khong (do theo thoi gian),
# Name exact vs Contains, tren cua so chrome launch >= 2.
$ErrorActionPreference = 'Stop'
$out = Join-Path $env:TEMP 'uia_probe_conditions_out.txt'
if (Test-Path $out) { Remove-Item $out -Force }
function Log([string]$m) { Write-Output $m; Add-Content -Path $out -Value $m }
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public class UW7 {
    public delegate bool EnumProc(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
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
$AE = [System.Windows.Automation.AutomationElement]
$TS = [System.Windows.Automation.TreeScope]::Descendants
$PC = [System.Windows.Automation.PropertyCondition]
$CT = [System.Windows.Automation.ControlType]
$TrueC = [System.Windows.Automation.Condition]::TrueCondition
$PCF = [System.Windows.Automation.PropertyConditionFlags]

foreach ($p in (Get-CimInstance Win32_Process -Filter "Name='chrome.exe'" | Where-Object { $_.CommandLine -like '*uia_spike_chrome_profile*' })) {
    try { & taskkill /PID $p.ProcessId /T /F 2>&1 | Out-Null } catch {}
}
Start-Sleep -Seconds 1

$ud = Join-Path $env:TEMP 'uia_spike_chrome_profile'
$fix = ([System.Uri](Join-Path $PSScriptRoot 'uia_fixture.html')).AbsoluteUri
$ch = Start-Process 'C:\Program Files\Google\Chrome\Application\chrome.exe' `
    -ArgumentList @('--user-data-dir="' + $ud + '"', '--new-window', '--no-first-run', $fix) -PassThru
$hChrome = [IntPtr]::Zero
for ($i = 0; $i -lt 40 -and $hChrome -eq [IntPtr]::Zero; $i++) {
    foreach ($w in [UW7]::WinsAll()) { $s = $w.Split('|'); if ($s[2] -like 'Chrome_WidgetWin*' -and $s[3] -like '*fixture*') { $hChrome = [IntPtr][long]$s[0]; break } }
    if ($hChrome -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 500 }
}
Log ("hwnd=" + $hChrome.ToInt64())
if ($hChrome -eq [IntPtr]::Zero) { Log 'KHONG TIM THAY'; exit 1 }
$win = $AE::FromHandle($hChrome)

$cPwd = New-Object $PC $AE::IsPasswordProperty, $true
$cEdit = New-Object $PC ([System.Windows.Automation.AutomationElement]::ControlTypeProperty), ([System.Windows.Automation.ControlType]::Edit)
$cNameExact = New-Object $PC $AE::NameProperty, 'Address and search bar'
$cNamePart = New-Object $PC ([System.Windows.Automation.AutomationElement]::NameProperty), 'address'
# NOTE: PropertyConditionFlags .NET Framework chi co None/IgnoreCase -> khong co Contains.
# Name phan -> filter client-side tren TrueCondition.
$cNameCont = $null

# kiem tra dieu kien co "hoat dong luc nao": do lap lai trong 8s
$flip = -1
for ($t = 1; $t -le 16; $t++) {
    $pws = $win.FindAll($TS, $cPwd)
    $eds = $win.FindAll($TS, $cEdit)
    if ($pws.Count -gt 0) { $flip = $t; Log ("pwdCond=1 lan thu " + $t + " (t~" + [math]::Round($t * 0.5,1) + "s) sau khi editCond=" + $eds.Count); break }
    Start-Sleep -Milliseconds 500
}
if ($flip -lt 0) { Log ("pwdCond=0 sau 8s (editCond=" + $win.FindAll($TS, $cEdit).Count + ")") }

# dung lai 2s nua, do on dinh + Name test
Start-Sleep -Seconds 2
$p1 = $win.FindAll($TS, $cPwd).Count
$p2 = $win.FindAll($TS, $cPwd).Count
$rE = $win.FindFirst($TS, $cNameExact)
$rP = $win.FindFirst($TS, $cNamePart)
$rC = $null
$allNow = $win.FindAll($TS, $TrueC)
foreach ($e in $allNow) { if ($e.Current.Name -like '*address*') { $rC = $e; break } }
Log ("pwdCond on dinh: " + $p1 + ", " + $p2)
Log ("Name exact full 'Address and search bar': " + $(if ($rE) { 'THAY' } else { 'KHONG' }))
Log ("Name khong phan 'address': " + $(if ($rP) { 'THAY' } else { 'KHONG' }))
Log ("Name Contains 'address': " + $(if ($rC) { 'THAY (' + $rC.Current.Name + ')' } else { 'KHONG' }))

& taskkill /PID $ch.Id /T /F 2>&1 | Out-Null
Start-Sleep -Seconds 1
Log 'DONE'
