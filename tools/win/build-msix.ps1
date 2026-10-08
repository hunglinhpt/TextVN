# SPDX-License-Identifier: GPL-3.0-or-later
# build-msix.ps1 - Goi MSIX du phong cua TextVN (khi ban .exe bi Store tu choi).
#
# Dac diem: FULL-TRUST (rescap:runFullTrust) - bo go TSF khong chay duoc trong
# sandbox; cai xong nguoi dung mo TextVN mot lan, app tu dang ky TSF nhu ban
# portable (0.2.16: tu de nghi dang ky pham vi may qua UAC neu Windows tu choi
# per-user).
#
# Vi sao nop UNSIGNED: Microsoft Store ky lai goi khi publish. Muon sideload de
# test thi phai ky bang cert tin cay (khong thuoc pham vi script nay).
#
# Publisher/Identity trong manifest PHAI khap dung "Identity" ma Partner Center
# cap khi reserve ten (xem docs/release/store-submission.md muc MSIX) - script
# nhan tham so -Publisher/-IdentityName de chinh.
#
# Cach dung:
#   powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\build-msix.ps1
#   ... -Version 0.2.16 -Publisher "CN=ABCDEF12-..." -IdentityName "12345LinhBH.TextVN"
#
# ASCII-only (quy tac G7/A5).

