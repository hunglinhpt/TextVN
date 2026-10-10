# build-release.ps1 - Build release TextVN cho Windows
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Usage:
#   powershell -File build-release.ps1
#   powershell -File build-release.ps1 -SkipTests
#   powershell -File build-release.ps1 -BuildInstaller
#   powershell -File build-release.ps1 -IncludeCompatibilityHook
#   powershell -File build-release.ps1 -SigningCertificateThumbprint "<SHA1>"
#   powershell -File build-release.ps1 -Version "0.2.0"
#   powershell -File build-release.ps1 -Channel candidate

param(
    [switch]$SkipTests,
    [switch]$BuildInstaller,
    # Build them BIEN THE MAY (PrivilegesRequired=admin, ARP o HKLM) - goi danh
    # cho Package validation cua Microsoft Store (vong 5: validator khong thay
    # entry HKCU). File ra: TextVN-setup-<ver>-windows-x64-machine.exe.
    [switch]$MachineInstaller,
    # Hook WH_KEYBOARD_LL legacy chi co trong goi Compatibility duoc yeu cau.
    [switch]$IncludeCompatibilityHook,
    # SHA-1 thumbprint cua Authenticode code-signing certificate trong CurrentUser\My
    # hoac LocalMachine\My. Khong ghi private key vao repo hay artifact.
    [string]$SigningCertificateThumbprint = "",
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

function Resolve-SignTool {
    $command = Get-Command "signtool.exe" -ErrorAction SilentlyContinue
    if ($null -ne $command) { return $command.Source }

    $candidates = @(
        "${env:ProgramFiles(x86)}\Windows Kits\10\bin\x64\signtool.exe",
        "${env:ProgramFiles}\Windows Kits\10\bin\x64\signtool.exe"
    )
    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) { return $candidate }
    }
    return $null
}

function Sign-TextVNFile([string]$SignTool, [string]$Thumbprint, [string]$Path) {
    & $SignTool sign /sha1 $Thumbprint /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $Path
    if ($LASTEXITCODE -ne 0) { Write-Fail "Authenticode signing failed: $Path" }
    $signature = Get-AuthenticodeSignature -FilePath $Path
    if ($signature.Status -ne "Valid") {
        Write-Fail "Authenticode signature is not valid for $Path (status: $($signature.Status))"
    }
    Write-Ok "Authenticode signed: $(Split-Path -Leaf $Path)"
}

# ISCC (Inno Setup 6): winget cai per-user vao %LOCALAPPDATA%\Programs nen
# `Get-Command` (PATH) khong thay - truoc day build 0.2.19/0.2.20 that bai o
# buoc installer du moi buoc khac xanh. Do tung ung vien nhu Resolve-SignTool.
function Resolve-Iscc {
    $command = Get-Command "iscc.exe" -ErrorAction SilentlyContinue
    if ($null -ne $command) { return $command.Source }

    $candidates = @(
        "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles}\Inno Setup 6\ISCC.exe"
    )
    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) { return $candidate }
    }
    return $null
}

