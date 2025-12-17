/**
 * Nginx Modules Management
 * 
 * This module handles the installation and configuration of additional nginx modules
 * required by XyNginC, specifically the headers-more-nginx-module for custom server headers.
 */

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

/// Check if headers-more module is installed and loaded
pub fn check_headers_more_module() -> Result<bool, String> {
    // Check if the module file exists
    let module_paths = vec![
        "/usr/share/nginx/modules/ngx_http_headers_more_filter_module.so",
        "/usr/lib/nginx/modules/ngx_http_headers_more_filter_module.so",
    ];
    
    let module_exists = module_paths.iter().any(|path| Path::new(path).exists());
    
    if !module_exists {
        return Ok(false);
    }
    
    // Check if module is loaded in nginx.conf
    let nginx_conf = "/etc/nginx/nginx.conf";
    if Path::new(nginx_conf).exists() {
        let content = fs::read_to_string(nginx_conf)
            .map_err(|e| format!("Failed to read nginx.conf: {}", e))?;
        
        if content.contains("ngx_http_headers_more_filter_module.so") {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Get nginx version
fn get_nginx_version() -> Result<String, String> {
    let output = Command::new("nginx")
        .arg("-v")
        .output()
        .map_err(|e| format!("Failed to get nginx version: {}", e))?;
    
    let version_output = String::from_utf8_lossy(&output.stderr);
    
    // Extract version number (format: nginx version: nginx/1.28.0)
    let version = version_output
        .split('/')
        .nth(1)
        .and_then(|v| v.split_whitespace().next())
        .ok_or("Failed to parse nginx version")?
        .to_string();
    
    Ok(version)
}

/// Install build dependencies
fn install_build_dependencies() -> Result<(), String> {
    println!("   → Installing build dependencies...");
    
    let packages = vec![
        "build-essential",
        "libpcre3-dev",
        "zlib1g-dev",
        "libssl-dev",
        "git",
    ];
    
    let mut cmd = Command::new("apt-get");
    cmd.arg("install")
       .arg("-y")
       .arg("-qq");
    
    for package in &packages {
        cmd.arg(package);
    }
    
    let output = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("Failed to install build dependencies: {}", e))?;
    
    if !output.status.success() {
        return Err("Failed to install build dependencies".to_string());
    }
    
    println!("   ✓ Build dependencies installed");
    Ok(())
}

/// Download and compile headers-more module
pub fn install_headers_more_module() -> Result<(), String> {
    println!("\n> Installing headers-more-nginx-module...\n");
    
    // Check if already installed
    if check_headers_more_module()? {
        println!("   ✓ headers-more module already installed and configured");
        return Ok(());
    }
    
    // Get nginx version
    let nginx_version = get_nginx_version()?;
    println!("   → Detected nginx version: {}", nginx_version);
    
    // Install build dependencies
    install_build_dependencies()?;
    
    // Create build directory
    let build_dir = "/usr/local/src/xynginc-build";
    fs::create_dir_all(build_dir)
        .map_err(|e| format!("Failed to create build directory: {}", e))?;
    
    // Download nginx source
    println!("   → Downloading nginx source...");
    let nginx_tar = format!("nginx-{}.tar.gz", nginx_version);
    let nginx_url = format!("http://nginx.org/download/{}", nginx_tar);
    
    let download_output = Command::new("wget")
        .args(&["-q", "-O", &format!("{}/{}", build_dir, nginx_tar), &nginx_url])
        .current_dir(build_dir)
        .output()
        .map_err(|e| format!("Failed to download nginx source: {}", e))?;
    
    if !download_output.status.success() {
        return Err(format!("Failed to download nginx source from {}", nginx_url));
    }
    
    // Extract nginx source
    println!("   → Extracting nginx source...");
    let extract_output = Command::new("tar")
        .args(&["-xzf", &nginx_tar])
        .current_dir(build_dir)
        .output()
        .map_err(|e| format!("Failed to extract nginx source: {}", e))?;
    
    if !extract_output.status.success() {
        return Err("Failed to extract nginx source".to_string());
    }
    
    // Clone headers-more module
    println!("   → Cloning headers-more-nginx-module...");
    let module_dir = format!("{}/headers-more-nginx-module", build_dir);
    
    // Remove if exists
    if Path::new(&module_dir).exists() {
        fs::remove_dir_all(&module_dir)
            .map_err(|e| format!("Failed to remove old module directory: {}", e))?;
    }
    
    let clone_output = Command::new("git")
        .args(&[
            "clone",
            "--quiet",
            "https://github.com/openresty/headers-more-nginx-module.git",
            &module_dir,
        ])
        .current_dir(build_dir)
        .output()
        .map_err(|e| format!("Failed to clone headers-more module: {}", e))?;
    
    if !clone_output.status.success() {
        return Err("Failed to clone headers-more module".to_string());
    }
    
    // Configure and compile module
    println!("   → Compiling module (this may take a moment)...");
    let nginx_src_dir = format!("{}/nginx-{}", build_dir, nginx_version);
    
    let configure_output = Command::new("./configure")
        .args(&[
            "--with-compat",
            &format!("--add-dynamic-module={}", module_dir),
        ])
        .current_dir(&nginx_src_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| format!("Failed to configure nginx: {}", e))?;
    
    if !configure_output.status.success() {
        let stderr = String::from_utf8_lossy(&configure_output.stderr);
        return Err(format!("Failed to configure nginx: {}", stderr));
    }
    
    let make_output = Command::new("make")
        .arg("modules")
        .current_dir(&nginx_src_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| format!("Failed to compile module: {}", e))?;
    
    if !make_output.status.success() {
        let stderr = String::from_utf8_lossy(&make_output.stderr);
        return Err(format!("Failed to compile module: {}", stderr));
    }
    
    // Install module
    println!("   → Installing module...");
    let module_file = format!("{}/objs/ngx_http_headers_more_filter_module.so", nginx_src_dir);
    
    if !Path::new(&module_file).exists() {
        return Err("Compiled module not found".to_string());
    }
    
    // Create modules directory (handle symlinks)
    let modules_dir = "/usr/share/nginx/modules";
    let real_modules_dir = if Path::new(modules_dir).is_symlink() {
        // Follow symlink
        fs::read_link(modules_dir)
            .map(|p| {
                if p.is_absolute() {
                    p
                } else {
                    Path::new("/usr/share/nginx").join(p)
                }
            })
            .unwrap_or_else(|_| Path::new("/usr/lib/nginx/modules").to_path_buf())
    } else {
        Path::new(modules_dir).to_path_buf()
    };
    
    // Create the actual directory
    fs::create_dir_all(&real_modules_dir)
        .map_err(|e| format!("Failed to create modules directory: {}", e))?;
    
    // Determine the final module path
    let module_dest = real_modules_dir.join("ngx_http_headers_more_filter_module.so");
    
    // Copy module
    fs::copy(&module_file, &module_dest)
        .map_err(|e| format!("Failed to copy module: {}", e))?;
    
    println!("   ✓ Module compiled and installed");
    
    // Configure nginx to load the module
    configure_nginx_module()?;
    
    // Cleanup build directory
    println!("   → Cleaning up build files...");
    fs::remove_dir_all(build_dir)
        .map_err(|e| format!("Failed to cleanup build directory: {}", e))?;
    
    println!("\n✅ headers-more module installed successfully!");
    
    Ok(())
}

/// Configure nginx.conf to load the headers-more module
fn configure_nginx_module() -> Result<(), String> {
    println!("   → Configuring nginx to load module...");
    
    let nginx_conf = "/etc/nginx/nginx.conf";
    
    if !Path::new(nginx_conf).exists() {
        return Err("nginx.conf not found".to_string());
    }
    
    let content = fs::read_to_string(nginx_conf)
        .map_err(|e| format!("Failed to read nginx.conf: {}", e))?;
    
    // Check if module is already loaded
    if content.contains("ngx_http_headers_more_filter_module.so") {
        println!("   ✓ Module already configured in nginx.conf");
        return Ok(());
    }
    
    // Add load_module directive at the beginning of the file
    let load_directive = "load_module modules/ngx_http_headers_more_filter_module.so;\n";
    
    // Find the first non-comment, non-empty line
    let mut new_content = String::new();
    let mut module_added = false;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Add module directive before the first non-comment directive
        if !module_added && !trimmed.is_empty() && !trimmed.starts_with('#') {
            new_content.push_str(load_directive);
            module_added = true;
        }
        
        new_content.push_str(line);
        new_content.push('\n');
    }
    
    // If we didn't add it yet (file only has comments), add it at the beginning
    if !module_added {
        new_content = format!("{}{}", load_directive, new_content);
    }
    
    // Write updated configuration
    fs::write(nginx_conf, new_content)
        .map_err(|e| format!("Failed to write nginx.conf: {}", e))?;
    
    println!("   ✓ Module configured in nginx.conf");
    
    // Test nginx configuration
    println!("   → Testing nginx configuration...");
    let test_output = Command::new("nginx")
        .arg("-t")
        .output()
        .map_err(|e| format!("Failed to test nginx configuration: {}", e))?;
    
    if !test_output.status.success() {
        let stderr = String::from_utf8_lossy(&test_output.stderr);
        return Err(format!("Nginx configuration test failed: {}", stderr));
    }
    
    println!("   ✓ Nginx configuration test passed");
    
    Ok(())
}