# Gia tri mac dinh = Partner Center cua chu tai khoan (docs/release/msix-submission.md):
#   Publisher ID (Account settings > Windows publisher ID) va Publisher display
#   name "LinhBH.CoM"; DisplayName PHAI trung ten da reserve ("TextVN") - Store
#   tu choi goi co DisplayName khong nam trong danh sach ten da reserve.
#   IdentityName (Product identity > Package/Identity/Name) CHUA co trong repo:
#   truyen -IdentityName hoac bien MSIX_IDENTITY_NAME; thieu thi goi chi de test.
param(
    [string]$Version = "",
    [string]$Publisher = "CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545",
    [string]$IdentityName = "",
    [string]$DisplayName = "TextVN",
    [string]$PublisherDisplay = "LinhBH.CoM",
    [string]$TargetDir = "target\x86_64-pc-windows-msvc\release",
    [string]$OutDir = "dist",
    # Release/nop Store: thieu IdentityName that -> FAIL thay vi canh bao.
    [switch]$RequireStoreIdentity
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $root

# CI/secret co the bom Identity THAT (Partner Center -> Product identity) qua
# bien moi truong - khi do khong can truyen -Publisher/-IdentityName:
#   MSIX_IDENTITY_NAME, MSIX_PUBLISHER, MSIX_PUBLISHER_DISPLAY, MSIX_DISPLAY_NAME
# Tham so truyen tuong minh luon thang bien moi truong.
if ($env:MSIX_IDENTITY_NAME -and -not $PSBoundParameters.ContainsKey('IdentityName')) { $IdentityName = $env:MSIX_IDENTITY_NAME.Trim() }
if ($env:MSIX_PUBLISHER -and -not $PSBoundParameters.ContainsKey('Publisher')) { $Publisher = $env:MSIX_PUBLISHER.Trim() }
if ($env:MSIX_PUBLISHER_DISPLAY -and -not $PSBoundParameters.ContainsKey('PublisherDisplay')) { $PublisherDisplay = $env:MSIX_PUBLISHER_DISPLAY.Trim() }
if ($env:MSIX_DISPLAY_NAME -and -not $PSBoundParameters.ContainsKey('DisplayName')) { $DisplayName = $env:MSIX_DISPLAY_NAME.Trim() }
$PlaceholderIdentity = 'LinhBH.CoM.TextVN'
if (-not $IdentityName) { $IdentityName = $PlaceholderIdentity }
# Package/Identity/Name: 3-50 ky tu [A-Za-z0-9.-] (schema AppxManifest ST_PackageName).
if ($IdentityName -notmatch '^[A-Za-z0-9.-]{3,50}$') { throw "IdentityName khong hop le: '$IdentityName'" }
if ($Publisher -notmatch '^CN=') { throw "Publisher phai bat dau bang 'CN=': '$Publisher'" }

# CI-04 (audit vong 4, muc 3.4.3.4): goi MSIX con Identity PLACEHOLDER se bi
# Partner Center tu choi ngay khi upload ("package identity mismatch").
# -RequireStoreIdentity (release khi da dat bien MSIX_IDENTITY_NAME) -> FAIL cung;
# con lai chi canh bao to de khong ai nop nham ban test.
if ($IdentityName -eq $PlaceholderIdentity) {
    if ($RequireStoreIdentity) {
        throw 'MSIX: thieu Package/Identity/Name that (-IdentityName hoac MSIX_IDENTITY_NAME) - khong build goi nop Store bang placeholder.'
    }
    Write-Host ''
    Write-Host '!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!'
    Write-Host '!! CANH BAO: MSIX dang dung IDENTITY PLACEHOLDER:                 !!'
    Write-Host "!!   IdentityName = $IdentityName"
    Write-Host "!!   Publisher    = $Publisher"
    Write-Host '!! KHONG nop file nay len Partner Center - se bi tu choi.          !!'
    Write-Host '!! Lay that tu Partner Center -> Product identity roi build lai:   !!'
    Write-Host '!!   tools\win\build-msix.ps1 -Publisher "CN=..." -IdentityName "..."'
    Write-Host '!! (hoac dat bien moi truong MSIX_PUBLISHER / MSIX_IDENTITY_NAME)  !!'
    Write-Host '!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!'
    Write-Host ''
}

# 1. Version: mac dinh lay tu Cargo.toml (workspace.package).
if (-not $Version) {
    $line = Select-String -Path 'Cargo.toml' -Pattern '^version = "([^"]+)"' | Select-Object -First 1
    if (-not $line) { throw 'Khong doc duoc version tu Cargo.toml' }
    $Version = $line.Matches[0].Groups[1].Value
}
$msixVersion = "$Version.0"   # manifest can 4 phan A.B.C.D

# 2. Payload.
$need = @(
    (Join-Path $TargetDir 'TextVN.exe'),
    (Join-Path $TargetDir 'textvn-cli.exe')
)
foreach ($f in $need) {
    if (-not (Test-Path $f)) { throw "Thieu $f - chay build-release.ps1 truoc (hoac -TargetDir dung)." }
}
# Ban build trong workspace dat ten textvn_win_tsf.dll; ban phat hanh (ZIP) dat
# ten textvn-tsf.dll - nhan ca hai de co the dong goi tu chinh ZIP release.
$dll = (Join-Path $TargetDir 'textvn_win_tsf.dll')
if (-not (Test-Path $dll)) { $dll = (Join-Path $TargetDir 'textvn-tsf.dll') }
if (-not (Test-Path $dll)) { throw "Thieu textvn_win_tsf.dll/textvn-tsf.dll trong $TargetDir" }

# 3. Tim makeappx.exe (PATH -> Windows Kits moi nhat theo x64).
$makeappx = (Get-Command makeappx.exe -ErrorAction SilentlyContinue | Select-Object -First 1).Source
if (-not $makeappx) {
    $kits = "${env:ProgramFiles(x86)}\Windows Kits\10\bin"
    if (Test-Path $kits) {
        $makeappx = Get-ChildItem "$kits\*\x64\makeappx.exe" -ErrorAction SilentlyContinue |
            Sort-Object FullName -Descending | Select-Object -First 1 -ExpandProperty FullName
    }
}
if (-not $makeappx) { throw 'makeappx.exe khong tim thay (can Windows SDK). CI windows-2022 co san.' }
Write-Host "makeappx: $makeappx"

# 4. Stage.
$stage = Join-Path $OutDir 'msix-stage'
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Path $stage | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stage 'Assets') | Out-Null

