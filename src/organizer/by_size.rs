use std::path::{Path, PathBuf};
use anyhow::Result;
use tokio::fs;
use chrono::Utc;

use super::{OrganizeAction, FileMove, log_action, handle_name_conflict};

const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const GB: u64 = MB * 1024;

pub async fn organize_by_size(folder: &Path) -> Result<()> {
    let mut entries = fs::read_dir(folder).await?;
    let mut moves = Vec::new();

    // Create folders for different sizes
    let small_folder = folder.join("Small (0-1MB)");
    let medium_folder = folder.join("Medium (1MB-100MB)");
    let large_folder = folder.join("Large (100MB-1GB)");
    let huge_folder = folder.join("Huge (1GB+)");

    fs::create_dir_all(&small_folder).await?;
    fs::create_dir_all(&medium_folder).await?;
    fs::create_dir_all(&large_folder).await?;
    fs::create_dir_all(&huge_folder).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() || path.file_name().unwrap().to_string_lossy().starts_with('.') {
            continue;
        }

        let metadata = fs::metadata(&path).await?;
        let size = metadata.len();
        let destination_folder = if size < MB {
            &small_folder
        } else if size < 100 * MB {
            &medium_folder
        } else if size < GB {
            &large_folder
        } else {
            &huge_folder
        };

        let new_path = destination_folder.join(path.file_name().unwrap());
        let new_path = handle_name_conflict(&new_path).await?;

        fs::rename(&path, &new_path).await?;
        moves.push(FileMove {
            from: path,
            to: new_path,
        });
    }

    let action = OrganizeAction {
        timestamp: Utc::now(),
        action_type: "by_size".to_string(),
        moves,
    };
    log_action(folder, action.clone()).await?;

    println!("Successfully organized {} files by size", action.moves.len());
    Ok(())
}