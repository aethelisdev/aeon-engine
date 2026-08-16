// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::{EngineUi, EngineUiAction};

impl EngineUi {
    /// Renders the internal content of the Asset Browser (3D Models & 2D Textures).
    /// Exposes loaded model and texture assets with instant viewport spawning,
    /// asset metadata indicators, and manual GPU garbage collection to reclaim VRAM.
    pub fn draw_assets_content(
        ui: &mut egui::Ui,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        ui.horizontal(|ui| {
            if ui
                .button("🗑 Garbage Collect")
                .on_hover_text("Clean up unused model and texture resources to reclaim VRAM")
                .clicked()
            {
                ui_actions.push(EngineUiAction::GarbageCollect);
            }
            ui.label(
                egui::RichText::new(format!(
                    "{} Models • {} Textures loaded",
                    models.len(),
                    textures.len()
                ))
                .color(egui::Color32::from_gray(140))
                .size(11.0),
            );
        });
        ui.separator();

        ui.spacing_mut().item_spacing.y = 8.0;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                // Models Column
                ui.vertical(|ui| {
                    ui.heading("Models (3D)");
                    ui.separator();
                    for (handle, model) in models.iter() {
                        let path = std::path::Path::new(&model.source_path);
                        let filename = path
                            .file_stem()
                            .unwrap_or(std::ffi::OsStr::new("Unknown"))
                            .to_string_lossy();

                        ui.horizontal(|ui| {
                            ui.label(format!("📦 {}", filename));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("➕ Spawn").clicked() {
                                        ui_actions.push(EngineUiAction::SpawnModel(handle));
                                    }
                                },
                            );
                        });
                    }
                    if models.is_empty() {
                        ui.label(
                            egui::RichText::new("No models loaded yet.")
                                .italics()
                                .color(egui::Color32::from_gray(120)),
                        );
                    }
                });

                ui.add_space(20.0);

                // Textures Column
                ui.vertical(|ui| {
                    ui.heading("Textures (2D)");
                    ui.separator();
                    for (handle, texture) in textures.iter() {
                        let path = std::path::Path::new(&texture.source_path);
                        let filename = path
                            .file_stem()
                            .unwrap_or(std::ffi::OsStr::new("Unknown"))
                            .to_string_lossy();

                        ui.horizontal(|ui| {
                            ui.label(format!("🖼 {}", filename));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("➕ Spawn").clicked() {
                                        ui_actions.push(EngineUiAction::SpawnSprite(handle));
                                    }
                                },
                            );
                        });
                    }
                    if textures.is_empty() {
                        ui.label(
                            egui::RichText::new("No textures loaded yet.")
                                .italics()
                                .color(egui::Color32::from_gray(120)),
                        );
                    }
                });
            });
        });
    }
}