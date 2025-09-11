#![windows_subsystem = "windows"]
use clap::{Arg, Command};
use std::path::PathBuf;
use anyhow::Result;

mod organizer;
mod undo;

use organizer::{by_type, by_date, by_name, by_modified_date, by_size, flatten, remove_duplicates};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("organizer")
        .version("1.0")
        .about("File organization tool for Windows Explorer context menu")
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_name("MODE")
                .help("Organization mode: type, date, modified_date, name, size, flatten, remove_duplicates, or undo")
                .required(true)
        )
        .arg(
            Arg::new("path")
                .help("Target folder path")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::new("ranges")
                .short('r')
                .long("ranges")
                .value_name("NUMBER")
                .help("Number of alphabetical ranges for name organization (default: 4)")
                .default_value("4")
        )
        .get_matches();

    let mode = matches.get_one::<String>("mode").unwrap();
    let path = PathBuf::from(matches.get_one::<String>("path").unwrap());
    let ranges: usize = matches.get_one::<String>("ranges").unwrap().parse().unwrap_or(4);

    if !path.exists() || !path.is_dir() {
        eprintln!("Error: Path does not exist or is not a directory: {}", path.display());
        std::process::exit(1);
    }

    match mode.as_str() {
        "type" => by_type::organize_by_type(&path).await?,
        "date" => by_date::organize_by_date(&path).await?,
        "modified_date" => by_modified_date::organize_by_modified_date(&path).await?,
        "name" => by_name::organize_by_name(&path, ranges).await?,
        "size" => by_size::organize_by_size(&path).await?,
        "flatten" => flatten::flatten_folder(&path).await?,
        "remove_duplicates" => remove_duplicates::remove_duplicates(&path).await?,
        "undo" => undo::undo_last_action(&path).await?,
        _ => {
            eprintln!("Error: Invalid mode. Use: type, date, name, size, flatten, remove_duplicates, or undo");
            std::process::exit(1);
        }
    }

    Ok(())
}