use std::process::{Command, Stdio};

use crate::mods::logger::{log_step, log_success, log_warning};
use crate::mods::models::DomainConfig;

/// Check if certbot nginx plugin is available
fn check_certbot_nginx_plugin() -> bool {
    if let Ok(output) = Command::new("certbot")
        .args(&["plugins", "--text"])
        .output()
    {
        let plugins_text = String::from_utf8_lossy(&output.stdout);
        plugins_text.contains("nginx") || plugins_text.contains("* nginx")
    } else {
        false
    }
}

/// Install certbot nginx plugin
fn install_certbot_nginx_plugin() -> Result<(), String> {
    log_warning("⚠️  Certbot nginx plugin not found. Installing...");
    
    let output = Command::new("apt-get")
        .args(&["install", "-y", "python3-certbot-nginx"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("Failed to install certbot nginx plugin: {}", e))?;
    
    if !output.status.success() {
        return Err("Failed to install python3-certbot-nginx package".to_string());
    }
    
    log_success("✓ Certbot nginx plugin installed");
    Ok(())
}

pub fn setup_ssl(config: &DomainConfig) -> Result<(), String> {
    log_step(&format!("> Setting up SSL for {}...", config.domain));

    let email = config.email.as_ref().ok_or("Email required for SSL")?;

    // Check if nginx plugin is available, install if missing
    if !check_certbot_nginx_plugin() {
        install_certbot_nginx_plugin()?;
    }

    let output = Command::new("certbot")
        .args(&[
            "certonly",
            "--nginx",
            "-d",
            &config.domain,
            "--email",
            email,
            "--agree-tos",
            "--non-interactive",
        ])
        .output()
        .map_err(|e| format!("Failed to run certbot: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Check if it's a plugin error and try to fix
        if stderr.contains("does not appear to be installed") || stderr.contains("nginx plugin") {
            log_warning("⚠️  Certbot nginx plugin error detected. Attempting to fix...");
            install_certbot_nginx_plugin()?;
            
            // Retry certbot
            log_step("> Retrying SSL certificate request...");
            let retry_output = Command::new("certbot")
                .args(&[
                    "certonly",
                    "--nginx",
                    "-d",
                    &config.domain,
                    "--email",
                    email,
                    "--agree-tos",
                    "--non-interactive",
                ])
                .output()
                .map_err(|e| format!("Failed to retry certbot: {}", e))?;
            
            if !retry_output.status.success() {
                let retry_stderr = String::from_utf8_lossy(&retry_output.stderr);
                return Err(format!("Certbot failed after plugin installation:\n{}", retry_stderr));
            }
        } else {
            return Err(format!("Certbot failed:\n{}\n{}", stderr, stdout));
        }
    }

    log_success("✓ SSL certificate obtained");
    Ok(())
}
