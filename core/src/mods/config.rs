use std::fs;
use std::io::Write;
use std::path::Path;

use crate::mods::constants::{ERROR_HTML, INDEX_HTML, NON_SSL_TEMPLATE, SSL_TEMPLATE};
use crate::mods::logger::{log_info, log_success};
use crate::mods::models::DomainConfig;

/// Load configuration template from embedded content
pub fn load_template(template_path: &str) -> Result<String, String> {
    match template_path {
        "non_ssl_template.conf" => Ok(NON_SSL_TEMPLATE.to_string()),
        "ssl_template.conf" => Ok(SSL_TEMPLATE.to_string()),
        _ => Err(format!("Unknown template: {}", template_path)),
    }
}

/// Replace template variables with actual values
pub fn replace_template_variables(template: &str, variables: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    
    result
}

/// Replace HTML variables with actual values
pub fn replace_html_variables(template: &str, variables: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    
    result
}

/// Generate nginx configuration using templates
pub fn generate_nginx_config(config: &DomainConfig) -> Result<(), String> {
    use crate::mods::constants::{NGINX_SITES_AVAILABLE};
    
    log_info(&format!("> Generating nginx configuration for {}", config.domain));
    
    // Load appropriate template based on SSL configuration
    let template_name = if config.ssl {
        "ssl_template.conf"
    } else {
        "non_ssl_template.conf"
    };
    
    let template = load_template(template_name)?;
    
    // Prepare template variables
    let port_str = config.port.to_string();
    let domain_hash = crate::mods::utils::get_domain_hash(&config.domain);
    let variables: Vec<(&str, &str)> = vec![
        ("DOMAIN_NAME", &config.domain),
        ("BACKEND_HOST", &config.host),
        ("BACKEND_PORT", &port_str),
        ("MAX_BODY_SIZE", &config.max_body_size),
        ("DOMAIN_HASH", &domain_hash),
    ];
    
    // Replace variables in template
    let nginx_config = replace_template_variables(&template, &variables);
    
    let config_path = format!("{}/{}", NGINX_SITES_AVAILABLE, config.domain);
    let mut file = fs::File::create(&config_path)
        .map_err(|e| format!("Failed to create config file: {}", e))?;

    file.write_all(nginx_config.as_bytes())
        .map_err(|e| format!("Failed to write config: {}", e))?;

    log_success(&format!("✓ Config written to $/{domain}", domain = config.domain));

    // Set up error pages, index page, and default config
    log_info("   > Setting up web pages and default config...");
    ensure_error_pages_exist(Some(&config.domain))
        .map_err(|e| format!("Failed to set up error pages: {}", e))?;
    ensure_index_page_exists()
        .map_err(|e| format!("Failed to set up index page: {}", e))?;
    ensure_default_config_exists()
        .map_err(|e| format!("Failed to set up default config: {}", e))?;

    Ok(())
}

/// Install or update the main nginx configuration
/// This replaces /etc/nginx/nginx.conf with our optimized configuration
pub fn ensure_nginx_main_config_exists() -> Result<(), String> {
    use crate::mods::constants::NGINX_MAIN_CONFIG;
    
    let nginx_conf_path = "/etc/nginx/nginx.conf";
    
    log_info("> Installing main nginx configuration...");
    
    // Always overwrite the main config to ensure it's up to date
    fs::write(nginx_conf_path, NGINX_MAIN_CONFIG)
        .map_err(|e| format!("Failed to write main nginx config: {}", e))?;
    
    log_success(&format!("✓ Main nginx config installed at {}", nginx_conf_path));
    
    Ok(())
}

