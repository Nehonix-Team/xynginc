use std::fs;

use crate::mods::backup::{create_backup, restore_latest_backup};
use crate::mods::cleanup::{detect_broken_configs, remove_config_files};
use crate::mods::config::{config_exists, generate_nginx_config};
use crate::mods::domain::enable_site;
use crate::mods::logger::{log_error, log_info, log_step, log_success, log_warning};
use crate::mods::models::Config;
use crate::mods::nginx::{reload_nginx, test_nginx};
use crate::mods::ssl::setup_ssl;

pub fn apply_config(config_path: &str, no_backup: bool, force: bool) -> Result<(), String> {
    log_step("> Applying configuration...");

    let config_content = if config_path == "-" {
        log_info("> Reading from stdin...");
        std::io::read_to_string(std::io::stdin())
            .map_err(|e| format!("Failed to read stdin: {}", e))?
    } else {
        fs::read_to_string(config_path).map_err(|e| format!("Failed to read config file: {}", e))?
    };

    let config: Config =
        serde_json::from_str(&config_content).map_err(|e| format!("Invalid JSON config: {}", e))?;

    log_success(&format!("✓ Config parsed: {} domain(s)", config.domains.len()));

    // ÉTAPE 0: Créer un backup avant toute modification
    if !no_backup {
        log_step("\n> Creating backup...");
        create_backup()?;
    }

    // ÉTAPE 1: Détecter et nettoyer les configs cassées
    log_step("\n> Checking for broken configurations...");
    let broken_configs = detect_broken_configs()?;
    
    if !broken_configs.is_empty() {
        log_warning(&format!("⚠️  Found {} broken configuration(s)", broken_configs.len()));
        for broken in &broken_configs {
            log_info(&format!("   - {}", broken));
        }
        
        log_step("> Cleaning broken configurations...");
        for broken in &broken_configs {
            let _ = remove_config_files(broken); // Ignore errors
        }
        log_success("✓ Cleanup complete");
    } else {
        log_success("✓ No broken configurations found");
    }

    // ÉTAPE 2: Appliquer les nouvelles configurations
    for domain_config in &config.domains {
        log_step(&format!("\n🌐 Processing: {}", domain_config.domain));
        
        // Vérifier si une config existe déjà
        if config_exists(&domain_config.domain) {
            log_info("> Configuration already exists, will be overwritten");
        }
        
        generate_nginx_config(domain_config)?;
        enable_site(&domain_config.domain)?;

        if domain_config.ssl {
            setup_ssl(domain_config)?;
        }
    }

    // ÉTAPE 3: Tester la configuration avant reload
    log_step("\n🧪 Testing nginx configuration...");
    match test_nginx() {
        Ok(_) => log_success("✓ Configuration is valid"),
        Err(e) => {
            if force {
                log_warning("⚠️  Configuration test failed but --force is enabled");
                log_warning(&format!("   Error: {}", e));
            } else {
                log_error("❌ Configuration test failed!");
                log_error(&format!("   {}", e));
                log_step("\n🔄 Rolling back changes...");
                
                // Restaurer le backup
                if !no_backup {
                    restore_latest_backup()?;
                }
                
                return Err("Configuration test failed. Changes have been rolled back.".to_string());
            }
        }
    }

    // ÉTAPE 4: Reload nginx si auto_reload est activé
    if config.auto_reload {
        log_step("\n🔄 Auto-reload enabled");
        reload_nginx()?;
    }

    log_success("\n✅ Configuration applied successfully!");
    Ok(())
}
