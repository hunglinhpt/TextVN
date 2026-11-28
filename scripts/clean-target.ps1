# SPDX-License-Identifier: GPL-3.0-or-later
<#
.SYNOPSIS
    Safe disk space inspection and cleanup helper for TextVN Cargo target artifacts.
.DESCRIPTION
    Inspects Cargo target directory breakdown, verifies no build or IDE processes
    are actively locking files under target/, and safely reclaims disk space via
    cargo-sweep, targeted cleanup, or cargo clean.
#>
[CmdletBinding()]
param(
    [ValidateSet('All', 'Incremental', 'StaleTriples', 'Sweep', 'DebugBuild')]
    [string]$Mode = 'All',

    [ValidateRange(0, 3650)]
    [int]$SweepDays = 0,

    [switch]$Force,

    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Format-Bytes {
    param([long]$Bytes)
    if ($Bytes -ge 1GB) {
        return ('{0:N2} GiB ({1:N2} GB)' -f ($Bytes / 1GB), ($Bytes / 1000000000.0))
    }
    if ($Bytes -ge 1MB) {
        return ('{0:N2} MiB' -f ($Bytes / 1MB))
    }
    if ($Bytes -ge 1KB) {
        return ('{0:N2} KiB' -f ($Bytes / 1KB))
    }
    return ('{0} B' -f $Bytes)
}

function Get-DirectoryStats {
    param([string]$RootPath)
    if (-not (Test-Path -LiteralPath $RootPath)) {
        return [pscustomobject]@{ Bytes = [long]0; Files = [long]0 }
    }
    [long]$totalBytes = 0
    [long]$totalFiles = 0
    $pending = New-Object 'System.Collections.Generic.Stack[string]'
    $pending.Push($RootPath)
    while ($pending.Count -gt 0) {
        $current = $pending.Pop()
        try {
            $dirInfo = [System.IO.DirectoryInfo]::new($current)
            foreach ($file in $dirInfo.EnumerateFiles()) {
                $totalBytes += $file.Length
                $totalFiles++
            }
            foreach ($sub in $dirInfo.EnumerateDirectories()) {
                if (($sub.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -eq 0) {
                    $pending.Push($sub.FullName)
                }
            }
        } catch {
            Write-Verbose ("Skipped inaccessible path {0}: {1}" -f $current, $_.Exception.Message)
        }
    }
    return [pscustomobject]@{ Bytes = $totalBytes; Files = $totalFiles }
}

function Get-LockingProcesses {
    param([string]$TargetPath)
    $normalized = [System.IO.Path]::GetFullPath($TargetPath).TrimEnd('\', '/')
    $toolNames = @('cargo', 'rustc', 'rust-analyzer', 'rust-analyzer-proc-macro-srv', 'link', 'lld-link')
    $hits = @()
    foreach ($proc in (Get-Process -ErrorAction SilentlyContinue)) {
        $procName = $proc.ProcessName
        $procPath = $null
        try { $procPath = $proc.Path } catch { $procPath = $null }

        $isBuildTool = $toolNames -contains $procName.ToLowerInvariant()
        $insideTarget = $false
        if ($procPath) {
            $fullProcPath = [System.IO.Path]::GetFullPath($procPath)
            if ($fullProcPath.StartsWith($normalized + '\', [System.StringComparison]::OrdinalIgnoreCase) -or
                $fullProcPath.StartsWith($normalized + '/', [System.StringComparison]::OrdinalIgnoreCase)) {
                $insideTarget = $true
            }
        }
        if ($isBuildTool -or $insideTarget) {
            $hits += [pscustomobject]@{
                Id   = $proc.Id
                Name = $procName
                Path = $(if ($procPath) { $procPath } else { '<unavailable>' })
            }
        }
    }
    return $hits
}

$RepoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$TargetDir = if ($env:CARGO_TARGET_DIR -and $env:CARGO_TARGET_DIR.Trim()) {
    [System.IO.Path]::GetFullPath($env:CARGO_TARGET_DIR.Trim())
} else {
    Join-Path $RepoRoot 'target'
}

if ($SweepDays -gt 0 -and $Mode -eq 'All') {
    $Mode = 'Sweep'
}
if ($Mode -eq 'Sweep' -and $SweepDays -eq 0) {
    $SweepDays = 14
}

$isDryRun = $DryRun.IsPresent

Write-Host ('Repo root : {0}' -f $RepoRoot)
Write-Host ('Target dir: {0}' -f $TargetDir)
Write-Host ('Mode      : {0}{1}' -f $Mode, $(if ($isDryRun) { ' [DRY-RUN]' } else { '' }))

if (-not (Test-Path -LiteralPath $TargetDir)) {
    Write-Host 'Target directory does not exist; nothing to clean.'
    exit 0
}

$lockers = @(Get-LockingProcesses -TargetPath $TargetDir)
if ($lockers.Count -gt 0) {
    Write-Warning 'Detected active build/IDE processes that may lock target artifacts:'
    foreach ($l in $lockers) {
        Write-Warning ('  PID {0}: {1} ({2})' -f $l.Id, $l.Name, $l.Path)
    }
    if (-not $Force -and -not $isDryRun) {
        Write-Error 'Aborting cleanup. Close locking tools (cargo/rustc/rust-analyzer) or re-run with -Force.'
        exit 1
    }
}

$beforeStats = Get-DirectoryStats -RootPath $TargetDir
Write-Host ''
Write-Host ('Before cleanup: {0} across {1:N0} files' -f (Format-Bytes $beforeStats.Bytes), $beforeStats.Files)

$topDirs = @(Get-ChildItem -LiteralPath $TargetDir -Directory -Force -ErrorAction SilentlyContinue | Sort-Object Name)
foreach ($d in $topDirs) {
    $s = Get-DirectoryStats -RootPath $d.FullName
    Write-Host ('  - {0,-32} {1,22} ({2:N0} files)' -f $d.Name, (Format-Bytes $s.Bytes), $s.Files)
}

$driveRoot = [System.IO.Path]::GetPathRoot($TargetDir).TrimEnd('\', ':')
$driveBefore = (Get-PSDrive -Name $driveRoot -ErrorAction SilentlyContinue).Free

Push-Location $RepoRoot
try {
    switch ($Mode) {
        'Sweep' {
            $hasSweep = $null -ne (Get-Command cargo-sweep -ErrorAction SilentlyContinue)
            if (-not $hasSweep) {
                Write-Error "cargo-sweep is not installed. Install via 'cargo install cargo-sweep' or use '-Mode All' / '-Mode Incremental'."
                exit 1
            }
            Write-Host ''
            if ($isDryRun) {
                Write-Host ("[DRY-RUN] Would run: cargo sweep -t {0}" -f $SweepDays)
            } else {
                & cargo sweep -t $SweepDays
                if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            }
        }
        'Incremental' {
            $incDirs = @()
            foreach ($cand in @('debug\incremental', 'release\incremental')) {
                $p = Join-Path $TargetDir $cand
                if (Test-Path -LiteralPath $p) { $incDirs += $p }
            }
            foreach ($d in $topDirs) {
                if ($d.Name -like '*-*-*') {
                    foreach ($sub in @('debug\incremental', 'release\incremental')) {
                        $p = Join-Path $d.FullName $sub
                        if (Test-Path -LiteralPath $p) { $incDirs += $p }
                    }
                }
            }
            Write-Host ''
            if ($incDirs.Count -eq 0) {
                Write-Host 'No incremental cache directories found.'
            } else {
                foreach ($inc in $incDirs) {
                    if ($isDryRun) {
                        Write-Host ('[DRY-RUN] Would remove: {0}' -f $inc)
                    } else {
                        Remove-Item -LiteralPath $inc -Recurse -Force
                        Write-Host ('Removed: {0}' -f $inc)
                    }
                }
            }
        }
        'StaleTriples' {
            $tripleDirs = @($topDirs | Where-Object { $_.Name -like '*-*-*' })
            Write-Host ''
            if ($tripleDirs.Count -eq 0) {
                Write-Host 'No target-triple subdirectories found.'
            } else {
                foreach ($td in $tripleDirs) {
                    if ($isDryRun) {
                        Write-Host ('[DRY-RUN] Would remove: {0}' -f $td.FullName)
                    } else {
                        Remove-Item -LiteralPath $td.FullName -Recurse -Force
                        Write-Host ('Removed: {0}' -f $td.FullName)
                    }
                }
            }
        }
        'DebugBuild' {
            $debugBuildDirs = @()
            $rootDebugBuild = Join-Path $TargetDir 'debug\build'
            if (Test-Path -LiteralPath $rootDebugBuild) {
                $debugBuildDirs += $rootDebugBuild
            }
            foreach ($d in $topDirs) {
                if ($d.Name -like '*-*-*') {
                    $tripleDebugBuild = Join-Path $d.FullName 'debug\build'
                    if (Test-Path -LiteralPath $tripleDebugBuild) {
                        $debugBuildDirs += $tripleDebugBuild
                    }
                }
            }
            Write-Host ''
            if ($debugBuildDirs.Count -eq 0) {
                Write-Host 'No debug\build directories found.'
            } else {
                foreach ($dbDir in $debugBuildDirs) {
                    if ($isDryRun) {
                        Write-Host ('[DRY-RUN] Would remove: {0}' -f $dbDir)
                    } else {
                        Remove-Item -LiteralPath $dbDir -Recurse -Force
                        Write-Host ('Removed: {0}' -f $dbDir)
                    }
                }
            }
        }
        'All' {
            Write-Host ''
            if ($isDryRun) {
                Write-Host ('[DRY-RUN] Would run: cargo clean (target: {0})' -f $TargetDir)
            } else {
                & cargo clean
                if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            }
        }
    }
} finally {
    Pop-Location
}

$afterStats = Get-DirectoryStats -RootPath $TargetDir
$reclaimed = [Math]::Max([long]0, $beforeStats.Bytes - $afterStats.Bytes)
$driveAfter = (Get-PSDrive -Name $driveRoot -ErrorAction SilentlyContinue).Free

Write-Host ''
Write-Host ('After cleanup : {0} across {1:N0} files' -f (Format-Bytes $afterStats.Bytes), $afterStats.Files)
Write-Host ('Reclaimed     : {0}' -f (Format-Bytes $reclaimed))
if ($null -ne $driveBefore -and $null -ne $driveAfter) {
    Write-Host ('Drive {0}:\ free: {1} -> {2}' -f $driveRoot, (Format-Bytes ([long]$driveBefore)), (Format-Bytes ([long]$driveAfter)))
}
