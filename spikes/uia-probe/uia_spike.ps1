# SPDX-License-Identifier: GPL-3.0-or-later
# WIN-004 - UIA spike: do latency + doc role/IsPassword tren 10 phan tu.
# Output: %TEMP%\uia_spike_out.txt (ASCII, khong ghi text/nguoi dung - S2)
$ErrorActionPreference = 'Stop'
$out = Join-Path $env:TEMP 'uia_spike_out.txt'
if (Test-Path $out) { Remove-Item $out -Force }
function Log([string]$m) { Write-Output $m; Add-Content -Path $out -Value $m }

# G15: dot-source lib dung chung (truoc do script tu define class UW + Add-Type - trung lap)
. (Join-Path $PSScriptRoot '..\..\tools\win\lib\win32-uia.lib.ps1')
Initialize-VietimeUiA
if (-not ('System.Windows.Automation.TreeScope' -as [type])) { throw 'UIA types khong load duoc' }

$AE = [System.Windows.Automation.AutomationElement]
$TS = [System.Windows.Automation.TreeScope]::Descendants

# NOTE (common-error A10): phai goi [Type]::StaticProp bang ngoac () khi truyen vao function,
# vi PowerShell parse [Type]::Member trong argument mode khong nhu expression mode.
# G15: ham local nay da go bo - dung New-UiACond cua lib (win32-uia.lib).

function Wait-Win([uint32]$pidWant, [string]$titleLike, [int]$sec = 30) {
    for ($i = 0; $i -lt $sec; $i++) {
        $wins = [VtWin]::WinsOf($pidWant)
        foreach ($w in $wins) {
            $p = $w.Split('|')
            if ($titleLike -eq '' -or $p[2] -like $titleLike) { return [IntPtr][long]$p[0] }
        }
        Start-Sleep -Seconds 1
    }
    return [IntPtr]::Zero
}
function Measure-It([scriptblock]$act, [int]$n = 20) {
    $t = New-Object System.Collections.Generic.List[double]
    $first = -1.0; $fail = 0
    for ($i = 0; $i -lt $n; $i++) {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        try { & $act | Out-Null } catch { $fail++ }
        $sw.Stop()
        if ($i -eq 0) { $first = $sw.Elapsed.TotalMilliseconds }
        $t.Add($sw.Elapsed.TotalMilliseconds)
    }
    if ($t.Count -eq 0) { return $null }
    $sorted = @($t | Sort-Object)
    $p95i = [int][math]::Floor(0.95 * ($sorted.Count - 1))
    [pscustomobject]@{
        First = [math]::Round($first, 1)
        Min   = [math]::Round($sorted[0], 1)
        Avg   = [math]::Round(($t | Measure-Object -Average).Average, 1)
        P95   = [math]::Round($sorted[$p95i], 1)
        Fail  = $fail
    }
}
function Test-Target([string]$name, [IntPtr]$hwnd, [scriptblock]$find) {
    Log ("--- " + $name + " (hwnd=" + $hwnd.ToInt64() + ") ---")
    if ($hwnd -eq [IntPtr]::Zero) { Log "  SKIP: khong co window"; return }
    $h = $hwnd
    $mFrom = Measure-It { [System.Windows.Automation.AutomationElement]::FromHandle($h) } 10
    Log ("  FromHandle ms: first=" + $mFrom.First + " avg=" + $mFrom.Avg + " p95=" + $mFrom.P95)
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
    $mf = Measure-It { $e = & $find $win; if (-not $e) { throw 'not found' } } 20
    if (-not $mf) { Log "  Find: LOI"; return }
    Log ("  Find ms:      first=" + $mf.First + " avg=" + $mf.Avg + " min=" + $mf.Min + " p95=" + $mf.P95 + " fail=" + $mf.Fail + "/20")
    $el = & $find $win
    if (-not $el) { Log "  element: KHONG TIM THAY (rule R10 -> unknown, can heuristic ClassName)"; return }
    # doc 6 property: do cold tung cai + warm toan bo
    $props = @(
        @('ControlType', [System.Windows.Automation.AutomationElement]::ControlTypeProperty),
        @('ClassName', [System.Windows.Automation.AutomationElement]::ClassNameProperty),
        @('Name', [System.Windows.Automation.AutomationElement]::NameProperty),
        @('AutomationId', [System.Windows.Automation.AutomationElement]::AutomationIdProperty),
        @('IsPassword', [System.Windows.Automation.AutomationElement]::IsPasswordProperty),
        @('IsKeyboardFocusable', [System.Windows.Automation.AutomationElement]::IsKeyboardFocusableProperty)
    )
    $cold = @()
    foreach ($p in $props) {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        $v = $el.Current.GetType().GetProperty($p[0]).GetValue($el.Current, $null)
        $sw.Stop()
        $cold += ($p[0] + "=" + $v + " (" + [math]::Round($sw.Elapsed.TotalMilliseconds, 1) + "ms)")
    }
    $mp = Measure-It {
        [void]$el.Current.ControlType; [void]$el.Current.ClassName; [void]$el.Current.Name
        [void]$el.Current.AutomationId; [void]$el.Current.IsPassword; [void]$el.Current.IsKeyboardFocusable
    } 20
    Log ("  cold props: " + ($cold -join ' | '))
    Log ("  6-props warm ms: first=" + $mp.First + " avg=" + $mp.Avg + " p95=" + $mp.P95)
}

