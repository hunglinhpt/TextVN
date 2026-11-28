# SPDX-License-Identifier: GPL-3.0-or-later
# test-typing.ps1 -Dir <thu muc chua TextVN.exe, textvn-cli.exe, textvn-tsf.dll>
#
# Go tieng Viet THAT qua TSF: mo Notepad, gui phim bang SendInput (VK + scan code nhu ban
# phim that - KHONG dung KEYEVENTF_UNICODE vi TIP bo qua VK_PACKET), doc lai noi dung o
# soan thao va so voi ky vong. Dung chung cho kich ban portable va kich ban cai dat.
# Chi ghi ket qua dung/sai + ma ky tu (S2: khong ghi text nguoi dung that).

param(
    [Parameter(Mandatory = $true)][string]$Dir,
    # ARM-W5: cac app bo sung cung kien truc khac (duong dan exe, phan cach ';')
    # de E2E ma tran tien trinh. Env TEXTVN_E2E_EXTRA_EDITORS cung gia tri nay -
    # test-portable khong can doi.
    [string]$ExtraApps = $env:TEXTVN_E2E_EXTRA_EDITORS
)
$ErrorActionPreference = 'Stop'

if (-not ('TvKeys' -as [type])) {
    Add-Type @'
using System;
using System.Text;
using System.Threading;
using System.Runtime.InteropServices;
public static class TvKeys {
    [StructLayout(LayoutKind.Sequential)]
    struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Sequential)]
    struct MOUSEINPUT { public int dx; public int dy; public uint mouseData; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Explicit, Size = 40)]
    struct INPUT { [FieldOffset(0)] public uint type; [FieldOffset(8)] public KEYBDINPUT ki; [FieldOffset(8)] public MOUSEINPUT mi; }
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] p, int cb);
    [DllImport("user32.dll")] static extern uint MapVirtualKeyW(uint code, uint type);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool f);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] public static extern IntPtr GetKeyboardLayout(uint tid);
    [DllImport("user32.dll")] public static extern short GetKeyState(int vk);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowExW(IntPtr parent, IntPtr after, string cls, string title);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, StringBuilder l);
    [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, IntPtr l);
    // E89: thang focus cho session khong co foreground window (runner windows-11-arm:
    // GetForegroundWindow() = NULL lam AttachThreadInput(0) vo nghia va SetForegroundWindow
    // tu choi theo thuat toan foreground-lock cua Win32).
    [DllImport("user32.dll")] static extern void SwitchToThisWindow(IntPtr h, bool altTab);
    [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] static extern int GetSystemMetrics(int i);
    // E89 vong 2: Notepad tren runner co the MO O TRANG THAI MINIMIZED - GetWindowRect
    // luc do tra (-32000,-32000) nen click absolute-normalize roi vao goc man hinh,
    // khong bao gio cham cua so (bang chung: ci-windows 9a2dbe64 fg=0x901E8 khong doi).
    [DllImport("user32.dll")] static extern bool IsIconic(IntPtr h);
    [DllImport("user32.dll")] static extern bool GetCursorPos(out POINT p);
    [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
    // E89 vong 3: chan doan quyet dinh - click phai tinh theo VI MAN HINH AO (goc
    // SM_XVIRTUALSCREEN/Y origin, kich thuoc CX/CYVIRTUALSCREEN); tren session DPI-scale
    // hoac multi-monitor, normalize theo man hinh chinh lam con tro roi sai cho. Neu
    // SetCursorPos + GetCursorPos khong ket xuyen thi session khong route input.
    [DllImport("user32.dll", SetLastError = true)] static extern IntPtr OpenInputDesktop(uint flags, bool inherit, uint access);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern bool GetUserObjectInformationW(IntPtr h, int idx, System.Text.StringBuilder buf, int len, out int needed);
    [DllImport("user32.dll")] static extern bool CloseDesktop(IntPtr h);
    [DllImport("user32.dll")] static extern IntPtr GetThreadDesktop(uint tid);
    [StructLayout(LayoutKind.Sequential)]
    struct POINT { public int X; public int Y; }

    // Chan doan cho PS: cua so co dang minimized khong (E89v2).
    public static bool IsWindowMinimized(IntPtr h) { return IsIconic(h); }

    // E89v3: ten desktop cua thread goi va desktop NHAP LUAU (neu khac nhau hoac
    // OpenInputDesktop that bai = session khong nhan input = khong the E2E go chu).
    public static string InputDesktopDiag() {
        var sb = new StringBuilder(256);
        int need;
        string cur = "?";
        IntPtr td = GetThreadDesktop(GetCurrentThreadId());
        if (td != IntPtr.Zero && GetUserObjectInformationW(td, 2 /*UOI_NAME*/, sb, 256, out need)) cur = sb.ToString();
        string inp = "?";
        IntPtr id = OpenInputDesktop(0, false, 0x0200 /*DESKTOP_READOBJECTS*/);
        if (id != IntPtr.Zero) {
            sb.Clear();
            if (GetUserObjectInformationW(id, 2, sb, 256, out need)) inp = sb.ToString();
            CloseDesktop(id);
            return "thread-desktop=" + cur + " input-desktop=" + inp;
        }
        return "thread-desktop=" + cur + " input-desktop=KHONG MO DUOC (session khong route input)";
    }

    // E89v3: kiem chung duong con tro - di chuyen toi (x,y) roi doc lai; sai >4px
    // nghia la session khong nhan input (khong cam click nua - vo ich).
    public static bool CursorMovesTo(int x, int y) {
        if (!SetCursorPos(x, y)) return false;
        Thread.Sleep(120);
        POINT p;
        if (!GetCursorPos(out p)) return false;
        return Math.Abs(p.X - x) <= 4 && Math.Abs(p.Y - y) <= 4;
    }

    // E89v3: dong bo chan doan con tro cho PS dump.
    public static string CursorWhere() {
        POINT p;
        if (!GetCursorPos(out p)) return "?";
        return p.X + "," + p.Y;
    }


    const uint INPUT_KEYBOARD = 1, KEYEVENTF_KEYUP = 2, KEYEVENTF_EXTENDEDKEY = 1;

    static INPUT Key(ushort vk, bool up) {
        var i = new INPUT();
        i.type = INPUT_KEYBOARD;
        i.ki.wVk = vk;
        i.ki.wScan = (ushort)MapVirtualKeyW(vk, 0);
        i.ki.dwFlags = up ? KEYEVENTF_KEYUP : 0;
        return i;
    }

    public static void Tap(ushort vk) {
        if (SendInput(2, new[] { Key(vk, false), Key(vk, true) }, Marshal.SizeOf(typeof(INPUT))) != 2)
            throw new InvalidOperationException("SendInput failed");
        Thread.Sleep(40);
    }

    public static void Down(ushort vk) { if (SendInput(1, new[] { Key(vk, false) }, Marshal.SizeOf(typeof(INPUT))) != 1) throw new InvalidOperationException("SendInput failed"); Thread.Sleep(30); }
    public static void Up(ushort vk) { if (SendInput(1, new[] { Key(vk, true) }, Marshal.SizeOf(typeof(INPUT))) != 1) throw new InvalidOperationException("SendInput failed"); Thread.Sleep(30); }

    // Go chuoi ASCII: chu hoa = Shift + phim (tru khi Caps Lock dang bat), ' ' = Space,
    // '\n' = Enter, '\t' = Tab, ',' '.' = phim dau cau (layout US), '^' = Home.
    public static void Type(string s) { Type(s, (GetKeyState(0x14) & 1) != 0); }

    // GetKeyState chi phan anh trang thai cua thread goi khi thread do da xu ly
    // input; ngay sau SendInput(CapsLock) no co the con cu -> tuong Caps tat, bam
    // Shift cho chu hoa, Shift+Caps ra chu thuong (CI: 'viet' thay vi 'VIET').
    // Test tu bat Caps Lock thi truyen trang thai tuong minh.
    public static void Type(string s, bool caps) {
        foreach (char c in s) {
            if (c == ' ') { Tap(0x20); continue; }
            if (c == '\n') { Tap(0x0D); continue; }
            if (c == '\t') { Tap(0x09); continue; }
            if (c == ',') { Tap(0xBC); continue; }
            if (c == '.') { Tap(0xBE); continue; }
            if (c == '^') { Tap(0x24); continue; }  // Home
            ushort vk = (ushort)char.ToUpperInvariant(c);
            bool shift = char.IsUpper(c) != caps && char.IsLetter(c);
            if (shift) Down(0x10);
            Tap(vk);
            if (shift) Up(0x10);
        }
    }

    // Ctrl+Shift nhan roi nha (phim chuyen kieu UniKey).
    public static void CtrlShiftTap() { Down(0x11); Down(0x10); Up(0x10); Up(0x11); Thread.Sleep(80); }

    public static bool Focus(IntPtr h) {
        uint pid;
        uint fgThread = GetWindowThreadProcessId(GetForegroundWindow(), out pid);
        uint me = GetCurrentThreadId();
        AttachThreadInput(me, fgThread, true);
        ShowWindow(h, 9); /*SW_RESTORE - SW_SHOW(5) khong go bo minimized (E89 v2)*/
        BringWindowToTop(h);
        bool ok = SetForegroundWindow(h);
        AttachThreadInput(me, fgThread, false);
        Thread.Sleep(200);
        return ok && GetForegroundWindow() == h;
    }

    // E89: click chuot that vao tam cua so chinh - duong focus tin cay nhat khi session
    // khong co foreground window: click dat foreground + focus edit cung luc (edit cua
    // Notepad/WordPad phu phan lon client nen tam cua so nam trong vung go chu).
    // Vong 2: khoi phuc truoc neu minimized (rect -32000 lam click roi goc man hinh)
    // va kiem tra toa do nam tren man hinh truoc khi gui click.
    public static bool ClickFocus(IntPtr main, IntPtr edit) {
        if (IsIconic(main)) {
            ShowWindow(main, 9); /*SW_RESTORE*/
            Thread.Sleep(400);
        }
        RECT r;
        if (!GetWindowRect(main, out r)) return false;
        int vx = GetSystemMetrics(76), vy = GetSystemMetrics(77);
        int vw = GetSystemMetrics(78), vh = GetSystemMetrics(79);
        if (vw <= 0 || vh <= 0) { vx = 0; vy = 0; vw = GetSystemMetrics(0); vh = GetSystemMetrics(1); }
        int cx = (r.Left + r.Right) / 2, cy = (r.Top + r.Bottom) / 2;
        if (cx < vx || cx >= vx + vw || cy < vy || cy >= vy + vh) {
            return false; /*tam cua so ngoai man hinh ao - khong gui click muu*/
        }
        // E89v3: kiem chung duong con truoc - SetCursorPos den tam roi doc lai;
        // khong ket xuyen = session khong nhan input, click se vo ich.
        if (!CursorMovesTo(cx, cy)) return false;
        int ax = (int)((long)(cx - vx) * 65535 / (vw - 1));
        int ay = (int)((long)(cy - vy) * 65535 / (vh - 1));
        const uint MOVE = 0x0001 | 0x4000 /*ABSOLUTE*/, DOWN = 0x0002, UP = 0x0004;
        var seq = new[] {
            new INPUT { type = 0 }, new INPUT { type = 0 }, new INPUT { type = 0 } };
        seq[0].mi = new MOUSEINPUT { dx = ax, dy = ay, dwFlags = MOVE };
        seq[1].mi = new MOUSEINPUT { dwFlags = DOWN };
        seq[2].mi = new MOUSEINPUT { dwFlags = UP };
        if (SendInput(3, seq, Marshal.SizeOf(typeof(INPUT))) != 3) return false;
        Thread.Sleep(250);
        // Chan doan: con tro phai den dung diem click - neu khong thi bang chung
        // click bi an (vi du cursor bi khop) chu khong phai loi cua so.
        POINT cp;
        GetCursorPos(out cp);
        return GetForegroundWindow() == main
            && Math.Abs(cp.X - cx) <= 4 && Math.Abs(cp.Y - cy) <= 4;
    }

    // E89: thang focus 4 muc - tra ten muc thanh cong ("" neu that bai het). Mun
    // chan danh cho CI: muc (1) nhanh; (2) SwitchToThisWindow cho session "lanh";
    // (3) Alt-trick mo khoa foreground-lock; (4) click that vao edit (dat FOREGROUND
    // + focus cung luc - khong bao gio chet tren runner co desktop tuong tac).
    public static string FocusLadder(IntPtr main, IntPtr edit) {
        if (Focus(main)) return "setforeground";
        for (int k = 0; k < 3; k++) {
            SwitchToThisWindow(main, true);
            Thread.Sleep(200);
            if (GetForegroundWindow() == main) return "switchtothiswindow";
        }
        for (int k = 0; k < 3; k++) {
            SendInput(2, new[] { Key(0x12 /*VK_MENU*/, false), Key(0x12, true) },
                Marshal.SizeOf(typeof(INPUT)));
            if (Focus(main)) return "alt-trick";
        }
        for (int k = 0; k < 3; k++) {
            Focus(main);
            if (ClickFocus(main, edit)) return "click-edit";
        }
        return "";
    }

    public static IntPtr EditOf(IntPtr main) {
        foreach (var cls in new[] { "Edit", "RichEditD2DPT", "RICHEDIT50W" }) {
            IntPtr e = FindWindowExW(main, IntPtr.Zero, cls, null);
            if (e != IntPtr.Zero) return e;
        }
        // Notepad Win11 (ban Store): moi tab mot NotepadTextBox > RichEditD2DPT, tab
        // khoi phuc tu phien truoc cung nam day. Chi lay tab DANG HIEN (tab moi mo).
        IntPtr box = IntPtr.Zero;
        while ((box = FindWindowExW(main, box, "NotepadTextBox", null)) != IntPtr.Zero) {
            if (!IsWindowVisible(box)) continue;
            IntPtr e = FindWindowExW(box, IntPtr.Zero, "RichEditD2DPT", null);
            if (e != IntPtr.Zero) return e;
        }
        return IntPtr.Zero;
    }

    public static string Text(IntPtr edit) {
        int n = (int)SendMessageW(edit, 0x000E /*WM_GETTEXTLENGTH*/, IntPtr.Zero, IntPtr.Zero);
        var sb = new StringBuilder(n + 1);
        SendMessageW(edit, 0x000D /*WM_GETTEXT*/, (IntPtr)(n + 1), sb);
        return sb.ToString();
    }

    public static void Clear(IntPtr edit) {
        SendMessageW(edit, 0x000C /*WM_SETTEXT*/, IntPtr.Zero, new StringBuilder(""));
    }
}
'@
}

