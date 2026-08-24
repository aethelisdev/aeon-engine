// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Safe File & Directory Operations Module.
//!
//! Provides directory creation, asset renaming, secure deletion, and path migration
//! with modal confirmation dialogues and error isolation.
//!

use super::types::AssetBrowserState;
use egui::{Color32, CornerRadius, RichText, Stroke, Vec2};
use std::path::{Path, PathBuf};

/// Creates a new subfolder under the specified parent directory.
pub fn create_subfolder(parent: &Path, name: &str) -> std::io::Result<PathBuf> {
    let clean_name = name.trim();
    if clean_name.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Folder name cannot be empty",
        ));
    }

    let target_dir = parent.join(clean_name);
    std::fs::create_dir_all(&target_dir)?;
    log::info!("📁 Created folder: {}", target_dir.display());
    Ok(target_dir)
}

/// Renames a file or directory on disk to a new name.
/// Preserves the original file extension if renaming a file and the extension is omitted.
pub fn rename_asset_or_folder(target: &Path, new_name: &str) -> std::io::Result<PathBuf> {
    let clean_name = new_name.trim();
    if clean_name.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Target name cannot be empty",
        ));
    }

    let parent = target.parent().unwrap_or_else(|| Path::new("assets"));
    let final_name = if target.is_file() && !clean_name.contains('.') {
        if let Some(ext) = target.extension().and_then(|e| e.to_str()) {
            format!("{}.{}", clean_name, ext)
        } else {
            clean_name.to_string()
        }
    } else {
        clean_name.to_string()
    };

    let new_path = parent.join(final_name);
    std::fs::rename(target, &new_path)?;
    log::info!(
        "🔄 Renamed '{}' -> '{}'",
        target.display(),
        new_path.display()
    );
    Ok(new_path)
}

/// Deletes a file or directory from the file system.
/// Directories are removed recursively.
pub fn delete_asset_or_folder(target: &Path) -> std::io::Result<()> {
    if target.is_dir() {
        std::fs::remove_dir_all(target)?;
        log::info!("🗑️ Removed directory: {}", target.display());
    } else if target.is_file() {
        std::fs::remove_file(target)?;
        log::info!("🗑️ Removed file: {}", target.display());
    }
    Ok(())
}

/// Moves a file or directory into a destination directory.
pub fn move_asset(source: &Path, destination_dir: &Path) -> std::io::Result<PathBuf> {
    let file_name = source.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid source path")
    })?;
    let target_path = destination_dir.join(file_name);
    std::fs::rename(source, &target_path)?;
    log::info!(
        "📦 Moved '{}' -> '{}'",
        source.display(),
        target_path.display()
    );
    Ok(target_path)
}

/// Opens the specified file or folder in the operating system's native file explorer.
pub fn open_in_file_explorer(path: &Path) -> std::io::Result<()> {
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

/// Helper creating standard engine modal window frame matching the Preferences window style.
fn modal_window_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgb(20, 20, 25))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 48, 60)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(egui::Margin::ZERO)
        .shadow(egui::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_rgba_premultiplied(0, 0, 0, 180),
        })
}

/// Renders the custom sleek header bar matching the Preferences window.
fn draw_modal_header(ui: &mut egui::Ui, title: &str, on_close: &mut bool) {
    egui::Frame::new()
        .fill(Color32::from_rgb(15, 15, 20))
        .inner_margin(egui::Margin::symmetric(14, 8))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 48, 60)))
        .corner_radius(CornerRadius {
            nw: 8,
            ne: 8,
            sw: 0,
            se: 0,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(title)
                        .strong()
                        .size(13.0)
                        .color(Color32::from_gray(225)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("✖").size(11.0).color(Color32::from_gray(160)),
                            )
                            .fill(Color32::TRANSPARENT)
                            .frame(false),
                        )
                        .on_hover_text("Close")
                        .clicked()
                    {
                        *on_close = true;
                    }
                });
            });
        });
}

