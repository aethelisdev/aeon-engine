// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::widgets::{
    draw_inspector_card, draw_vec3_row, euler_deg_to_quaternion, quaternion_to_euler_deg,
};
use crate::ui::{EngineUi, EngineUiAction};

impl EngineUi {
    /// Renders the internal content of the Entity Component Inspector panel.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_inspector_content(
        ui: &mut egui::Ui,
        world: &hecs::World,
        selected_entity: &mut Option<hecs::Entity>,
        last_selected_entity: &mut Option<hecs::Entity>,
        inspector_euler: &mut [f32; 3],
        inspector_color_hex: &mut String,
        saved_swatches: &mut Vec<[f32; 4]>,
        is_editing: bool,
        ui_actions: &mut Vec<EngineUiAction>,
        editor_state: &ae_editor::editor_state::EditorState,
        camera: &ae_renderer::camera::Camera,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        _textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    ) {
        let ctx = ui.ctx().clone();
        ui.add_enabled_ui(is_editing, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(entity) = *selected_entity {
                    if world.contains(entity) {
                        if let Ok(name) = world.get::<&ae_core::ecs::Name>(entity) {
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
                                            - if world
                                                .get::<&ae_core::ecs::TransformDirty>(entity)
                                                .is_ok()
                                            {
                                                60.0
                                            } else {
                                                0.0
                                            };
                                        let resp = ui.add_sized(
                                            egui::vec2(text_width.max(60.0), 19.0),
                                            egui::TextEdit::singleline(&mut temp_name),
                                        );
                                        if editor_state.focus_rename {
                                            resp.request_focus();
                                        }

                                        if world
                                            .get::<&ae_core::ecs::TransformDirty>(entity)
                                            .is_ok()
                                        {
                                            ui.label(
                                                egui::RichText::new("[DIRTY]")
                                                    .color(egui::Color32::RED)
                                                    .strong(),
                                            )
                                            .on_hover_text("Awaiting physics update...");
                                        }
                                        if resp.changed() {
                                            ctx.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new(("name_edit", entity)),
                                                    temp_name.clone(),
                                                )
                                            });
                                        }

                                        if resp.lost_focus() && temp_name != old_name {
                                            ui_actions.push(EngineUiAction::ModifyName(
                                                entity, old_name, temp_name,
                                            ));
                                            ctx.data_mut(|d| {
                                                d.remove::<String>(egui::Id::new((
                                                    "name_edit",
                                                    entity,
                                                )))
                                            });
                                        }
                                    });
                                });
                            ui.add_space(4.0);
                        }

                        {
                            draw_inspector_card(
                                ui,
                                "Transform",
                                "📐",
                                egui::Color32::WHITE,
                                false,
                                |ui| {
                                    if *last_selected_entity != Some(entity) {
                                        *last_selected_entity = Some(entity);
                                        if let Ok(rot) = world.get::<&ae_core::ecs::Rotation>(entity) {
                                            *inspector_euler = quaternion_to_euler_deg(*rot);
                                        }
                                    }

                                    egui::Grid::new(("transform_grid", entity))
                                        .num_columns(2)
                                        .spacing([4.0, 4.0])
                                        .show(ui, |ui| {
                                        // 1. Position Row
                                        let mut px = 0.0;
                                        let mut py = 0.0;
                                        let mut pz = 0.0;
                                        if let Ok(pos) = world.get::<&ae_core::ecs::Position>(entity) {
                                            px = pos.x;
                                            py = pos.y;
                                            pz = pos.z;
                                        }

                                        let mut pos_arr = [px, py, pz];
                                let (
                                    changed,
                                    drag_started,
                                    drag_stopped,
                                    is_dragging,
                                    reset_clicked,
                                ) = draw_vec3_row(
                                    ui,
                                    "Position",
                                    &mut pos_arr,
                                    0.05,
                                    3,
                                    0.0,
                                );
                                px = pos_arr[0];
                                py = pos_arr[1];
                                pz = pos_arr[2];

                                if reset_clicked {
                                    for &ent in &editor_state.selected_entities {
                                        if let Ok(old) = world.get::<&ae_core::ecs::Position>(ent) {
                                            ui_actions.push(EngineUiAction::ModifyPosition(
                                                ent,
                                                *old,
                                                ae_core::ecs::Position {
                                                    x: 0.0,
                                                    y: 0.0,
                                                    z: 0.0,
                                                },
                                            ));
                                        }
                                    }
                                }
                                let pos_id = egui::Id::new(("drag_pos", entity));
                                if drag_started
                                    && let Ok(old) = world.get::<&ae_core::ecs::Position>(entity) {
                                        ctx.data_mut(|d| {
                                            d.insert_temp(
                                                pos_id,
                                                [old.x, old.y, old.z],
                                            )
                                        });
                                    }
                                if changed {
                                    let new_p = ae_core::ecs::Position {
                                        x: px,
                                        y: py,
                                        z: pz,
                                    };
                                    if is_dragging {
                                        ui_actions.push(
                                            EngineUiAction::LiveUpdatePosition(
                                                entity, new_p,
                                            ),
                                        );
                                    } else {
                                        let old_pos = ctx
                                            .data(|d| d.get_temp::<[f32; 3]>(pos_id))
                                            .map(|arr| ae_core::ecs::Position {
                                                x: arr[0],
                                                y: arr[1],
                                                z: arr[2],
                                            })
                                            .unwrap_or_else(|| {
                                                if let Ok(p) = world.get::<&ae_core::ecs::Position>(entity) { *p } else { new_p }
                                            });
                                        ui_actions.push(
                                            EngineUiAction::ModifyPosition(
                                                entity, old_pos, new_p,
                                            ),
                                        );
                                    }
                                }
                                if drag_stopped {
                                    let new_p = ae_core::ecs::Position {
                                        x: px,
                                        y: py,
                                        z: pz,
                                    };
                                    if let Some(arr) =
                                        ctx.data(|d| d.get_temp::<[f32; 3]>(pos_id))
                                    {
                                        let old_pos = ae_core::ecs::Position {
                                            x: arr[0],
                                            y: arr[1],
                                            z: arr[2],
                                        };
                                        if old_pos.x != new_p.x
                                            || old_pos.y != new_p.y
                                            || old_pos.z != new_p.z
                                        {
                                            ui_actions.push(
                                                EngineUiAction::ModifyPosition(
                                                    entity, old_pos, new_p,
                                                ),
                                            );
                                        }
                                    }
                                    ctx.data_mut(|d| {
                                        d.remove::<[f32; 3]>(pos_id)
                                    });
                                }

                                // 2. Rotation Row
                                let mut rx = inspector_euler[0];
                                let mut ry = inspector_euler[1];
                                let mut rz = inspector_euler[2];

                                let mut rot_arr = [rx, ry, rz];
                                let (
                                    changed,
                                    drag_started,
                                    drag_stopped,
                                    is_dragging,
                                    reset_clicked,
                                ) = draw_vec3_row(
                                    ui,
                                    "Rotation",
                                    &mut rot_arr,
                                    1.0,
                                    1,
                                    0.0,
                                );
                                rx = rot_arr[0];
                                ry = rot_arr[1];
                                rz = rot_arr[2];

                                if reset_clicked {
                                    for &ent in &editor_state.selected_entities {
                                        if let Ok(old) = world.get::<&ae_core::ecs::Rotation>(ent) {
                                            ui_actions.push(EngineUiAction::ModifyRotation(
                                                ent,
                                                *old,
                                                ae_core::ecs::Rotation::identity(),
                                            ));
                                        }
                                    }
                                }
                                let rot_id = egui::Id::new(("drag_rot", entity));
                                if drag_started
                                    && let Ok(old) = world.get::<&ae_core::ecs::Rotation>(entity) {
                                        ctx.data_mut(|d| {
                                            d.insert_temp(
                                                rot_id,
                                                [old.x, old.y, old.z, old.w],
                                            )
                                        });
                                    }
                                if changed {
                                    inspector_euler[0] = rx;
                                    inspector_euler[1] = ry;
                                    inspector_euler[2] = rz;

                                    let new_r = euler_deg_to_quaternion(
                                        inspector_euler[0],
                                        inspector_euler[1],
                                        inspector_euler[2],
                                    );
                                    if is_dragging {
                                        ui_actions.push(
                                            EngineUiAction::LiveUpdateRotation(
                                                entity, new_r,
                                            ),
                                        );
                                    } else {
                                        let old_rot = ctx
                                            .data(|d| d.get_temp::<[f32; 4]>(rot_id))
                                            .map(|arr| ae_core::ecs::Rotation {
                                                x: arr[0],
                                                y: arr[1],
                                                z: arr[2],
                                                w: arr[3],
                                            })
                                            .unwrap_or_else(|| {
                                                if let Ok(r) = world.get::<&ae_core::ecs::Rotation>(entity) { *r } else { new_r }
                                            });
                                        ui_actions.push(
                                            EngineUiAction::ModifyRotation(
                                                entity, old_rot, new_r,
                                            ),
                                        );
                                    }
                                }
                                if drag_stopped {
                                    let new_r = euler_deg_to_quaternion(
                                        inspector_euler[0],
                                        inspector_euler[1],
                                        inspector_euler[2],
                                    );
                                    if let Some(arr) =
                                        ctx.data(|d| d.get_temp::<[f32; 4]>(rot_id))
                                    {
                                        let old_rot = ae_core::ecs::Rotation {
                                            x: arr[0],
                                            y: arr[1],
                                            z: arr[2],
                                            w: arr[3],
                                        };
                                        if old_rot.x != new_r.x
                                            || old_rot.y != new_r.y
                                            || old_rot.z != new_r.z
                                            || old_rot.w != new_r.w
                                        {
                                            ui_actions.push(
                                                EngineUiAction::ModifyRotation(
                                                    entity, old_rot, new_r,
                                                ),
                                            );
                                        }
                                    }
                                    ctx.data_mut(|d| {
                                        d.remove::<[f32; 4]>(rot_id)
                                    });
                                }

                                // 3. Scale Row
                                let mut sx = 1.0;
                                let mut sy = 1.0;
                                let mut sz = 1.0;
                                if let Ok(scale) = world.get::<&ae_core::ecs::Scale>(entity) {
                                    sx = scale.x;
                                    sy = scale.y;
                                    sz = scale.z;
                                }

                                let mut scale_arr = [sx, sy, sz];
                                let (
                                    changed,
                                    drag_started,
                                    drag_stopped,
                                    is_dragging,
                                    reset_clicked,
                                ) = draw_vec3_row(
                                    ui,
                                    "Scale",
                                    &mut scale_arr,
                                    0.01,
                                    3,
                                    1.0,
                                );
                                sx = scale_arr[0];
                                sy = scale_arr[1];
                                sz = scale_arr[2];

                                if reset_clicked {
                                    for &ent in &editor_state.selected_entities {
                                        if let Ok(old) = world.get::<&ae_core::ecs::Scale>(ent) {
                                            ui_actions.push(EngineUiAction::ModifyScale(
                                                ent,
                                                *old,
                                                ae_core::ecs::Scale {
                                                    x: 1.0,
                                                    y: 1.0,
                                                    z: 1.0,
                                                },
                                            ));
                                        }
                                    }
                                }
                                let scale_id = egui::Id::new(("drag_scale", entity));
                                if drag_started
                                    && let Ok(old) = world.get::<&ae_core::ecs::Scale>(entity) {
                                        ctx.data_mut(|d| {
                                            d.insert_temp(
                                                scale_id,
                                                [old.x, old.y, old.z],
                                            )
                                        });
                                    }
                                if changed {
                                    let new_s = ae_core::ecs::Scale {
                                        x: sx,
                                        y: sy,
                                        z: sz,
                                    };
                                    if is_dragging {
                                        ui_actions.push(
                                            EngineUiAction::LiveUpdateScale(
                                                entity, new_s,
                                            ),
                                        );
                                    } else {
                                        let old_scale = ctx
                                            .data(|d| d.get_temp::<[f32; 3]>(scale_id))
                                            .map(|arr| ae_core::ecs::Scale {
                                                x: arr[0],
                                                y: arr[1],
                                                z: arr[2],
                                            })
                                            .unwrap_or_else(|| {
                                                if let Ok(s) = world.get::<&ae_core::ecs::Scale>(entity) { *s } else { new_s }
                                            });
                                        ui_actions.push(
                                            EngineUiAction::ModifyScale(
                                                entity, old_scale, new_s,
                                            ),
                                        );
                                    }
                                }
                                if drag_stopped {
                                    let new_s = ae_core::ecs::Scale {
                                        x: sx,
                                        y: sy,
                                        z: sz,
                                    };
                                    if let Some(arr) =
                                        ctx.data(|d| d.get_temp::<[f32; 3]>(scale_id))
                                    {
                                        let old_scale = ae_core::ecs::Scale {
                                            x: arr[0],
                                            y: arr[1],
                                            z: arr[2],
                                        };
                                        if old_scale.x != new_s.x
                                            || old_scale.y != new_s.y
                                            || old_scale.z != new_s.z
                                        {
                                            ui_actions.push(
                                                EngineUiAction::ModifyScale(
                                                    entity, old_scale, new_s,
                                                ),
                                            );
                                        }
                                    }
                                    ctx.data_mut(|d| {
                                        d.remove::<[f32; 3]>(scale_id)
                                    });
                                }
                            });
                        });
                    }

                        // --- APPEARANCE (Color & Swatches) ---
                        Self::draw_appearance_section(
                            ui,
                            world,
                            entity,
                            inspector_color_hex,
                            saved_swatches,
                            ui_actions,
                        );

                        // --- MATERIAL EDITOR QUICK LINK ---
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

                        // --- LIGHTING SECTION ---
                        if let Ok(light) = world.get::<&ae_core::ecs::Light>(entity) {
                            draw_inspector_card(
                                ui,
                                "Lighting Settings",
                                "💡",
                                egui::Color32::from_rgb(255, 230, 100),
                                false,
                                |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Color:");
                                        let mut edit_color = light.color;
                                        let res = ui.color_edit_button_rgb(&mut edit_color);
                                        if res.changed() {
                                            ui_actions.push(EngineUiAction::ModifyLightColor(
                                                entity,
                                                light.color,
                                                edit_color,
                                            ));
                                        }
                                    });
                                },
                            );
                        }

                        // --- RIGIDBODY SECTION ---
                        Self::draw_rigidbody_section(ui, world, entity, ui_actions);

                        // --- COLLIDER SECTION ---
                        Self::draw_collider_section(ui, world, entity, ui_actions);

                        // --- CHARACTER CONTROLLER SECTION ---
                        Self::draw_character_controller_section(
                            ui, world, entity, ui_actions,
                        );

                        // --- ANIMATION PLAYER QUICK LINK ---
                        if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                            let state_str = match player.state {
                                ae_animation::AnimationState::Playing => "▶ Playing",
                                ae_animation::AnimationState::Paused => "⏸ Paused",
                                ae_animation::AnimationState::Stopped => "⏹ Stopped",
                            };
                            draw_inspector_card(
                                ui,
                                "Animation Status",
                                "🎬",
                                egui::Color32::from_rgb(255, 150, 200),
                                false,
                                |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Status:");
                                        ui.label(
                                            egui::RichText::new(state_str)
                                                .color(egui::Color32::GREEN)
                                                .strong(),
                                        );
                                        if ui
                                            .button("Open Timeline Studio ↗")
                                            .on_hover_text("Open bottom Animation Timeline Studio panel")
                                            .clicked()
                                        {
                                            ui_actions.push(EngineUiAction::OpenPanel(
                                                crate::ui::panel_layout::PanelId::AnimationTimeline,
                                            ));
                                        }
                                    });
                                },
                            );
                        }

                        // --- AUDIO SOURCE SECTION ---
                        Self::draw_audio_source_section(ui, world, entity, ui_actions);

                        // --- AUDIO LISTENER SECTION ---
                        Self::draw_audio_listener_section(ui, world, entity, ui_actions);

                        // --- PLAYER TAG SECTION ---
                        Self::draw_player_tag_section(ui, world, entity, ui_actions);

                        // --- HIERARCHY / PARENTING SECTION ---
                        Self::draw_parenting_section(ui, world, entity, ui_actions);

                        // --- LOD GROUP SECTION ---
                        Self::draw_lod_section(ui, world, entity, camera, models, ui_actions);

                        // --- BOTTOM ACTION BUTTONS ---
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            Self::draw_add_component_button(ui, world, entity, ui_actions);
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