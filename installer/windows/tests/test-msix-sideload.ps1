# SPDX-License-Identifier: GPL-3.0-or-later
# test-msix-sideload.ps1 -Msix <TextVN-*.msix>
#
# Bang chung kenh Store (MSIX) chay THAT tren Windows (R2-02/R2-04): ky goi bang
# cert tu tao (Subject = Publisher trong manifest), cai bang Add-AppxPackage, mo
# app qua shell:AppsFolder, roi kiem tra TU SHELL KHONG CO PACKAGE IDENTITY (mot
# app khac nhin thay gi):
#   1. %LOCALAPPDATA%\Programs\TextVN-Store\stage.json + thu muc phien ban co TextVN.exe
#      (ghi that, khong bi ao hoa vao hive/thu muc rieng cua goi);
#   2. HKCU CLSID TIP -> InprocServer32 tro vao thu muc do (Notepad/Explorer thay TIP);
#   3. tien trinh TextVN chay tu thu muc do (da thoat khoi container);
#   4. HKCU Run 'TextVN' tro vao thu muc do (guard chay moi lan dang nhap);
#   5. Remove-AppxPackage roi chay '<dir>\TextVN.exe --msix-guard' (nhu luc dang nhap):
#      dang ky TIP va Run value bi don sach.
# Can quyen admin (cai cert vao LocalMachine\TrustedPeople). ASCII-only (G7).
param(
    [Parameter(Mandatory = $true)][string]$Msix
)
$ErrorActionPreference = 'Stop'
$clsid = '{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}'
$inproc = "HKCU:\Software\Classes\CLSID\$clsid\InprocServer32"
$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$storeRoot = Join-Path $env:LOCALAPPDATA 'Programs\TextVN-Store'
$fail = 0
function Pass([string]$m) { Write-Host "PASS $m" }
function Fail([string]$m) { Write-Host "FAIL $m"; $script:fail++ }
function WaitFor([scriptblock]$cond, [int]$seconds) {
    for ($i = 0; $i -lt ($seconds * 2); $i++) {
        if (& $cond) { return $true }
        Start-Sleep -Milliseconds 500
    }
    return [bool](& $cond)
}

$Msix = (Resolve-Path -LiteralPath $Msix).Path
# Runner CI: cac kich ban truoc (portable/Inno) da go, nhung Run 'TextVN' cua kenh
# khac con sot lai thi kenh Store co y khong ghi de (khong cuop muc cua ban khac).
$prev = (Get-ItemProperty -Path $runKey -Name 'TextVN' -ErrorAction SilentlyContinue).TextVN
if ($prev -and -not $prev.Contains('TextVN-Store')) {
    Write-Host "Don Run 'TextVN' cua kenh khac con sot: $prev"
    Remove-ItemProperty -Path $runKey -Name 'TextVN' -ErrorAction SilentlyContinue
}
$work = Join-Path $env:RUNNER_TEMP ('msix-sideload-' + [guid]::NewGuid().ToString('N'))
if (-not $env:RUNNER_TEMP) { $work = Join-Path $env:TEMP ('msix-sideload-' + [guid]::NewGuid().ToString('N')) }
New-Item -ItemType Directory -Path $work | Out-Null
$pkg = Join-Path $work 'TextVN-test.msix'
Copy-Item -LiteralPath $Msix -Destination $pkg

# Manifest: Publisher + Identity/Name.
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::OpenRead($pkg)
try {
    $entry = $zip.GetEntry('AppxManifest.xml')
    $reader = New-Object System.IO.StreamReader($entry.Open(), [System.Text.Encoding]::UTF8)
    [xml]$manifest = $reader.ReadToEnd()
    $reader.Close()
} finally { $zip.Dispose() }
$publisher = $manifest.Package.Identity.Publisher
$identityName = $manifest.Package.Identity.Name
Write-Host "Identity: $identityName / $publisher / $($manifest.Package.Identity.Version)"

# Cert tu tao: Subject PHAI trung Publisher; EKU Code Signing; tin cay qua TrustedPeople.
$cert = New-SelfSignedCertificate -Type Custom -Subject $publisher -KeyUsage DigitalSignature `
    -FriendlyName 'TextVN MSIX sideload test' -CertStoreLocation 'Cert:\CurrentUser\My' `
    -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')
$cer = Join-Path $work 'test.cer'
Export-Certificate -Cert $cert -FilePath $cer | Out-Null
Import-Certificate -FilePath $cer -CertStoreLocation 'Cert:\LocalMachine\TrustedPeople' | Out-Null

$signtool = (Get-Command signtool.exe -ErrorAction SilentlyContinue | Select-Object -First 1).Source
if (-not $signtool) {
    $signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
        Sort-Object FullName -Descending | Select-Object -First 1 -ExpandProperty FullName
}
if (-not $signtool) { throw 'signtool.exe khong tim thay (can Windows SDK)' }
& $signtool sign /fd SHA256 /sha1 $cert.Thumbprint $pkg
if ($LASTEXITCODE -ne 0) { throw "signtool sign failed ($LASTEXITCODE)" }

