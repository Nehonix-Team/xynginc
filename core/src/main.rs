use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

const NGINX_SITES_AVAILABLE: &str = "/etc/nginx/sites-available";
const NGINX_SITES_ENABLED: &str = "/etc/nginx/sites-enabled";

#[derive(Parser)]
#[command(name = "xynginc")]
#[command(version = "1.0.0")]
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
    },

    /// Check system requirements (nginx, certbot)
    Check,

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
        Commands::Apply { config } => apply_config(config),
        Commands::Check => check_requirements(),
        Commands::List => list_domains(),
        Commands::Add {
            domain,
            port,
            ssl,
            email,
        } => add_domain(domain, *port, *ssl, email.as_deref()),
        Commands::Remove { domain } => remove_domain(domain),
        Commands::Test => test_nginx(),
        Commands::Reload => reload_nginx(),
        Commands::Status => show_status(),
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

fn apply_config(config_path: &str) -> Result<(), String> {
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

    for domain_config in &config.domains {
        println!("\n🌐 Processing: {}", domain_config.domain);
        generate_nginx_config(domain_config)?;
        enable_site(&domain_config.domain)?;

        if domain_config.ssl {
            setup_ssl(domain_config)?;
        }
    }

    if config.auto_reload {
        println!("\n🔄 Auto-reload enabled");
        reload_nginx()?;
    }

    println!("\n✅ Configuration applied successfully!");
    Ok(())
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

fn add_domain(domain: &str, port: u16, ssl: bool, email: Option<&str>) -> Result<(), String> {
    if ssl && email.is_none() {
        return Err("Email is required when SSL is enabled".to_string());
    }

    let config = DomainConfig {
        domain: domain.to_string(),
        port,
        ssl,
        email: email.map(|s| s.to_string()),
    };

    println!("➕ Adding domain: {}", domain);
    generate_nginx_config(&config)?;
    enable_site(domain)?;

    if ssl {
        setup_ssl(&config)?;
    }

    println!("✅ Domain {} added successfully!", domain);
    Ok(())
}

fn remove_domain(domain: &str) -> Result<(), String> {
    println!("➖ Removing domain: {}", domain);

    let available_path = format!("{}/{}", NGINX_SITES_AVAILABLE, domain);
    let enabled_path = format!("{}/{}", NGINX_SITES_ENABLED, domain);

    if Path::new(&enabled_path).exists() {
        fs::remove_file(&enabled_path).map_err(|e| format!("Failed to remove symlink: {}", e))?;
    }

    if Path::new(&available_path).exists() {
        fs::remove_file(&available_path).map_err(|e| format!("Failed to remove config: {}", e))?;
    }

    reload_nginx()?;
    println!("✅ Domain {} removed successfully!", domain);
    Ok(())
}

fn generate_nginx_config(config: &DomainConfig) -> Result<(), String> {
    let nginx_config = if config.ssl {
        format!(
            r#"server {{
    listen 80;
    server_name {};

    location / {{
        return 301 https://$host$request_uri;
    }}
}}

server {{
    listen 443 ssl http2;
    server_name {};

    ssl_certificate /etc/letsencrypt/live/{}/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/{}/privkey.pem;

    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    location / {{
        proxy_pass http://127.0.0.1:{};
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }}
}}
"#,
            config.domain, config.domain, config.domain, config.domain, config.port
        )
    } else {
        format!(
            r#"server {{
    listen 80;
    server_name {};

    location / {{
        proxy_pass http://127.0.0.1:{};
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }}
}}
"#,
            config.domain, config.port
        )
    };

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

    if !Path::new(&enabled_path).exists() {
        std::os::unix::fs::symlink(&available_path, &enabled_path)
            .map_err(|e| format!("Failed to create symlink: {}", e))?;
        println!("   ✓ Site enabled");
    }

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
    println!("🧪 Testing nginx configuration...");

    let output = Command::new("nginx")
        .arg("-t")
        .output()
        .map_err(|e| format!("Failed to run nginx -t: {}", e))?;

    if output.status.success() {
        println!("✅ Nginx configuration is valid!");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Nginx config test failed:\n{}", stderr))
    }
}

fn reload_nginx() -> Result<(), String> {
    println!("🔄 Reloading nginx...");

    let output = Command::new("systemctl")
        .args(&["reload", "nginx"])
        .output()
        .map_err(|e| format!("Failed to reload nginx: {}", e))?;

    if output.status.success() {
        println!("✅ Nginx reloaded successfully!");
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

    // List domains
    println!("\nConfigured domains:");
    list_domains()?;

    Ok(())
}
