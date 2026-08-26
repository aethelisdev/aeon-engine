// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Inspector UI Handlers for Interactive Controls (`UiButton`, `UiSlider`, `UiCheckbox`).
//!

use crate::ui::panels::inspector::registry::{ComponentUiHandler, InspectorContext};
use crate::ui::types::EngineUiAction;
use ae_core::ecs::{UiButton, UiCheckbox, UiSlider};

/// UI Handler for `UiButton` interactive buttons.
pub struct UiButtonUiHandler;

impl ComponentUiHandler for UiButtonUiHandler {
    fn component_name(&self) -> &'static str {
        "UiButton"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Interactive Button",
            "🔘",
            egui::Color32::from_rgb(180, 70, 220),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "Interactive Button")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&UiButton>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut InspectorContext) {
        if let Ok(btn) = ctx.world.get::<&UiButton>(ctx.entity) {
            let mut new_btn = (*btn).clone();
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Label:");
                if ui.text_edit_singleline(&mut new_btn.text).changed() {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Normal Color:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_btn.normal_color)
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Hover Color:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_btn.hover_color)
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Pressed Color:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_btn.pressed_color)
                    .changed()
                {
                    changed = true;
                }
            });

            if ui.checkbox(&mut new_btn.is_enabled, "Is Enabled").changed() {
                changed = true;
            }

            if changed && new_btn != *btn {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity, "UiButton", &new_btn,
                ));
            }
        }
    }
}

/// UI Handler for `UiSlider` numeric sliders.
pub struct UiSliderUiHandler;

impl ComponentUiHandler for UiSliderUiHandler {
    fn component_name(&self) -> &'static str {
        "UiSlider"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Numeric Slider",
            "🎚️",
            egui::Color32::from_rgb(50, 180, 200),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "Numeric Slider")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&UiSlider>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut InspectorContext) {
        if let Ok(slider) = ctx.world.get::<&UiSlider>(ctx.entity) {
            let mut new_slider = *slider;
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Value:");
                if ui
                    .add(
                        egui::DragValue::new(&mut new_slider.value)
                            .speed(0.05)
                            .range(new_slider.min..=new_slider.max),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Min:");
                if ui
                    .add(egui::DragValue::new(&mut new_slider.min).speed(0.1))
                    .changed()
                {
                    changed = true;
                }
                ui.label("Max:");
                if ui
                    .add(egui::DragValue::new(&mut new_slider.max).speed(0.1))
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Track Color:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_slider.track_color)
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Thumb Color:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_slider.thumb_color)
                    .changed()
                {
                    changed = true;
                }
            });

            if changed && new_slider != *slider {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity,
                    "UiSlider",
                    &new_slider,
                ));
            }
        }
    }
}

/// UI Handler for `UiCheckbox` boolean toggle boxes.
pub struct UiCheckboxUiHandler;

impl ComponentUiHandler for UiCheckboxUiHandler {
    fn component_name(&self) -> &'static str {
        "UiCheckbox"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Toggle Checkbox",
            "☑️",
            egui::Color32::from_rgb(70, 200, 140),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "Toggle Checkbox")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&UiCheckbox>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut InspectorContext) {
        if let Ok(chk) = ctx.world.get::<&UiCheckbox>(ctx.entity) {
            let mut new_chk = (*chk).clone();
            let mut changed = false;

            if ui.checkbox(&mut new_chk.is_checked, "Is Checked").changed() {
                changed = true;
            }

            ui.horizontal(|ui| {
                ui.label("Label:");
                if ui.text_edit_singleline(&mut new_chk.label).changed() {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Box Color:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_chk.box_color)
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Check Color:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_chk.check_color)
                    .changed()
                {
                    changed = true;
                }
            });

            if changed && new_chk != *chk {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity,
                    "UiCheckbox",
                    &new_chk,
                ));
            }
        }
    }
}