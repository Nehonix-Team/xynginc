use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xynginc")]
#[command(version = "1.4.5")]
#[command(about = "XyPriss Nginx Controller - Simplified Nginx and SSL management", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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

        /// Maximum client body size (e.g., 20M, 100M, 1G)
        #[arg(long, default_value = "20M")]
        max_body_size: String,
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
