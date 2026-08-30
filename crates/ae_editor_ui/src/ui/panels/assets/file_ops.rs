// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Safe File & Directory Operations Module.
//!
//! Provides directory creation, asset renaming, secure deletion, and path migration
//! with cross-platform desktop integration and error isolation.
//!

use std::path::{Path, PathBuf};

/// Creates a new subfolder under the specified parent directory.
pub fn create_subfolder(parent: &Path, name: &str) -> std::io::Result<PathBuf> {
    let clean_name = name.trim();
    if clean_name.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Folder name cannot be empty",
        ));
    }

    let target_dir = parent.join(clean_name);
    std::fs::create_dir_all(&target_dir)?;
    log::info!("📁 Created folder: {}", target_dir.display());
    Ok(target_dir)
}

/// Renames a file or directory on disk to a new name.
/// Preserves the original file extension if renaming a file and the extension is omitted.
pub fn rename_asset_or_folder(target: &Path, new_name: &str) -> std::io::Result<PathBuf> {
    let clean_name = new_name.trim();
    if clean_name.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Target name cannot be empty",
        ));
    }

    let parent = target.parent().unwrap_or_else(|| Path::new("assets"));
    let final_name = if target.is_file() && !clean_name.contains('.') {
        if let Some(ext) = target.extension().and_then(|e| e.to_str()) {
            format!("{}.{}", clean_name, ext)
        } else {
            clean_name.to_string()
        }
    } else {
        clean_name.to_string()
    };

    let new_path = parent.join(final_name);
    std::fs::rename(target, &new_path)?;
    log::info!(
        "🔄 Renamed '{}' -> '{}'",
        target.display(),
        new_path.display()
    );
    Ok(new_path)
}

/// Deletes a file or directory from the file system.
/// Directories are removed recursively.
pub fn delete_asset_or_folder(target: &Path) -> std::io::Result<()> {
    if target.is_dir() {
        std::fs::remove_dir_all(target)?;
        log::info!("🗑️ Removed directory: {}", target.display());
    } else if target.is_file() {
        std::fs::remove_file(target)?;
        log::info!("🗑️ Removed file: {}", target.display());
    }
    Ok(())
}

/// Moves a file or directory into a destination directory.
pub fn move_asset(source: &Path, destination_dir: &Path) -> std::io::Result<PathBuf> {
    let file_name = source.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid source path")
    })?;
    let target_path = destination_dir.join(file_name);
    std::fs::rename(source, &target_path)?;
    log::info!(
        "📦 Moved '{}' -> '{}'",
        source.display(),
        target_path.display()
    );
    Ok(target_path)
}

/// Opens the specified file or folder in the operating system's native file explorer.
pub fn open_in_file_explorer(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path.to_string_lossy().as_ref())
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path.to_string_lossy().as_ref())
            .spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path.to_string_lossy().as_ref())
            .spawn()?;
    }
    Ok(())
}