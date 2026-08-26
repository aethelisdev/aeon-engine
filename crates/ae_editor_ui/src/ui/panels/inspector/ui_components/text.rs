// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Inspector UI Handlers for Typography and Text Inputs (`UiText`, `UiTextInput`).
//!

use crate::ui::panels::inspector::registry::{ComponentUiHandler, InspectorContext};
use crate::ui::types::EngineUiAction;
use ae_core::ecs::{UiText, UiTextAlignment, UiTextInput};

/// UI Handler for `UiText` typography and alignment.
pub struct UiTextUiHandler;

impl ComponentUiHandler for UiTextUiHandler {
    fn component_name(&self) -> &'static str {
        "UiText"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        ("Text Label", "🔤", egui::Color32::from_rgb(220, 180, 40))
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "Text Label")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&UiText>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut InspectorContext) {
        if let Ok(text_comp) = ctx.world.get::<&UiText>(ctx.entity) {
            let mut new_text = (*text_comp).clone();
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Text:");
                if ui.text_edit_singleline(&mut new_text.text).changed() {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Font Size:");
                if ui
                    .add(
                        egui::DragValue::new(&mut new_text.font_size)
                            .speed(0.5)
                            .range(8.0..=128.0),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Color:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_text.color)
                    .changed()
                {
                    changed = true;
                }
            });

            let align_labels = ["Left", "Center", "Right"];
            let current_align = match new_text.alignment {
                UiTextAlignment::Left => 0,
                UiTextAlignment::Center => 1,
                UiTextAlignment::Right => 2,
            };
            let mut selected_align = current_align;

            ui.horizontal(|ui| {
                ui.label("Alignment:");
                egui::ComboBox::from_id_salt("ui_text_align")
                    .selected_text(align_labels[selected_align])
                    .show_ui(ui, |ui| {
                        for (i, label) in align_labels.iter().enumerate() {
                            if ui
                                .selectable_value(&mut selected_align, i, *label)
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
            });

            if selected_align != current_align {
                new_text.alignment = match selected_align {
                    0 => UiTextAlignment::Left,
                    1 => UiTextAlignment::Center,
                    _ => UiTextAlignment::Right,
                };
                changed = true;
            }

            if changed && new_text != *text_comp {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity, "UiText", &new_text,
                ));
            }
        }
    }
}

/// UI Handler for `UiTextInput` text input boxes.
pub struct UiTextInputUiHandler;

impl ComponentUiHandler for UiTextInputUiHandler {
    fn component_name(&self) -> &'static str {
        "UiTextInput"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Text Input Field",
            "📝",
            egui::Color32::from_rgb(200, 130, 60),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "Text Input Field")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&UiTextInput>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut InspectorContext) {
        if let Ok(input) = ctx.world.get::<&UiTextInput>(ctx.entity) {
            let mut new_input = (*input).clone();
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Content:");
                if ui.text_edit_singleline(&mut new_input.text).changed() {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Placeholder:");
                if ui
                    .text_edit_singleline(&mut new_input.placeholder)
                    .changed()
                {
                    changed = true;
                }
            });

            if changed && new_input != *input {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity,
                    "UiTextInput",
                    &new_input,
                ));
            }
        }
    }
}