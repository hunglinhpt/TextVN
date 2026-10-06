# SPDX-License-Identifier: GPL-3.0-or-later
# test-installer.ps1 -Setup <TextVN-setup-*.exe>
#
# Hai kich ban cai dat:
#   (1) PER-USER (mac dinh cua nguoi dung, khong /ALLUSERS): dang ky TSF per-user
#       phai THANH CONG khong can admin (bug "phai chay bang admin" 2026-10-01).
#   (2) /ALLUSERS: machine profile + user activation -> kiem tra file, TSF, Run key
#       -> go tieng Viet that trong Notepad -> go cai dat im lang -> khong con file/dang ky,
#       cau hinh nguoi dung van con.

param(
    [Parameter(Mandatory = $true)][string]$Setup
)
$ErrorActionPreference = 'Stop'
$clsid = '{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}'
$inproc = "HKLM:\Software\Classes\CLSID\$clsid\InprocServer32"
$userInproc = "HKCU:\Software\Classes\CLSID\$clsid\InprocServer32"
$app = Join-Path $env:ProgramFiles 'TextVN'
$log = Join-Path $env:TEMP 'textvn-setup.log'

# Phim tat doi bo cuc cua Windows ve mac dinh (Ctrl+Shift) de kiem tac vu "Danh Ctrl + Shift".
$toggle = 'HKCU:\Keyboard Layout\Toggle'
if (-not (Test-Path $toggle)) { New-Item -Path $toggle -Force | Out-Null }
Set-ItemProperty -Path $toggle -Name 'Layout Hotkey' -Value '2'

function Show-RegisterLogTail {
    # Exit 10 = textvn-cli register that bai (GetCustomSetupExitCode); setup chay
    # CLI an console nen ly do chi nam trong register.log.
    $regLog = Join-Path $env:LOCALAPPDATA 'TextVN\logs\register.log'
    if (Test-Path $regLog) {
        Write-Host '--- register.log ---'
        Get-Content $regLog -ErrorAction SilentlyContinue | Select-Object -Last 60
    }
}

# ---- Kich ban (1): PER-USER - khong /ALLUSERS, khong elevation ----
# /CURRENTUSER + /DIR. Runner GHA chay elevated va Inno co the mac dinh admin
# mode bat chap /CURRENTUSER ({autopf} ve Program Files a refusal + exit 10):
# phat hien qua dong Log "refused elevated TSF registration" thi SKIP co kiem
# soat a duong per-user that su da duoc portable scenario phu (TextVN.exe tu
# dang ky user scope); may non-admin thi {autopf}={userpf} khop va nhanh per-user
# cua .iss duoc test that.
$userApp = Join-Path $env:LOCALAPPDATA 'Programs\TextVN'
$p = Start-Process -FilePath $Setup -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/CURRENTUSER', "/DIR=$userApp", "/LOG=$log") -Wait -PassThru
$setupLog = (Get-Content $log -ErrorAction SilentlyContinue | Out-String)
if ($p.ExitCode -eq 10 -and $setupLog -match 'refused elevated TSF registration outside Program Files') {
    Write-Host 'NOTE runner elevated: Inno admin mode despite /CURRENTUSER - per-user installer path skipped here (portable scenario covers user-scope registration)'
    $pu = Start-Process -FilePath (Join-Path $userApp 'unins000.exe') -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') -Wait -PassThru
    for ($i = 0; $i -lt 60 -and (Test-Path (Join-Path $userApp 'TextVN.exe')); $i++) { Start-Sleep -Milliseconds 500 }
    if (Test-Path $userInproc) { throw 'skipped per-user probe left HKCU registration behind' }
} elseif ($p.ExitCode -ne 0) {
    Get-Content $log -ErrorAction SilentlyContinue | Select-Object -Last 40
    Show-RegisterLogTail
    throw "per-user setup exit $($p.ExitCode)"
} else {
    foreach ($f in @('TextVN.exe', 'textvn-cli.exe', 'textvn-tsf.dll', 'unins000.exe')) {
        if (-not (Test-Path (Join-Path $userApp $f))) { throw "per-user installed file missing: $f" }
    }
    # 0.2.19: silent install = pure file copy (khong dang ky TSF trong installer
    # de tranh tre API TSF trong sandbox validator). Mo TextVN mot lan -> tu
    # dang ky - dung luong that cua nguoi dung sau khi cai tu Store.
    Start-Process -FilePath (Join-Path $userApp 'TextVN.exe') -WorkingDirectory $userApp | Out-Null
    $selfReg = $false
    for ($i = 0; $i -lt 40 -and -not $selfReg; $i++) {
        Start-Sleep -Milliseconds 500
        $v = (Get-ItemProperty -Path $userInproc -ErrorAction SilentlyContinue).'(default)'
        $selfReg = ($v -eq (Join-Path $userApp 'textvn-tsf.dll'))
    }
    if (-not $selfReg) { throw 'TextVN.exe first run did not self-register TSF per-user' }
    Write-Host 'PASS TextVN.exe first run self-registered TSF per-user'
    Stop-Process -Name TextVN -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500
    $v = (Get-ItemProperty -Path $userInproc -ErrorAction SilentlyContinue).'(default)'
    if ($v -ne (Join-Path $userApp 'textvn-tsf.dll')) { throw "per-user TSF CLSID not registered to installed DLL (got '$v')" }
    if (Test-Path $inproc) { throw 'per-user install must not write machine HKLM COM registration' }
    & (Join-Path $userApp 'textvn-cli.exe') register status | Out-Host
    if ($LASTEXITCODE -ne 0) { throw 'per-user TSF registration is not usable without admin' }
    $pu = Start-Process -FilePath (Join-Path $userApp 'unins000.exe') -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') -Wait -PassThru
    for ($i = 0; $i -lt 60 -and (Test-Path (Join-Path $userApp 'TextVN.exe')); $i++) { Start-Sleep -Milliseconds 500 }
    if (Test-Path (Join-Path $userApp 'TextVN.exe')) { throw 'per-user TextVN.exe still present after uninstall' }
    if (Test-Path $userInproc) { throw 'per-user COM registration left behind after uninstall' }
    Write-Host 'PASS silent per-user install + TSF registration WITHOUT admin (files, TSF, uninstall)'
}

