// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Window icon loading and FreeDesktop XDG desktop integration for Aeon Launcher.
//!
//! Provides cross-platform icon generation for Winit windows and registers standard
//! application identifiers (`app_id`) on Wayland and X11 to ensure proper taskbar,
//! dock, and titlebar icon resolution across desktop environments.
//!

use image::GenericImageView;
use winit::window::Icon;

/// Decodes raw image bytes into a Winit-compatible `Icon` structure.
/// If the image exceeds standard 256x256 dimensions, it is downsampled with
/// high-fidelity Lanczos3 filtering for crisp display across taskbars and titlebars.
pub fn load_icon_from_memory(bytes: &[u8]) -> Option<Icon> {
    let img = image::load_from_memory(bytes).ok()?;
    let (width, height) = img.dimensions();
    let img = if width > 256 || height > 256 {
        img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let (w, h) = img.dimensions();
    let rgba = img.to_rgba8().into_raw();
    Icon::from_rgba(rgba, w, h).ok()
}

/// Registers the application icon and desktop entry in the FreeDesktop XDG hierarchy.
/// Ensures proper icon binding under Wayland and X11 compositors for matching `app_id`
/// entries (`com.aeonengine.Launcher`, `ae_launcher`, etc.).
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

    let icon_dest = data_home.join("icons/hicolor/256x256/apps/com.aeonengine.Launcher.png");
    let desktop_dest = data_home.join("applications/com.aeonengine.Launcher.desktop");

    let needs_update = match std::fs::read(&icon_dest) {
        Ok(existing) => existing != icon_bytes,
        Err(_) => true,
    } || !desktop_dest.exists();

    if !needs_update {
        return;
    }

    let bytes_owned = icon_bytes.to_vec();

    std::thread::spawn(move || {
        let app_ids = [
            "com.aeonengine.Launcher",
            "com.aeonengine.Hub",
            "ae_launcher",
            "aeon-launcher",
        ];

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

        if let Some(parent) = desktop_dest.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let desktop_content = "[Desktop Entry]\n\
            Type=Application\n\
            Name=Aeon Engine Hub\n\
            Comment=Aeon Engine Project Hub and Launcher\n\
            Exec=cargo run -p ae_launcher --release\n\
            Icon=com.aeonengine.Launcher\n\
            Terminal=false\n\
            Categories=Development;GameDevelopment;\n\
            StartupNotify=true\n\
            StartupWMClass=com.aeonengine.Launcher\n\
            Keywords=engine;gamedev;launcher;hub;aeon;\n";
        let _ = std::fs::write(&desktop_dest, desktop_content);

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