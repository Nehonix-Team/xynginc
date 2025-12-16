# XyNginC Module Dependency Graph

## Legend

- `→` : depends on
- `*` : base module (no internal dependencies)

## Base Modules (no dependencies)

```
logger.rs *
constants.rs *
models.rs *
cli.rs *
```

## Level 1 Modules (depend only on base modules)

```
backup.rs
  → constants (BACKUP_DIR, NGINX_SITES_*)
  → logger (log_success, log_step)

ssl.rs
  → logger (log_step, log_success)
  → models (DomainConfig)

check.rs
  → constants (NGINX_SITES_*, BACKUP_DIR)
  → logger (log_error, log_info, log_step, log_success)
```

## Level 2 Modules (depend on level 1 modules)

```
cleanup.rs
  → constants (NGINX_SITES_*)
  → logger (log_info, log_step, log_success, log_warning)

config.rs
  → constants (ERROR_HTML, INDEX_HTML, NON_SSL_TEMPLATE, SSL_TEMPLATE, NGINX_SITES_AVAILABLE)
  → logger (log_info, log_success)
  → models (DomainConfig)

nginx.rs
  → backup (list_backups)
  → logger (log_error, log_info, log_step, log_success)
```

## Level 3 Modules (depend on level 2 modules)

```
domain.rs
  → backup (create_backup)
  → cleanup (remove_config_files)
  → config (generate_nginx_config)
  → constants (NGINX_SITES_*)
  → logger (log_info, log_step, log_success)
  → models (DomainConfig)
  → nginx (reload_nginx, test_nginx)
  → ssl (setup_ssl)

apply.rs
  → backup (create_backup, restore_latest_backup)
  → cleanup (detect_broken_configs, remove_config_files)
  → config (config_exists, generate_nginx_config)
  → domain (enable_site)
  → logger (log_error, log_info, log_step, log_success, log_warning)
  → models (Config)
  → nginx (reload_nginx, test_nginx)
  → ssl (setup_ssl)
```

## Main Entry Point

```
main.rs
  → requirements (interactive_install) [existing module]
  → apply (apply_config)
  → backup (restore_backup)
  → check (check_requirements)
  → cleanup (clean_broken_configs)
  → cli (Cli, Commands)
  → domain (add_domain, list_domains, remove_domain)
  → logger (log_error)
  → nginx (reload_nginx, show_status, test_nginx)
```

## Hierarchical Visualization

```
Level 0 (Base)
┌─────────────────────────────────────────┐
│ logger  constants  models  cli          │
└─────────────────────────────────────────┘
              ↑
Level 1
┌─────────────────────────────────────────┐
│ backup  ssl  check                      │
└─────────────────────────────────────────┘
              ↑
Level 2
┌─────────────────────────────────────────┐
│ cleanup  config  nginx                  │
└─────────────────────────────────────────┘
              ↑
Level 3
┌─────────────────────────────────────────┐
│ domain  apply                           │
└─────────────────────────────────────────┘
              ↑
Entry Point
┌─────────────────────────────────────────┐
│ main.rs                                 │
└─────────────────────────────────────────┘
```

## Design Principles

### 1. Separation of Concerns

Each module has a single, well-defined responsibility.

### 2. Unidirectional Dependencies

Higher-level modules depend on lower-level modules, never the reverse.

### 3. Reusable Base Modules

Base modules (`logger`, `constants`, `models`, `cli`) can be used anywhere without creating circular dependencies.

### 4. Isolation of Side Effects

- `logger`: Only module responsible for display
- `constants`: Only module containing global constants
- `models`: Only module defining data structures

### 5. Functional Composition

High-level modules (`apply`, `domain`) compose functionality from low-level modules.

## Architecture Benefits

✅ **No Circular Dependencies**

- Clear layered architecture
- Facilitates understanding of data flow

✅ **Testability**

- Base modules can be tested in isolation
- High-level modules can be tested with mocks

✅ **Maintainability**

- Changes localized to a single module
- Limited impact on other modules

✅ **Scalability**

- Easy to add new modules
- Extensible structure without major refactoring
