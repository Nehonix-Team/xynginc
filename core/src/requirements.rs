/**
 * Requirements Management Module
 * 
 * This module handles the installation and configuration of system requirements
 * for XyNginC including nginx, certbot, and necessary directory structures.
 */

use std::fs;
use std::path::Path;
use std::process::Command;

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

/// Install missing system requirements
pub fn install_missing_requirements(requirements: &SystemRequirements) -> Result<(), String> {
    println!("\n📦 Installing missing requirements...\n");

    let mut install_commands = vec![];

    // Check if we need to install nginx
    if !requirements.nginx {
        println!("📋 Adding nginx installation...");
        install_commands.push(("nginx", "apt-get update && apt-get install -y nginx"));
    }

    // Check if we need to install certbot
    if !requirements.certbot {
        println!("📋 Adding certbot installation...");
        install_commands.push(("certbot", "apt-get update && apt-get install -y certbot python3-certbot-nginx"));
    }

    // Create missing directories
    if !requirements.sites_available_dir {
        println!("📋 Creating sites-available directory...");
        if let Err(e) = Command::new("mkdir")
            .args(&["-p", "/etc/nginx/sites-available"])
            .output() 
        {
            return Err(format!("Failed to create sites-available directory: {}", e));
        }
    }

    if !requirements.sites_enabled_dir {
        println!("📋 Creating sites-enabled directory...");
        if let Err(e) = Command::new("mkdir")
            .args(&["-p", "/etc/nginx/sites-enabled"])
            .output() 
        {
            return Err(format!("Failed to create sites-enabled directory: {}", e));
        }
    }

    if !requirements.backup_dir {
        println!("📋 Creating backup directory...");
        if let Err(e) = fs::create_dir_all("/var/backups/xynginc") {
            return Err(format!("Failed to create backup directory: {}", e));
        }
    }

    // Execute installation commands
    for (package, command) in install_commands {
        println!("🔄 Installing {}...", package);
        
        // Split command into parts for execution
        let parts: Vec<&str> = command.split(" && ").collect();
        
        for part in parts {
            let args: Vec<&str> = part.split_whitespace().collect();
            if args.is_empty() {
                continue;
            }

            let output = Command::new(args[0])
                .args(&args[1..])
                .output()
                .map_err(|e| format!("Failed to execute command '{}': {}", part, e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to install {}: {}", package, stderr));
            }
        }
        
        println!("   ✓ {} installed successfully", package);
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
    println!("⚙️  Configuring nginx...");

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
        return Err(format!("Failed to start nginx service: {}", stderr));
    }

    // Create or update nginx configuration to include sites-enabled
    let nginx_conf_path = "/etc/nginx/nginx.conf";
    
    if Path::new(nginx_conf_path).exists() {
        let conf_content = fs::read_to_string(nginx_conf_path)
            .map_err(|e| format!("Failed to read nginx.conf: {}", e))?;

        // Check if sites-enabled is already included
        if !conf_content.contains("sites-enabled") {
            println!("📝 Adding sites-enabled to nginx configuration...");
            
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

    println!("\n❓ Do you want to proceed with installation? (y/N): ");
    
    // Read user input for confirmation
    let mut user_input = String::new();
    std::io::stdin()
        .read_line(&mut user_input)
        .map_err(|e| format!("Failed to read user input: {}", e))?;
    
    let user_input = user_input.trim().to_lowercase();
    
    // Check if user wants to proceed
    if user_input != "y" && user_input != "yes" {
        println!("Installation cancelled by user.");
        return Ok(());
    }

    println!("   → Proceeding with installation...");

    // Install missing requirements
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