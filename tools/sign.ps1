# Signs the release build. Run it after `cargo build --release`.
#
# The certificate is never committed and never passed on the command line as plain
# text: point WSMF_CERT at a .pfx and WSMF_CERT_PASSWORD at its password, both as
# environment variables, or pass -Thumbprint for a certificate already in the store
# (which is how a hardware token works).
param(
    [string]$Exe = "$PSScriptRoot\..\target\release\wsmf.exe",
    [string]$Thumbprint,
    [string]$TimestampUrl = "http://timestamp.digicert.com"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $Exe)) {
    throw "not built yet: $Exe"
}

$signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match "x64" } |
    Sort-Object FullName -Descending |
    Select-Object -First 1
if (-not $signtool) {
    throw "signtool.exe not found; install the Windows SDK signing tools"
}

$arguments = @("sign", "/fd", "sha256", "/td", "sha256", "/tr", $TimestampUrl)

if ($Thumbprint) {
    $arguments += @("/sha1", $Thumbprint)
} elseif ($env:WSMF_CERT) {
    if (-not (Test-Path $env:WSMF_CERT)) { throw "WSMF_CERT does not point at a file" }
    $arguments += @("/f", $env:WSMF_CERT)
    if ($env:WSMF_CERT_PASSWORD) { $arguments += @("/p", $env:WSMF_CERT_PASSWORD) }
} else {
    throw "no certificate: set WSMF_CERT, or pass -Thumbprint"
}

$arguments += $Exe
& $signtool.FullName @arguments
if ($LASTEXITCODE -ne 0) { throw "signing failed with exit code $LASTEXITCODE" }

& $signtool.FullName verify /pa /v $Exe
if ($LASTEXITCODE -ne 0) { throw "the signature did not verify" }

$hash = (Get-FileHash $Exe -Algorithm SHA256).Hash
Set-Content -Path "$Exe.sha256" -Value $hash -Encoding ascii
Write-Output "signed, and SHA-256 is $hash"
Write-Output ""
Write-Output "Before publishing: upload it to VirusTotal and check the result."
Write-Output "A new certificate has no SmartScreen reputation yet, so the first few"
Write-Output "hundred downloads will still see a warning. That is expected."
