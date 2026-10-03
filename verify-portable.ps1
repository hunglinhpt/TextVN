# Portable verify: activate + Notepad typing with Win+Space cycling. ASCII-only.
$ErrorActionPreference = 'Continue'
$dir = 'D:\TextVN-verify'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -MemberDefinition @'
[DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, System.UIntPtr dwExtraInfo);
[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
[DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
'@ -Name K -Namespace W

function TapVK([int]$vk) {
  [W.K]::keybd_event([byte]$vk, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 35
  [W.K]::keybd_event([byte]$vk, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 35
}
function WinSpace {
  [W.K]::keybd_event(0x5B, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 60
  [W.K]::keybd_event(0x20, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 60
  [W.K]::keybd_event(0x20, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 60
  [W.K]::keybd_event(0x5B, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 800
}
function TypeRaw([string]$s) {
  foreach ($ch in $s.ToCharArray()) {
    if ($ch -eq ' ') { TapVK 0x20; continue }
    TapVK ([int][char]::ToUpper($ch))
  }
}
function ReadNotepad([int]$targetPid) {
  $p = Get-Process -Id $targetPid -ErrorAction SilentlyContinue
  if (-not $p) { return '<no proc>' }
  try {
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    $edit = $null
    foreach ($ct in @([System.Windows.Automation.ControlType]::Document, [System.Windows.Automation.ControlType]::Edit)) {
      $cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, $ct)
      $edit = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $cond)
      if ($edit) { break }
    }
    if (-not $edit) { return '<no edit>' }
    $vp = $null
    if ($edit.TryGetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern, [ref]$vp)) { return $vp.Current.Value }
    $tp = $null
    if ($edit.TryGetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern, [ref]$tp)) { return $tp.DocumentRange.GetText(-1) }
    return '<no pattern>'
  } catch { return ('<uia err: ' + $_.Exception.Message + '>') }
}

# --- activate (ky vong B7: E_FAIL tren may nay) ---
$act = & (Join-Path $dir 'textvn-cli.exe') activate 2>&1
Write-Host ('activate exit=' + $LASTEXITCODE + ' | ' + ($act | Select-Object -Last 1))

Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 1000
Start-Process -FilePath (Join-Path (Join-Path $env:SystemRoot 'System32') 'notepad.exe') | Out-Null
$npPid = 0
for ($k = 0; $k -lt 20; $k++) {
  Start-Sleep -Milliseconds 500
  $npp = Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
  if ($npp) { $npPid = $npp.Id; break }
}
if ($npPid -eq 0) { Write-Host 'VERIFY: FAIL - no notepad'; exit 2 }
$npHwnd = (Get-Process -Id $npPid).MainWindowHandle

$duocN = (([string]([char]0x0111) + [char]0x01B0 + [char]0x1EE3 + 'c')).Normalize([Text.NormalizationForm]::FormC)
$tetN  = (('t' + [char]0x1EBF + 't')).Normalize([Text.NormalizationForm]::FormC)

for ($i = 1; $i -le 6; $i++) {
  for ($a = 0; $a -lt 10; $a++) {
    (New-Object -ComObject WScript.Shell).AppActivate($npPid) | Out-Null
    [W.K]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
    [W.K]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
    [W.K]::SetForegroundWindow($npHwnd) | Out-Null
    Start-Sleep -Milliseconds 400
    if ([W.K]::GetForegroundWindow() -eq $npHwnd) { break }
  }
  # clear
  [W.K]::keybd_event(0x11, 0, 0, [UIntPtr]::Zero)
  [W.K]::keybd_event(0x41, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40
  [W.K]::keybd_event(0x41, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40
  [W.K]::keybd_event(0x11, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 120
  TapVK 0x2E
  Start-Sleep -Milliseconds 250
  TypeRaw 'dduocj text '
  Start-Sleep -Milliseconds 1000
  $t = ReadNotepad $npPid
  $tN = $t.Normalize([Text.NormalizationForm]::FormC)
  $hasDuoc = $tN.Contains($duocN); $hasText = $tN.Contains('text'); $hasTet = $tN.Contains($tetN); $raw = $tN.Contains('dduocj')
  Write-Host ("attempt {0}: duoc={1} text={2} tet={3} raw={4} | '{5}'" -f $i, $hasDuoc, $hasText, $hasTet, $raw, $tN)
  if ($hasDuoc -and $hasText -and -not $hasTet) { Write-Host 'VERIFY PORTABLE TYPING: PASS - TextVN go dung'; exit 0 }
  WinSpace
}
Write-Host 'VERIFY PORTABLE TYPING: FAIL - khong lan nao ra TextVN'