$spawned = New-Object System.Collections.Generic.List[int]
# kill chrome cu cung profile (SingletonLock -> forward sang instance cu, pid moi khong co window)
foreach ($p in (Get-CimInstance Win32_Process -Filter "Name='chrome.exe'" | Where-Object { $_.CommandLine -like '*uia_spike_chrome_profile*' })) {
    try { & taskkill /PID $p.ProcessId /T /F 2>&1 | Out-Null } catch {}
}
Start-Sleep -Seconds 1
Log ("UIA spike WIN-004 - " + (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'))
Log "Budget P1-3 sec 2: query <= 2ms (async + cache 2s/hwnd)"

# === 1-3: Chrome fixture (omnibox, password, textarea) ===
$fix = ([System.Uri](Join-Path $PSScriptRoot 'uia_fixture.html')).AbsoluteUri
$ud = Join-Path $env:TEMP 'uia_spike_chrome_profile'
$ch = Start-Process 'C:\Program Files\Google\Chrome\Application\chrome.exe' `
    -ArgumentList @('--user-data-dir="' + $ud + '"', '--new-window', '--no-first-run', $fix) -PassThru
$spawned.Add($ch.Id)
$hChrome = Wait-Win $ch.Id '*fixture*' 30
if ($hChrome -eq [IntPtr]::Zero) {
    foreach ($w in [VtWin]::WinsOf($ch.Id)) { $p = $w.Split('|'); if ($p[1] -like 'Chrome_WidgetWin*') { $hChrome = [IntPtr][long]$p[0]; break } }
}
if ($hChrome -eq [IntPtr]::Zero) {
    # fallback: chrome co the forward sang instance cu (SingletonLock) -> tim toan bo theo title
    $all = Get-Process chrome -EA SilentlyContinue | Where-Object { $_.MainWindowTitle -like '*fixture*' } | Select-Object -First 1
    if ($all) {
        foreach ($w in [VtWin]::WinsOf([uint32]$all.Id)) { $p = $w.Split('|'); if ($p[1] -like 'Chrome_WidgetWin*' -and $p[2] -like '*fixture*') { $hChrome = [IntPtr][long]$p[0]; break } }
    }
}
Log ("Chrome window = " + $hChrome.ToInt64())
Test-Target 'chrome_omnibox (R3 address_bar)' $hChrome {
    param($w)
    $c = New-UiACond ([System.Windows.Automation.AutomationElement]::NameProperty) 'address'
    $e = $w.FindFirst($TS, $c)
    if (-not $e) { $c2 = New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::Edit); $e = $w.FindFirst($TS, $c2) }
    $e
}
Test-Target 'chrome_input_password (R1 secure)' $hChrome {
    param($w)
    $c = New-UiACond ([System.Windows.Automation.AutomationElement]::IsPasswordProperty) $true
    $w.FindFirst($TS, $c)
}
Test-Target 'chrome_textarea (R8)' $hChrome {
    param($w)
    $c = New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::TextArea)
    $w.FindFirst($TS, $c)
}

# === walk-all metric (tai sao phai dung condition) ===
if ($hChrome -ne [IntPtr]::Zero) {
    $winEl = [System.Windows.Automation.AutomationElement]::FromHandle($hChrome)
    $mWalk = Measure-It { $winEl.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition).Count } 10
    Log ("chrome FindAll ALL descendants: first=" + $mWalk.First + " avg=" + $mWalk.Avg + " p95=" + $mWalk.P95 + " (khong dung condition)")
}

# === 4: Notepad ===
$txt = Join-Path $env:TEMP 'uia_spike_note.txt'
Set-Content -Path $txt -Value ' VietIME UIA spike note' -Encoding ASCII
$np = Start-Process notepad.exe -ArgumentList ('"' + $txt + '"') -PassThru
$spawned.Add($np.Id)
$hNp = Wait-Win $np.Id '*' 30
if ($hNp -eq [IntPtr]::Zero) { foreach ($cand in [VtWin]::WinsByClass('Notepad')) { $hNp = [IntPtr][long]$cand.Split('|')[0]; break } }
Test-Target 'notepad_edit (R6 editbox)' $hNp {
    param($w)
    $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::Edit)))
    if (-not $e) { $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::Document))) }
    if (-not $e) { $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::TextArea))) }
    $e
}

# === 5: Word (COM -> co Document that) ===
$doc = $null; $word = $null
try {
    $word = New-Object -ComObject Word.Application
    $word.Visible = $true
    $doc = $word.Documents.Add()
    Start-Sleep -Seconds 2
    $hWord = [IntPtr]::Zero
    foreach ($w in [VtWin]::WinsByClass('OpusApp')) { $hWord = [IntPtr][long]$w.Split('|')[0]; break }
    $wpidObj = Get-Process WINWORD -EA SilentlyContinue | Select-Object -First 1
    if ($wpidObj) { $spawned.Add($wpidObj.Id) }
    if ($hWord -eq [IntPtr]::Zero) { $hWord = Wait-Win ([uint32]$wpidObj.Id) '*' 20 }
    Test-Target 'word_document (R6 body)' $hWord {
        param($w)
        $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::Document)))
        if (-not $e) { $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::TextArea))) }
        $e
    }
} catch { Log ("Word: LOI " + $_.Exception.Message.Split("`n")[0]) }

# === 6: Excel (COM -> Grid) ===
$wb = $null; $xl = $null
try {
    $xl = New-Object -ComObject Excel.Application
    $xl.Visible = $true
    $wb = $xl.Workbooks.Add()
    Start-Sleep -Seconds 2
    $hXl = [IntPtr]::Zero
    foreach ($w in [VtWin]::WinsByClass('XLMAIN')) { $hXl = [IntPtr][long]$w.Split('|')[0]; break }
    $xpidObj = Get-Process EXCEL -EA SilentlyContinue | Select-Object -First 1
    if ($xpidObj) { $spawned.Add($xpidObj.Id) }
    if ($hXl -eq [IntPtr]::Zero) { $hXl = Wait-Win ([uint32]$xpidObj.Id) '*' 20 }
    Test-Target 'excel_grid (R5 candidate)' $hXl {
        param($w)
        $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::Grid)))
        if (-not $e) { $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::Table))) }
        if (-not $e) { $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::DataItem))) }
        $e
    }
} catch { Log ("Excel: LOI " + $_.Exception.Message.Split("`n")[0]) }

