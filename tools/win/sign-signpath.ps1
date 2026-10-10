# SPDX-License-Identifier: GPL-3.0-or-later
# sign-signpath.ps1 -File <path>[,<path>...] -Description <text>
#
# Ky Authenticode qua SignPath REST API (goi Open Source cua SignPath Foundation).
# Bat khi cac bien moi truong sau duoc dat (khong dat SIGNPATH_API_TOKEN = khong lam
# gi, exit 0):
#   SIGNPATH_API_TOKEN                - API token (CI user) tu signpath.io
#   SIGNPATH_ORGANIZATION_ID          - Organization ID (GUID)
#   SIGNPATH_PROJECT_SLUG             - project slug (ten cu SIGNPATH_PROJECT_KEY van nhan)
#   SIGNPATH_POLICY                   - signing policy slug (vd. release-signing)
#   SIGNPATH_ARTIFACT_CONFIGURATION   - (tuy chon) artifact configuration slug
#
# R2-72: ban truoc goi route/ten truong khong ton tai (`signing-requests?project-key=`,
# body octet-stream, `/signed-artifact`) va coi moi trang thai khac InProgress la loi
# (ke ca WaitingForApproval cua goi OSS - can nguoi duyet). Gio dung dung API cong bo
# (https://about.signpath.io/documentation/build-system-integration#rest-api):
#   POST {base}/SigningRequests  multipart: ProjectSlug, SigningPolicySlug,
#        ArtifactConfigurationSlug, Description, Artifact(file)   -> 201 + Location
#   GET  {Location}               -> { status: InProgress|WaitingForApproval|...|Completed }
#   GET  {Location}/SignedArtifact -> file da ky
# Nhieu file -> dong goi 1 ZIP (artifact configuration cua SignPath phai la ZIP
# "deep sign" *.exe/*.dll) => MOT lan duyet cho ca bo binary, roi giai nen ghi de.
param(
    [Parameter(Mandatory = $true)][string[]]$File,
    [string]$Description = "TextVN release binaries",
    [int]$TimeoutMinutes = 60
)
$ErrorActionPreference = 'Stop'

if (-not $env:SIGNPATH_API_TOKEN) { Write-Host 'SKIP SignPath (SIGNPATH_API_TOKEN not set)'; exit 0 }
$project = $env:SIGNPATH_PROJECT_SLUG
if (-not $project) { $project = $env:SIGNPATH_PROJECT_KEY }
foreach ($pair in @(@('SIGNPATH_ORGANIZATION_ID', $env:SIGNPATH_ORGANIZATION_ID), @('SIGNPATH_PROJECT_SLUG', $project), @('SIGNPATH_POLICY', $env:SIGNPATH_POLICY))) {
    if (-not $pair[1]) { Write-Error "Missing environment variable $($pair[0]) for SignPath signing"; exit 1 }
}
$files = @()
foreach ($f in $File) {
    foreach ($part in ($f -split ',')) {
        $p = $part.Trim()
        if (-not $p) { continue }
        if (-not (Test-Path -LiteralPath $p -PathType Leaf)) { Write-Error "file not found: $p"; exit 1 }
        $files += (Resolve-Path -LiteralPath $p).Path
    }
}
if ($files.Count -eq 0) { Write-Error 'no file to sign'; exit 1 }

