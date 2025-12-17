# XyNginC v1.4.0 - Auto-Healing & Enhanced Logging

**Release Date**: December 17, 2025

## Quick Start

### Installation

```bash
# Download the release
cd xynginc-v1.4.0

# Install (requires root)
sudo ./install.sh
```

### First Run

```bash
# Install system requirements (nginx, certbot, modules)
sudo xynginc install

# Apply your configuration
sudo xynginc apply config.json

# Check status
sudo xynginc status
```

## What's New in v1.4.0

### 🔧 Automatic Module Installation

XyNginC now automatically detects and installs the `headers-more-nginx-module`:

- **No manual compilation** required
- **Version-matched** to your nginx installation
- **Automatic detection** of missing modules
- **Seamless integration** into the workflow

### 🩹 Auto-Healing Configuration

Intelligent error detection and automatic repair:

- **Detects module errors** in nginx configuration
- **Automatically installs** missing modules
- **Retests configuration** after repair
- **Rollback protection** if repair fails

### 🎨 Enhanced Visual Logging

Improved readability with better visual hierarchy:

- **Red bold arrows** (`>`) for better visual distinction
- **Color preservation** for the rest of the message
- **Easier to follow** execution flow

## Files in This Release

- `xynginc` - Main binary (1.2MB, optimized)
- `install.sh` - Installation script
- `RELEASE_NOTES.md` - Detailed release notes
- `README.md` - This file

## System Requirements

- **OS**: Linux (Ubuntu 20.04+, Debian 11+, Kali Linux)
- **Nginx**: 1.18+ (automatically installed if missing)
- **Certbot**: Latest (automatically installed if missing)
- **Build tools**: Automatically installed when needed
- **Disk space**: ~100MB for module compilation (temporary)

## Configuration Example

```json
{
  "domains": [
    {
      "domain": "example.com",
      "port": 3000,
      "ssl": true,
      "email": "admin@example.com",
      "host": "localhost"
    }
  ],
  "auto_reload": true
}
```

## Commands

```bash
# Install system requirements
sudo xynginc install

# Apply configuration
sudo xynginc apply config.json

# Apply from stdin
echo '{"domains":[...]}' | sudo xynginc apply --config -

# Check status
sudo xynginc status

# List domains
sudo xynginc list

# Remove domain
sudo xynginc remove example.com

# Restore from backup
sudo xynginc restore

# Show version
xynginc --version
```

## Upgrade from v1.3.0

Simply replace the binary:

```bash
sudo cp xynginc /usr/local/bin/xynginc
```

On next run, the headers-more module will be automatically installed if missing.

## Troubleshooting

### Module Installation Fails

If automatic module installation fails:

1. **Check internet connection** - Required for downloads
2. **Verify nginx version** - Run `nginx -v`
3. **Check disk space** - Need ~100MB temporary space
4. **Review error logs** - Detailed messages provided

### Manual Module Installation

If needed, install manually:

```bash
# Ubuntu/Debian
sudo apt install nginx-module-headers-more

# Or compile from source
# (Follow the detailed instructions in error message)
```

## Support

- **Documentation**: See RELEASE_NOTES.md for detailed information
- **Issues**: Report on GitHub repository
- **Community**: Join the XyPriss community

## License

XyNginC is part of the XyPriss ecosystem.

---

**XyPriss Team**  
_Simplify infrastructure, amplify innovation_