# === 7: Windows Terminal (dang chay) ===
$hTerm = [IntPtr]::Zero
foreach ($cand in [VtWin]::WinsByClass('CASCADIA_HOSTING_WINDOW_CLASS')) { $hTerm = [IntPtr][long]$cand.Split('|')[0]; break }
Test-Target 'terminal (R9)' $hTerm {
    param($w)
    $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::Document)))
    if (-not $e) { $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::Edit))) }
    if (-not $e) { $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::Pane))) }
    $e
}

# === 8: VS Code (dang chay) ===
$hVs = [IntPtr]::Zero
$codePid = (Get-Process code -EA SilentlyContinue | Select-Object -First 1).Id
if ($codePid) {
    foreach ($w in [VtWin]::WinsOf([uint32]$codePid)) { $p = $w.Split('|'); if ($p[1] -like 'Chrome_WidgetWin*') { $hVs = [IntPtr][long]$p[0]; break } }
}
Test-Target 'vscode_editor (R7 web/electron)' $hVs {
    param($w)
    $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::TextArea)))
    if (-not $e) { $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::Document))) }
    $e
}

# === 9: Explorer (shell explorer.exe - khong kill!) ===
$hEx = [IntPtr]::Zero
foreach ($cand in [VtWin]::WinsByClass('CabinetWClass')) { $hEx = [IntPtr][long]$cand.Split('|')[0]; break }
if ($hEx -eq [IntPtr]::Zero) {
    Start-Process explorer.exe
    Start-Sleep -Seconds 4
    foreach ($cand in [VtWin]::WinsByClass('CabinetWClass')) { $hEx = [IntPtr][long]$cand.Split('|')[0]; break }
}
Test-Target 'explorer_address (R3/R4 address_bar+combo)' $hEx {
    param($w)
    $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::Edit)))
    if (-not $e) { $e = $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::ComboBox))) }
    $e
}

