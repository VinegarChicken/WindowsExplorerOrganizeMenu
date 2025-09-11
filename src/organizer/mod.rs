pub mod by_type;
pub mod by_date;
pub mod by_name;
pub(crate) mod flatten;
pub(crate) mod remove_duplicates;
pub(crate) mod by_modified_date;
pub(crate) mod by_size;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrganizeAction {
    pub timestamp: DateTime<Utc>,
    pub action_type: String,
    pub moves: Vec<FileMove>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMove {
    pub from: PathBuf,
    pub to: PathBuf,
}

pub fn get_log_path(folder: &std::path::Path) -> PathBuf {
    folder.join(".organize_log.json")
}

pub async fn log_action(folder: &std::path::Path, action: OrganizeAction) -> anyhow::Result<()> {
    let log_path = get_log_path(folder);
    let mut actions: Vec<OrganizeAction> = if log_path.exists() {
        let content = tokio::fs::read_to_string(&log_path).await?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    actions.push(action);

    // Keep only last 10 actions to prevent log from growing too large
    if actions.len() > 10 {
        actions = actions.clone().into_iter().skip(actions.len() - 10).collect();
    }

    let content = serde_json::to_string_pretty(&actions)?;
    tokio::fs::write(&log_path, content).await?;

    // Hide the log file
    #[cfg(windows)]
    {
        use std::ffi::CString;
        use winapi::um::fileapi::SetFileAttributesA;
        use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;

        if let Ok(c_path) = CString::new(log_path.to_string_lossy().as_bytes()) {
            unsafe {
                SetFileAttributesA(c_path.as_ptr(), FILE_ATTRIBUTE_HIDDEN);
            }
        }
    }

    Ok(())
}

pub async fn handle_name_conflict(path: &Path) -> anyhow::Result<PathBuf> {
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