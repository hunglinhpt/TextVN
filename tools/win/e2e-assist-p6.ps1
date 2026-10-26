# SPDX-License-Identifier: GPL-3.0-or-later
# tools/win/e2e-assist-p6.ps1 -DevDir <thu muc chua textvn_win_tsf.dll + textvn-cli.exe> [-Only <ten case>]
#
# E2E tren UNG DUNG THAT (Notepad Win11, Word, Excel) cho P6 du doan van ban, Ctrl+Space,
# tro ly ngu phap (popup on-demand + popup song R2-129), OP-002/003/004. Dang ky TSF DLL
# build tu cay hien tai lam HKCU override TAM, luon tra dang ky Program Files o finally.
# Build: $env:CARGO_TARGET_DIR=<thu muc tam>; cargo build --release -p textvn-win-tsf -p textvn-cli
# (E27: khong ghi de DLL dang duoc nap). Can ban cai Program Files (ProdCli) de khoi phuc.
# Chiem ban phim ~3 phut (SendInput). Bat che do V cho test va tra lai che do cua nguoi dung.
# Chi ghi chuoi test (S2). Ket qua 2026-10-26: 28/28 PASS (P6-TEXT-PREDICTION.md); Ctrl+Space sua ngay tu R2-131.
# Notepad/Edge/Notepad++ + phim tat Alt + ghost text: tools/win/e2e-assist-apps.ps1.
param(
    [Parameter(Mandatory = $true)][string]$DevDir,
    [string]$ProdCli = 'C:\Program Files\TextVN\textvn-cli.exe',
    [string]$Only = ''
)
$ErrorActionPreference = 'Stop'

Add-Type @'
using System;
using System.Text;
using System.Threading;
using System.Runtime.InteropServices;
public static class E2E {
    [StructLayout(LayoutKind.Sequential)]
    struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Explicit, Size = 40)]
    struct INPUT { [FieldOffset(0)] public uint type; [FieldOffset(8)] public KEYBDINPUT ki; }
    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] p, int cb);
    [DllImport("user32.dll")] static extern uint MapVirtualKeyW(uint code, uint type);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool f);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] public static extern short GetKeyState(int vk);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowExW(IntPtr parent, IntPtr after, string cls, string title);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowW(string cls, string title);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, StringBuilder l);
    [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, IntPtr l);
    const uint INPUT_KEYBOARD = 1, KEYEVENTF_KEYUP = 2;
    static INPUT Key(ushort vk, bool up) {
        var i = new INPUT(); i.type = INPUT_KEYBOARD; i.ki.wVk = vk;
        i.ki.wScan = (ushort)MapVirtualKeyW(vk, 0); i.ki.dwFlags = up ? KEYEVENTF_KEYUP : 0; return i;
    }
    public static void Tap(ushort vk) {
        if (SendInput(2, new[] { Key(vk, false), Key(vk, true) }, Marshal.SizeOf(typeof(INPUT))) != 2) throw new InvalidOperationException("SendInput failed");
        Thread.Sleep(45);
    }
    public static void Down(ushort vk) { SendInput(1, new[] { Key(vk, false) }, Marshal.SizeOf(typeof(INPUT))); Thread.Sleep(30); }
    public static void Up(ushort vk) { SendInput(1, new[] { Key(vk, true) }, Marshal.SizeOf(typeof(INPUT))); Thread.Sleep(30); }
    // '~' = Ctrl+Space, '<' = Esc, '\n' Enter, '\t' Tab, '=' '(' ')' ':' supported (US layout).
    public static void Type(string s) {
        foreach (char c in s) {
            if (c == ' ') { Tap(0x20); continue; }
            if (c == '\n') { Tap(0x0D); continue; }
            if (c == '\t') { Tap(0x09); continue; }
            if (c == ',') { Tap(0xBC); continue; }
            if (c == '.') { Tap(0xBE); continue; }
            if (c == '=') { Tap(0xBB); continue; }
            if (c == '<') { Tap(0x1B); continue; }
            if (c == '\b') { Tap(0x08); continue; }
            if (c == '>') { Tap(0x23); Thread.Sleep(100); continue; }  // End: engine forgets its tail
            if (c == '~') { Down(0x11); Tap(0x20); Up(0x11); Thread.Sleep(150); continue; }
            if (c == '(') { Down(0x10); Tap(0x39); Up(0x10); continue; }
            if (c == ')') { Down(0x10); Tap(0x30); Up(0x10); continue; }
            if (c == ':') { Down(0x10); Tap(0xBA); Up(0x10); continue; }
            if (char.IsDigit(c)) { Tap((ushort)c); continue; }
            ushort vk = (ushort)char.ToUpperInvariant(c);
            bool shift = char.IsUpper(c);
            if (shift) Down(0x10);
            Tap(vk);
            if (shift) Up(0x10);
        }
    }
    public static void CtrlShiftTap() { Down(0x11); Down(0x10); Up(0x10); Up(0x11); Thread.Sleep(150); }
    public static bool Focus(IntPtr h) {
        uint pid; uint fg = GetWindowThreadProcessId(GetForegroundWindow(), out pid); uint me = GetCurrentThreadId();
        AttachThreadInput(me, fg, true); ShowWindow(h, 5); BringWindowToTop(h); bool ok = SetForegroundWindow(h); AttachThreadInput(me, fg, false);
        Thread.Sleep(250); return ok && GetForegroundWindow() == h;
    }
    public static IntPtr NotepadEdit(IntPtr main) {
        IntPtr box = IntPtr.Zero;
        while ((box = FindWindowExW(main, box, "NotepadTextBox", null)) != IntPtr.Zero) {
            if (!IsWindowVisible(box)) continue;
            IntPtr e = FindWindowExW(box, IntPtr.Zero, "RichEditD2DPT", null);
            if (e != IntPtr.Zero) return e;
        }
        foreach (var cls in new[] { "Edit", "RichEditD2DPT" }) { IntPtr e = FindWindowExW(main, IntPtr.Zero, cls, null); if (e != IntPtr.Zero) return e; }
        return IntPtr.Zero;
    }
    public static string Text(IntPtr edit) {
        int n = (int)SendMessageW(edit, 0x000E, IntPtr.Zero, IntPtr.Zero);
        var sb = new StringBuilder(n + 1); SendMessageW(edit, 0x000D, (IntPtr)(n + 1), sb); return sb.ToString();
    }
    public static void Clear(IntPtr edit) { SendMessageW(edit, 0x000C, IntPtr.Zero, new StringBuilder("")); }
    public static bool OverlayVisible() {
        IntPtr h = IntPtr.Zero;
        while ((h = FindWindowExW(IntPtr.Zero, h, "TextVNAssistOverlay", null)) != IntPtr.Zero) { if (IsWindowVisible(h)) return true; }
        return false;
    }
}
'@

