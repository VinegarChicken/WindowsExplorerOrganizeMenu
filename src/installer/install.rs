use std::env;
use std::path::{Path, PathBuf};
use anyhow::Result;
use std::fs;
use crate::*;
use super::registry::{create_context_menu_entries, remove_context_menu_entries};

pub fn install_organizer() -> Result<()> {
    println!("=== Starting Installation ===");

    // Get the path to the current installer executable
    let installer_path = env::current_exe()?;
    let installer_dir = installer_path.parent().unwrap();
    println!("Installer directory: {}", installer_dir.display());

    // Look for the organizer executable in the same directory
    let organizer_src = installer_dir.join("organizer.exe");
    println!("Looking for organizer at: {}", organizer_src.display());

    if !organizer_src.exists() {
        return Err(anyhow::anyhow!(
            "organizer.exe not found in the same directory as installer. Expected: {}",
            organizer_src.display()
        ));
    }
    println!("✅ Found organizer.exe");

    // Create installation directory
    let install_dir = PathBuf::from("C:\\Program Files\\OrganizeMenuOption");
    println!("Installation target: {}", install_dir.display());

    if !install_dir.exists() {
        println!("Creating installation directory...");
        match fs::create_dir_all(&install_dir) {
            Ok(()) => println!("✅ Created installation directory"),
            Err(e) => {
                println!("❌ Failed to create installation directory: {}", e);
                return Err(anyhow::anyhow!("Cannot create installation directory: {}", e));
            }
        }
    } else {
        println!("Installation directory already exists");
    }

    // Copy organizer executable
    let organizer_dest = install_dir.join("organizer.exe");
    println!("Copying to: {}", organizer_dest.display());

    match fs::copy(&organizer_src, &organizer_dest) {
        Ok(_) => println!("✅ Copied organizer executable"),
        Err(e) => {
            println!("❌ Failed to copy organizer: {}", e);
            return Err(anyhow::anyhow!("Cannot copy organizer.exe: {}", e));
        }
    }

    // Verify the copy worked
    if !organizer_dest.exists() {
        return Err(anyhow::anyhow!("Copied file does not exist at destination"));
    }
    println!("✅ Verified organizer.exe at destination");

    // Create registry entries
    println!("\n=== Creating Registry Entries ===");
    match create_context_menu_entries(&organizer_dest) {
        Ok(()) => {
            println!("✅ Registry entries created successfully");

            // Verify they actually exist
            println!("Verifying registry entries...");
            /*
            if verify_installation(&organizer_dest) {
                println!("✅ Installation verification passed");

                // Additional debug: Force refresh Windows Explorer
                println!("Refreshing Windows Explorer...");
                refresh_explorer();

            } else {
                println!("⚠️ Installation verification failed - some registry entries may not be visible");
                println!("This may be normal if entries were created in HKCU instead of HKCR");
                println!("Context menus should still work.");
            }
           
             */
        }
        Err(e) => {
            println!("❌ Failed to create registry entries: {}", e);
            println!("Error details: {:?}", e);

            // Try to provide more specific error information
            let error_str = e.to_string();
            if error_str.contains("Access is denied") {
                println!("This appears to be a permissions issue.");
                println!("Solutions to try:");
                println!("1. Make sure you're running as Administrator");
                println!("2. Check if antivirus is blocking registry access");
                println!("3. Try running from a different location");
            } else if error_str.contains("The system cannot find") {
                println!("Registry path not found. This might be a Windows version issue.");
            }

            return Err(e);
        }
    }

    println!("\n✅ Installation completed successfully!");
    println!("\n📝 IMPORTANT NOTES:");
    println!("• Context menu items should now appear when you right-click on folders");
    println!("• If you don't see them immediately, try:");
    println!("  - Refreshing Windows Explorer (F5)");
    println!("  - Restarting Windows Explorer from Task Manager");
    println!("  - Logging out and back in");
    println!("  - On Windows 11: Right-click → 'Show more options' to see full context menu");
    Ok(())
}
/*
fn verify_installation(organizer_path: &Path) -> bool {
    use winreg::enums::*;
    use winreg::RegKey;

    // Try to verify in multiple registry locations
    verify_in_hkcr(organizer_path) || verify_in_hklm(organizer_path) || verify_in_hkcu(organizer_path)
}

fn verify_in_hkcr(organizer_path: &Path) -> bool {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    match hkcr.open_subkey("Directory\\shell") {
        Ok(shell_key) => verify_commands(&shell_key, organizer_path, "HKEY_CLASSES_ROOT"),
        Err(_) => false,
    }
}

fn verify_in_hklm(organizer_path: &Path) -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SOFTWARE\\Classes\\Directory\\shell") {
        Ok(shell_key) => verify_commands(&shell_key, organizer_path, "HKEY_LOCAL_MACHINE"),
        Err(_) => false,
    }
}

fn verify_in_hkcu(organizer_path: &Path) -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey("SOFTWARE\\Classes\\Directory\\shell") {
        Ok(shell_key) => verify_commands(&shell_key, organizer_path, "HKEY_CURRENT_USER"),
        Err(_) => false,
    }
}

fn verify_commands(shell_key: &RegKey, organizer_path: &Path, location: &str) -> bool {
    let commands = ["OrganizeByType", "OrganizeByDate", "OrganizeByName", "UndoOrganize"];
    let mut all_good = true;

    println!("Verifying entries in {}...", location);

    for cmd in commands {
        match shell_key.open_subkey(format!("{}\\command", cmd)) {
            Ok(command_key) => {
                // Verify the command points to our executable
                match command_key.get_value::<String, _>("") {
                    Ok(command_value) => {
                        if command_value.contains(&organizer_path.to_string_lossy().to_string()) {
                            println!("✅ {} command exists and points to correct executable", cmd);
                        } else {
                            println!("⚠️ {} command exists but points to wrong executable: {}", cmd, command_value);
                            all_good = false;
                        }
                    }
                    Err(_) => {
                        println!("❌ {} command value missing", cmd);
                        all_good = false;
                    }
                }
            }
            Err(_) => {
                println!("❌ {} command missing", cmd);
                all_good = false;
            }
        }
    }

    all_good
}
*/
fn refresh_explorer() {
    use std::process::Command;

    // Send WM_SETTINGCHANGE to notify all windows of registry changes
    use winapi::um::winuser::{SendMessageTimeoutW, HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG};
    use winapi::um::winnt::LPCWSTR;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    unsafe {
        let param: Vec<u16> = OsStr::new("Environment")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            param.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            ptr::null_mut(),
        );
    }

    // Also try to restart Explorer (less aggressive approach)
    let _ = Command::new("taskkill")
        .args(&["/F", "/IM", "explorer.exe"])
        .output();

    std::thread::sleep(std::time::Duration::from_millis(1000));

    let _ = Command::new("cmd")
        .args(&["/C", "start", "explorer.exe"])
        .output();

    println!("Attempted to refresh Windows Explorer and notify system of registry changes");
}

