# Binance-Claw installer for Windows
# Usage (PowerShell): irm https://raw.githubusercontent.com/deepcon3/Binance-Claw/main/scripts/install.ps1 | iex
# Usage (CMD):        powershell -c "irm https://raw.githubusercontent.com/deepcon3/Binance-Claw/main/scripts/install.ps1 | iex"

$ErrorActionPreference = 'Stop'

$REPO     = "deepcon3/Binance-Claw"
$BINARY   = "binance-claw.exe"
$ARTIFACT = "binance-claw-windows-x86_64.exe"
$INSTALL_DIR = if ($env:BINANCE_CLAW_INSTALL_DIR) { $env:BINANCE_CLAW_INSTALL_DIR } else { "$env:LOCALAPPDATA\binance-claw" }

function Write-Info    { param($msg) Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Success { param($msg) Write-Host "[OK]   $msg" -ForegroundColor Green }
function Write-Warn    { param($msg) Write-Host "[WARN] $msg" -ForegroundColor Yellow }
function Write-Err     { param($msg) Write-Host "[ERR]  $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "  ⚡ Binance-Claw Installer" -ForegroundColor Yellow -NoNewline
Write-Host " — Lightning-fast crypto price sniper"
Write-Host ""

# Get latest release
Write-Info "Fetching latest release from GitHub..."
try {
    $releaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest" -UseBasicParsing
    $version = $releaseInfo.tag_name
} catch {
    Write-Err "Failed to fetch latest release: $_"
}

Write-Info "Latest version: $version"
$downloadUrl = "https://github.com/$REPO/releases/download/$version/$ARTIFACT"

# Create install directory
if (-not (Test-Path $INSTALL_DIR)) {
    New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
}

$destPath = Join-Path $INSTALL_DIR $BINARY

# Download
Write-Info "Downloading $ARTIFACT..."
try {
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $downloadUrl -OutFile $destPath -UseBasicParsing
    $ProgressPreference = 'Continue'
} catch {
    Write-Err "Download failed: $_"
}

Write-Success "Installed to: $destPath"

# Add to PATH if not already present
$currentPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$INSTALL_DIR*") {
    Write-Info "Adding $INSTALL_DIR to user PATH..."
    $newPath = "$INSTALL_DIR;$currentPath"
    [System.Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    $env:Path = "$INSTALL_DIR;$env:Path"
    Write-Success "PATH updated. You may need to restart your terminal."
} else {
    Write-Info "$INSTALL_DIR already in PATH."
}

Write-Host ""
Write-Host "  ✓ binance-claw installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "  Try it (open a new CMD or PowerShell window):"
Write-Host "    binance-claw ping               " -NoNewline; Write-Host "- check connectivity" -ForegroundColor DarkGray
Write-Host "    binance-claw price BTCUSDT      " -NoNewline; Write-Host "- get BTC price" -ForegroundColor DarkGray
Write-Host "    binance-claw watch ETHUSDT      " -NoNewline; Write-Host "- real-time stream" -ForegroundColor DarkGray
Write-Host "    binance-claw claw BTCUSDT above 70000  " -NoNewline; Write-Host "- price alert" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  NOTE: Restart your terminal for PATH changes to take effect." -ForegroundColor Yellow
Write-Host ""
