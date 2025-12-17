use std::path::Path;
use std::process::Command;

use crate::mods::constants::{BACKUP_DIR, NGINX_SITES_AVAILABLE, NGINX_SITES_ENABLED};
use crate::mods::logger::{log_error, log_info, log_step, log_success};

pub fn check_requirements() -> Result<(), String> {
    log_step("> Checking system requirements...\n");

    let mut all_ok = true;

    // Check nginx
    print!("   nginx:   ");
    match Command::new("nginx").arg("-v").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stderr);
            log_success(&format!("✓ {}", version.trim()));
        }
        Err(_) => {
            log_error("❌ Not installed");
            all_ok = false;
        }
    }

    // Check certbot
    print!("   certbot: ");
    match Command::new("certbot").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            log_success(&format!("✓ {}", version.trim()));
        }
        Err(_) => {
            log_error("❌ Not installed");
            all_ok = false;
        }
    }

    // Check directories
    print!("   nginx sites-available: ");
    if Path::new(NGINX_SITES_AVAILABLE).exists() {
        log_success(&format!("✓ {}", NGINX_SITES_AVAILABLE));
    } else {
        log_error("❌ Not found");
        all_ok = false;
    }

    print!("   nginx sites-enabled:   ");
    if Path::new(NGINX_SITES_ENABLED).exists() {
        log_success(&format!("✓ {}", NGINX_SITES_ENABLED));
    } else {
        log_error("❌ Not found");
        all_ok = false;
    }

    // Check backup directory
    print!("   backup directory:      ");
    if Path::new(BACKUP_DIR).exists() {
        log_success(&format!("✓ {}", BACKUP_DIR));
    } else {
        log_info(&format!("it will be created: {}", BACKUP_DIR));
    }

    if all_ok {
        log_success("\n✅ All requirements met!");
        Ok(())
    } else {
        Err("Some requirements are missing. Please install nginx and certbot.".to_string())
    }
}
