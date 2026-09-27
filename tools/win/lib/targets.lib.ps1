# SPDX-License-Identifier: GPL-3.0-or-later
# Shared targets-JSON loader + schema check (G15): dung chung cho
# tools/win/check_targets.ps1 va tools/win/verify_targets.ps1.
# ASCII-only (G7).

# field_role whitelist - truc tiep tu P0-3 sec 2.1 (mapping field_role <-> IME_FIELD_*)
$script:VtFieldRoles = @(
    'unknown', 'body', 'editbox', 'address_bar', 'search', 'combo',
    'candidate', 'textarea', 'web', 'terminal', 'secure'
)
# locator keys whitelist - AND semantics; regex dung client-side (s4: PropertyCondition khong co Contains)
$script:VtLocatorKeys = @(
    'control_type', 'automation_id', 'name', 'name_regex', 'class_name', 'class_regex'
)
$script:VtLaunchKinds = @('exe', 'com', 'shell')

function Get-TargetFiles([string]$dir) {
    if (-not (Test-Path -LiteralPath $dir)) { throw ("Targets dir not found: " + $dir) }
    Get-ChildItem -LiteralPath $dir -Filter '*.json' | Sort-Object Name
}

function Read-TargetFile([string]$path) {
    $json = [System.IO.File]::ReadAllText($path, [System.Text.Encoding]::UTF8)
    return $json | ConvertFrom-Json
}

# Returns [string[]] errors (empty = OK).
function Test-TargetSchema($o, [string]$file) {
    $e = New-Object System.Collections.Generic.List[string]
    if ($null -eq $o) { $e.Add($file + ': JSON parse loi'); return ,$e }
    if ($o.targets_version -ne 1) { $e.Add($file + ': targets_version phai = 1') }
    $stem = [System.IO.Path]::GetFileNameWithoutExtension($file)
    if (-not $o.app_id) { $e.Add($file + ': thieu app_id') }
    elseif ($o.app_id -ne $stem) { $e.Add($file + ': app_id khac ten file (' + $o.app_id + ' vs ' + $stem + ')') }
    # match.any[0].exe
    $exe = $null
    try { $exe = $o.match.any[0].exe } catch {}
    if (-not $exe) { $e.Add($file + ': thieu match.any[0].exe') }
    # launch
    if (-not $o.launch) { $e.Add($file + ': thieu launch') }
    else {
        if ($script:VtLaunchKinds -notcontains $o.launch.kind) {
            $e.Add($file + ': launch.kind phai la ' + ($script:VtLaunchKinds -join '|'))
        }
        if ($o.launch.kind -eq 'shell' -and -not $o.launch.command) { $e.Add($file + ': launch.kind=shell thieu command') }
        if ($o.launch.kind -ne 'shell') {
            $hasPath = $false
            try { $hasPath = (@($o.launch.paths).Count -gt 0) } catch {}
            if (-not $hasPath) { $e.Add($file + ': launch.kind=' + $o.launch.kind + ' thieu paths') }
        }
        if (-not $o.launch.ready_class) { $e.Add($file + ': thieu launch.ready_class') }
        # launch.profile tuy chon (f6-13): phai co args[] (vd Firefox: -no-remote -profile {profile_dir})
        if ($o.launch.profile) {
            $pa = @()
            try { $pa = @($o.launch.profile.args) } catch {}
            if ($pa.Count -eq 0) { $e.Add($file + ': launch.profile co nhung thieu args[]') }
        }
    }
    # fields
    $fields = @()
    try { $fields = @($o.fields) } catch {}
    if ($fields.Count -eq 0) { $e.Add($file + ': thieu fields[]') }
    $totalLoc = 0
    $fieldIdx = 0
    foreach ($f in $fields) {
        $fid = 'fields[' + $fieldIdx + ']'
        $fieldIdx++
        if (-not $f.field_role) { $e.Add($file + ': ' + $fid + ' thieu field_role') }
        elseif ($script:VtFieldRoles -notcontains $f.field_role) {
            $e.Add($file + ': ' + $fid + ' field_role sai whitelist: ' + $f.field_role)
        }
        $locs = @()
        try { $locs = @($f.locators) } catch {}
        if ($locs.Count -lt 2) { $e.Add($file + ': ' + $fid + ' (role=' + $f.field_role + ') can >= 2 locator, co ' + $locs.Count) }
        $li = 0
        foreach ($l in $locs) {
            $keys = @()
            foreach ($p in $l.PSObject.Properties.Name) { $keys += $p }
            $bad = @($keys | Where-Object { $script:VtLocatorKeys -notcontains $_ })
            if ($bad.Count -gt 0) { $e.Add($file + ': locator[' + $li + '] key la sai whitelist: ' + ($bad -join ',')) }
            if ($keys.Count -eq 0) { $e.Add($file + ': locator[' + $li + '] rong') }
            foreach ($rxKey in @('name_regex', 'class_regex')) {
                $v = $null
                try { $v = $l.$rxKey } catch {}
                if ($v) {
                    try { $null = [regex]::new([string]$v) } catch { $e.Add($file + ': locator[' + $li + '] ' + $rxKey + ' khong hop le: ' + $v) }
                }
            }
            $li++
        }
        $totalLoc += $locs.Count
    }
    # acceptance WIN-061: moi app >= 2 locator (tong)
    if ($totalLoc -lt 2 -and $fields.Count -gt 0) { $e.Add($file + ': tong locator < 2') }
    return ,$e
}
