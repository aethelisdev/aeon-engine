// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Panel Orchestrator.
//!
//! Ties together scanning, toolbar navigation, category filtering,
//! card grid / table views, and memory status footers.
//!

use super::grid_view::draw_asset_grid_view;
use super::list_view::draw_asset_list_view;
use super::scanner::rescan_assets_if_needed;
use super::toolbar::draw_asset_toolbar;
use super::types::{AssetBrowserState, AssetCategory, AssetViewMode};
use crate::ui::types::EngineUiAction;
use ae_renderer::asset::{AssetStorage, ShaderAsset};
use ae_renderer::render::{ModelAsset, TextureAsset};
use egui::{Color32, RichText, Ui};

impl crate::ui::EngineUi {
    /// Renders the complete Content / Asset Browser panel.
    pub fn draw_assets_content(
        ui: &mut Ui,
        state: &mut AssetBrowserState,
        models: &AssetStorage<ModelAsset>,
        textures: &AssetStorage<TextureAsset>,
        shaders: &AssetStorage<ShaderAsset>,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        // 1. Rescan disk and correlate with active GPU storages
        rescan_assets_if_needed(state, models, textures, shaders);

        // 2. Render Top Toolbar & Category Chips
        draw_asset_toolbar(ui, state, ui_actions);

        // 3. Filter Items by Category & Search Query
        let query_lower = state.search_query.trim().to_ascii_lowercase();
        let filtered_items: Vec<_> = state
            .cached_items
            .iter()
            .filter(|item| {
                if state.active_category != AssetCategory::All
                    && item.category != state.active_category
                {
                    return false;
                }
                if !query_lower.is_empty()
                    && !item.name.to_ascii_lowercase().contains(&query_lower)
                    && !item
                        .relative_path
                        .to_ascii_lowercase()
                        .contains(&query_lower)
                {
                    return false;
                }
                true
            })
            .cloned()
            .collect();

        // 4. Content Area
        if filtered_items.is_empty() {
            ui.add_space(16.0);
            egui::Frame::NONE
                .fill(Color32::from_rgb(18, 20, 26))
                .corner_radius(egui::CornerRadius::same(6))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(45, 48, 60)))
                .inner_margin(egui::Margin::symmetric(24, 20))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("📁").size(26.0).color(Color32::from_gray(160)));
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(if state.search_query.is_empty() {
                                "No Assets Discovered in 'assets/'"
                            } else {
                                "No Assets Matching Search Query"
                            })
                            .strong()
                            .size(13.0)
                            .color(Color32::WHITE),
                        );
                        ui.label(
                            RichText::new("Place 3D models (.gltf, .glb, .fbx), textures (.png), shaders (.wgsl), or scenes (.aee) into the assets directory.")
                                .size(11.0)
                                .color(Color32::from_gray(150)),
                        );
                        ui.add_space(8.0);
                        if ui.button("➕ Import 3D Model").clicked() {
                            ui_actions.push(EngineUiAction::OpenModelDialog);
                        }
                    });
                });
        } else {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    match state.view_mode {
                        AssetViewMode::Grid => {
                            draw_asset_grid_view(ui, state, &filtered_items, ui_actions);
                        }
                        AssetViewMode::List => {
                            draw_asset_list_view(ui, state, &filtered_items, ui_actions);
                        }
                    }
                    ui.add_space(12.0);
                });
        }

        // 5. Bottom Status Footer
        ui.separator();
        ui.horizontal(|ui| {
            let total_size: u64 = state.cached_items.iter().map(|i| i.file_size_bytes).sum();
            let in_memory_count = state
                .cached_items
                .iter()
                .filter(|i| i.is_loaded_in_memory)
                .count();

            ui.label(
                RichText::new(format!(
                    "{} Items ({} Loaded in VRAM)  •  Total Disk Footprint: {}",
                    state.cached_items.len(),
                    in_memory_count,
                    AssetBrowserState::format_file_size(total_size)
                ))
                .color(Color32::from_gray(130))
                .size(10.5),
            );
        });
    }
}