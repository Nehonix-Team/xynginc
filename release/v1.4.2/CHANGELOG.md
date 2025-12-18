# XyNginC v1.4.2 - Changelog

## Version 1.4.2 - "IP SSL Fix" (2025-12-17)

### 🐛 Bug Fixes

#### IP Address Handling

- **FIXED**: Certbot failure when `ssl: true` is set for an IP address.
- **NEW**: Automatic detection of IP addresses in domain configuration.
- **NEW**: Graceful fallback to HTTP for IP addresses (SSL disabled automatically).
- **NEW**: Warning message explaining why SSL was disabled for the IP.

### 🔧 Technical Changes

- **Modified**: `core/src/mods/apply.rs` - Added IP detection logic using `std::net::IpAddr`.
- **Modified**: `core/Cargo.toml` - Version bump to 1.4.2.

### 📊 Statistics

- **Lines Modified**: ~20
- **Binary Size**: 1.2MB

### 🔄 Migration Notes

**From v1.4.1 to v1.4.2:**

1. Replace binary: `sudo cp xynginc /usr/local/bin/xynginc`
2. No configuration changes required. Existing configs with IPs will now work without error.

---

**Full Release Notes**: See `RELEASE_NOTES.md`  
**Quick Start**: See `README.md`
