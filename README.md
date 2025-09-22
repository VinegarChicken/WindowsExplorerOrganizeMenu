# File Organizer

A Windows file organization tool that adds convenient context menu options for organizing folders directly from Windows Explorer.

## Features

- **Organize by File Type** - Groups files into folders like Images, Documents, Videos, Audio, etc.
- **Organize by Date Created** - Creates year/month folder structure based on file creation dates
- **Organize by Date Modified** - Creates year/month folder structure based on file modification dates
- **Organize by File Size** - Groups files into Small (0-1MB), Medium (1MB-100MB), Large (100MB-1GB), and Huge (1GB+) folders
- **Organize by Name** - Sorts files into alphabetical ranges (A-F, G-M, etc.)
- **Flatten Folder Structure** - Moves all files from subdirectories to the root folder
- **Remove Duplicate Files** - Finds and removes duplicate files based on content hash
- **Undo Last Organization** - Reverses the last organization operation

## Installation

1. Download the installer and organizer executable
2. Run the installer as Administrator
3. Choose "Install" from the menu
4. The installer will:
   - Copy the organizer to `C:\Program Files\OrganizeMenuOption\`
   - Add context menu entries to Windows Explorer

## Usage

1. Right-click on any folder in Windows Explorer
2. Select "Organize" from the context menu
3. Choose your desired organization method
4. Files will be automatically organized with a backup log created

On Windows 11, you may need to click "Show more options" to see the full context menu.

## Command Line Usage

You can also run the organizer directly from the command line:

```bash
organizer.exe --mode <MODE> <FOLDER_PATH>
```

Available modes:
- `type` - Organize by file type
- `date` - Organize by creation date
- `modified_date` - Organize by modification date
- `name` - Organize by name (use `--ranges N` to specify number of alphabetical groups)
- `size` - Organize by file size
- `flatten` - Flatten folder structure
- `remove_duplicates` - Remove duplicate files
- `undo` - Undo last organization

Example:
```bash
organizer.exe --mode type "C:\Users\Username\Downloads"
organizer.exe --mode name --ranges 6 "C:\Users\Username\Documents"
```

## Safety Features

- **Undo Functionality** - Each organization creates a log file that allows you to undo operations
- **Name Conflict Resolution** - Automatically handles duplicate filenames by adding numbers
- **Hidden Files Skipped** - System and hidden files are left untouched
- **Non-Destructive** - Files are moved, not copied or deleted (except for duplicate removal)

## Uninstallation

1. Run the installer as Administrator
2. Choose "Uninstall" from the menu
3. The installer will remove all files and registry entries

## System Requirements

- Windows 10/11
- Administrator privileges for installation
- .NET Framework (included with Windows)

## Technical Details

- Built with Rust for performance and safety
- Async file operations for handling large directories
- SHA-256 hashing for duplicate detection
- JSON logging for undo functionality
- Windows Registry integration for context menus

## License

This software is provided as-is for personal and commercial use.