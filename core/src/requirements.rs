/**
 * Requirements Management Module
 * 
 * This module handles the installation and configuration of system requirements
 * for XyNginC including nginx, certbot, and necessary directory structures.
 * Includes automatic APT repository error detection and fixing.
 */

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::Write;

/// System requirements that XyNginC needs to function
#[derive(Debug, Clone)]
pub struct SystemRequirements {
    pub nginx: bool,
    pub certbot: bool,
    pub sites_available_dir: bool,
    pub sites_enabled_dir: bool,
    pub backup_dir: bool,
}

/// Check which system requirements are missing
pub fn check_missing_requirements() -> Result<SystemRequirements, String> {
    println!("🔍 Checking system requirements...\n");

    let mut requirements = SystemRequirements {
        nginx: false,
        certbot: false,
        sites_available_dir: false,
        sites_enabled_dir: false,
        backup_dir: false,
    };

    // Check nginx
    print!("   nginx:   ");
    match Command::new("nginx").arg("-v").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stderr);
            println!("✓ {}", version.trim());
            requirements.nginx = true;
        }
        Err(_) => {
            println!("❌ Not installed");
        }
    }

    // Check certbot
    print!("   certbot: ");
    match Command::new("certbot").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("✓ {}", version.trim());
            requirements.certbot = true;
        }
        Err(_) => {
            println!("❌ Not installed");
        }
    }

    // Check directories
    let sites_available = "/etc/nginx/sites-available";
    let sites_enabled = "/etc/nginx/sites-enabled";
    let backup_dir = "/var/backups/xynginc";

    print!("   nginx sites-available: ");
    if Path::new(sites_available).exists() {
        println!("✓ {}", sites_available);
        requirements.sites_available_dir = true;
    } else {
        println!("❌ Not found");
    }

    print!("   nginx sites-enabled:   ");
    if Path::new(sites_enabled).exists() {
        println!("✓ {}", sites_enabled);
        requirements.sites_enabled_dir = true;
    } else {
        println!("❌ Not found");
    }

    print!("   backup directory:      ");
    if Path::new(backup_dir).exists() {
        println!("✓ {}", backup_dir);
        requirements.backup_dir = true;
    } else {
        println!("ℹ️  Will be created: {}", backup_dir);
    }

    Ok(requirements)
}

/// Detect if APT has repository errors
fn detect_apt_errors(error_message: &str) -> bool {
    // Common APT repository errors
    let error_patterns = vec![
        "n'a pas de fichier Release",
        "does not have a Release file",
        "Policy will reject signature",
        "NO_PUBKEY",
        "The repository",
        "is not signed",
    ];
    
    for pattern in error_patterns {
        if error_message.contains(pattern) {
            return true;
        }
    }
    
    false
}

/// Fix common APT repository problems
fn fix_apt_repositories() -> Result<(), String> {
    println!("\n🔧 Detecting APT repository issues...");
    
    // Test apt-get update to see if there are errors
    let test_output = Command::new("apt-get")
        .arg("update")
        .output()
        .map_err(|e| format!("Failed to test apt-get: {}", e))?;
    
    let stderr = String::from_utf8_lossy(&test_output.stderr);
    let stdout = String::from_utf8_lossy(&test_output.stdout);
    let combined_output = format!("{}\n{}", stdout, stderr);
    
    if !detect_apt_errors(&combined_output) && test_output.status.success() {
        println!("✓ APT repositories are healthy");
        return Ok(());
    }
    
    println!("⚠️  APT repository errors detected. Attempting automatic fix...\n");
    
    // Backup sources.list.d
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_dir = format!("/etc/apt/sources.list.d.backup.{}", timestamp);
    
    println!("📋 Step 1: Backing up current sources...");
    if Path::new("/etc/apt/sources.list.d").exists() {
        let output = Command::new("cp")
            .args(&["-r", "/etc/apt/sources.list.d", &backup_dir])
            .output()
            .map_err(|e| format!("Failed to backup sources: {}", e))?;
        
        if output.status.success() {
            println!("   ✓ Backup created at {}", backup_dir);
        }
    }
    
    println!("\n📋 Step 2: Disabling problematic repositories...");
    
    // List of problematic repository files to disable
    let problematic_repos = vec![
        "/etc/apt/sources.list.d/pgdg.list",
        "/etc/apt/sources.list.d/docker.list",
        "/etc/apt/sources.list.d/llvm.list",
    ];
    
    for repo_file in problematic_repos {
        if Path::new(repo_file).exists() {
            let disabled_file = format!("{}.disabled", repo_file);
            println!("   → Disabling {}", repo_file);
            
            fs::rename(repo_file, &disabled_file)
                .map_err(|e| format!("Failed to disable {}: {}", repo_file, e))?;
        }
    }
    
    // Scan for files containing problematic URLs
    if let Ok(entries) = fs::read_dir("/etc/apt/sources.list.d") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("list") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let has_problems = content.contains("ftp.postgresql.org") ||
                                          content.contains("download.docker.com/linux/ubuntu") ||
                                          content.contains("apt.llvm.org");
                        
                        if has_problems {
                            let disabled_path = format!("{}.disabled", path.display());
                            println!("   → Disabling {:?}", path.file_name().unwrap());
                            fs::rename(&path, &disabled_path)
                                .map_err(|e| format!("Failed to disable {:?}: {}", path, e))?;
                        }
                    }
                }
            }
        }
    }
    
    println!("   ✓ Problematic repositories disabled");
    
    println!("\n📋 Step 3: Verifying Kali main repositories...");
    
    // Ensure Kali official repos are present
    let sources_list = "/etc/apt/sources.list";
    if Path::new(sources_list).exists() {
        let content = fs::read_to_string(sources_list)
            .map_err(|e| format!("Failed to read sources.list: {}", e))?;
        
        if !content.contains("kali-rolling") {
            println!("   → Adding Kali official repositories");
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(sources_list)
                .map_err(|e| format!("Failed to open sources.list: {}", e))?;
            
            writeln!(file, "\n# Dépôts Kali officiels").map_err(|e| format!("Failed to write to sources.list: {}", e))?;
            writeln!(file, "deb http://http.kali.org/kali kali-rolling main contrib non-free non-free-firmware").map_err(|e| format!("Failed to write to sources.list: {}", e))?;
            writeln!(file, "deb-src http://http.kali.org/kali kali-rolling main contrib non-free non-free-firmware").map_err(|e| format!("Failed to write to sources.list: {}", e))?;
        }
    }
    
    println!("   ✓ Kali repositories verified");
    
    println!("\n📋 Step 4: Updating package lists...");
    
    let update_output = Command::new("apt-get")
        .arg("update")
        .output()
        .map_err(|e| format!("Failed to update package lists: {}", e))?;
    
    if !update_output.status.success() {
        let stderr = String::from_utf8_lossy(&update_output.stderr);
        println!("   ⚠️  Warning: Some repositories still have issues");
        println!("   {}", stderr);
    } else {
        println!("   ✓ Package lists updated successfully");
    }
    
    println!("\n✅ APT repositories fixed!");
    println!("\n💡 Note: Disabled repositories are saved with .disabled extension");
    println!("   To re-enable: sudo mv /etc/apt/sources.list.d/repo.list.disabled /etc/apt/sources.list.d/repo.list\n");
    
    Ok(())
}

