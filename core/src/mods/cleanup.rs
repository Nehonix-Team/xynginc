use std::fs;
use std::path::Path;
use std::process::Command;

use crate::mods::constants::{NGINX_SITES_AVAILABLE, NGINX_SITES_ENABLED};
use crate::mods::logger::{log_info, log_step, log_success, log_warning};

pub fn detect_broken_configs() -> Result<Vec<String>, String> {
    let mut broken = vec![];

    // Test nginx config
    let output = Command::new("nginx")
        .arg("-t")
        .output()
        .map_err(|e| format!("Failed to run nginx -t: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Parser le output pour trouver les fichiers problématiques
        for line in stderr.lines() {
            // Chercher les patterns comme "cannot load certificate" ou "unknown directive"
            if line.contains("in /etc/nginx/sites-enabled/") {
                if let Some(start) = line.find("/etc/nginx/sites-enabled/") {
                    if let Some(domain_end) = line[start + 25..].find(':') {
                        let domain = &line[start + 25..start + 25 + domain_end];
                        if !broken.contains(&domain.to_string()) {
                            broken.push(domain.to_string());
                        }
                    }
                }
            } else if line.contains("cannot load certificate") {
                // Extraire le domaine du path du certificat
                if let Some(start) = line.find("/etc/letsencrypt/live/") {
                    if let Some(end) = line[start + 22..].find('/') {
                        let domain = &line[start + 22..start + 22 + end];
                        // Trouver la config correspondante
                        let config_path = format!("{}/{}", NGINX_SITES_ENABLED, domain);
                        if Path::new(&config_path).exists() && !broken.contains(&domain.to_string()) {
                            broken.push(domain.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(broken)
}

pub fn clean_broken_configs(dry_run: bool) -> Result<(), String> {
    log_step("🧹 Cleaning broken configurations...\n");

    let broken = detect_broken_configs()?;

    if broken.is_empty() {
        log_success("✓ No broken configurations found");
        return Ok(());
    }

    log_warning(&format!("Found {} broken configuration(s):", broken.len()));
    for domain in &broken {
        log_info(&format!("   - {}", domain));
    }

    if dry_run {
        log_warning("\nDry run mode: no changes made");
        return Ok(());
    }

    log_step("\n🗑️  Removing broken configurations...");
    for domain in &broken {
        match remove_config_files(domain) {
            Ok(_) => log_success(&format!("   ✓ Removed: {}", domain)),
            Err(e) => log_warning(&format!("   ⚠️  Failed to remove {}: {}", domain, e)),
        }
    }

    log_success("\n✅ Cleanup complete!");
    Ok(())
}

pub fn remove_config_files(domain: &str) -> Result<(), String> {
    let available_path = format!("{}/{}", NGINX_SITES_AVAILABLE, domain);
    let enabled_path = format!("{}/{}", NGINX_SITES_ENABLED, domain);

    if Path::new(&enabled_path).exists() {
        fs::remove_file(&enabled_path).map_err(|e| format!("Failed to remove symlink: {}", e))?;
    }

    if Path::new(&available_path).exists() {
        fs::remove_file(&available_path).map_err(|e| format!("Failed to remove config: {}", e))?;
    }

    Ok(())
}
