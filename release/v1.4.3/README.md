# XyNginC v1.4.3 - Robustness Update

**Release Date**: December 17, 2025  
**Type**: Improvement

## Quick Install

```bash
# Download and install
cd xynginc-v1.4.3
sudo ./install.sh
```

## What's New

### 🛡️ Fault Tolerance

- **Non-blocking SSL**: SSL failures no longer stop the entire configuration process.
- **Auto-Fallback**: Domains with failed SSL setup automatically revert to HTTP.
- **IP Handling**: Automatically handles IP addresses by disabling SSL.

## Upgrade

```bash
sudo cp xynginc /usr/local/bin/xynginc
```

## Files

- `xynginc` - Binary (1.2MB)
- `install.sh` - Installation script
- `RELEASE_NOTES.md` - Detailed notes
- `README.md` - This file

---

**XyPriss Team**  
_Simplify infrastructure, amplify innovation_
