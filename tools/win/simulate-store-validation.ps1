# SPDX-License-Identifier: GPL-3.0-or-later
# simulate-store-validation.ps1 - Mo phong DUNG flow cua Microsoft Store validator:
#
#   1. Download installer tu raw URL (MOTW tu dong gan vi download tu internet)
#   2. Chay installer voi switches tu form Store (/VERYSILENT /SUPPRESSMSGBOXES /NORESTART)
#   3. Kiem tra: silent install (exit 0, khong tuong tac), ARP entry, bundleware
#
# CHAY TRONG CI: runner GHA windows-latest la MOI TRUONG SACH (khong co TextVN
# truoc do)  -  giong VM cua Microsoft validator.
#
# Diem khac biet voi harness thuong: download qua Invoke-WebRequest de co MOTW
# (Mark of the Web  -  Windows gan vao file download tu internet, co the kich hoat
# SmartScreen/chinh sach tren VM)  -  dung nhu Microsoft lam.
#
# ASCII-only (G7/A5).
param(
    [string]$Url = "https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.20/TextVN-setup-0.2.20-windows-x64.exe",
    [string]$ExpectName = "TextVN",
    [string]$ExpectPublisher = "LinhBH.CoM",
    [string]$ExpectVersion = "0.2.20",
    [string]$Switches = "/VERYSILENT /SUPPRESSMSGBOXES /NORESTART"
)
$ErrorActionPreference = 'Stop'

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
                }
            }
        }
    }
    $list
}

Write-Host "==== SIMULATE MICROSOFT STORE VALIDATION ===="
Write-Host "URL: $Url"
Write-Host "Switches: $Switches"
Write-Host "Expect: name~'$ExpectName' publisher='$ExpectPublisher' version='$ExpectVersion'"
Write-Host ""

# ---- B0: Download (MOTW tu dong gan) ----
$rt = $env:RUNNER_TEMP
if (-not $rt) { $rt = $env:TEMP }
$tempDir = Join-Path $rt ('store-sim-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
$setupFile = Join-Path $tempDir 'TextVN-setup.exe'
Write-Host "[B0] Downloading installer (MOTW will be applied automatically)..."
Invoke-WebRequest -Uri $Url -OutFile $setupFile -UseBasicParsing
$fileSize = (Get-Item $setupFile).Length
if ($fileSize -lt 100000) { throw "Downloaded file too small: $fileSize bytes" }
Write-Host "  Downloaded: $fileSize bytes"

# Kiem tra MOTW
$zone = Get-Content $setupFile -Stream Zone.Identifier -ErrorAction SilentlyContinue
if ($zone) { Write-Host "  MOTW: present (Zone.Identifier found)" }
else { Write-Host "  MOTW: not found (file may be trusted)" }

$before = Get-ArpEntries
Write-Host ("  ARP before: {0} entries total" -f $before.Count)

# ---- B1: Silent install ----
Write-Host "[B1] Running silent install (no elevation, no interaction)..."
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$p = Start-Process -FilePath $setupFile -ArgumentList $Switches.Split(' ') -Wait -PassThru
$sw.Stop()
Write-Host ("  Exit code: {0}" -f $p.ExitCode)
Write-Host ("  Duration: {0:N1}s" -f $sw.Elapsed.TotalSeconds)
if ($p.ExitCode -ne 0) {
    throw "FAIL B1: Silent install exit code $($p.ExitCode) (expected 0)"
}
if ($sw.Elapsed.TotalSeconds -gt 120) {
    throw "FAIL B1: Silent install took too long ($([int]$sw.Elapsed.TotalSeconds)s) - possible UI blocking"
}
Write-Host "PASS B1: Silent install completed without interaction"

# ---- B2: Entry in add/remove programs ----
Write-Host "[B2] Checking Add/Remove Programs entry..."
$after = Get-ArpEntries
$newEntries = @($after | Where-Object {
    $k = $_
    -not ($before | Where-Object { $_.Root -eq $k.Root -and $_.Key -eq $k.Key })
})
Write-Host ("  New entries after install: {0}" -f $newEntries.Count)
$newEntries | ForEach-Object { Write-Host ("    - Root={0} Key={1}" -f $_.Root, $_.Key) ; Write-Host ("      Name='{0}' Publisher='{1}' Version='{2}'" -f $_.Name, $_.Publisher, $_.Version) }

$textvn = @($newEntries | Where-Object { $_.Name -like "*$ExpectName*" })
if ($textvn.Count -eq 0) {
    # Th?m chi ti?t ?? debug
    Write-Host "  DETAIL: khong tim thay entry moi chua '$ExpectName'"
    Write-Host "  Toan bo ARP sau cai dat:"
    $after | ForEach-Object { Write-Host ("    {0}: {1} / {2}" -f $_.Root, $_.Name, $_.Publisher) }
    throw "FAIL B2: Khong tim thay entry Add/Remove cho '$ExpectName'"
}
if ($textvn.Count -gt 1) { throw "FAIL B2: Nhieu entry ($($textvn.Count)) chua '$ExpectName'" }
$e = $textvn[0]
if ($e.Publisher -ne $ExpectPublisher) {
    throw "FAIL B2: Publisher '$($e.Publisher)' != '$ExpectPublisher'"
}
if ($e.Version -notlike "*$ExpectVersion*") {
    Write-Host ("  WARNING: Version '{0}' khong chua '{1}'" -f $e.Version, $ExpectVersion)
}
Write-Host "PASS B2: ARP entry Name='$($e.Name)' Publisher='$($e.Publisher)' Version='$($e.Version)' Root=$($e.Root)"

# ---- B3: Bundleware ----
Write-Host "[B3] Checking for bundleware..."
if ($newEntries.Count -gt 1) {
    Write-Host "FAIL B3: $($newEntries.Count) entries moi (bundleware detected):"
    $newEntries | ForEach-Object { Write-Host ("  - {0} / {1}" -f $_.Name, $_.Publisher) }
    throw "Bundleware detected"
}
Write-Host "PASS B3: Chi 1 entry moi (khong bundleware)"

# ---- B4: Silent uninstall (bonus) ----
Write-Host "[B4] Silent uninstall..."
if ($e.Uninstall) {
    $uninstCmd = $e.Uninstall -replace '^"', '' -replace '"$', ''
    $up = Start-Process -FilePath $uninstCmd -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') -Wait -PassThru
    Write-Host ("  Uninstall exit: {0}" -f $up.ExitCode)
    Start-Sleep -Seconds 2
    $left = Get-ArpEntries | Where-Object { $_.Key -eq $e.Key }
    if ($left) { throw "FAIL B4: ARP entry van con sau uninstall" }
    Write-Host "PASS B4: Uninstall sach (entry bien mat)"
}

# Cleanup
Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
Write-Host ""
Write-Host "==== KET QUA: TOAN BO VALIDATION PASS ===="
Write-Host "Package dat du 3 tieu chi cua Microsoft Store validator."