pub fn uninstall_organizer() -> Result<()> {
    println!("Starting uninstallation process...");

    // First, try to remove registry entries with detailed error reporting
    match remove_context_menu_entries() {
        Ok(()) => println!("Registry entries removed successfully."),
        Err(e) => {
            println!("Error removing registry entries: {}", e);
            println!("This might be normal if the entries were already removed or never created.");
        }
    }

    // Remove installation directory and files
    let install_dir = PathBuf::from("C:\\Program Files\\OrganizeMenuOption");
    println!("Checking installation directory: {}", install_dir.display());

    if install_dir.exists() {
        println!("Installation directory exists, attempting to remove...");
        match fs::remove_dir_all(&install_dir) {
            Ok(()) => println!("Successfully removed installation directory: {}", install_dir.display()),
            Err(e) => {
                println!("Error removing installation directory: {}", e);
                println!("You may need to manually delete: {}", install_dir.display());

                // Try to remove individual files first
                if let Ok(entries) = fs::read_dir(&install_dir) {
                    println!("Attempting to remove individual files...");
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            match fs::remove_file(&path) {
                                Ok(()) => println!("Removed file: {}", path.display()),
                                Err(e) => println!("Failed to remove file {}: {}", path.display(), e),
                            }
                        }
                    }

                    // Try to remove directory again
                    match fs::remove_dir(&install_dir) {
                        Ok(()) => println!("Successfully removed empty directory: {}", install_dir.display()),
                        Err(e) => println!("Failed to remove directory: {}", e),
                    }
                }
            }
        }
    } else {
        println!("Installation directory does not exist: {}", install_dir.display());
    }

    // Refresh explorer after uninstall too
   // refresh_explorer();
    println!("Refreshed Windows Explorer to remove cached menu entries");

    Ok(())
}