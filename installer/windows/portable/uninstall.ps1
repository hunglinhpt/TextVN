# SPDX-License-Identifier: GPL-3.0-or-later
# uninstall.ps1 - tat TextVN va go dang ky bo go (TSF) truoc khi xoa thu muc portable.
# Cau hinh nguoi dung trong %APPDATA%\TextVN duoc giu lai.
$dir = Split-Path -Parent $MyInvocation.MyCommand.Path
Write-Host 'Tat TextVN...'
Start-Process -Wait -WindowStyle Hidden -FilePath (Join-Path $dir 'TextVN.exe') -ArgumentList '--stop' -ErrorAction SilentlyContinue
for ($i = 0; $i -lt 20 -and (Get-Process -Name TextVN -ErrorAction SilentlyContinue); $i++) { Start-Sleep -Milliseconds 250 }
# Tu khoi dong tro vao thu muc nay thi bo (khong dung toi ban TextVN da cai o noi khac).
$run = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$cmd = (Get-ItemProperty -Path $run -Name 'TextVN' -ErrorAction SilentlyContinue).TextVN
if ($cmd -and $cmd.Trim().TrimStart('"').StartsWith($dir.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) {
    Remove-ItemProperty -Path $run -Name 'TextVN' -ErrorAction SilentlyContinue
}
Write-Host 'Go dang ky TSF...'

# BUG-07 (audit 2026-10-04): Tra lai Ctrl + Shift cho Windows. Ban portable chi
# "danh Ctrl + Shift" o lan chay dau va ghi marker %APPDATA%\TextVN\ctrl_shift_default_applied
# (tray/src/main.rs apply_ctrl_shift_default_once) - chi restore khi co marker, de
# khong xoa lua chon "Not assigned" ma user tu dat trong Settings (CR-35).
$toggle = 'HKCU:\Keyboard Layout\Toggle'
$marker = Join-Path $env:APPDATA 'TextVN\ctrl_shift_default_applied'
$layout = (Get-ItemProperty -Path $toggle -Name 'Layout Hotkey' -ErrorAction SilentlyContinue).'Layout Hotkey'
if ((Test-Path -LiteralPath $marker -PathType Leaf) -and $layout -eq '3') {
    Remove-ItemProperty -Path $toggle -Name 'Layout Hotkey' -ErrorAction SilentlyContinue
    Write-Host 'Da tra lai Ctrl + Shift cho Windows (xoa Layout Hotkey override).'
}
Remove-Item -LiteralPath $marker -Force -ErrorAction SilentlyContinue

$r = Start-Process -Wait -PassThru -WindowStyle Hidden -FilePath (Join-Path $dir 'textvn-cli.exe') -ArgumentList 'unregister'
if ($r.ExitCode -ne 0) {
    Write-Error ('Huy dang ky TSF that bai, ma loi ' + $r.ExitCode + '. Khong xoa thu muc portable cho den khi "textvn-cli.exe doctor" bao ro nguyen nhan.')
    exit $r.ExitCode
}
# textvn-tsf.dll bi TSF nap trong cac app dang mo (explorer, Notepad...) nen thuong
# KHONG xoa duoc ngay. Tu dong don o lan dang nhap ke tiep bang RunOnce (khong can
# admin): luc do TIP da go dang ky nen DLL khong con bi nap.
$left = $null
# -LiteralPath: ten thu muc co '[' / ']' bi -Path hieu la wildcard -> khong xoa gi.
try { Remove-Item -LiteralPath $dir -Recurse -Force -ErrorAction Stop } catch { $left = $_ }
if ($left) {
    $runOnce = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\RunOnce'
    $cmd = 'cmd.exe /c rmdir /s /q "' + $dir + '"'
    Set-ItemProperty -Path $runOnce -Name 'TextVNCleanup' -Value $cmd -ErrorAction SilentlyContinue
    if ($?) {
        Write-Host 'Mot so file con dang duoc Windows dung (bo go da duoc go dang ky).'
        Write-Host 'TextVN se TU DONG don sach thu muc nay o lan dang nhap ke tiep - khong can lam gi them.'
    } else {
        Write-Host 'Khong xoa het duoc thu muc. Hay dang xuat roi xoa thu muc nay.'
    }
} else {
    Write-Host 'Da xoa thu muc portable.'
}
Write-Host 'Cau hinh (go tat, tuy chon) van o %APPDATA%\TextVN.'