/// Ensure the custom error page exists in the web directory
/// Replace the default nginx welcome page with XyNginC index
pub fn ensure_index_page_exists() -> Result<(), String> {
    let index_page_path = "/var/www/html/index.html";
    let default_nginx_index = "/var/www/html/index.nginx-debian.html";

    log_info("   > Setting up XyNginC index page");

    // Remove default nginx welcome page
    if Path::new(default_nginx_index).exists() {
        log_info("Removing default nginx welcome page");
        fs::remove_file(default_nginx_index)
            .map_err(|e| format!("Failed to remove default nginx index: {}", e))?;
    }

    // Create XyNginC index page if it doesn't exist
    if !Path::new(index_page_path).exists() {
        log_info(" Creating XyNginC index page");
        let index_html = generate_index_html();
        fs::write(index_page_path, index_html)
            .map_err(|e| format!("Failed to write index page: {}", e))?;
        
        log_success(&format!("   ✓ XyNginC index page created at {}", index_page_path));
    } else {
        log_success(&format!("   ✓ XyNginC index page already exists."));
    }

    Ok(())
}

/// Generate XyNginC index HTML
pub fn generate_index_html() -> String {
    replace_html_variables(INDEX_HTML, &[
        ("TITLE", "XyNginC"),
        ("DESCRIPTION", "Nginx Controller for XyPriss Applications"),
    ])
}

pub fn config_exists(domain: &str) -> bool {
    use crate::mods::constants::NGINX_SITES_AVAILABLE;
    let available_path = format!("{}/{}", NGINX_SITES_AVAILABLE, domain);
    Path::new(&available_path).exists()
}

/// Install or update the default nginx configuration
/// This replaces /etc/nginx/sites-available/default with our professional default config
pub fn ensure_default_config_exists() -> Result<(), String> {
    use crate::mods::constants::{DEFAULT_CONFIG, NGINX_SITES_AVAILABLE};
    
    let default_config_path = format!("{}/default", NGINX_SITES_AVAILABLE);
    
    log_info("> Installing default nginx configuration...");
    
    // Always overwrite the default config to ensure it's up to date
    fs::write(&default_config_path, DEFAULT_CONFIG)
        .map_err(|e| format!("Failed to write default config: {}", e))?;
    
    log_success(&format!("   ✓ Default nginx config installed at {}", default_config_path));

    Ok(())
}

/// Ensure error pages exist in the web directory
/// If domain is provided, files are named using domain hash
pub fn ensure_error_pages_exist(domain: Option<&str>) -> Result<(), String> {
    use crate::mods::constants::{ERROR_301_HTML, ERROR_400_HTML, ERROR_401_HTML, ERROR_403_HTML, ERROR_404_HTML, ERROR_50X_HTML, ERROR_HTML};
    use crate::mods::utils::get_domain_hash;
    
    let error_page_dir = "/var/www/html/errors";
    
    log_info("> Setting up pages...");
    
    // Create errors directory if it doesn't exist
    if !Path::new(error_page_dir).exists() {
        log_info("   Creating pages directory");
        fs::create_dir_all(error_page_dir)
            .map_err(|e| format!("Failed to create error pages directory {}: {}", error_page_dir, e))?;
    }
    
    let suffix = if let Some(d) = domain {
        format!("{}.", get_domain_hash(d))
    } else {
        "".to_string()
    };

    // Write error pages
    let error_pages = vec![
        (format!("{}301.html", suffix), ERROR_301_HTML),
        (format!("{}400.html", suffix), ERROR_400_HTML),
        (format!("{}401.html", suffix), ERROR_401_HTML),
        (format!("{}403.html", suffix), ERROR_403_HTML),
        (format!("{}404.html", suffix), ERROR_404_HTML),
        (format!("{}50x.html", suffix), ERROR_50X_HTML),
        (format!("{}error.html", suffix), ERROR_HTML),
    ];
    
    for (filename, content) in error_pages {
        let error_page_path = format!("{}/{}", error_page_dir, filename);
        
        // Only write if it doesn't exist to avoid unnecessary I/O
        if !Path::new(&error_page_path).exists() {
            log_info(&format!("   Writing error page: {}", filename));
            fs::write(&error_page_path, content)
                .map_err(|e| format!("Failed to write error page {}: {}", error_page_path, e))?;
        }
    }
    
    Ok(())
}
