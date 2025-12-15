use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

mod requirements;
use requirements::interactive_install;

const NGINX_SITES_AVAILABLE: &str = "/etc/nginx/sites-available";
const NGINX_SITES_ENABLED: &str = "/etc/nginx/sites-enabled";
const BACKUP_DIR: &str = "/var/backups/xynginc";

#[derive(Parser)]
#[command(name = "xynginc")]
#[command(version = "1.0.1")]
#[command(about = "XyPriss Nginx Controller - Simplifie la gestion de Nginx et SSL", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply configuration from JSON file or stdin
    Apply {
        /// Path to config file (use '-' for stdin)
        #[arg(short, long)]
        config: String,

        /// Skip backup before applying
        #[arg(long)]
        no_backup: bool,

        /// Force apply even if nginx test fails
        #[arg(long)]
        force: bool,
    },

    /// Check system requirements (nginx, certbot)
    Check,

    /// Install and configure missing system requirements
    Install,

    /// List all configured domains
    List,

    /// Add a new domain configuration
    Add {
        /// Domain name (e.g., api.example.com)
        #[arg(short, long)]
        domain: String,

        /// Port to proxy to
        #[arg(short, long)]
        port: u16,

        /// Enable SSL with Let's Encrypt
        #[arg(short, long)]
        ssl: bool,

        /// Email for Let's Encrypt (required if ssl=true)
        #[arg(short, long)]
        email: Option<String>,
    },

    /// Remove a domain configuration
    Remove {
        /// Domain name to remove
        domain: String,
    },

    /// Test nginx configuration
    Test,

    /// Reload nginx
    Reload,

    /// Show status of all domains
    Status,

    /// Clean broken or conflicting configurations
    Clean {
        /// Dry run (don't delete, just show)
        #[arg(long)]
        dry_run: bool,
    },

    /// Restore from backup
    Restore {
        /// Backup timestamp to restore (or 'latest')
        backup_id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    domains: Vec<DomainConfig>,
    #[serde(default)]
    auto_reload: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DomainConfig {
    domain: String,
    port: u16,
    #[serde(default)]
    ssl: bool,
    email: Option<String>,
    #[serde(default = "default_host")]
    host: String,
}

fn default_host() -> String {
    "localhost".to_string()
}

fn main() {
    let cli = Cli::parse();

    // Check if running as root
    if !is_root() {
        eprintln!("❌ Error: XyNginC requires root privileges");
        eprintln!("   Please run with sudo: sudo xynginc ...");
        std::process::exit(1);
    }

    let result = match &cli.command {
        Commands::Apply { config, no_backup, force } => apply_config(config, *no_backup, *force),
        Commands::Check => check_requirements(),
        Commands::Install => install_requirements(),
        Commands::List => list_domains(),
        Commands::Add {
            domain,
            port,
            ssl,
            email,
        } => add_domain(domain, *port, *ssl, email.as_deref(), None),
        Commands::Remove { domain } => remove_domain(domain),
        Commands::Test => test_nginx(),
        Commands::Reload => reload_nginx(),
        Commands::Status => show_status(),
        Commands::Clean { dry_run } => clean_broken_configs(*dry_run),
        Commands::Restore { backup_id } => restore_backup(backup_id),
    };

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

fn install_requirements() -> Result<(), String> {
    interactive_install()
}

fn apply_config(config_path: &str, no_backup: bool, force: bool) -> Result<(), String> {
    println!("📋 Applying configuration...");

    let config_content = if config_path == "-" {
        println!("📥 Reading from stdin...");
        std::io::read_to_string(std::io::stdin())
            .map_err(|e| format!("Failed to read stdin: {}", e))?
    } else {
        fs::read_to_string(config_path).map_err(|e| format!("Failed to read config file: {}", e))?
    };

    let config: Config =
        serde_json::from_str(&config_content).map_err(|e| format!("Invalid JSON config: {}", e))?;

    println!("✓ Config parsed: {} domain(s)", config.domains.len());

    // ÉTAPE 0: Créer un backup avant toute modification
    if !no_backup {
        println!("\n💾 Creating backup...");
        create_backup()?;
    }

    // ÉTAPE 1: Détecter et nettoyer les configs cassées
    println!("\n🔍 Checking for broken configurations...");
    let broken_configs = detect_broken_configs()?;
    
    if !broken_configs.is_empty() {
        println!("⚠️  Found {} broken configuration(s)", broken_configs.len());
        for broken in &broken_configs {
            println!("   - {}", broken);
        }
        
        println!("🧹 Cleaning broken configurations...");
        for broken in &broken_configs {
            let _ = remove_config_files(broken); // Ignore errors
        }
        println!("✓ Cleanup complete");
    } else {
        println!("✓ No broken configurations found");
    }

    // ÉTAPE 2: Appliquer les nouvelles configurations
    for domain_config in &config.domains {
        println!("\n🌐 Processing: {}", domain_config.domain);
        
        // Vérifier si une config existe déjà
        if config_exists(&domain_config.domain) {
            println!("   ℹ️  Configuration already exists, will be overwritten");
        }
        
        generate_nginx_config(domain_config)?;
        enable_site(&domain_config.domain)?;

        if domain_config.ssl {
            setup_ssl(domain_config)?;
        }
    }

    // ÉTAPE 3: Tester la configuration avant reload
    println!("\n🧪 Testing nginx configuration...");
    match test_nginx() {
        Ok(_) => println!("✓ Configuration is valid"),
        Err(e) => {
            if force {
                println!("⚠️  Configuration test failed but --force is enabled");
                println!("   Error: {}", e);
            } else {
                println!("❌ Configuration test failed!");
                println!("   {}", e);
                println!("\n🔄 Rolling back changes...");
                
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
        println!("\n🔄 Auto-reload enabled");
        reload_nginx()?;
    }

    println!("\n✅ Configuration applied successfully!");
    Ok(())
}

fn create_backup() -> Result<(), String> {
    // Créer le répertoire de backup s'il n'existe pas
    if !Path::new(BACKUP_DIR).exists() {
        fs::create_dir_all(BACKUP_DIR)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_path = format!("{}/backup_{}", BACKUP_DIR, timestamp);
    
    fs::create_dir_all(&backup_path)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    // Copier sites-available
    copy_directory(NGINX_SITES_AVAILABLE, &format!("{}/sites-available", backup_path))?;
    
    // Copier sites-enabled (symlinks)
    copy_directory(NGINX_SITES_ENABLED, &format!("{}/sites-enabled", backup_path))?;

    println!("   ✓ Backup created: {}", backup_path);
    Ok(())
}

fn copy_directory(src: &str, dst: &str) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Failed to create directory: {}", e))?;
    
    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let file_name = entry.file_name();
        let src_path = entry.path();
        let dst_path = Path::new(dst).join(&file_name);
        
        if src_path.is_file() {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }
    
    Ok(())
}

fn restore_latest_backup() -> Result<(), String> {
    let backups = list_backups()?;
    
    if backups.is_empty() {
        return Err("No backups available".to_string());
    }
    
    let latest = &backups[0];
    restore_backup(latest)?;
    
    Ok(())
}

fn list_backups() -> Result<Vec<String>, String> {
    if !Path::new(BACKUP_DIR).exists() {
        return Ok(vec![]);
    }

    let mut backups = vec![];
    
    for entry in fs::read_dir(BACKUP_DIR).map_err(|e| format!("Failed to read backups: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let name = entry.file_name();
        backups.push(name.to_string_lossy().to_string());
    }
    
    backups.sort();
    backups.reverse(); // Plus récent en premier
    
    Ok(backups)
}

fn restore_backup(backup_id: &str) -> Result<(), String> {
    let backup_path = if backup_id == "latest" {
        let backups = list_backups()?;
        if backups.is_empty() {
            return Err("No backups available".to_string());
        }
        format!("{}/{}", BACKUP_DIR, backups[0])
    } else {
        format!("{}/{}", BACKUP_DIR, backup_id)
    };

    if !Path::new(&backup_path).exists() {
        return Err(format!("Backup not found: {}", backup_path));
    }

    println!("🔄 Restoring from backup: {}", backup_path);

    // Supprimer les configs actuelles
    for entry in fs::read_dir(NGINX_SITES_ENABLED).map_err(|e| format!("Failed to read sites-enabled: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if path.is_file() || path.is_symlink() {
            fs::remove_file(&path).map_err(|e| format!("Failed to remove file: {}", e))?;
        }
    }

    // Restaurer sites-available
    copy_directory(&format!("{}/sites-available", backup_path), NGINX_SITES_AVAILABLE)?;
    
    // Restaurer sites-enabled
    copy_directory(&format!("{}/sites-enabled", backup_path), NGINX_SITES_ENABLED)?;

    println!("✓ Backup restored successfully");
    
    Ok(())
}

fn detect_broken_configs() -> Result<Vec<String>, String> {
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

fn clean_broken_configs(dry_run: bool) -> Result<(), String> {
    println!("🧹 Cleaning broken configurations...\n");

    let broken = detect_broken_configs()?;

    if broken.is_empty() {
        println!("✓ No broken configurations found");
        return Ok(());
    }

    println!("Found {} broken configuration(s):", broken.len());
    for domain in &broken {
        println!("   - {}", domain);
    }

    if dry_run {
        println!("\n⚠️  Dry run mode: no changes made");
        return Ok(());
    }

    println!("\n🗑️  Removing broken configurations...");
    for domain in &broken {
        match remove_config_files(domain) {
            Ok(_) => println!("   ✓ Removed: {}", domain),
            Err(e) => println!("   ⚠️  Failed to remove {}: {}", domain, e),
        }
    }

    println!("\n✅ Cleanup complete!");
    Ok(())
}

fn remove_config_files(domain: &str) -> Result<(), String> {
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

fn config_exists(domain: &str) -> bool {
    let available_path = format!("{}/{}", NGINX_SITES_AVAILABLE, domain);
    Path::new(&available_path).exists()
}

fn check_requirements() -> Result<(), String> {
    println!("🔍 Checking system requirements...\n");

    let mut all_ok = true;

    // Check nginx
    print!("   nginx:   ");
    match Command::new("nginx").arg("-v").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stderr);
            println!("✓ {}", version.trim());
        }
        Err(_) => {
            println!("❌ Not installed");
            all_ok = false;
        }
    }

    // Check certbot
    print!("   certbot: ");
    match Command::new("certbot").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("✓ {}", version.trim());
        }
        Err(_) => {
            println!("❌ Not installed");
            all_ok = false;
        }
    }

    // Check directories
    print!("   nginx sites-available: ");
    if Path::new(NGINX_SITES_AVAILABLE).exists() {
        println!("✓ {}", NGINX_SITES_AVAILABLE);
    } else {
        println!("❌ Not found");
        all_ok = false;
    }

    print!("   nginx sites-enabled:   ");
    if Path::new(NGINX_SITES_ENABLED).exists() {
        println!("✓ {}", NGINX_SITES_ENABLED);
    } else {
        println!("❌ Not found");
        all_ok = false;
    }

    // Check backup directory
    print!("   backup directory:      ");
    if Path::new(BACKUP_DIR).exists() {
        println!("✓ {}", BACKUP_DIR);
    } else {
        println!("ℹ️  Will be created: {}", BACKUP_DIR);
    }

    if all_ok {
        println!("\n✅ All requirements met!");
        Ok(())
    } else {
        Err("Some requirements are missing. Please install nginx and certbot.".to_string())
    }
}

fn list_domains() -> Result<(), String> {
    println!("📋 Configured domains:\n");

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
                "✓ enabled"
            } else {
                "○ disabled"
            };
            println!("   {} - {}", name_str, status);
            count += 1;
        }
    }

    if count == 0 {
        println!("   (no domains configured)");
    }

    Ok(())
}

