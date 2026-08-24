// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Context Menu & Right-Click Actions.
//!
//! Provides right-click operations such as Spawn, Open Scene,
//! Copy Path, and Reveal in File Explorer.
//!

use super::types::{AssetCategory, AssetItem};
use crate::ui::types::EngineUiAction;
use egui::Response;

/// Attaches the asset context menu to an egui Response.
pub fn attach_asset_context_menu(
    response: &Response,
    item: &AssetItem,
    ui_actions: &mut Vec<EngineUiAction>,
) {
    response.context_menu(|ui| {
        ui.set_min_width(180.0);
        ui.label(
            egui::RichText::new(&item.name)
                .strong()
                .color(item.category.badge_color()),
        );
        ui.separator();

        // 1. Primary Spawn / Open Action
        match item.category {
            AssetCategory::Models3D => {
                if let Some(handle) = item.model_handle {
                    if ui.button("➕ Spawn into Scene").clicked() {
                        ui_actions.push(EngineUiAction::SpawnModel(handle));
                        ui.close();
                    }
                } else {
                    ui.label(
                        egui::RichText::new("Drag into scene to load")
                            .italics()
                            .color(egui::Color32::from_gray(140)),
                    );
                }
            }
            AssetCategory::Textures2D => {
                if let Some(handle) = item.texture_handle {
                    if ui.button("🖼 Spawn as Sprite").clicked() {
                        ui_actions.push(EngineUiAction::SpawnSprite(handle));
                        ui.close();
                    }
                }
            }
            AssetCategory::Scenes => {
                if ui.button("🎬 Load Scene").clicked() {
                    ui_actions.push(EngineUiAction::LoadSceneFromPath(item.path.clone()));
                    ui.close();
                }
            }
            AssetCategory::Shaders if item.is_loaded_in_memory => {
                ui.label(
                    egui::RichText::new("⚡ Registered GPU Shader Module")
                        .color(egui::Color32::from_rgb(255, 190, 60))
                        .size(11.0),
                );
            }
            _ => {}
        }

        ui.separator();

        // 2. Utility Actions
        if ui.button("📋 Copy File Path").clicked() {
            let path_str = item.path.to_string_lossy().to_string();
            ui.ctx().copy_text(path_str);
            ui.close();
        }

        if ui.button("📁 Reveal in File Explorer").clicked() {
            let dir_to_open = if item.path.is_file() {
                item.path.parent().unwrap_or(&item.path)
            } else {
                &item.path
            };
            let _ = open_in_file_explorer(dir_to_open);
            ui.close();
        }
    });
}

/// Opens the specified path in the operating system's native file explorer.
fn open_in_file_explorer(path: &std::path::Path) -> std::io::Result<()> {
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