function U([string]$s) { [regex]::Unescape($s) }
function Codes([string]$s) { ($s.ToCharArray() | ForEach-Object { 'U+{0:X4}' -f [int]$_ }) -join ' ' }
function Norm([string]$s) { (($s -replace "`r`n", "`n") -replace "`r", "`n") -replace "\u000b", "`n" }
$script:pass = 0; $script:fail = 0
$script:log = New-Object System.Collections.Generic.List[string]
function Report([string]$app, [string]$name, [bool]$ok, [string]$want, [string]$got, [string]$extra) {
    if ($ok) { $script:pass++; $line = "PASS [$app] $name $extra" }
    else { $script:fail++; $line = "FAIL [$app] $name want [$want] got [$got] $extra" }
    Write-Host $line; $script:log.Add($line)
}


function Get-ViEnabled {
    try { return [bool]((Get-Content (Join-Path $env:APPDATA 'TextVN\state.json') -Raw | ConvertFrom-Json).global_enabled) } catch { return $true }
}
function Set-ViEnabled([bool]$want, [IntPtr]$focusWin) {
    for ($attempt = 0; $attempt -lt 3; $attempt++) {
        if ((Get-ViEnabled) -eq $want) { return $true }
        if ($focusWin -ne [IntPtr]::Zero) { [E2E]::Focus($focusWin) | Out-Null }
        [E2E]::CtrlShiftTap()
        for ($i = 0; $i -lt 10; $i++) { Start-Sleep -Milliseconds 200; if ((Get-ViEnabled) -eq $want) { return $true } }
    }
    return ((Get-ViEnabled) -eq $want)
}
$script:viOriginal = Get-ViEnabled
$script:viFocus = [IntPtr]::Zero
$devDll = Join-Path $DevDir 'textvn_win_tsf.dll'
$devCli = Join-Path $DevDir 'textvn-cli.exe'
foreach ($f in @($devDll, $devCli, $ProdCli)) { if (-not (Test-Path $f)) { throw "missing $f" } }
$hkcuInproc = 'HKCU:\Software\Classes\CLSID\{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}\InprocServer32'
$owned = New-Object System.Collections.Generic.List[int]
$word = $null; $xl = $null