fn add_domain(domain: &str, port: u16, ssl: bool, email: Option<&str>, host: Option<&str>) -> Result<(), String> {
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

    println!("➕ Adding domain: {}", domain);
    
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

    println!("✅ Domain {} added successfully!", domain);
    Ok(())
}

fn remove_domain(domain: &str) -> Result<(), String> {
    println!("➖ Removing domain: {}", domain);

    // Backup avant suppression
    create_backup()?;

    remove_config_files(domain)?;
    
    test_nginx()?;
    reload_nginx()?;
    
    println!("✅ Domain {} removed successfully!", domain);
    Ok(())
}

/// Load configuration template from file
fn load_template(template_path: &str) -> Result<String, String> {
    let template_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/configs");
    let full_path = template_dir.join(template_path);
    
    fs::read_to_string(&full_path)
        .map_err(|e| format!("Failed to read template {}: {}", template_path, e))
}

/// Replace template variables with actual values
fn replace_template_variables(template: &str, variables: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    
    for (key, value) in variables {
        let placeholder = format!("{{{{{} }}}}", key);
        result = result.replace(&placeholder, value);
    }
    
    result
}

/// Generate nginx configuration using templates
fn generate_nginx_config(config: &DomainConfig) -> Result<(), String> {
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

    println!("   ✓ Config written to {}", config_path);
    Ok(())
}

