# v1.0.0 - Initial Release

We are excited to announce the first stable release of **XyNginC** (XyPriss Nginx Controller)! 🚀

XyNginC is a production-grade tool designed to simplify Nginx and SSL management for XyPriss applications. It automates the complex tasks of reverse proxy configuration and certificate management, allowing developers to focus on their code rather than infrastructure.

## 🌟 Key Features

- **Automated Reverse Proxy**: Seamlessly maps domains to local ports.
- **One-Command SSL**: Integrated Let's Encrypt and Certbot support for automatic HTTPS.
- **Zero-Config Nginx**: Generates optimized, production-ready Nginx configurations automatically.
- **Multi-Domain Support**: Manage multiple domains and subdomains from a single configuration.
- **High Performance**: Core logic is powered by a lightweight, high-speed Rust binary.
- **Type Safety**: Built with TypeScript, offering full type definitions and autocompletion.

## 📦 Installation

```bash
npm install xynginc
```

## 🚀 Quick Start

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

## 🛠️ Assets

This release includes the following binary artifacts:

- `xynginc-linux-x64`: For 64-bit Linux systems (Intel/AMD).
- `xynginc-linux-arm64`: For 64-bit ARM Linux systems.

> **Note**: The npm package automatically downloads the appropriate binary for your system during installation.
