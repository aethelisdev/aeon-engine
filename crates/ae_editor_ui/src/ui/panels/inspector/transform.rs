// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Transform Card
//!
//! Renders Position, Rotation (Euler/Quaternion), and Scale vector rows with live updating and undo/redo support.

use super::widgets::{
    draw_inspector_card, draw_vec3_row, euler_deg_to_quaternion, quaternion_to_euler_deg,
};
use crate::ui::EngineUiAction;

/// Parameters for rendering the Transform inspector card.
pub struct TransformCardParams<'a> {
    pub world: &'a hecs::World,
    pub entity: hecs::Entity,
    pub inspector_euler: &'a mut [f32; 3],
    pub selection_changed: bool,
    pub editor_state: &'a ae_editor::editor_state::EditorState,
    pub ui_actions: &'a mut Vec<EngineUiAction>,
}

/// Renders the Transform inspector card with Position, Rotation, and Scale rows.
pub fn draw_transform_card(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    params: TransformCardParams<'_>,
) {
    let world = params.world;
    let entity = params.entity;
    let inspector_euler = params.inspector_euler;
    let selection_changed = params.selection_changed;
    let editor_state = params.editor_state;
    let ui_actions = params.ui_actions;

    draw_inspector_card(ui, "Transform", "📐", egui::Color32::WHITE, false, |ui| {
        ui.push_id(("transform_scope", entity), |ui| {
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
                    let (changed, drag_started, drag_stopped, is_dragging, reset_clicked) =
                        draw_vec3_row(ui, "Position", &mut pos_arr, 0.05, 3, 0.0);
                    px = pos_arr[0];
                    py = pos_arr[1];
                    pz = pos_arr[2];

                    if reset_clicked && !selection_changed {
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
                        && !selection_changed
                        && let Ok(old) = world.get::<&ae_core::ecs::Position>(entity)
                    {
                        ctx.data_mut(|d| d.insert_temp(pos_id, [old.x, old.y, old.z]));
                    }
                    if changed && !selection_changed {
                        let new_p = ae_core::ecs::Position {
                            x: px,
                            y: py,
                            z: pz,
                        };
                        if is_dragging {
                            ui_actions.push(EngineUiAction::LiveUpdatePosition(entity, new_p));
                        } else {
                            let old_pos = ctx
                                .data(|d| d.get_temp::<[f32; 3]>(pos_id))
                                .map(|arr| ae_core::ecs::Position {
                                    x: arr[0],
                                    y: arr[1],
                                    z: arr[2],
                                })
                                .unwrap_or_else(|| {
                                    if let Ok(p) = world.get::<&ae_core::ecs::Position>(entity) {
                                        *p
                                    } else {
                                        new_p
                                    }
                                });
                            ui_actions.push(EngineUiAction::ModifyPosition(entity, old_pos, new_p));
                        }
                    }
                    if drag_stopped && !selection_changed {
                        let new_p = ae_core::ecs::Position {
                            x: px,
                            y: py,
                            z: pz,
                        };
                        if let Some(arr) = ctx.data(|d| d.get_temp::<[f32; 3]>(pos_id)) {
                            let old_pos = ae_core::ecs::Position {
                                x: arr[0],
                                y: arr[1],
                                z: arr[2],
                            };
                            if old_pos.x != new_p.x || old_pos.y != new_p.y || old_pos.z != new_p.z
                            {
                                ui_actions
                                    .push(EngineUiAction::ModifyPosition(entity, old_pos, new_p));
                            }
                        }
                        ctx.data_mut(|d| d.remove::<[f32; 3]>(pos_id));
                    }

                    // 2. Rotation Row
                    if !selection_changed
                        && !changed
                        && !is_dragging
                        && let Ok(rot) = world.get::<&ae_core::ecs::Rotation>(entity)
                    {
                        *inspector_euler = quaternion_to_euler_deg(*rot);
                    }
                    let mut rx = inspector_euler[0];
                    let mut ry = inspector_euler[1];
                    let mut rz = inspector_euler[2];

                    let mut rot_arr = [rx, ry, rz];
                    let (
                        rot_changed,
                        rot_drag_started,
                        rot_drag_stopped,
                        rot_is_dragging,
                        rot_reset_clicked,
                    ) = draw_vec3_row(ui, "Rotation", &mut rot_arr, 1.0, 1, 0.0);
                    rx = rot_arr[0];
                    ry = rot_arr[1];
                    rz = rot_arr[2];

                    if rot_reset_clicked && !selection_changed {
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
                    if rot_drag_started
                        && !selection_changed
                        && let Ok(old) = world.get::<&ae_core::ecs::Rotation>(entity)
                    {
                        ctx.data_mut(|d| d.insert_temp(rot_id, [old.x, old.y, old.z, old.w]));
                    }
                    if rot_changed && !selection_changed {
                        inspector_euler[0] = rx;
                        inspector_euler[1] = ry;
                        inspector_euler[2] = rz;

                        let new_r = euler_deg_to_quaternion(
                            inspector_euler[0],
                            inspector_euler[1],
                            inspector_euler[2],
                        );
                        if rot_is_dragging {
                            ui_actions.push(EngineUiAction::LiveUpdateRotation(entity, new_r));
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
                                    if let Ok(r) = world.get::<&ae_core::ecs::Rotation>(entity) {
                                        *r
                                    } else {
                                        new_r
                                    }
                                });
                            ui_actions.push(EngineUiAction::ModifyRotation(entity, old_rot, new_r));
                        }
                    }
                    if rot_drag_stopped && !selection_changed {
                        let new_r = euler_deg_to_quaternion(
                            inspector_euler[0],
                            inspector_euler[1],
                            inspector_euler[2],
                        );
                        if let Some(arr) = ctx.data(|d| d.get_temp::<[f32; 4]>(rot_id)) {
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
                                ui_actions
                                    .push(EngineUiAction::ModifyRotation(entity, old_rot, new_r));
                            }
                        }
                        ctx.data_mut(|d| d.remove::<[f32; 4]>(rot_id));
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
                        scale_changed,
                        scale_drag_started,
                        scale_drag_stopped,
                        scale_is_dragging,
                        scale_reset_clicked,
                    ) = draw_vec3_row(ui, "Scale", &mut scale_arr, 0.01, 3, 1.0);
                    sx = scale_arr[0];
                    sy = scale_arr[1];
                    sz = scale_arr[2];

                    if scale_reset_clicked && !selection_changed {
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
                    if scale_drag_started
                        && !selection_changed
                        && let Ok(old) = world.get::<&ae_core::ecs::Scale>(entity)
                    {
                        ctx.data_mut(|d| d.insert_temp(scale_id, [old.x, old.y, old.z]));
                    }
                    if scale_changed && !selection_changed {
                        let new_s = ae_core::ecs::Scale {
                            x: sx,
                            y: sy,
                            z: sz,
                        };
                        if scale_is_dragging {
                            ui_actions.push(EngineUiAction::LiveUpdateScale(entity, new_s));
                        } else {
                            let old_scale = ctx
                                .data(|d| d.get_temp::<[f32; 3]>(scale_id))
                                .map(|arr| ae_core::ecs::Scale {
                                    x: arr[0],
                                    y: arr[1],
                                    z: arr[2],
                                })
                                .unwrap_or_else(|| {
                                    if let Ok(s) = world.get::<&ae_core::ecs::Scale>(entity) {
                                        *s
                                    } else {
                                        new_s
                                    }
                                });
                            ui_actions.push(EngineUiAction::ModifyScale(entity, old_scale, new_s));
                        }
                    }
                    if scale_drag_stopped && !selection_changed {
                        let new_s = ae_core::ecs::Scale {
                            x: sx,
                            y: sy,
                            z: sz,
                        };
                        if let Some(arr) = ctx.data(|d| d.get_temp::<[f32; 3]>(scale_id)) {
                            let old_scale = ae_core::ecs::Scale {
                                x: arr[0],
                                y: arr[1],
                                z: arr[2],
                            };
                            if old_scale.x != new_s.x
                                || old_scale.y != new_s.y
                                || old_scale.z != new_s.z
                            {
                                ui_actions
                                    .push(EngineUiAction::ModifyScale(entity, old_scale, new_s));
                            }
                        }
                        ctx.data_mut(|d| d.remove::<[f32; 3]>(scale_id));
                    }
                });
        });
    });
}