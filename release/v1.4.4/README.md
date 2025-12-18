# XyNginC v1.4.4 - 301 Error Page & Robustness

**Release Date**: December 18, 2025  
**Version**: 1.4.4

## Overview

XyNginC v1.4.4 focuses on improving user experience during redirects and enhancing the overall robustness of the Nginx configuration process.

## Key Changes

### 1. Custom 301 Error Page

A new professional error page for HTTP 301 (Moved Permanently) has been added. This ensures that users are gracefully informed when a resource has changed its location.

### 2. Template Synchronization

Nginx templates for both SSL and non-SSL sites have been updated to include the new 301 error page handling.

### 3. Plugin Optimization

The TypeScript plugin has been streamlined by removing redundant response header logic, delegating header management entirely to the high-performance Nginx layer.

## Installation

To upgrade to v1.4.4:

```bash
cd release/v1.4.4
sudo ./install.sh
```

## Verification

After installation, verify the version:

```bash
xynginc --version
```

For detailed changes, please refer to the [Release Notes](RELEASE_NOTES.md).
