use std::path::Path;
use anyhow::Result;
use tokio::fs;

use crate::organizer::{OrganizeAction, get_log_path};

pub async fn undo_last_action(folder: &Path) -> Result<()> {
    let log_path = get_log_path(folder);

    if !log_path.exists() {
        println!("No organization history found for this folder.");
        return Ok(());
    }

    let content = fs::read_to_string(&log_path).await?;
    let mut actions: Vec<OrganizeAction> = serde_json::from_str(&content)?;

    if actions.is_empty() {
        println!("No actions to undo.");
        return Ok(());
    }

    // Get the last action
    let last_action = actions.pop().unwrap();
    println!("Undoing {} operation with {} file moves...", last_action.action_type, last_action.moves.len());

    let mut successful_undos = 0;
    let mut failed_undos = 0;

    // Reverse all moves
    for file_move in last_action.moves.iter().rev() {
        if file_move.to.exists() {
            match fs::rename(&file_move.to, &file_move.from).await {
                Ok(()) => successful_undos += 1,
                Err(e) => {
                    eprintln!("Failed to move {} back to {}: {}",
                              file_move.to.display(),
                              file_move.from.display(),
                              e);
                    failed_undos += 1;
                }
            }
        } else {
            eprintln!("File not found for undo: {}", file_move.to.display());
            failed_undos += 1;
        }
    }

    // Try to remove empty directories created during organization
    remove_empty_dirs(folder).await;

    // Update the log file
    if actions.is_empty() {
        let _ = fs::remove_file(&log_path).await;
    } else {
        let content = serde_json::to_string_pretty(&actions)?;
        fs::write(&log_path, content).await?;
    }

    println!("Undo completed: {} successful, {} failed", successful_undos, failed_undos);
    Ok(())
}

async fn remove_empty_dirs(folder: &Path) {
    if let Ok(mut entries) = fs::read_dir(folder).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() && !path.file_name().unwrap().to_string_lossy().starts_with('.') {
                // Try to remove if empty
                if let Ok(mut sub_entries) = fs::read_dir(&path).await {
                    if sub_entries.next_entry().await.ok().flatten().is_none() {
                        let _ = fs::remove_dir(&path).await;
                    }
                }
            }
        }
    }
}