function U([string]$s) { [regex]::Unescape($s) }
function Codes([string]$s) { ($s.ToCharArray() | ForEach-Object { 'U+{0:X4}' -f [int]$_ }) -join ' ' }

# Focus must be VERIFIED before every burst: a single SetForegroundWindow call
# loses races on slow shared runners and keys land nowhere ("got []" FAILs).
# E89: session khong co foreground window (windows-11-arm) lam SetForegroundWindow
# tu choi -> thang 4 muc (setforeground/switch/alt-trick/click-edit) + dump chan doan.
function Ensure-Focus($app) {
    for ($i = 0; $i -lt 10; $i++) {
        $how = [TvKeys]::FocusLadder($app.Main, $app.Edit)
        if ($how -ne '') {
            if ($i -gt 0) { Write-Host "focus OK (lan $i, kenh: $how)" }
            return
        }
        Start-Sleep -Milliseconds 300
    }
    $fg = [TvKeys]::GetForegroundWindow()
    throw "could not focus $($app.Name) main window (E89v3: fg=0x$($fg.ToString('X')), main=0x$($app.Main.ToString('X')), iconic=$([TvKeys]::IsWindowMinimized($app.Main)), cursor=$([TvKeys]::CursorWhere()), $([TvKeys]::InputDesktopDiag()))"
}

