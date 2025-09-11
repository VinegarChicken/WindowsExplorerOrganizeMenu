use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use anyhow::Result;
use winreg::enums::*;
use winreg::RegKey;

mod registry;
mod elevation;
mod install;

use install::{install_organizer, uninstall_organizer};

fn main() -> Result<()> {
    // Check if running as admin
    if !elevation::is_elevated() {
        println!("This installer requires administrator privileges.");
        println!("Requesting elevation...");
        elevation::restart_as_admin()?;
        return Ok(());
    }

    println!("=== Organize Menu Option Installer ===");
    println!("1. Install");
    println!("2. Uninstall");
    print!("Choose an option (1-2): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => {
            println!("\nInstalling Organize Menu Option...");
            install_organizer()?;
            println!("Installation completed successfully!");
            println!("You can now right-click on any folder and select 'Organize' from the context menu.");
        }
        "2" => {
            println!("\nUninstalling Organize Menu Option...");
            uninstall_organizer()?;
            println!("Uninstallation completed successfully!");
        }
        _ => {
            println!("Invalid option. Exiting.");
            std::process::exit(1);
        }
    }

    println!("Press Enter to exit...");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}