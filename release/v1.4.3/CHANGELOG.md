# XyNginC v1.4.3 - Changelog

## Version 1.4.3 - "Robustness Update" (2025-12-17)

### 🚀 Improvements

#### Fault Tolerance

- **NEW**: Implemented non-blocking SSL setup. Errors during certificate acquisition for one domain no longer abort the entire configuration process.
- **NEW**: Automatic fallback to HTTP for domains where SSL setup fails.
- **NEW**: Detailed error logging for failed domains while continuing execution.

### 🐛 Bug Fixes

#### IP Address Handling (Included from v1.4.2)

- **FIXED**: Certbot failure when `ssl: true` is set for an IP address.
- **NEW**: Automatic detection and SSL disablement for IP addresses.

### 🔧 Technical Changes

- **Modified**: `core/src/mods/apply.rs` - Refactored `apply_config` loop to handle `setup_ssl` errors gracefully using `match` instead of `?`.
- **Modified**: `core/Cargo.toml` - Version bump to 1.4.3.

### 📊 Statistics

- **Lines Modified**: ~30
- **Binary Size**: 1.2MB

### 🔄 Migration Notes

**From v1.4.1 to v1.4.3:**

1. Replace binary: `sudo cp xynginc /usr/local/bin/xynginc`
2. Re-run your configuration apply command. Any previously failed domains should now work (at least in HTTP mode), and valid domains will get SSL.

---

**Full Release Notes**: See `RELEASE_NOTES.md`  
**Quick Start**: See `README.md`
