# XyNginC - XyPriss Nginx Controller

> Production-grade Nginx and SSL management for XyPriss applications with automatic requirements installation, embedded templates, template variable replacement fix, and custom error pages.

[![npm version](https://badge.fury.io/js/%40xypriss%2Fxynginc.svg)](https://www.npmjs.com/package/xynginc)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

XyNginC (XyPriss Nginx Controller) streamlines the deployment of XyPriss applications by automating Nginx reverse proxy configuration and SSL certificate management. It eliminates the need for manual Nginx configuration editing and simplifies the production setup process into a few lines of TypeScript.

## What's New in v1.1.5

**Critical Bug Fixes and Complete Template System:**

- **Template Variable Replacement Fixed**: Resolved the critical bug where template variables like `{{DOMAIN_NAME}}` were not being replaced in generated nginx configurations
- **Correct Web Directory Structure**: Fixed error page and index page paths to use standard `/var/www/html/` directory structure
- **Automatic Index Page Replacement**: Automatically replaces default nginx welcome page with XyNginC branded index page
- **Complete Template Embedding**: All templates (nginx configs, error pages, index page) embedded directly in binary
- **Enhanced Web Page Management**: Comprehensive setup of both error pages and index pages for production-ready deployment

### Template Variable Replacement Fix

**Problem (v1.1.4 and earlier):**
Generated nginx configs contained literal template variables:

```nginx
server {
    listen 80;
    server_name {{DOMAIN_NAME}};  # ❌ Not replaced!
    proxy_pass http://{{BACKEND_HOST}}:{{BACKEND_PORT}};  # ❌ Not replaced!
}
```

**Solution (v1.1.5):**
Template variables are now properly replaced:

```nginx
server {
    listen 80;
    server_name server.nehonix.xyz;  # ✅ Properly replaced
    proxy_pass http://localhost:9837;  # ✅ Properly replaced
}
```

### Web Directory Structure Improvements

**Before (v1.1.4):**

- Error pages: `/var/www/xynginc/error.html` (non-standard)
- Index page: Not replaced (showed nginx default)

**After (v1.1.5):**

- Error pages: `/var/www/html/errors/error.html` (standard location)
- Index page: `/var/www/html/index.html` (replaces nginx default)
- Automatic cleanup of default nginx files

## What's New in v1.1.4

**Template Embedding and Interactive Installation:**

- **Embedded Templates**: All configuration templates embedded directly in the binary for reliable deployment
- **Interactive Package Installation**: Real-time feedback during nginx and certbot installation
- **No External Dependencies**: Templates work immediately after installation without filesystem access

## What's New in v1.1.1

**Major Improvements to Interactive Installation:**

- **Interactive Package Installation**: Now uses `Stdio::inherit()` instead of `.output()` for full user interaction during package installation
- **Real-time Package Management**: Users can now see package download progress, confirmation prompts, and error messages in real-time
- **Single-Command Installation**: All packages are installed in a single `apt-get install` command for better efficiency
- **Enhanced Error Recovery**: Automatic APT repository issue detection and fixing with retry mechanisms

## Key Features

- **Automated Reverse Proxy**: Maps domains to local ports seamlessly.
- **One-Command SSL**: Integrated Let's Encrypt and Certbot support for automatic HTTPS.
- **Interactive Package Installation**: Real-time feedback during nginx and certbot installation
- **Fixed Template Variable Replacement**: Proper replacement of all template variables in nginx configurations
- **Standard Web Directory Structure**: Uses `/var/www/html/` for all web files
- **Automatic Index Page**: Replaces default nginx welcome page with XyNginC branded page
- **Embedded Templates**: Self-contained binary with all configuration templates included
- **Automatic Nginx Reload**: Applies configuration changes without manual service restarts.
- **Multi-Domain Support**: Manages multiple domains and subdomains within a single configuration.
- **Optimized Configuration**: Generates production-ready Nginx configuration files.
- **High Performance**: Core logic executed via a Rust-based CLI for speed and reliability.
- **Type Safety**: Full TypeScript support with comprehensive type definitions.
- **Custom Error Pages**: Professional styled error pages for 404, 502, and other HTTP errors
- **Smart Requirements Check**: Automatic detection and installation of missing dependencies
- **APT Repository Management**: Automatic detection and fixing of common APT repository issues with interactive recovery

## Installation

Install the package via npm:

```bash
npm install xynginc
```

The necessary binary for your architecture (Linux x64/arm64) will be downloaded automatically during installation.

### Quick Auto-Setup

If you don't have nginx or certbot installed, XyNginC can install them automatically with full interactivity:

```bash
# Interactive installation with real-time feedback
sudo xynginc install
```

Or enable automatic installation in your plugin configuration:

```typescript
XNCP({
  domains: [
    {
      domain: "api.example.com",
      port: 3000,
      ssl: true,
      email: "admin@example.com",
    },
  ],
  installRequirements: true, // Auto-install missing dependencies with full interactivity
  autoDownload: true,
});
```

### Prerequisites

- **Operating System**: Linux (Ubuntu/Debian/Kali recommended)
- **Node.js**: Version 18.0.0 or higher
- **Nginx**: Will be auto-installed if missing (or `sudo apt install nginx`)
- **Certbot**: Will be auto-installed if missing (or `sudo apt install certbot python3-certbot-nginx`)

Verify the installation and prerequisites:

```bash
sudo xynginc check
```

## Interactive Installation Experience

The v1.1.1+ releases provide a significantly improved installation experience:

### Real-time Feedback

Users now see:

- Package download progress in real-time
- APT confirmation prompts directly in the terminal
- Error messages as they occur
- Sudo password prompts when needed
- Detailed installation logs

### Smart Error Recovery

If `apt-get update` fails:

- Automatic detection of APT repository issues
- Automatic execution of `fix_apt_repositories()`
- Retry mechanism after repository fixes
- Continued installation process

### Efficient Package Management

- All packages installed in a single command
- Reduced installation time
- Better dependency resolution
- Atomic installation process

## Quick Start

### Basic Configuration

Integrate XyNginC into your XyPriss server:

```typescript
import { createServer } from "xypriss";
import XNCP from "xynginc";

const app = createServer({
  plugins: {
    register: [
      XNCP({
        domains: [
          {
            domain: "api.example.com",
            port: 3000,
            ssl: true,
            email: "admin@example.com",
          },
        ],
        installRequirements: true, // Auto-setup mode with full interactivity
      }),
    ],
  },
});

app.start();
```

### Multiple Domains Configuration

Configure multiple environments or services simultaneously:

```typescript
XNCP({
  domains: [
    {
      domain: "api.example.com",
      port: 3000,
      ssl: true,
      email: "admin@example.com",
    },
    {
      domain: "admin.example.com",
      port: 3001,
      ssl: true,
      email: "admin@example.com",
    },
    {
      domain: "dev.example.com",
      port: 3002,
      ssl: false,
    },
  ],
  autoReload: true,
  installRequirements: true, // Enable auto-setup with interactive installation
});
```

### Dynamic Management

Manage domains programmatically at runtime:

```typescript
app.on("ready", async () => {
  // Install requirements if needed (with full interactivity)
  await app.xynginc.installRequirements();

  // Add a new domain
  await app.xynginc.addDomain(
    "new.example.com",
    4000,
    true,
    "admin@example.com"
  );

  // List configured domains
  const domains = await app.xynginc.listDomains();
  console.log("Configured domains:", domains);

  // Validate Nginx configuration
  const isValid = await app.xynginc.test();

  // Reload Nginx service
  await app.xynginc.reload();

  // Remove a domain
  await app.xynginc.removeDomain("old.example.com");
});
```

## API Reference

### Plugin Options

```typescript
interface XyNginCPluginOptions {
  /** List of domains to configure */
  domains: Array<{
    domain: string; // Domain name (e.g., api.example.com)
    port: number; // Local port to proxy (e.g., 3000)
    ssl?: boolean; // Enable SSL via Let's Encrypt
    email?: string; // Email for Let's Encrypt registration (required if ssl=true)
  }>;

  /** Automatically reload Nginx after configuration changes (default: true) */
  autoReload?: boolean;

  /** Automatically install system requirements if missing (default: false) */
  installRequirements?: boolean;

  /** Custom path to the xynginc binary - recommended */
  binaryPath?: string;

  /** Automatically download the binary if missing (default: true) */
  autoDownload?: boolean;

  /** Specific GitHub release version to download (default: "latest") */
  version?: string;
}
```

### Server Methods

The following methods are exposed on `server.xynginc` after the plugin is registered:

```typescript
// Install system requirements (nginx, certbot, directories) - now fully interactive
await server.xynginc.installRequirements(): Promise<void>

// Add a domain configuration
await server.xynginc.addDomain(
  domain: string,
  port: number,
  ssl: boolean,
  email?: string
): Promise<void>

// Remove a domain configuration
await server.xynginc.removeDomain(domain: string): Promise<void>

// List all configured domains
await server.xynginc.listDomains(): Promise<string[]>

// Reload Nginx service
await server.xynginc.reload(): Promise<void>

// Test Nginx configuration validity
await server.xynginc.test(): Promise<boolean>

// Get status of managed sites
await server.xynginc.status(): Promise<string>
```

## CLI Usage

The `xynginc` command-line interface allows for direct management without the Node.js application context.

```bash
# Install system requirements with full interactivity
sudo xynginc install

# Check prerequisites
sudo xynginc check

# Add a domain
sudo xynginc add --domain api.example.com --port 3000 --ssl --email admin@example.com

# List domains
sudo xynginc list

# Apply configuration from a JSON file
sudo xynginc apply --config config.json

# Test Nginx configuration
sudo xynginc test

# Reload Nginx
sudo xynginc reload

# Remove a domain
sudo xynginc remove api.example.com

# View status
sudo xynginc status
```

### Configuration File Example

```json
{
  "domains": [
    {
      "domain": "api.example.com",
      "port": 3000,
      "ssl": true,
      "email": "admin@example.com"
    }
  ],
  "auto_reload": true
}
```

## Custom Error Pages

XyNginC includes professional, custom-styled error pages that replace the default nginx error messages:

### Features:

- **Modern Design**: TailwindCSS-styled professional error pages
- **Responsive**: Works perfectly on desktop and mobile
- **Interactive**: Auto-refresh suggestions for temporary errors (502, 503, 504)
- **Branded**: Consistent XyNginC branding and French localization
- **Dynamic**: Automatically detects and displays the correct HTTP error code
- **Standard Location**: Stored in `/var/www/html/errors/error.html` for standard web directory structure

### Error Types Handled:

- **404** - Page not found
- **500** - Internal server error
- **502** - Bad Gateway)
- **503 (backend service down** - Service unavailable
- **504** - Gateway timeout
- And many others with helpful messaging

## Custom Index Page

XyNginC automatically replaces the default nginx welcome page with a professional branded index page:

### Features:

- **Professional Design**: Modern TailwindCSS-styled landing page
- **XyNginC Branding**: Consistent branding and visual identity
- **Status Indication**: Clear indication that the server is running successfully
- **Responsive**: Works perfectly on desktop and mobile
- **Automatic Replacement**: Replaces `index.nginx-debian.html` automatically

### What It Replaces:

The default nginx page:

```
Welcome to nginx!
If you see this page, the nginx web server is successfully installed and working...
```

With XyNginC branded page showing:

- XyNginC logo and branding
- "Server is running successfully!" status
- Version information
- Professional, modern design

## APT Repository Management

XyNginC includes intelligent APT repository management to handle common package management issues:

### Automatic Detection and Fixing:

- **Repository Error Detection**: Automatically identifies common APT repository problems
- **Smart Repository Management**: Disables problematic repositories while preserving official sources
- **Backup and Recovery**: Creates backups before making changes
- **Kali Linux Support**: Ensures proper Kali rolling repository configuration
- **PostgreSQL/Docker/LLVM Handling**: Safely disables problematic third-party repositories

### Automatic Fixes Applied:

- Backs up current repository configuration
- Disables problematic repositories (PostgreSQL, Docker, LLVM)
- Ensures Kali official repositories are present
- Updates package lists after fixes
- Provides rollback instructions

## Architecture

The system operates through a three-tier architecture:

1.  **XyPriss Application**: The Node.js application running the server.
2.  **XyNginC Plugin**: A TypeScript wrapper that interfaces with the application and manages the binary.
3.  **XyNginC Binary**: A Rust-based CLI tool that performs system-level operations with embedded templates and fixed variable replacement.

## Security Considerations

XyNginC requires elevated privileges to perform the following actions:

- Writing to `/etc/nginx/sites-available/`
- Creating symbolic links in `/etc/nginx/sites-enabled/`
- Executing `certbot` for SSL certificate generation
- Reloading the Nginx service
- Installing system packages (nginx, certbot) with full interactivity
- Writing to `/var/www/html/` for web pages

**Note**: Ensure your XyPriss server is started with `sudo` privileges when using this plugin, or configure `sudoers` to allow specific commands for the user.

## Troubleshooting

### Binary Not Found

If the binary fails to download automatically:

```bash
# Manually trigger postinstall
npm run postinstall

# Or specify the path manually in options
XNCP({
  binaryPath: "/usr/local/bin/xynginc",
  autoDownload: false,
})
```

### Template Variable Issues (Fixed in v1.1.5)

**v1.1.5+**: No longer applicable - template variables are properly replaced.

**v1.1.4 and earlier**: If you encounter template variables not being replaced:

```bash
# Ensure the binary is properly installed
sudo xynginc check

# Test configuration generation
sudo xynginc test
```

### Web Directory Issues

If web pages are not created correctly:

```bash
# Check that /var/www/html exists and has proper permissions
ls -la /var/www/html/

# Manually trigger web page setup
sudo xynginc check
```

### Interactive Installation Issues

If you encounter issues with the interactive installation:

```bash
# Run installation manually with full output
sudo xynginc install

# Check specific requirements
sudo xynginc check

# For non-interactive environments, set environment variable
XYNC_INSTALL_MODE=non-interactive sudo xynginc install
```

### Permission Denied

If you encounter permission errors:

```bash
# Run with sudo
sudo node server.js
```

### Certbot Failure

If SSL generation fails:

1.  Verify DNS propagation: `dig api.example.com`
2.  Ensure firewall allows traffic on ports 80 and 443:
    ```bash
    sudo ufw allow 80
    sudo ufw allow 443
    ```

## Contributing

Contributions are welcome. Please follow the standard pull request process.

1.  Clone the repository
2.  Install dependencies
3.  Build the Rust CLI and TypeScript package
4.  Run tests

## License

MIT
