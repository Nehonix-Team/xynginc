use std::fs;
use std::path::Path;

use crate::mods::constants::{BACKUP_DIR, NGINX_SITES_AVAILABLE, NGINX_SITES_ENABLED};
use crate::mods::logger::log_success;

pub fn create_backup() -> Result<(), String> {
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

    log_success(&format!("   ✓ Backup created: {}", backup_path));
    Ok(())
}

pub fn copy_directory(src: &str, dst: &str) -> Result<(), String> {
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

pub fn restore_latest_backup() -> Result<(), String> {
    let backups = list_backups()?;
    
    if backups.is_empty() {
        return Err("No backups available".to_string());
    }
    
    let latest = &backups[0];
    restore_backup(latest)?;
    
    Ok(())
}

pub fn list_backups() -> Result<Vec<String>, String> {
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

pub fn restore_backup(backup_id: &str) -> Result<(), String> {
    use crate::mods::logger::log_step;
    
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

    log_step(&format!("🔄 Restoring from backup: {}", backup_path));

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

    log_success("✓ Backup restored successfully");
    
    Ok(())
}
