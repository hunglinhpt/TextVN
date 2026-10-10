# SPDX-License-Identifier: GPL-3.0-or-later
# uninstall.ps1 - tat TextVN va go dang ky bo go (TSF) truoc khi xoa thu muc portable.
# Cau hinh nguoi dung trong %APPDATA%\TextVN duoc giu lai.
$dir = Split-Path -Parent $MyInvocation.MyCommand.Path
# Start-Process noi -ArgumentList bang dau cach, khong tu trich dan: path co dau cach
# phai boc "...", va '\' cuoi (vd. D:\) nhan doi de khong nuot dau " dong.
function Format-PathArg([string]$p) { '"' + ($p -replace '(\\+)$', '$1$1') + '"' }
$dirArg = Format-PathArg $dir
Write-Host 'Tat TextVN...'
# R2-93: chi dung tray chay tu CHINH thu muc nay - ban TextVN cai o noi khac (bo cai,
# Store) dang dung thi giu nguyen.
Start-Process -Wait -WindowStyle Hidden -FilePath (Join-Path $dir 'TextVN.exe') -ArgumentList @('--stop', '--if-image-under', $dirArg) -ErrorAction SilentlyContinue
$ownTray = {
    Get-Process -Name TextVN -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and $_.Path.StartsWith($dir.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase) }
}
for ($i = 0; $i -lt 20 -and (& $ownTray); $i++) { Start-Sleep -Milliseconds 250 }
# Tu khoi dong tro vao thu muc nay thi bo (khong dung toi ban TextVN da cai o noi khac).
$run = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$cmd = (Get-ItemProperty -Path $run -Name 'TextVN' -ErrorAction SilentlyContinue).TextVN
if ($cmd -and $cmd.Trim().TrimStart('"').StartsWith($dir.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) {
    Remove-ItemProperty -Path $run -Name 'TextVN' -ErrorAction SilentlyContinue
}
Write-Host 'Go dang ky TSF...'

# BUG-07 (audit 2026-10-04): Tra lai Ctrl + Shift cho Windows. Marker
# %APPDATA%\TextVN\ctrl_shift_default_applied (tray/src/lib.rs) ghi gia tri TRUOC KHI
# TextVN "danh Ctrl + Shift": dong dau 'freed', moi dong sau '<ten>=<gia tri cu>' (rong =
# truoc do khong co). Ban <= 0.2.27 ghi '1' = chi Layout Hotkey. 'declined'/khong co
# marker = TextVN khong doi gi -> giu nguyen (CR-35). Chi tra muc VAN la '3' (khong
# gan) - nguoi dung tu doi sau do thi giu lua chon cua ho (R2-40, ca Language Hotkey).
$toggle = 'HKCU:\Keyboard Layout\Toggle'
$marker = Join-Path $env:APPDATA 'TextVN\ctrl_shift_default_applied'
$record = @()
if (Test-Path -LiteralPath $marker -PathType Leaf) {
    $lines = @(Get-Content -LiteralPath $marker -ErrorAction SilentlyContinue | ForEach-Object { $_.Trim() })
    if ($lines.Count -gt 0 -and $lines[0] -eq '1') {
        $record = @(, @('Layout Hotkey', ''))
    } elseif ($lines.Count -gt 0 -and $lines[0] -eq 'freed') {
        foreach ($line in ($lines | Select-Object -Skip 1)) {
            $eq = $line.IndexOf('=')
            if ($eq -lt 1) { continue }
            $name = $line.Substring(0, $eq).Trim()
            $prev = $line.Substring($eq + 1).Trim()
            if (@('Layout Hotkey', 'Language Hotkey', 'Hotkey') -notcontains $name) { continue }
            if ($prev -notmatch '^[1-4]?$') { continue }
            $record += , @($name, $prev)
        }
    }
}
foreach ($entry in $record) {
    $name = $entry[0]
    $prev = $entry[1]
    $cur = (Get-ItemProperty -Path $toggle -Name $name -ErrorAction SilentlyContinue).$name
    if ($cur -ne '3') { continue }
    if ($prev) {
        Set-ItemProperty -Path $toggle -Name $name -Value $prev
    } else {
        Remove-ItemProperty -Path $toggle -Name $name -ErrorAction SilentlyContinue
    }
    Write-Host "Da tra lai Ctrl + Shift cho Windows ($name)."
}
Remove-Item -LiteralPath $marker -Force -ErrorAction SilentlyContinue

# R2-93: chi go dang ky TSF khi no thuoc thu muc nay (hoac mo coi) - xem
# `textvn-cli unregister --if-owned-by`; dang ky cua ban cai khac duoc giu.
$r = Start-Process -Wait -PassThru -WindowStyle Hidden -FilePath (Join-Path $dir 'textvn-cli.exe') -ArgumentList @('unregister', '--if-owned-by', $dirArg)
if ($r.ExitCode -ne 0) {
    Write-Error ('Huy dang ky TSF that bai, ma loi ' + $r.ExitCode + '. Khong xoa thu muc portable cho den khi "textvn-cli.exe doctor" bao ro nguyen nhan.')
    exit $r.ExitCode
}
# R2-16 (P0): ZIP portable khong co thu muc goc - giai nen "Extract here" vao
# Downloads/Desktop thi $dir CHINH LA thu muc do. Truoc day `Remove-Item -Recurse`
# xoa sach ca thu muc cua nguoi dung. Gio chi xoa DUNG cac file TextVN biet ten,
# roi xoa thu muc con/thu muc goc khi (va chi khi) chung da rong.
$known = @(
    'TextVN.exe', 'textvn-cli.exe', 'textvn-tsf.dll', 'textvn-tsf-x86.dll', 'textvn-hook.exe',
    'install.ps1', 'HUONG_DAN_SU_DUNG.txt', 'README.md', 'CHANGELOG.md', 'LICENSE',
    'RELEASE_REPORT.json', 'resources\textvn.ico', 'resources\textvn_v.ico', 'resources\textvn_e.ico'
)
$locked = @()
foreach ($rel in $known) {
    $p = Join-Path $dir $rel
    if (Test-Path -LiteralPath $p -PathType Leaf) {
        try { Remove-Item -LiteralPath $p -Force -ErrorAction Stop } catch { $locked += $p }
    }
}
# Ban sao luu khi nang cap (TextVN.exe.old-*, textvn-tsf*.dll.old-*).
Get-ChildItem -LiteralPath $dir -File -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match '^(TextVN\.exe|textvn-tsf(-x86)?\.dll)\.old-' } |
    ForEach-Object {
        try { Remove-Item -LiteralPath $_.FullName -Force -ErrorAction Stop } catch { $locked += $_.FullName }
    }
