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

param(
    [string]$Version = "",
    [string]$Publisher = "CN=LinhBH.CoM",
    [string]$IdentityName = "LinhBH.CoM.TextVN",
    [string]$DisplayName = "TextVN - Bo go tieng Viet",
    [string]$PublisherDisplay = "LinhBH.CoM",
    [string]$TargetDir = "target\x86_64-pc-windows-msvc\release",
    [string]$OutDir = "dist"
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $root

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
$tpl = Get-Content 'installer\windows\msix\AppxManifest.xml' -Raw
$manifest = $tpl.Replace('{{VERSION}}', $msixVersion).
    Replace('{{PUBLISHER}}', $Publisher).
    Replace('{{IDENTITY_NAME}}', $IdentityName).
    Replace('{{DISPLAY_NAME}}', $DisplayName).
    Replace('{{PUBLISHER_DISPLAY}}', $PublisherDisplay)
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
