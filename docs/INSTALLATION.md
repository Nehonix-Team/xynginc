# Installation Guide - XyNginC

## Automatic Installation (Recommended)

### Via npm

The easiest way to install XyNginC is through npm:

```bash
npm install xynginc
```

The installation script will automatically:

- Check for required dependencies (Nginx, Certbot)
- Download the latest binary
- Install it to `/usr/local/bin/`
- Verify the installation

⚠️ **Note**: This requires sudo privileges. You'll be prompted for your password.

### Manual Binary Installation

If you prefer to install just the binary:

```bash
curl -L -o xynginc https://github.com/Nehonix-Team/xynginc/releases/latest/download/xynginc
chmod +x xynginc
sudo mv xynginc /usr/local/bin/
xynginc --version
```

### Full Installation with Dependencies

To install everything (binary + dependencies) using the installation script:

```bash
curl -fsSL https://raw.githubusercontent.com/Nehonix-Team/xynginc/master/scripts/install.sh | sudo bash
```

Or download and run locally:

```bash
git clone https://github.com/Nehonix-Team/xynginc.git
cd xynginc
sudo bash scripts/install.sh
```

## Prerequisites

XyNginC requires:

- **Operating System**: Linux (Ubuntu/Debian recommended)
- **Node.js**: Version 18.0.0 or higher (for npm installation)
- **Nginx**: Must be installed
- **Certbot**: Recommended for SSL/TLS certificates

### Installing Prerequisites

#### Ubuntu/Debian

```bash
# Update package list
sudo apt update

# Install Nginx
sudo apt install nginx

# Install Certbot
sudo apt install certbot python3-certbot-nginx

# Install Node.js (if needed)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
```

#### CentOS/RHEL/Fedora

```bash
# Install Nginx
sudo yum install nginx  # or dnf for Fedora

# Install Certbot
sudo yum install certbot python3-certbot-nginx

# Install Node.js
curl -fsSL https://rpm.nodesource.com/setup_20.x | sudo bash -
sudo yum install -y nodejs
```

## Verification

After installation, verify everything is working:

```bash
# Check XyNginC installation
xynginc --version

# Check system requirements
sudo xynginc check

# Check Nginx
nginx -v

# Check Certbot (optional)
certbot --version
```

## Troubleshooting

### Binary Not Found

If `xynginc` command is not found after installation:

1. Check if the binary exists:

   ```bash
   ls -la /usr/local/bin/xynginc
   ```

2. Verify `/usr/local/bin` is in your PATH:

   ```bash
   echo $PATH | grep "/usr/local/bin"
   ```

3. If not, add it to your PATH:
   ```bash
   echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

### Empty Binary Downloaded

If the downloaded binary is empty (0 bytes):

1. Check your internet connection
2. Verify the GitHub release exists
3. Try manual installation with curl (see above)
4. Check the logs for any error messages

### Permission Denied

If you get permission errors:

```bash
# Make sure the binary is executable
sudo chmod +x /usr/local/bin/xynginc

# Check ownership
sudo chown root:root /usr/local/bin/xynginc
```

### npm Installation Fails

If npm installation fails:

1. Try manual script installation:

   ```bash
   npm run install:manual
   ```

2. Or install the binary separately:
   ```bash
   curl -L -o xynginc https://github.com/Nehonix-Team/xynginc/releases/latest/download/xynginc
   chmod +x xynginc
   sudo mv xynginc /usr/local/bin/
   ```

## Uninstallation

To remove XyNginC:

```bash
# Remove binary
sudo rm /usr/local/bin/xynginc

# Remove npm package (if installed via npm)
npm uninstall xynginc

# Remove configuration (optional)
sudo rm -rf /etc/xynginc
```

## Support

If you encounter any issues:

1. Check the [GitHub Issues](https://github.com/Nehonix-Team/xynginc/issues)
2. Run `sudo xynginc check` for diagnostic information
3. Create a new issue with:
   - Your OS version
   - Node.js version
   - Error messages
   - Output of `sudo xynginc check`
