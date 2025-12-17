# Installation Guide

## System Requirements

XyNginC (XNCP) is strictly designed for **production environments** running on **Linux**. It relies on system-level components (Nginx, Certbot, systemd) that are native to Linux distributions.

**Recommended Environment:**

- **Platform**: Virtual Private Server (VPS) or Dedicated Server
- **OS**: Ubuntu 20.04 LTS or newer (Debian 11+, Kali Linux supported)
- **Architecture**: x86_64 (Standard 64-bit Linux)
- **Access**: Root or sudo privileges required

**Warning**: XNCP provides a pre-compiled binary for standard **Linux x86_64** systems only. Other architectures (like ARM/Raspberry Pi) are not officially supported via the pre-compiled binary and would require manual compilation from source. Windows and macOS are not supported.

## Installation Methods

### Option 1: NPM Installation (Recommended for Node.js Projects)

If you are integrating XNCP into a Node.js application (e.g., with XyPriss), installing via npm is the most streamlined approach. The post-install script will automatically download the binary.

```bash
npm install xynginc
```

**Note**: The installation process requires `sudo` privileges to place the binary in `/usr/local/bin` and configure system permissions. You may be prompted for your password during installation.

### Option 2: Manual Binary Installation

For standalone usage or environments without Node.js, you can install the binary directly using curl.

```bash
curl -L -o xynginc https://github.com/Nehonix-Team/xynginc/releases/latest/download/xynginc
chmod +x xynginc
sudo mv xynginc /usr/local/bin/
```

Verify the installation:

```bash
xynginc --version
```

### Option 3: Automated Install Script

We provide a shell script that installs the binary and attempts to resolve system dependencies (Nginx, Certbot).

```bash
curl -fsSL https://raw.githubusercontent.com/Nehonix-Team/xynginc/master/scripts/install.sh | sudo bash
```

## Post-Installation Setup

### 1. Install System Dependencies

XNCP requires Nginx and Certbot to function. While the automated script attempts to install them, we recommend verifying or installing them manually to ensure the latest stable versions are used.

**On Ubuntu/Debian:**

```bash
# Update package lists
sudo apt update

# Install Nginx
sudo apt install nginx

# Install Certbot and the Nginx plugin
sudo apt install certbot python3-certbot-nginx
```

### 2. Verify Environment

Run the built-in check command to ensure your system is ready for XNCP.

```bash
sudo xynginc check
```

This command validates:

- Nginx installation and version
- Certbot installation and Nginx plugin availability
- System permissions
- Network connectivity

## Troubleshooting

### Binary Not Found

If the `xynginc` command is not recognized after installation, ensure `/usr/local/bin` is in your system's PATH.

```bash
echo $PATH
```

If missing, add it to your shell configuration (e.g., `.bashrc` or `.profile`):

```bash
export PATH="/usr/local/bin:$PATH"
```

### Permission Issues

XNCP performs privileged operations such as writing to `/etc/nginx` and reloading services. Always execute XNCP commands with `sudo`.

```bash
sudo xynginc apply --config config.json
```

### SSL/Certbot Errors

If SSL certificate generation fails, ensure that:

1.  Your domain's DNS records (A/AAAA) are correctly pointing to your server's public IP.
2.  Ports 80 (HTTP) and 443 (HTTPS) are open in your firewall (UFW, iptables, or cloud provider security groups).

```bash
# Allow HTTP and HTTPS on UFW
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
```