Copy-Item (Join-Path $TargetDir 'TextVN.exe') $stage
Copy-Item (Join-Path $TargetDir 'textvn-cli.exe') $stage
Copy-Item $dll (Join-Path $stage 'textvn-tsf.dll')
# Vong 14 (Zalo 32-bit): x86 DLL di theo package - bootstrap stage-out giai
# nen ra thu muc cai dat, app 32-bit (Zalo/Office x86) moi nap duoc TIP.
$x86Dll = Join-Path $TargetDir 'textvn-tsf-x86.dll'
if (Test-Path $x86Dll) {
    Copy-Item $x86Dll (Join-Path $stage 'textvn-tsf-x86.dll')
} else {
    Write-Host 'WARN: textvn-tsf-x86.dll khong co trong payload (app x86 se khong dung duoc TIP)'
}
# Vong 12 audit MSIX: tray staged (bootstrap stage-out) can resources (icon)
# + data (appdb.default.json) - truoc day payload thieu, tray staged thieu icon.
foreach ($pair in @(@('tray\resources', 'resources'), @('data', 'data'))) {
    $srcDir = $pair[0]; $dstName = $pair[1]
    if (Test-Path $srcDir) { Copy-Item $srcDir (Join-Path $stage $dstName) -Recurse -Force }
}
# Icon .ico: uu tien trong ZIP release (resources\textvn.ico) roi toi repo (tray\resources).
$ico = $null
foreach ($cand in @((Join-Path $TargetDir 'resources\textvn.ico'), 'tray\resources\textvn.ico')) {
    if (Test-Path $cand) { $ico = $cand; break }
}
if ($ico) { Copy-Item $ico (Join-Path $stage 'TextVN.ico') }

# Tai lieu + huong dan di kem (giong ban portable).
foreach ($doc in @('README.md', 'CHANGELOG.md', 'LICENSE', 'PRIVACY_POLICY.txt')) {
    if (Test-Path $doc) { Copy-Item $doc $stage }
}
Copy-Item 'installer\windows\portable\HUONG_DAN_SU_DUNG.txt' $stage
Copy-Item 'installer\windows\msix\Assets\*' (Join-Path $stage 'Assets')

# 5. Manifest tu template.
# Doc UTF-8 TUONG MINH: Windows PowerShell 5.1 `Get-Content -Raw` doc file UTF-8
# khong BOM theo ANSI (cp1252) -> Description tieng Viet trong goi 0.2.27 bi
# mojibake (UTF-8 bi ma hoa 2 lan), hien sai trong Settings > Apps va Store.
$tplPath = Join-Path $root 'installer\windows\msix\AppxManifest.xml'
$tpl = [System.IO.File]::ReadAllText($tplPath, (New-Object System.Text.UTF8Encoding($false)))
# Gia tri chen vao thuoc tinh/phan tu XML phai escape (& < > " ') - Publisher
# dang "CN=..., O=..." hay ten co '&' se lam manifest hong.
function Esc([string]$v) { return [System.Security.SecurityElement]::Escape($v) }
$manifest = $tpl.Replace('{{VERSION}}', (Esc $msixVersion)).
    Replace('{{PUBLISHER}}', (Esc $Publisher)).
    Replace('{{IDENTITY_NAME}}', (Esc $IdentityName)).
    Replace('{{DISPLAY_NAME}}', (Esc $DisplayName)).
    Replace('{{PUBLISHER_DISPLAY}}', (Esc $PublisherDisplay))
if ($manifest -match '\{\{[A-Z_]+\}\}') { throw "Manifest con placeholder chua thay: $($Matches[0])" }
# Chot chong mojibake: chuoi tieng Viet trong template phai con nguyen sau khi thay.
if (-not $manifest.Contains([string][char]0x1ED9)) { throw 'Manifest mat ky tu tieng Viet (encoding sai)' }
# Ghi UTF-8 (khong BOM) - template co tieng Viet trong Description.
[System.IO.File]::WriteAllText((Join-Path $stage 'AppxManifest.xml'), $manifest, (New-Object System.Text.UTF8Encoding($false)))

# 6. Dong goi (khong ky - Store ky lai).
if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Path $OutDir | Out-Null }
$out = Join-Path $OutDir "TextVN-$Version-windows-x64.msix"
if (Test-Path $out) { Remove-Item $out -Force }
& $makeappx pack /o /d $stage /p $out
if ($LASTEXITCODE -ne 0) { throw "makeappx pack failed ($LASTEXITCODE)" }
Remove-Item $stage -Recurse -Force

$hash = (Get-FileHash $out -Algorithm SHA256).Hash.ToLower()
Write-Host "MSIX: $out"
Write-Host "SHA256: $hash"
Write-Host "Luu y: MSIX nop Store KHONG can ky. Sideload test can ky cert rieng."
