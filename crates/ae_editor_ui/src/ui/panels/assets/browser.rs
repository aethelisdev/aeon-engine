// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Panel Orchestrator.
//!
//! Ties together folder tree sidebar, scanning, toolbar navigation, category filtering,
//! card grid / table views, drag-and-drop tooltips, modal dialogs, and memory status footers.
//!

use super::drag_drop::draw_drag_cursor_tooltip;
use super::folder_tree::draw_folder_tree_sidebar;
use super::grid_view::draw_asset_grid_view;
use super::list_view::draw_asset_list_view;
use super::scanner::rescan_assets_if_needed;
use super::toolbar::{draw_asset_bottom_bar, draw_asset_category_chips, draw_asset_top_bar};
use super::types::{AssetBrowserState, AssetCategory, AssetViewMode, PreviewModalState};
use crate::ui::types::EngineUiAction;
use ae_renderer::asset::{AssetStorage, ShaderAsset};
use ae_renderer::render::{ModelAsset, TextureAsset};
use egui::{Color32, RichText, Ui, Vec2};
use std::path::Path;

impl crate::ui::EngineUi {
    /// Renders the complete Content / Asset Browser panel.
    pub fn draw_assets_content(
        ui: &mut Ui,
        state: &mut AssetBrowserState,
        models: &AssetStorage<ModelAsset>,
        textures: &AssetStorage<TextureAsset>,
        shaders: &AssetStorage<ShaderAsset>,
        is_editing: bool,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        // 1. Rescan disk and correlate with active GPU storages
        rescan_assets_if_needed(state, models, textures, shaders);

        // 2. Space key shortcut for Quick Asset Inspector (Only in Edit mode when panel is hovered)
        if is_editing
            && ui.ui_contains_pointer()
            && ui.input(|i| i.key_pressed(egui::Key::Space))
            && state.preview_modal.is_none()
            && state.rename_state.is_none()
            && state.delete_confirmation.is_none()
            && state.new_folder_parent.is_none()
            && let Some(selected) = &state.selected_asset
            && let Some(item) = state.cached_items.iter().find(|i| &i.path == selected)
        {
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

        // 3. Render Top Full-Width Toolbar (Breadcrumbs on Left, Actions, ViewMode & Search on Right)
        draw_asset_top_bar(ui, state, ui_actions);
        ui.separator();

        // 4. Filter Items by Folder, Category & Search Query
        let query_lower = state.search_query.trim().to_ascii_lowercase();
        let is_root_folder = state.current_folder == Path::new("assets");

        let filtered_items: Vec<_> = state
            .cached_items
            .iter()
            .filter(|item| {
                // Folder hierarchy filter
                if !is_root_folder && !item.path.starts_with(&state.current_folder) {
                    return false;
                }
                // Category filter
                if state.active_category != AssetCategory::All
                    && item.category != state.active_category
                {
                    return false;
                }
                // Search query filter
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

        // 5. Compute Pinned Footer & Middle Height
        let footer_height = 28.0;
        let available_h = ui.available_height();
        let middle_height = (available_h - footer_height - 6.0).max(60.0);

        // 6. Main Split Body: Folder Tree Sidebar (Left) + Content Area with Category Chips (Right)
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), middle_height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.horizontal(|ui| {
                    // Left Column: Folder Tree Sidebar (Full Middle Height)
                    if !state.sidebar_collapsed {
                        ui.allocate_ui_with_layout(
                            Vec2::new(state.sidebar_width, middle_height),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                draw_folder_tree_sidebar(ui, state);
                            },
                        );
                        ui.separator();
                    }

                    // Right Column: Category Chips Row + Scrollable Asset View
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), middle_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            // Category Filter Chips
                            draw_asset_category_chips(ui, state);
                            ui.add_space(2.0);
                            ui.separator();

                            // Scrollable Asset Grid / List View filling remaining height
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
                                                    "No Assets Found in Active Directory"
                                                } else {
                                                    "No Assets Matching Search Query"
                                                })
                                                .strong()
                                                .size(13.0)
                                                .color(Color32::WHITE),
                                            );
                                            ui.label(
                                                RichText::new("Place 3D models (.gltf, .glb, .fbx), textures (.png), shaders (.wgsl), or scenes (.aee) into this folder.")
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
                                        ui.add_space(8.0);
                                    });
                            }
                        },
                    );
                });
            },
        );

        // 7. Pinned Bottom Status Footer
        ui.separator();
        draw_asset_bottom_bar(ui, state, filtered_items.len());

        // 8. Render Floating Drag Tooltip
        let ctx = ui.ctx();
        draw_drag_cursor_tooltip(ctx, state);
    }
}