# ---- Kich ban (2): /ALLUSERS - machine + user activation ----
$p = Start-Process -FilePath $Setup -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/ALLUSERS', "/LOG=$log") -Wait -PassThru
if ($p.ExitCode -ne 0) {
    Get-Content $log -ErrorAction SilentlyContinue | Select-Object -Last 40
    Show-RegisterLogTail
    throw "setup exit $($p.ExitCode)"
}
foreach ($f in @('TextVN.exe', 'textvn-cli.exe', 'textvn-tsf.dll', 'unins000.exe')) {
    if (-not (Test-Path (Join-Path $app $f))) { throw "installed file missing: $f" }
}
# 0.2.19: silent install = pure file copy; TextVN.exe first run tu dang ky TSF
# per-user (HKCU COM -> DLL trong Program Files). Machine COM (HKLM) chi xuat
# hien sau khi nguoi dung chap nhan UAC promotion trong app (offer_machine_...).
Start-Process -FilePath (Join-Path $app 'TextVN.exe') -WorkingDirectory $app | Out-Null
$selfReg = $false
for ($i = 0; $i -lt 40 -and -not $selfReg; $i++) {
    Start-Sleep -Milliseconds 500
    $v = (Get-ItemProperty -Path $userInproc -ErrorAction SilentlyContinue).'(default)'
    $selfReg = ($v -eq (Join-Path $app 'textvn-tsf.dll'))
}
if (-not $selfReg) { throw 'TextVN.exe first run did not self-register TSF (HKCU -> installed DLL)' }
Write-Host 'PASS TextVN.exe first run self-registered TSF per-user (from machine install)'
Stop-Process -Name TextVN -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500
$v = (Get-ItemProperty -Path $userInproc -ErrorAction SilentlyContinue).'(default)'
if ($v -ne (Join-Path $app 'textvn-tsf.dll')) { throw "TSF CLSID not registered to installed DLL (got '$v')" }
$run = (Get-ItemProperty -Path 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Run' -ErrorAction SilentlyContinue).TextVN
if (-not $run -or $run -notlike '*TextVN.exe*--autostart*') { throw "autostart Run key missing (got '$run')" }
$layout = (Get-ItemProperty -Path $toggle -ErrorAction SilentlyContinue).'Layout Hotkey'
if ($layout -eq '2') { throw 'installer did not free Ctrl+Shift from the Windows layout hotkey' }
& (Join-Path $app 'textvn-cli.exe') register status | Out-Host
if ($LASTEXITCODE -ne 0) { throw 'TSF profile is not enabled for the installing user' }
$wowInproc = (Get-ItemProperty 'Registry::HKEY_CLASSES_ROOT\WOW6432Node\CLSID\{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}\InprocServer32' -ErrorAction SilentlyContinue).'(default)'
if (-not $wowInproc -or $wowInproc -notlike '*textvn-tsf-x86.dll') { throw "WOW6432Node COM (32-bit view) missing or not pointing at x86 DLL (got '$wowInproc')" }
Write-Host 'PASS WOW6432Node COM registered to textvn-tsf-x86.dll (Zalo/Office 32-bit)'
Write-Host 'PASS silent machine install + user TSF activation (files, TSF, autostart, Ctrl+Shift for TextVN)'

& (Join-Path $PSScriptRoot 'test-typing.ps1') -Dir $app

$u = Start-Process -FilePath (Join-Path $app 'unins000.exe') -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') -Wait -PassThru
# Trinh go cai dat chay ban sao trong %TEMP% roi thoat: cho no xoa xong.
for ($i = 0; $i -lt 60 -and (Test-Path (Join-Path $app 'TextVN.exe')); $i++) { Start-Sleep -Milliseconds 500 }
if (Test-Path (Join-Path $app 'TextVN.exe')) { throw 'TextVN.exe still present after uninstall' }
if (Test-Path $inproc) { throw 'TSF CLSID still registered after uninstall' }

# Vong 13 (ghost uninstall): quet moi key/value con chua GUID TIP sau khi go -
# bat that: go 0.2.19 con sot 27 path (Assemblies Default + InputMethodOverride
# + HKLM machine tree). Cai machine chay elevated nen phai SACH ca HKLM.
$guid = '6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8'
function Test-TipGhost {
    param([string[]]$Roots)
    $hits = @()
    foreach ($r in $Roots) {
        if (-not (Test-Path $r)) { continue }
        $keys = Get-ChildItem $r -Recurse -ErrorAction SilentlyContinue |
            Where-Object { $_.PSChildName -like "*$guid*" -or $_.Name -like "*$guid*" }
        foreach ($k in $keys) {
            $p = Get-ItemProperty $k.PSPath -ErrorAction SilentlyContinue
            $valHit = $p -and ($p.PSObject.Properties | Where-Object {
                $_.Name -notlike 'PS*' -and ($_.Value -is [string]) -and ($_.Value -like "*$guid*")
            })
            if ($valHit) { $hits += $k.Name }
        }
    }
    return $hits
}
$ghostUser = Test-TipGhost @(
    'HKCU:\Software\Classes\CLSID',
    'HKCU:\Software\Classes\WOW6432Node\CLSID',
    'HKCU:\Software\Microsoft\CTF',
    'HKCU:\Software\WOW6432Node\Microsoft\CTF',
    'HKCU:\Control Panel\International\User Profile'
)
if ($ghostUser.Count -gt 0) { throw ("ghost HKCU keys with TIP GUID left after uninstall: " + ($ghostUser -join '; ')) }
$ghostMachine = Test-TipGhost @(
    'HKLM:\SOFTWARE\Classes\CLSID',
    'HKLM:\SOFTWARE\WOW6432Node\CLSID',
    'HKLM:\SOFTWARE\Microsoft\CTF\TIP',
    'HKLM:\SOFTWARE\WOW6432Node\Microsoft\CTF\TIP'
)
if ($ghostMachine.Count -gt 0) { throw ("ghost HKLM keys with TIP GUID left after machine uninstall: " + ($ghostMachine -join '; ')) }
$imo = (Get-ItemProperty 'HKCU:\Control Panel\International\User Profile' -ErrorAction SilentlyContinue).InputMethodOverride
if ($imo -and $imo -like "*$guid*") { throw 'InputMethodOverride still points at TextVN TIP after uninstall' }
Write-Host 'PASS no TIP ghost keys left after uninstall (HKCU + HKLM + InputMethodOverride)'

$run = (Get-ItemProperty -Path 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Run' -ErrorAction SilentlyContinue).TextVN
if ($run) { throw 'autostart Run key left behind' }
if (Test-Path (Join-Path $app 'textvn-tsf.dll')) {
    Write-Host 'NOTE textvn-tsf.dll still loaded by a running app; Windows removes it after sign-out'
}
if (-not (Test-Path (Join-Path $env:APPDATA 'TextVN'))) { throw 'user config was not kept' }
Write-Host 'PASS installer: install -> type -> uninstall'
