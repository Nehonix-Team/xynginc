// Embedded templates - included directly in the binary
pub const NON_SSL_TEMPLATE: &str = include_str!("../configs/non_ssl_template.conf");
pub const SSL_TEMPLATE: &str = include_str!("../configs/ssl_template.conf");
pub const ERROR_HTML: &str = include_str!("../configs/error.html");
pub const INDEX_HTML: &str = include_str!("../configs/index.html");
pub const DEFAULT_CONFIG: &str = include_str!("../configs/default.conf");
pub const NGINX_MAIN_CONFIG: &str = include_str!("../configs/nginx_main.conf");

// Error pages
pub const ERROR_301_HTML: &str = include_str!("../configs/errors/301.html");
pub const ERROR_400_HTML: &str = include_str!("../configs/errors/400.html");
pub const ERROR_401_HTML: &str = include_str!("../configs/errors/401.html");
pub const ERROR_403_HTML: &str = include_str!("../configs/errors/403.html");
pub const ERROR_404_HTML: &str = include_str!("../configs/errors/404.html");
pub const ERROR_50X_HTML: &str = include_str!("../configs/errors/50x.html");

pub const NGINX_SITES_AVAILABLE: &str = "/etc/nginx/sites-available";
pub const NGINX_SITES_ENABLED: &str = "/etc/nginx/sites-enabled";
pub const BACKUP_DIR: &str = "/var/backups/xynginc";
