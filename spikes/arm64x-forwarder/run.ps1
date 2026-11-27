# SPDX-License-Identifier: GPL-3.0-or-later
# SPK-ARM64X run (GHA windows-11-arm). Uses the output of build.ps1.
# Copies forwarder + both target DLLs into a per-user-style folder WITH A SPACE (TextVN installs
# per-user, not into System32 like Weasel), then from a DIFFERENT working directory:
#   - probe-x64.exe (emulated) and probe-arm64.exe (native) load tsf_min.dll the way COM loads
#     InprocServer32 (LoadLibraryExW + LOAD_WITH_ALTERED_SEARCH_PATH) and plain LoadLibraryW,
#     call DllGetClassObject -> CreateInstance(ITfTextInputProcessor), and check the forwarded
#     target DLL matches the process architecture;
#   - negative control: the x64 probe must NOT load tsf_min_arm64.dll directly.
# Writes nothing to the registry. Exit code 0 = every expectation met.
param([string]$Dir = '')
$ErrorActionPreference = 'Stop'
if (-not $Dir) { $Dir = Join-Path $PSScriptRoot '..\..\target\spikes\arm64x-out' }
$base = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { $env:TEMP }
$inst = Join-Path $base 'TextVN Spike ARM64X\bin'
$cwd = Join-Path $base 'spk-arm64x-cwd'
foreach ($d in @($inst, $cwd)) {
    if (Test-Path $d) { Remove-Item $d -Recurse -Force }
    New-Item -ItemType Directory -Force $d | Out-Null
}
foreach ($f in @('tsf_min.dll', 'tsf_min_x64.dll', 'tsf_min_arm64.dll')) {
    Copy-Item (Join-Path $Dir $f) (Join-Path $inst $f)
}
# tsf-min logs HRESULTs to %LOCALAPPDATA%\TextVN\logs - keep the spike log out of real profiles.
$env:LOCALAPPDATA = Join-Path $base 'spk-arm64x-localappdata'
New-Item -ItemType Directory -Force $env:LOCALAPPDATA | Out-Null

Write-Output ("host PROCESSOR_ARCHITECTURE=" + $env:PROCESSOR_ARCHITECTURE)
$fwd = Join-Path $inst 'tsf_min.dll'
$fail = 0
Push-Location $cwd
try {
    foreach ($p in @('probe-x64.exe', 'probe-arm64.exe')) {
        $exe = Join-Path $Dir $p
        foreach ($mode in @('', '--plain')) {
            $argv = @($fwd)
            if ($mode) { $argv += $mode }
            & $exe @argv
            if ($LASTEXITCODE -ne 0) { $fail++ }
        }
    }
    # Negative control: x64 process + ARM64 DLL must fail (ERROR_BAD_EXE_FORMAT).
    & (Join-Path $Dir 'probe-x64.exe') (Join-Path $inst 'tsf_min_arm64.dll')
    if ($LASTEXITCODE -eq 0) {
        Write-Output 'SPK-ARM64X negative control FAILED: x64 process loaded an ARM64 DLL'
        $fail++
    } else {
        Write-Output 'SPK-ARM64X negative control: x64 process rejected ARM64 DLL (expected)'
    }
} finally {
    Pop-Location
}
if ($fail -ne 0) { throw "SPK-ARM64X: $fail expectation(s) failed" }
Write-Output 'SPK-ARM64X run: PASS'
