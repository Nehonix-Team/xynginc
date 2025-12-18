# XyNginC

This project uses code developed by NEHONIX (www.nehonix.com) under the NEHONIX Open Source License (NOSL) v1.0.

XyPriss Nginx Controller - Simplifie la gestion de Nginx et SSL.

[![npm version](https://badge.fury.io/js/%40xypriss%2Fxynginc.svg)](https://www.npmjs.com/package/xynginc)
[![License: NOSL](https://img.shields.io/badge/License-NOSL-blue.svg)](https://dll.nehonix.com/licenses/NOSL)

## Overview

XyNginC (XyPriss Nginx Controller) automates Nginx reverse proxy configuration, SSL certificate management, and provides optimized, production-ready configs for security, performance, and best practices. It eliminates manual Nginx editing, simplifying XyPriss deployment to just a few lines of TypeScript. Check out the [demo project on GitHub](https://github.com/iDevo-ll/XYNC-Demo).

## Key Features

- **Automated Reverse Proxy**: Maps domains to local ports seamlessly.
- **One-Command SSL**: Integrated Let's Encrypt and Certbot support for automatic HTTPS.
- **Automatic Nginx Reload**: Applies configuration changes without manual service restarts.
- **Multi-Domain Support**: Manages multiple domains and subdomains within a single configuration.
- **Optimized Configuration**: Generates production-ready Nginx configuration files.
- **High Performance**: Core logic executed via a Rust-based CLI for speed and reliability.
- **Type Safety**: Full TypeScript support with comprehensive type definitions.

## Installation

For detailed installation instructions, please refer to the [Installation Guide](docs/INSTALLATION.md).
For building from source (custom architectures), see the [Build Guide](docs/BUILD_FROM_SOURCE.md).

XyNginC is designed for production environments running on Linux. We strongly recommend using Ubuntu on a Virtual Private Server (VPS) for the best security and stability.

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
app.on("ready", async () => {
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

## Architecture

The system operates through a three-tier architecture:

1.  **XyPriss Application**: The Node.js application running the server.
2.  **XyNginC Plugin**: A TypeScript wrapper that interfaces with the application and manages the binary.
3.  **XyNginC Binary**: A Rust-based CLI tool that performs system-level operations (Nginx configuration, Certbot execution).

## Security Considerations

XyNginC requires elevated privileges to perform the following actions:

- Writing to `/etc/nginx/sites-available/`
- Creating symbolic links in `/etc/nginx/sites-enabled/`
- Executing `certbot` for SSL certificate generation
- Reloading the Nginx service

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
