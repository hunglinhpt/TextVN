# SPDX-License-Identifier: GPL-3.0-or-later
# test-portable.ps1 -Zip <TextVN-portable-*.zip>
#
# Kich ban "giai nen ra dung luon": giai nen vao thu muc tam -> chay TextVN.exe (tu dang
# ky TSF per-user tu chinh thu muc do) -> go tieng Viet that trong Notepad -> uninstall.ps1
# -> dang ky TSF da go, cau hinh nguoi dung van con.
#
# LUU Y MOI TRUONG (B7, 2026-10-03): runner CI chay ELEVATED nen register trong buoc nay
# that ra ghi ca HKLM (RegisterProfile API thanh cong voi admin) - kich ban KHONG chung
# minh duoc typing voi dang ky THUAN per-user tren may thuong. Tren Windows 11 24H2+
# (build 26300), per-user thuan co the bi tu choi ActivateProfile - xem
# docs/specs/win-test-common-errors.md B7 + docs/user-guide.md (dung bo cai pham vi may).
#
# LUU Y MOI TRUONG (B7, 2026-10-03): runner CI chay ELEVATED nen register trong buoc nay
# minh duoc typing voi dang ky THUAN per-user tren may thuong. Tren Windows 11 24H2+
# docs/specs/win-test-common-errors.md B7 + docs/user-guide.md (dung bo cai pham vi may).

param(
    [Parameter(Mandatory = $true)][string]$Zip
)
$ErrorActionPreference = 'Stop'
$clsid = '{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}'
$inproc = "HKCU:\Software\Classes\CLSID\$clsid\InprocServer32"

$tempRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { $env:TEMP }
$dir = Join-Path $tempRoot ('textvn-portable-' + [guid]::NewGuid().ToString('N'))
try {
Expand-Archive -Path $Zip -DestinationPath $dir
foreach ($f in @('TextVN.exe', 'textvn-cli.exe', 'textvn-tsf.dll', 'textvn-tsf-x86.dll', 'install.ps1', 'uninstall.ps1', 'HUONG_DAN_SU_DUNG.txt', 'RELEASE_REPORT.json')) {
    if (-not (Test-Path (Join-Path $dir $f))) { throw "portable zip is missing $f" }
}
Write-Host "PASS zip layout ($dir)"

# R2-40: ban portable danh Ctrl + Shift o lan chay dau va ghi gia tri cu vao marker;
# uninstall.ps1 phai tra lai DUNG gia tri do. Dat Windows ve mac dinh (Ctrl+Shift = '2').
$toggle = 'HKCU:\Keyboard Layout\Toggle'
if (-not (Test-Path $toggle)) { New-Item -Path $toggle -Force | Out-Null }
Set-ItemProperty -Path $toggle -Name 'Layout Hotkey' -Value '2'
Remove-Item -LiteralPath (Join-Path $env:APPDATA 'TextVN\ctrl_shift_default_applied') -Force -ErrorAction SilentlyContinue

# Nhan dup TextVN.exe: tray tu dang ky TIP tro vao DLL trong thu muc giai nen.
Start-Process -FilePath (Join-Path $dir 'TextVN.exe') -WorkingDirectory $dir | Out-Null
$registered = $false
for ($i = 0; $i -lt 40 -and -not $registered; $i++) {
    Start-Sleep -Milliseconds 500
    $v = (Get-ItemProperty -Path $inproc -ErrorAction SilentlyContinue).'(default)'
    $registered = ($v -eq (Join-Path $dir 'textvn-tsf.dll'))
}
if (-not $registered) { throw 'TextVN.exe did not register the TSF TIP from the extracted folder' }
Write-Host 'PASS TextVN.exe registered TSF from the extracted folder'
$layout = (Get-ItemProperty -Path $toggle -ErrorAction SilentlyContinue).'Layout Hotkey'
if ($layout -ne '3') { throw "portable first run did not reserve Ctrl+Shift (Layout Hotkey='$layout')" }

# Release test-typing.ps1 owns its own tray instance; do not attach to this one.
& (Join-Path $dir 'TextVN.exe') --stop | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'TextVN tray did not stop before typing test' }
Start-Sleep -Milliseconds 500
if (Get-Process -Name TextVN -ErrorAction SilentlyContinue) {
    throw 'TextVN tray is still running before typing test'
}

& (Join-Path $PSScriptRoot 'test-typing.ps1') -Dir $dir

# R2-16: ZIP khong co thu muc goc - nguoi dung "Extract here" vao Downloads thi
# thu muc chua ca file cua ho. uninstall.ps1 chi duoc xoa file TextVN biet ten.
$userFile = Join-Path $dir 'tai-lieu-cua-nguoi-dung.txt'
$userSub = Join-Path $dir 'thu-muc-rieng'
Set-Content -LiteralPath $userFile -Value 'khong duoc xoa'
New-Item -ItemType Directory -Path $userSub | Out-Null
Set-Content -LiteralPath (Join-Path $userSub 'anh.txt') -Value 'khong duoc xoa'

