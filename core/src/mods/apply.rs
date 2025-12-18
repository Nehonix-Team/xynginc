use std::fs;

use crate::mods::backup::{create_backup, restore_latest_backup};
use crate::mods::cleanup::{detect_broken_configs, remove_config_files};
use crate::mods::config::{config_exists, generate_nginx_config, ensure_nginx_main_config_exists, ensure_error_pages_exist};
use crate::mods::domain::enable_site;
use crate::mods::logger::{log_error, log_info, log_step, log_success, log_warning};
use crate::mods::models::Config;
use crate::mods::nginx::{reload_nginx, test_nginx_with_autofix};
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

    // ÉTAPE 2: Installer la configuration principale nginx
    log_step("\n> Installing main nginx configuration...");
    ensure_nginx_main_config_exists()?;

    // ÉTAPE 3: Installer les pages d'erreur personnalisées
    ensure_error_pages_exist(None)?;

    // ÉTAPE 5: Appliquer les nouvelles configurations
    for domain_config in &config.domains {
        log_step(&format!("\n🌐 Processing: {}", domain_config.domain));
        
        // Vérifier si une config existe déjà
        if config_exists(&domain_config.domain) {
            log_info("> Configuration already exists, will be overwritten");
        }
        
        // Vérifier si le domaine est une adresse IP
        let is_ip = domain_config.domain.parse::<std::net::IpAddr>().is_ok();
        
        // Si SSL est demandé, générer d'abord une config HTTP temporaire
        if domain_config.ssl {
            if is_ip {
                log_warning(&format!("⚠️  SSL requested for IP address '{}', but Let's Encrypt does not support IP addresses.", domain_config.domain));
                log_warning("   Falling back to HTTP for this domain.");
                
                // Désactiver SSL pour cette entrée
                let mut http_config = domain_config.clone();
                http_config.ssl = false;
                
                generate_nginx_config(&http_config)?;
                enable_site(&http_config.domain)?;
            } else {
                log_info("> SSL requested - generating temporary HTTP configuration first");
                
                // Créer une config temporaire sans SSL
                let mut temp_config = domain_config.clone();
                temp_config.ssl = false;
                
                generate_nginx_config(&temp_config)?;
                enable_site(&temp_config.domain)?;
                
                // Recharger nginx pour que certbot puisse l'utiliser
                log_info("> Reloading nginx for certbot validation...");
                reload_nginx()?;
                
                // Obtenir le certificat SSL
                match setup_ssl(domain_config) {
                    Ok(_) => {
                        // Maintenant générer la vraie config avec SSL
                        log_info("> Generating final HTTPS configuration...");
                        generate_nginx_config(domain_config)?;
                        enable_site(&domain_config.domain)?;
                    },
                    Err(e) => {
                        log_error(&format!("❌ SSL setup failed for {}: {}", domain_config.domain, e));
                        log_warning("   ⚠️  Falling back to HTTP only for this domain due to SSL error.");
                        
                        // Revenir à la config HTTP (déjà générée plus haut, mais on s'assure qu'elle est active)
                        // Pas besoin de régénérer car temp_config était déjà appliquée
                        // Mais pour être sûr (au cas où setup_ssl aurait cassé quelque chose)
                        let mut http_config = domain_config.clone();
                        http_config.ssl = false;
                        generate_nginx_config(&http_config)?;
                        enable_site(&http_config.domain)?;
                    }
                }
            }
        } else {
            // Pas de SSL, générer directement la config HTTP
            generate_nginx_config(domain_config)?;
            enable_site(&domain_config.domain)?;
        }
    }

    // ÉTAPE 6: Tester la configuration avant reload (avec auto-fix)
    log_step("\n> Testing nginx configuration...");
    match test_nginx_with_autofix() {
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

    // ÉTAPE 7: Reload nginx si auto_reload est activé
    if config.auto_reload {
        log_step("\n🔄 Auto-reload enabled");
        reload_nginx()?;
    }

    log_success("\n✅ Configuration applied successfully!");
    Ok(())
}
