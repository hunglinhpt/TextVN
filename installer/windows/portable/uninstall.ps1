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
# R2-16 (P0): ZIP portable khong co thu muc goc - giai nen "Extract here" vao
# Downloads/Desktop thi $dir CHINH LA thu muc do. Truoc day `Remove-Item -Recurse`
# xoa sach ca thu muc cua nguoi dung. Gio chi xoa DUNG cac file TextVN biet ten,
# roi xoa thu muc con/thu muc goc khi (va chi khi) chung da rong.
$known = @(
    'TextVN.exe', 'textvn-cli.exe', 'textvn-tsf.dll', 'textvn-tsf-x86.dll', 'textvn-hook.exe',
    'install.ps1', 'HUONG_DAN_SU_DUNG.txt', 'README.md', 'CHANGELOG.md', 'LICENSE',
    'RELEASE_REPORT.json', 'resources\textvn.ico', 'resources\textvn_v.ico', 'resources\textvn_e.ico'
)
$locked = @()
foreach ($rel in $known) {
    $p = Join-Path $dir $rel
    if (Test-Path -LiteralPath $p -PathType Leaf) {
        try { Remove-Item -LiteralPath $p -Force -ErrorAction Stop } catch { $locked += $p }
    }
}
# Ban sao luu khi nang cap (TextVN.exe.old-*, textvn-tsf*.dll.old-*).
Get-ChildItem -LiteralPath $dir -File -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match '^(TextVN\.exe|textvn-tsf(-x86)?\.dll)\.old-' } |
    ForEach-Object {
        try { Remove-Item -LiteralPath $_.FullName -Force -ErrorAction Stop } catch { $locked += $_.FullName }
    }
# Thu muc chi xoa khi da rong - KHONG bao gio -Recurse.
$res = Join-Path $dir 'resources'
if ((Test-Path -LiteralPath $res -PathType Container) -and -not (Get-ChildItem -LiteralPath $res -Force)) {
    Remove-Item -LiteralPath $res -Force -ErrorAction SilentlyContinue
}
# uninstall.ps1 (chinh file nay) xoa sau cung; PowerShell da nap script nen xoa duoc.
Remove-Item -LiteralPath (Join-Path $dir 'uninstall.ps1') -Force -ErrorAction SilentlyContinue
$dirEmpty = -not (Get-ChildItem -LiteralPath $dir -Force -ErrorAction SilentlyContinue)
if ($dirEmpty) { Remove-Item -LiteralPath $dir -Force -ErrorAction SilentlyContinue }

if ($locked.Count -gt 0) {
    # DLL con bi nap trong app dang mo: hen xoa DUNG cac file do o lan dang nhap ke
    # tiep (RunOnce, khong can admin) - luc do TIP da go dang ky. `rmdir` khong /s:
    # chi xoa thu muc neu da rong.
    $runOnce = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\RunOnce'
    $cmd = 'cmd.exe /c del /f /q ' + (($locked | ForEach-Object { '"' + $_ + '"' }) -join ' ')
    if ($dirEmpty -eq $false) { $cmd += ' & rmdir "' + $res + '" & rmdir "' + $dir + '"' }
    Set-ItemProperty -Path $runOnce -Name 'TextVNCleanup' -Value $cmd -ErrorAction SilentlyContinue
    if ($?) {
        Write-Host 'Mot so file con dang duoc Windows dung (bo go da duoc go dang ky).'
        Write-Host 'TextVN se TU DONG xoa cac file do o lan dang nhap ke tiep - khong can lam gi them.'
    } else {
        Write-Host 'Khong xoa het duoc file cua TextVN. Hay dang xuat roi xoa cac file con lai.'
    }
} elseif (Test-Path -LiteralPath $dir) {
    Write-Host 'Da xoa cac file cua TextVN. Thu muc con file khac cua ban nen duoc giu nguyen.'
} else {
    Write-Host 'Da xoa thu muc portable.'
}
Write-Host 'Cau hinh (go tat, tuy chon) van o %APPDATA%\TextVN.'
