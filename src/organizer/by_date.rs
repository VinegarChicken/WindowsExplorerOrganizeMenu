use std::path::{Path, PathBuf};
use anyhow::Result;
use tokio::fs;
use chrono::{DateTime, Utc};

use super::{OrganizeAction, FileMove, log_action};

pub async fn organize_by_date(folder: &Path) -> Result<()> {
    let mut entries = fs::read_dir(folder).await?;
    let mut moves = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Skip directories and hidden files
        if path.is_dir() || path.file_name().unwrap().to_string_lossy().starts_with('.') {
            continue;
        }

        let metadata = fs::metadata(&path).await?;
        let created = metadata.created()?;
        let datetime: DateTime<Utc> = created.into();

        // Create year folder
        let year = datetime.format("%Y").to_string();
        let year_folder = folder.join(&year);
        if !year_folder.exists() {
            fs::create_dir(&year_folder).await?;
        }

        // Create month folder
        let month = datetime.format("%Y-%m").to_string();
        let month_folder = year_folder.join(&month);
        if !month_folder.exists() {
            fs::create_dir(&month_folder).await?;
        }

        // Move file
        let new_path = month_folder.join(path.file_name().unwrap());
        let new_path = handle_name_conflict(&new_path).await?;

        fs::rename(&path, &new_path).await?;
        moves.push(FileMove {
            from: path,
            to: new_path,
        });
    }

    // Log the action
    let action = OrganizeAction {
        timestamp: Utc::now(),
        action_type: "by_date".to_string(),
        moves,
    };
    log_action(folder, action.clone()).await?;

    println!("Successfully organized {} files by date", action.moves.len());
    Ok(())
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