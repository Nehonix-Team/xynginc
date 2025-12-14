# XyNginC (XyPriss Nginx Controller)

Official XyPriss plugin for managing Nginx reverse proxies and SSL certificates automatically.

## Architecture

This project uses a hybrid architecture:

- **TypeScript Plugin**: Integrates with XyPriss lifecycle (`src/`).
- **Rust Binary**: Handles system-level operations (`core/`).

## Installation

```bash
npm install @xypriss/xynginc
```

## Usage

```typescript
import { createServer, Plugin } from "xypriss";
import XNCP from "@xypriss/xynginc";

const app = createServer({
  // ...
});

Plugin.exec(
  XNCP({
    domains: [
      {
        domain: "api.example.com",
        port: 3000,
        ssl: true,
        email: "admin@example.com",
      },
    ],
  })
);
```

## Development

1. Build the Rust binary:

   ```bash
   cd core
   cargo build --release
   ```

2. Build the TypeScript plugin:
   ```bash
   npm install
   npm run build
   ```
