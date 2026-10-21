# SPDX-License-Identifier: GPL-3.0-or-later
<#
.SYNOPSIS
  Sinh corpus CAU SACH tu van ban tieng Viet trong docs/ cua repo (P4-0 D19).

.DESCRIPTION
  Van ban that day chinh la tieng Viet "that" (tai docs/ cua chinh repo):
  - khong can nguon ben ngoai -> khong rang buoc giay phep
  - trai voi cau mau to hop (Fassist-80) -> do FPR tren van ban that
  - tach lap duoc (script nay) -> ta lap lai duoc

  Quy tac loc (de corpus la "van ban thoai"):
  - bo code fence (```), dong co bang (|), dong chua backtick
  - chi lay cau ket thuc bang . ! ? va it nhat 6 tu, toi da 40 tu
  - bo cau chi co ky tu lat latin (khong phai tieng Viet)

.EXAMPLE
  powershell -NoProfile -ExecutionPolicy Bypass -File tools/assist/gen_clean_corpus.ps1
#>
[CmdletBinding()]
param(
    [string]$OutFile = "corpus/assist/vi_docs_clean.ass",
    [int]$MinWords = 6,
    [int]$MaxWords = 40
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSCommandPath))
# Backtick la escape char cua PowerShell -> dung [char]96 de so sanh chuoi.
$bt = [char]96
$fence = [string]::Concat($bt, $bt, $bt)
$codeSpan = [string]::Concat("[", $bt, "[^", $bt, "]*", $bt)

function Test-VietnameseHeavy {
    param([string]$s)
    $letters = @($s.ToCharArray() | Where-Object { [char]::IsLetter($_) })
    if ($letters.Count -eq 0) { return $false }
    $marked = @($letters | Where-Object {
        $_ -match "[\u00C0-\u1EF9]" -and $_ -notmatch "[a-zA-Z]"
    }).Count
    return ($marked -gt 0) -and (($marked * 100) -ge ($letters.Count * 35))
}

$sentences = New-Object System.Collections.Generic.List[string]
$seen = New-Object System.Collections.Generic.HashSet[string]

$files = @()
$files += Get-ChildItem -Path (Join-Path $root "docs") -Recurse -File -Filter *.md
foreach ($extra in @("README.md", "PLAN.md", "CHANGELOG.md")) {
    $p = Join-Path $root $extra
    if (Test-Path $p) { $files += Get-Item $p }
}

foreach ($f in $files) {
    $inFence = $false
    foreach ($line in (Get-Content $f.FullName -Encoding UTF8)) {
        $t = $line.Trim()
        if ($t.StartsWith($fence)) { $inFence = -not $inFence; continue }
        if ($inFence) { continue }
        if ($t -eq "" -or $t.StartsWith("|") -or $t.StartsWith("#")) { continue }
        # tach cau theo dau ket thuc
        $parts = [regex]::Split($t, "(?<=[.!?])\s+")
        foreach ($p in $parts) {
            $s = $p.Trim()
            # bo code span inline (de giu cau van ban thoai)
            $s = [regex]::Replace($s, $codeSpan, "")
            if ($s.Length -lt 20) { continue }
            if (-not (Test-VietnameseHeavy $s)) { continue }
            $words = @($s -split "\s+" | Where-Object { $_ -ne "" })
            if ($words.Count -lt $MinWords -or $words.Count -gt $MaxWords) { continue }
            if ($s.Contains("http://") -or $s.Contains("https://")) { continue }
            # bo dong ky thuat / danh sach / mau cau (khong phai van ban thoai)
            if ($s.StartsWith("- ") -or $s.StartsWith("> ") -or $s.Contains("..")) { continue }
            if ($s -match "[{}\[\]<>|]") { continue }
            if ($seen.Add($s)) { $sentences.Add($s) }
        }
    }
}

$sb = New-Object System.Text.StringBuilder
[void]$sb.AppendLine("# corpus/assist/vi_docs_clean.ass - cau SACH lay tu van ban docs/ cua repo (P4-0 D19)")
[void]$sb.AppendLine("# Nguon: chinh tai lieu cua TextVN (GPL-3.0-or-later) -> khong rang buoc giay phep.")
[void]$sb.AppendLine("# Sinh lap: tools/assist/gen_clean_corpus.ps1 (loc: bo code fence/bang/backtick, 6..40 tu, >=50% chu co dau).")
[void]$sb.AppendLine("# Dung de do FPR tren VAN BAN THAT (khong phai cau mau) - Fassist-80.")
foreach ($s in $sentences) {
    [void]$sb.AppendLine($s + "`t")
}

$outPath = Join-Path $root $OutFile
$dir = Split-Path -Parent $outPath
if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
[System.IO.File]::WriteAllText($outPath, $sb.ToString(), (New-Object System.Text.UTF8Encoding($false)))
Write-Output "wrote $($sentences.Count) clean sentences -> $OutFile"
