# XyNginC v1.4.5 Release Notes

## New Features

This release introduces the ability to customize the maximum client body size, resolving HTTP 413 (Payload Too Large) errors during large file uploads.

- **Customizable Max Body Size**: Developers can now specify the `maxBodySize` for each domain. This value is passed directly to the Nginx `client_max_body_size` directive.
- **CLI Support**: The `add` command now supports a `--max-body-size` flag.
- **Plugin Support**: The TypeScript plugin now accepts a `maxBodySize` property in the domain configuration object.
- **License Change**: The project is now licensed under the **NEHONIX Open Source License (NOSL) v1.0**. This change reinforces the proprietary nature of the source code while allowing collaborative use under specific conditions.
- **Proprietary Notices**: Mandatory proprietary notices have been added to all core configuration templates.

## Improvements

- **Template Integration**: Both SSL and non-SSL templates now use the dynamic `{{MAX_BODY_SIZE}}` placeholder, defaulting to `20M` if not specified.
- **TypeScript Mapping**: The plugin automatically maps camelCase `maxBodySize` to the snake_case `max_body_size` required by the Rust core.

## Upgrade Instructions

1. Replace the `xynginc` binary with version 1.4.5.
2. Update your TypeScript configuration if you need to increase upload limits:
   ```typescript
   XNCP({
     domains: [
       {
         domain: "api.example.com",
         port: 3000,
         maxBodySize: "100M", // Increase to 100MB
       },
     ],
   });
   ```
3. Re-apply your configuration: `sudo xynginc apply --config your-config.json`.

## Acknowledgments

Thanks to the users for reporting the 413 error during large file transfers, which helped us make XyNginC more flexible for data-intensive applications.