# Go dang ky nhu nguoi dung (uninstall.ps1 trong zip).
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $dir 'uninstall.ps1')
Start-Sleep -Seconds 1
if (-not (Test-Path -LiteralPath $userFile)) { throw 'uninstall.ps1 deleted a user file next to TextVN (R2-16)' }
if (-not (Test-Path -LiteralPath (Join-Path $userSub 'anh.txt'))) { throw 'uninstall.ps1 deleted a user folder next to TextVN (R2-16)' }
if (Test-Path -LiteralPath (Join-Path $dir 'textvn-cli.exe')) { throw 'uninstall.ps1 left textvn-cli.exe behind' }
Write-Host 'PASS portable uninstall keeps unrelated user files'
$layout = (Get-ItemProperty -Path $toggle -ErrorAction SilentlyContinue).'Layout Hotkey'
if ($layout -ne '2') { throw "uninstall.ps1 did not give Ctrl+Shift back (Layout Hotkey='$layout', expected '2' as before)" }
Write-Host 'PASS portable uninstall restored the Windows Ctrl+Shift hotkey to its previous value (R2-40)'
if (Get-Process -Name TextVN -ErrorAction SilentlyContinue) { throw 'tray still running after uninstall.ps1' }
if (Test-Path $inproc) { throw 'TSF CLSID still registered after uninstall.ps1' }
if (-not (Test-Path (Join-Path $env:APPDATA 'TextVN'))) { throw 'user config was not kept' }
Write-Host 'PASS portable: extract -> run -> type -> unregister'

# R2-92: nang cap tu portable <= 0.2.27 - ban cu dat Layout Hotkey=3 ma KHONG ghi marker
# (state.json co last_version cu). Ban moi phai nhan lai muc do de go cai dat van tra
# Ctrl + Shift (ban cu xoa Layout Hotkey khi go).
$stateFile = Join-Path $env:APPDATA 'TextVN\state.json'
$stateBackup = if (Test-Path -LiteralPath $stateFile) { [System.IO.File]::ReadAllText($stateFile) } else { $null }
$dir2 = Join-Path $tempRoot ('textvn-portable-up-' + [guid]::NewGuid().ToString('N'))
try {
    Expand-Archive -Path $Zip -DestinationPath $dir2
    Set-ItemProperty -Path $toggle -Name 'Layout Hotkey' -Value '3'
    Remove-Item -LiteralPath (Join-Path $env:APPDATA 'TextVN\ctrl_shift_default_applied') -Force -ErrorAction SilentlyContinue
    [System.IO.File]::WriteAllText($stateFile, '{"global_enabled":true,"apps":{},"last_version":"0.2.27"}')
    Start-Process -FilePath (Join-Path $dir2 'TextVN.exe') -ArgumentList '--autostart' -WorkingDirectory $dir2 | Out-Null
    $markerFile = Join-Path $env:APPDATA 'TextVN\ctrl_shift_default_applied'
    for ($i = 0; $i -lt 40 -and -not (Test-Path -LiteralPath $markerFile); $i++) { Start-Sleep -Milliseconds 500 }
    & (Join-Path $dir2 'TextVN.exe') --stop | Out-Null
    $markerText = (Get-Content -LiteralPath $markerFile -Raw -ErrorAction SilentlyContinue)
    if ($markerText -notmatch '(?m)^Layout Hotkey=\s*$') { throw "upgrade from 0.2.27 lost the legacy Layout Hotkey record (marker: [$markerText])" }
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $dir2 'uninstall.ps1')
    $layout = (Get-ItemProperty -Path $toggle -ErrorAction SilentlyContinue).'Layout Hotkey'
    if ($null -ne $layout) { throw "uninstall after upgrade from 0.2.27 did not give Ctrl+Shift back (Layout Hotkey='$layout')" }
    Write-Host 'PASS portable upgrade from 0.2.27: uninstall gives Ctrl+Shift back (R2-92)'
} finally {
    if ($null -ne $stateBackup) { [System.IO.File]::WriteAllText($stateFile, $stateBackup) }
    Set-ItemProperty -Path $toggle -Name 'Layout Hotkey' -Value '2'
    if (Test-Path -LiteralPath (Join-Path $dir2 'TextVN.exe')) { & (Join-Path $dir2 'TextVN.exe') --stop *> $null }
    Remove-Item -LiteralPath $dir2 -Recurse -Force -ErrorAction SilentlyContinue
}
} finally {
    $trayPath = Join-Path $dir 'TextVN.exe'
    if (Test-Path -LiteralPath $trayPath) {
        & $trayPath --stop *> $null
    }
    Start-Sleep -Milliseconds 300
    $ownedProcess = Get-Process -Name TextVN -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and $_.Path.StartsWith($dir, [System.StringComparison]::OrdinalIgnoreCase) }
    $registeredPath = (Get-ItemProperty -Path $inproc -ErrorAction SilentlyContinue).'(default)'
    $registeredHere = $registeredPath -and $registeredPath.StartsWith($dir, [System.StringComparison]::OrdinalIgnoreCase)
    if ((Test-Path -LiteralPath $dir) -and -not $ownedProcess -and -not $registeredHere) {
        $resolvedTemp = [System.IO.Path]::GetFullPath($tempRoot).TrimEnd('\') + '\'
        $resolvedDir = (Resolve-Path -LiteralPath $dir).Path
        if (-not $resolvedDir.StartsWith($resolvedTemp, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Unsafe portable test cleanup path: $resolvedDir"
        }
        # TSF da go dang ky nhung textvn-tsf.dll van nap trong tien trinh dang chay
        # (ctfmon, editor) toi khi chung thoat - dung nhu uninstall.ps1 bao nguoi
        # dung. Don tam la best-effort, khong lam fail kich ban da PASS.
        try {
            Remove-Item -LiteralPath $resolvedDir -Recurse -Force -ErrorAction Stop
            Write-Host 'PASS portable: temporary extraction cleaned'
        } catch {
            Write-Warning "Temporary extraction kept ($($_.Exception.Message)): $resolvedDir"
        }
    } elseif (Test-Path -LiteralPath $dir) {
        Write-Warning "Portable test files retained because TextVN or TSF registration still uses $dir"
    }
}