# Xac dinh version tu Cargo.toml
if ($Version -eq "") {
    $cargoContent = Get-Content Cargo.toml -Raw
    # Neo dau dong = [workspace.package] version; khong fallback so cung
    # (review R3: fallback 0.1.0 gay version-skew im lang).
    if ($cargoContent -match '(?m)^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"') {
        $Version = $Matches[1]
    } else {
        throw "Cannot read workspace version from Cargo.toml"
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
    "Named-pipe DACL is not production-verified"
)
$SigningRequested = -not [string]::IsNullOrWhiteSpace($SigningCertificateThumbprint)
if (-not $SigningRequested) {
    $ProductionBlockers += "Authenticode release signing was not requested"
}
$ReleaseChecks = [ordered]@{}
$FeatureProfile = if ($IncludeCompatibilityHook) { "tsf-plus-legacy-hook" } else { "tsf-only" }

# Kiem tra rust target - ca i686 cho TIP DLL x86 (Vong 14): thieu target nay
# thi buoc build x86 phia duoi moi fail, sau khi da chay het test/clippy.
$TargetX86 = "i686-pc-windows-msvc"
Write-Step "Check Rust targets $Target, $TargetX86"
$installedTargets = rustup target list --installed 2>&1
foreach ($t in @($Target, $TargetX86)) {
    if ($installedTargets -notcontains $t) {
        Write-Warn "Target $t chua cai - dang cai..."
        rustup target add $t
        if ($LASTEXITCODE -ne 0) { Write-Fail "rustup target add $t FAIL" }
    }
    Write-Ok "Target $t OK"
}

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
cargo run -q -p textvn-cli -- replay corpus/shared corpus/win --adapter tsf
if ($LASTEXITCODE -ne 0) { Write-Fail "TSF composition corpus replay FAIL" }
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
cargo build --release --locked @CargoWorkspaceScope --target $Target
if ($LASTEXITCODE -ne 0) { Write-Fail "cargo build release FAIL" }

# Vong 14 (Zalo 32-bit): build them TIP DLL x86 - app 32-bit (Zalo, Office x86) khong the nap DLL 64-bit (ERROR_BAD_EXE_FORMAT 193). CRT tinh (khong
# phu thuoc VC redist x86 tren may nguoi dung).
Write-Step "Build TIP DLL x86 (WOW64: Zalo/Office 32-bit)"
$env:RUSTFLAGS = "-C target-feature=+crt-static"
cargo build --release --locked --target $TargetX86 -p textvn-win-tsf
if ($LASTEXITCODE -ne 0) { Write-Fail "cargo build x86 TIP FAIL" }
Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue
$x86Dll = "target/$TargetX86/release/textvn_win_tsf.dll"
if (-not (Test-Path $x86Dll)) { Write-Fail "x86 TIP DLL missing: $x86Dll" }
# Gate: PE machine phai la I386 (0x014C)
$peBytes = [System.IO.File]::ReadAllBytes($x86Dll)
$peOff = [BitConverter]::ToInt32($peBytes, 0x3C)
$machine = [BitConverter]::ToUInt16($peBytes, $peOff + 4)
# Ngoac quanh -f: khong co thi "-f" bi truyen nhu tham so cho Write-Fail va
# thong bao in nguyen chuoi "0x{0:X4}".
if ($machine -ne 0x014C) { Write-Fail ("x86 TIP DLL wrong machine: 0x{0:X4}" -f $machine) }
$x86Dst = Join-Path $ReleaseDir "textvn-tsf-x86.dll"
# DLL x86 dang duoc TSF nap trong app 32-bit (Zalo/Office) - Windows tu choi
# xoa file image-mapped; doi ten file cu (Windows CHO PHEP rename) roi copy (R2-88).
# Ban .old-* khong vao goi: $BinFiles liet ke dich danh tung file.
try {
    Copy-Item $x86Dll $x86Dst -Force -ErrorAction Stop
} catch {
    if (Test-Path $x86Dst) {
        $backup = "$x86Dst.old-$(Get-Date -Format 'yyyyMMddHHmmss')"
        Rename-Item -LiteralPath $x86Dst -NewName (Split-Path $backup -Leaf) -Force
        Copy-Item $x86Dll $x86Dst -Force
        Write-Warn "x86 TIP DLL dang bi nap (app 32-bit dang mo) - da rename ban cu thanh .old-*"
    } else {
        Write-Fail "x86 TIP DLL copy FAIL: $_"
    }
}
Write-Ok "x86 TIP DLL: textvn-tsf-x86.dll (I386, CRT static)"

Write-Ok "Build release DONE"

# R2-18: moi PE phat hanh phai link CRT tinh (.cargo/config.toml +crt-static) -
# TIP DLL nap vao moi tien trinh, may thieu VC++ redist thi khong nap duoc.
Write-Step "Check PE imports (khong phu thuoc VC++ runtime)"
$PeFiles = @("$ReleaseDir\TextVN.exe", "$ReleaseDir\textvn-cli.exe", "$ReleaseDir\textvn_win_tsf.dll", "$ReleaseDir\textvn-tsf-x86.dll")
if ($IncludeCompatibilityHook) { $PeFiles += "$ReleaseDir\textvn-hook.exe" }
$py = Get-Command python -ErrorAction SilentlyContinue
if ($null -eq $py) { $py = Get-Command python3 -ErrorAction SilentlyContinue }
if ($null -eq $py) {
    Write-Warn "python khong co - bo qua check-pe-imports (CI luon chay)"
    $ReleaseChecks["pe_static_crt"] = "skipped"
} else {
    & $py.Source tools\win\check-pe-imports.py @PeFiles
    if ($LASTEXITCODE -ne 0) { Write-Fail "PE con phu thuoc VC++ runtime (xem tren)" }
    $ReleaseChecks["pe_static_crt"] = "passed"
}

# Ky tat ca PE file phan phoi truoc runtime smoke va truoc khi dong goi. Ky la
# cach phat hanh chuan de xay dung reputation; khong co co che che giau binary.
# Gom ca TIP DLL x86: no duoc nap vao moi tien trinh 32-bit (Zalo, Office x86)
# nen thieu chu ky la DLL "la" duy nhat trong goi da ky (CR-36).
$ReleaseSignFiles = @(
    "$ReleaseDir\TextVN.exe",
    "$ReleaseDir\textvn-cli.exe",
    "$ReleaseDir\textvn_win_tsf.dll",
    "$ReleaseDir\textvn-tsf-x86.dll"
)
if ($IncludeCompatibilityHook) { $ReleaseSignFiles += "$ReleaseDir\textvn-hook.exe" }
if ($SigningRequested) {
    Write-Step "Sign release binaries with Authenticode"
    $signTool = Resolve-SignTool
    if ($null -eq $signTool) { Write-Fail "signtool.exe was not found; install the Windows SDK signing tools or remove -SigningCertificateThumbprint" }
    foreach ($binary in $ReleaseSignFiles) {
        if (-not (Test-Path $binary)) { Write-Fail "Cannot sign missing binary: $binary" }
        Sign-TextVNFile $signTool $SigningCertificateThumbprint $binary
    }
    $ReleaseChecks["authenticode"] = "passed"
} elseif ($env:SIGNPATH_API_TOKEN) {
    # Ky qua SignPath Foundation (goi Open Source, mien phi) - chi khi secret
    # duoc cau hinh; khong co thi giu nguyen release-candidate chua ky.
    # Xem docs/release/code-signing-plan.md
    Write-Step "Sign release binaries with SignPath"
    foreach ($binary in $ReleaseSignFiles) {
        if (-not (Test-Path $binary)) { Write-Fail "Cannot sign missing binary: $binary" }
    }
    # R2-72: MOT signing request (ZIP, deep sign) cho ca bo binary = mot lan duyet.
    powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\sign-signpath.ps1 -File ($ReleaseSignFiles -join ',')
    if ($LASTEXITCODE -ne 0) { Write-Fail "SignPath signing failed" }
    $ReleaseChecks["authenticode"] = "signpath"
} else {
    $ReleaseChecks["authenticode"] = "not-signed"
}

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
$smokeStop = ""
# Vong 13: tray tao window SAU khi dang ky TSF/activate (1-3s tren may moi don
# HKLM) - lenh --stop goi qua som se in "No running TextVN instance detected".
# Retry toi 15s: coi la ok khi window da co mat (output khac thong bao do).
$stopOk = $false
for ($i = 0; $i -lt 30; $i++) {
    Start-Sleep -Milliseconds 500
    $smokeStop = (& $trayExe --stop 2>&1 | Out-String)
    if ($smokeStop -notmatch "No running TextVN instance detected") {
        $stopOk = $true
        break
    }
}
if (-not $stopOk) {
    Stop-Process -Id $smokeProcess.Id -Force -ErrorAction SilentlyContinue
    Write-Fail "Runtime smoke did not stop cleanly within 15s: $smokeStop"
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
    @{ src = "textvn-tsf-x86.dll";  dst = "textvn-tsf-x86.dll" }
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

# Huong dan + script cai/go cua ban portable: file that trong installer\windows\portable
# (review duoc, test duoc) thay vi chuoi sinh trong script nay.
foreach ($f in @("HUONG_DAN_SU_DUNG.txt", "install.ps1", "uninstall.ps1")) {
    Copy-Item "installer\windows\portable\$f" "$ZipDir\" -ErrorAction Stop
}
Write-Ok "Copied HUONG_DAN_SU_DUNG.txt, install.ps1, uninstall.ps1"

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
        # R2-86: build.rs chi canh bao khi thieu rc.exe/rc loi -> binary mat VERSIONINFO
        # va manifest nhung van "build xanh". Phat hanh thi bat buoc dung metadata.
        if ($vi.CompanyName -eq "LinhBH.CoM" -and $vi.FileVersion -eq "$Version.0") {
            Write-Ok "$($entry.dst): CompanyName='$($vi.CompanyName)', Ver='$($vi.FileVersion)'"
        } else {
            Write-Fail "$($entry.dst): CompanyName='$($vi.CompanyName)', FileVersion='$($vi.FileVersion)' (expected 'LinhBH.CoM', '$Version.0') - rc.exe thieu hoac loi?"
        }
    }
}