fn enable_site(domain: &str) -> Result<(), String> {
    let available_path = format!("{}/{}", NGINX_SITES_AVAILABLE, domain);
    let enabled_path = format!("{}/{}", NGINX_SITES_ENABLED, domain);

    // Supprimer le symlink existant s'il existe
    if Path::new(&enabled_path).exists() {
        fs::remove_file(&enabled_path)
            .map_err(|e| format!("Failed to remove existing symlink: {}", e))?;
    }

    std::os::unix::fs::symlink(&available_path, &enabled_path)
        .map_err(|e| format!("Failed to create symlink: {}", e))?;
    
    println!("   ✓ Site enabled");
    Ok(())
}
 
fn setup_ssl(config: &DomainConfig) -> Result<(), String> {
    println!("🔒 Setting up SSL for {}...", config.domain);

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

    println!("   ✓ SSL certificate obtained");
    Ok(())
}

fn test_nginx() -> Result<(), String> {
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

fn reload_nginx() -> Result<(), String> {
    let output = Command::new("systemctl")
        .args(&["reload", "nginx"])
        .output()
        .map_err(|e| format!("Failed to reload nginx: {}", e))?;

    if output.status.success() {
        println!("   ✓ Nginx reloaded successfully!");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to reload nginx:\n{}", stderr))
    }
}

fn show_status() -> Result<(), String> {
    println!("📊 XyNginC Status\n");

    // Nginx status
    print!("Nginx service: ");
    let output = Command::new("systemctl")
        .args(&["is-active", "nginx"])
        .output()
        .map_err(|e| format!("Failed to check nginx status: {}", e))?;

    if output.status.success() {
        println!("✓ active");
    } else {
        println!("○ inactive");
    }

    // Configuration test
    print!("Configuration: ");
    match test_nginx() {
        Ok(_) => println!("✓ valid"),
        Err(_) => println!("❌ invalid"),
    }

    // List backups
    let backups = list_backups().unwrap_or_default();
    println!("\nBackups: {} available", backups.len());
    if !backups.is_empty() {
        println!("   Latest: {}", backups[0]);
    }

    // List domains
    println!("\nConfigured domains:");
    list_domains()?;

    Ok(())
}