/// Install missing system requirements with interactive mode
pub fn install_missing_requirements(requirements: &SystemRequirements) -> Result<(), String> {
    println!("\n📦 Installing missing requirements...\n");

    let mut packages_to_install = vec![];

    // Check if we need to install nginx
    if !requirements.nginx {
        println!("📋 Adding nginx installation...");
        packages_to_install.push("nginx");
    }

    // Check if we need to install certbot
    if !requirements.certbot {
        println!("📋 Adding certbot installation...");
        packages_to_install.push("certbot");
        packages_to_install.push("python3-certbot-nginx");
    }

    // Create missing directories first
    if !requirements.sites_available_dir {
        println!("📋 Creating sites-available directory...");
        Command::new("mkdir")
            .args(&["-p", "/etc/nginx/sites-available"])
            .status()
            .map_err(|e| format!("Failed to create sites-available directory: {}", e))?;
    }

    if !requirements.sites_enabled_dir {
        println!("📋 Creating sites-enabled directory...");
        Command::new("mkdir")
            .args(&["-p", "/etc/nginx/sites-enabled"])
            .status()
            .map_err(|e| format!("Failed to create sites-enabled directory: {}", e))?;
    }

    if !requirements.backup_dir {
        println!("📋 Creating backup directory...");
        fs::create_dir_all("/var/backups/xynginc")
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    }

    // Install packages if needed
    if !packages_to_install.is_empty() {
        println!("🔄 Installing packages: {}", packages_to_install.join(", "));
        println!("   This may take a few moments and require confirmation...\n");
        
        // Update package list first
        println!("📥 Updating package lists...");
        let update_status = Command::new("apt-get")
            .arg("update")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to update packages: {}", e))?;
        
        if !update_status.success() {
            println!("\n⚠️  Package update had issues. Attempting to fix APT repositories...");
            fix_apt_repositories()?;
            
            // Retry update
            println!("\n🔄 Retrying package update...");
            let retry_update = Command::new("apt-get")
                .arg("update")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .map_err(|e| format!("Failed to retry update: {}", e))?;
            
            if !retry_update.success() {
                return Err("Failed to update package lists after fixing repositories".to_string());
            }
        }
        
        // Install packages with full interactivity
        println!("\n📦 Installing {}...", packages_to_install.join(", "));
        let mut install_cmd = Command::new("apt-get");
        install_cmd.arg("install")
                   .arg("-y");
        
        for package in &packages_to_install {
            install_cmd.arg(package);
        }
        
        // Use inherited stdio for full interactivity
        let install_status = install_cmd
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to install packages: {}", e))?;
        
        if !install_status.success() {
            return Err("Package installation failed. Check the errors above.".to_string());
        }
        
        println!("\n   ✓ Packages installed successfully");
    }

    // Configure nginx if it was just installed
    if !requirements.nginx {
        configure_nginx()?;
    }

    println!("\n✅ All requirements installed and configured successfully!");
    Ok(())
}

