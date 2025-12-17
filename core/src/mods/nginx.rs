use std::process::Command;

use crate::mods::backup::list_backups;
use crate::mods::logger::{log_error, log_info, log_step, log_success, log_warning};
use crate::mods::nginx_modules;

pub fn test_nginx() -> Result<(), String> {
    let output = Command::new("nginx")
        .arg("-t")
        .output()
        .map_err(|e| format!("Failed to run nginx -t: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Nginx config test failed:\n{}", stderr))
    }
}

/// Test nginx configuration and auto-fix module errors
pub fn test_nginx_with_autofix() -> Result<(), String> {
    let output = Command::new("nginx")
        .arg("-t")
        .output()
        .map_err(|e| format!("Failed to run nginx -t: {}", e))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check if error is related to headers-more module
    if stderr.contains("ngx_http_headers_more_filter_module.so") {
        log_warning("⚠️  Current nginx config has errors. Attempting to fix...");
        
        // Try to install headers-more module
        match nginx_modules::install_headers_more_module() {
            Ok(_) => {
                log_success("✓ Module installed, retesting configuration...");
                
                // Retry nginx test
                let retry_output = Command::new("nginx")
                    .arg("-t")
                    .output()
                    .map_err(|e| format!("Failed to retest nginx: {}", e))?;
                
                if retry_output.status.success() {
                    log_success("✓ Configuration is now valid!");
                    return Ok(());
                } else {
                    let retry_stderr = String::from_utf8_lossy(&retry_output.stderr);
                    return Err(format!("Nginx config test still failed after module installation:\n{}", retry_stderr));
                }
            }
            Err(e) => {
                return Err(format!("Failed to install headers-more module: {}\nOriginal nginx error:\n{}", e, stderr));
            }
        }
    }
    
    // If not a module error, return the original error
    Err(format!("Nginx config test failed:\n{}", stderr))
}

pub fn reload_nginx() -> Result<(), String> {
    let output = Command::new("systemctl")
        .args(&["reload", "nginx"])
        .output()
        .map_err(|e| format!("Failed to reload nginx: {}", e))?;

    if output.status.success() {
        log_success("✓ Nginx reloaded successfully!");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to reload nginx:\n{}", stderr))
    }
}

pub fn show_status() -> Result<(), String> {
    use crate::mods::domain::list_domains;
    
    log_step(" XyNginC Status\n");

    // Nginx status
    print!("Nginx service: ");
    let output = Command::new("systemctl")
        .args(&["is-active", "nginx"])
        .output()
        .map_err(|e| format!("Failed to check nginx status: {}", e))?;

    if output.status.success() {
        log_success("✓ active");
    } else {
        log_info("◯ inactive");
    }

    // Configuration test
    print!("Configuration: ");
    match test_nginx() {
        Ok(_) => log_success("✓ valid"),
        Err(_) => log_error("❌ invalid"),
    }

    // List backups
    let backups = list_backups().unwrap_or_default();
    log_info(&format!("\nBackups: {} available", backups.len()));
    if !backups.is_empty() {
        log_info(&format!("   Latest: {}", backups[0]));
    }

    // List domains
    log_info("\nConfigured domains:");
    list_domains()?;

    Ok(())
}
