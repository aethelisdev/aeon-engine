// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::{EngineUi, EngineUiAction};

impl EngineUi {
    /// Legacy egui drawing routine for the Material & Submesh Editor panel.
    /// Active editor docking rendering has been migrated to 100% native Iris UI GPU SDF
    /// under [`crate::ui::iris_bridge::material`]. This method is retained for fallback reference.
    pub fn draw_material_editor_content(
        ui: &mut egui::Ui,
        world: &hecs::World,
        selected_entity: Option<hecs::Entity>,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        if let Some(entity) = selected_entity {
            if world.contains(entity) {
                Self::draw_texture_section(ui, world, entity, textures, models, ui_actions);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Selected entity does not exist in world.");
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No object selected. Select a 3D model or sprite in the viewport or hierarchy to edit materials.");
            });
        }
    }
}