/// Configure nginx for XyNginC usage
fn configure_nginx() -> Result<(), String> {
    println!("\n⚙️  Configuring nginx...");

    // Ensure nginx service is enabled and started
    let output = Command::new("systemctl")
        .args(&["enable", "nginx"])
        .output()
        .map_err(|e| format!("Failed to enable nginx service: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to enable nginx service: {}", stderr));
    }

    let output = Command::new("systemctl")
        .args(&["start", "nginx"])
        .output()
        .map_err(|e| format!("Failed to start nginx service: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Don't fail if nginx is already running
        if !stderr.contains("already") {
            return Err(format!("Failed to start nginx service: {}", stderr));
        }
    }

    // Create or update nginx configuration to include sites-enabled
    let nginx_conf_path = "/etc/nginx/nginx.conf";
    
    if Path::new(nginx_conf_path).exists() {
        let conf_content = fs::read_to_string(nginx_conf_path)
            .map_err(|e| format!("Failed to read nginx.conf: {}", e))?;

        // Check if sites-enabled is already included
        if !conf_content.contains("sites-enabled") {
            println!("   🔧 Adding sites-enabled to nginx configuration...");
            
            // Find the http block and add include directive
            let mut updated_content = String::new();
            
            for line in conf_content.lines() {
                updated_content.push_str(line);
                updated_content.push('\n');
                
                if line.trim() == "http {" {
                    updated_content.push_str("    include /etc/nginx/sites-enabled/*;\n");
                }
            }

            // Write updated configuration
            fs::write(nginx_conf_path, updated_content)
                .map_err(|e| format!("Failed to write nginx.conf: {}", e))?;

            // Test nginx configuration
            let output = Command::new("nginx")
                .arg("-t")
                .output()
                .map_err(|e| format!("Failed to test nginx configuration: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Nginx configuration test failed: {}", stderr));
            }

            // Reload nginx
            let output = Command::new("systemctl")
                .args(&["reload", "nginx"])
                .output()
                .map_err(|e| format!("Failed to reload nginx: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to reload nginx: {}", stderr));
            }
        }
    }

    println!("   ✓ Nginx configured successfully");
    Ok(())
}

/// Interactive installation process
pub fn interactive_install() -> Result<(), String> {
    println!("🚀 XyNginC Interactive Installer\n");
    println!("This installer will check and install all required dependencies for XyNginC.");
    println!("You may be prompted for your password to install system packages.\n");

    // Check current requirements
    let requirements = check_missing_requirements()?;
    
    // Count missing requirements
    let missing_count = [
        !requirements.nginx,
        !requirements.certbot,
        !requirements.sites_available_dir,
        !requirements.sites_enabled_dir,
        !requirements.backup_dir,
    ].iter().filter(|&&x| x).count();

    if missing_count == 0 {
        println!("✅ All requirements are already satisfied!");
        return Ok(());
    }

    println!("\n📊 Summary:");
    println!("   - {} requirement(s) missing", missing_count);
    let mut install_list = String::new();
    if !requirements.nginx {
        install_list.push_str("nginx ");
    }
    if !requirements.certbot {
        install_list.push_str("certbot ");
    }
    if !requirements.sites_available_dir || !requirements.sites_enabled_dir {
        install_list.push_str("nginx directories ");
    }
    if !requirements.backup_dir {
        install_list.push_str("backup directory");
    }
    
    println!("   - Will install: {}", install_list.trim());

    // Check for non-interactive mode via environment variable or terminal detection
    let is_non_interactive = std::env::var("XYNC_INSTALL_MODE").ok() == Some("non-interactive".to_string())
        || !atty::is(atty::Stream::Stdin);
    
    let proceed = if is_non_interactive {
        // Non-interactive mode (automated installation)
        println!("\n→ Automated installation mode detected, proceeding...");
        true
    } else {
        // Interactive mode - ask for confirmation
        println!("\n❓ Do you want to proceed with installation? (y/N): ");
        
        // Read user input for confirmation
        let mut user_input = String::new();
        std::io::stdin()
            .read_line(&mut user_input)
            .map_err(|e| format!("Failed to read user input: {}", e))?;
        
        let user_input = user_input.trim().to_lowercase();
        user_input == "y" || user_input == "yes"
    };
    
    if !proceed {
        println!("Installation cancelled by user.");
        return Ok(());
    }

    println!("   → Proceeding with installation...");

    // Install missing requirements (with automatic APT fixing and full interactivity)
    install_missing_requirements(&requirements)?;
    
    // Final verification
    println!("\n🔍 Final verification...");
    let final_check = check_missing_requirements()?;
    
    let all_satisfied = final_check.nginx && final_check.certbot && 
                       final_check.sites_available_dir && final_check.sites_enabled_dir;
    
    if all_satisfied {
        println!("\n🎉 Installation completed successfully!");
        println!("XyNginC is now ready to use.");
    } else {
        return Err("Installation completed but some requirements are still missing.".to_string());
    }

    Ok(())
}