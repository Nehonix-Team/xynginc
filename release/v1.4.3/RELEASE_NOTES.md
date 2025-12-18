# XyNginC v1.4.3 - Release Notes

## Robustness Update

**Release Date**: December 17, 2025  
**Version**: 1.4.3  
**Type**: Improvement & Bug Fix

## Key Improvements

### 🛡️ Fault-Tolerant SSL Setup

**Problem**: In previous versions, if SSL setup failed for one domain (e.g., due to rate limits, DNS issues, or invalid IP), the entire configuration process would stop. This meant that subsequent domains in the list would not be configured at all.

**Solution**: XyNginC v1.4.3 introduces a "fail-safe" mechanism for SSL:

- If SSL setup fails for a domain, XyNginC logs the error but **continues** processing.
- The failing domain automatically falls back to **HTTP-only** mode, ensuring the service remains accessible (albeit insecure).
- Other domains in the configuration are processed normally.

### 🐛 IP Address Handling (from v1.4.2)

- **Automatic IP Detection**: Detects if a domain is an IP address.
- **Auto-Disable SSL**: Automatically disables SSL for IP addresses (since Let's Encrypt doesn't support them) to prevent Certbot errors.

## Testing

### Scenario: Mixed Success

```json
{
  "domains": [
    { "domain": "valid.com", "ssl": true },
    { "domain": "invalid-dns.com", "ssl": true },
    { "domain": "another-valid.com", "ssl": true }
  ]
}
```

**Behavior**:

1. `valid.com`: Configured with SSL ✅
2. `invalid-dns.com`: SSL fails -> Fallback to HTTP (Warning logged) ⚠️
3. `another-valid.com`: Configured with SSL ✅

## Migration

Simply replace the binary.

```bash
sudo cp xynginc /usr/local/bin/xynginc
```

## Support

- **GitHub**: Report issues on repository
- **Docs**: See RELEASE_NOTES.md

---

**XyPriss Team**  
_Simplify infrastructure, amplify innovation_
