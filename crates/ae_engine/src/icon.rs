// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Aeon Engine Icon Loader Module
//!
//! This module provides cross-platform functionality to load and convert
//! image assets into winit-compatible Window Icons.
//!
//! It supports various file formats (PNG, JPG, etc.) through the `image` crate.
//!

use image::GenericImageView;
use winit::window::Icon;

/// Loads a window icon from the specified file path.
/// # Arguments
/// * `path` - A string slice that holds the relative or absolute path to the image file.
/// # Returns
/// * `Some(Icon)` - If the image was successfully loaded and converted to RGBA pixels.
/// * `None` - If the file was not found, couldn't be decoded, or conversion failed.
/// # Implementation Details
/// 1. Opens the image file.
/// 2. Converts it to RGBA8 format (required by winit).
/// 3. Extracts dimensions and raw pixel data.
/// 4. Creates a `winit::window::Icon`.
pub fn load_window_icon(path: &str) -> Option<Icon> {
    // 1. Open the image file using `image` crate
    let img = image::open(path).ok()?;

    // 2. Get dimensions
    let (width, height) = img.dimensions();

    // 3. Convert to RGBA8 (Standard for winit Icon)
    let rgba = img.to_rgba8().into_raw();

    // 4. Transform into winit::window::Icon
    Icon::from_rgba(rgba, width, height).ok()
}

/// Loads a window icon from raw image data (bytes).
/// This is preferred for cross-platform stability as it allows
/// embedding the icon in the binary using `include_bytes!`.
/// # Arguments
/// * `bytes` - The raw image data (e.g., contents of a .png file).
pub fn load_icon_from_memory(bytes: &[u8]) -> Option<Icon> {
    // 1. Load image from memory
    let img = image::load_from_memory(bytes).ok()?;

    // 2. Normalize to standard 256x256 window icon size with Lanczos3 filtering
    let (width, height) = img.dimensions();
    let img = if width > 256 || height > 256 {
        img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    let (w, h) = img.dimensions();
    let rgba = img.to_rgba8().into_raw();

    // 3. Transform into winit::window::Icon
    Icon::from_rgba(rgba, w, h).ok()
}

/// Ensures that the XDG Desktop entry and multi-resolution icons are registered
/// in accordance with the FreeDesktop.org standard.
/// This provides universal icon and application matching across ALL Linux Desktop
/// Environments (GNOME, KDE Plasma, XFCE, Cinnamon, MATE, LXQt, Hyprland, Sway, COSMIC, etc.)
/// without requiring manual setup scripts or desktop-environment-specific hacks.
#[cfg(target_os = "linux")]
pub fn ensure_xdg_desktop_integration(icon_bytes: &[u8]) {
    use std::path::PathBuf;

    let data_home = match std::env::var_os("XDG_DATA_HOME") {
        Some(val) => PathBuf::from(val),
        None => {
            if let Some(home) = std::env::var_os("HOME") {
                PathBuf::from(home).join(".local/share")
            } else {
                return;
            }
        }
    };

    let icon_dest = data_home.join("icons/hicolor/256x256/apps/com.aeonengine.Editor.png");
    let icon_titlebar_dest = data_home.join("icons/hicolor/22x22/apps/com.aeonengine.Editor.png");
    let desktop_dest = data_home.join("applications/com.aeonengine.Editor.desktop");

    // Fast check: if primary and titlebar icons already exist with identical bytes, skip regeneration
    let needs_update = match std::fs::read(&icon_dest) {
        Ok(existing) => existing != icon_bytes,
        Err(_) => true,
    } || !icon_titlebar_dest.exists()
        || !desktop_dest.exists();

    if !needs_update {
        return;
    }

    let bytes_owned = icon_bytes.to_vec();

    // Perform installation and cache update in a detached background thread to prevent any UI delay
    std::thread::spawn(move || {
        let app_ids = [
            "com.aeonengine.Editor",
            "com.aeengine.Editor",
            "ae_engine",
            "ae-engine",
            "aeon-engine",
            "aeon_engine",
        ];

        // 1. Automatically generate and install standard FreeDesktop hicolor resolutions
        // 16x16: task tray / menu
        // 22x22, 24x24: window titlebar / decoration (KDE, XFCE)
        // 32x32, 48x48: file managers / app launchers
        // 64x64, 128x128, 256x256, 512x512: dock / dash / HiDPI
        if let Ok(img) = image::load_from_memory(&bytes_owned) {
            for size in [16, 22, 24, 32, 48, 64, 128, 256, 512] {
                let size_dir = data_home.join(format!("icons/hicolor/{}x{}/apps", size, size));
                let _ = std::fs::create_dir_all(&size_dir);
                let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
                for app_id in app_ids {
                    let target_path = size_dir.join(format!("{}.png", app_id));
                    let _ = resized.save_with_format(&target_path, image::ImageFormat::Png);
                }
            }
        }

        // 2. Write standard FreeDesktop .desktop file
        if let Some(parent) = desktop_dest.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let desktop_content = "[Desktop Entry]\n\
            Type=Application\n\
            Name=Aeon Engine\n\
            Comment=Advanced cross-platform game engine editor\n\
            Exec=cargo run --release\n\
            Icon=com.aeonengine.Editor\n\
            Terminal=false\n\
            Categories=Development;GameDevelopment;\n\
            StartupNotify=true\n\
            StartupWMClass=com.aeonengine.Editor\n\
            Keywords=engine;gamedev;3d;rust;\n";
        let _ = std::fs::write(&desktop_dest, desktop_content);

        // Also update legacy desktop file name for compatibility
        let legacy_desktop = data_home.join("applications/com.aeengine.Editor.desktop");
        let _ = std::fs::write(&legacy_desktop, desktop_content);

        // 3. Trigger standard FreeDesktop / XDG cache refresh utilities silently
        let _ = std::process::Command::new("xdg-icon-resource")
            .args(["forceupdate"])
            .output();

        let hicolor_dir = data_home.join("icons/hicolor");
        if let Some(hicolor_str) = hicolor_dir.to_str() {
            let _ = std::process::Command::new("gtk-update-icon-cache")
                .args(["-f", "-t", hicolor_str])
                .output();
        }

        let _ = std::process::Command::new("update-desktop-database")
            .arg(data_home.join("applications"))
            .output();
    });
}