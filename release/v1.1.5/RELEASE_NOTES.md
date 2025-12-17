# v1.1.5 - Critical Bug Fixes and Complete Template System

This release of XyNginC (XyPriss Nginx Controller) addresses critical bugs that prevented proper nginx configuration generation and fixes web directory structure issues for production-ready deployment.

## 🚨 Critical Bug Fixes

### 1. Template Variable Replacement Bug (CRITICAL FIX)

**Issue**: Template variables like `{{DOMAIN_NAME}}`, `{{BACKEND_HOST}}`, `{{BACKEND_PORT}}` were not being replaced in generated nginx configurations, causing nginx test failures.

**Error Encountered**:

```nginx
# Generated config contained literal template variables:
server {
    listen 80;
    server_name {{DOMAIN_NAME}};  # ❌ Should be: server.nehonix.xyz
    proxy_pass http://{{BACKEND_HOST}}:{{BACKEND_PORT}};  # ❌ Should be: localhost:9837
}
```

**Root Cause**:
The `replace_template_variables()` function had incorrect brace formatting:

```rust
// Before (incorrect):
let placeholder = format!("{{{{{} }}}}}", key);  // Results in "{{DOMAIN_NAME }}"

// After (correct):
let placeholder = format!("{{{{{}}}}}", key);    // Results in "{{DOMAIN_NAME}}"
```

**Solution Applied**:

- Fixed brace formatting to match template syntax exactly
- Removed extra space before closing brace
- Ensured consistent variable replacement throughout all templates

### 2. Web Directory Structure Fix (CRITICAL)

**Issue**: Web pages were being created in non-standard locations and default nginx welcome page was not replaced.

**Problems Fixed**:

- **Error pages**: Were in `/var/www/xynginc/error.html` instead of standard `/var/www/html/errors/error.html`
- **Index page**: Default nginx welcome page (`index.nginx-debian.html`) was not being replaced
- **Directory structure**: Non-standard web directory usage

**Solution Applied**:

- **Updated error page location**: Now creates `/var/www/html/errors/error.html`
- **Created index page replacement**: Automatically replaces default nginx welcome page
- **Added comprehensive web page management**: Both error and index pages setup
- **Automatic cleanup**: Removes default nginx files when appropriate

### 3. Template Embedding Enhancement

**Added Missing Template**: Created and embedded `index.html` template for the index page replacement.

**Templates Now Embedded**:

1. `non_ssl_template.conf` - Nginx configuration for non-SSL domains
2. `ssl_template.conf` - Nginx configuration for SSL-enabled domains
3. `error.html` - Professional error page with TailwindCSS styling
4. `index.html` - XyNginC branded index page (NEW)

## Technical Improvements

### Template System Overhaul

#### Fixed Variable Replacement Function

```rust
/// Replace template variables with actual values
fn replace_template_variables(template: &str, variables: &[(&str, &str)]) -> String {
    let mut result = template.to_string();

    for (key, value) in variables {
        // Fixed: Removed space before closing brace
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }

    result
}
```

#### Enhanced Web Page Management

```rust
/// Ensure the custom error page exists in the web directory
fn ensure_error_page_exists() -> Result<(), String> {
    let error_page_dir = "/var/www/html/errors";  // Fixed path
    let error_page_path = format!("{}/error.html", error_page_dir);

    // Create errors directory if it doesn't exist
    if !Path::new(error_page_dir).exists() {
        fs::create_dir_all(error_page_dir)
            .map_err(|e| format!("Failed to create error page directory: {}", e))?;
    }

    // Write error page using embedded template
    fs::write(&error_page_path, ERROR_HTML)
        .map_err(|e| format!("Failed to write error page: {}", e))?;

    Ok(())
}

/// Replace the default nginx welcome page with XyNginC index
fn ensure_index_page_exists() -> Result<(), String> {
    let index_page_path = "/var/www/html/index.html";  // Fixed path
    let default_nginx_index = "/var/www/html/index.nginx-debian.html";

    // Remove default nginx welcome page
    if Path::new(default_nginx_index).exists() {
        fs::remove_file(default_nginx_index)
            .map_err(|e| format!("Failed to remove default nginx index: {}", e))?;
    }

    // Create XyNginC index page using embedded template
    let index_html = generate_index_html();
    fs::write(index_page_path, index_html)
        .map_err(|e| format!("Failed to write index page: {}", e))?;

    Ok(())
}
```

### Template Updates

#### Updated Nginx Templates

Fixed all nginx templates to use correct error page paths:

**non_ssl_template.conf**:

```nginx
# Serve error page
location = /error.html {
    root /var/www/html/errors;  # Fixed: was /var/www/xynginc
    internal;
}
```

**ssl_template.conf**:

```nginx
# Serve error page (both HTTP and HTTPS sections)
location = /error.html {
    root /var/www/html/errors;  # Fixed: was /var/www/xynginc
    internal;
}
```

### Embedded Template Constants

```rust
// Embedded templates - included directly in the binary
const NON_SSL_TEMPLATE: &str = include_str!("configs/non_ssl_template.conf");
const SSL_TEMPLATE: &str = include_str!("configs/ssl_template.conf");
const ERROR_HTML: &str = include_str!("configs/error.html");
const INDEX_HTML: &str = include_str!("configs/index.html");  // NEW
```

## User Experience Improvements

### Before Fix (v1.1.4)

