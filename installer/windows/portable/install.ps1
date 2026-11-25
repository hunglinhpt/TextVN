# SPDX-License-Identifier: GPL-3.0-or-later
# install.ps1 - dang ky TextVN (TSF) cho tai khoan hien tai tu thu muc giai nen, roi mo TextVN.
# Khong bat buoc: nhan dup TextVN.exe cung tu dang ky (hoac mo Huong dan -> [Cai & bat TSF]).
# Script nay de chay lai khi can (vi du sau khi doi ten/di chuyen thu muc).
$dir = Split-Path -Parent $MyInvocation.MyCommand.Path
$dll = Join-Path $dir 'textvn-tsf.dll'
if (-not (Test-Path -LiteralPath $dll)) {
    Write-Error ('Thieu textvn-tsf.dll trong ' + $dir + '. Hay giai nen day du goi portable.')
    exit 1
}
Write-Host 'Dang ky bo go TextVN (TSF) cho tai khoan hien tai...'
$r = Start-Process -Wait -PassThru -WindowStyle Hidden -FilePath (Join-Path $dir 'textvn-cli.exe') -ArgumentList @('register', '--dll', $dll)
if ($r.ExitCode -eq 0) {
    Write-Host 'Da dang ky. Mo TextVN...'
} else {
    Write-Warning ('Dang ky TSF chua hoan tat (ma loi ' + $r.ExitCode + '). Xem %LOCALAPPDATA%\TextVN\logs\register.log hoac mo Bang dieu khien -> Huong dan -> [Cai & bat TSF].')
    Write-Warning 'Tren Windows 11 Enterprise 23H2/24H2+, neu Windows chan dang ky per-user, hay dung bo cai TextVN-setup-*-windows-x64-machine.exe.'
}
Start-Process -FilePath (Join-Path $dir 'TextVN.exe') -WorkingDirectory $dir
