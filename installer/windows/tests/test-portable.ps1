# SPDX-License-Identifier: GPL-3.0-or-later
# test-portable.ps1 -Zip <TextVN-portable-*.zip>
#
# Kich ban "giai nen ra dung luon": giai nen vao thu muc tam -> chay TextVN.exe (tu dang
# ky TSF per-user tu chinh thu muc do) -> go tieng Viet that trong Notepad -> uninstall.ps1
# -> dang ky TSF da go, cau hinh nguoi dung van con.
#
# LUU Y MOI TRUONG (B7, 2026-10-03): runner CI chay ELEVATED nen register trong buoc nay
# that ra ghi ca HKLM (RegisterProfile API thanh cong voi admin) — kich ban KHONG chung
# minh duoc typing voi dang ky THUAN per-user tren may thuong. Tren Windows 11 24H2+
# (build 26300), per-user thuan co the bi tu choi ActivateProfile — xem
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
foreach ($f in @('TextVN.exe', 'textvn-cli.exe', 'textvn-tsf.dll', 'install.ps1', 'uninstall.ps1', 'HUONG_DAN_SU_DUNG.txt', 'RELEASE_REPORT.json')) {
    if (-not (Test-Path (Join-Path $dir $f))) { throw "portable zip is missing $f" }
}
Write-Host "PASS zip layout ($dir)"

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

# Release test-typing.ps1 owns its own tray instance; do not attach to this one.
& (Join-Path $dir 'TextVN.exe') --stop | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'TextVN tray did not stop before typing test' }
Start-Sleep -Milliseconds 500
if (Get-Process -Name TextVN -ErrorAction SilentlyContinue) {
    throw 'TextVN tray is still running before typing test'
}

& (Join-Path $PSScriptRoot 'test-typing.ps1') -Dir $dir

# Go dang ky nhu nguoi dung (uninstall.ps1 trong zip).
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $dir 'uninstall.ps1')
Start-Sleep -Seconds 1
if (Get-Process -Name TextVN -ErrorAction SilentlyContinue) { throw 'tray still running after uninstall.ps1' }
if (Test-Path $inproc) { throw 'TSF CLSID still registered after uninstall.ps1' }
if (-not (Test-Path (Join-Path $env:APPDATA 'TextVN'))) { throw 'user config was not kept' }
Write-Host 'PASS portable: extract -> run -> type -> unregister'
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
