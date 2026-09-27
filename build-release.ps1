# build-release.ps1 - Build release TextVN cho Windows
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Usage:
#   powershell -File build-release.ps1
#   powershell -File build-release.ps1 -SkipTests
#   powershell -File build-release.ps1 -BuildInstaller
#   powershell -File build-release.ps1 -Version "0.1.0"

param(
    [switch]$SkipTests,
    [switch]$BuildInstaller,
    [string]$Version = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Step($msg) { Write-Host "" ; Write-Host "== $msg ==" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host "   OK: $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "   WARN: $msg" -ForegroundColor Yellow }
function Write-Fail($msg) { Write-Host "   FAIL: $msg" -ForegroundColor Red; exit 1 }

# Xac dinh version tu Cargo.toml
if ($Version -eq "") {
    $cargoContent = Get-Content Cargo.toml -Raw
    if ($cargoContent -match 'version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"') {
        $Version = $Matches[1]
    } else {
        $Version = "0.1.0"
    }
}
Write-Step "TextVN Release Build v$Version"

$Target = "x86_64-pc-windows-msvc"
$ReleaseDir = "target\$Target\release"
$DistDir = "dist"

# Kiem tra rust target
Write-Step "Check Rust target $Target"
$installedTargets = rustup target list --installed 2>&1
if ($installedTargets -notcontains $Target) {
    Write-Warn "Target $Target chua cai - dang cai..."
    rustup target add $Target
    if ($LASTEXITCODE -ne 0) { Write-Fail "rustup target add FAIL" }
}
Write-Ok "Target $Target OK"

# Chay tests
if (-not $SkipTests) {
    Write-Step "Running tests --workspace"
    cargo test --workspace
    if ($LASTEXITCODE -ne 0) { Write-Fail "Tests FAIL" }
    Write-Ok "All tests PASS"
}

# Build release
Write-Step "Build release --workspace --target $Target"
cargo build --release --workspace --target $Target
if ($LASTEXITCODE -ne 0) { Write-Fail "cargo build release FAIL" }

Write-Ok "Build release DONE"

# Dung cac tien trinh TextVN dang chay de tranh file locked
Write-Step "Check running processes"
Get-Process -Name "TextVN", "textvn-hook", "textvn-cli", "textvn", "textvn-tray", "textvn-hook", "textvn" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300
Write-Ok "Process lock check DONE"

# Tao thu muc dist
New-Item -ItemType Directory -Force $DistDir | Out-Null

# Tao portable ZIP
Write-Step "Package portable ZIP"
$ZipName = "TextVN-portable-$Version-windows-x64"
$ZipDir  = "$DistDir\$ZipName"
if (Test-Path $ZipDir) { Remove-Item $ZipDir -Recurse -Force }
New-Item -ItemType Directory -Force $ZipDir | Out-Null

$BinFiles = @(
    @{ src = "TextVN.exe";           dst = "TextVN.exe" },
    @{ src = "textvn-hook.exe";      dst = "textvn-hook.exe" },
    @{ src = "textvn-cli.exe";       dst = "textvn-cli.exe" },
    @{ src = "textvn_win_tsf.dll";  dst = "textvn-tsf.dll" },
    @{ src = "textvn_ffi.dll";      dst = "textvn_ffi.dll" }
)

foreach ($entry in $BinFiles) {
    $src = "$ReleaseDir\$($entry.src)"
    $dst = "$ZipDir\$($entry.dst)"
    if (Test-Path $src) {
        try {
            Copy-Item $src $dst -ErrorAction Stop
            $sz = [math]::Round((Get-Item $src).Length / 1024)
            Write-Ok "$($entry.src) -> $($entry.dst) ($sz KB)"
        } catch {
            Write-Warn "$($entry.src) copy FAIL (file locked?): $_"
        }
    } else {
        Write-Warn "$($entry.src) not found (skip)"
    }
}

# Copy resources
$resSrc = "tray\resources"
$resDst = "$ZipDir\resources"
if (Test-Path $resSrc) {
    Copy-Item $resSrc $resDst -Recurse -Force
    Write-Ok "Copied TextVN tray resources"
}

# Copy docs
foreach ($doc in @("README.md", "CHANGELOG.md", "LICENSE")) {
    if (Test-Path $doc) {
        Copy-Item $doc "$ZipDir\"
        Write-Ok "Copied $doc"
    }
}

# Tao HUONG_DAN_SU_DUNG.txt
$quickstartContent = "=================================================================`r`n" +
    "TextVN (Bo go Tieng Viet hien dai, ma nguon mo)`r`n" +
    "=================================================================`r`n`r`n" +
    "1. SU DUNG NGAY (KHONG CAN CAI DAT):`r`n" +
    "   - Nhan dup chuot vao file TextVN.exe`r`n" +
    "   - Bang dieu khien TextVN se hien thi ngay tren man hinh.`r`n" +
    "   - Ban co the tuy chon Bang ma (Unicode, TCVN3, VNI...), Kieu go (Telex, VNI...), phim chuyen.`r`n" +
    "   - Khi nhan [Dong] hoac tat cua so, TextVN se thu gon vao khay he thong (System Tray).`r`n`r`n" +
    "2. CHUYEN DOI TIENG VIET / TIENG ANH:`r`n" +
    "   - Phim tat mac dinh: Ctrl + Shift (hoac Alt + Z)`r`n" +
    "   - Icon khay he thong:`r`n" +
    "       + Chu [V] mau TIM: Dang bat go Tieng Viet`r`n" +
    "       + Chu [E] mau XANH: Che do Tieng Anh (tat go Tieng Viet)`r`n`r`n" +
    "3. BANG DIEU KHIEN & MENU:`r`n" +
    "   - Click chuot phai hoac chuot trai vao icon khay he thong de mo Bảng điều khiển / Menu lua chon.`r`n" +
    "   - De thoat han ung dung: Chon [Ket thuc] tren Bang dieu khien hoac Menu khay he thong.`r`n`r`n" +
    "4. DANG KY TSF HE THONG (TUY CHON):`r`n" +
    "   - Neu ban muon tich hop Text Services Framework (TSF) vao Windows:`r`n" +
    "     Chay file install.ps1 bang PowerShell.`r`n" +
    "   - De go bo TSF: Chay file uninstall.ps1.`r`n"
[System.IO.File]::WriteAllText("$ZipDir\HUONG_DAN_SU_DUNG.txt", $quickstartContent, [System.Text.Encoding]::UTF8)
Write-Ok "Created HUONG_DAN_SU_DUNG.txt"

# Verify PE metadata cho toan bo binary
Write-Step "Verify PE metadata and VersionInfo"
foreach ($entry in $BinFiles) {
    $dst = "$ZipDir\$($entry.dst)"
    if (Test-Path $dst) {
        $vi = (Get-Item $dst).VersionInfo
        if ($vi.CompanyName -eq "hunglinhpt") {
            Write-Ok "$($entry.dst): CompanyName='$($vi.CompanyName)', Ver='$($vi.FileVersion)'"
        } else {
            Write-Warn "$($entry.dst): CompanyName='$($vi.CompanyName)' (expected 'hunglinhpt')"
        }
    }
}

# Tao install.ps1 script trong ZIP
$installContent = "# install.ps1 - Dang ky TextVN TSF TIP tuy chon`r`n" +
    "`$dir = Split-Path -Parent `$MyInvocation.MyCommand.Path`r`n" +
    "Write-Host 'Dang ky TextVN TSF TIP vao he thong (no-taskbar)...'`r`n" +
    "`$r = Start-Process -Wait -PassThru -FilePath `"`$dir\textvn-cli.exe`" -ArgumentList 'register --no-taskbar'`r`n" +
    "if (`$r.ExitCode -eq 0) {`r`n" +
    "    Write-Host 'Dang ky TSF thanh cong! Khoi dong TextVN...'`r`n" +
    "    Start-Process -FilePath `"`$dir\TextVN.exe`"`r`n" +
    "} else {`r`n" +
    "    Write-Host ('Dang ky TSF that bai, ma loi=' + `$r.ExitCode)`r`n" +
    "    Write-Host 'TextVN van chay binh thuong qua che do Hook...'`r`n" +
    "    Start-Process -FilePath `"`$dir\TextVN.exe`"`r`n" +
    "}`r`n"
[System.IO.File]::WriteAllText("$ZipDir\install.ps1", $installContent, [System.Text.Encoding]::ASCII)

$uninstallContent = "# uninstall.ps1 - Go dang ky TextVN TSF TIP`r`n" +
    "`$dir = Split-Path -Parent `$MyInvocation.MyCommand.Path`r`n" +
    "Write-Host 'Dung tien trinh TextVN...'`r`n" +
    "Start-Process -Wait -FilePath `"`$dir\TextVN.exe`" -ArgumentList '--stop' -ErrorAction SilentlyContinue`r`n" +
    "Start-Sleep -Milliseconds 500`r`n" +
    "Write-Host 'Huy dang ky TSF TIP...'`r`n" +
    "Start-Process -Wait -FilePath `"`$dir\textvn-cli.exe`" -ArgumentList 'unregister'`r`n" +
    "Write-Host 'Hoan tat. Cau hinh tai %APPDATA%\TextVN\ van duoc giu lai.'`r`n"
[System.IO.File]::WriteAllText("$ZipDir\uninstall.ps1", $uninstallContent, [System.Text.Encoding]::ASCII)

Write-Ok "install.ps1 + uninstall.ps1 created"

# Nen thanh ZIP
Start-Sleep -Milliseconds 600
$ZipPath = "$DistDir\$ZipName.zip"
if (Test-Path $ZipPath) { Remove-Item $ZipPath -Force }

$zipSuccess = $false
for ($attempt = 1; $attempt -le 5; $attempt++) {
    try {
        Compress-Archive -Path "$ZipDir\*" -DestinationPath $ZipPath -ErrorAction Stop
        $zipSuccess = $true
        break
    } catch {
        Write-Warn "Compress-Archive attempt $attempt failed ($($_)), retrying in 500ms..."
        Start-Sleep -Milliseconds 500
    }
}
if (-not $zipSuccess) {
    Write-Fail "Compress-Archive failed after 5 attempts"
}
$zipSz = [math]::Round((Get-Item $ZipPath).Length / 1024)
Write-Ok "ZIP: $ZipPath ($zipSz KB)"

# Tao ma bam SHA256 checksums de xac thuc an toan (Antivirus Whitelist standard)
Write-Step "Generate SHA256 Checksums"
$shaFile = "$DistDir\SHA256SUMS.txt"
$hashLines = @()
$fullZipDir = (Resolve-Path $ZipDir).Path
Get-ChildItem -Path $ZipDir -File -Recurse | ForEach-Object {
    $h = Get-FileHash -Path $_.FullName -Algorithm SHA256
    $rel = $_.FullName.Substring($fullZipDir.Length).TrimStart('\', '/')
    $hashLines += "$($h.Hash)  $rel"
}
$zipHash = Get-FileHash -Path $ZipPath -Algorithm SHA256
$hashLines += "$($zipHash.Hash)  $ZipName.zip"
[System.IO.File]::WriteAllLines($shaFile, $hashLines, [System.Text.Encoding]::ASCII)
Write-Ok "SHA256 checksums saved to $shaFile"

# Build installer Inno Setup (optional)
if ($BuildInstaller) {
    Write-Step "Build Inno Setup installer"
    $isccExe = Get-Command iscc.exe -ErrorAction SilentlyContinue
    if (-not $isccExe) {
        Write-Warn "iscc.exe not found - install Inno Setup 6 from https://jrsoftware.org/isdl.php"
    } else {
        $targetDirArg = "/DTargetDir=..\..\$ReleaseDir"
        iscc.exe "/DMyAppVersion=$Version" $targetDirArg "installer\windows\TextVN-setup.iss"
        $setupExe = "dist\TextVN-setup-$Version.exe"
        if (Test-Path "target\installer\TextVN-setup-$Version.exe") {
            Copy-Item "target\installer\TextVN-setup-$Version.exe" $setupExe -Force
            Write-Ok "Installer: $setupExe"
        } elseif (Test-Path "Output\TextVN-setup.exe") {
            Move-Item "Output\TextVN-setup.exe" $setupExe -Force
            Write-Ok "Installer: $setupExe"
        } else {
            Write-Warn "Installer output not found"
        }
    }
}

Write-Step "Build COMPLETE"
Write-Host ""
Write-Host "Output:" -ForegroundColor White
Write-Host "  Portable ZIP : $ZipPath" -ForegroundColor Green
if ($BuildInstaller) {
    $installerPath = "$DistDir\TextVN-setup-$Version.exe"
    if (Test-Path $installerPath) {
        Write-Host "  Installer    : $installerPath" -ForegroundColor Green
    }
}
Write-Host ""
Write-Host "To test portable build:" -ForegroundColor White
Write-Host "  Extract $ZipPath and double-click TextVN.exe" -ForegroundColor Gray
