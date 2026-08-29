// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Entity Component Inspector Panel
//!
//! Orchestrates the inspection, reflection, modification, and prefab export of selected ECS entities.

use super::add_component::draw_add_component_button;
use super::header::draw_entity_header;
use super::transform::{TransformCardParams, draw_transform_card};
use super::widgets::{draw_inspector_card, quaternion_to_euler_deg};
use crate::ui::{EngineUi, EngineUiAction};

/// Parameters for drawing the Inspector panel contents.
pub struct InspectorContentParams<'a> {
    pub world: &'a hecs::World,
    pub selected_entity: &'a mut Option<hecs::Entity>,
    pub last_selected_entity: &'a mut Option<hecs::Entity>,
    pub inspector_euler: &'a mut [f32; 3],
    pub inspector_color_hex: &'a mut String,
    pub saved_swatches: &'a mut Vec<[f32; 4]>,
    pub is_editing: bool,
    pub ui_actions: &'a mut Vec<EngineUiAction>,
    pub editor_state: &'a ae_editor::editor_state::EditorState,
    pub camera: &'a ae_renderer::camera::Camera,
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
}

impl EngineUi {
    /// Renders the internal content of the Entity Component Inspector panel.
    pub fn draw_inspector_content(ui: &mut egui::Ui, params: InspectorContentParams<'_>) {
        let world = params.world;
        let selected_entity = params.selected_entity;
        let last_selected_entity = params.last_selected_entity;
        let inspector_euler = params.inspector_euler;
        let inspector_color_hex = params.inspector_color_hex;
        let saved_swatches = params.saved_swatches;
        let is_editing = params.is_editing;
        let ui_actions = params.ui_actions;
        let editor_state = params.editor_state;
        let camera = params.camera;
        let models = params.models;
        let textures = params.textures;

        let ctx = ui.ctx().clone();
        ui.add_enabled_ui(is_editing, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(entity) = *selected_entity {
                    if world.contains(entity) {
                        let selection_changed = *last_selected_entity != Some(entity);
                        if selection_changed {
                            *last_selected_entity = Some(entity);
                            ctx.memory_mut(|m| m.stop_text_input());
                            if let Ok(rot) = world.get::<&ae_core::ecs::Rotation>(entity) {
                                *inspector_euler = quaternion_to_euler_deg(*rot);
                            }
                            if let Ok(col) = world.get::<&ae_core::ecs::Color>(entity) {
                                let r = (col.r.clamp(0.0, 1.0) * 255.0) as u8;
                                let g = (col.g.clamp(0.0, 1.0) * 255.0) as u8;
                                let b = (col.b.clamp(0.0, 1.0) * 255.0) as u8;
                                *inspector_color_hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
                            }
                        }

                        // 1. Entity Name & Status Header
                        if let Ok(name) = world.get::<&ae_core::ecs::Name>(entity) {
                            draw_entity_header(
                                ui,
                                &ctx,
                                super::header::EntityHeaderParams {
                                    world,
                                    entity,
                                    name: &name,
                                    selection_changed,
                                    focus_rename: editor_state.focus_rename,
                                    ui_actions,
                                },
                            );
                        }

                        // 2. Transform Card (Position, Rotation, Scale)
                        draw_transform_card(
                            ui,
                            &ctx,
                            TransformCardParams {
                                world,
                                entity,
                                inspector_euler,
                                selection_changed,
                                editor_state,
                                ui_actions,
                            },
                        );

                        // 3. Dynamic Component Rendering Via Registry & Reflection
                        let mut ctx = super::registry::InspectorContext {
                            world,
                            entity,
                            ui_actions,
                            editor_state,
                            camera,
                            models,
                            textures,
                            inspector_color_hex,
                            saved_swatches,
                        };

                        let mut rendered_component_types = std::collections::HashSet::new();

                        // 3a. Specialized UI handlers from InspectorUiRegistry
                        let ui_registry = super::registry::InspectorUiRegistry::global();
                        for handler in ui_registry.handlers() {
                            if handler.has_component(world, entity) {
                                rendered_component_types.insert(handler.component_name());
                                handler.render_ui(ui, &mut ctx);
                            }
                        }

                        // 3b. Fallback: Automatically render via dynamic reflection
                        let comp_registry = ae_core::registry::ComponentRegistry::global();
                        for handler in comp_registry.handlers() {
                            let type_name = handler.type_name();
                            if !rendered_component_types.contains(type_name)
                                && !super::dynamic_reflection::is_internal_or_specialized(type_name)
                                && handler.has_component(world, entity)
                            {
                                super::dynamic_reflection::draw_dynamic_component_card(
                                    ui, &mut ctx, &**handler,
                                );
                            }
                        }

                        // 4. Material / Submesh Quick Links
                        if let Ok(model_id) = world.get::<&ae_core::ecs::ModelId>(entity) {
                            let submesh_count = models
                                .get(model_id.0)
                                .map_or(0, |m| m.submeshes.len());
                            draw_inspector_card(
                                ui,
                                &format!("Material ({} slots)", submesh_count),
                                "🎨",
                                egui::Color32::WHITE,
                                false,
                                |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Submesh Materials:");
                                        if ui
                                            .button("Open Material Editor ↗")
                                            .on_hover_text("Open dedicated Material & Submesh Editor tab")
                                            .clicked()
                                        {
                                            ui_actions.push(EngineUiAction::OpenPanel(
                                                crate::ui::panel_layout::PanelId::MaterialEditor,
                                            ));
                                        }
                                    });
                                },
                            );
                        } else if world.get::<&ae_core::ecs::SpriteId>(entity).is_ok() {
                            draw_inspector_card(
                                ui,
                                "Texture & Material",
                                "🖼️",
                                egui::Color32::WHITE,
                                false,
                                |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Sprite Material:");
                                        if ui
                                            .button("Open Material Editor ↗")
                                            .clicked()
                                        {
                                            ui_actions.push(EngineUiAction::OpenPanel(
                                                crate::ui::panel_layout::PanelId::MaterialEditor,
                                            ));
                                        }
                                    });
                                },
                            );
                        }

                        // 5. Bottom Action Buttons (Add Component, Save as Prefab)
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            draw_add_component_button(ui, world, entity, ui_actions);
                            if ui
                                .button("💾 Save as Prefab")
                                .on_hover_text(
                                    "Save selected entity and its components as a reusable .aeprefab asset",
                                )
                                .clicked()
                            {
                                let ent_name = world
                                    .get::<&ae_core::ecs::Name>(entity)
                                    .map(|n| n.0.clone())
                                    .unwrap_or_else(|_| "Entity".to_string());
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Aeon Prefab", &["aeprefab"])
                                    .set_file_name(format!("{}.aeprefab", ent_name))
                                    .save_file()
                                {
                                    ui_actions.push(EngineUiAction::SaveEntityAsPrefab(
                                        entity, path,
                                    ));
                                }
                            }
                        });
                    } else {
                        *selected_entity = None;
                    }
                } else {
                    ui.label(
                        "No object selected. Select an object from the list on the left.",
                    );
                }
            });
        });
    }
}