use std::path::{Path, PathBuf};
use anyhow::Result;
use tokio::fs;
use chrono::Utc;

use super::{OrganizeAction, FileMove, log_action};

pub async fn organize_by_name(folder: &Path, num_ranges: usize) -> Result<()> {
    let mut entries = fs::read_dir(folder).await?;
    let mut moves = Vec::new();

    let ranges = create_alphabetical_ranges(num_ranges);

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Skip directories and hidden files
        if path.is_dir() || path.file_name().unwrap().to_string_lossy().starts_with('.') {
            continue;
        }

        if let Some(filename) = path.file_name() {
            let first_char = filename.to_string_lossy()
                .chars()
                .next()
                .unwrap_or('_')
                .to_ascii_uppercase();

            let range_folder_name = find_range_for_char(first_char, &ranges);
            let range_folder = folder.join(&range_folder_name);

            if !range_folder.exists() {
                fs::create_dir(&range_folder).await?;
            }

            // Move file
            let new_path = range_folder.join(filename);
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
        action_type: format!("by_name_{}", num_ranges),
        moves,
    };
    log_action(folder, action.clone()).await?;

    println!("Successfully organized {} files by name into {} ranges", action.moves.len(), num_ranges);
    Ok(())
}

fn create_alphabetical_ranges(num_ranges: usize) -> Vec<(char, char, String)> {
    let letters_per_range = 26 / num_ranges;
    let mut ranges = Vec::new();

    for i in 0..num_ranges {
        let start = ((i * letters_per_range) as u8 + b'A') as char;
        let end = if i == num_ranges - 1 {
            'Z'  // Last range gets all remaining letters
        } else {
            (((i + 1) * letters_per_range - 1) as u8 + b'A') as char
        };

        let folder_name = if start == end {
            format!("{}", start)
        } else {
            format!("{}-{}", start, end)
        };

        ranges.push((start, end, folder_name));
    }

    ranges
}

fn find_range_for_char(ch: char, ranges: &[(char, char, String)]) -> String {
    for (start, end, folder_name) in ranges {
        if ch >= *start && ch <= *end {
            return folder_name.clone();
        }
    }

    // For non-alphabetic characters
    "Other".to_string()
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