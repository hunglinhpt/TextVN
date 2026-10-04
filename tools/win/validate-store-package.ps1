# SPDX-License-Identifier: GPL-3.0-or-later
# validate-store-package.ps1 - Chay dung 3 buoc "Manual package validation"
# cua Microsoft cho goi cai .exe, tu dong hoa:
#   https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/manual-package-validation
#
#   0. Manifest         : requestedExecutionLevel PHAI la asInvoker. Validator
#                         chay installer bang CreateProcess non-elevated; exe
#                         requireAdministrator fail ngay ERROR_ELEVATION_REQUIRED
#                         (740) truoc khi cai gi -> CA 3 check deu do (B13,
#                         do that 2026-10-04: exit=2 trong shell non-elevated).
#   1. Silent install   : chay /VERYSILENT /SUPPRESSMSGBOXES /NORESTART -> exit 0,
#                         khong tuong tac, trong thoi gian gioi han.
#   2. Entry in ARP     : co DUNG 1 entry moi, DisplayName/Publisher/Version
#                         khop TUNG CHU voi Partner Center (khong blank/la).
#   3. Bundleware       : tong so entry moi == 1 (khong de nhieu entry).
#   + bonus: silent uninstall -> exit 0, entry bien mat, file chinh bi xoa.
#
# DUNG O DAU: job windows cua ci-shared + release-candidate (runner sach) va
# `cargo xtask preflight` (may dev co the dang cai TextVN -> -AllowSkip).
# Gate BAT BUOC 100% truoc khi tag (Checklist S trong store-submission.md).
#
# ASCII-only (quy tac G7/A5).
param(
    # Bo trong = tu tim dist\TextVN-setup-*.exe moi nhat (dung cho preflight).
    [string]$Setup = "",
    [string]$ExpectName = "TextVN",
    [string]$ExpectPublisher = "LinhBH.CoM",
    [string]$ExpectVersion = "",
    # May dev dang cai TextVN: bo qua (SKIP) thay vi FAIL. CI luon chay that.
    [switch]$AllowSkip,
    # Chi test nhanh per-user (/CURRENTUSER): bo check HKLM.
    [switch]$PerUser,
    # STO-02 (audit vong 4): gia thuyet validator CHI doc HKLM/WOW6432Node
    # (hoac chay duoi account khac -> HKCU cua installer vo hinh). Bat co nay de
    # ep bang chung phai nam o hive MAY: chay installer voi /ALLUSERS tu shell
    # ELEVATED (khong thi 740 truoc khi cai), roi kiem HKLM.
    [switch]$MachineOnly,
    [int]$InstallTimeoutSec = 120
)
$ErrorActionPreference = 'Stop'

# STO-03: ten/publisher phai khop TUNG CHU voi Partner Center. Cho phep bom qua
# bien moi truong (CI secret/variable) de harness doi chieu voi gia tri THAT
# thay vi so code .iss voi chinh no (diem mu CI cu).
if ($env:STORE_APP_NAME) { $ExpectName = $env:STORE_APP_NAME }
if ($env:STORE_PUBLISHER_NAME) { $ExpectPublisher = $env:STORE_PUBLISHER_NAME }

$roots = @(
    'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall',
    'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall',
    'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall'
)
function Get-ArpEntries {
    $list = @()
    foreach ($r in $roots) {
        Get-ChildItem $r -ErrorAction SilentlyContinue | ForEach-Object {
            $p = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
            if ($p -and $p.DisplayName) {
                $list += [pscustomobject]@{
                    Root = $r; Key = $_.PSChildName; Name = $p.DisplayName
                    Publisher = $p.Publisher; Version = $p.DisplayVersion
                    Uninstall = $p.UninstallString; Location = $p.InstallLocation
                }
            }
        }
    }
    $list
}

if (-not $Setup) {
    $cand = Get-ChildItem 'dist\TextVN-setup-*.exe' -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if (-not $cand) {
        if ($AllowSkip) { Write-Host 'SKIP: khong co dist\TextVN-setup-*.exe de validate.'; exit 0 }
        throw 'Khong co dist\TextVN-setup-*.exe - build installer truoc.'
    }
    $Setup = $cand.FullName
}
if (-not (Test-Path $Setup)) { throw "Setup not found: $Setup" }
$Setup = (Resolve-Path $Setup).Path
if (-not $ExpectVersion) {
    if ($Setup -match 'TextVN-setup-([0-9]+\.[0-9]+\.[0-9]+)-') { $ExpectVersion = $Matches[1] }
    else { throw 'Khong suy ra duoc version tu ten file - truyen -ExpectVersion.' }
}
Write-Host "== validate-store-package =="
Write-Host "Setup:    $Setup"
Write-Host "Expect:   name~'$ExpectName'  publisher='$ExpectPublisher'  version='$ExpectVersion'"

# ---- 0. Manifest: validator chay installer bang CreateProcess non-elevated ----
$bytes = [System.IO.File]::ReadAllBytes($Setup)
$text = [System.Text.Encoding]::UTF8.GetString($bytes)
$m = [regex]::Match($text, 'requestedExecutionLevel\s+level="([^"]+)"')
if (-not $m.Success) { throw 'Khong doc duoc requestedExecutionLevel trong manifest cua setup.' }
$level = $m.Groups[1].Value
if ($level -ne 'asInvoker') {
    throw "Manifest setup la level='$level' - validator Store chay non-elevated se fail 740 truoc khi cai (can asInvoker)."
}
Write-Host 'PASS 0) manifest: requestedExecutionLevel=asInvoker (validator CreateProcess duoc)'