# Thu muc chi xoa khi da rong - KHONG bao gio -Recurse.
$res = Join-Path $dir 'resources'
if ((Test-Path -LiteralPath $res -PathType Container) -and -not (Get-ChildItem -LiteralPath $res -Force)) {
    Remove-Item -LiteralPath $res -Force -ErrorAction SilentlyContinue
}
# uninstall.ps1 (chinh file nay) xoa sau cung; PowerShell da nap script nen xoa duoc.
Remove-Item -LiteralPath (Join-Path $dir 'uninstall.ps1') -Force -ErrorAction SilentlyContinue
$dirEmpty = -not (Get-ChildItem -LiteralPath $dir -Force -ErrorAction SilentlyContinue)
if ($dirEmpty) { Remove-Item -LiteralPath $dir -Force -ErrorAction SilentlyContinue }

if ($locked.Count -gt 0) {
    # DLL con bi nap trong app dang mo: hen xoa DUNG cac file do o lan dang nhap ke
    # tiep (RunOnce, khong can admin) - luc do TIP da go dang ky. `rmdir` khong /s:
    # chi xoa thu muc neu da rong.
    # R2-94: MOT muc RunOnce cho MOI file/thu muc - Windows khuyen nghi lenh RunOnce
    # <= 260 ky tu; gop moi file vao mot lenh 'del' vuot nguong khi path dai. Muc thu
    # muc dat ten sau cung (RunOnce chay theo thu tu tao); rmdir khong /s.
    $runOnce = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\RunOnce'
    $cmds = @($locked | ForEach-Object { 'cmd.exe /c del /f /q "' + $_ + '"' })
    if ($dirEmpty -eq $false) { $cmds += @(('cmd.exe /c rmdir "' + $res + '"'), ('cmd.exe /c rmdir "' + $dir + '"')) }
    $scheduled = $true
    for ($n = 0; $n -lt $cmds.Count; $n++) {
        if ($cmds[$n].Length -gt 260) { $scheduled = $false; continue }
        Set-ItemProperty -Path $runOnce -Name ('TextVNCleanup' + ($n + 1)) -Value $cmds[$n] -ErrorAction SilentlyContinue
        if (-not $?) { $scheduled = $false }
    }
    if ($scheduled) {
        Write-Host 'Mot so file con dang duoc Windows dung (bo go da duoc go dang ky).'
        Write-Host 'TextVN se TU DONG xoa cac file do o lan dang nhap ke tiep - khong can lam gi them.'
    } else {
        Write-Host 'Khong xoa het duoc file cua TextVN. Hay dang xuat roi xoa cac file con lai.'
    }
} elseif (Test-Path -LiteralPath $dir) {
    Write-Host 'Da xoa cac file cua TextVN. Thu muc con file khac cua ban nen duoc giu nguyen.'
} else {
    Write-Host 'Da xoa thu muc portable.'
}
Write-Host 'Cau hinh (go tat, tuy chon) van o %APPDATA%\TextVN.'