# Nen thanh ZIP
Start-Sleep -Milliseconds 600
$ZipPath = "$DistDir\$ZipName.zip"
if (Test-Path $ZipPath) { Write-Fail "Artifact ZIP already exists: $ZipPath" }

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
$shaFile = "$DistDir\SHA256SUMS-$BuildId.txt"
if (Test-Path $shaFile) { Write-Fail "Checksum file already exists: $shaFile" }
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
    $isccExe = Resolve-Iscc
    if (-not $isccExe) {
        throw "-BuildInstaller requires iscc.exe (Inno Setup 6: winget install JRSoftware.InnoSetup hoac https://jrsoftware.org/isdl.php); installer was not produced"
    } else {
        $targetDirArg = "/DTargetDir=..\..\$ReleaseDir"
        $installerArgs = @("/DMyAppVersion=$Version", $targetDirArg)
        if ($IncludeCompatibilityHook) { $installerArgs += "/DIncludeCompatibilityHook=1" }
        $installerArgs += "installer\windows\TextVN-setup.iss"
        & $isccExe @installerArgs
        # -BuildInstaller la yeu cau ro rang: ISCC loi thi dung, khong WARN roi
        # bao "Build COMPLETE" (loi [Code] cua .iss tung lot qua nhu vay).
        if ($LASTEXITCODE -ne 0) { throw "ISCC failed with exit code $LASTEXITCODE" }
        $setupExe = "dist\TextVN-setup-$Version-windows-x64.exe"
        if (Test-Path $setupExe) {
            if ($SigningRequested) {
                Sign-TextVNFile $signTool $SigningCertificateThumbprint $setupExe
            } elseif ($env:SIGNPATH_API_TOKEN) {
                powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\sign-signpath.ps1 -File $setupExe -Description "TextVN installer"
                if ($LASTEXITCODE -ne 0) { throw "SignPath signing failed: $setupExe" }
            }
            Write-Ok "Installer: $setupExe"
        } else {
            throw "Installer output not found: $setupExe"
        }

        # Bien the MAY (Store): cung .iss voi /DMachineInstall=1 -> ARP o HKLM,
        # Program Files. Audit vong 5 goc re 1: truoc day chi build per-user nen
        # goi nop Store thieu lua chon may. -MachineInstaller bat buoc build CA
        # HAI; file ra co duoi "-machine.exe" (khong ghi de ban thuong).
        if ($MachineInstaller) {
            $machineArgs = @("/DMyAppVersion=$Version", $targetDirArg, "/DMachineInstall=1", '/DOutputSuffix="-machine"')
            if ($IncludeCompatibilityHook) { $machineArgs += "/DIncludeCompatibilityHook=1" }
            $machineArgs += "installer\windows\TextVN-setup.iss"
            & $isccExe @machineArgs
            if ($LASTEXITCODE -ne 0) { throw "ISCC (machine) failed with exit code $LASTEXITCODE" }
            $machineExe = "dist\TextVN-setup-$Version-windows-x64-machine.exe"
            if (-not (Test-Path $machineExe)) { throw "Machine installer output not found: $machineExe" }
            # R2-17: ban -machine.exe la ban NOP STORE - phai ky giong ban per-user.
            if ($SigningRequested) {
                Sign-TextVNFile $signTool $SigningCertificateThumbprint $machineExe
            } elseif ($env:SIGNPATH_API_TOKEN) {
                powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\sign-signpath.ps1 -File $machineExe -Description "TextVN installer (machine)"
                if ($LASTEXITCODE -ne 0) { throw "SignPath signing failed: $machineExe" }
            }
            Write-Ok "Installer (machine/HKLM): $machineExe"
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
    if ($MachineInstaller) {
        $machinePath = "$DistDir\TextVN-setup-$Version-windows-x64-machine.exe"
        if (Test-Path $machinePath) {
            Write-Host "  Machine      : $machinePath" -ForegroundColor Green
        }
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
