use std::process::Command;

use crate::mods::backup::list_backups;
use crate::mods::logger::{log_error, log_info, log_step, log_success};

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
