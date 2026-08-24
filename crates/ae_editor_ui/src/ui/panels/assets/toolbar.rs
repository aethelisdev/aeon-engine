// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Navigation Toolbar, Category Filter Chips & Bottom Status Bar.
//!
//! Provides interactive breadcrumbs, search input, category chips, view-mode toggles,
//! action buttons, and pinned telemetry footer.
//!

use super::types::{AssetBrowserState, AssetCategory, AssetViewMode};
use crate::ui::types::EngineUiAction;
use egui::{Color32, RichText, Stroke, Ui};
use std::path::PathBuf;

/// Draws the top full-width toolbar (Breadcrumbs on left, Actions, View Mode & Search on right).
pub fn draw_asset_top_bar(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    ui_actions: &mut Vec<EngineUiAction>,
) {
    ui.horizontal(|ui| {
        // 1. Breadcrumb Folder Navigation
        ui.label(RichText::new("📁").size(14.0));
        let segments: Vec<String> = state
            .current_folder
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();

        let mut new_folder = None;

        for (i, segment) in segments.iter().enumerate() {
            if i > 0 {
                ui.label(RichText::new(">").color(Color32::from_gray(100)).size(11.0));
            }
            let is_last = i == segments.len() - 1;
            let btn_text = if is_last {
                RichText::new(segment).strong().color(Color32::WHITE)
            } else {
                RichText::new(segment).color(Color32::from_gray(180))
            };

            if ui.small_button(btn_text).clicked() {
                let target_path: PathBuf = segments[..=i].iter().collect();
                new_folder = Some(target_path);
            }
        }

        if let Some(f) = new_folder {
            state.current_folder = f;
        }

        // 2. Right Aligned: Search Box, View Mode, and Compact Action Buttons
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Search Input
            if !state.search_query.is_empty()
                && ui
                    .small_button("❌")
                    .on_hover_text("Clear search")
                    .clicked()
            {
                state.search_query.clear();
            }

            ui.add(
                egui::TextEdit::singleline(&mut state.search_query)
                    .hint_text("🔍 Search assets...")
                    .desired_width(160.0),
            );

            ui.separator();

            // View Mode Toggle (Grid vs List)
            let is_grid = state.view_mode == AssetViewMode::Grid;
            let grid_btn = ui.selectable_label(is_grid, "⊞ Grid");
            if grid_btn.clicked() {
                state.view_mode = AssetViewMode::Grid;
            }
            let list_btn = ui.selectable_label(!is_grid, "☰ List");
            if list_btn.clicked() {
                state.view_mode = AssetViewMode::List;
            }

            ui.separator();

            // Action Buttons: Clean, Reveal, Import
            if ui
                .button("🗑 Clean")
                .on_hover_text("Sweep unreferenced GPU models and textures (Clean VRAM)")
                .clicked()
            {
                ui_actions.push(EngineUiAction::GarbageCollect);
            }

            if ui
                .button("📁 Reveal")
                .on_hover_text("Open active folder in system file explorer")
                .clicked()
            {
                let _ = super::file_ops::open_in_file_explorer(&state.current_folder);
            }

            if ui
                .button("➕ Import")
                .on_hover_text("Import 3D model (glTF, GLB, FBX, OBJ)")
                .clicked()
            {
                ui_actions.push(EngineUiAction::OpenModelDialog);
            }
        });
    });
}

/// Draws the category filter chips row for the main content area.
pub fn draw_asset_category_chips(ui: &mut Ui, state: &mut AssetBrowserState) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 4.0);

        let categories = [
            AssetCategory::All,
            AssetCategory::Models3D,
            AssetCategory::Textures2D,
            AssetCategory::Shaders,
            AssetCategory::Scenes,
            AssetCategory::Materials,
            AssetCategory::Audio,
        ];

        for cat in categories {
            let count = if cat == AssetCategory::All {
                state.cached_items.len()
            } else {
                state
                    .cached_items
                    .iter()
                    .filter(|i| i.category == cat)
                    .count()
            };

            let is_selected = state.active_category == cat;
            let chip_text = format!("{} ({})", cat.label(), count);

            let stroke = if is_selected {
                Stroke::new(1.0, cat.badge_color())
            } else {
                Stroke::NONE
            };

            let text_color = if is_selected {
                Color32::WHITE
            } else {
                Color32::from_gray(160)
            };

            let resp = ui.add(
                egui::Button::new(RichText::new(chip_text).color(text_color).size(11.0))
                    .fill(if is_selected {
                        Color32::from_rgb(30, 34, 45)
                    } else {
                        Color32::from_rgb(20, 22, 28)
                    })
                    .stroke(stroke)
                    .corner_radius(egui::CornerRadius::same(4)),
            );

            if resp.clicked() {
                state.active_category = cat;
            }
        }
    });
}

/// Draws the bottom pinned status bar / footer.
pub fn draw_asset_bottom_bar(ui: &mut Ui, state: &mut AssetBrowserState, filtered_count: usize) {
    ui.horizontal(|ui| {
        // 1. Sidebar collapse toggle
        let toggle_icon = if state.sidebar_collapsed {
            "📂 ▶"
        } else {
            "📂 ◀"
        };
        if ui
            .small_button(toggle_icon)
            .on_hover_text("Toggle Folder Tree Sidebar")
            .clicked()
        {
            state.sidebar_collapsed = !state.sidebar_collapsed;
        }

        ui.separator();

        ui.label(
            RichText::new(format!("📁 {}", state.current_folder.display()))
                .color(Color32::from_gray(140))
                .size(10.5),
        );

        // 2. Right Aligned Telemetry
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let total_size: u64 = state.cached_items.iter().map(|i| i.file_size_bytes).sum();
            let in_memory_count = state
                .cached_items
                .iter()
                .filter(|i| i.is_loaded_in_memory)
                .count();

            ui.label(
                RichText::new(format!(
                    "{} Items in Scope ({} Total, {} in VRAM)  •  Disk: {}",
                    filtered_count,
                    state.cached_items.len(),
                    in_memory_count,
                    AssetBrowserState::format_file_size(total_size)
                ))
                .color(Color32::from_gray(130))
                .size(10.5),
            );
        });
    });
}