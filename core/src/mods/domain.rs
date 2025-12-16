use colored::*;
use std::fs;
use std::path::Path;

use crate::mods::backup::create_backup;
use crate::mods::cleanup::remove_config_files;
use crate::mods::config::generate_nginx_config;
use crate::mods::constants::{NGINX_SITES_AVAILABLE, NGINX_SITES_ENABLED};
use crate::mods::logger::{log_info, log_step, log_success};
use crate::mods::models::DomainConfig;
use crate::mods::nginx::{reload_nginx, test_nginx};
use crate::mods::ssl::setup_ssl;

pub fn list_domains() -> Result<(), String> {
    log_step("Configured domains:\n");

    let sites = fs::read_dir(NGINX_SITES_AVAILABLE)
        .map_err(|e| format!("Failed to read sites-available: {}", e))?;

    let mut count = 0;
    for entry in sites {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str != "default" {
            let enabled = Path::new(&format!("{}/{}", NGINX_SITES_ENABLED, name_str)).exists();
            let status = if enabled {
                "✓ enabled".green()
            } else {
                "◯ disabled".normal()
            };
            println!("   {} - {}", name_str, status);
            count += 1;
        }
    }

    if count == 0 {
        log_info("   (no domains configured)");
    }

    Ok(())
}

pub fn add_domain(domain: &str, port: u16, ssl: bool, email: Option<&str>, host: Option<&str>) -> Result<(), String> {
    if ssl && email.is_none() {
        return Err("Email is required when SSL is enabled".to_string());
    }

    let config = DomainConfig {
        domain: domain.to_string(),
        port,
        ssl,
        email: email.map(|s| s.to_string()),
        host: host.unwrap_or("localhost").to_string(),
    };

    log_step(&format!("Adding domain: {}", domain));
    
    // Backup avant modification
    create_backup()?;
    
    generate_nginx_config(&config)?;
    enable_site(domain)?;

    if ssl {
        setup_ssl(&config)?;
    }

    // Test avant reload
    test_nginx()?;
    reload_nginx()?;

    log_success(&format!("✅ Domain {} added successfully!", domain));
    Ok(())
}

pub fn remove_domain(domain: &str) -> Result<(), String> {
    log_step(&format!("Removing domain: {}", domain));

    // Backup avant suppression
    create_backup()?;

    remove_config_files(domain)?;
    
    test_nginx()?;
    reload_nginx()?;
    
    log_success(&format!("✅ Domain {} removed successfully!", domain));
    Ok(())
}

pub fn enable_site(domain: &str) -> Result<(), String> {
    let available_path = format!("{}/{}", NGINX_SITES_AVAILABLE, domain);
    let enabled_path = format!("{}/{}", NGINX_SITES_ENABLED, domain);

    // Supprimer le symlink existant s'il existe
    if Path::new(&enabled_path).exists() {
        fs::remove_file(&enabled_path)
            .map_err(|e| format!("Failed to remove existing symlink: {}", e))?;
    }

    std::os::unix::fs::symlink(&available_path, &enabled_path)
        .map_err(|e| format!("Failed to create symlink: {}", e))?;
    
    log_success("   ✓ Site enabled");
    Ok(())
}
