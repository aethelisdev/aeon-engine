// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Context Menu & Right-Click Actions.
//!
//! Provides right-click operations such as Quick Inspect, Spawn, Rename,
//! Delete, Copy Path, and Reveal in File Explorer.
//!

use super::types::{AssetBrowserState, AssetCategory, AssetItem, PreviewModalState, RenamingState};
use crate::ui::types::EngineUiAction;
use egui::Response;

/// Renders a neatly formatted, column-aligned context menu item with fixed-width icon column.
pub fn context_menu_item(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    let padding_x = 6.0;
    let icon_width = 18.0;
    let height = 22.0;
    let item_width = 165.0;

    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(item_width, height), egui::Sense::click());

    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(4),
            egui::Color32::from_rgb(38, 44, 58),
        );
    }

    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x + padding_x, rect.min.y),
        egui::vec2(icon_width, height),
    );
    let text_pos = egui::pos2(
        rect.min.x + padding_x + icon_width + 6.0,
        rect.min.y + (height - 12.0) * 0.5 - 1.0,
    );

    let (icon_color, text_color) = if response.hovered() {
        (egui::Color32::WHITE, egui::Color32::WHITE)
    } else {
        (egui::Color32::from_gray(180), egui::Color32::from_gray(210))
    };

    ui.painter().text(
        icon_rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(12.0),
        icon_color,
    );

    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(12.0),
        text_color,
    );

    response
}

/// Attaches the asset context menu to an egui Response.
pub fn attach_asset_context_menu(
    response: &Response,
    item: &AssetItem,
    state: &mut AssetBrowserState,
    ui_actions: &mut Vec<EngineUiAction>,
) {
    response.context_menu(|ui| {
        ui.set_width(165.0);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(&item.name)
                    .strong()
                    .size(11.0)
                    .color(item.category.badge_color()),
            );
        });
        ui.separator();

        // 1. Quick Inspect Modal
        if context_menu_item(ui, "🔍", "Quick Inspect (Space)").clicked() {
            state.preview_modal = Some(PreviewModalState {
                item: item.clone(),
                orbit_yaw: 0.0,
                orbit_pitch: 0.3,
                zoom_distance: 1.0,
                show_wireframe: true,
                channel_mask: [true, true, true, true],
                wgsl_source: None,
            });
            ui.close();
        }

        // 2. Primary Spawn / Open Action
        match item.category {
            AssetCategory::Models3D => {
                if let Some(handle) = item.model_handle
                    && context_menu_item(ui, "➕", "Spawn into Scene").clicked()
                {
                    ui_actions.push(EngineUiAction::SpawnModel(handle));
                    ui.close();
                } else if item.model_handle.is_none() {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("Drag into scene to load")
                                .italics()
                                .size(11.0)
                                .color(egui::Color32::from_gray(140)),
                        );
                    });
                }
            }
            AssetCategory::Textures2D => {
                if let Some(handle) = item.texture_handle
                    && context_menu_item(ui, "🖼", "Spawn as Sprite").clicked()
                {
                    ui_actions.push(EngineUiAction::SpawnSprite(handle));
                    ui.close();
                }
            }
            AssetCategory::Scenes => {
                if context_menu_item(ui, "🎬", "Load Scene").clicked() {
                    ui_actions.push(EngineUiAction::LoadSceneFromPath(item.path.clone()));
                    ui.close();
                }
            }
            AssetCategory::Shaders if item.is_loaded_in_memory => {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("⚡ Registered GPU Module")
                            .color(egui::Color32::from_rgb(255, 190, 60))
                            .size(11.0),
                    );
                });
            }
            _ => {}
        }

        ui.separator();

        // 3. File Operations (Rename & Delete)
        if context_menu_item(ui, "🔄", "Rename (F2)").clicked() {
            state.rename_state = Some(RenamingState {
                target_path: item.path.clone(),
                current_name: item.name.clone(),
                is_folder: false,
            });
            ui.close();
        }

        if context_menu_item(ui, "🗑", "Delete Asset").clicked() {
            state.delete_confirmation = Some(item.path.clone());
            ui.close();
        }

        ui.separator();

        // 4. Utility Actions
        if context_menu_item(ui, "📋", "Copy File Path").clicked() {
            let path_str = item.path.to_string_lossy().to_string();
            ui.ctx().copy_text(path_str);
            ui.close();
        }

        if context_menu_item(ui, "📁", "Reveal in Explorer").clicked() {
            let dir_to_open = if item.path.is_file() {
                item.path.parent().unwrap_or(&item.path)
            } else {
                &item.path
            };
            let _ = super::file_ops::open_in_file_explorer(dir_to_open);
            ui.close();
        }
    });
}