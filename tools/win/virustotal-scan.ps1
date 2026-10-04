# SPDX-License-Identifier: GPL-3.0-or-later
# virustotal-scan.ps1 - Quet file qua VirusTotal API v3 de phat hien AV bao nham.
#
# Cach dung:
#   # Lookup by hash (nhanh, khong tinh vao upload quota):
#   .\virustotal-scan.ps1 -Files "dist\TextVN.exe","dist\textvn-cli.exe"
#
#   # Upload neu chua co tren VT (ton upload quota 4/min):
#   .\virustotal-scan.ps1 -Files "dist\TextVN.exe" -Upload
#
#   # Chi bao cao, khong fail (mac dinh: fail neu detections >= -MaxDetections):
#   .\virustotal-scan.ps1 -Files "dist\*.exe" -MaxDetections 5
#
# API key: lay tu env VIRUSTOTAL_API_KEY hoac GitHub secret cung ten.
# Rate limit free tier: 4 lookups/min (15s giua cac request), 500/day.
#
# Ket qua: moi file in so AV detect / tong so AV. Neu detections >=
# MaxDetections thi thoat voi exit 1 (de release.yml chay lai buoc nay).
#
# ASCII-only (G7/A5).
param(
    [Parameter(Mandatory = $true)][string[]]$Files,
    [switch]$Upload,
    [int]$MaxDetections = 3,
    [int]$DelayMs = 15500
)
$ErrorActionPreference = 'Stop'

$apiKey = $env:VIRUSTOTAL_API_KEY
if (-not $apiKey) { throw 'VIRUSTOTAL_API_KEY env var not set' }

$headers = @{ 'x-apikey' = $apiKey }
$baseUrl = 'https://www.virustotal.com/api/v3'

function Get-FileSha256([string]$path) {
    (Get-FileHash $path -Algorithm SHA256).Hash.ToLower()
}

function Lookup-VtHash([string]$sha256) {
    try {
        $r = Invoke-RestMethod -Uri "$baseUrl/files/$sha256" -Headers $headers -Method Get
        return $r.data
    } catch {
        if ($_.Exception.Response.StatusCode -eq 404) { return $null }
        throw
    }
}

