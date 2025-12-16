# XyNginC v1.2.0 Release Notes

## Major Refactoring

This release marks a significant milestone in the development of XyNginC. The codebase has been completely modularized to improve maintainability, scalability, and testing.

- **Code Modularization**: The monolithic main file has been split into 12 specialized modules (nginx, ssl, domain, config, etc.), improving organization without changing external behavior.

## New Features

- **Professional Default Configuration**: XyNginC now automatically installs a production-optimized default.conf in /etc/nginx/sites-available/default. It includes OWASP-recommended security headers and robust error handling.
- **Dynamic Error Pages**: Transitioned from static templates to dynamic JavaScript-based management. Error pages now display real error codes, real-time timestamps, unique Ray IDs for debugging, and an auto-refresh countdown.
- **Enhanced Templates**:
  - **SSL**: Updated to modern standards (A+ SSL Labs rating), TLS 1.2/1.3 only, HSTS enabled by default, and OCSP stapling.
  - **Non-SSL**: Added IPv6 support, improved proxy buffering, and native WebSocket support.

## Security Improvements

- Nginx version hiding (server_tokens off).
- Strict blocking of sensitive file extensions (.bak, .config, .sql, etc.).
- Comprehensive security headers added to all generated configurations.

## Upgrade Instructions

1. Replace the xynginc binary with the new version.
2. Run xynginc check to verify requirements.
3. Existing configurations will continue to work.
4. New configurations generated will use the new enhanced templates.

## Acknowledgments

Thanks to the development team for this major refactoring that ensures a solid future for the project.
