// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::widgets::draw_vec3_row;
use crate::ui::{EngineUi, EngineUiAction};

impl EngineUi {
    /// Renders the right-side Inspector panel.
    pub fn draw_inspector_panel(
        selected_entity: &mut Option<hecs::Entity>,
        last_selected_entity: &mut Option<hecs::Entity>,
        inspector_euler: &mut [f32; 3],
        inspector_color_hex: &mut String,
        saved_swatches: &mut Vec<[f32; 4]>,
        _current_edit_snapshot: &mut Option<ae_editor::undo_redo::EntitySnapshot>,
        ui: &mut egui::Ui,
        world: &hecs::World,
        _undo_stack: &[ae_editor::undo_redo::Command],
        _redo_stack: &[ae_editor::undo_redo::Command],
        is_editing: bool,
        ui_actions: &mut Vec<EngineUiAction>,
        editor_state: &ae_editor::editor_state::EditorState,
        camera: &ae_renderer::camera::Camera,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    ) -> Option<egui::Rect> {
        let ctx = ui.ctx().clone();
        let response = egui::Panel::right("inspector_panel")
            .resizable(true)
            .default_size(350.0)
            .show(ui, |ui| {
                ui.add_enabled_ui(is_editing, |ui| {
                    ui.heading("Inspector");
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(entity) = *selected_entity {
                            if world.contains(entity) {
                                if let Ok(name) = world.get::<&ae_core::ecs::Name>(entity) {
                                    let mut temp_name = ctx.data_mut(|d| {
                                        d.get_temp::<String>(egui::Id::new(("name_edit", entity)))
                                            .unwrap_or_else(|| name.0.clone())
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Name:");
                                        let old_name = name.0.clone();

                                        let resp = ui.text_edit_singleline(&mut temp_name);
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
                                }
                                ui.add_space(10.0);

                                {
                                    ui.group(|ui| {
                                        ui.set_width(ui.available_width());
                                        ui.style_mut().spacing.item_spacing =
                                            egui::vec2(8.0, 8.0);
                                        ui.label(
                                            egui::RichText::new("Transform")
                                                .strong()
                                                .color(egui::Color32::WHITE),
                                        );

                                        // --- SELECTION CHANGE SYNC ---
                                        if *last_selected_entity != *selected_entity {
                                            *last_selected_entity = *selected_entity;
                                            ctx.memory_mut(|m| m.stop_text_input());

                                            if let Ok(r) =
                                                world.get::<&ae_core::ecs::Rotation>(entity)
                                            {
                                                let current_q =
                                                    cgmath::Quaternion::new(r.w, r.x, r.y, r.z);
                                                let euler_rad: cgmath::Euler<cgmath::Rad<f32>> =
                                                    cgmath::Euler::from(current_q);
                                                inspector_euler[0] =
                                                    cgmath::Deg::from(euler_rad.x).0;
                                                inspector_euler[1] =
                                                    cgmath::Deg::from(euler_rad.y).0;
                                                inspector_euler[2] =
                                                    cgmath::Deg::from(euler_rad.z).0;
                                            }
                                            if let Ok(c) =
                                                world.get::<&ae_core::ecs::Color>(entity)
                                            {
                                                *inspector_color_hex = format!(
                                                    "#{:02x}{:02x}{:02x}",
                                                    (c.r * 255.0) as u8,
                                                    (c.g * 255.0) as u8,
                                                    (c.b * 255.0) as u8
                                                );
                                            } else {
                                                *inspector_color_hex = "#4d4d4d".to_string();
                                            }
                                        } else if let Ok(r) =
                                            world.get::<&ae_core::ecs::Rotation>(entity)
                                        {
                                            let current_q =
                                                cgmath::Quaternion::new(r.w, r.x, r.y, r.z);
                                            let ui_q =
                                                cgmath::Quaternion::from(cgmath::Euler {
                                                    x: cgmath::Deg(inspector_euler[0]),
                                                    y: cgmath::Deg(inspector_euler[1]),
                                                    z: cgmath::Deg(inspector_euler[2]),
                                                });
                                            let dot = current_q.v.x * ui_q.v.x
                                                + current_q.v.y * ui_q.v.y
                                                + current_q.v.z * ui_q.v.z
                                                + current_q.s * ui_q.s;
                                            if dot.abs() < 0.9999 {
                                                let euler_rad: cgmath::Euler<cgmath::Rad<f32>> =
                                                    cgmath::Euler::from(current_q);
                                                inspector_euler[0] =
                                                    cgmath::Deg::from(euler_rad.x).0;
                                                inspector_euler[1] =
                                                    cgmath::Deg::from(euler_rad.y).0;
                                                inspector_euler[2] =
                                                    cgmath::Deg::from(euler_rad.z).0;
                                            }
                                        }

                                        ui.push_id(("transform_grid_scope", entity), |ui| {
                                            egui::Grid::new("transform_grid")
                                                .num_columns(5)
                                                .spacing([5.0, 10.0])
                                                .min_col_width(52.0)
                                                .show(ui, |ui| {
                                                    // --- POSITION ---
                                                    let (mut px, mut py, mut pz) = {
                                                        if let Ok(p) = world
                                                            .get::<&ae_core::ecs::Position>(entity)
                                                        {
                                                            (p.x, p.y, p.z)
                                                        } else {
                                                            (0.0, 0.0, 0.0)
                                                        }
                                                    };
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
                                                        0.1,
                                                        3,
                                                        0.0,
                                                    );
                                                    px = pos_arr[0];
                                                    py = pos_arr[1];
                                                    pz = pos_arr[2];

                                                    if reset_clicked {
                                                        for &ent in &editor_state.selected_entities
                                                        {
                                                            if let Ok(old) = world
                                                                .get::<&ae_core::ecs::Position>(ent)
                                                            {
                                                                ui_actions.push(
                                                                    EngineUiAction::ModifyPosition(
                                                                        ent,
                                                                        *old,
                                                                        ae_core::ecs::Position {
                                                                            x: 0.0,
                                                                            y: 0.0,
                                                                            z: 0.0,
                                                                        },
                                                                    ),
                                                                );
                                                            }
                                                        }
                                                    }
                                                    let pos_id =
                                                        egui::Id::new(("drag_pos", entity));
                                                    if drag_started {
                                                        if let Ok(old) = world
                                                            .get::<&ae_core::ecs::Position>(entity)
                                                        {
                                                            ctx.data_mut(|d| {
                                                                d.insert_temp(
                                                                    pos_id,
                                                                    [old.x, old.y, old.z],
                                                                )
                                                            });
                                                        }
                                                    }
                                                    if changed {
                                                        let new_pos = ae_core::ecs::Position {
                                                            x: px,
                                                            y: py,
                                                            z: pz,
                                                        };
                                                        if is_dragging {
                                                            ui_actions.push(
                                                                EngineUiAction::LiveUpdatePosition(
                                                                    entity, new_pos,
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
                                                                    if let Ok(p) = world.get::<&ae_core::ecs::Position>(entity) { *p } else { new_pos }
                                                                });
                                                            ui_actions.push(
                                                                EngineUiAction::ModifyPosition(
                                                                    entity, old_pos, new_pos,
                                                                ),
                                                            );
                                                        }
                                                    }
                                                    if drag_stopped {
                                                        let new_pos = ae_core::ecs::Position {
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
                                                            if old_pos.x != new_pos.x
                                                                || old_pos.y != new_pos.y
                                                                || old_pos.z != new_pos.z
                                                            {
                                                                ui_actions.push(
                                                                    EngineUiAction::ModifyPosition(
                                                                        entity, old_pos, new_pos,
                                                                    ),
                                                                );
                                                            }
                                                        }
                                                        ctx.data_mut(|d| {
                                                            d.remove::<[f32; 3]>(pos_id)
                                                        });
                                                    }

                                                    // --- ROTATION ---
                                                    let (
                                                        changed,
                                                        drag_started,
                                                        drag_stopped,
                                                        is_dragging,
                                                        reset_clicked,
                                                    ) = draw_vec3_row(
                                                        ui,
                                                        "Rotation",
                                                        inspector_euler,
                                                        1.0,
                                                        1,
                                                        0.0,
                                                    );

                                                    if reset_clicked {
                                                        let ident = ae_core::ecs::Rotation {
                                                            x: 0.0,
                                                            y: 0.0,
                                                            z: 0.0,
                                                            w: 1.0,
                                                        };
                                                        for &ent in &editor_state.selected_entities
                                                        {
                                                            if let Ok(old) = world
                                                                .get::<&ae_core::ecs::Rotation>(ent)
                                                            {
                                                                ui_actions.push(
                                                                    EngineUiAction::ModifyRotation(
                                                                        ent, *old, ident,
                                                                    ),
                                                                );
                                                            }
                                                        }
                                                    }
                                                    let rot_id =
                                                        egui::Id::new(("drag_rot", entity));
                                                    if drag_started {
                                                        if let Ok(old) = world
                                                            .get::<&ae_core::ecs::Rotation>(entity)
                                                        {
                                                            ctx.data_mut(|d| {
                                                                d.insert_temp(
                                                                    rot_id,
                                                                    [old.x, old.y, old.z, old.w],
                                                                )
                                                            });
                                                        }
                                                    }
                                                    if changed {
                                                        let q =
                                                            cgmath::Quaternion::from(cgmath::Euler {
                                                                x: cgmath::Deg(inspector_euler[0]),
                                                                y: cgmath::Deg(inspector_euler[1]),
                                                                z: cgmath::Deg(inspector_euler[2]),
                                                            });
                                                        let new_rot = ae_core::ecs::Rotation {
                                                            x: q.v.x,
                                                            y: q.v.y,
                                                            z: q.v.z,
                                                            w: q.s,
                                                        };
                                                        if is_dragging {
                                                            ui_actions.push(
                                                                EngineUiAction::LiveUpdateRotation(
                                                                    entity, new_rot,
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
                                                                    if let Ok(r) = world.get::<&ae_core::ecs::Rotation>(entity) { *r } else { new_rot }
                                                                });
                                                            ui_actions.push(
                                                                EngineUiAction::ModifyRotation(
                                                                    entity, old_rot, new_rot,
                                                                ),
                                                            );
                                                        }
                                                    }
                                                    if drag_stopped {
                                                        let q =
                                                            cgmath::Quaternion::from(cgmath::Euler {
                                                                x: cgmath::Deg(inspector_euler[0]),
                                                                y: cgmath::Deg(inspector_euler[1]),
                                                                z: cgmath::Deg(inspector_euler[2]),
                                                            });
                                                        let new_rot = ae_core::ecs::Rotation {
                                                            x: q.v.x,
                                                            y: q.v.y,
                                                            z: q.v.z,
                                                            w: q.s,
                                                        };
                                                        if let Some(arr) =
                                                            ctx.data(|d| d.get_temp::<[f32; 4]>(rot_id))
                                                        {
                                                            let old_rot = ae_core::ecs::Rotation {
                                                                x: arr[0],
                                                                y: arr[1],
                                                                z: arr[2],
                                                                w: arr[3],
                                                            };
                                                            if old_rot.x != new_rot.x
                                                                || old_rot.y != new_rot.y
                                                                || old_rot.z != new_rot.z
                                                                || old_rot.w != new_rot.w
                                                            {
                                                                ui_actions.push(
                                                                    EngineUiAction::ModifyRotation(
                                                                        entity, old_rot, new_rot,
                                                                    ),
                                                                );
                                                            }
                                                        }
                                                        ctx.data_mut(|d| {
                                                            d.remove::<[f32; 4]>(rot_id)
                                                        });
                                                    }

                                                    // --- SCALE ---
                                                    let (mut sx, mut sy, mut sz) = {
                                                        if let Ok(s) = world
                                                            .get::<&ae_core::ecs::Scale>(entity)
                                                        {
                                                            (s.x, s.y, s.z)
                                                        } else {
                                                            (1.0, 1.0, 1.0)
                                                        }
                                                    };
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
                                                        for &ent in &editor_state.selected_entities
                                                        {
                                                            if let Ok(old) = world
                                                                .get::<&ae_core::ecs::Scale>(ent)
                                                            {
                                                                ui_actions.push(
                                                                    EngineUiAction::ModifyScale(
                                                                        ent,
                                                                        *old,
                                                                        ae_core::ecs::Scale {
                                                                            x: 1.0,
                                                                            y: 1.0,
                                                                            z: 1.0,
                                                                        },
                                                                    ),
                                                                );
                                                            }
                                                        }
                                                    }
                                                    let scale_id =
                                                        egui::Id::new(("drag_scale", entity));
                                                    if drag_started {
                                                        if let Ok(old) = world
                                                            .get::<&ae_core::ecs::Scale>(entity)
                                                        {
                                                            ctx.data_mut(|d| {
                                                                d.insert_temp(
                                                                    scale_id,
                                                                    [old.x, old.y, old.z],
                                                                )
                                                            });
                                                        }
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

                                // --- TEXTURE & MATERIAL SECTION ---
                                Self::draw_texture_section(ui, world, entity, textures, ui_actions);

                                // --- LIGHTING SECTION ---
                                if let Ok(light) = world.get::<&ae_core::ecs::Light>(entity) {
                                    ui.group(|ui| {
                                        ui.set_width(ui.available_width());
                                        ui.label("Lighting Settings");
                                        ui.horizontal(|ui| {
                                            ui.label("Color:");
                                            let mut edit_color = light.color.clone();
                                            let res = ui.color_edit_button_rgb(&mut edit_color);
                                            if res.changed() {
                                                ui_actions.push(EngineUiAction::ModifyLightColor(
                                                    entity,
                                                    light.color,
                                                    edit_color,
                                                ));
                                            }
                                        });
                                    });
                                }

                                // --- RIGIDBODY SECTION ---
                                Self::draw_rigidbody_section(ui, world, entity, ui_actions);

                                // --- COLLIDER SECTION ---
                                Self::draw_collider_section(ui, world, entity, ui_actions);

                                // --- CHARACTER CONTROLLER SECTION ---
                                Self::draw_character_controller_section(
                                    ui, world, entity, ui_actions,
                                );

                                // --- ANIMATION PLAYER SECTION ---
                                Self::draw_animation_section(ui, world, entity, models, ui_actions);

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
            });
        Some(response.response.rect)
    }
}