```bash
$ sudo xynginc apply --config domains.json
🌐 Processing: server.nehonix.xyz
   ✓ Config written to /etc/nginx/sites-available/server.nehonix.xyz
🧪 Testing nginx configuration...
❌ Configuration test failed!
# nginx: [emerg] unexpected "{" in /etc/nginx/sites-available/server.nehonix.xyz:10
```

### After Fix (v1.1.5)

```bash
$ sudo xynginc apply --config domains.json
🌐 Processing: server.nehonix.xyz
   > Generating nginx configuration for server.nehonix.xyz
   ✓ Config written to /etc/nginx/sites-available/server.nehonix.xyz
   > Setting up web pages...
   📁 Creating error page directory: /var/www/html/errors
   📝 Writing error page HTML...
   ✓ Error page created at /var/www/html/errors/error.html
   > Setting up XyNginC index page
   🗑️  Removing default nginx welcome page
   📝 Creating XyNginC index page
   ✓ XyNginC index page created at /var/www/html/index.html
🧪 Testing nginx configuration...
✓ Configuration is valid
✅ Configuration applied successfully!
```

### Web Directory Structure (Before vs After)

**Before v1.1.5:**

```
/var/www/
├── html/
│   └── index.nginx-debian.html  # Default nginx welcome page
└── xynginc/
    └── error.html               # Non-standard location
```

**After v1.1.5:**

```
/var/www/
└── html/
    ├── index.html               # XyNginC branded page (replaces default)
    ├── index.nginx-debian.html  # Removed
    └── errors/
        └── error.html           # Standard location for error pages
```

## Installation

```bash
npm install xynginc@1.1.5
```

### Binary Update

This release includes an updated Rust binary with the following improvements:

- ✅ **Fixed Template Variable Replacement**: All variables now properly replaced
- ✅ **Standard Web Directory Structure**: Uses `/var/www/html/` for all web files
- ✅ **Automatic Index Page Replacement**: Replaces default nginx welcome page
- ✅ **Enhanced Web Page Management**: Comprehensive setup of error and index pages
- ✅ **Complete Template Embedding**: All templates embedded including new index.html
- ✅ **Production Ready**: Suitable for production deployments

## Verification

Verify that all fixes work correctly:

```bash
# Check system requirements
sudo xynginc check

# Test configuration generation (this should work now)
echo '{"domains":[{"domain":"test.example.com","port":3000,"ssl":false}],"auto_reload":false}' | sudo xynginc apply --config -

# Verify generated configuration has proper variables replaced
cat /etc/nginx/sites-available/test.example.com

# Check that web pages are created in correct locations
ls -la /var/www/html/
ls -la /var/www/html/errors/

# Verify index page content
curl http://localhost/  # Should show XyNginC branded page
```

## Compatibility

- **Backward Compatible**: This release is fully backward compatible with v1.1.4 and earlier
- **Configuration Compatible**: Existing configurations work without changes
- **API Unchanged**: All function signatures and behavior remain the same
- **Template Compatible**: Existing templates continue to work with fixed variable replacement

## Upgrade Path

### From v1.1.4 to v1.1.5 (Critical Fix)

1. Update the npm package:

   ```bash
   npm install xynginc@1.1.5
   ```

2. The binary will be automatically downloaded with bug fixes

3. Test that the template replacement works:

   ```bash
   sudo xynginc check
   sudo xynginc test
   ```

4. Verify configuration generation works:
   ```bash
   echo '{"domains":[{"domain":"test.example.com","port":3000,"ssl":false}]}' | sudo xynginc apply --config -
   ```

### From v1.1.0-v1.1.3 to v1.1.5

Follow the same upgrade path, benefiting from all fixes:

- Template embedding from v1.1.2
- Interactive installation from v1.1.1
- Template variable replacement fix from v1.1.5
- Web directory structure fix from v1.1.5

## Performance Impact

- **Template Processing**: Improved - variables are now properly replaced
- **Web Page Creation**: Enhanced - uses standard directory structure
- **Binary Size**: Slightly increased due to embedded index.html template
- **Runtime Performance**: Improved - no more configuration generation failures
- **Overall**: Significant improvement in reliability and functionality

## Security

- **No Security Changes**: This release focuses on bug fixes without security modifications
- **Template Integrity**: All embedded templates are compiled into the binary
- **Web Directory Security**: Uses standard `/var/www/html/` permissions
- **Existing Security**: All security features from previous versions remain intact

## Known Issues

**None** - This release specifically addresses all known template and web directory issues. All functionality works as expected.

## Environment Variables

No changes to environment variable handling in this release.

## What's Next

Future releases will focus on:

- Enhanced SSL certificate management with interactive prompts
- Load balancing capabilities with dynamic configuration
- Advanced monitoring and metrics collection
- Docker container support for isolated environments
- Performance optimizations for high-traffic scenarios
- Web-based management interface

## Support

If you encounter any issues with this release:

1. Ensure you're using the latest binary: `npm install xynginc@1.1.5`
2. Check system requirements: `sudo xynginc check`
3. Test template replacement: `sudo xynginc test`
4. Verify web page creation: Check `/var/www/html/` directory structure
5. For template-related issues, verify the binary version: `sudo xynginc --version`

---

**Critical update highly recommended** for all users to resolve template variable replacement issues and ensure proper nginx configuration generation!