function Upload-VtFile([string]$path) {
    $uri = "$baseUrl/files"
    $boundary = [guid]::NewGuid().ToString()
    $fileName = [System.IO.Path]::GetFileName($path)
    $fileBytes = [System.IO.File]::ReadAllBytes($path)
    $crlf = "`r`n"
    $bodyStart = "--$boundary$crlf" +
        "Content-Disposition: form-data; name=`"file`"; filename=`"$fileName`"$crlf" +
        "Content-Type: application/octet-stream$crlf$crlf"
    $bodyEnd = "$crlf--$boundary--$crlf"
    $startBytes = [System.Text.Encoding]::UTF8.GetBytes($bodyStart)
    $endBytes = [System.Text.Encoding]::UTF8.GetBytes($bodyEnd)
    $body = New-Object byte[] ($startBytes.Length + $fileBytes.Length + $endBytes.Length)
    [Array]::Copy($startBytes, 0, $body, 0, $startBytes.Length)
    [Array]::Copy($fileBytes, 0, $body, $startBytes.Length, $fileBytes.Length)
    [Array]::Copy($endBytes, 0, $body, $startBytes.Length + $fileBytes.Length, $endBytes.Length)
    $r = Invoke-RestMethod -Uri $uri -Method Post -Headers $headers `
        -ContentType "multipart/form-data; boundary=$boundary" -Body $body
    return $r.data
}

function Get-VtAnalysis([string]$analysisId) {
    try {
        $r = Invoke-RestMethod -Uri "$baseUrl/analyses/$analysisId" -Headers $headers -Method Get
        return $r.data
    } catch { return $null }
}

# Expand wildcards
$allFiles = @()
foreach ($f in $Files) {
    if ($f.Contains('*')) {
        $allFiles += Get-ChildItem $f -File | Select-Object -ExpandProperty FullName
    } elseif (Test-Path $f) {
        $allFiles += (Resolve-Path $f).Path
    } else {
        Write-Host "WARN: file khong ton tai: $f"
    }
}
if ($allFiles.Count -eq 0) { throw 'Khong co file nao de quet' }

Write-Host "==== VIRUSTOTAL SCAN ===="
Write-Host "Files: $($allFiles.Count) | MaxDetections: $MaxDetections | Upload: $Upload"
Write-Host ""

$results = @()
$failCount = 0
$needDelay = $false

foreach ($file in $allFiles) {
    $fileName = [System.IO.Path]::GetFileName($file)
    $sha256 = Get-FileSha256 $file

    if ($needDelay) {
        Write-Host "  (rate limit: cho 15s...)"
        Start-Sleep -Milliseconds $DelayMs
    }

    Write-Host "[$fileName] SHA256: $sha256"

    # Lookup by hash (khong ton upload quota)
    $vtFile = Lookup-VtHash $sha256
    $needDelay = $true  # moi API call deu tinh vao rate limit

    if (-not $vtFile) {
        if (-not $Upload) {
            Write-Host "  Chua co tren VT (dung -Upload de quet)"
            $results += [pscustomobject]@{ File = $fileName; SHA256 = $sha256; Detections = -1; Total = 0; Status = 'NOT_ON_VT' }
            continue
        }
        Write-Host "  Chua co tren VT - uploading..."
        Start-Sleep -Milliseconds $DelayMs
        $uploadResult = Upload-VtFile $file
        $analysisId = $uploadResult.id
        Write-Host "  Analysis ID: $analysisId - cho ket qua..."

        # Cho analysis hoan thanh (toi da 60s, check moi 15s)
        $analysis = $null
        for ($i = 0; $i -lt 4; $i++) {
            Start-Sleep -Milliseconds $DelayMs
            $analysis = Get-VtAnalysis $analysisId
            if ($analysis -and $analysis.attributes.status -eq 'completed') { break }
        }
        if ($analysis -and $analysis.attributes.status -eq 'completed') {
            $stats = $analysis.attributes.stats
            $detections = $stats.malicious + $stats.suspicious
            $total = $stats.harmless + $stats.undetected + $stats.suspicious + $stats.malicious
            Write-Host "  Detections: $detections / $total"
            $status = if ($detections -ge $MaxDetections) { 'FLAGGED' } else { 'CLEAN' }
            $results += [pscustomobject]@{ File = $fileName; SHA256 = $sha256; Detections = $detections; Total = $total; Status = $status }
            if ($detections -ge $MaxDetections) { $failCount++ }
            continue
        }
        Write-Host "  Analysis chua hoan thanh - kiem lai bang lookup..."
        Start-Sleep -Milliseconds $DelayMs
        $vtFile = Lookup-VtHash $sha256
        if (-not $vtFile) {
            Write-Host "  Van chua co ket qua - thu lai lan sau"
            $results += [pscustomobject]@{ File = $fileName; SHA256 = $sha256; Detections = -1; Total = 0; Status = 'PENDING' }
            continue
        }
    }

    # Co ket qua tu lookup
    $stats = $vtFile.attributes.last_analysis_stats
    $detections = $stats.malicious + $stats.suspicious
    $total = $stats.harmless + $stats.undetected + $stats.suspicious + $stats.malicious
    $status = if ($detections -ge $MaxDetections) { 'FLAGGED' } else { 'CLEAN' }
    Write-Host "  Detections: $detections / $total $(if ($detections -gt 0) { '(AV bao nham co the)' })"
    $results += [pscustomobject]@{ File = $fileName; SHA256 = $sha256; Detections = $detections; Total = $total; Status = $status }
    if ($detections -ge $MaxDetections) { $failCount++ }
}

Write-Host ""
Write-Host "==== KET QUA VIRUSTOTAL ===="
$results | Format-Table File, Detections, Total, Status -AutoSize

$flagged = @($results | Where-Object { $_.Status -eq 'FLAGGED' })
if ($flagged.Count -gt 0) {
    Write-Host "FAIL: $($flagged.Count) file co detections >= $MaxDetections"
    exit 1
}
$pending = @($results | Where-Object { $_.Status -in @('PENDING', 'NOT_ON_VT') })
if ($pending.Count -eq $results.Count) {
    Write-Host "NOTE: Khong file nao co ket qua VT (dung -Upload de quet) - khong fail"
} else {
    Write-Host "OK - khong co file nao bi bao nham vuot nguong"
}
