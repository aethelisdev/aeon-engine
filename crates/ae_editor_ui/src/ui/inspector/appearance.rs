// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::{EngineUi, EngineUiAction};

impl EngineUi {
    /// Renders the Object Appearance (Color, Hex, Palette Swatches) inspector panel section.
    pub(super) fn draw_appearance_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        inspector_color_hex: &mut String,
        saved_swatches: &mut Vec<[f32; 4]>,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 8.0);
            ui.label(
                egui::RichText::new("Appearance")
                    .strong()
                    .color(egui::Color32::WHITE),
            );
            let mut color = if let Ok(c) = world.get::<&ae_core::ecs::Color>(entity) {
                [c.r, c.g, c.b, c.a]
            } else {
                [0.3, 0.3, 0.3, 1.0] // Default Dark Gray
            };

            ui.horizontal(|ui| {
                ui.label("Object Color:");
                let res = ui.color_edit_button_rgba_unmultiplied(&mut color);

                // Hex Input Field
                ui.add_space(5.0);
                ui.label("Hex:");
                let hex_res =
                    ui.add(egui::TextEdit::singleline(inspector_color_hex).desired_width(65.0));

                if res.changed() {
                    let old_color = if let Ok(c) = world.get::<&ae_core::ecs::Color>(entity) {
                        Some(*c)
                    } else {
                        Some(ae_core::ecs::Color {
                            r: 0.3,
                            g: 0.3,
                            b: 0.3,
                            a: 1.0,
                        })
                    };
                    let new_color = ae_core::ecs::Color {
                        r: color[0],
                        g: color[1],
                        b: color[2],
                        a: color[3],
                    };
                    ui_actions.push(EngineUiAction::ModifyColor(
                        entity,
                        old_color.unwrap_or(ae_core::ecs::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        new_color,
                    ));

                    *inspector_color_hex = format!(
                        "#{:02x}{:02x}{:02x}",
                        (color[0] * 255.0) as u8,
                        (color[1] * 255.0) as u8,
                        (color[2] * 255.0) as u8
                    );
                } else if hex_res.changed() {
                    let clean_hex = inspector_color_hex.trim_start_matches('#');
                    if clean_hex.len() == 6 {
                        if let Ok(rgb) = u32::from_str_radix(clean_hex, 16) {
                            let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
                            let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
                            let b = (rgb & 0xFF) as f32 / 255.0;

                            let old_color = if let Ok(c) = world.get::<&ae_core::ecs::Color>(entity)
                            {
                                *c
                            } else {
                                ae_core::ecs::Color {
                                    r: 0.3,
                                    g: 0.3,
                                    b: 0.3,
                                    a: 1.0,
                                }
                            };
                            let new_color = ae_core::ecs::Color { r, g, b, a: 1.0 };

                            ui_actions
                                .push(EngineUiAction::ModifyColor(entity, old_color, new_color));
                        }
                    }
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Add to Palette:");
                if ui
                    .button("✚")
                    .on_hover_text("Save selected color to palette")
                    .clicked()
                {
                    if !saved_swatches.contains(&color) && saved_swatches.len() < 22 {
                        saved_swatches.push(color);
                    }
                }
                if ui.button("🗑").on_hover_text("Clear palette").clicked() {
                    saved_swatches.clear();
                }
            });

            // --- SWATCH GRID ---
            if !saved_swatches.is_empty() {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                    for &swatch in saved_swatches.iter() {
                        let swatch_size = egui::vec2(14.0, 14.0);
                        let (rect, res) = ui.allocate_at_least(swatch_size, egui::Sense::click());

                        let color32 = egui::Color32::from_rgba_unmultiplied(
                            (swatch[0] * 255.0) as u8,
                            (swatch[1] * 255.0) as u8,
                            (swatch[2] * 255.0) as u8,
                            (swatch[3] * 255.0) as u8,
                        );

                        if res.hovered() {
                            let glow_rect = rect.expand(1.5);
                            ui.painter().rect_filled(glow_rect, 2.0, color32);
                            ui.painter().rect_stroke(
                                glow_rect,
                                2.0,
                                egui::Stroke::new(1.5, egui::Color32::WHITE),
                                egui::StrokeKind::Outside,
                            );
                        } else {
                            ui.painter().rect_filled(rect, 2.0, color32);
                        }

                        if res.clicked() {
                            let old_color = if let Ok(c) = world.get::<&ae_core::ecs::Color>(entity)
                            {
                                *c
                            } else {
                                ae_core::ecs::Color {
                                    r: 0.3,
                                    g: 0.3,
                                    b: 0.3,
                                    a: 1.0,
                                }
                            };
                            let new_color = ae_core::ecs::Color {
                                r: swatch[0],
                                g: swatch[1],
                                b: swatch[2],
                                a: swatch[3],
                            };

                            ui_actions
                                .push(EngineUiAction::ModifyColor(entity, old_color, new_color));
                        }
                    }
                });
            }
        });
    }

    /// Renders the Texture & Material inspector panel section.
    /// Shows active sprite/texture reference, handle metadata, and provides interactive buttons
    /// for picking a texture file from disk or removing/assigning a texture.
    pub fn draw_texture_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 8.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("🖼️ Texture & Material")
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if world.get::<&ae_core::ecs::SpriteId>(entity).is_ok() {
                        if ui
                            .button("🗑")
                            .on_hover_text("Remove Texture from Entity")
                            .clicked()
                        {
                            ui_actions.push(EngineUiAction::RemoveTextureFromEntity(entity));
                        }
                    }
                });
            });
            ui.separator();

            if let Ok(sprite_ref) = world.get::<&ae_core::ecs::SpriteId>(entity) {
                if let Some(asset) = textures.get(sprite_ref.0) {
                    let file_name = std::path::Path::new(&asset.source_path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| asset.source_path.clone());

                    ui.horizontal(|ui| {
                        ui.label("Path:");
                        ui.label(
                            egui::RichText::new(&file_name)
                                .color(egui::Color32::LIGHT_BLUE)
                                .strong(),
                        )
                        .on_hover_text(&asset.source_path);
                    });

                    let max_dim = asset.width.max(asset.height);
                    let mip_levels = if max_dim > 0 { max_dim.ilog2() + 1 } else { 1 };
                    ui.horizontal(|ui| {
                        ui.label("Info:");
                        ui.label(
                            egui::RichText::new(format!(
                                "{} x {} px | sRGB | Mipmaps: {}",
                                asset.width, asset.height, mip_levels
                            ))
                            .color(egui::Color32::GREEN)
                            .strong(),
                        );
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Status:");
                        ui.label(
                            egui::RichText::new("Texture Attached")
                                .color(egui::Color32::GREEN)
                                .strong(),
                        );
                    });
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui
                        .button("📁 Change Texture")
                        .on_hover_text("Browse disk for .png, .jpg, .tga file to change texture")
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Texture Image", &["png", "jpg", "jpeg", "tga", "bmp"])
                            .pick_file()
                        {
                            ui_actions.push(EngineUiAction::AssignTextureToEntity(
                                entity,
                                path.to_string_lossy().to_string(),
                            ));
                        }
                    }
                });

                // --- TILING & SAMPLER CONTROLS ---
                ui.separator();
                ui.label(
                    egui::RichText::new("🧱 Tiling & Sampler Settings")
                        .strong()
                        .color(egui::Color32::WHITE),
                );

                ui.horizontal(|ui| {
                    ui.label("Wrap U:");
                    ui.label(
                        egui::RichText::new("Repeat")
                            .color(egui::Color32::LIGHT_GREEN)
                            .strong(),
                    )
                    .on_hover_text("Horizontal texture coordinate repeating");
                });

                ui.horizontal(|ui| {
                    ui.label("Wrap V:");
                    ui.label(
                        egui::RichText::new("Repeat")
                            .color(egui::Color32::LIGHT_GREEN)
                            .strong(),
                    )
                    .on_hover_text("Vertical texture coordinate repeating");
                });

                ui.horizontal(|ui| {
                    ui.label("Anisotropy:");
                    ui.label(
                        egui::RichText::new("16x")
                            .color(egui::Color32::GOLD)
                            .strong(),
                    )
                    .on_hover_text("16x Anisotropic filtering for oblique surface clarity");
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("No Texture Assigned");
                    if ui
                        .button("➕ Add Texture")
                        .on_hover_text("Browse disk for .png, .jpg file to assign texture")
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Texture Image", &["png", "jpg", "jpeg", "tga", "bmp"])
                            .pick_file()
                        {
                            ui_actions.push(EngineUiAction::AssignTextureToEntity(
                                entity,
                                path.to_string_lossy().to_string(),
                            ));
                        }
                    }
                });
            }
        });
    }
}