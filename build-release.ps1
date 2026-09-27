# build-release.ps1 - Build release VietIME cho Windows
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Usage:
#   powershell -File build-release.ps1
#   powershell -File build-release.ps1 -SkipTests
#   powershell -File build-release.ps1 -BuildInstaller
#   powershell -File build-release.ps1 -Version "0.2.0"

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
        $Version = "0.0.0"
    }
}
Write-Step "VietIME Release Build v$Version"

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

# Build tray voi icon nhung
Write-Step "Build tray with embedded icon (feature embed-resources)"
try {
    $null = cargo build --release -p vietime-tray --target $Target --features embed-resources 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Ok "embed-resources OK"
    } else {
        Write-Warn "embed-resources FAIL (exit $LASTEXITCODE) - using default icon fallback"
    }
} catch {
    Write-Warn "embed-resources skipped: $_"
}

Write-Ok "Build release DONE"

# Tao thu muc dist
New-Item -ItemType Directory -Force $DistDir | Out-Null

# Tao portable ZIP
Write-Step "Package portable ZIP"
$ZipName = "vietime-portable-$Version-windows-x64"
$ZipDir  = "$DistDir\$ZipName"
if (Test-Path $ZipDir) { Remove-Item $ZipDir -Recurse -Force }
New-Item -ItemType Directory -Force $ZipDir | Out-Null

$BinFiles = @(
    @{ src = "vietime.exe";          dst = "vietime.exe" },
    @{ src = "vietime-tray.exe";     dst = "vietime-tray.exe" },
    @{ src = "vietime_win_tsf.dll";  dst = "vietime-tsf.dll" },
    @{ src = "vietime_ffi.dll";      dst = "vietime_ffi.dll" },
    @{ src = "vietime-hook.exe";     dst = "vietime-hook.exe" }
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

# Copy docs
foreach ($doc in @("README.md", "CHANGELOG.md", "LICENSE")) {
    if (Test-Path $doc) {
        Copy-Item $doc "$ZipDir\"
        Write-Ok "Copied $doc"
    }
}

# Tao install.ps1 script trong ZIP
$installContent = "# install.ps1 - Install VietIME portable`r`n" +
    "`$dir = Split-Path -Parent `$MyInvocation.MyCommand.Path`r`n" +
    "Write-Host 'Registering VietIME TSF TIP...'`r`n" +
    "`$r = Start-Process -Wait -PassThru -FilePath `"`$dir\vietime.exe`" -ArgumentList 'register'`r`n" +
    "if (`$r.ExitCode -eq 0) {`r`n" +
    "    Write-Host 'Registration OK! Starting tray...'`r`n" +
    "    Start-Process -FilePath `"`$dir\vietime-tray.exe`"`r`n" +
    "} else {`r`n" +
    "    Write-Host ('FAIL exit=' + `$r.ExitCode)`r`n" +
    "}`r`n"
[System.IO.File]::WriteAllText("$ZipDir\install.ps1", $installContent, [System.Text.Encoding]::ASCII)

$uninstallContent = "# uninstall.ps1 - Uninstall VietIME portable`r`n" +
    "`$dir = Split-Path -Parent `$MyInvocation.MyCommand.Path`r`n" +
    "Write-Host 'Stopping tray...'`r`n" +
    "Start-Process -Wait -FilePath `"`$dir\vietime-tray.exe`" -ArgumentList '--stop' -ErrorAction SilentlyContinue`r`n" +
    "Start-Sleep -Milliseconds 500`r`n" +
    "Write-Host 'Unregistering TSF TIP...'`r`n" +
    "Start-Process -Wait -FilePath `"`$dir\vietime.exe`" -ArgumentList 'unregister'`r`n" +
    "Write-Host 'Done. Config in %APPDATA%\VietIME\ is kept.'`r`n"
[System.IO.File]::WriteAllText("$ZipDir\uninstall.ps1", $uninstallContent, [System.Text.Encoding]::ASCII)

Write-Ok "install.ps1 + uninstall.ps1 created"

# Nen thanh ZIP
$ZipPath = "$DistDir\$ZipName.zip"
if (Test-Path $ZipPath) { Remove-Item $ZipPath }
Compress-Archive -Path "$ZipDir\*" -DestinationPath $ZipPath
$zipSz = [math]::Round((Get-Item $ZipPath).Length / 1024)
Write-Ok "ZIP: $ZipPath ($zipSz KB)"

# Build installer Inno Setup (optional)
if ($BuildInstaller) {
    Write-Step "Build Inno Setup installer"
    $isccExe = Get-Command iscc.exe -ErrorAction SilentlyContinue
    if (-not $isccExe) {
        Write-Warn "iscc.exe not found - install Inno Setup 6 from https://jrsoftware.org/isdl.php"
    } else {
        iscc.exe "/DMyAppVersion=$Version" "installer\windows\vietime-setup.iss"
        $setupExe = "dist\vietime-setup-$Version.exe"
        if (Test-Path "Output\vietime-setup.exe") {
            Move-Item "Output\vietime-setup.exe" $setupExe -Force
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
    $installerPath = "$DistDir\vietime-setup-$Version.exe"
    if (Test-Path $installerPath) {
        Write-Host "  Installer    : $installerPath" -ForegroundColor Green
    }
}
Write-Host ""
Write-Host "To test portable build:" -ForegroundColor White
Write-Host "  Expand-Archive $ZipPath -DestinationPath .\test-portable" -ForegroundColor Gray
Write-Host "  .\test-portable\vietime.exe doctor" -ForegroundColor Gray
