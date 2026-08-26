// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Inspector UI Handler for Image and Sprite Display (`UiImage`).
//!

use crate::ui::panels::inspector::registry::{ComponentUiHandler, InspectorContext};
use crate::ui::types::EngineUiAction;
use ae_core::ecs::{UiImage, UiSliceMode};

/// UI Handler for `UiImage` sprite and 9-slice textures.
pub struct UiImageUiHandler;

impl ComponentUiHandler for UiImageUiHandler {
    fn component_name(&self) -> &'static str {
        "UiImage"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        ("Image / Icon", "🖼️", egui::Color32::from_rgb(220, 100, 160))
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "Image / Icon")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&UiImage>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut InspectorContext) {
        if let Ok(img) = ctx.world.get::<&UiImage>(ctx.entity) {
            let mut new_img = *img;
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Sprite ID:");
                let mut id_str = new_img
                    .sprite_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "None".to_string());
                if ui.text_edit_singleline(&mut id_str).changed() {
                    new_img.sprite_id = id_str.parse::<u64>().ok();
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Tint Color:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_img.tint)
                    .changed()
                {
                    changed = true;
                }
            });

            let slice_labels = ["Stretch", "Fit", "Nine-Slice", "Tile"];
            let current_slice = match new_img.slice_mode {
                UiSliceMode::Stretch => 0,
                UiSliceMode::Fit => 1,
                UiSliceMode::NineSlice => 2,
                UiSliceMode::Tile => 3,
            };
            let mut selected_slice = current_slice;

            ui.horizontal(|ui| {
                ui.label("Slice Mode:");
                egui::ComboBox::from_id_salt("ui_image_slice_mode")
                    .selected_text(slice_labels[selected_slice])
                    .show_ui(ui, |ui| {
                        for (i, label) in slice_labels.iter().enumerate() {
                            if ui
                                .selectable_value(&mut selected_slice, i, *label)
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
            });

            if selected_slice != current_slice {
                new_img.slice_mode = match selected_slice {
                    0 => UiSliceMode::Stretch,
                    1 => UiSliceMode::Fit,
                    2 => UiSliceMode::NineSlice,
                    _ => UiSliceMode::Tile,
                };
                changed = true;
            }

            if changed && new_img != *img {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity, "UiImage", &new_img,
                ));
            }
        }
    }
}