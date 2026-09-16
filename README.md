# XyNginC (XNCP) — XyPriss Nginx & Infrastructure Controller

[![XyPriss Plugin](https://img.shields.io/badge/XyPriss%20Plugin-Official-00e5a0?style=flat-square)](https://xypriss.nehonix.com/docs/plugins/official/xynginc)
[![License: NOSL](https://img.shields.io/badge/License-NOSL-blue.svg?style=flat-square)](https://dll.nehonix.com/licenses/NOSL)
[![Go Core](https://img.shields.io/badge/Go%20Core-go--ed--1.1.20-00ADD8?logo=go&logoColor=white&style=flat-square)](https://github.com/Nehonix-Team/xynginc/releases)

XyNginC (XNCP) is an enterprise-grade infrastructure controller designed for the XyPriss web ecosystem. It automates Nginx reverse-proxy generation, Let's Encrypt SSL/TLS lifecycle management with multi-domain certificate grouping, and systemd service orchestration.

---

## Official Documentation

For complete technical specifications, architecture guides, and API references, please visit the official documentation:

- **Web Documentation**: [https://xypriss.nehonix.com/docs/plugins/official/xynginc](https://xypriss.nehonix.com/docs/plugins/official/xynginc)
- **Local Documentation**: [docs/README.md](./docs/README.md)

---

## Architecture & Features

- **Automated Reverse Proxy**: Dynamic Nginx site configuration generation tailored for XyPriss Multi-Server (XMS) applications.
- **Grouped Multi-Domain SSL**: Grouped Let's Encrypt TLS certificate generation (`certbot`) with single-cycle Nginx reloads.
- **SSL Rate-Limit Cooldown Registry**: Automatic caching and handling of Let's Encrypt rate limits to ensure zero boot delay.
- **Systemd Service Orchestration**: Background service management via `xfpmx xncp deploy`.
- **Pre-flight System Diagnostics**: Requirements verification for Nginx, Certbot, modules, and system permissions via `xfpmx xncp check`.
- **Multi-Architecture Support**: Cross-platform Go binary execution (`x64`, `arm64`, `ia32`).

---

## Quick Start

### 1. Package Installation
Install XyNginC in your XyPriss project using `xfpm`:

```bash
xfpm add xynginc
```

### 2. Service Deployment
Deploy and launch your background systemd service:

```bash
sudo xfpmx xncp deploy
```

### 3. Service Management & Status
Inspect service health and real-time execution logs:

```bash
# Check service and domain status
xfpmx xncp status

# Follow real-time application logs
xfpmx xncp logs -f

# Verify system requirements
xfpmx xncp check
```

---

## License

Distributed under the **NEHONIX Open Source License (NOSL)** v1.0. See [LICENSE](https://dll.nehonix.com/licenses/NOSL) for details.

Copyright (c) 2026 NEHONIX Engineering. All rights reserved.
