#!/bin/bash

# XyNginC v1.4.3 Installation Script
# Auto-Healing & Enhanced Logging Release

set -e

VERSION="1.4.3"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="xynginc"

echo "=================================="
echo "XyNginC v${VERSION} Installation"
echo "=================================="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "❌ This script must be run as root (use sudo)"
    exit 1
fi

# Check if binary exists
if [ ! -f "./${BINARY_NAME}" ]; then
    echo "❌ Binary '${BINARY_NAME}' not found in current directory"
    exit 1
fi

# Backup existing installation
if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
    echo "> Backing up existing installation..."
    cp "${INSTALL_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}.backup.$(date +%Y%m%d_%H%M%S)"
    echo "✓ Backup created"
fi

# Install binary
echo "> Installing ${BINARY_NAME} to ${INSTALL_DIR}..."
cp "./${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
echo "✓ Binary installed"

# Verify installation
echo ""
echo "> Verifying installation..."
if command -v ${BINARY_NAME} &> /dev/null; then
    INSTALLED_VERSION=$(${BINARY_NAME} --version 2>&1 | grep -oP 'xynginc \K[0-9.]+' || echo "unknown")
    echo "✓ XyNginC installed successfully!"
    echo "  Version: ${INSTALLED_VERSION}"
else
    echo "❌ Installation verification failed"
    exit 1
fi

echo ""
echo "=================================="
echo "Installation Complete!"
echo "=================================="
echo ""
echo "What's new in v1.4.3:"
echo "  • Automatic headers-more module installation"
echo "  • Auto-healing configuration errors"
echo "  • Enhanced visual logging with red arrows"
echo "  • Intelligent module detection and compilation"
echo ""
echo "Next steps:"
echo "  1. Run: sudo xynginc install"
echo "     (Installs nginx, certbot, and required modules)"
echo ""
echo "  2. Apply your configuration:"
echo "     sudo xynginc apply config.json"
echo ""
echo "  3. Check status:"
echo "     sudo xynginc status"
echo ""
echo "For more information, see RELEASE_NOTES.md"
echo ""
