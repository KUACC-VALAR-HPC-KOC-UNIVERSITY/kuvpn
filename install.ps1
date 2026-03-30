# KUVPN Windows Installer
# Usage: irm https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.ps1 | iex

$ErrorActionPreference = 'Stop'

$Repo    = "ealtun21/kuvpn-actions"
$ExeName = "KUVPN-Setup-windows-x86_64.exe"

function Write-Info    { Write-Host "[INFO] $args" -ForegroundColor Cyan }
function Write-Ok      { Write-Host "[OK]   $args" -ForegroundColor Green }
function Write-Warn    { Write-Host "[WARN] $args" -ForegroundColor Yellow }
function Write-Fail    { Write-Host "[FAIL] $args" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Info "KUVPN Windows Installer"
Write-Host ""

# --- Resolve latest version ---
Write-Info "Resolving latest version..."
try {
    $releaseApi = "https://api.github.com/repos/$Repo/releases/latest"
    $release    = Invoke-RestMethod -Uri $releaseApi -UseBasicParsing
    $Tag        = $release.tag_name
} catch {
    Write-Fail "Could not fetch release info: $_"
}

if (-not $Tag) { Write-Fail "Unable to resolve latest version." }
Write-Info "Selected version: $Tag"

# --- Download ---
$DownloadUrl = "https://github.com/$Repo/releases/download/$Tag/$ExeName"
$TempFile    = Join-Path $env:TEMP $ExeName

Write-Info "Downloading installer from: $DownloadUrl"
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile -UseBasicParsing
} catch {
    Write-Fail "Download failed: $_"
}
Write-Ok "Downloaded to $TempFile"

# --- Run installer ---
Write-Info "Installing silently (this may take a moment)..."
$proc = Start-Process -FilePath $TempFile -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -Wait -PassThru

if ($proc.ExitCode -ne 0) {
    Write-Fail "Installer exited with code $($proc.ExitCode). Installation may have failed or been cancelled."
}

Write-Host ""
Write-Ok "Done! KUVPN has been installed."
Write-Host "  OpenConnect is bundled — no additional dependencies needed."
Write-Host ""
