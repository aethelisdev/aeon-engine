// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Inspector UI Handler for Progress and Health Bars (`UiProgressBar`).
//!

use crate::ui::panels::inspector::registry::{ComponentUiHandler, InspectorContext};
use crate::ui::types::EngineUiAction;
use ae_core::ecs::UiProgressBar;

/// UI Handler for `UiProgressBar` gauges and meters.
pub struct UiProgressBarUiHandler;

impl ComponentUiHandler for UiProgressBarUiHandler {
    fn component_name(&self) -> &'static str {
        "UiProgressBar"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Progress / Health Bar",
            "📊",
            egui::Color32::from_rgb(40, 200, 100),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "Progress / Health Bar")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&UiProgressBar>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut InspectorContext) {
        if let Ok(bar) = ctx.world.get::<&UiProgressBar>(ctx.entity) {
            let mut new_bar = *bar;
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Value:");
                if ui
                    .add(
                        egui::DragValue::new(&mut new_bar.value)
                            .speed(1.0)
                            .range(new_bar.min..=new_bar.max),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Min:");
                if ui
                    .add(egui::DragValue::new(&mut new_bar.min).speed(1.0))
                    .changed()
                {
                    changed = true;
                }
                ui.label("Max:");
                if ui
                    .add(egui::DragValue::new(&mut new_bar.max).speed(1.0))
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Fill Color:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_bar.fill_color)
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Background:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_bar.background_color)
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Corner Radius:");
                if ui
                    .add(
                        egui::DragValue::new(&mut new_bar.corner_radius)
                            .speed(0.5)
                            .range(0.0..=30.0),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            // Preview bar
            let fraction = new_bar.fraction();
            ui.add(egui::ProgressBar::new(fraction).text(format!(
                "{:.1} / {:.1} ({:.0}%)",
                new_bar.value,
                new_bar.max,
                fraction * 100.0
            )));

            if changed && new_bar != *bar {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity,
                    "UiProgressBar",
                    &new_bar,
                ));
            }
        }
    }
}