$pfn = $null
$dir = $null
try {
    Add-AppxPackage -Path $pkg
    $installed = Get-AppxPackage -Name $identityName
    if (-not $installed) { throw "Add-AppxPackage xong nhung Get-AppxPackage khong thay $identityName" }
    $pfn = $installed.PackageFamilyName
    Pass "Add-AppxPackage: $($installed.PackageFullName)"

    # Mo app dung nhu nguoi dung bam Start menu.
    Start-Process -FilePath 'explorer.exe' -ArgumentList "shell:AppsFolder\$pfn!TextVN"

    $stageJson = Join-Path $storeRoot 'stage.json'
    if (WaitFor { Test-Path -LiteralPath $stageJson } 90) {
        $stage = Get-Content -LiteralPath $stageJson -Raw | ConvertFrom-Json
        $dir = $stage.dir
        if ($stage.pfn -ne $pfn) { Fail "stage.json pfn '$($stage.pfn)' != '$pfn'" }
        if ($dir -and (Test-Path -LiteralPath (Join-Path $dir 'TextVN.exe'))) {
            Pass "stage.json + $dir\TextVN.exe nhin thay tu ngoai goi (khong bi ao hoa)"
        } else {
            Fail "thu muc phien ban thieu TextVN.exe: $dir"
        }
    } else {
        Fail "khong thay $stageJson sau 90 s (ghi bi ao hoa hoac relay/install khong chay)"
    }

    if ($dir) {
        $regOk = WaitFor {
            $v = (Get-ItemProperty -Path $inproc -ErrorAction SilentlyContinue).'(default)'
            $v -and $v.StartsWith($dir, [System.StringComparison]::OrdinalIgnoreCase)
        } 60
        if ($regOk) { Pass 'HKCU TIP InprocServer32 tro vao ban Store da stage' }
        else { Fail "HKCU TIP chua dang ky vao $dir (gia tri: $((Get-ItemProperty -Path $inproc -ErrorAction SilentlyContinue).'(default)'))" }

        $procOk = WaitFor {
            [bool](Get-Process -Name TextVN -ErrorAction SilentlyContinue |
                Where-Object { $_.Path -and $_.Path.StartsWith($dir, [System.StringComparison]::OrdinalIgnoreCase) })
        } 30
        if ($procOk) { Pass 'tray TextVN chay tu thu muc da stage (ngoai container)' }
        else { Fail 'khong thay tien trinh TextVN chay tu thu muc da stage' }

        $run = (Get-ItemProperty -Path $runKey -Name 'TextVN' -ErrorAction SilentlyContinue).TextVN
        if ($run -and $run.Contains($dir)) { Pass "HKCU Run TextVN = $run" }
        else { Fail "HKCU Run TextVN khong tro vao ban Store: '$run'" }
    }

    # Go goi nhu nguoi dung, roi chay guard nhu luc dang nhap.
    if ($dir) { & (Join-Path $dir 'TextVN.exe') --stop *> $null }
    Start-Sleep -Seconds 2
    Remove-AppxPackage -Package $installed.PackageFullName
    $installed = $null
    Pass 'Remove-AppxPackage'
    if ($dir) {
        $guard = Start-Process -FilePath (Join-Path $dir 'TextVN.exe') -ArgumentList '--msix-guard' -PassThru
        if (-not $guard.WaitForExit(90000)) { Fail '--msix-guard khong ket thuc trong 90 s' }
        if (Test-Path $inproc) { Fail 'guard sau khi go goi: dang ky TIP HKCU van con' }
        else { Pass 'guard sau khi go goi: dang ky TIP HKCU da xoa' }
        $run = (Get-ItemProperty -Path $runKey -Name 'TextVN' -ErrorAction SilentlyContinue).TextVN
        if ($run -and $run.Contains($dir)) { Fail "guard sau khi go goi: Run TextVN van con '$run'" }
        else { Pass 'guard sau khi go goi: Run TextVN da xoa' }
        # R2-96: cai lai truoc lan dang nhap ke phai la cai MOI (tu khoi dong bat lai).
        if (Test-Path -LiteralPath $stageJson) { Fail 'guard sau khi go goi: stage.json van con (cai lai se tat tu khoi dong)' }
        else { Pass 'guard sau khi go goi: stage.json da xoa' }
    }
} finally {
    if ($dir -and (Test-Path -LiteralPath (Join-Path $dir 'TextVN.exe'))) {
        & (Join-Path $dir 'TextVN.exe') --stop *> $null
    }
    if ($installed) { Remove-AppxPackage -Package $installed.PackageFullName -ErrorAction SilentlyContinue }
    Get-ChildItem 'Cert:\LocalMachine\TrustedPeople' | Where-Object { $_.Thumbprint -eq $cert.Thumbprint } |
        Remove-Item -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath ("Cert:\CurrentUser\My\" + $cert.Thumbprint) -ErrorAction SilentlyContinue
}

if ($fail -gt 0) {
    Write-Host "MSIX sideload: $fail kiem tra FAIL"
    exit 1
}
Write-Host 'MSIX sideload: PASS toan bo (kenh Store chay that ngoai container, go goi don sach)'
