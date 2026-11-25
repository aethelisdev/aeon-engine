// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Cross-platform window icon loading and FreeDesktop XDG desktop entry installation.

use image::GenericImageView;
use winit::window::Icon;

/// Loads a window icon from an embedded encoded image byte slice (resized up to 256x256).
#[must_use]
pub fn load_icon_from_memory(bytes: &[u8]) -> Option<Icon> {
    let img = image::load_from_memory(bytes).ok()?;
    let (width, height) = img.dimensions();
    let img = if width > 256 || height > 256 {
        img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.to_rgba8().into_raw(), w, h).ok()
}

/// Ensures FreeDesktop `.desktop` entry and multi-resolution hicolor icons are installed on Linux.
#[cfg(target_os = "linux")]
pub fn ensure_xdg_desktop_integration(icon_bytes: &[u8]) {
    use std::path::PathBuf;

    let data_home = match std::env::var_os("XDG_DATA_HOME") {
        Some(val) => PathBuf::from(val),
        None => match std::env::var_os("HOME") {
            Some(home) => PathBuf::from(home).join(".local/share"),
            None => return,
        },
    };

    let icon_dest = data_home.join("icons/hicolor/256x256/apps/com.aeonengine.Editor.png");
    let icon_titlebar_dest = data_home.join("icons/hicolor/22x22/apps/com.aeonengine.Editor.png");
    let desktop_dest = data_home.join("applications/com.aeonengine.Editor.desktop");

    let needs_update = !matches!(std::fs::read(&icon_dest), Ok(existing) if existing == icon_bytes)
        || !icon_titlebar_dest.exists()
        || !desktop_dest.exists();
    if !needs_update {
        return;
    }

    let bytes_owned = icon_bytes.to_vec();
    std::thread::spawn(move || {
        let app_ids = [
            "com.aeonengine.Editor",
            "com.aeengine.Editor",
            "ae_editor_app",
            "ae-engine",
            "aeon-engine",
            "aeon_engine",
        ];

        if let Ok(img) = image::load_from_memory(&bytes_owned) {
            for size in [16, 22, 24, 32, 48, 64, 128, 256, 512] {
                let size_dir = data_home.join(format!("icons/hicolor/{size}x{size}/apps"));
                let _ = std::fs::create_dir_all(&size_dir);
                let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
                for app_id in app_ids {
                    let target_path = size_dir.join(format!("{app_id}.png"));
                    let _ = resized.save_with_format(&target_path, image::ImageFormat::Png);
                }
            }
        }

        if let Some(parent) = desktop_dest.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let desktop_content = "[Desktop Entry]\n\
            Type=Application\n\
            Name=Aeon Engine\n\
            Comment=Cross-platform 3D/2D game creation studio\n\
            Exec=ae_editor_app\n\
            Icon=com.aeonengine.Editor\n\
            Terminal=false\n\
            Categories=Development;GameDevelopment;\n\
            StartupNotify=true\n\
            StartupWMClass=com.aeonengine.Editor\n\
            Keywords=engine;gamedev;3d;rust;\n";
        let _ = std::fs::write(&desktop_dest, desktop_content);
        let legacy_desktop = data_home.join("applications/com.aeengine.Editor.desktop");
        let _ = std::fs::write(&legacy_desktop, desktop_content);

        let _ = std::process::Command::new("xdg-icon-resource")
            .args(["forceupdate"])
            .output();
        if let Some(hicolor_str) = data_home.join("icons/hicolor").to_str() {
            let _ = std::process::Command::new("gtk-update-icon-cache")
                .args(["-f", "-t", hicolor_str])
                .output();
        }
        let _ = std::process::Command::new("update-desktop-database")
            .arg(data_home.join("applications"))
            .output();
    });
}