# Clear must be VERIFIED before typing the next case: keys landing after a lost
# clear corrupt the following verdict.
function Clear-Verified($app) {
    for ($i = 0; $i -lt 10; $i++) {
        [TvKeys]::Clear($app.Edit)
        Start-Sleep -Milliseconds 150
        if ([TvKeys]::Text($app.Edit).Length -eq 0) { return }
    }
    throw "could not clear $($app.Name) edit control"
}

$cli = Join-Path $Dir 'textvn-cli.exe'
$tray = Join-Path $Dir 'TextVN.exe'
foreach ($f in @($cli, $tray)) { if (-not (Test-Path $f)) { throw "missing $f" } }

# Smoke test thay doi input source va mode; khong dung instance cua nguoi dung.
if (Get-Process -Name TextVN,notepad,wordpad -ErrorAction SilentlyContinue) {
    throw 'Close existing TextVN/Notepad/WordPad before running the destructive typing smoke test'
}

# E89 (ket luan 2026-11-28, 4 vong bang chung 46113896660/46116060861/cb28db20):
# runner windows-11-arm (PROCESSOR_ARCHITECTURE=ARM64) khong route input - thang
# focus 4 muc + click virtual-screen + probe cursor deu bat kiet: foreground KHONG
# doi, con tro KHONG den dung diem. CANH BAO: OpenInputDesktop that bai KHONG phai
# dau hieu - no cung that bai tren windows-2022 noi typing van hoat dong (run
# 504960ff skip nham ca 2022/2025 - da sua). Skip theo ARCH, x64 luon go that.
if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
    Write-Host 'SKIP E2E go chu (E89): anh runner Windows ARM64 (windows-11-arm) hien khong route input - thang focus/click/probe cursor deu bat kiet (bang chung run 46116060861 + cb28db20).'
    Write-Host '  Cac gate khac cua job ARM64 (build/verify/PE/ACL/installer/portable structure) van chay that. Ma tran kien truc x64/x86 do o job windows-2022/2025.'
    exit 0
}
$script:ownedEditor = $null
# Cac case gia dinh dang o che do V. May dev co the dang de E (state.json
# global_enabled=false) -> bat V cho phien test, finally tra lai NGUYEN VAN file
# cua nguoi dung (Ctrl+Shift trong test cung ghi file nay).
$stateFile = Join-Path $env:APPDATA 'TextVN\state.json'
$stateBackup = $null
# R2-98: tai khoan da tat tu khoi dong (marker) thi tray '--autostart' tu thu muc ma muc
# Run HKLM tro toi se thoat ngay - cat marker sang ben trong luc test, finally tra lai.
$autoMarker = Join-Path $env:APPDATA 'TextVN\autostart_disabled'
$autoMarkerAside = "$autoMarker.test-typing"
$autoMarkerMoved = $false
try {
if (Test-Path -LiteralPath $autoMarker) {
    Move-Item -LiteralPath $autoMarker -Destination $autoMarkerAside -Force
    $autoMarkerMoved = $true
}
if (Test-Path $stateFile) {
    $stateBackup = [System.IO.File]::ReadAllText($stateFile)
    $st = $stateBackup | ConvertFrom-Json
    if ($st.global_enabled -eq $false) {
        $st.global_enabled = $true
        [System.IO.File]::WriteAllText($stateFile, ($st | ConvertTo-Json -Depth 10), (New-Object System.Text.UTF8Encoding $false))
        Write-Host 'NOTE state.json: global_enabled=false -> true cho phien test (se tra lai)'
    }
}
$ownedTray = Start-Process -FilePath $tray -ArgumentList '--autostart' -WorkingDirectory $Dir -WindowStyle Hidden -PassThru
$ready = $false
for ($i = 0; $i -lt 30 -and -not $ready; $i++) {
    Start-Sleep -Milliseconds 500
    $ready = ((& $tray --status 2>&1 | Out-String) -match 'RUNNING')
}
if (-not $ready) { throw 'TextVN tray did not start' }

# Kich hoat profile TextVN cho phien (giong nguoi dung chon TextVN o thanh ngon ngu).
& $cli register | Out-Host
if ($LASTEXITCODE -ne 0) { throw "TextVN TSF registration failed ($LASTEXITCODE)" }
# "Danh Ctrl + Shift cho TextVN" (installer mac dinh chon; ban portable: o trong Bang dieu
# khien): phim tat doi bo cuc cua Windows khong con nuot Ctrl+Shift.
& $tray --free-ctrl-shift | Out-Host
if ($LASTEXITCODE -ne 0) { throw "TextVN Ctrl+Shift configuration failed ($LASTEXITCODE)" }

$cases = @(
    @{ name = 'telex dduocj';      keys = 'dduocj ';       want = (U '\u0111\u01b0\u1ee3c ') },
    @{ name = 'Vieetj Nam';        keys = 'Vieetj Nam ';   want = (U 'Vi\u1ec7t Nam ') },
    @{ name = 'nguowif (uow)';     keys = 'nguowif ';      want = (U 'ng\u01b0\u1eddi ') },
    @{ name = 'cuar (tone rule)';  keys = 'cuar ';         want = (U 'c\u1ee7a ') },
    @{ name = 'english hello';     keys = 'hello ';        want = 'hello ' },
    @{ name = 'comma boundary';    keys = 'Vieetj, Nam ';  want = (U 'Vi\u1ec7t, Nam ') },
    # Enter phai toi app nhu phim that; sau Enter chu dau cau tu viet hoa (mac dinh).
    @{ name = 'Enter boundary';    keys = "chaof`nbanj ";  want = (U "ch\u00e0o`nB\u1ea1n ") },
    @{ name = 'Tab boundary';      keys = "tieengs`tx ";   want = (U "ti\u1ebfng`tx ") },
    # Phim dieu huong khi dang go: tu duoc chot tai cho, Home di dung ve dau dong.
    @{ name = 'Home mid-word';     keys = 'chaof^x ';      want = (U 'x ch\u00e0o') }
)

$script:fail = 0
$script:results = New-Object System.Collections.Generic.List[string]
# Edit tra "\r\n", RichEdit tra "\r": quy ve "\n" truoc khi so.
function Norm([string]$s) { ($s -replace "`r`n", "`n") -replace "`r", "`n" }
function Check($app, [string]$name, [string]$want) {
    # TSF composition is asynchronous. Two equal reads are NOT proof of final
    # state: a busy editor can stay unchanged for >200ms then commit later.
    # Poll up to 4s for exact match; only the final observed value can fail.
    $got = $null
    for ($i = 0; $i -lt 20; $i++) {
        Start-Sleep -Milliseconds 200
        $cur = Norm ([TvKeys]::Text($app.Edit))
        if ($cur -ceq $want) { $got = $cur; break }
        $got = $cur
    }
    if ($got -ceq $want) {
        $line = "PASS [{0}] {1}" -f $app.Name, $name
    } else {
        $line = "FAIL [{0}] {1}: want [{2}] got [{3}]" -f $app.Name, $name, (Codes $want), (Codes $got)
        $script:fail++
    }
    Write-Host $line
    $script:results.Add($line)
    Clear-Verified $app
}

function Open-App([string]$name, [string]$exe) {
    $procName = [System.IO.Path]::GetFileNameWithoutExtension($exe)
    # B5: runner GHA dung chung doi khi mo app cham hon 30 s (2026-10-12, PR #11:
    # "wordpad window not found after 30 s", rerun cung commit xanh). Thu toi da 2 lan;
    # truoc lan 2 chi dong DUNG tien trinh do chinh lan thu truoc mo ra.
    for ($attempt = 1; $attempt -le 2; $attempt++) {
        $before = @(Get-Process -Name $procName -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
        $p = Start-Process $exe -PassThru
        $script:ownedEditor = $p
        $main = [IntPtr]::Zero
        # Never attach to another user's document if the newly started process
        # hands off to an existing editor instance.
        for ($i = 0; $i -lt 60 -and $main -eq [IntPtr]::Zero; $i++) {
            Start-Sleep -Milliseconds 500
            $p.Refresh()
            if (-not $p.HasExited) {
                # Tien trinh da thoat thi MainWindowHandle la $null - khong bang IntPtr.Zero.
                if ($p.MainWindowHandle) { $main = $p.MainWindowHandle }
                continue
            }
            # Notepad Win11: notepad.exe la launcher, thoat ngay; cua so nam o tien trinh
            # Notepad MOI. Dau script da tu choi chay khi co Notepad mo san nen tien trinh
            # moi la cua test, khong phai tai lieu cua nguoi dung.
            $q = Get-Process -Name $procName -ErrorAction SilentlyContinue |
                Where-Object { $before -notcontains $_.Id -and $_.MainWindowHandle -ne [IntPtr]::Zero } |
                Select-Object -First 1
            if ($q) {
                $p = $q
                $script:ownedEditor = $q
                $main = $q.MainWindowHandle
            }
        }
        if ($main -ne [IntPtr]::Zero) { break }
        Write-Host "WARN: $name window not found after 30 s (attempt $attempt/2) - slow shared runner (B5)"
        Get-Process -Name $procName -ErrorAction SilentlyContinue |
            Where-Object { $before -notcontains $_.Id } |
            Stop-Process -Force -ErrorAction SilentlyContinue
    }
    if ($main -eq [IntPtr]::Zero) { throw "$name window not found after 2 x 30 s" }
    $edit = [IntPtr]::Zero
    for ($i = 0; $i -lt 20 -and $edit -eq [IntPtr]::Zero; $i++) {
        $edit = [TvKeys]::EditOf($main)
        if ($edit -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 250 }
    }
    if ($edit -eq [IntPtr]::Zero) { throw "$name edit control not found" }
    @{ Name = $name; Proc = $p; Main = $main; Edit = $edit }
}

# Notepad (Win32 Edit, IMM32 qua CUAS) va WordPad (RichEdit, TSF-aware) neu co.
$apps = @(@{ Name = 'notepad'; Exe = 'notepad.exe' })
$wordpad = Join-Path $env:ProgramFiles 'Windows NT\Accessories\wordpad.exe'
if (Test-Path $wordpad) { $apps += @{ Name = 'wordpad'; Exe = $wordpad } }

# ARM-W5: app bo sung cung kien truc khac (x64 gia lap / x86 tren may ARM64) -
# chung minh duong TIP dung kien truc (forwarder ARM64X -> DLL x64/x86) hoat dong
# trong tien trinh cua kien truc do, khong phai chi DLL ARM64.
foreach ($raw in ($ExtraApps -split ';' | Where-Object { $_ -and $_.Trim() })) {
    $exe = $raw.Trim()
    if (-not (Test-Path $exe)) { throw "extra e2e app missing: $exe" }
    $apps += @{ Name = [System.IO.Path]::GetFileNameWithoutExtension($exe); Exe = (Resolve-Path $exe).Path }
}

# Nhat ky chan doan cua key sink (khong chua noi dung go) - app ke thua bien moi truong.
$trace = Join-Path $env:TEMP 'textvn-tsf-trace.log'
Remove-Item $trace -ErrorAction SilentlyContinue
$env:TEXTVN_TSF_TRACE = $trace

foreach ($a in $apps) {
    Add-Content -Path $trace -Value ("=== " + $a.Name)
    $app = Open-App $a.Name $a.Exe
    Ensure-Focus $app
    Clear-Verified $app

    foreach ($c in $cases) {
        Ensure-Focus $app
        [TvKeys]::Type($c.keys)
        Check $app $c.name $c.want
    }

    # Ctrl+Shift: tat tieng Viet roi bat lai.
    [TvKeys]::CtrlShiftTap()
    [TvKeys]::Type('as ')
    Check $app 'Ctrl+Shift -> EN' 'as '
    [TvKeys]::CtrlShiftTap()
    [TvKeys]::Type('as ')
    Check $app 'Ctrl+Shift -> VN' (U '\u00e1 ')

    # Caps Lock: phim dau viet hoa van la phim dau.
    [TvKeys]::Tap(0x14)
    [TvKeys]::Type('VIEETJ ', $true)
    [TvKeys]::Tap(0x14)
    Check $app 'Caps Lock VIEETJ' (U 'VI\u1ec6T ')

    if ($script:fail -gt 0) {
        $npPid = [uint32]0
        $npTid = [TvKeys]::GetWindowThreadProcessId($app.Main, [ref]$npPid)
        Write-Host ('{0} HKL = 0x{1:X}' -f $app.Name, [TvKeys]::GetKeyboardLayout($npTid).ToInt64())
    }
    Stop-Process -Id $app.Proc.Id -Force -ErrorAction SilentlyContinue
    $script:ownedEditor = $null
}

if ($script:fail -gt 0) {
    Write-Host '--- diagnostics'
    & $cli doctor 2>&1 | Out-Host
    if (Test-Path $trace) {
        Write-Host '--- TSF key trace (TEXTVN_TSF_TRACE, chi phim dieu khien)'
        Get-Content $trace | Where-Object { $_ -notmatch ' chr ' } | Select-Object -Last 150 | Out-Host
    }
    Write-Host '--- summary'
    $script:results | Out-Host
    throw "$($script:fail) typing case(s) failed"
}
Write-Host 'typing: OK'
} finally {
    if ($script:ownedEditor) {
        Stop-Process -Id $script:ownedEditor.Id -Force -ErrorAction SilentlyContinue
    }
    & $tray --stop | Out-Null
    if ($null -ne $stateBackup) {
        # Tray ghi state.json khi doi V/E: chi tra lai sau khi no da thoat.
        for ($i = 0; $i -lt 20 -and (Get-Process -Name TextVN -ErrorAction SilentlyContinue); $i++) {
            Start-Sleep -Milliseconds 500
        }
        [System.IO.File]::WriteAllText($stateFile, $stateBackup, (New-Object System.Text.UTF8Encoding $false))
    }
    if ($autoMarkerMoved) {
        Move-Item -LiteralPath $autoMarkerAside -Destination $autoMarker -Force -ErrorAction SilentlyContinue
    }
}
