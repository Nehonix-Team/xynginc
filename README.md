# XyNginC

This project uses code developed by NEHONIX (www.nehonix.com) under the NEHONIX Open Source License (NOSL) v1.0.

XyPriss Nginx Controller - Simplifie la gestion de Nginx et SSL.

[![xfpm version](https://badge.fury.io/js/%40xypriss%2Fxynginc.svg)](https://www.npmjs.com/package/xynginc)
[![License: NOSL](https://img.shields.io/badge/License-NOSL-blue.svg)](https://dll.nehonix.com/licenses/NOSL)

## Overview

XyNginC (XyPriss Nginx Controller) automates Nginx reverse proxy configuration, SSL certificate management, and provides optimized, production-ready configs for security, performance, and best practices. It eliminates manual Nginx editing, simplifying XyPriss deployment to just a few lines of TypeScript. Check out the [demo project on GitHub](https://github.com/iDevo-ll/XYNC-Demo).

> [!IMPORTANT]
> XyNginC is a plugin **exclusively designed for XyPriss projects**. It is not intended for use outside the XyPriss ecosystem, in development environments, or on non-Linux systems. For the best integration experience, **XFPM** (XyPriss Fast Package Manager) is required — see [Installation](#installation).

> [!CAUTION]
> XyNginC only runs on **Linux production servers** (VPS or Dedicated). Supported architectures: **x64**, **arm64**, and **ia32**. Windows and macOS are not supported.

## Key Features

- **Automated Reverse Proxy**: Maps domains to local ports seamlessly.
- **One-Command SSL**: Integrated Let's Encrypt and Certbot support for automatic HTTPS.
- **Automatic Nginx Reload**: Applies configuration changes without manual service restarts.
- **Multi-Domain Support**: Manages multiple domains and subdomains within a single configuration.
- **Optimized Configuration**: Generates production-ready Nginx configuration files via dynamic GitHub template fetching.
- **High Performance**: Core logic executed via a Go-based CLI for speed and reliability.
- **Type Safety**: Full TypeScript support with comprehensive type definitions.

## Installation

For detailed installation instructions, please refer to the [Installation Guide](docs/INSTALLATION.md).
For building from source (custom architectures), see the [Build Guide](docs/BUILD_FROM_SOURCE.md).

XyNginC is exclusively designed for **XyPriss projects running in production on Linux**. We strongly recommend using Ubuntu on a Virtual Private Server (VPS) for the best security and stability.

### Prerequisites

Before installing XyNginC, you need **XFPM** — the official package manager for the XyPriss ecosystem.

```bash
curl -fsSL https://raw.githubusercontent.com/Nehonix-Team/XFPM/master/scripts/install.sh | sudo bash
```

```bash
xfpm --version
```

### Install XyNginC

```bash
xfpm install xynginc
```

> [!NOTE]
> The installation requires `sudo` privileges to place the binary in `/usr/local/bin` and configure system permissions.

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
});
```

### Dynamic Management

Manage domains programmatically at runtime:

```typescript
app.start(undefined, async () => {
  // Add a new domain
  await app.xynginc.addDomain(
    "new.example.com",
    4000,
    true,
    "admin@example.com",
  );

  // List configured domains
  const domains = await app.xynginc.listDomains();
  console.log("Configured domains:", domains);

  // Validate XCNP configuration
  const isValid = await app.xynginc.test();

  // Reload XyNginC service
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

  /** Custom path to the xynginc binary - recommended */
  binaryPath?: string;

  /** Automatically download the binary if missing (default: true) */
  autoDownload?: boolean;

  /** Specific GitHub release version to download (default: "latest") */
  version?: string;

  /** Password to execute sudo commands silently in background environments like PM2 */
  sudoPassword?: string;

  /** Automatically open Port 80 and 443 in the firewall (UFW) if they are blocked (default: false) */
  autoFixFirewall?: boolean;
}
```

### Server Methods

The following methods are exposed on `server.xynginc` after the plugin is registered:

```typescript
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

// Reload XyNginC service
await server.xynginc.reload(): Promise<void>

// Test XyNginC configuration validity
await server.xynginc.test(): Promise<boolean>

// Get status of managed sites
await server.xynginc.status(): Promise<string>
```

## CLI Usage

The `xynginc` command-line interface allows for direct management without the XyPriss application context.

```bash
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
  "auto_reload": true,
  "autofix_firewall": true
}
```

## Architecture

The system operates through a three-tier architecture:

1. **XyPriss Application**: The XyPriss application running the server.
2. **XyNginC Plugin**: A TypeScript wrapper that interfaces with the application and executes the underlying Go binary.
3. **XyNginC Go Binary**: A high-performance Go-based CLI tool that performs system-level operations (Nginx configuration, Certbot execution). It dynamically fetches the latest config templates from GitHub (`Nehonix-Team/xynginc`) to guarantee up-to-date and optimized Nginx setups.

## Security Considerations

XyNginC requires elevated privileges to perform the following actions:

- Writing to `/etc/nginx/sites-available/`
- Creating symbolic links in `/etc/nginx/sites-enabled/`
- Executing `certbot` for SSL certificate generation
- Reloading the Nginx service

> [!WARNING]
> XyNginC requires elevated privileges to manage Nginx and firewall rules. Always review the commands being executed and restrict sudo access to only what is necessary.

- **Sudoers**: We recommend configuring `sudoers` to allow specific commands for the user.
- **Sudo Password**: If your application runs in the background (e.g. using PM2), you can use the `sudoPassword` option or the `SUDO_PASSWORD` environment variable.
  - **Reassurance**: The password is used **only** to execute the local `xynginc` binary and system commands like `ufw` or `service`. It is never stored, logged, or transmitted outside of your server.
- **Firewall**: The `autoFixFirewall` option allows XyNginC to automatically detect and open Port 80/443 if `ufw` is active. This is useful for automated SSL validation.

## Troubleshooting

### Binary Not Found

If the binary fails to download automatically:

```bash
# Manually trigger with XFPM
xfpm run postinstall

# Or specify the path manually in options
XNCP({
  binaryPath: "/usr/local/bin/xynginc",
  autoDownload: false,
})
```

### Permission Denied

If you encounter permission errors:

```bash
# Run with sudo
sudo node server.js
```

### Certbot Failure

If SSL generation fails:

1. Verify DNS propagation: `dig api.example.com`
2. Ensure firewall allows traffic on ports 80 and 443:

```bash
sudo ufw allow 80
sudo ufw allow 443
```

> [!NOTE]
> Certbot requires ports 80 and 443 to be publicly accessible for domain validation. Make sure your cloud provider's security groups also allow this traffic, not just UFW.

## Contributing

Contributions are welcome. Please follow the standard pull request process.

1. Clone the repository
2. Install dependencies
3. Build the Go CLI and TypeScript package (`xfpm run build:all`)
4. Run tests

## License

NOSL
