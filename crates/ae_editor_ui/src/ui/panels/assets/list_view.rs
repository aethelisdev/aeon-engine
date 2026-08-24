// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Detailed List View.
//!
//! Renders structured multi-column table rows with metadata and instant actions.
//!

use super::context_menu::attach_asset_context_menu;
use super::drag_drop::handle_asset_drag_source;
use super::types::{AssetBrowserState, AssetCategory, AssetItem, PreviewModalState};
use crate::ui::types::EngineUiAction;
use egui::{Color32, RichText, Ui};

/// Draws the asset browser items as a structured detailed list table.
pub fn draw_asset_list_view(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    items: &[AssetItem],
    ui_actions: &mut Vec<EngineUiAction>,
) {
    egui::Grid::new("asset_list_table")
        .num_columns(5)
        .spacing([16.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            // Table Header
            ui.label(
                RichText::new("Name")
                    .strong()
                    .color(Color32::from_gray(170)),
            );
            ui.label(
                RichText::new("Category")
                    .strong()
                    .color(Color32::from_gray(170)),
            );
            ui.label(
                RichText::new("Size / Metric")
                    .strong()
                    .color(Color32::from_gray(170)),
            );
            ui.label(
                RichText::new("Status")
                    .strong()
                    .color(Color32::from_gray(170)),
            );
            ui.label(
                RichText::new("Actions")
                    .strong()
                    .color(Color32::from_gray(170)),
            );
            ui.end_row();

            for item in items {
                let is_selected = state.selected_asset.as_ref() == Some(&item.path);

                // 1. Icon & Name
                let icon = match item.category {
                    AssetCategory::Models3D => "📦",
                    AssetCategory::Textures2D => "🖼",
                    AssetCategory::Shaders => "⚡",
                    AssetCategory::Scenes => "🎬",
                    AssetCategory::Materials => "🎨",
                    AssetCategory::Audio => "🔊",
                    AssetCategory::All => "📄",
                };

                let name_label = format!("{} {}", icon, item.name);
                let text_color = if is_selected {
                    Color32::from_rgb(0, 229, 255)
                } else {
                    Color32::WHITE
                };

                let response = ui.selectable_label(
                    is_selected,
                    RichText::new(name_label).color(text_color).strong(),
                );

                if response.clicked() {
                    state.selected_asset = Some(item.path.clone());
                }

                // Drag Source
                handle_asset_drag_source(&response, item, state);

                if response.double_clicked() {
                    match item.category {
                        AssetCategory::Models3D => {
                            if let Some(handle) = item.model_handle {
                                ui_actions.push(EngineUiAction::SpawnModel(handle));
                            }
                        }
                        AssetCategory::Textures2D => {
                            if let Some(handle) = item.texture_handle {
                                ui_actions.push(EngineUiAction::SpawnSprite(handle));
                            }
                        }
                        AssetCategory::Scenes => {
                            ui_actions.push(EngineUiAction::LoadSceneFromPath(item.path.clone()));
                        }
                        _ => {
                            state.preview_modal = Some(PreviewModalState {
                                item: item.clone(),
                                orbit_yaw: 0.0,
                                orbit_pitch: 0.3,
                                zoom_distance: 1.0,
                                show_wireframe: true,
                                channel_mask: [true, true, true, true],
                                wgsl_source: None,
                            });
                        }
                    }
                }

                attach_asset_context_menu(&response, item, state, ui_actions);

                // 2. Category
                ui.label(
                    RichText::new(item.category.badge())
                        .color(item.category.badge_color())
                        .size(10.0)
                        .strong(),
                );

                // 3. Size / Metric
                ui.label(
                    RichText::new(&item.metadata_badge)
                        .color(Color32::from_gray(160))
                        .size(11.0),
                );

                // 4. Memory Residency Status
                if item.is_loaded_in_memory {
                    ui.label(
                        RichText::new("● In VRAM")
                            .color(Color32::from_rgb(0, 229, 255))
                            .size(11.0),
                    );
                } else {
                    ui.label(
                        RichText::new("○ On Disk")
                            .color(Color32::from_gray(120))
                            .size(11.0),
                    );
                }

                // 5. Actions Button
                ui.horizontal(|ui| match item.category {
                    AssetCategory::Models3D => {
                        if let Some(handle) = item.model_handle
                            && ui.small_button("➕ Spawn").clicked()
                        {
                            ui_actions.push(EngineUiAction::SpawnModel(handle));
                        }
                    }
                    AssetCategory::Textures2D => {
                        if let Some(handle) = item.texture_handle
                            && ui.small_button("➕ Spawn").clicked()
                        {
                            ui_actions.push(EngineUiAction::SpawnSprite(handle));
                        }
                    }
                    AssetCategory::Scenes if ui.small_button("🎬 Load").clicked() => {
                        ui_actions.push(EngineUiAction::LoadSceneFromPath(item.path.clone()));
                    }
                    _ => {
                        if ui.small_button("🔍 Inspect").clicked() {
                            state.preview_modal = Some(PreviewModalState {
                                item: item.clone(),
                                orbit_yaw: 0.0,
                                orbit_pitch: 0.3,
                                zoom_distance: 1.0,
                                show_wireframe: true,
                                channel_mask: [true, true, true, true],
                                wgsl_source: None,
                            });
                        }
                    }
                });

                ui.end_row();
            }
        });
}