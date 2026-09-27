# SPDX-License-Identifier: GPL-3.0-or-later
# WIN-061 - live verify targets JSON: mo app (attach truoc, spawn neu can),
# tim element theo tung locator, ghi evidence. Chi doc UIA (S2: khong log Name/text).
# Run:  powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\verify_targets.ps1
#       ... -Only chrome,notepad   ... -NoSpawn   ... -OutMd <path>
param(
    [string]$Dir = (Join-Path (Split-Path $PSScriptRoot -Parent) 'appcomptest\targets'),
    [string[]]$Only = @(),
    [switch]$NoSpawn,
    [string]$OutMd = ''
)
$ErrorActionPreference = 'Stop'
# -File truyen -Only thanh 1 string 'a,b' -> tach ra mang (A14)
$Only = @($Only | ForEach-Object { $_ -split ',' } | Where-Object { $_ -ne '' })
. (Join-Path $PSScriptRoot 'lib\targets.lib.ps1')
. (Join-Path $PSScriptRoot 'lib\win32-uia.lib.ps1')
Initialize-TextVNUiA

$AE = [System.Windows.Automation.AutomationElement]
$rows = New-Object System.Collections.Generic.List[object]
$spawnedHwnds = New-Object System.Collections.Generic.List[object]   # @{hwnd;pid;close}
$comApps = New-Object System.Collections.Generic.List[object]        # @{obj;kind}

