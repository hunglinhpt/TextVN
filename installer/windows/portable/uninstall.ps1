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
$r = Start-Process -Wait -PassThru -WindowStyle Hidden -FilePath (Join-Path $dir 'textvn-cli.exe') -ArgumentList 'unregister'
if ($r.ExitCode -ne 0) {
    Write-Error ('Huy dang ky TSF that bai, ma loi ' + $r.ExitCode + '. Khong xoa thu muc portable cho den khi "textvn-cli.exe doctor" bao ro nguyen nhan.')
    exit $r.ExitCode
}
Write-Host 'Xong. Neu Windows bao textvn-tsf.dll dang duoc dung, hay dang xuat roi xoa thu muc.'
Write-Host 'Cau hinh (go tat, tuy chon) van o %APPDATA%\TextVN.'
