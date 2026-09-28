# SPDX-License-Identifier: GPL-3.0-or-later
# uninstall.ps1 - tat TextVN va go dang ky bo go (TSF) truoc khi xoa thu muc portable.
# Cau hinh nguoi dung trong %APPDATA%\TextVN duoc giu lai.
$dir = Split-Path -Parent $MyInvocation.MyCommand.Path
Write-Host 'Tat TextVN...'
Start-Process -Wait -WindowStyle Hidden -FilePath (Join-Path $dir 'TextVN.exe') -ArgumentList '--stop' -ErrorAction SilentlyContinue
for ($i = 0; $i -lt 20 -and (Get-Process -Name TextVN -ErrorAction SilentlyContinue); $i++) { Start-Sleep -Milliseconds 250 }
Write-Host 'Go dang ky TSF...'
Start-Process -Wait -WindowStyle Hidden -FilePath (Join-Path $dir 'textvn-cli.exe') -ArgumentList 'unregister'
Write-Host 'Xong. Neu Windows bao textvn-tsf.dll dang duoc dung, hay dang xuat roi xoa thu muc.'
Write-Host 'Cau hinh (go tat, tuy chon) van o %APPDATA%\TextVN.'
