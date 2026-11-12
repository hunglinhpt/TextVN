# SPDX-License-Identifier: GPL-3.0-or-later
# R2-159: go phim khi focus o control KHONG nhap van ban (danh sach file/cay thu muc) thi TextVN
# khong duoc mo composition: khong co khung `CiceroUIWndFrame` o goc trai man hinh, va phim toi app
# (type-ahead cua ListView chon dung muc). Chay tren may da cai TextVN (TIP dang active):
#   powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\e2e-non-text-focus.ps1
# ASCII-only (PowerShell 5.1, A5). Khong go vao app cua nguoi dung: chi cua so probe tu tao (B1).
param([switch]$ForceVietnamese)
$ErrorActionPreference = 'Stop'
$stateFile = Join-Path $env:APPDATA 'TextVN\state.json'
function Get-VnOn { try { return [bool]((Get-Content $stateFile -Raw | ConvertFrom-Json).global_enabled) } catch { return $true } }
$probeTitle = 'TVN_NONTEXT_PROBE'
$sel = Join-Path $env:TEMP 'tvn_nontext_selected.txt'
Remove-Item $sel -ErrorAction SilentlyContinue

$probe = @'
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.Form
$f.Text = 'TVN_NONTEXT_PROBE'; $f.StartPosition = 'Manual'
$f.Location = New-Object System.Drawing.Point(120, 120); $f.Size = New-Object System.Drawing.Size(500, 300)
$lv = New-Object System.Windows.Forms.ListView; $lv.View = 'List'; $lv.Dock = 'Fill'
foreach ($n in 'Alpha', 'Bravo', 'Foxtrot', 'Golf') { [void]$lv.Items.Add($n) }
$f.Controls.Add($lv)
$f.Add_Shown({ $f.Activate(); $lv.Focus(); $lv.Items[0].Selected = $true })
$t = New-Object System.Windows.Forms.Timer; $t.Interval = 20000; $t.Add_Tick({ $f.Close() }); $t.Start()
$f.Add_FormClosing({ if ($lv.SelectedItems.Count -gt 0) { Set-Content -Path $env:TVN_SEL_FILE -Value $lv.SelectedItems[0].Text -Encoding ASCII } })
[System.Windows.Forms.Application]::Run($f)
'@
$probePath = Join-Path $env:TEMP 'tvn_nontext_probe.ps1'
Set-Content -Path $probePath -Value $probe -Encoding ASCII

Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices; using System.Text; using System.Collections.Generic;
public static class NT {
  public delegate bool CB(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(CB cb, IntPtr l);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string t);
  public static string ClassOf(IntPtr h) { var s = new StringBuilder(256); GetClassName(h, s, 256); return s.ToString(); }
  public static uint PidOf(IntPtr h) { uint p; GetWindowThreadProcessId(h, out p); return p; }
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte sc, uint fl, UIntPtr ex);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  // Khung Cicero nho (< 400x200) cua tien trinh `pid` bat ky dau tren man hinh.
  public static List<string> CiceroFrames(uint pid) {
    var o = new List<string>();
    EnumWindows((h, l) => {
      if (!IsWindowVisible(h)) return true;
      uint p; GetWindowThreadProcessId(h, out p); if (p != pid) return true;
      var c = new StringBuilder(256); GetClassName(h, c, 256);
      if (c.ToString() != "CiceroUIWndFrame") return true;
      RECT r; GetWindowRect(h, out r);
      if (r.R - r.L < 400 && r.B - r.T < 200) o.Add("rect=" + r.L + "," + r.T + "," + r.R + "," + r.B);
      return true; }, IntPtr.Zero);
    return o; }
}
'@

