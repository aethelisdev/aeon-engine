// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Navigation Toolbar & Filter Chips.
//!
//! Provides interactive breadcrumbs, search input, category chips,
//! and view-mode toggles.
//!

use super::types::{AssetBrowserState, AssetCategory, AssetViewMode};
use crate::ui::types::EngineUiAction;
use egui::{Color32, RichText, Stroke, Ui};

/// Draws the complete top toolbar and category filter chips.
pub fn draw_asset_toolbar(
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
                RichText::new(segment).color(Color32::from_rgb(0, 229, 255))
            };

            if ui.small_button(btn_text).clicked() {
                let target_path: std::path::PathBuf = segments[..=i].iter().collect();
                new_folder = Some(target_path);
            }
        }

        if let Some(f) = new_folder {
            state.current_folder = f;
        }

        ui.add_space(12.0);

        // 2. Search Box
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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

            ui.add_space(8.0);

            // Clear search button if active
            if !state.search_query.is_empty()
                && ui
                    .small_button("❌")
                    .on_hover_text("Clear search")
                    .clicked()
            {
                state.search_query.clear();
            }

            let search_edit = ui.add(
                egui::TextEdit::singleline(&mut state.search_query)
                    .hint_text("🔍 Search assets...")
                    .desired_width(180.0),
            );
            if search_edit.changed() {
                // Instantly filters through cached items
            }
        });
    });

    ui.add_space(4.0);

    // 3. Category Filter Chips Row & Action Buttons
    ui.horizontal_wrapped(|ui| {
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

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button("🗑 Clean VRAM")
                .on_hover_text("Sweep unreferenced GPU models and textures")
                .clicked()
            {
                ui_actions.push(EngineUiAction::GarbageCollect);
            }

            if ui
                .button("➕ Import Model...")
                .on_hover_text("Import 3D model (glTF, GLB, FBX, OBJ)")
                .clicked()
            {
                ui_actions.push(EngineUiAction::OpenModelDialog);
            }
        });
    });

    ui.separator();
}