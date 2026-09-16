# XyNginC (XNCP) — XyPriss Nginx & Infrastructure Controller

[![XyPriss Plugin](https://img.shields.io/badge/XyPriss%20Plugin-Official-00e5a0?style=flat-square)](https://xypriss.nehonix.com/docs/plugins/official/xynginc)
[![License: NOSL](https://img.shields.io/badge/License-NOSL-blue.svg?style=flat-square)](https://dll.nehonix.com/licenses/NOSL)
[![Go Core](https://img.shields.io/badge/Go%20Core-go--ed--1.1.20-00ADD8?logo=go&logoColor=white&style=flat-square)](https://github.com/Nehonix-Team/xynginc/releases)

**XyNginC (XNCP)** is the enterprise-grade infrastructure controller for the XyPriss web ecosystem. It provides automated Nginx reverse-proxy configuration, Let's Encrypt SSL/TLS lifecycle management, multi-domain certificate grouping, and systemd service orchestration.

---

## 📖 Official Documentation

For comprehensive guides, API references, architecture overviews, and configuration options, visit the official XyPriss Documentation:

🌐 **[XyNginC Official Documentation](https://xypriss.nehonix.com/docs/plugins/official/xynginc)**

Local documentation is also available in the [`docs/`](./docs/README.md) directory.

---

## ✨ Key Features

- **Automated Reverse Proxy**: Instant Nginx site configuration generation for XyPriss multi-server applications (XMS).
- **Grouped Multi-Domain SSL**: Grouped Let's Encrypt TLS certificate generation (`certbot`) with single Nginx reloads.
- **SSL Rate-Limit Cooldown Registry**: Automatic detection and caching of Let's Encrypt rate limits to prevent boot delays.
- **Systemd Service Orchestration**: Zero-downtime background service management via `xfpmx xncp deploy`.
- **Pre-flight System Diagnostics**: Instant requirement checks for Nginx, Certbot, headers-more module, and permissions via `xfpmx xncp check`.
- **Multi-Arch Binary Distribution**: Cross-platform Go binary execution (`x64`, `arm64`, `ia32`).

---

## 🚀 Quick Start

### 1. Installation via `xfpm`
```bash
xfpm add xynginc
```

### 2. Service Deployment
Inside your XyPriss project directory:
```bash
sudo xfpmx xncp deploy
```

### 3. Service Management & Status
```bash
# Check status
xfpmx xncp status

# Follow real-time logs
xfpmx xncp logs -f

# Verify requirements
xfpmx xncp check
```

---

## 📄 License

Distributed under the **NEHONIX Open Source License (NOSL)** v1.0. See [LICENSE](https://dll.nehonix.com/licenses/NOSL) for details.

Developed & Maintained by [NEHONIX Engineering](https://www.nehonix.com).