# Che do EN thi TextVN khong an phim -> bai test vo nghia. -ForceVietnamese: bat tam bang Ctrl+Shift
# (chi khi probe la foreground) roi TRA LAI trang thai cu o cuoi.
$vnWasOn = Get-VnOn
$toggled = $false
$env:TVN_SEL_FILE = $sel
$proc = Start-Process powershell -ArgumentList '-NoProfile', '-STA', '-File', $probePath -PassThru
$fail = @()
try {
  Start-Sleep -Seconds 4
  $proc.Refresh()
  $h = $proc.MainWindowHandle
  if ($h -eq [IntPtr]::Zero) { throw 'probe window not found' }
  [void][NT]::SetForegroundWindow($h)
  Start-Sleep -Milliseconds 800
  # B1: chi gui phim khi cua so probe dang la foreground.
  if ([NT]::GetForegroundWindow() -ne $h) { throw 'probe is not foreground - refusing to send keys' }
  function Send-CtrlShiftTap {
    [NT]::keybd_event(0x11, 0, 0, [UIntPtr]::Zero); [NT]::keybd_event(0x10, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 80
    [NT]::keybd_event(0x10, 0, 2, [UIntPtr]::Zero); [NT]::keybd_event(0x11, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 700
  }
  if (-not $vnWasOn) {
    if (-not $ForceVietnamese) { Write-Output 'SKIP: TextVN dang o che do EN (dung -ForceVietnamese de bat tam)'; exit 2 }
    Send-CtrlShiftTap; $toggled = $true
    if (-not (Get-VnOn)) { throw 'khong bat duoc che do tieng Viet bang Ctrl+Shift' }
  }
  # 'f' (0x46): type-ahead phai nhay toi Foxtrot; khong duoc co khung Cicero.
  [NT]::keybd_event(0x46, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 60
  [NT]::keybd_event(0x46, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 1500
  $frames = [NT]::CiceroFrames([uint32]$proc.Id)
  if ($frames.Count -gt 0) { $fail += ('CiceroUIWndFrame hien ra khi go o ListView: ' + ($frames -join ';')) }
}
finally {
  if ($toggled -and (Get-VnOn) -ne $vnWasOn -and -not $proc.HasExited -and [NT]::GetForegroundWindow() -eq $proc.MainWindowHandle) { Send-CtrlShiftTap }
  Start-Sleep -Milliseconds 300
  if (-not $proc.HasExited) { [void]$proc.CloseMainWindow(); Start-Sleep -Milliseconds 800 }
  if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
}
# R2-161: thanh tac vu (Shell_TrayWnd, XAML) cung khong duoc mo composition. Luu y: explorer.exe nap TIP
# luc dang nhap -> sau khi cai ban moi can dang xuat/dang nhap lai (hoac khoi dong lai explorer) moi co hieu luc.
try {
  $tray = [NT]::FindWindow('Shell_TrayWnd', [NullString]::Value)
  if ($tray -ne [IntPtr]::Zero) {
    [void][NT]::SetForegroundWindow($tray); Start-Sleep -Milliseconds 700
    if ([NT]::ClassOf([NT]::GetForegroundWindow()) -eq 'Shell_TrayWnd') {
      [NT]::keybd_event(0x46, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 60
      [NT]::keybd_event(0x46, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 1500
      $tf = [NT]::CiceroFrames([NT]::PidOf($tray))
      if ($tf.Count -gt 0) { $fail += ('CiceroUIWndFrame hien ra khi go o thanh tac vu: ' + ($tf -join ';') + ' (neu vua cai ban moi: dang xuat/dang nhap lai de explorer nap TIP moi)') }
    } else { Write-Output 'SKIP taskbar: khong dua duoc thanh tac vu len foreground' }
  }
} catch { Write-Output ('SKIP taskbar: ' + $_.Exception.Message) }
$picked = if (Test-Path $sel) { (Get-Content $sel -Raw).Trim() } else { '' }
if ($picked -ne 'Foxtrot') { $fail += ("type-ahead khong chon Foxtrot (chon: '" + $picked + "') - phim khong toi ListView") }
if ($fail.Count -eq 0) { Write-Output 'PASS non-text focus: khong co khung Cicero, type-ahead chon dung'; exit 0 }
$fail | ForEach-Object { Write-Output ('FAIL ' + $_) }
exit 1
