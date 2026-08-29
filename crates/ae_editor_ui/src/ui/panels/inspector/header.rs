// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Entity Header
//!
//! Renders the selected entity's Name field, dirty physics indicator, and rename focus handler.

use crate::ui::EngineUiAction;

/// Parameters for rendering the entity name and status header.
pub struct EntityHeaderParams<'a> {
    pub world: &'a hecs::World,
    pub entity: hecs::Entity,
    pub name: &'a ae_core::ecs::Name,
    pub selection_changed: bool,
    pub focus_rename: bool,
    pub ui_actions: &'a mut Vec<EngineUiAction>,
}

/// Renders the entity name editor header bar with dirty transform warning flag.
pub fn draw_entity_header(ui: &mut egui::Ui, ctx: &egui::Context, params: EntityHeaderParams<'_>) {
    let world = params.world;
    let entity = params.entity;
    let name = params.name;
    let selection_changed = params.selection_changed;
    let focus_rename = params.focus_rename;
    let ui_actions = params.ui_actions;

    let mut temp_name = ctx.data_mut(|d| {
        d.get_temp::<String>(egui::Id::new(("name_edit", entity)))
            .unwrap_or_else(|| name.0.clone())
    });

    egui::Frame::NONE
        .fill(egui::Color32::from_rgb(18, 20, 26))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(38, 42, 54)))
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("🏷 Name:")
                        .strong()
                        .size(11.5)
                        .color(egui::Color32::from_gray(180)),
                );
                let old_name = name.0.clone();

                let text_width = ui.available_width()
                    - if world.get::<&ae_core::ecs::TransformDirty>(entity).is_ok() {
                        60.0
                    } else {
                        0.0
                    };
                let resp = ui.add_sized(
                    egui::vec2(text_width.max(60.0), 19.0),
                    egui::TextEdit::singleline(&mut temp_name),
                );
                if focus_rename {
                    resp.request_focus();
                }

                if world.get::<&ae_core::ecs::TransformDirty>(entity).is_ok() {
                    ui.label(
                        egui::RichText::new("[DIRTY]")
                            .color(egui::Color32::RED)
                            .strong(),
                    )
                    .on_hover_text("Awaiting physics update...");
                }
                if resp.changed() && !selection_changed {
                    ctx.data_mut(|d| {
                        d.insert_temp(egui::Id::new(("name_edit", entity)), temp_name.clone())
                    });
                }

                if resp.lost_focus() && !selection_changed && temp_name != old_name {
                    ui_actions.push(EngineUiAction::ModifyName(entity, old_name, temp_name));
                    ctx.data_mut(|d| d.remove::<String>(egui::Id::new(("name_edit", entity))));
                }
            });
        });
    ui.add_space(4.0);
}