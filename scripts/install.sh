#!/usr/bin/env bash
# Binance-Claw installer for Linux and macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/deepcon3/Binance-Claw/main/scripts/install.sh | bash

set -euo pipefail

REPO="deepcon3/Binance-Claw"
BINARY="binance-claw"
INSTALL_DIR="${BINANCE_CLAW_INSTALL_DIR:-$HOME/.local/bin}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

info()    { echo -e "${CYAN}[INFO]${RESET} $*"; }
success() { echo -e "${GREEN}[OK]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[WARN]${RESET} $*"; }
error()   { echo -e "${RED}[ERR]${RESET} $*" >&2; exit 1; }

echo -e ""
echo -e "${BOLD}${YELLOW}  ⚡ Binance-Claw Installer${RESET}"
echo -e "  Lightning-fast crypto price sniper & OpenClaw skill"
echo -e ""

# Detect OS and architecture
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Linux)  OS_LABEL="linux" ;;
  Darwin) OS_LABEL="macos" ;;
  *)      error "Unsupported OS: $OS" ;;
esac

case "$ARCH" in
  x86_64)          ARCH_LABEL="x86_64" ;;
  aarch64|arm64)   ARCH_LABEL="aarch64" ;;
  *)               error "Unsupported architecture: $ARCH" ;;
esac

ARTIFACT="${BINARY}-${OS_LABEL}-${ARCH_LABEL}"
info "Detected: ${OS} / ${ARCH} → ${ARTIFACT}"

# Get latest release tag from GitHub
info "Fetching latest release..."
if command -v curl &>/dev/null; then
    LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
elif command -v wget &>/dev/null; then
    LATEST=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
else
    error "Neither curl nor wget found. Please install one of them."
fi

if [[ -z "$LATEST" ]]; then
    error "Could not determine latest release version."
fi

info "Latest version: ${LATEST}"

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${ARTIFACT}"
TMP_DIR=$(mktemp -d)
TMP_BIN="$TMP_DIR/$BINARY"

# Download
info "Downloading ${ARTIFACT}..."
if command -v curl &>/dev/null; then
    curl -fsSL --progress-bar "$DOWNLOAD_URL" -o "$TMP_BIN"
else
    wget --show-progress -qO "$TMP_BIN" "$DOWNLOAD_URL"
fi

chmod +x "$TMP_BIN"

# Install
mkdir -p "$INSTALL_DIR"
mv "$TMP_BIN" "$INSTALL_DIR/$BINARY"
rm -rf "$TMP_DIR"

success "Installed to: ${INSTALL_DIR}/${BINARY}"

# Add to PATH if needed
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    warn "${INSTALL_DIR} is not in your PATH."
    echo ""
    echo -e "  Add this to your shell profile (${YELLOW}~/.bashrc${RESET}, ${YELLOW}~/.zshrc${RESET}, etc.):"
    echo -e "  ${CYAN}export PATH=\"\$HOME/.local/bin:\$PATH\"${RESET}"
    echo ""
fi

# Verify
if command -v "$BINARY" &>/dev/null || [[ -x "$INSTALL_DIR/$BINARY" ]]; then
    echo ""
    echo -e "${BOLD}${GREEN}  ✓ binance-claw installed successfully!${RESET}"
    echo ""
    echo "  Try it:"
    echo -e "    ${CYAN}binance-claw ping${RESET}               — check connectivity"
    echo -e "    ${CYAN}binance-claw price BTCUSDT${RESET}      — get BTC price"
    echo -e "    ${CYAN}binance-claw watch ETHUSDT${RESET}      — real-time stream"
    echo -e "    ${CYAN}binance-claw claw BTCUSDT above 70000${RESET} — price alert"
    echo ""
else
    warn "Binary installed but not yet in PATH. Run:"
    echo "  export PATH=\"\$INSTALL_DIR:\$PATH\""
fi
