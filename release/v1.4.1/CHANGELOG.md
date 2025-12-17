# XyNginC v1.4.1 - Changelog

## Version 1.4.1 - "SSL Workflow Fix" (2025-12-17)

### 🐛 Critical Bug Fixes

#### SSL Bootstrap Workflow

- **FIXED**: "File not found" error when enabling SSL for the first time.
- **CHANGED**: Implemented a multi-step workflow for SSL setup:
  1. Generate temporary HTTP configuration
  2. Reload Nginx
  3. Obtain SSL certificate via Certbot
  4. Generate final HTTPS configuration
  5. Reload Nginx
- **RESULT**: Seamless SSL setup without manual intervention.

#### Certbot Plugin Management

- **FIXED**: "The requested nginx plugin does not appear to be installed" error.
- **NEW**: Automatic detection of `python3-certbot-nginx` plugin.
- **NEW**: Auto-installation of the plugin if missing during SSL setup.
- **NEW**: Enhanced requirements check to verify plugin presence.

### 🔧 Technical Changes

- **Modified**: `core/src/mods/apply.rs` - Updated `apply_config` to handle the HTTP-first workflow.
- **Modified**: `core/src/mods/ssl.rs` - Added plugin check and auto-installation logic.
- **Modified**: `core/src/requirements.rs` - Added plugin verification to system checks.
- **Modified**: `core/Cargo.toml` - Version bump to 1.4.1.

### 📊 Statistics

- **Lines Modified**: ~100
- **New Functions**: 2
- **Binary Size**: 1.2MB

### 🔄 Migration Notes

**From v1.4.0 to v1.4.1:**

1. Replace binary: `sudo cp xynginc /usr/local/bin/xynginc`
2. No configuration changes required.

---

**Full Release Notes**: See `RELEASE_NOTES.md`  
**Quick Start**: See `README.md`
