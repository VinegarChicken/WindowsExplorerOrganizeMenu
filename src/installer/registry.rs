use std::path::Path;
use anyhow::Result;
use winreg::enums::*;
use winreg::RegKey;

pub fn create_context_menu_entries(organizer_path: &Path) -> Result<()> {
    println!("Creating context menu entries...");

    let result = try_create_in_hkcr(organizer_path)
        .or_else(|_| try_create_in_hklm(organizer_path))
        .or_else(|_| try_create_in_hkcu(organizer_path));

    match result {
        Ok(()) => {
            println!("Registry entries created successfully.");
            Ok(())
        }
        Err(e) => {
            println!("Failed to create registry entries in all attempted locations.");
            Err(e)
        }
    }
}

fn try_create_in_hkcr(organizer_path: &Path) -> Result<()> {
    println!("Attempting to create entries in HKEY_CLASSES_ROOT...");
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    let directory_key = match hkcr.open_subkey_with_flags("Directory\\Background", KEY_READ | KEY_WRITE) {
        Ok(key) => key,
        Err(e) => {
            println!("Cannot access Directory\\Background key: {}", e);
            return Err(anyhow::anyhow!("Cannot access HKCR\\Directory\\Background: {}", e));
        }
    };

    let directory_shell = match directory_key.create_subkey_with_flags("shell", KEY_READ | KEY_WRITE) {
        Ok((key, _)) => key,
        Err(e) => {
            println!("Cannot create/access shell key: {}", e);
            return Err(anyhow::anyhow!("Cannot access HKCR\\Directory\\Background\\shell: {}", e));
        }
    };

    create_menu_entries(&directory_shell, organizer_path)?;
    println!("✅ Successfully created entries in HKEY_CLASSES_ROOT");
    Ok(())
}

fn try_create_in_hklm(organizer_path: &Path) -> Result<()> {
    println!("Attempting to create entries in HKEY_LOCAL_MACHINE...");
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    let software_key = hklm.open_subkey_with_flags("SOFTWARE", KEY_READ | KEY_WRITE)?;
    let classes_key = software_key.create_subkey_with_flags("Classes", KEY_READ | KEY_WRITE)?.0;
    let directory_key = classes_key.create_subkey_with_flags("Directory\\Background", KEY_READ | KEY_WRITE)?.0;
    let shell_key = directory_key.create_subkey_with_flags("shell", KEY_READ | KEY_WRITE)?.0;

    create_menu_entries(&shell_key, organizer_path)?;
    println!("✅ Successfully created entries in HKEY_LOCAL_MACHINE");
    Ok(())
}

fn try_create_in_hkcu(organizer_path: &Path) -> Result<()> {
    println!("Attempting to create entries in HKEY_CURRENT_USER...");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let software_key = hkcu.open_subkey_with_flags("SOFTWARE", KEY_READ | KEY_WRITE)?;
    let classes_key = software_key.create_subkey_with_flags("Classes", KEY_READ | KEY_WRITE)?.0;
    let directory_key = classes_key.create_subkey_with_flags("Directory\\Background", KEY_READ | KEY_WRITE)?.0;
    let shell_key = directory_key.create_subkey_with_flags("shell", KEY_READ | KEY_WRITE)?.0;

    create_menu_entries(&shell_key, organizer_path)?;
    println!("✅ Successfully created entries in HKEY_CURRENT_USER");
    Ok(())
}