function Get-ProcNameOfHwnd([IntPtr]$h) {
    $pid2 = 0
    [void][VtWin]::GetWindowThreadProcessId($h, [ref]$pid2)
    if ($pid2 -eq 0) { return '' }
    $p = Get-Process -Id $pid2 -ErrorAction SilentlyContinue
    if ($p) { return $p.ProcessName } return ''
}
function Find-AppWindow($o) {
    $wantProc = $null
    try { if ($o.launch.process) { $wantProc = [System.IO.Path]::GetFileNameWithoutExtension($o.launch.process) } } catch {}
    if (-not $wantProc) { $wantProc = [System.IO.Path]::GetFileNameWithoutExtension(@($o.match.any)[0].exe) }
    # 2 pha (f6-9): truoc chon cua so khong owner (loai popup/dialog vd "Translate this page?"
    # Chrome_WidgetWin_1 cung class), neu khong co thi moi chap nhan cua so co owner.
    $owned = [IntPtr]::Zero
    foreach ($cand in [VtWin]::WinsByClass($o.launch.ready_class)) {
        $p = $cand.Split('|')
        $h = [IntPtr][long]$p[0]
        if ((Get-ProcNameOfHwnd $h) -ieq $wantProc) {
            if ([VtWin]::GetOwner($h) -eq [IntPtr]::Zero) { return $h }
            if ($owned -eq [IntPtr]::Zero) { $owned = $h }
        }
    }
    return $owned
}
function Expand-Path([string]$p) { [Environment]::ExpandEnvironmentVariables($p) }
$script:FixtureUri = ''
$script:VtTempProfiles = New-Object System.Collections.Generic.List[string]   # profile tam de cleanup (f6-13)
function Get-FixtureUri {
    # trang test nho (contenteditable) - chromium khong expose Document voi about:blank
    if ($script:FixtureUri) { return $script:FixtureUri }
    $p = Join-Path $env:TEMP 'textvn_verify_page.html'
    $html = '<!doctype html><meta charset="utf-8"><title>verify</title><div contenteditable="true">x</div>'
    [System.IO.File]::WriteAllText($p, $html, [System.Text.UTF8Encoding]::new($false))
    $script:FixtureUri = ([System.Uri]$p).AbsoluteUri
    return $script:FixtureUri
}
function New-ProfileDir($o) {
    # profile tam + user.js (f6-13): Firefox profile MOI bi modal TOU_ONBOARDING che toan bo
    # toolbar/urlbar (36 Menu + modal, khong co ComboBox/Edit) -> dat pref truoc khi chay.
    $pd = Join-Path $env:TEMP ('vt_prof_' + $o.app_id + '_' + [guid]::NewGuid().ToString('N').Substring(0, 6))
    New-Item -ItemType Directory -Force -Path $pd | Out-Null
    $prefs = $null
    try { $prefs = $o.launch.profile.prefs } catch {}
    if ($prefs) {
        $jl = New-Object System.Collections.Generic.List[string]
        foreach ($pn in $prefs.PSObject.Properties.Name) {
            $pv = $prefs.PSObject.Properties[$pn].Value
            $jl.Add('user_pref("' + $pn + '", ' + (ConvertTo-Json $pv) + ');')
        }
        [System.IO.File]::WriteAllLines((Join-Path $pd 'user.js'), $jl, [System.Text.UTF8Encoding]::new($false))
    }
    $script:VtTempProfiles.Add($pd)
    return $pd
}
function Get-ResolvedArgs($o) {
    $r = @()
    foreach ($a in @($o.launch.args)) {
        if ($a -eq '{fixture}') { $r += (Get-FixtureUri) } else { $r += $a }
    }
    # launch.profile: tao profile tam + append args (co the chua {profile_dir}) - kieu Firefox
    $prof = $null
    try { $prof = $o.launch.profile } catch {}
    if ($prof) {
        $pd = New-ProfileDir $o
        foreach ($a in @($prof.args)) {
            if ($a -eq '{profile_dir}') { $r += $pd } else { $r += $a }
        }
    }
    return $r
}
function Resolve-CommandPath([string]$name) {
    $c = Get-Command $name -ErrorAction SilentlyContinue
    if ($c -and $c.Source) { return $c.Source }
    return $null
}
function Get-AppVersion($o) {
    foreach ($p in @($o.launch.paths)) {
        $ep = Expand-Path $p
        if ($ep -and (Test-Path -LiteralPath $ep)) {
            try { return (Get-Item -LiteralPath $ep).VersionInfo.ProductVersion } catch {}
        }
    }
    # appx package (f6-5: vd Windows Terminal - wt.exe la app-execution-alias, khong co version)
    try {
        if ($o.launch.appx_package) {
            $apx = Get-AppxPackage -Name $o.launch.appx_package -ErrorAction SilentlyContinue
            if ($apx) { return $apx.Version }
        }
    } catch {}
    if ($o.launch.kind -eq 'shell') {
        $c = Get-Command $o.launch.command -ErrorAction SilentlyContinue
        if ($c -and $c.Path) { try { return (Get-Item -LiteralPath $c.Path).VersionInfo.ProductVersion } catch {} }
    }
    # PATH fallback: .cmd shim khong co version -> do exe o thu muc cha (bin\..\X.exe)
    $exe = @($o.match.any)[0].exe
    $stem = [System.IO.Path]::GetFileNameWithoutExtension($exe)
    $src = Resolve-CommandPath $stem
    if (-not $src) { $src = Resolve-CommandPath $exe }
    if ($src) {
        $v = ''
        try { $v = (Get-Item -LiteralPath $src).VersionInfo.ProductVersion } catch {}
        if ($v) { return $v }
        if ($src -match '\.(cmd|bat)$') {
            $root = Split-Path (Split-Path $src -Parent) -Parent
            $alt = Join-Path $root ($stem + '.exe')
            if (Test-Path -LiteralPath $alt) { try { return (Get-Item -LiteralPath $alt).VersionInfo.ProductVersion } catch {} }
        }
    }
    return ''
}
function Test-Installed($o) {
    foreach ($p in @($o.launch.paths)) {
        $ep = Expand-Path $p
        if ($ep -and (Test-Path -LiteralPath $ep)) { return $true }
    }
    if ($o.launch.kind -eq 'shell') {
        if (Get-Command $o.launch.command -ErrorAction SilentlyContinue) { return $true }
    }
    # PATH fallback (vd VS Code cai o vi tri tuy chinh nhu D:\AIInstall)
    $exe = @($o.match.any)[0].exe
    $stem = [System.IO.Path]::GetFileNameWithoutExtension($exe)
    if ((Resolve-CommandPath $stem) -or (Resolve-CommandPath $exe)) { return $true }
    return $false
}
function Spawn-App($o, [IntPtr]$current) {
    if ($NoSpawn) { return $current }
    if ($current -ne [IntPtr]::Zero) { return $current }
    $kind = $o.launch.kind
    if ($kind -eq 'com') {
        $progId = $o.launch.com_prog_id
        $app = New-Object -ComObject $progId
        $app.Visible = $true
        if ($o.launch.com_add -eq 'doc') { [void]$app.Documents.Add() }
        elseif ($o.launch.com_add -eq 'wb') { [void]$app.Workbooks.Add() }
        $comApps.Add(@{ obj = $app; kind = $progId })
        for ($i = 0; $i -lt 30; $i++) {
            $h = Find-AppWindow $o
            if ($h -ne [IntPtr]::Zero) { return $h }
            Start-Sleep -Milliseconds 500
        }
        return [IntPtr]::Zero
    }
    if ($kind -eq 'shell') {
        $argList = Get-ResolvedArgs $o
        if ($argList.Count -gt 0) { Start-Process $o.launch.command -ArgumentList $argList }
        else { Start-Process $o.launch.command }
    } else {
        $exePath = $null
        foreach ($p in @($o.launch.paths)) {
            $ep = Expand-Path $p
            if ($ep -and (Test-Path -LiteralPath $ep)) { $exePath = $ep; break }
        }
        if (-not $exePath) {
            # PATH fallback
            $exeNm = @($o.match.any)[0].exe
            $stemNm = [System.IO.Path]::GetFileNameWithoutExtension($exeNm)
            $exePath = Resolve-CommandPath $stemNm
            if (-not $exePath) { $exePath = Resolve-CommandPath $exeNm }
        }
        if (-not $exePath) { return [IntPtr]::Zero }
        $argList = Get-ResolvedArgs $o
        if ($argList.Count -gt 0) { Start-Process $exePath -ArgumentList $argList }
        else { Start-Process $exePath }
    }
    for ($i = 0; $i -lt 40; $i++) {
        $h = Find-AppWindow $o
        if ($h -ne [IntPtr]::Zero) { return $h }
        Start-Sleep -Milliseconds 500
    }
    return [IntPtr]::Zero
}

