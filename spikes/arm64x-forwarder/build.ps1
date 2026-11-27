# SPDX-License-Identifier: GPL-3.0-or-later
# SPK-ARM64X build (spike for ARM-W2/ARM-W3, P0-7 lane W). Runs on an x64 host (GHA windows-2022):
#   1. spikes/tsf-min cdylib for x86_64 + aarch64  -> out\tsf_min_x64.dll, out\tsf_min_arm64.dll
#   2. probe exe for x86_64 + aarch64              -> out\probe-x64.exe, out\probe-arm64.exe
#   3. ARM64X PURE FORWARDER out\tsf_min.dll: native (ARM64) exports forward to tsf_min_arm64.dll,
#      EC (x64) exports forward to tsf_min_x64.dll (Microsoft "Arm64X pure forwarder" recipe,
#      Weasel arm64x_wrapper precedent) - one InprocServer32 path for both process architectures.
# Needs: rustup targets x86_64-pc-windows-msvc + aarch64-pc-windows-msvc; VS C++ ARM64 + ARM64EC tools.
# Never registers anything; output only under target\spikes\arm64x-out (gitignored).
param([string]$Out = '')
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
if (-not $Out) { $Out = Join-Path $root 'target\spikes\arm64x-out' }
$env:CARGO_TARGET_DIR = Join-Path $root 'target\spikes'
if (Test-Path $Out) { Remove-Item $Out -Recurse -Force }
New-Item -ItemType Directory -Force $Out | Out-Null

function Invoke-Checked([string]$Exe, [string[]]$ArgList) {
    & $Exe @ArgList
    if ($LASTEXITCODE -ne 0) { throw "$Exe $($ArgList -join ' ') failed (exit $LASTEXITCODE)" }
}

$targets = [ordered]@{ 'x64' = 'x86_64-pc-windows-msvc'; 'arm64' = 'aarch64-pc-windows-msvc' }
foreach ($k in $targets.Keys) {
    $t = $targets[$k]
    Invoke-Checked 'cargo' @('build', '--release', '--lib', '--target', $t, '--manifest-path', (Join-Path $root 'spikes\tsf-min\Cargo.toml'))
    Invoke-Checked 'cargo' @('build', '--release', '--target', $t, '--manifest-path', (Join-Path $PSScriptRoot 'probe\Cargo.toml'))
    Copy-Item (Join-Path $env:CARGO_TARGET_DIR "$t\release\tsf_min.dll") (Join-Path $Out "tsf_min_$k.dll")
    Copy-Item (Join-Path $env:CARGO_TARGET_DIR "$t\release\arm64x-probe.exe") (Join-Path $Out "probe-$k.exe")
}

# --- ARM64X pure forwarder --------------------------------------------------------------
# (SPK-RSI da hap thu vao tools/bench 2026-11-28 - khong build rsi-oracle-arm64 nua.)
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.ARM64 -property installationPath
if (-not $vs) { throw 'SPK-ARM64X: Visual Studio C++ ARM64 build tools not found (vswhere)' }
$vcvars = Join-Path $vs 'VC\Auxiliary\Build\vcvarsall.bat'
$vcArch = if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') { 'arm64' } else { 'x64_arm64' }

$fwd = Join-Path $Out 'fwd'
New-Item -ItemType Directory -Force $fwd | Out-Null
$exports = @('DllGetClassObject', 'DllCanUnloadNow')
$x64Def = @('EXPORTS') + ($exports | ForEach-Object { "    $_=tsf_min_x64.$_" })
$a64Def = @('EXPORTS') + ($exports | ForEach-Object { "    $_=tsf_min_arm64.$_" })
Set-Content -Path (Join-Path $fwd 'x64.def') -Value $x64Def -Encoding ascii
Set-Content -Path (Join-Path $fwd 'arm64.def') -Value $a64Def -Encoding ascii
Set-Content -Path (Join-Path $fwd 'empty.cpp') -Value '// SPK-ARM64X: intentionally empty' -Encoding ascii
$cmd = @(
    '@echo off',
    "call `"$vcvars`" $vcArch >nul || exit /b 1",
    "cd /d `"$fwd`" || exit /b 1",
    'cl /nologo /c /Foempty_arm64.obj empty.cpp || exit /b 1',
    'cl /nologo /c /arm64EC /Foempty_x64.obj empty.cpp || exit /b 1',
    'link /lib /nologo /machine:x64 /def:x64.def /out:fwd_x64.lib || exit /b 1',
    'link /lib /nologo /machine:arm64 /def:arm64.def /out:fwd_arm64.lib || exit /b 1',
    'link /nologo /dll /noentry /machine:arm64x /defArm64Native:arm64.def /def:x64.def empty_arm64.obj empty_x64.obj /out:tsf_min.dll fwd_arm64.lib fwd_x64.lib || exit /b 1',
    'dumpbin /nologo /headers tsf_min.dll > dumpbin-headers.txt || exit /b 1',
    'dumpbin /nologo /exports tsf_min.dll > dumpbin-exports.txt || exit /b 1'
)
Set-Content -Path (Join-Path $fwd 'fwd.cmd') -Value $cmd -Encoding ascii
Invoke-Checked 'cmd.exe' @('/c', (Join-Path $fwd 'fwd.cmd'))
Copy-Item (Join-Path $fwd 'tsf_min.dll') (Join-Path $Out 'tsf_min.dll')
Copy-Item (Join-Path $fwd 'dumpbin-*.txt') $Out

$hdr = Get-Content (Join-Path $Out 'dumpbin-headers.txt') -Raw
if ($hdr -notmatch 'ARM64X') { throw 'SPK-ARM64X: forwarder is not an ARM64X image (dumpbin /headers)' }
Write-Output 'SPK-ARM64X build: OK'
Get-ChildItem $Out -File | ForEach-Object { Write-Output ("  {0,-26} {1,10}" -f $_.Name, $_.Length) }
