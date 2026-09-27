# build-release.ps1 - Build release TextVN cho Windows
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Usage:
#   powershell -File build-release.ps1
#   powershell -File build-release.ps1 -SkipTests
#   powershell -File build-release.ps1 -BuildInstaller
#   powershell -File build-release.ps1 -IncludeCompatibilityHook
#   powershell -File build-release.ps1 -Version "0.1.0"
#   powershell -File build-release.ps1 -Channel candidate

param(
    [switch]$SkipTests,
    [switch]$BuildInstaller,
    # Hook WH_KEYBOARD_LL legacy chi co trong goi Compatibility duoc yeu cau.
    [switch]$IncludeCompatibilityHook,
    [string]$Version = "",
    [ValidateSet("candidate", "production")]
    [string]$Channel = "candidate"
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
$BuildStartedUtc = (Get-Date).ToUniversalTime().ToString("o")
$BuildId = (Get-Date).ToUniversalTime().ToString("yyyyMMddHHmmss")
$GitCommit = (git rev-parse HEAD 2>$null)
if ($LASTEXITCODE -ne 0) { $GitCommit = "unknown" }
$GitDirtyOutput = (git status --porcelain 2>$null)
$GitTreeClean = ($LASTEXITCODE -eq 0 -and [string]::IsNullOrWhiteSpace(($GitDirtyOutput -join "`n")))

# Nhung hang muc nay phai duoc chung minh tren may Windows that truoc khi
# artifact co the duoc goi la production. Unit/corpus test khong thay the UIA,
# TSF composition, DACL pipe hay ky phat hanh.
$ProductionBlockers = @(
    "Native UI Automation is not integrated in the TSF/Hook runtime",
    "TSF composition lifecycle is not verified end-to-end",
    "Named-pipe DACL and Authenticode release signing are not production-verified"
)
$ReleaseChecks = [ordered]@{}
$FeatureProfile = if ($IncludeCompatibilityHook) { "tsf-plus-legacy-hook" } else { "tsf-only" }

# Kiem tra rust target
Write-Step "Check Rust target $Target"
$installedTargets = rustup target list --installed 2>&1
if ($installedTargets -notcontains $Target) {
    Write-Warn "Target $Target chua cai - dang cai..."
    rustup target add $Target
    if ($LASTEXITCODE -ne 0) { Write-Fail "rustup target add FAIL" }
}
Write-Ok "Target $Target OK"

# Release gate: moi build deu chay cac kiem tra tinh, ABI va corpus. `-SkipTests`
# chi bo test workspace de ho tro chan doan cuc bo; artifact van la candidate.
Write-Step "Running release checks"
cargo fmt --check
if ($LASTEXITCODE -ne 0) { Write-Fail "cargo fmt --check FAIL" }
$ReleaseChecks["format"] = "passed"

$CargoWorkspaceScope = @("--workspace")
if (-not $IncludeCompatibilityHook) {
    # Release mac dinh khong build/phan phoi raw global hook. Day la giam
    # false-positive theo kien truc, khong che giau hay ne AV.
    $CargoWorkspaceScope += @("--exclude", "textvn-win-hook")
}

cargo clippy @CargoWorkspaceScope --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { Write-Fail "cargo clippy FAIL" }
$ReleaseChecks["clippy"] = "passed"

cargo run -q -p textvn-cli -- verify
if ($LASTEXITCODE -ne 0) { Write-Fail "ABI verify FAIL" }
$ReleaseChecks["abi_verify"] = "passed"

cargo run -q -p textvn-cli -- replay corpus/shared corpus/win --adapter win
if ($LASTEXITCODE -ne 0) { Write-Fail "Windows corpus replay FAIL" }
$ReleaseChecks["windows_corpus"] = "passed"

if (-not $SkipTests) {
    Write-Step "Running tests --workspace"
    cargo test @CargoWorkspaceScope
    if ($LASTEXITCODE -ne 0) { Write-Fail "Tests FAIL" }
    Write-Ok "All tests PASS"
    $ReleaseChecks["workspace_tests"] = "passed"
} else {
    $ReleaseChecks["workspace_tests"] = "skipped"
}

# Build release
Write-Step "Build release --workspace --target $Target"
cargo build --release @CargoWorkspaceScope --target $Target
if ($LASTEXITCODE -ne 0) { Write-Fail "cargo build release FAIL" }

Write-Ok "Build release DONE"

# Khong duoc tu y tat bo go dang dung cua nguoi dung. Neu artifact dang bi khoa,
# dung build va yeu cau dong instance truoc khi tiep tuc.
Write-Step "Check running TextVN processes"
$runningTextVN = @(
    Get-Process -Name "TextVN", "textvn-hook", "textvn-cli", "textvn", "textvn-tray" -ErrorAction SilentlyContinue
)
if ($runningTextVN.Count -gt 0) {
    $runningIds = ($runningTextVN | Select-Object -ExpandProperty Id) -join ", "
    Write-Fail "TextVN process(es) still running (PID: $runningIds). Close them, then build again."
}
$ReleaseChecks["no_running_textvn_processes"] = "passed"
Write-Ok "No TextVN process is running"

# Runtime gate on the exact release binary, before it is packaged. It verifies
# the Windows tray can start, expose its IPC endpoint, and shut down gracefully.
Write-Step "Windows runtime smoke test"
$trayExe = Join-Path $ReleaseDir "TextVN.exe"
if (-not (Test-Path $trayExe)) { Write-Fail "Runtime smoke binary missing: $trayExe" }
$smokeProcess = Start-Process -FilePath $trayExe -ArgumentList "--autostart" -WorkingDirectory $ReleaseDir -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 2
$smokeStatus = (& $trayExe --status 2>&1 | Out-String)
if ($smokeStatus -notmatch "TextVN IPC Server: RUNNING") {
    & $trayExe --stop *> $null
    Write-Fail "Runtime smoke did not expose a running IPC server: $smokeStatus"
}
$smokeStop = (& $trayExe --stop 2>&1 | Out-String)
Start-Sleep -Milliseconds 500
if (Get-Process -Id $smokeProcess.Id -ErrorAction SilentlyContinue) {
    & $trayExe --stop *> $null
    Write-Fail "Runtime smoke did not stop cleanly: $smokeStop"
}
$ReleaseChecks["windows_runtime_smoke"] = "passed"
if ($IncludeCompatibilityHook) {
    $ReleaseChecks["compatibility_hook"] = "included-on-explicit-request"
    Write-Ok "Start and IPC status PASS (Compatibility Hook is opt-in)"
} else {
    $ReleaseChecks["compatibility_hook"] = "excluded-from-default-release"
    Write-Ok "Start and IPC status PASS (TSF-only release; legacy Hook excluded)"
}

# Tao thu muc dist
New-Item -ItemType Directory -Force $DistDir | Out-Null

# Tao portable ZIP
Write-Step "Package portable ZIP"
$ZipName = "TextVN-portable-$Version-windows-x64-$BuildId"
$ZipDir  = "$DistDir\$ZipName"
# Artifact bat bien: khong xoa release cu vi Windows/Explorer co the dang giu
# mot EXE trong do. Moi build nhan Build ID rieng de co the truy vet va rollback.
if (Test-Path $ZipDir) { Write-Fail "Artifact directory already exists: $ZipDir" }
New-Item -ItemType Directory -Force $ZipDir | Out-Null

$BinFiles = @(
    @{ src = "TextVN.exe";           dst = "TextVN.exe" },
    @{ src = "textvn-cli.exe";       dst = "textvn-cli.exe" },
    @{ src = "textvn_win_tsf.dll";  dst = "textvn-tsf.dll" },
    @{ src = "textvn_ffi.dll";      dst = "textvn_ffi.dll" }
)
if ($IncludeCompatibilityHook) {
    $BinFiles += @{ src = "textvn-hook.exe"; dst = "textvn-hook.exe" }
}

foreach ($entry in $BinFiles) {
    $src = "$ReleaseDir\$($entry.src)"
    $dst = "$ZipDir\$($entry.dst)"
    if (-not (Test-Path $src)) { Write-Fail "Required release binary is missing: $src" }
    try {
        Copy-Item $src $dst -ErrorAction Stop
        $sz = [math]::Round((Get-Item $src).Length / 1024)
        Write-Ok "$($entry.src) -> $($entry.dst) ($sz KB)"
    } catch {
        Write-Fail "$($entry.src) copy FAIL (file locked?): $_"
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
    "   - Click chuot phai hoac chuot trai vao icon khay he thong de mo Bang dieu khien / Menu lua chon.`r`n" +
    "   - De thoat han ung dung: Chon [Ket thuc] tren Bang dieu khien hoac Menu khay he thong.`r`n`r`n" +
    "4. DANG KY TSF HE THONG (TUY CHON):`r`n" +
    "   - Neu ban muon tich hop Text Services Framework (TSF) vao Windows:`r`n" +
    "     Chay file install.ps1 bang PowerShell.`r`n" +
    "   - De go bo TSF: Chay file uninstall.ps1.`r`n"
[System.IO.File]::WriteAllText("$ZipDir\HUONG_DAN_SU_DUNG.txt", $quickstartContent, [System.Text.Encoding]::UTF8)
Write-Ok "Created HUONG_DAN_SU_DUNG.txt"

# Evidence report travels inside every ZIP. It makes the release state explicit
# and prevents a locally-built candidate from being misrepresented as production.
$ReleaseReport = [ordered]@{
    schema_version = 1
    product = "TextVN"
    version = $Version
    build_id = $BuildId
    channel_requested = $Channel
    build_started_utc = $BuildStartedUtc
    git_commit = $GitCommit.Trim()
    source_tree_clean = $GitTreeClean
    target = $Target
    feature_profile = $FeatureProfile
    status = "release-candidate"
    checks = $ReleaseChecks
    production_blockers = $ProductionBlockers
}
[System.IO.File]::WriteAllText(
    "$ZipDir\RELEASE_REPORT.json",
    ($ReleaseReport | ConvertTo-Json -Depth 5),
    [System.Text.Encoding]::UTF8
)
Write-Ok "Created RELEASE_REPORT.json (release-candidate)"

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
    "    Write-Host 'TextVN su dung TSF. Kiem tra loi dang ky o tren neu chua go duoc.'`r`n" +
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
        $installerArgs = @("/DMyAppVersion=$Version", $targetDirArg)
        if ($IncludeCompatibilityHook) { $installerArgs += "/DIncludeCompatibilityHook=1" }
        $installerArgs += "installer\windows\TextVN-setup.iss"
        iscc.exe @installerArgs
        $setupExe = "dist\TextVN-setup-$Version-windows-x64.exe"
        if (Test-Path $setupExe) {
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
    $installerPath = "$DistDir\TextVN-setup-$Version-windows-x64.exe"
    if (Test-Path $installerPath) {
        Write-Host "  Installer    : $installerPath" -ForegroundColor Green
    }
}
Write-Host ""
Write-Host "To test portable build:" -ForegroundColor White
Write-Host "  Extract $ZipPath and double-click TextVN.exe" -ForegroundColor Gray

if ($Channel -eq "production") {
    Write-Host "Production promotion blocked:" -ForegroundColor Yellow
    $ProductionBlockers | ForEach-Object { Write-Host "  - $_" -ForegroundColor Yellow }
    exit 2
}