Add-Type -AssemblyName System.Net.Http
Add-Type -AssemblyName System.IO.Compression.FileSystem
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$work = Join-Path ([System.IO.Path]::GetTempPath()) ('signpath-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work | Out-Null
try {
    # 1 file -> gui nguyen file; nhieu file -> ZIP (ten file trong ZIP la ten goc, phai duy nhat).
    if ($files.Count -eq 1) {
        $upload = $files[0]
    } else {
        $names = $files | ForEach-Object { [System.IO.Path]::GetFileName($_) }
        if (($names | Select-Object -Unique).Count -ne $names.Count) { Write-Error 'duplicate file names in one signing request'; exit 1 }
        $upload = Join-Path $work 'textvn-unsigned.zip'
        $zip = [System.IO.Compression.ZipFile]::Open($upload, 'Create')
        try {
            foreach ($f in $files) {
                [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $f, [System.IO.Path]::GetFileName($f)) | Out-Null
            }
        } finally { $zip.Dispose() }
    }

    $base = "https://app.signpath.io/API/v1/$($env:SIGNPATH_ORGANIZATION_ID)"
    $client = New-Object System.Net.Http.HttpClient
    $client.Timeout = [TimeSpan]::FromMinutes(10)
    $client.DefaultRequestHeaders.Authorization = New-Object System.Net.Http.Headers.AuthenticationHeaderValue('Bearer', $env:SIGNPATH_API_TOKEN)

    $form = New-Object System.Net.Http.MultipartFormDataContent
    $form.Add((New-Object System.Net.Http.StringContent($project)), 'ProjectSlug')
    $form.Add((New-Object System.Net.Http.StringContent($env:SIGNPATH_POLICY)), 'SigningPolicySlug')
    if ($env:SIGNPATH_ARTIFACT_CONFIGURATION) {
        $form.Add((New-Object System.Net.Http.StringContent($env:SIGNPATH_ARTIFACT_CONFIGURATION)), 'ArtifactConfigurationSlug')
    }
    $form.Add((New-Object System.Net.Http.StringContent($Description)), 'Description')
    $stream = [System.IO.File]::OpenRead($upload)
    $fileContent = New-Object System.Net.Http.StreamContent($stream)
    $fileContent.Headers.ContentType = New-Object System.Net.Http.Headers.MediaTypeHeaderValue('application/octet-stream')
    $form.Add($fileContent, 'Artifact', [System.IO.Path]::GetFileName($upload))

    Write-Host "SignPath: submitting $($files.Count) file(s) as $([System.IO.Path]::GetFileName($upload))"
    $resp = $client.PostAsync("$base/SigningRequests", $form).GetAwaiter().GetResult()
    $stream.Dispose()
    if ([int]$resp.StatusCode -ne 201) {
        $body = $resp.Content.ReadAsStringAsync().GetAwaiter().GetResult()
        Write-Error "SignPath: unexpected status $([int]$resp.StatusCode): $body"
        exit 1
    }
    $requestUrl = $resp.Headers.Location
    if (-not $requestUrl) { Write-Error 'SignPath: no Location header in signing request response'; exit 1 }
    if (-not $requestUrl.IsAbsoluteUri) { $requestUrl = New-Object System.Uri((New-Object System.Uri($base + '/')), $requestUrl) }
    Write-Host "SignPath: request $requestUrl"

    # Poll: WaitingForApproval/QueuedForProcessing/Processing/InProgress = cho tiep;
    # Completed = xong; Failed/Denied/Canceled = loi. Goi OSS co the can nguoi duyet.
    $deadline = (Get-Date).AddMinutes($TimeoutMinutes)
    $status = ''
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Seconds 10
        $json = $client.GetStringAsync($requestUrl).GetAwaiter().GetResult() | ConvertFrom-Json
        $status = [string]$json.status
        if ($status -eq 'Completed') { break }
        if (@('Failed', 'Denied', 'Canceled') -contains $status) {
            Write-Error "SignPath: signing request ended with status $status"
            exit 1
        }
        Write-Host "SignPath: waiting ($status)..."
    }
    if ($status -ne 'Completed') { Write-Error "SignPath: timed out after $TimeoutMinutes min (last status: $status)"; exit 1 }

    $signed = Join-Path $work ('signed-' + [System.IO.Path]::GetFileName($upload))
    $bytes = $client.GetByteArrayAsync("$requestUrl/SignedArtifact").GetAwaiter().GetResult()
    [System.IO.File]::WriteAllBytes($signed, $bytes)
    if ($files.Count -eq 1) {
        Copy-Item -LiteralPath $signed -Destination $files[0] -Force
    } else {
        $out = Join-Path $work 'signed'
        [System.IO.Compression.ZipFile]::ExtractToDirectory($signed, $out)
        foreach ($f in $files) {
            $src = Join-Path $out ([System.IO.Path]::GetFileName($f))
            if (-not (Test-Path -LiteralPath $src)) { Write-Error "SignPath: signed artifact is missing $([System.IO.Path]::GetFileName($f))"; exit 1 }
            Copy-Item -LiteralPath $src -Destination $f -Force
        }
    }
    # Chot: moi file phai co chu ky Authenticode hop le sau khi ghi de.
    foreach ($f in $files) {
        $sig = Get-AuthenticodeSignature -FilePath $f
        if ($sig.Status -ne 'Valid') { Write-Error "SignPath: $f signature status $($sig.Status)"; exit 1 }
        Write-Host "OK: SignPath signed $([System.IO.Path]::GetFileName($f)) ($($sig.SignerCertificate.Subject))"
    }
} finally {
    Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
}
