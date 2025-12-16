use std::process::Command;

use crate::mods::logger::{log_step, log_success};
use crate::mods::models::DomainConfig;

pub fn setup_ssl(config: &DomainConfig) -> Result<(), String> {
    log_step(&format!("> Setting up SSL for {}...", config.domain));

    let email = config.email.as_ref().ok_or("Email required for SSL")?;

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
        return Err(format!("Certbot failed: {}", stderr));
    }

    log_success("✓ SSL certificate obtained");
    Ok(())
}
