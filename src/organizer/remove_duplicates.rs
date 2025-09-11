use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::Result;
use tokio::fs;
use tokio::io::AsyncReadExt;
use sha2::{Sha256, Digest};
use chrono::Utc;

use super::{OrganizeAction, FileMove, log_action};

pub async fn remove_duplicates(folder: &Path) -> Result<()> {
    let mut file_hashes: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut files_to_delete = Vec::new();
    let mut moves = Vec::new();

    let mut all_files = Vec::new();
    collect_all_files_recursively(folder, &mut all_files).await?;

    for path in &all_files {
        if let Ok(hash) = calculate_hash(path).await {
            file_hashes.entry(hash).or_default().push(path.clone());
        }
    }

    for (_, paths) in file_hashes {
        if paths.len() > 1 {
            for path_to_delete in paths.into_iter().skip(1) {
                if let Ok(_) = fs::remove_file(&path_to_delete).await {
                    files_to_delete.push(path_to_delete);
                }
            }
        }
    }

    for path in files_to_delete {
        moves.push(FileMove {
            from: path.clone(),
            to: PathBuf::from(format!("DELETED::{}", path.display())),
        });
    }

    let action = OrganizeAction {
        timestamp: Utc::now(),
        action_type: "remove_duplicates".to_string(),
        moves,
    };
    log_action(folder, action.clone()).await?;

    println!("Successfully removed {} duplicate files", action.moves.len());
    Ok(())
}

async fn collect_all_files_recursively(current_dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(current_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            Box::pin(collect_all_files_recursively(&path, files)).await?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

async fn calculate_hash(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];

    while let Ok(n) = file.read(&mut buffer).await {
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }

    let hash_bytes = hasher.finalize();
    Ok(hash_bytes.iter().map(|b| format!("{:02x}", b)).collect())
}