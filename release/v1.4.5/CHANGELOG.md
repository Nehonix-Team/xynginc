# XyNginC v1.4.5 - Changelog

## [1.4.5] - 2025-12-18

### Added

- `max_body_size` field in `DomainConfig` (Rust).
- `--max-body-size` flag in `xynginc add` command.
- `maxBodySize` property in `XyNginCDomainConfig` (TypeScript).
- `{{MAX_BODY_SIZE}}` placeholder in Nginx templates.
- Mandatory NEHONIX proprietary notices in configuration templates.
- Project license changed to NEHONIX Open Source License (NOSL) v1.0.

### Changed

- Default `client_max_body_size` remains `20M` if not specified.
- Updated `core/Cargo.toml` to version 1.4.5.
- Professionalized `install.sh` output (continued from v1.4.4).

### Fixed

- Resolved HTTP 413 error by allowing configurable upload limits.
