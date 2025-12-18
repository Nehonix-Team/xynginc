# XyNginC v1.4.2 - IP SSL Fix

**Release Date**: December 17, 2025  
**Type**: Bug Fix

## Quick Install

```bash
# Download and install
cd xynginc-v1.4.2
sudo ./install.sh
```

## What's Fixed

### 🐛 IP Address SSL Error

**Problem**: Requesting SSL for an IP address caused Certbot to fail.

**Fixed**:

- ✅ **Auto-detect IP addresses**
- ✅ **Disable SSL automatically** for IPs (fallback to HTTP)
- ✅ **Prevent Certbot errors**

## Upgrade from v1.4.1

```bash
sudo cp xynginc /usr/local/bin/xynginc
```

## Files

- `xynginc` - Fixed binary (1.2MB)
- `install.sh` - Installation script
- `RELEASE_NOTES.md` - Detailed notes
- `README.md` - This file

---

**XyPriss Team**  
_Simplify infrastructure, amplify innovation_
