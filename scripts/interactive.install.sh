#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
GITHUB_REPO="Nehonix-Team/xynginc"
BINARY_NAME="xynginc"
INSTALL_DIR="/usr/local/bin"

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   XyNginC Installation Script         ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Check if running as root or with sudo
if [ "$EUID" -ne 0 ]; then 
    echo -e "${YELLOW}⚠️  This script requires sudo privileges${NC}"
    echo -e "${YELLOW}   Re-running with sudo...${NC}"
    exec sudo "$0" "$@"
fi

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install package
install_package() {
    local package=$1
    echo -e "${BLUE}📦 Installing $package...${NC}"
    
    if command_exists apt-get; then
        apt-get update -qq
        apt-get install -y "$package"
    elif command_exists yum; then
        yum install -y "$package"
    elif command_exists dnf; then
        dnf install -y "$package"
    else
        echo -e "${RED}❌ Package manager not supported${NC}"
        return 1
    fi
}

echo -e "${BLUE}> Checking system requirements...${NC}"
echo ""

# Check and install Nginx
if command_exists nginx; then
    NGINX_VERSION=$(nginx -v 2>&1 | cut -d'/' -f2)
    echo -e "${GREEN}✅ Nginx is installed (version $NGINX_VERSION)${NC}"
else
    echo -e "${YELLOW}⚠️  Nginx not found${NC}"
    read -p "   Do you want to install Nginx? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        install_package nginx
        echo -e "${GREEN}✅ Nginx installed successfully${NC}"
    else
        echo -e "${RED}❌ Nginx is required. Installation aborted.${NC}"
        exit 1
    fi
fi

# Check and install Certbot
if command_exists certbot; then
    CERTBOT_VERSION=$(certbot --version 2>&1 | cut -d' ' -f2)
    echo -e "${GREEN}✅ Certbot is installed (version $CERTBOT_VERSION)${NC}"
else
    echo -e "${YELLOW}⚠️  Certbot not found${NC}"
    read -p "   Do you want to install Certbot? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        install_package certbot
        install_package python3-certbot-nginx
        echo -e "${GREEN}✅ Certbot installed successfully${NC}"
    else
        echo -e "${YELLOW}⚠️  Certbot is recommended but not required${NC}"
    fi
fi

# Check Node.js
if command_exists node; then
    NODE_VERSION=$(node -v | cut -d'v' -f2)
    NODE_MAJOR=$(echo $NODE_VERSION | cut -d'.' -f1)
    if [ "$NODE_MAJOR" -ge 18 ]; then
        echo -e "${GREEN}✅ Node.js is installed (version $NODE_VERSION)${NC}"
    else
        echo -e "${YELLOW}⚠️  Node.js version $NODE_VERSION is too old (requires >= 18.0.0)${NC}"
        echo -e "${YELLOW}   Please upgrade Node.js manually${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Node.js not found (required for npm usage)${NC}"
    echo -e "${YELLOW}   Please install Node.js >= 18.0.0 manually${NC}"
fi

# Check curl
if ! command_exists curl; then
    echo -e "${YELLOW}⚠️  curl not found, installing...${NC}"
    install_package curl
fi

echo ""
echo -e "${BLUE}📥 Downloading XyNginC binary...${NC}"

# Download binary
DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/${BINARY_NAME}"
TEMP_FILE="/tmp/${BINARY_NAME}"

echo -e "${BLUE}   URL: $DOWNLOAD_URL${NC}"

if curl -L -f -o "$TEMP_FILE" "$DOWNLOAD_URL"; then
    # Check if file is not empty
    if [ -s "$TEMP_FILE" ]; then
        echo -e "${GREEN}✅ Binary downloaded successfully${NC}"
        
        # Make executable
        chmod +x "$TEMP_FILE"
        
        # Move to installation directory
        mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"
        
        echo -e "${GREEN}✅ Binary installed to $INSTALL_DIR/$BINARY_NAME${NC}"
        
        # Verify installation
        if command_exists xynginc; then
            VERSION=$($BINARY_NAME --version 2>&1 || echo "unknown")
            echo -e "${GREEN}✅ Installation successful!${NC}"
            echo -e "${GREEN}   Version: $VERSION${NC}"
        else
            echo -e "${RED}❌ Installation failed: binary not found in PATH${NC}"
            exit 1
        fi
    else
        echo -e "${RED}❌ Downloaded file is empty${NC}"
        rm -f "$TEMP_FILE"
        exit 1
    fi
else
    echo -e "${RED}❌ Failed to download binary${NC}"
    echo -e "${RED}   Please check your internet connection and try again${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  🎉 XyNginC installed successfully!   ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo -e "  1. Run: ${GREEN}xynginc check${NC} to verify installation"
echo -e "  2. Run: ${GREEN}xynginc --help${NC} to see available commands"
echo -e "  3. Configure your XyPriss plugin to use XyNginC"
echo ""