# SPDX-License-Identifier: GPL-3.0-or-later
# WIN-061 - static check targets JSON (schema + >= 2 locator/app + control_type/regex).
# Run: powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\check_targets.ps1
# Exit 0 = pass; exit 1 = co error (G13 - chua xanh thi chua Done).
param(
    [string]$Dir = (Join-Path (Split-Path $PSScriptRoot -Parent) 'appcomptest\targets')
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'lib\targets.lib.ps1')
. (Join-Path $PSScriptRoot 'lib\win32-uia.lib.ps1')
Initialize-TextVNUiA

# 12 app CI (P1-5 sec 3 - cot "CI?")
$ciApps = @('notepad', 'word', 'vscode', 'chrome', 'edge', 'firefox',
            'excel', 'terminal', 'explorer', 'slack', 'discord', 'jetbrains')

$files = Get-TargetFiles $Dir
$errors = New-Object System.Collections.Generic.List[string]
$rows = @()

if ($files.Count -lt 12) { $errors.Add(('file JSON < 12: co ' + $files.Count)) }
foreach ($want in $ciApps) {
    if (-not ($files | Where-Object { $_.Name -eq ($want + '.json') })) { $errors.Add('thieu file CI: ' + $want + '.json') }
}

foreach ($f in $files) {
    $o = $null
    try { $o = Read-TargetFile $f.FullName } catch { $errors.Add($f.Name + ': parse loi - ' + $_.Exception.Message); continue }
    foreach ($er in (Test-TargetSchema $o $f.Name)) { $errors.Add($er) }
    # control_type phai la ControlType that cua UIA (khong chi la chuoi)
    foreach ($fl in @($o.fields)) {
        foreach ($l in @($fl.locators)) {
            if ($l.control_type) {
                try { $null = Get-UiAControlType $l.control_type }
                catch { $errors.Add($f.Name + ': control_type khong ton tai: ' + $l.control_type) }
            }
        }
    }
    $nLoc = 0; foreach ($fl in @($o.fields)) { $nLoc += @($fl.locators).Count }
    $exeVal = '?'
    try { $exeVal = @($o.match.any)[0].exe } catch {}
    $rows += [pscustomobject]@{
        App    = $o.app_id
        Fields = @($o.fields).Count
        Locs   = $nLoc
        Launch = $o.launch.kind
        Exe    = $exeVal
    }
}

$rows | Format-Table -AutoSize | Out-String -Width 160 | Write-Output
if ($errors.Count -gt 0) {
    Write-Output ('FAIL - ' + $errors.Count + ' loi:')
    foreach ($er in $errors) { Write-Output ('  - ' + $er) }
    exit 1
}
Write-Output ('PASS - ' + $files.Count + ' file, moi app >= 2 locator (WIN-061 acceptance 1/2).')
exit 0
