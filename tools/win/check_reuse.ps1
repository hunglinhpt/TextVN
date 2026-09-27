# SPDX-License-Identifier: GPL-3.0-or-later
$env:PYTHONUTF8 = "1"
$env:PYTHONIOENCODING = "utf-8"
reuse lint
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