# === 10: conhost legacy (ky vong UIA khong tra edit -> heuristic) ===
$hc = $null
try { $hc = Start-Process conhost.exe -ArgumentList 'cmd.exe' -PassThru } catch { Log "conhost: khong khoi tao duoc" }
$hCon = [IntPtr]::Zero
if ($hc) { $spawned.Add($hc.Id); $hCon = Wait-Win $hc.Id '*' 15 }
if ($hCon -eq [IntPtr]::Zero) {
    foreach ($cand in [VtWin]::WinsByClass('ConsoleWindowClass')) { $hCon = [IntPtr][long]$cand.Split('|')[0]; break }
}
Test-Target 'conhost_legacy (R9/R10)' $hCon {
    param($w)
    $w.FindFirst($TS, (New-UiACond ([System.Windows.Automation.AutomationElement]::ControlTypeProperty) ([System.Windows.Automation.ControlType]::Edit)))
}

# === duong thuc hook that: FocusedElement ===
$mfoc = Measure-It {
    $f = [System.Windows.Automation.AutomationElement]::FocusedElement
    if ($f) { [void]$f.Current.ControlType; [void]$f.Current.IsPassword; [void]$f.Current.ClassName } else { throw 'null' }
} 20
Log ("FocusedElement path (hook that): first=" + $mfoc.First + " avg=" + $mfoc.Avg + " p95=" + $mfoc.P95 + " fail=" + $mfoc.Fail)

# === cleanup ===
try { if ($doc) { $doc.Close(0) } ; if ($word) { $word.Quit(0) } } catch {}
try { if ($wb) { $wb.Close($false) } ; if ($xl) { $xl.Quit() } } catch {}
Start-Sleep -Seconds 1
foreach ($id in ($spawned | Select-Object -Unique)) {
    # taskkill /T: dot het process tree (chrome co child) - chi kill pid minh spawn
    try { & taskkill /PID $id /T /F 2>&1 | Out-Null } catch {}
}
# chrome neu forward sang instance cu -> kill theo profile
foreach ($p in (Get-CimInstance Win32_Process -Filter "Name='chrome.exe'" | Where-Object { $_.CommandLine -like '*uia_spike_chrome_profile*' })) {
    try { & taskkill /PID $p.ProcessId /T /F 2>&1 | Out-Null } catch {}
}
Log 'DONE'
