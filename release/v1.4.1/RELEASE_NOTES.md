# XyNginC v1.4.1 - Release Notes

## Patch Release

**Release Date**: December 17, 2025  
**Version**: 1.4.1  
**Type**: Critical Bug Fix Release

## Critical Fixes

### 1. 🐛 SSL Workflow Logic Correction

**Problem**: When enabling SSL for the first time, XyNginC generated the full HTTPS configuration _before_ obtaining the SSL certificate. This caused Nginx validation to fail because the referenced certificate files (e.g., `/etc/letsencrypt/live/.../fullchain.pem`) did not exist yet.

**Solution**: The SSL provisioning workflow has been completely rewritten:

1.  **Step 1**: Generate a temporary **HTTP-only** configuration.
2.  **Step 2**: Reload Nginx to apply this valid configuration.
3.  **Step 3**: Run Certbot to obtain the SSL certificate (now that the server is reachable via HTTP).
4.  **Step 4**: Generate the final **HTTPS** configuration referencing the new certificates.
5.  **Step 5**: Reload Nginx with the secure configuration.

This ensures a seamless "zero-touch" SSL setup without chicken-and-egg errors.

### 2. 🐛 Certbot Nginx Plugin Auto-Installation

**Problem**: The `python3-certbot-nginx` plugin was not being verified during requirements check. If missing, Certbot would fail with "The requested nginx plugin does not appear to be installed".

**Solution**:

- **Enhanced Verification**: The `requirements` check now explicitly verifies the presence of the Nginx plugin for Certbot.
- **Auto-Installation**: If the plugin is missing during SSL setup, XyNginC automatically installs it via `apt-get`.
- **Auto-Retry**: The system automatically retries the SSL request after installing the missing plugin.

## Technical Changes

### SSL Provisioning Workflow

The `apply_config` logic has been updated to handle the SSL bootstrap process:

```rust
if domain_config.ssl {
    log_info("> SSL requested - generating temporary HTTP configuration first");

    // 1. Create temporary HTTP config
    let mut temp_config = domain_config.clone();
    temp_config.ssl = false;
    generate_nginx_config(&temp_config)?;
    enable_site(&temp_config.domain)?;

    // 2. Reload for Certbot validation
    reload_nginx()?;

    // 3. Obtain Certificate
    setup_ssl(domain_config)?;

    // 4. Generate final HTTPS config
    generate_nginx_config(domain_config)?;
    enable_site(&domain_config.domain)?;
}
```

### Enhanced Requirements Check

The requirements module now parses `certbot plugins` output:

```rust
// Checks certbot AND nginx plugin availability
let plugins_output = Command::new("certbot").args(&["plugins", "--text"]).output();
// ... parses output for "nginx" ...
```

## Complete Change List

### Fixes

- **Fixed SSL Bootstrap**: Implemented HTTP-first workflow to prevent "certificate file not found" errors during initial setup.
- **Fixed Certbot Plugin**: Added automatic detection and installation of `python3-certbot-nginx`.
- **Fixed Requirements**: Updated `check_missing_requirements` to validate Certbot plugin status.
- **Fixed Error Handling**: Better error messages when Certbot fails.

## Migration from v1.4.0

Migration is automatic. Simply replace the binary:

1.  **Update binary**: Replace with new v1.4.1 binary.
2.  **Run**: No special commands needed. The next time you apply a configuration with SSL, the new workflow will be used.

## Installation

### Fresh Installation

```bash
# Download and install
sudo xynginc install
```

### Update from v1.4.0

```bash
# Simply replace the binary
sudo cp xynginc /usr/local/bin/xynginc
```

## Testing

### Test Scenario: Enabling SSL

```json
{
  "domains": [
    {
      "domain": "example.com",
      "port": 3000,
      "ssl": true,
      "email": "admin@example.com"
    }
  ]
}
```

```bash
sudo xynginc apply config.json
```

**Expected Behavior**:

1.  XyNginC generates HTTP config.
2.  Nginx reloads.
3.  Certbot runs (installs plugin if missing).
4.  Certificates obtained.
5.  XyNginC generates HTTPS config.
6.  Nginx reloads with SSL.

## Support

For any questions or issues:

- Open an issue on GitHub repository
- Consult complete documentation
- Join the user community

---

**XyPriss Team**  
_Simplify infrastructure, amplify innovation_
