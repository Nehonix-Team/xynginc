# v1.0.7 - Major Feature Release

This is a major feature release of XyNginC (XyPriss Nginx Controller) that introduces automatic requirements installation and custom error pages.

## New Features

### Automatic Requirements Installation

- **New Install Command**: Automatically install nginx, certbot, and required directories
- **Interactive Installer**: User-friendly installation wizard with progress reporting
- **Smart Detection**: Only installs what's missing, skips already satisfied requirements
- **Auto-Setup Option**: Enable automatic installation via `installRequirements: true` in plugin config
- **Requirements Module**: New dedicated Rust module for system dependency management

### Custom Error Pages

- **Custom HTML Pages**: Professional error pages with TailwindCSS styling
- **Responsive Design**: Works perfectly on desktop and mobile devices
- **French Localization**: All error messages in French for better UX
- **Interactive Elements**: Auto-refresh suggestions for temporary errors (502, 503, 504)
- **Dynamic Error Display**: Automatically detects and shows correct HTTP error codes
- **Branded Experience**: Consistent XyNginC branding and professional appearance

### Enhanced CLI and Plugin

- **New Server Method**: `server.xynginc.installRequirements()` for programmatic installation
- **Improved Error Handling**: Better error messages and user guidance
- **Template System**: Modular configuration templates for better maintainability
- **Health Checks**: Added `/health` endpoint for monitoring
- **Timeout Settings**: Improved proxy timeout configuration

## Technical Improvements

### Code Architecture

- **Modular Design**: Separated requirements management into dedicated module
- **Template Engine**: Dynamic configuration generation with variable replacement
- **Error Page System**: Automatic deployment of custom error pages
- **Configuration Templates**: SSL and non-SSL templates with error handling

### User Experience

- **Zero-Config Setup**: Automatic detection and installation of missing dependencies
- **Professional Error Pages**: Custom, branded error pages instead of default nginx messages
- **Better Feedback**: Enhanced logging and progress reporting
- **Development Friendly**: Perfect for development environments

## Installation

```bash
npm install xynginc@1.0.7
```

### Quick Auto-Setup

```bash
# Automatic installation of all requirements
sudo xynginc install
```

Or enable in your plugin:

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
  installRequirements: true, // Auto-install missing dependencies
});
```

## Quick Start

### Basic Configuration with Auto-Setup

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
        installRequirements: true, // Enable auto-setup
      }),
    ],
  },
});

app.start();
```

### Manual Requirements Installation

```typescript
app.on("ready", async () => {
  // Install requirements if needed
  await app.xynginc.installRequirements();

  // Continue with domain configuration...
});
```

## Error Pages

When errors occur (404, 502, etc.), users now see:

- **Professional Design**: TailwindCSS-styled pages with gradient backgrounds
- **Clear Messaging**: Error explanations in French
- **Interactive Elements**: Retry and back buttons
- **Auto-Detection**: Automatic error code display
- **Helpful Actions**: Auto-refresh suggestions for temporary issues

## CLI Usage

```bash
# Install system requirements (nginx, certbot, directories)
sudo xynginc install

# Check prerequisites
sudo xynginc check

# Add a domain
sudo xynginc add --domain api.example.com --port 3000 --ssl --email admin@example.com

# Test configuration
sudo xynginc test

# Reload Nginx
sudo xynginc reload
```

## Breaking Changes

**None** - This release is fully backward compatible.

## Bug Fixes

- Fixed nginx template variable replacement
- Improved error handling in requirements checking
- Enhanced binary compilation process
- Fixed TypeScript type definitions

## Known Issues

**None** - This release has no known issues.

## Assets

This release includes the following binary artifacts:

- `xynginc`: Single unified binary for Linux x64/arm64 systems

> **Note**: The npm package automatically downloads the appropriate binary for your system during installation.

## What's Next

- Enhanced SSL certificate management
- Load balancing capabilities
- Advanced monitoring and metrics
- Docker container support

---

**Upgrade recommended** for all users to benefit from the new automatic setup and professional error pages!
