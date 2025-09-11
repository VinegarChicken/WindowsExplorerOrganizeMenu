use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::Result;
use tokio::fs;
use chrono::Utc;

use super::{OrganizeAction, FileMove, log_action};

pub async fn organize_by_type(folder: &Path) -> Result<()> {
    let mut entries = fs::read_dir(folder).await?;
    let mut moves = Vec::new();
    let mut type_folders: HashMap<String, PathBuf> = HashMap::new();

    // Define file type mappings
    let type_mappings = get_type_mappings();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Skip directories and hidden files
        if path.is_dir() || path.file_name().unwrap().to_string_lossy().starts_with('.') {
            continue;
        }

        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            let file_type = type_mappings.get(&ext).unwrap_or(&"Others".to_string()).clone();

            // Create type folder if it doesn't exist
            let type_folder = folder.join(&file_type);
            if !type_folders.contains_key(&file_type) {
                if !type_folder.exists() {
                    fs::create_dir(&type_folder).await?;
                }
                type_folders.insert(file_type.clone(), type_folder.clone());
            }

            // Move file
            let new_path = type_folder.join(path.file_name().unwrap());
            let new_path = handle_name_conflict(&new_path).await?;

            fs::rename(&path, &new_path).await?;
            moves.push(FileMove {
                from: path,
                to: new_path,
            });
        }
    }

    // Log the action
    let action = OrganizeAction {
        timestamp: Utc::now(),
        action_type: "by_type".to_string(),
        moves,
    };
    log_action(folder, action.clone()).await?;

    println!("Successfully organized {} files by type", action.moves.len());
    Ok(())
}

fn get_type_mappings() -> HashMap<String, String> {
    let mut mappings = HashMap::new();

    // Images
    for ext in ["jpg", "jpeg", "png", "gif", "bmp", "tiff", "svg", "webp", "ico"] {
        mappings.insert(ext.to_string(), "Images".to_string());
    }

    // Documents
    for ext in ["pdf", "doc", "docx", "txt", "rtf", "odt", "xls", "xlsx", "ppt", "pptx"] {
        mappings.insert(ext.to_string(), "Documents".to_string());
    }

    // Videos
    for ext in ["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v"] {
        mappings.insert(ext.to_string(), "Videos".to_string());
    }

    // Audio
    for ext in ["mp3", "wav", "flac", "aac", "ogg", "wma", "m4a"] {
        mappings.insert(ext.to_string(), "Audio".to_string());
    }

    // Archives
    for ext in ["zip", "rar", "7z", "tar", "gz", "bz2"] {
        mappings.insert(ext.to_string(), "Archives".to_string());
    }

    // Executables
    for ext in ["exe", "msi", "deb", "rpm", "dmg", "app"] {
        mappings.insert(ext.to_string(), "Executables".to_string());
    }

    mappings
}

async fn handle_name_conflict(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        return Ok(path.to_path_buf());
    }

    let parent = path.parent().unwrap();
    let filename = path.file_stem().unwrap().to_string_lossy();
    let extension = path.extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    for i in 1..1000 {
        let new_name = format!("{} ({}){}", filename, i, extension);
        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return Ok(new_path);
        }
    }

    Err(anyhow::anyhow!("Could not resolve name conflict for: {}", path.display()))
}