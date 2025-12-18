# XyNginC v1.4.5 - Custom Upload Limits

**Release Date**: December 18, 2025  
**Version**: 1.4.5

## Overview

XyNginC v1.4.5 adds support for custom `client_max_body_size` in Nginx, allowing for larger file uploads and preventing HTTP 413 errors.

## Key Changes

### 1. Max Body Size Support

You can now define the maximum allowed size for client request bodies. This is useful for applications that handle large file uploads.

### 2. CLI Flag

New flag added to the `add` command:

```bash
sudo xynginc add --domain example.com --port 8080 --max-body-size 100M
```

### 3. Plugin Configuration

The TypeScript plugin now supports `maxBodySize`:

```typescript
{
  domain: "example.com",
  port: 8080,
  maxBodySize: "50M"
}
```

## Installation

```bash
cd release/v1.4.5
sudo ./install.sh
```

For more details, see [RELEASE_NOTES.md](RELEASE_NOTES.md).
