#!/bin/bash

# XyNginC v1.4.5 Installation Script

set -e

VERSION="1.4.5"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="xynginc"

echo "=================================="
echo "XyNginC v${VERSION} Installation"
echo "=================================="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Error: This script must be run as root (use sudo)"
    exit 1
fi

# Check if binary exists
if [ ! -f "./${BINARY_NAME}" ]; then
    echo "Error: Binary '${BINARY_NAME}' not found in current directory"
    exit 1
fi

# Backup existing installation
if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
    echo "Status: Backing up existing installation..."
    cp "${INSTALL_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}.backup.$(date +%Y%m%d_%H%M%S)"
    echo "Status: Backup created"
fi

# Install binary
echo "Status: Installing ${BINARY_NAME} to ${INSTALL_DIR}..."
cp "./${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
echo "Status: Binary installed"

# Verify installation
echo ""
echo "Status: Verifying installation..."
if command -v ${BINARY_NAME} &> /dev/null; then
    INSTALLED_VERSION=$(${BINARY_NAME} --version 2>&1 | grep -oP 'xynginc \K[0-9.]+' || echo "unknown")
    echo "Success: XyNginC installed successfully!"
    echo "Version: ${INSTALLED_VERSION}"
else
    echo "Error: Installation verification failed"
    exit 1
fi

echo ""
echo "=================================="
echo "Installation Complete"
echo "=================================="
echo ""
echo "New in v1.4.5:"
echo "  - Custom 301 Moved Permanently error page support"
echo "  - Updated Nginx templates for both SSL and non-SSL sites"
echo "  - Optimized TypeScript plugin (removed redundant header logic)"
echo "  - Enhanced fault tolerance for SSL acquisition"
echo "  - Licensed under NEHONIX Open Source License (NOSL) v1.0"
echo "  - Mandatory proprietary notices in configuration templates"
echo ""
echo "Next steps:"
echo "  1. Run: sudo xynginc install"
echo "     (Verify system requirements and install dependencies)"
echo ""
echo "  2. Apply your configuration:"
echo "     sudo xynginc apply --config your-config.json"
echo ""
echo "For detailed information, please refer to RELEASE_NOTES.md"
echo ""