try {
    & $devCli register --scope user --dll $devDll | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "dev register failed $LASTEXITCODE" }
    $reg = (Get-ItemProperty $hkcuInproc).'(default)'
    if ($reg -ne $devDll) { throw "HKCU override not pointing to dev DLL (got '$reg')" }
    Write-Host "dev DLL registered (HKCU override): $reg"

    $trace = Join-Path $env:TEMP 'textvn-e2e-trace.log'
    Remove-Item $trace -ErrorAction SilentlyContinue
    $env:TEXTVN_TSF_TRACE = $trace
    # ---------------- Notepad ----------------
    $before = @(Get-Process -Name notepad -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
    Start-Process notepad.exe | Out-Null
    $np = $null
    for ($i = 0; $i -lt 60 -and -not $np; $i++) {
        Start-Sleep -Milliseconds 500
        $np = Get-Process -Name notepad -ErrorAction SilentlyContinue | Where-Object { $before -notcontains $_.Id -and $_.MainWindowHandle -ne [IntPtr]::Zero } | Select-Object -First 1
    }
    if (-not $np) { throw 'notepad window not found' }
    $owned.Add($np.Id)
    $main = $np.MainWindowHandle
    $edit = [IntPtr]::Zero
    for ($i = 0; $i -lt 20 -and $edit -eq [IntPtr]::Zero; $i++) { $edit = [E2E]::NotepadEdit($main); if ($edit -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 250 } }
    if ($edit -eq [IntPtr]::Zero) { throw 'notepad edit not found' }
    $loaded = (Get-Process -Id $np.Id).Modules | Where-Object { $_.FileName -like '*textvn*' } | ForEach-Object { $_.FileName }

    function NP-Case([string]$name, [string]$keys, [scriptblock]$ok, [string]$want, [switch]$Overlay) {
        if ($Only -and $name -notlike "*$Only*") { return }
        for ($i = 0; $i -lt 10 -and -not [E2E]::Focus($main); $i++) { Start-Sleep -Milliseconds 300 }
        [E2E]::Clear($edit); Start-Sleep -Milliseconds 200
        [E2E]::Type('>')
        [E2E]::Type($keys)
        $got = $null; $res = $false
        for ($i = 0; $i -lt 15; $i++) { Start-Sleep -Milliseconds 200; $got = Norm ([E2E]::Text($edit)); if (& $ok $got) { $res = $true; break } }
        Report 'notepad' $name $res $want $got ''
        [E2E]::Type('<'); Start-Sleep -Milliseconds 150
    }
    function Eq([string]$w) { $w2 = $w; return { param($g) $g -ceq $w2 }.GetNewClosure() }

    # The tests need Vietnamese mode (V); the user's own mode is restored in finally.
    $script:viFocus = $main
    if (-not (Set-ViEnabled $true $main)) { throw 'could not switch TextVN to Vietnamese mode for the test' }
    Write-Host ("VI mode for test (user had: " + $script:viOriginal + ")")
    # warm up: TSF loads the TIP on first focus/keystroke
    [E2E]::Focus($main) | Out-Null; [E2E]::Type('a'); Start-Sleep -Milliseconds 600; [E2E]::Clear($edit)
    $loaded = (Get-Process -Id $np.Id).Modules | Where-Object { $_.FileName -like '*textvn*' } | ForEach-Object { $_.FileName }
    Write-Host ("notepad loaded: " + ($loaded -join ', '))

    $w = U 'ph\u00e1t tri\u1ec3n';                      NP-Case 'P6 prefix phat+Tab' "phats`t" (Eq $w) $w
    $w = U 'ph\u00e1t sinh';                            NP-Case 'P6 cycle phat+Tab+Tab' "phats`t`t" (Eq $w) $w
    $w = U 'ph\u00e1t tri\u1ec3n ph\u1ea7n m\u1ec1m';   NP-Case 'P6 context phat trien+Tab (R2-120)' "phats trieenr`t" (Eq $w) $w
    $w = U 'K\u00ednh g\u1eedi qu\u00fd kh\u00e1ch h\u00e0ng'; NP-Case 'P6 Kinh gui+Tab no space (R2-120)' "Kinhs guwir`t" (Eq $w) $w
    $w = U 'h\u1ee3p \u0111\u1ed3ng kinh t\u1ebf';      NP-Case 'P6 hop dong+Tab (R2-120)' "hopwj ddoongf`t" (Eq $w) $w
    $w = 'Thank you very much';                         NP-Case 'P6 EN Thank you+Tab' "Thank you`t" (Eq $w) $w
    $w = U 'Xin ch\u00e0o';                             NP-Case 'P6 casing Xin+Tab (R2-120)' "Xin `t" (Eq $w) $w
    $w = U 'C\u1ed9ng h\u00f2a x\u00e3 h\u1ed9i ch\u1ee7 ngh\u0129a Vi\u1ec7t Nam'; NP-Case 'P6 proper noun (R2-120)' "Coongj hoaf `t" (Eq $w) $w
    $w = (U 'K\u00ednh g\u1eedi') + "`n`t";             NP-Case 'P6 no prediction across newline (R2-120)' "Kinhs guwir`n`t" (Eq $w) $w
    $w = U 'ph\u00e1t tri\u1ec3n';                      NP-Case 'P6 Ctrl+Space prefix' "phats~" (Eq $w) $w
    # R2-121: Ctrl+Space with no prediction must release the word - next tone key starts a new word.
    $okA = U 'h\u1ecdcs'; $okB = U 'h\u1ecdc s'
    NP-Case 'Ctrl+Space no-match releases word (R2-121)' "hocj~s" { param($g) $g -ceq $okA -or $g -ceq $okB }.GetNewClosure() "$okA | $okB"
    $w = U 'M\u01b0\u1eddi l\u0103m tri\u1ec7u \u0111\u1ed3ng ch\u1eb5n'; NP-Case 'OP-003 =15000000bc+Space' "=15000000bc " (Eq $w) $w
    $w = U '\u0111\u01b0\u1ee3c ';                      NP-Case 'sanity dduocj' "dduocj " (Eq $w) $w

    if ($Only -and 'assist' -notlike "*$Only*") { throw 'ONLY-DONE' }
    # R2-131: Ctrl+Space fixes the whole sentence at once (no popup to pick from).
    for ($i = 0; $i -lt 10 -and -not [E2E]::Focus($main); $i++) { Start-Sleep -Milliseconds 300 }
    [E2E]::Clear($edit); Start-Sleep -Milliseconds 200
    [E2E]::Type('>')
    [E2E]::Type('tooi tooi ddi hocj'); Start-Sleep -Milliseconds 300
    [E2E]::Type('~'); Start-Sleep -Milliseconds 900
    $got = Norm ([E2E]::Text($edit)); $w = U 't\u00f4i \u0111i h\u1ecdc'
    Report 'notepad' 'Ctrl+Space fixes the whole sentence (R2-131)' ($got -ceq $w) $w $got ''
    [E2E]::Type('<')

    # R2-122: auto popup after sentence end must NOT swallow Tab.
    [E2E]::Clear($edit); Start-Sleep -Milliseconds 200
    [E2E]::Type('>')
    [E2E]::Type('tooi tooi ddi hocj.'); Start-Sleep -Milliseconds 900
    $ov = [E2E]::OverlayVisible()
    [E2E]::Type("`t"); Start-Sleep -Milliseconds 600
    $got = Norm ([E2E]::Text($edit)); $w = (U 't\u00f4i t\u00f4i \u0111i h\u1ecdc.') + "`t"
    Report 'notepad' 'auto popup lets Tab through (R2-122)' ($got -ceq $w) $w $got "overlay=$ov"
    [E2E]::Type('<')

    # R2-122: the pick-and-fix popup (Ctrl+Shift+G since R2-131) uses Tab to move between issues.
    [E2E]::Clear($edit); Start-Sleep -Milliseconds 200
    [E2E]::Type('>')
    [E2E]::Type('tooi tooi ddi hocj'); Start-Sleep -Milliseconds 300
    [E2E]::Down(0x11); [E2E]::Down(0x10); [E2E]::Tap(0x47); [E2E]::Up(0x10); [E2E]::Up(0x11); Start-Sleep -Milliseconds 700
    $ov = [E2E]::OverlayVisible()
    [E2E]::Type("`t"); Start-Sleep -Milliseconds 400
    $ov2 = [E2E]::OverlayVisible()
    [E2E]::Type('<'); Start-Sleep -Milliseconds 400
    $got = Norm ([E2E]::Text($edit)); $w = U 't\u00f4i t\u00f4i \u0111i h\u1ecdc'
    Report 'notepad' 'on-demand popup: Tab stays in popup (R2-122)' ($got -ceq $w -and $ov) $w $got "overlay=$ov afterTab=$ov2"

    # OP-004 collocation fixed by the Ctrl+Space hotkey (R2-131).
    [E2E]::Clear($edit); Start-Sleep -Milliseconds 200
    [E2E]::Type('>')
    [E2E]::Type('chungs tooi boor xung taif lieeuj'); Start-Sleep -Milliseconds 300
    [E2E]::Type('~'); Start-Sleep -Milliseconds 900
    $got = Norm ([E2E]::Text($edit)); $w = U 'ch\u00fang t\u00f4i b\u1ed5 sung t\u00e0i li\u1ec7u'
    Report 'notepad' 'OP-004 bo xung fixed by Ctrl+Space' ($got -ceq $w) $w $got ''
    [E2E]::Type('<')
    [E2E]::Clear($edit)

    # R2-129 live popup: wrong sentence -> shows at the word boundary; typing keeps it;
    # fixing the sentence hides it; Enter goes to the app (newline), never "applies".
    [E2E]::Clear($edit); Start-Sleep -Milliseconds 200; [E2E]::Type('>')
    [E2E]::Type('chungs tooi boor xung '); Start-Sleep -Milliseconds 500
    $ovShown = [E2E]::OverlayVisible()
    [E2E]::Type('t'); Start-Sleep -Milliseconds 300
    $ovKept = [E2E]::OverlayVisible()
    [E2E]::Type("`b`b`b`b`b`b"); [E2E]::Type('sung '); Start-Sleep -Milliseconds 500
    $ovHidden = -not [E2E]::OverlayVisible()
    $got = Norm ([E2E]::Text($edit)); $w = U 'ch\u00fang t\u00f4i b\u1ed5 sung '
    Report 'notepad' 'live popup shows on error, stays while typing, hides when fixed (R2-129)' ($ovShown -and $ovKept -and $ovHidden -and $got -ceq $w) $w $got "shown=$ovShown kept=$ovKept hidden=$ovHidden"
    [E2E]::Clear($edit); Start-Sleep -Milliseconds 200; [E2E]::Type('>')
    [E2E]::Type('tooi tooi ddi hocj '); Start-Sleep -Milliseconds 500
    $ovShown = [E2E]::OverlayVisible()
    [E2E]::Type("`nxin"); Start-Sleep -Milliseconds 500
    $got = Norm ([E2E]::Text($edit)); $w = (U 't\u00f4i t\u00f4i \u0111i h\u1ecdc ') + "`nXin"
    Report 'notepad' 'live popup: Enter is a newline, not a fix (R2-129)' ($ovShown -and $got -ceq $w -and -not [E2E]::OverlayVisible()) $w $got "shown=$ovShown"
    [E2E]::Type('<')
    [E2E]::Clear($edit)

    if ($Only) { throw 'ONLY-DONE' }
    # ---------------- Word (COM) ----------------
    $word = New-Object -ComObject Word.Application
    $word.Visible = $true
    $doc = $word.Documents.Add()
    # Word AutoCorrect (sentence caps, spelling-checker replacement) is Word's own
    # feature (typing "hangf " by hand also becomes "hang" in an English-proofed doc):
    # turn it off for the test and restore the user's values afterwards.
    $script:wordAc = @($word.AutoCorrect.CorrectSentenceCaps, $word.AutoCorrect.ReplaceTextFromSpellingChecker)
    $word.AutoCorrect.CorrectSentenceCaps = $false
    $word.AutoCorrect.ReplaceTextFromSpellingChecker = $false
    Start-Sleep -Seconds 2
    $wp = Get-Process -Name WINWORD | Sort-Object StartTime -Descending | Select-Object -First 1
    $owned.Add($wp.Id)
    $wmain = $wp.MainWindowHandle
    function W-Case([string]$name, [string]$keys, [string]$want) {
        for ($i = 0; $i -lt 10 -and -not [E2E]::Focus($wmain); $i++) { Start-Sleep -Milliseconds 300 }
        $doc.Content.Delete() | Out-Null; Start-Sleep -Milliseconds 200
        [E2E]::Type('>')
        [E2E]::Type($keys)
        $got = $null; $res = $false
        for ($i = 0; $i -lt 15; $i++) { Start-Sleep -Milliseconds 250; $got = Norm (($doc.Content.Text).TrimEnd("`r")); if ($got -ceq $want) { $res = $true; break } }
        Report 'word' $name $res $want $got ''
    }
    [E2E]::Focus($wmain) | Out-Null; [E2E]::Type('a'); Start-Sleep -Milliseconds 2500
    W-Case 'sanity Vieetj Nam' 'Vieetj Nam ' (U 'Vi\u1ec7t Nam ')
    W-Case 'P6 prefix phat+Tab' "phats`t" (U 'ph\u00e1t tri\u1ec3n')
    W-Case 'P6 context phat trien+Tab' "phats trieenr`t" (U 'ph\u00e1t tri\u1ec3n ph\u1ea7n m\u1ec1m')
    W-Case 'P6 Kinh gui+Tab' "Kinhs guwir`t" (U 'K\u00ednh g\u1eedi qu\u00fd kh\u00e1ch h\u00e0ng')
    W-Case 'P6 Ctrl+Space prefix' "phats~" (U 'ph\u00e1t tri\u1ec3n')
    W-Case 'OP-003 =125000d+Space' "=125000d " (U '125.000 \u0111\u1ed3ng')
    $word.AutoCorrect.CorrectSentenceCaps = $script:wordAc[0]
    $word.AutoCorrect.ReplaceTextFromSpellingChecker = $script:wordAc[1]
    $doc.Saved = $true; $doc.Close(); $doc = $null
    $word.Quit(); [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($word); $word = $null

    # ---------------- Excel (COM) ----------------
    $xl = New-Object -ComObject Excel.Application
    $xl.Visible = $true
    $wb = $xl.Workbooks.Add()
    $ws = $wb.Worksheets.Item(1)
    Start-Sleep -Seconds 2
    $xp = Get-Process -Name EXCEL | Sort-Object StartTime -Descending | Select-Object -First 1
    $owned.Add($xp.Id)
    $xmain = $xp.MainWindowHandle
    for ($i = 0; $i -lt 10 -and -not [E2E]::Focus($xmain); $i++) { Start-Sleep -Milliseconds 300 }
    $ws.Range('C1').Select() | Out-Null
    [E2E]::Type("=sum(a1:a2)`n"); Start-Sleep -Milliseconds 800
    $f = $ws.Range('C1').Formula
    Report 'excel' 'OP-002 formula guard =sum(a1:a2)' ($f -ceq '=SUM(A1:A2)') '=SUM(A1:A2)' $f ''
    $ws.Range('C3').Select() | Out-Null
    [E2E]::Type("Vieetj Nam`n"); Start-Sleep -Milliseconds 800
    $v = [string]$ws.Range('C3').Value2
    Report 'excel' 'Vietnamese in a cell' ($v -ceq (U 'Vi\u1ec7t Nam')) (U 'Vi\u1ec7t Nam') $v ''
    $ws.Range('C5').Select() | Out-Null
    [E2E]::Type("phats`t`n"); Start-Sleep -Milliseconds 800
    $v = [string]$ws.Range('C5').Value2
    Report 'excel' 'Tab in a cell moves to the next cell (Excel owns Tab, no completion)' ($v -ceq (U 'ph\u00e1t')) (U 'ph\u00e1t') $v ''
    $wb.Close($false); $xl.Quit(); [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($xl); $xl = $null
}
catch { if ($_.Exception.Message -ne 'ONLY-DONE') { throw } }
finally {
    if (-not $script:viOriginal) {
        $np2 = Get-Process -Name notepad -ErrorAction SilentlyContinue | Where-Object { $owned -contains $_.Id } | Select-Object -First 1
        $fw = if ($np2) { $np2.MainWindowHandle } else { [IntPtr]::Zero }
        $back = Set-ViEnabled $false $fw
        Write-Host ("restore user mode E: " + $back)
    }
    try { if ($word) { $word.Quit() } } catch {}
    try { if ($xl) { $xl.Quit() } } catch {}
    foreach ($id in $owned) { Stop-Process -Id $id -Force -ErrorAction SilentlyContinue }
    Start-Sleep -Milliseconds 500
    & $ProdCli register | Out-Host
    $after = (Get-ItemProperty $hkcuInproc -ErrorAction SilentlyContinue).'(default)'
    Write-Host ("restore: HKCU override after = '" + $after + "' (expected empty)")
    if ($trace -and (Test-Path $trace)) { Write-Host '--- trace (assist lines) ---'; Get-Content $trace | Where-Object { $Only -or $_ -match 'assist|ctrl' } | Select-Object -Last 60 }
    Write-Host ("SUMMARY pass=" + $script:pass + " fail=" + $script:fail)
}
