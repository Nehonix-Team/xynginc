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

# Check for non-interactive mode (when run from npm)
AUTO_INSTALL=false
if [ "$SKIP_PROMPTS" = "1" ]; then
    AUTO_INSTALL=true
fi

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   XyNginC Installation Script         ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Check if running as root or with sudo
if [ "$EUID" -ne 0 ]; then 
    echo -e "${YELLOW}⚠️  This script requires sudo privileges${NC}"
    if [ "$AUTO_INSTALL" = false ]; then
        echo -e "${YELLOW}   Re-running with sudo...${NC}"
        exec sudo "$0" "$@"
    else
        echo -e "${RED}❌ Please run with sudo or as root${NC}"
        exit 1
    fi
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
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -qq >/dev/null 2>&1
        apt-get install -y -qq "$package" >/dev/null 2>&1
    elif command_exists yum; then
        yum install -y -q "$package" >/dev/null 2>&1
    elif command_exists dnf; then
        dnf install -y -q "$package" >/dev/null 2>&1
    else
        echo -e "${RED}❌ Package manager not supported${NC}"
        return 1
    fi
}

# Function to prompt for installation
prompt_install() {
    local package=$1
    local required=$2
    
    if [ "$AUTO_INSTALL" = true ]; then
        if [ "$required" = true ]; then
            install_package "$package"
            return 0
        else
            return 1
        fi
    else
        read -p "   Do you want to install $package? (y/n) " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            install_package "$package"
            return 0
        else
            return 1
        fi
    fi
}

echo -e "${BLUE}🔍 Checking system requirements...${NC}"
echo ""

# Check and install Nginx
if command_exists nginx; then
    NGINX_VERSION=$(nginx -v 2>&1 | cut -d'/' -f2)
    echo -e "${GREEN}✅ Nginx is installed (version $NGINX_VERSION)${NC}"
else
    echo -e "${YELLOW}⚠️  Nginx not found${NC}"
    if prompt_install "nginx" true; then
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
    if prompt_install "certbot" false; then
        if command_exists apt-get; then
            install_package "python3-certbot-nginx"
        fi
        echo -e "${GREEN}✅ Certbot installed successfully${NC}"
    else
        echo -e "${YELLOW}⚠️  Certbot is recommended but not required${NC}"
    fi
fi

# Check Node.js (only warning, not required for binary)
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
    if [ "$AUTO_INSTALL" = false ]; then
        echo -e "${YELLOW}⚠️  Node.js not found (required for npm usage)${NC}"
        echo -e "${YELLOW}   Please install Node.js >= 18.0.0 manually${NC}"
    fi
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
TEMP_FILE="/tmp/${BINARY_NAME}.$$"

if [ "$AUTO_INSTALL" = false ]; then
    echo -e "${BLUE}   URL: $DOWNLOAD_URL${NC}"
fi

if curl -L -f -s -o "$TEMP_FILE" "$DOWNLOAD_URL"; then
    # Check if file is not empty
    if [ -s "$TEMP_FILE" ]; then
        FILE_SIZE=$(stat -f%z "$TEMP_FILE" 2>/dev/null || stat -c%s "$TEMP_FILE" 2>/dev/null)
        echo -e "${GREEN}✅ Binary downloaded successfully (${FILE_SIZE} bytes)${NC}"
        
        # Make executable
        chmod +x "$TEMP_FILE"
        
        # Backup existing binary if present
        if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
            mv "$INSTALL_DIR/$BINARY_NAME" "$INSTALL_DIR/${BINARY_NAME}.backup.$(date +%s)"
            echo -e "${YELLOW}   Previous version backed up${NC}"
        fi
        
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
    echo -e "${RED}   URL: $DOWNLOAD_URL${NC}"
    rm -f "$TEMP_FILE"
    exit 1
fi

echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  🎉 XyNginC installed successfully!   ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""

if [ "$AUTO_INSTALL" = false ]; then
    echo -e "${BLUE}Next steps:${NC}"
    echo -e "  1. Run: ${GREEN}xynginc check${NC} to verify installation"
    echo -e "  2. Run: ${GREEN}xynginc --help${NC} to see available commands"
    echo -e "  3. Configure your XyPriss plugin to use XyNginC"
    echo ""
fi