use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "xynginc")]
#[command(about = "XyPriss Nginx Controller CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply configuration
    Apply {
        #[arg(short, long)]
        config: String,
    },
    /// Check system requirements
    Check,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    domains: Vec<DomainConfig>,
    auto_reload: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DomainConfig {
    domain: String,
    port: u16,
    ssl: Option<bool>,
    email: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Apply { config } => {
            println!("Applying config: {}", config);
            // TODO: Parse config and generate nginx files
        }
        Commands::Check => {
            println!("Checking system requirements...");
            // TODO: Check for nginx and certbot
        }
    }
}
