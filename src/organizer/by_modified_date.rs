use std::path::{Path, PathBuf};
use anyhow::Result;
use tokio::fs;
use chrono::{DateTime, Utc};

use super::{OrganizeAction, FileMove, log_action, handle_name_conflict};

pub async fn organize_by_modified_date(folder: &Path) -> Result<()> {
    let mut entries = fs::read_dir(folder).await?;
    let mut moves = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() || path.file_name().unwrap().to_string_lossy().starts_with('.') {
            continue;
        }

        let metadata = fs::metadata(&path).await?;
        let modified = metadata.modified()?;
        let datetime: DateTime<Utc> = modified.into();

        let year = datetime.format("%Y").to_string();
        let month = datetime.format("%Y-%m").to_string();

        let year_folder = folder.join(&year);
        if !year_folder.exists() {
            fs::create_dir(&year_folder).await?;
        }

        let month_folder = year_folder.join(&month);
        if !month_folder.exists() {
            fs::create_dir(&month_folder).await?;
        }

        let new_path = month_folder.join(path.file_name().unwrap());
        let new_path = handle_name_conflict(&new_path).await?;

        fs::rename(&path, &new_path).await?;
        moves.push(FileMove {
            from: path,
            to: new_path,
        });
    }

    let action = OrganizeAction {
        timestamp: Utc::now(),
        action_type: "by_modified_date".to_string(),
        moves,
    };
    log_action(folder, action.clone()).await?;

    println!("Successfully organized {} files by modified date", action.moves.len());
    Ok(())
}