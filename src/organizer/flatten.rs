use std::path::{Path, PathBuf};
use anyhow::Result;
use tokio::fs;
use tokio::io::ErrorKind;
use chrono::Utc;

use super::{OrganizeAction, FileMove, log_action, handle_name_conflict};

pub async fn flatten_folder(folder: &Path) -> Result<()> {
    let mut moves = Vec::new();
    let mut dirs_to_remove = Vec::new();

    collect_files_recursively(folder, folder, &mut moves).await?;

    for file_move in &moves {
        match fs::rename(&file_move.from, &file_move.to).await {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }

    collect_empty_dirs(folder, &mut dirs_to_remove).await?;

    dirs_to_remove.reverse();
    for dir in dirs_to_remove {
        let _ = fs::remove_dir(&dir).await;
    }

    let action = OrganizeAction {
        timestamp: Utc::now(),
        action_type: "flatten".to_string(),
        moves,
    };
    log_action(folder, action.clone()).await?;

    println!("Successfully flattened folder, moving {} files", action.moves.len());
    Ok(())
}

async fn collect_files_recursively(root: &Path, current_dir: &Path, moves: &mut Vec<FileMove>) -> Result<()> {
    let mut entries = fs::read_dir(current_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_dir() && !path.file_name().unwrap().to_string_lossy().starts_with('.') {
            Box::pin(collect_files_recursively(root, &path, moves)).await?;
        } else if path.is_file() {
            let relative_path = path.strip_prefix(root).unwrap();
            let new_file_name = relative_path.to_string_lossy().replace("\\", "___");
            let new_path = root.join(&new_file_name);

            moves.push(FileMove {
                from: path,
                to: new_path,
            });
        }
    }
    Ok(())
}

async fn collect_empty_dirs(path: &Path, dirs: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            Box::pin(collect_empty_dirs(&entry_path, dirs)).await?;
        }
    }

    let mut re_entries = fs::read_dir(path).await?;
    let is_empty = re_entries.next_entry().await?.is_none();

    if is_empty {
        dirs.push(path.to_path_buf());
    }

    Ok(())
}
