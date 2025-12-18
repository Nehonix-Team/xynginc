# XyNginC v1.4.2 - Release Notes

## Patch Release

**Release Date**: December 17, 2025  
**Version**: 1.4.2  
**Type**: Bug Fix Release

## Critical Fix

### 🐛 IP Address SSL Handling

**Problem**: When a configuration included an IP address as a domain (e.g., `173.249.48.198`) with `ssl: true`, XyNginC attempted to request a Let's Encrypt certificate. This failed because Let's Encrypt does not issue certificates for bare IP addresses, causing the entire configuration application to fail.

**Solution**:

- ✅ **Automatic IP Detection**: XyNginC now detects if a "domain" is actually an IP address.
- ✅ **Graceful Fallback**: If SSL is requested for an IP address, it automatically falls back to HTTP.
- ✅ **Warning Message**: Displays a clear warning to the user explaining why SSL was disabled for the IP.

## Changes in v1.4.2

### IP Address Logic

```rust
// Check if domain is an IP
let is_ip = domain_config.domain.parse::<std::net::IpAddr>().is_ok();

if domain_config.ssl && is_ip {
    log_warning("⚠️  SSL requested for IP address, but Let's Encrypt does not support IP addresses.");
    log_warning("   Falling back to HTTP for this domain.");
    // Disable SSL automatically
}
```

## Testing

### Test Scenario: IP with SSL

```json
{
  "domains": [
    {
      "domain": "192.168.1.1",
      "port": 3000,
      "ssl": true
    }
  ]
}
```

**Before v1.4.2**: Failed with Certbot error.  
**After v1.4.2**: Succeeds with HTTP configuration and warning.

## Migration

Simply replace the binary. No configuration changes needed.

```bash
sudo cp xynginc /usr/local/bin/xynginc
```

## Support

- **GitHub**: Report issues on repository
- **Docs**: See RELEASE_NOTES.md

---

**XyPriss Team**  
_Simplify infrastructure, amplify innovation_
