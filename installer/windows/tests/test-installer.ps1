# SPDX-License-Identifier: GPL-3.0-or-later
# test-installer.ps1 -Setup <TextVN-setup-*.exe>
#
# Kich ban "cai dat": cai im lang cho nguoi dung hien tai -> kiem tra file, TSF, Run key
# -> go tieng Viet that trong Notepad -> go cai dat im lang -> khong con file/dang ky,
# cau hinh nguoi dung van con.

param(
    [Parameter(Mandatory = $true)][string]$Setup
)
$ErrorActionPreference = 'Stop'
$clsid = '{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}'
$inproc = "HKCU:\Software\Classes\CLSID\$clsid\InprocServer32"
$app = Join-Path $env:LOCALAPPDATA 'Programs\TextVN'
$log = Join-Path $env:TEMP 'textvn-setup.log'

$p = Start-Process -FilePath $Setup -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/CURRENTUSER', "/LOG=$log") -Wait -PassThru
if ($p.ExitCode -ne 0) { Get-Content $log -ErrorAction SilentlyContinue | Select-Object -Last 40; throw "setup exit $($p.ExitCode)" }
foreach ($f in @('TextVN.exe', 'textvn-cli.exe', 'textvn-tsf.dll', 'unins000.exe')) {
    if (-not (Test-Path (Join-Path $app $f))) { throw "installed file missing: $f" }
}
$v = (Get-ItemProperty -Path $inproc -ErrorAction SilentlyContinue).'(default)'
if ($v -ne (Join-Path $app 'textvn-tsf.dll')) { throw "TSF CLSID not registered to installed DLL (got '$v')" }
$run = (Get-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -ErrorAction SilentlyContinue).TextVN
if (-not $run -or $run -notlike '*TextVN.exe*--autostart*') { throw "autostart Run key missing (got '$run')" }
Write-Host 'PASS silent per-user install (files, TSF, autostart)'

& (Join-Path $PSScriptRoot 'test-typing.ps1') -Dir $app

$u = Start-Process -FilePath (Join-Path $app 'unins000.exe') -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') -Wait -PassThru
# Trinh go cai dat chay ban sao trong %TEMP% roi thoat: cho no xoa xong.
for ($i = 0; $i -lt 60 -and (Test-Path (Join-Path $app 'TextVN.exe')); $i++) { Start-Sleep -Milliseconds 500 }
if (Test-Path (Join-Path $app 'TextVN.exe')) { throw 'TextVN.exe still present after uninstall' }
if (Test-Path $inproc) { throw 'TSF CLSID still registered after uninstall' }
$run = (Get-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -ErrorAction SilentlyContinue).TextVN
if ($run) { throw 'autostart Run key left behind' }
if (Test-Path (Join-Path $app 'textvn-tsf.dll')) {
    Write-Host 'NOTE textvn-tsf.dll still loaded by a running app; Windows removes it after sign-out'
}
if (-not (Test-Path (Join-Path $env:APPDATA 'TextVN'))) { throw 'user config was not kept' }
Write-Host 'PASS installer: install -> type -> uninstall'
