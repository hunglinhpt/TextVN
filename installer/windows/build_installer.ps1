# SPDX-License-Identifier: GPL-3.0-or-later
# PowerShell build script cho VietIME Windows Installer (WIN-054).
# Tuan thu Rule G7: ASCII-only. Chay bang: powershell -NoProfile -ExecutionPolicy Bypass -File build_installer.ps1

[CmdletBinding()]
param(
    [string]$Profile = "release",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoRoot = Resolve-Path "$ScriptDir\..\.."
Set-Location $RepoRoot

Write-Host "=== VietIME Windows Installer Builder ===" -ForegroundColor Cyan
Write-Host "Repo Root: $RepoRoot"
Write-Host "Build Profile: $Profile"

# 1. Build cac thanh phan Windows neu chua co
if (-not $SkipBuild) {
    Write-Host "`n[1/3] Building Rust binaries..." -ForegroundColor Yellow
    $CargoArgs = @("build", "-p", "vietime-cli", "-p", "vietime-win-tsf", "-p", "vietime-win-hook", "-p", "vietime-tray")
    if ($Profile -eq "release") {
        $CargoArgs += "--release"
    }
    & cargo @CargoArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Cargo build failed with exit code $LASTEXITCODE"
        exit 1
    }
}

# 2. Kiem tra su ton tai cua cac artifacts
Write-Host "`n[2/3] Checking artifacts..." -ForegroundColor Yellow
$TargetDir = Join-Path $RepoRoot "target\$Profile"
$RequiredFiles = @(
    "vietime.exe",
    "vietime-tray.exe",
    "vietime-hook.exe",
    "vietime_win_tsf.dll"
)

foreach ($f in $RequiredFiles) {
    $p = Join-Path $TargetDir $f
    if (-not (Test-Path $p)) {
        Write-Error "Missing required binary: $p"
        exit 1
    }
    $size = (Get-Item $p).Length
    Write-Host "  Found: $f ($size bytes)" -ForegroundColor Green
}

# 3. Kiem tra va goi Inno Setup Compiler (ISCC)
Write-Host "`n[3/3] Checking Inno Setup Compiler (ISCC)..." -ForegroundColor Yellow
$IsccPaths = @(
    "iscc.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles}\Inno Setup 6\ISCC.exe"
)

$FoundIscc = $null
foreach ($path in $IsccPaths) {
    if (Get-Command $path -ErrorAction SilentlyContinue) {
        $FoundIscc = $path
        break
    }
    if (Test-Path $path) {
        $FoundIscc = $path
        break
    }
}

$IssScript = Join-Path $ScriptDir "vietime-setup.iss"

if ($FoundIscc) {
    Write-Host "Compiling installer using: $FoundIscc" -ForegroundColor Green
    & $FoundIscc "$IssScript"
    if ($LASTEXITCODE -eq 0) {
        Write-Host "`nInstaller built successfully!" -ForegroundColor Green
    } else {
        Write-Error "ISCC compilation failed with code $LASTEXITCODE"
        exit 1
    }
} else {
    Write-Host "ISCC not found on this system. Inno Setup script is verified at: $IssScript" -ForegroundColor Yellow
    Write-Host "To compile installer manually, install Inno Setup 6: winget install JRSoftware.InnoSetup" -ForegroundColor Gray
}

Write-Host "`nDone." -ForegroundColor Cyan