# === main ===
$files = Get-TargetFiles $Dir
foreach ($f in $files) {
    $o = Read-TargetFile $f.FullName
    if ($Only.Count -gt 0 -and $Only -notcontains $o.app_id) { continue }
    $installed = Test-Installed $o
    $version = ''
    if ($installed) { $version = Get-AppVersion $o }
    if (-not $installed) {
        $rows.Add([pscustomobject]@{ App = $o.app_id; Status = 'NOT_INSTALLED'; Version = ''; Field = '-'; Hit = '-'; Resolves = '-'; Ms = '' })
        continue
    }
    $h0 = Find-AppWindow $o
    $h = Spawn-App $o $h0
    if ($h -eq [IntPtr]::Zero) {
        $rows.Add([pscustomobject]@{ App = $o.app_id; Status = 'NO_WINDOW'; Version = $version; Field = '-'; Hit = '-'; Resolves = '-'; Ms = '' })
        continue
    }
    # neu truoc do khong co cua so (script vua spawn) thi danh dau cleanup; explorer = never
    $closeMode = 'window'
    try { if ($o.launch.close) { $closeMode = $o.launch.close } } catch {}
    if ($h0 -eq [IntPtr]::Zero -and $closeMode -ne 'never') {
        $np = [uint32]0
        [void][VtWin]::GetWindowThreadProcessId($h, [ref]$np)
        $spawnedHwnds.Add(@{ hwnd = $h; pid = $np })
    }
    $root = $AE::FromHandle($h)
    # focus truoc khi probe neu launch.yeu cau (WT: TermControl chi xuat hien sau focus - diag3)
    $focusFirst = $false
    try { $focusFirst = [bool]$o.launch.focus_before_probe } catch {}
    if ($focusFirst) { [void](Focus-UiAWindow $h); Start-Sleep -Milliseconds 800 }
    # (f6-10) readiness theo TUNG field: readiness ANY-truoc do cho phep evaluate address_bar
    # khi moi Document (renderer) render truoc -> field chua xuat hien bi MISS som.
    # Moi field poll CHIN locators cua no toi da 10s truoc khi danh gia (S4).
    foreach ($fl in @($o.fields)) {
        $fieldReady = $false
        for ($i = 0; $i -lt 20 -and -not $fieldReady; $i++) {
            foreach ($l0 in @($fl.locators)) {
                if (Find-UiAElementByLocator $root $l0) { $fieldReady = $true; break }
            }
            if (-not $fieldReady) { Start-Sleep -Milliseconds 500 }
        }
        $hitIdx = -1; $hitInfo = ''; $hitMs = 0.0; $ok = 0; $total = 0
        $li = 0
        foreach ($l in @($fl.locators)) {
            $total++
            $sw = [System.Diagnostics.Stopwatch]::StartNew()
            $el = Find-UiAElementByLocator $root $l
            $sw.Stop()
            if ($el) {
                $ok++
                if ($hitIdx -lt 0) {
                    $hitIdx = $li
                    $ct = $el.Current.ControlType.ProgrammaticName -replace 'ControlType_', ''
                    $hitInfo = $ct + '/' + $el.Current.ClassName
                    $hitMs = [math]::Round($sw.Elapsed.TotalMilliseconds, 1)
                }
            }
            $li++
        }
        $fstatus = 'OK'
        if ($hitIdx -lt 0) { $fstatus = 'FIELD_MISS' }
        $rows.Add([pscustomobject]@{
            App = $o.app_id; Status = $fstatus; Version = $version
            Field = $fl.field_role
            Hit = $(if ($hitIdx -ge 0) { '#' + $hitIdx + ' ' + $hitInfo } else { 'MISS' })
            Resolves = ($ok.ToString() + '/' + $total)
            Ms = $hitMs
        })
        # dump cay ngan khi MISS de chan doan tren CI (S2: chi ControlType/ClassName/aId, khong Name)
        if ($fstatus -eq 'FIELD_MISS') {
            try {
                $allD = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants,
                    [System.Windows.Automation.Condition]::TrueCondition)
                Write-Output ('  [MISS-DUMP] ' + $o.app_id + '/' + $fl.field_role + ' total=' + $allD.Count)
                $dn = 0
                foreach ($eD in $allD) {
                    if ($dn -ge 25) { break }
                    $ctD = $eD.Current.ControlType.ProgrammaticName -replace 'ControlType_', ''
                    Write-Output ('    ct=' + $ctD + " cls='" + $eD.Current.ClassName + "' aId='" + $eD.Current.AutomationId + "'")
                    $dn++
                }
            } catch { Write-Output ('  [MISS-DUMP] loi doc cay: ' + $_.Exception.Message) }
        }
    }
}