fn create_menu_entries(shell_key: &RegKey, organizer_path: &Path) -> Result<()> {
    let organizer_path_str = organizer_path.to_string_lossy().to_string();

    let (parent_key, _) = shell_key.create_subkey_with_flags("OrganizeMenu", KEY_READ | KEY_WRITE)
        .map_err(|e| anyhow::anyhow!("Failed to create OrganizeMenu key: {}", e))?;

    parent_key.set_value("MUIVerb", &"Organize")
        .map_err(|e| anyhow::anyhow!("Failed to set MUIVerb for OrganizeMenu: {}", e))?;

    parent_key.set_value("SubCommands", &"")
        .map_err(|e| anyhow::anyhow!("Failed to set SubCommands for OrganizeMenu: {}", e))?;

    let (submenu_shell, _) = parent_key.create_subkey_with_flags("shell", KEY_READ | KEY_WRITE)
        .map_err(|e| anyhow::anyhow!("Failed to create shell under OrganizeMenu: {}", e))?;

    create_single_entry(&submenu_shell, "OrganizeByType", "Organize by File Type", &organizer_path_str, "type")?;
    create_single_entry(&submenu_shell, "OrganizeByDate", "Organize by Date Created", &organizer_path_str, "date")?;
    create_single_entry(&submenu_shell, "OrganizeByModifiedDate", "Organize by Date Modified", &organizer_path_str, "modified_date")?;
    create_single_entry(&submenu_shell, "OrganizeBySize", "Organize by File Size", &organizer_path_str, "size")?;
    create_single_entry(&submenu_shell, "OrganizeByName", "Organize by Name (Alphabetical)", &organizer_path_str, "name")?;
    create_single_entry(&submenu_shell, "FlattenFolder", "Flatten Folder Structure", &organizer_path_str, "flatten")?;
    create_single_entry(&submenu_shell, "RemoveDuplicates", "Remove Duplicate Files", &organizer_path_str, "remove_duplicates")?;
    create_single_entry(&submenu_shell, "UndoOrganize", "Undo Last Organization", &organizer_path_str, "undo")?;

    println!("✅ Created Organize submenu with child entries");
    Ok(())
}

fn create_single_entry(
    shell_key: &RegKey,
    key_name: &str,
    display_name: &str,
    organizer_path: &str,
    mode: &str,
) -> Result<()> {
    let (entry_key, _) = shell_key.create_subkey_with_flags(key_name, KEY_READ | KEY_WRITE)
        .map_err(|e| anyhow::anyhow!("Failed to create {} key: {}", key_name, e))?;

    entry_key.set_value("MUIVerb", &display_name)
        .map_err(|e| anyhow::anyhow!("Failed to set display name for {}: {}", key_name, e))?;

    let (command_key, _) = entry_key.create_subkey_with_flags("command", KEY_READ | KEY_WRITE)
        .map_err(|e| anyhow::anyhow!("Failed to create command key for {}: {}", key_name, e))?;

    let command_value = format!("\"{}\" --mode {} \"%V\"", organizer_path, mode);
    command_key.set_value("", &command_value)
        .map_err(|e| anyhow::anyhow!("Failed to set command value for {}: {}", key_name, e))?;

    println!("✅ Created '{}' entry", display_name);
    Ok(())
}

pub fn remove_context_menu_entries() -> Result<()> {
    println!("Attempting to remove registry entries...");

    let mut removal_attempted = false;
    let mut any_success = false;

    if let Ok(_) = try_remove_from_hkcr() {
        removal_attempted = true;
        any_success = true;
        println!("✅ Removed entries from HKEY_CLASSES_ROOT");
    }

    if let Ok(_) = try_remove_from_hklm() {
        removal_attempted = true;
        any_success = true;
        println!("✅ Removed entries from HKEY_LOCAL_MACHINE");
    }

    if let Ok(_) = try_remove_from_hkcu() {
        removal_attempted = true;
        any_success = true;
        println!("✅ Removed entries from HKEY_CURRENT_USER");
    }

    if !removal_attempted {
        return Err(anyhow::anyhow!("Could not access any registry location for removal"));
    }

    if any_success {
        println!("Registry cleanup completed successfully.");
    } else {
        println!("No registry entries found to remove (this is normal if already uninstalled).");
    }

    Ok(())
}

fn try_remove_from_hkcr() -> Result<()> {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let directory_shell = hkcr.open_subkey_with_flags("Directory\\Background\\shell", KEY_WRITE)?;
    remove_entries(&directory_shell)
}

fn try_remove_from_hklm() -> Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let shell_key = hklm.open_subkey_with_flags("SOFTWARE\\Classes\\Directory\\Background\\shell", KEY_WRITE)?;
    remove_entries(&shell_key)
}

fn try_remove_from_hkcu() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let shell_key = hkcu.open_subkey_with_flags("SOFTWARE\\Classes\\Directory\\Background\\shell", KEY_WRITE)?;
    remove_entries(&shell_key)
}

fn remove_entries(shell_key: &RegKey) -> Result<()> {
    let items_to_remove = vec!["OrganizeMenu"];

    for item in items_to_remove {
        match shell_key.open_subkey(item) {
            Ok(_) => {
                match shell_key.delete_subkey_all(item) {
                    Ok(()) => println!("Successfully deleted {} registry key", item),
                    Err(e) => println!("Warning: Error deleting {} key: {}", item, e),
                }
            }
            Err(_) => {
                // Key doesn't exist, which is fine
            }
        }
    }
    Ok(())
}