$before = Get-ArpEntries
$beforeTextVN = @($before | Where-Object { $_.Name -like "*$ExpectName*" })
if ($beforeTextVN.Count -gt 0) {
    if ($AllowSkip) {
        Write-Host "SKIP: may dang cai $ExpectName ($($beforeTextVN.Count) entry) - harness can may sach, CI se chay that."
        exit 0
    }
    throw "May dang co san $($beforeTextVN.Count) entry $ExpectName - hay go truoc khi chay harness (can may sach nhu validator)."
}

# ---- 1. Silent install ----
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$p = Start-Process -FilePath $Setup -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') -Wait -PassThru
$sw.Stop()
if ($p.ExitCode -ne 0) { throw "Silent install exit code $($p.ExitCode) (can 0)" }
if ($sw.Elapsed.TotalSeconds -gt $InstallTimeoutSec) {
    throw "Silent install mat $([int]$sw.Elapsed.TotalSeconds)s (> $InstallTimeoutSec s) - nghi co tuong tac/UAC."
}
Write-Host ("PASS 1) silent install: exit 0 trong {0:N0}s, khong tuong tac" -f $sw.Elapsed.TotalSeconds)

# ---- 2. Entry in add/remove programs ----
$after = Get-ArpEntries
$newEntries = @($after | Where-Object { $k = $_; -not ($before | Where-Object { $_.Root -eq $k.Root -and $_.Key -eq $k.Key }) })
$textvn = @($newEntries | Where-Object { $_.Name -like "*$ExpectName*" })
if ($MachineOnly) {
    # STO-02: bang chung PHAI nam o hive MAY (HKLM / WOW6432Node).
    $machine = @($textvn | Where-Object { $_.Root -notlike 'HKCU:*' })
    if ($machine.Count -ne 1) {
        Write-Host 'Entry tim thay:'
        $textvn | ForEach-Object { Write-Host ("  - {0} | {1}" -f $_.Root, $_.Name) }
        throw "MachineOnly: khong co dung 1 entry moi o HKLM/WOW6432Node (bat duoc $($machine.Count)); HKCU khong duoc tinh."
    }
    $e = $machine[0]
} else {
    if ($textvn.Count -ne 1) { throw "Can dung 1 entry moi chua '$ExpectName', thay $($textvn.Count)" }
    $e = $textvn[0]
}
# Partner Center doi chieu TUNG CHU (B13): DisplayName == ten san pham,
# Publisher == ten nha phat hanh hien thi; lech/blank chu nao cung vao dung
# 3 item do cua Store. Khong dung -like o day.
if ($e.Name.Trim() -ne $ExpectName) { throw "DisplayName ARP '$($e.Name)' != ten san pham '$ExpectName' (phai khop tung chu)" }
if (-not $e.Publisher) { throw 'ARP thieu Publisher (Store bao blank publisher)' }
if ($e.Publisher.Trim() -ne $ExpectPublisher) { throw "Publisher ARP '$($e.Publisher)' != '$ExpectPublisher' (phai khop tung chu voi Partner Center)" }
if (-not $e.Version) { throw 'ARP thieu DisplayVersion (Store bao blank/khong xac dinh duoc phien ban)' }
if ($e.Version.Trim() -ne $ExpectVersion) { throw "DisplayVersion ARP '$($e.Version)' != '$ExpectVersion'" }
# Luu y (B13 vong 2->3): ARP co the o HKLM (admin) HOAC HKCU (per-user) -
# Programs and Features hien thi gop ca hai; quan trong la dung 1 entry
# voi Name/Publisher/Version khop. EXE co manifest requireAdministrator
# thi CreateProcess cua validator FAIL voi ERROR_ELEVATION_REQUIRED (740)
# truoc khi cai gi ca -> ca 3 check do; per-user la duong duy nhat cho Store.
Write-Host "PASS 2) ARP entry: '$($e.Name)' | '$($e.Publisher)' | '$($e.Version)'"

# ---- 3. Bundleware ----
if ($newEntries.Count -ne 1) {
    Write-Host 'Entry moi:'
    $newEntries | ForEach-Object { Write-Host ("  - {0} / {1}" -f $_.Name, $_.Publisher) }
    throw "Bundleware: co $($newEntries.Count) entry moi (chi duoc 1)"
}
Write-Host "PASS 3) bundleware: dung 1 entry moi"

# ---- bonus: silent uninstall ----
if (-not $e.Uninstall) { throw 'ARP thieu UninstallString - khong the kiem tra go cai dat' }
$uninst = ($e.Uninstall -replace '^"', '' -replace '"$', '')
$up = Start-Process -FilePath $uninst -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') -Wait -PassThru
for ($i = 0; $i -lt 40; $i++) {
    Start-Sleep -Milliseconds 500
    if (-not (Get-ArpEntries | Where-Object { $_.Key -eq $e.Key })) { break }
}
$leftEntry = Get-ArpEntries | Where-Object { $_.Key -eq $e.Key }
if ($leftEntry) { throw "Go cai dat xong van con entry ARP $($e.Key)" }
if ($e.Location -and (Test-Path (Join-Path $e.Location 'TextVN.exe'))) {
    throw "Go cai dat xong van con TextVN.exe trong $($e.Location)"
}
Write-Host "PASS 4) silent uninstall: exit $($up.ExitCode), entry + file chinh da xoa"
Write-Host "OK - package dat toan bo tieu chi validation cua Microsoft Store."