# === cleanup: chi process ma script vua spawn; explorer = never; COM = Quit ===
foreach ($c in $comApps) {
    try {
        if ($c.kind -eq 'Word.Application') { $c.obj.Quit(0) }
        else { $c.obj.Quit() }
    } catch {}
}
foreach ($s in $spawnedHwnds) {
    $alive = $true
    try {
        $proc = Get-Process -Id $s.pid -ErrorAction Stop
        [void]$proc.CloseMainWindow()
        Start-Sleep -Seconds 2
        $proc.Refresh()
        if ($proc.HasExited) { $alive = $false }
    } catch { $alive = $false }
    if ($alive) {
        try { & taskkill /PID $s.pid /T /F 2>&1 | Out-Null } catch {}
    }
}
# profile tam (f6-13) - xoa sau khi app da dong (khong dot profile that cua user)
foreach ($d in $script:VtTempProfiles) {
    try { Remove-Item -LiteralPath $d -Recurse -Force -ErrorAction SilentlyContinue } catch {}
}

# === report ===
$rows | Format-Table -AutoSize | Out-String -Width 200 | Write-Output
$fail = @($rows | Where-Object { $_.Status -ne 'OK' -and $_.Status -ne 'NOT_INSTALLED' })
$notInst = @($rows | Where-Object { $_.Status -eq 'NOT_INSTALLED' })
Write-Output ('apps: ' + $rows.Count + ' | OK: ' + @($rows | Where-Object { $_.Status -eq 'OK' }).Count +
    ' | FIELD_MISS/NO_WINDOW: ' + $fail.Count + ' | NOT_INSTALLED: ' + $notInst.Count)

if (-not $OutMd) { $OutMd = Join-Path $Dir 'verify-evidence.md' }
$md = New-Object System.Collections.Generic.List[string]
$md.Add('# Verify evidence - WIN-061 targets (auto-generated)')
$md.Add('')
$md.Add('- Run: ' + (Get-Date -Format 'yyyy-MM-dd HH:mm:ss') + ' | machine Win ' + [System.Environment]::OSVersion.Version.ToString())
$md.Add('- Semantics: locator chain thu tu; `Resolves` = so locator cung resolve duoc luc nay (fallback doc lap).')
$md.Add('- S2: chi ghi ControlType/ClassName/chi so - KHONG ghi Name element.')
$md.Add('')
$md.Add('| app | status | version | field | hit | resolves | ms |')
$md.Add('|---|---|---|---|---|---|---|')
foreach ($r in $rows) {
    $md.Add('| ' + $r.App + ' | ' + $r.Status + ' | ' + $r.Version + ' | ' + $r.Field + ' | ' + $r.Hit + ' | ' + $r.Resolves + ' | ' + $r.Ms + ' |')
}
$md.Add('')
[IO.File]::WriteAllText($OutMd, (($md -join "`r`n") + "`r`n"), [Text.UTF8Encoding]::new($false))
Write-Output ('evidence -> ' + $OutMd)
if ($fail.Count -gt 0) { exit 1 }
exit 0
