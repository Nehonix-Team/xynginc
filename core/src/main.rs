use clap::Parser;

mod requirements;
mod mods;

use mods::apply::apply_config;
use mods::backup::restore_backup;
use mods::check::check_requirements;
use mods::cleanup::clean_broken_configs;
use mods::cli::{Cli, Commands};
use mods::domain::{add_domain, list_domains, remove_domain};
use mods::logger::log_error;
use mods::nginx::{reload_nginx, show_status, test_nginx};
use requirements::interactive_install;

fn main() {
    let cli = Cli::parse();

    // Check if running as root
    if !is_root() {
        log_error("❌ Error: XyNginC requires root privileges");
        log_error("   Please run with sudo: sudo xynginc ...");
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
            max_body_size,
        } => add_domain(domain, *port, *ssl, email.as_deref(), None, Some(max_body_size)),
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
            log_error(&format!("❌ Error: {}", e));
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