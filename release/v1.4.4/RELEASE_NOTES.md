# XyNginC v1.4.4 Release Notes

## New Features

This release introduces support for custom 301 Moved Permanently error pages and further refines the Nginx configuration templates for better reliability.

- **Custom 301 Error Page**: Added a dedicated `301.html` error page to the core configuration. This page provides a professional interface for users when a resource has been permanently moved, ensuring a consistent user experience even during redirects.
- **Template Updates**: Both SSL and non-SSL Nginx templates have been updated to natively support the new 301 error page.

## Improvements

- **Enhanced Error Mapping**: The global `error.html` template now includes explicit mapping for HTTP 301 status codes, providing accurate descriptions and user-friendly messages.
- **Code Cleanup**: Removed redundant server header logic from the TypeScript plugin to favor Nginx-level header management, improving performance and reducing complexity.

## Bug Fixes

- **IP SSL Handling**: Inherited and verified the fix for IP-based SSL requests, ensuring that bare IP addresses automatically fall back to HTTP to prevent Certbot validation failures.
- **Fault Tolerance**: Verified the non-blocking SSL setup mechanism, allowing the configuration process to continue even if individual domain SSL acquisition fails.

## Upgrade Instructions

1. Replace the `xynginc` binary with the new version 1.4.4.
2. Existing configurations will continue to work, but it is recommended to re-apply configurations to benefit from the new 301 error page support.
3. Run `sudo xynginc apply --config <your-config.json>` to update your Nginx site files.

## Acknowledgments

Special thanks to the community for reporting the Cloudflare sub-domain limitations and IP SSL issues, which led to these robustness improvements.
