# SPDX-License-Identifier: GPL-3.0-or-later
# sign-signpath.ps1 -File <path> -Description <text>
#
# Ky 1 file qua SignPath REST API (goi Open Source cua SignPath Foundation).
# Bat khi cac bien moi truong sau duoc dat (khong dat = khong lam gi, exit 0):
#   SIGNPATH_API_TOKEN        - API token tu signpath.io (Open Source plan)
#   SIGNPATH_ORGANIZATION_ID  - id to chuc tren signpath.io
#   SIGNPATH_PROJECT_KEY      - project key da cau hinh artifact configuration
#   SIGNPATH_POLICY           - signing policy (policy cua Foundation)
#
# Flow chinh tho (docs: https://signpath.io/docs -> "Signing from scripts"):
#   POST /api/v1/{org}/signing-requests  (file)  -> 201 + Location
#   GET  <location>                       poll den status Completed
#   GET  <location>/signed-artifact       -> file da ky (ghi de file goc)
param(
    [Parameter(Mandatory = $true)][string]$File,
    [string]$Description = "TextVN release binary"
)
$ErrorActionPreference = 'Stop'

if (-not $env:SIGNPATH_API_TOKEN) { Write-Host 'SKIP SignPath (SIGNPATH_API_TOKEN not set)'; exit 0 }
foreach ($v in @('SIGNPATH_ORGANIZATION_ID', 'SIGNPATH_PROJECT_KEY', 'SIGNPATH_POLICY')) {
    if (-not (Get-Item "env:$v" -ErrorAction SilentlyContinue)) {
        Write-Error "Missing environment variable $v for SignPath signing"
        exit 1
    }
}
if (-not (Test-Path $File)) { Write-Error "file not found: $File"; exit 1 }

$base = "https://app.signpath.io/api/v1/$($env:SIGNPATH_ORGANIZATION_ID)"
$headers = @{ Authorization = "Bearer $($env:SIGNPATH_API_TOKEN)" }
$description = [Uri]::EscapeDataString($Description)
$project = [Uri]::EscapeDataString($env:SIGNPATH_PROJECT_KEY)
$policy = [Uri]::EscapeDataString($env:SIGNPATH_POLICY)

Write-Host "SignPath: submitting signing request for $(Split-Path -Leaf $File)"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$create = Invoke-WebRequest -UseBasicParsing `
    -Uri "$base/signing-requests?project-key=$project`&signing-policy=$policy`&description=$description" `
    -Method Post -Headers $headers -InFile $File `
    -ContentType 'application/octet-stream'
if ($create.StatusCode -ne 201) { Write-Error "unexpected status $($create.StatusCode)"; exit 1 }
$requestUrl = $create.Headers['Location']
if (-not $requestUrl) { Write-Error 'no Location header in signing request response'; exit 1 }

# Poll den khi signing request khong con InProgress (timeout 10 phut).
$signed = $false
for ($i = 0; $i -lt 60; $i++) {
    Start-Sleep -Seconds 10
    $status = Invoke-RestMethod -UseBasicParsing -Uri $requestUrl -Headers $headers
    if ($status.Status -eq 'Completed') { $signed = $true; break }
    if ($status.Status -ne 'InProgress') {
        Write-Error ("signing request ended with status " + $status.Status)
        exit 1
    }
    Write-Host "SignPath: waiting ($($status.Status))..."
}
if (-not $signed) { Write-Error 'signing request timed out'; exit 1 }

$artifact = "$requestUrl/signed-artifact"
Invoke-WebRequest -UseBasicParsing -Uri $artifact -Headers $headers -OutFile $File
Write-Host "OK: SignPath signed $(Split-Path -Leaf $File)"
