# XyNginC v1.4.4 - Changelog

## [1.4.4] - 2025-12-18

### Added

- New `301.html` error page in `core/src/configs/errors/`.
- Support for `error_page 301` in Nginx templates (`ssl_template.conf`, `non_ssl_template.conf`).
- HTTP 301 mapping in the global `error.html` template.

### Changed

- Updated `core/Cargo.toml` version to 1.4.4.
- Removed `onResponse` header manipulation from `src/index.ts` to favor Nginx-level headers.
- Refined professional tone in release documentation.

### Fixed

- Verified robustness of SSL fallback and IP detection logic.
