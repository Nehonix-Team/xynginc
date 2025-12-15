# v1.0.5 - Update Release

This release includes several improvements and fixes to the XyNginC (XyPriss Nginx Controller).

## 🔧 Changes

- **Package Name Update**: Standardized package name to `xynginc` for consistency.
- **Download URL Fix**: Corrected download URLs for binary artifacts.
- **Documentation Updates**: Improved README and installation documentation.
- **Project Configuration**: Enhanced initial project setup and build scripts.

## 📦 Installation

```bash
npm install xynginc@1.0.5
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