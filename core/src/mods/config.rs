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
    let variables: Vec<(&str, &str)> = vec![
        ("DOMAIN_NAME", &config.domain),
        ("BACKEND_HOST", &config.host),
        ("BACKEND_PORT", &port_str),
    ];
    
    // Replace variables in template
    let nginx_config = replace_template_variables(&template, &variables);
    
    let config_path = format!("{}/{}", NGINX_SITES_AVAILABLE, config.domain);
    let mut file = fs::File::create(&config_path)
        .map_err(|e| format!("Failed to create config file: {}", e))?;

    file.write_all(nginx_config.as_bytes())
        .map_err(|e| format!("Failed to write config: {}", e))?;

    log_success(&format!("✓ Config written to {}", config_path));

    // Set up error pages, index page, and default config
    log_info("   🔧 Setting up web pages and default config...");
    ensure_error_page_exists()
        .map_err(|e| format!("Failed to set up error page: {}", e))?;
    ensure_index_page_exists()
        .map_err(|e| format!("Failed to set up index page: {}", e))?;
    ensure_default_config_exists()
        .map_err(|e| format!("Failed to set up default config: {}", e))?;

    Ok(())
}

/// Ensure the custom error page exists in the web directory
pub fn ensure_error_page_exists() -> Result<(), String> {
    let error_page_dir = "/var/www/html/errors";
    let error_page_path = format!("{}/error.html", error_page_dir);

    log_info("   Setting up error page at error.html");

    // Create errors directory if it doesn't exist
    if !Path::new(error_page_dir).exists() {
        log_info("   Creating error page directory");
        fs::create_dir_all(error_page_dir)
            .map_err(|e| format!("Failed to create error page directory {}: {}", error_page_dir, e))?;
    }

    // Write error page if it doesn't exist (no variable replacement needed - handled by nginx and JS)
    if !Path::new(&error_page_path).exists() {
        log_info("   Writing error page HTML...");
        fs::write(&error_page_path, ERROR_HTML)
            .map_err(|e| format!("Failed to write error page {}: {}", error_page_path, e))?;
        

        log_success("   Error page created at error.html");
    } else {
        log_success("   Error page already exists at error.html");
    }

    Ok(())
}

/// Replace the default nginx welcome page with XyNginC index
pub fn ensure_index_page_exists() -> Result<(), String> {
    let index_page_path = "/var/www/html/index.html";
    let default_nginx_index = "/var/www/html/index.nginx-debian.html";

    log_info("   🔧 Setting up XyNginC index page");

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
        log_success(&format!("   ✓ XyNginC index page already exists at {}", index_page_path));
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
    
    log_info("   🔧 Installing default nginx configuration...");
    
    // Always overwrite the default config to ensure it's up to date
    fs::write(&default_config_path, DEFAULT_CONFIG)
        .map_err(|e| format!("Failed to write default config: {}", e))?;
    
    log_success(&format!("   ✓ Default nginx config installed at {}", default_config_path));
    
    Ok(())
}
