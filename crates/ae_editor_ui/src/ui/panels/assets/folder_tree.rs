// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Hierarchical Folder Tree Sidebar Subsystem.
//!
//! Renders an interactive collapsible tree structure representing the workspace
//! directory hierarchy, providing directory-scoped item filtering and context actions.
//!

use super::types::{AssetBrowserState, RenamingState};
use egui::{Color32, RichText, Ui};
use std::path::{Path, PathBuf};

/// Draws the left hierarchical folder tree sidebar.
pub fn draw_folder_tree_sidebar(ui: &mut Ui, state: &mut AssetBrowserState) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("📂 FOLDERS")
                    .strong()
                    .size(11.0)
                    .color(Color32::from_gray(170)),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button("➕")
                    .on_hover_text("Create subfolder in current directory")
                    .clicked()
                {
                    state.new_folder_parent = Some(state.current_folder.clone());
                    state.new_folder_name.clear();
                }
            });
        });

        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let root_path = PathBuf::from("assets");
                render_folder_node(ui, &root_path, state);
            });
    });
}

/// Recursively renders a folder node and its child directories.
fn render_folder_node(ui: &mut Ui, path: &Path, state: &mut AssetBrowserState) {
    let folder_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("assets");

    let is_selected = state.current_folder == path;
    let is_root = path == Path::new("assets");

    // Gather child directories
    let mut child_dirs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let child_path = entry.path();
            if child_path.is_dir() {
                child_dirs.push(child_path);
            }
        }
    }
    child_dirs.sort();

    let has_children = !child_dirs.is_empty();

    let text_color = if is_selected {
        Color32::WHITE
    } else {
        Color32::from_gray(200)
    };

    let label_text = format!("📁 {}", folder_name);

    if has_children {
        let id = ui.make_persistent_id(path);
        let default_open = is_root || path.starts_with(&state.current_folder);

        egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            id,
            default_open,
        )
        .show_header(ui, |ui| {
            let response = ui.selectable_label(
                is_selected,
                RichText::new(&label_text).color(text_color).size(11.5),
            );

            if response.clicked() {
                state.current_folder = path.to_path_buf();
            }

            attach_folder_context_menu(&response, path, state);
        })
        .body(|ui| {
            for child in child_dirs {
                render_folder_node(ui, &child, state);
            }
        });
    } else {
        let response = ui.selectable_label(
            is_selected,
            RichText::new(&label_text).color(text_color).size(11.5),
        );

        if response.clicked() {
            state.current_folder = path.to_path_buf();
        }

        attach_folder_context_menu(&response, path, state);
    }
}

/// Attaches right-click context menu options to a folder node in the hierarchy.
fn attach_folder_context_menu(
    response: &egui::Response,
    path: &Path,
    state: &mut AssetBrowserState,
) {
    response.context_menu(|ui| {
        ui.set_width(165.0);
        let folder_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Folder");

        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!("📁 {}", folder_name))
                    .strong()
                    .size(11.0)
                    .color(Color32::from_gray(190)),
            );
        });
        ui.separator();

        if super::context_menu::context_menu_item(ui, "➕", "New Subfolder").clicked() {
            state.new_folder_parent = Some(path.to_path_buf());
            state.new_folder_name.clear();
            ui.close();
        }

        if path != Path::new("assets") {
            if super::context_menu::context_menu_item(ui, "🔄", "Rename Folder").clicked() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                state.rename_state = Some(RenamingState {
                    target_path: path.to_path_buf(),
                    current_name: name,
                    is_folder: true,
                });
                ui.close();
            }

            if super::context_menu::context_menu_item(ui, "🗑", "Delete Folder").clicked() {
                state.delete_confirmation = Some(path.to_path_buf());
                ui.close();
            }
        }

        ui.separator();

        if super::context_menu::context_menu_item(ui, "📁", "Reveal in Explorer").clicked() {
            let _ = super::file_ops::open_in_file_explorer(path);
            ui.close();
        }
    });
}