# SPDX-License-Identifier: GPL-3.0-or-later
# install.ps1 - dang ky TextVN (TSF) cho tai khoan hien tai tu thu muc giai nen, roi mo TextVN.
# Khong bat buoc: nhan dup TextVN.exe cung tu dang ky. Script nay de chay lai khi can
# (vi du sau khi doi ten/di chuyen thu muc).
$dir = Split-Path -Parent $MyInvocation.MyCommand.Path
Write-Host 'Dang ky bo go TextVN (TSF) cho tai khoan hien tai...'
$r = Start-Process -Wait -PassThru -WindowStyle Hidden -FilePath (Join-Path $dir 'textvn-cli.exe') -ArgumentList 'register'
if ($r.ExitCode -eq 0) {
    Write-Host 'Da dang ky. Mo TextVN...'
} else {
    Write-Error ('Dang ky that bai, ma loi ' + $r.ExitCode + '. TextVN khong duoc mo de tranh hien trang da cai nhung khong go duoc tieng Viet. Chay "textvn-cli.exe doctor" de xem nguyen nhan.')
    exit $r.ExitCode
}
Start-Process -FilePath (Join-Path $dir 'TextVN.exe') -WorkingDirectory $dir