/// Renders modal dialogs for New Folder, Rename, and Delete confirmations.
pub fn draw_file_operations_dialogs(
    ctx: &egui::Context,
    state: &mut AssetBrowserState,
) -> Option<egui::Rect> {
    let mut modal_rect = None;

    // 1. New Folder Modal Dialog
    if let Some(parent) = state.new_folder_parent.clone() {
        let mut close_dialog = false;
        let mut create_folder_now = false;

        let win_resp = egui::Window::new("create_new_folder_modal")
            .id(egui::Id::new("create_new_folder_modal"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(380.0, 155.0))
            .frame(modal_window_frame())
            .show(ctx, |ui| {
                draw_modal_header(ui, "📁  Create New Folder", &mut close_dialog);

                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("Location: {}", parent.display()))
                                .size(11.5)
                                .color(Color32::from_gray(150)),
                        );
                        ui.add_space(8.0);

                        let edit_resp = ui.add(
                            egui::TextEdit::singleline(&mut state.new_folder_name)
                                .hint_text("Enter folder name...")
                                .desired_width(f32::INFINITY),
                        );
                        if edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            create_folder_now = true;
                        }

                        ui.add_space(14.0);

                        ui.horizontal(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let create_btn = ui.add(
                                        egui::Button::new(
                                            RichText::new("Create Folder")
                                                .strong()
                                                .color(Color32::WHITE)
                                                .size(11.5),
                                        )
                                        .fill(Color32::from_rgb(32, 38, 52))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(55, 65, 90)))
                                        .corner_radius(CornerRadius::same(4))
                                        .min_size(Vec2::new(95.0, 24.0)),
                                    );
                                    if create_btn.clicked() {
                                        create_folder_now = true;
                                    }

                                    ui.add_space(6.0);

                                    let cancel_btn = ui.add(
                                        egui::Button::new(
                                            RichText::new("Cancel")
                                                .color(Color32::from_gray(180))
                                                .size(11.5),
                                        )
                                        .fill(Color32::from_rgb(22, 24, 30))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(40, 44, 55)))
                                        .corner_radius(CornerRadius::same(4))
                                        .min_size(Vec2::new(70.0, 24.0)),
                                    );
                                    if cancel_btn.clicked() {
                                        close_dialog = true;
                                    }
                                },
                            );
                        });
                    });
            });

        if let Some(r) = win_resp {
            modal_rect = Some(r.response.rect);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close_dialog = true;
        }

        if create_folder_now && !state.new_folder_name.trim().is_empty() {
            let _ = create_subfolder(&parent, &state.new_folder_name);
            state.new_folder_name.clear();
            close_dialog = true;
        }

        if close_dialog {
            state.new_folder_parent = None;
            state.new_folder_name.clear();
        }
    }

    // 2. Rename Modal Dialog
    if let Some(mut ren) = state.rename_state.clone() {
        let mut close_dialog = false;
        let mut apply_rename = false;

        let title = if ren.is_folder {
            "🔄  Rename Folder"
        } else {
            "🔄  Rename Asset"
        };

        let win_resp = egui::Window::new("rename_asset_modal")
            .id(egui::Id::new("rename_asset_modal"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(380.0, 155.0))
            .frame(modal_window_frame())
            .show(ctx, |ui| {
                draw_modal_header(ui, title, &mut close_dialog);

                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("Target: {}", ren.target_path.display()))
                                .size(11.5)
                                .color(Color32::from_gray(150)),
                        );
                        ui.add_space(8.0);

                        let edit_resp = ui.add(
                            egui::TextEdit::singleline(&mut ren.current_name)
                                .hint_text("Enter new name...")
                                .desired_width(f32::INFINITY),
                        );
                        if edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            apply_rename = true;
                        }

                        ui.add_space(14.0);

                        ui.horizontal(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let rename_btn = ui.add(
                                        egui::Button::new(
                                            RichText::new("Apply Rename")
                                                .strong()
                                                .color(Color32::WHITE)
                                                .size(11.5),
                                        )
                                        .fill(Color32::from_rgb(32, 38, 52))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(55, 65, 90)))
                                        .corner_radius(CornerRadius::same(4))
                                        .min_size(Vec2::new(95.0, 24.0)),
                                    );
                                    if rename_btn.clicked() {
                                        apply_rename = true;
                                    }

                                    ui.add_space(6.0);

                                    let cancel_btn = ui.add(
                                        egui::Button::new(
                                            RichText::new("Cancel")
                                                .color(Color32::from_gray(180))
                                                .size(11.5),
                                        )
                                        .fill(Color32::from_rgb(22, 24, 30))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(40, 44, 55)))
                                        .corner_radius(CornerRadius::same(4))
                                        .min_size(Vec2::new(70.0, 24.0)),
                                    );
                                    if cancel_btn.clicked() {
                                        close_dialog = true;
                                    }
                                },
                            );
                        });
                    });
            });

        if let Some(r) = win_resp {
            modal_rect = Some(r.response.rect);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close_dialog = true;
        }

        state.rename_state = Some(ren.clone());

        if apply_rename && !ren.current_name.trim().is_empty() {
            let _ = rename_asset_or_folder(&ren.target_path, &ren.current_name);
            close_dialog = true;
        }

        if close_dialog {
            state.rename_state = None;
        }
    }

    // 3. Delete Confirmation Modal Dialog
    if let Some(target) = state.delete_confirmation.clone() {
        let mut close_dialog = false;
        let mut confirm_delete = false;

        let win_resp = egui::Window::new("delete_confirmation_modal")
            .id(egui::Id::new("delete_confirmation_modal"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(390.0, 160.0))
            .frame(modal_window_frame())
            .show(ctx, |ui| {
                draw_modal_header(ui, "⚠️  Confirm Deletion", &mut close_dialog);

                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("Are you sure you want to permanently delete this item?")
                                .strong()
                                .size(12.0)
                                .color(Color32::from_rgb(255, 110, 110)),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(target.display().to_string())
                                .size(11.0)
                                .color(Color32::from_gray(170)),
                        );
                        ui.add_space(14.0);

                        ui.horizontal(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let del_btn = ui.add(
                                        egui::Button::new(
                                            RichText::new("🗑 Delete Permanently")
                                                .strong()
                                                .color(Color32::WHITE)
                                                .size(11.5),
                                        )
                                        .fill(Color32::from_rgb(160, 35, 35))
                                        .stroke(Stroke::NONE)
                                        .corner_radius(CornerRadius::same(4))
                                        .min_size(Vec2::new(135.0, 24.0)),
                                    );
                                    if del_btn.clicked() {
                                        confirm_delete = true;
                                    }

                                    ui.add_space(6.0);

                                    let cancel_btn = ui.add(
                                        egui::Button::new(
                                            RichText::new("Cancel")
                                                .color(Color32::from_gray(180))
                                                .size(11.5),
                                        )
                                        .fill(Color32::from_rgb(22, 24, 30))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(40, 44, 55)))
                                        .corner_radius(CornerRadius::same(4))
                                        .min_size(Vec2::new(70.0, 24.0)),
                                    );
                                    if cancel_btn.clicked() {
                                        close_dialog = true;
                                    }
                                },
                            );
                        });
                    });
            });

        if let Some(r) = win_resp {
            modal_rect = Some(r.response.rect);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close_dialog = true;
        }

        if confirm_delete {
            let _ = delete_asset_or_folder(&target);
            if state.selected_asset.as_ref() == Some(&target) {
                state.selected_asset = None;
            }
            close_dialog = true;
        }

        if close_dialog {
            state.delete_confirmation = None;
        }
    }

    modal_rect
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_operations_temp_lifecycle() {
        let temp_dir = std::env::temp_dir().join("ae_test_content_browser");
        let _ = std::fs::remove_dir_all(&temp_dir);
        let _ = std::fs::create_dir_all(&temp_dir);

        // 1. Create Subfolder
        let subfolder =
            create_subfolder(&temp_dir, "test_models").expect("Failed to create folder");
        assert!(subfolder.exists());
        assert!(subfolder.is_dir());

        // 2. Create Dummy File
        let test_file = subfolder.join("character.glb");
        std::fs::write(&test_file, b"test_payload").expect("Failed to write test file");
        assert!(test_file.exists());

        // 3. Rename File
        let renamed_file =
            rename_asset_or_folder(&test_file, "hero_character").expect("Failed to rename file");
        assert!(renamed_file.exists());
        assert_eq!(renamed_file.file_name().unwrap(), "hero_character.glb");

        // 4. Move File
        let moved_file = move_asset(&renamed_file, &temp_dir).expect("Failed to move file");
        assert!(moved_file.exists());
        assert_eq!(moved_file.parent().unwrap(), temp_dir.as_path());

        // 5. Delete File & Directory
        delete_asset_or_folder(&moved_file).expect("Failed to delete file");
        assert!(!moved_file.exists());

        delete_asset_or_folder(&temp_dir).expect("Failed to delete temp dir");
        assert!(!temp_dir.exists());
    }
}