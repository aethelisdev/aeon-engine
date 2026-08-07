// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::EngineUi;
use crate::ui::EngineUiAction;
use cgmath::InnerSpace;

mod parenting;
mod physics;

/// Helper to draw a single 3-component float (X, Y, Z) input row in the Inspector grid
/// with full wgpu/egui drag, undo snapshot, and reset triggers.
fn draw_vec3_row(
    ui: &mut egui::Ui,
    label: &str,
    values: &mut [f32; 3],
    speed: f32,
    decimals: usize,
    reset_val: f32,
) -> (bool, bool, bool, bool, bool) {
    ui.label(label);
    let r_x = ui.add(
        egui::DragValue::new(&mut values[0])
            .prefix("X: ")
            .speed(speed)
            .fixed_decimals(decimals),
    );
    let r_y = ui.add(
        egui::DragValue::new(&mut values[1])
            .prefix("Y: ")
            .speed(speed)
            .fixed_decimals(decimals),
    );
    let r_z = ui.add(
        egui::DragValue::new(&mut values[2])
            .prefix("Z: ")
            .speed(speed)
            .fixed_decimals(decimals),
    );

    let reset_clicked = ui.button("🔄").clicked();
    if reset_clicked {
        values[0] = reset_val;
        values[1] = reset_val;
        values[2] = reset_val;
    }
    ui.end_row();

    let drag_started = r_x.drag_started() || r_y.drag_started() || r_z.drag_started();
    let drag_stopped = r_x.drag_stopped() || r_y.drag_stopped() || r_z.drag_stopped();
    let changed = r_x.changed() || r_y.changed() || r_z.changed();
    let is_dragging = r_x.dragged() || r_y.dragged() || r_z.dragged();

    (
        changed,
        drag_started,
        drag_stopped,
        is_dragging,
        reset_clicked,
    )
}

impl EngineUi {
    /// Renders the right-side Inspector panel.
    pub(super) fn draw_inspector_panel(
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
                        // If entity no longer in world, swallow error (or drop selected state)
                        if world.contains(entity) {
                            if let Ok(name) = world.get::<&ae_core::ecs::Name>(entity) {
                                // Since we can't mutate name inline anymore, we track it locally
                                // or use a temp state. Wait, the simplest way for a text edit is to
                                // store the string in a temporary state in Egui
                                let mut temp_name = ctx.data_mut(|d| d.get_temp::<String>(egui::Id::new(("name_edit", entity))).unwrap_or_else(|| name.0.clone()));

                                ui.horizontal(|ui| {
                                    ui.label("Name:");
                                    let old_name = name.0.clone();

                                    let resp = ui.text_edit_singleline(&mut temp_name);
                                    if editor_state.focus_rename {
                                        resp.request_focus();
                                    }

                                    if world.get::<&ae_core::ecs::TransformDirty>(entity).is_ok() {
                                        ui.label(egui::RichText::new("[DIRTY]").color(egui::Color32::RED).strong())
                                            .on_hover_text("Awaiting physics update...");
                                    }
                                    if resp.changed() {
                                        ctx.data_mut(|d| d.insert_temp(egui::Id::new(("name_edit", entity)), temp_name.clone()));
                                    }

                                    if resp.lost_focus() && temp_name != old_name {
                                        ui_actions.push(EngineUiAction::ModifyName(entity, old_name, temp_name));
                                        // clear temp state
                                        ctx.data_mut(|d| d.remove::<String>(egui::Id::new(("name_edit", entity))));
                                    }
                                });
                            }
                            ui.add_space(10.0);

                            {
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 8.0);
                                    ui.label(egui::RichText::new("Transform").strong().color(egui::Color32::WHITE));

                                    // --- SELECTION CHANGE SYNC ---
                                    if *last_selected_entity != *selected_entity {
                                        *last_selected_entity = *selected_entity;
                                        ctx.memory_mut(|m| m.stop_text_input());

                                        if let Ok(r) = world.get::<&ae_core::ecs::Rotation>(entity) {
                                            let current_q = cgmath::Quaternion::new(r.w, r.x, r.y, r.z);
                                            let euler_rad: cgmath::Euler<cgmath::Rad<f32>> = cgmath::Euler::from(current_q);
                                            inspector_euler[0] = cgmath::Deg::from(euler_rad.x).0;
                                            inspector_euler[1] = cgmath::Deg::from(euler_rad.y).0;
                                            inspector_euler[2] = cgmath::Deg::from(euler_rad.z).0;
                                        }
                                        if let Ok(c) = world.get::<&ae_core::ecs::Color>(entity) {
                                            *inspector_color_hex = format!("#{:02x}{:02x}{:02x}",
                                                (c.r * 255.0) as u8,
                                                (c.g * 255.0) as u8,
                                                (c.b * 255.0) as u8);
                                        } else {
                                            *inspector_color_hex = "#4d4d4d".to_string();
                                        }
                                    } else if let Ok(r) = world.get::<&ae_core::ecs::Rotation>(entity) {
                                        let current_q = cgmath::Quaternion::new(r.w, r.x, r.y, r.z);
                                        let ui_q = cgmath::Quaternion::from(cgmath::Euler {
                                            x: cgmath::Deg(inspector_euler[0]),
                                            y: cgmath::Deg(inspector_euler[1]),
                                            z: cgmath::Deg(inspector_euler[2]),
                                        });
                                        let dot = current_q.v.x * ui_q.v.x + current_q.v.y * ui_q.v.y + current_q.v.z * ui_q.v.z + current_q.s * ui_q.s;
                                        if dot.abs() < 0.9999 {
                                            let euler_rad: cgmath::Euler<cgmath::Rad<f32>> = cgmath::Euler::from(current_q);
                                            inspector_euler[0] = cgmath::Deg::from(euler_rad.x).0;
                                            inspector_euler[1] = cgmath::Deg::from(euler_rad.y).0;
                                            inspector_euler[2] = cgmath::Deg::from(euler_rad.z).0;
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
                                                    if let Ok(p) = world.get::<&ae_core::ecs::Position>(entity) {
                                                        (p.x, p.y, p.z)
                                                    } else {
                                                        (0.0, 0.0, 0.0)
                                                    }
                                                };
                                                let mut pos_arr = [px, py, pz];
                                                let (changed, drag_started, drag_stopped, is_dragging, reset_clicked) =
                                                    draw_vec3_row(ui, "Position", &mut pos_arr, 0.1, 3, 0.0);
                                                px = pos_arr[0]; py = pos_arr[1]; pz = pos_arr[2];

                                                if reset_clicked {
                                                    for &ent in &editor_state.selected_entities {
                                                        if let Ok(old) = world.get::<&ae_core::ecs::Position>(ent) {
                                                            ui_actions.push(EngineUiAction::ModifyPosition(ent, *old, ae_core::ecs::Position { x: 0.0, y: 0.0, z: 0.0 }));
                                                        }
                                                    }
                                                }
                                                let pos_id = egui::Id::new(("drag_pos", entity));
                                                if drag_started {
                                                    if let Ok(old) = world.get::<&ae_core::ecs::Position>(entity) {
                                                        ctx.data_mut(|d| d.insert_temp(pos_id, [old.x, old.y, old.z]));
                                                    }
                                                }
                                                if changed {
                                                    let new_pos = ae_core::ecs::Position { x: px, y: py, z: pz };
                                                    if is_dragging {
                                                        ui_actions.push(EngineUiAction::LiveUpdatePosition(entity, new_pos));
                                                    } else {
                                                        let old_pos = ctx.data(|d| d.get_temp::<[f32; 3]>(pos_id))
                                                            .map(|arr| ae_core::ecs::Position { x: arr[0], y: arr[1], z: arr[2] })
                                                            .unwrap_or_else(|| {
                                                                if let Ok(p) = world.get::<&ae_core::ecs::Position>(entity) { *p } else { new_pos }
                                                            });
                                                        ui_actions.push(EngineUiAction::ModifyPosition(entity, old_pos, new_pos));
                                                    }
                                                }
                                                if drag_stopped {
                                                    let new_pos = ae_core::ecs::Position { x: px, y: py, z: pz };
                                                    if let Some(arr) = ctx.data(|d| d.get_temp::<[f32; 3]>(pos_id)) {
                                                        let old_pos = ae_core::ecs::Position { x: arr[0], y: arr[1], z: arr[2] };
                                                        if old_pos.x != new_pos.x || old_pos.y != new_pos.y || old_pos.z != new_pos.z {
                                                            ui_actions.push(EngineUiAction::ModifyPosition(entity, old_pos, new_pos));
                                                        }
                                                    }
                                                    ctx.data_mut(|d| d.remove::<[f32; 3]>(pos_id));
                                                }

                                                // --- ROTATION ---
                                                let (changed, drag_started, drag_stopped, is_dragging, reset_clicked) =
                                                    draw_vec3_row(ui, "Rotation", inspector_euler, 1.0, 1, 0.0);

                                                if reset_clicked {
                                                    let ident = ae_core::ecs::Rotation { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
                                                    for &ent in &editor_state.selected_entities {
                                                        if let Ok(old) = world.get::<&ae_core::ecs::Rotation>(ent) {
                                                            ui_actions.push(EngineUiAction::ModifyRotation(ent, *old, ident));
                                                        }
                                                    }
                                                }
                                                let rot_id = egui::Id::new(("drag_rot", entity));
                                                if drag_started {
                                                    if let Ok(old) = world.get::<&ae_core::ecs::Rotation>(entity) {
                                                        ctx.data_mut(|d| d.insert_temp(rot_id, [old.x, old.y, old.z, old.w]));
                                                    }
                                                }
                                                if changed {
                                                    let q = cgmath::Quaternion::from(cgmath::Euler {
                                                        x: cgmath::Deg(inspector_euler[0]),
                                                        y: cgmath::Deg(inspector_euler[1]),
                                                        z: cgmath::Deg(inspector_euler[2]),
                                                    });
                                                    let new_rot = ae_core::ecs::Rotation { x: q.v.x, y: q.v.y, z: q.v.z, w: q.s };
                                                    if is_dragging {
                                                        ui_actions.push(EngineUiAction::LiveUpdateRotation(entity, new_rot));
                                                    } else {
                                                        let old_rot = ctx.data(|d| d.get_temp::<[f32; 4]>(rot_id))
                                                            .map(|arr| ae_core::ecs::Rotation { x: arr[0], y: arr[1], z: arr[2], w: arr[3] })
                                                            .unwrap_or_else(|| {
                                                                if let Ok(r) = world.get::<&ae_core::ecs::Rotation>(entity) { *r } else { new_rot }
                                                            });
                                                        ui_actions.push(EngineUiAction::ModifyRotation(entity, old_rot, new_rot));
                                                    }
                                                }
                                                if drag_stopped {
                                                    let q = cgmath::Quaternion::from(cgmath::Euler {
                                                        x: cgmath::Deg(inspector_euler[0]),
                                                        y: cgmath::Deg(inspector_euler[1]),
                                                        z: cgmath::Deg(inspector_euler[2]),
                                                    });
                                                    let new_rot = ae_core::ecs::Rotation { x: q.v.x, y: q.v.y, z: q.v.z, w: q.s };
                                                    if let Some(arr) = ctx.data(|d| d.get_temp::<[f32; 4]>(rot_id)) {
                                                        let old_rot = ae_core::ecs::Rotation { x: arr[0], y: arr[1], z: arr[2], w: arr[3] };
                                                        if old_rot.x != new_rot.x || old_rot.y != new_rot.y || old_rot.z != new_rot.z || old_rot.w != new_rot.w {
                                                            ui_actions.push(EngineUiAction::ModifyRotation(entity, old_rot, new_rot));
                                                        }
                                                    }
                                                    ctx.data_mut(|d| d.remove::<[f32; 4]>(rot_id));
                                                }

                                                // --- SCALE ---
                                                let (mut sx, mut sy, mut sz) = {
                                                    if let Ok(s) = world.get::<&ae_core::ecs::Scale>(entity) {
                                                        (s.x, s.y, s.z)
                                                    } else {
                                                        (1.0, 1.0, 1.0)
                                                    }
                                                };
                                                let mut scale_arr = [sx, sy, sz];
                                                let (changed, drag_started, drag_stopped, is_dragging, reset_clicked) =
                                                    draw_vec3_row(ui, "Scale", &mut scale_arr, 0.01, 3, 1.0);
                                                sx = scale_arr[0]; sy = scale_arr[1]; sz = scale_arr[2];

                                                if reset_clicked {
                                                    for &ent in &editor_state.selected_entities {
                                                        if let Ok(old) = world.get::<&ae_core::ecs::Scale>(ent) {
                                                            ui_actions.push(EngineUiAction::ModifyScale(ent, *old, ae_core::ecs::Scale { x: 1.0, y: 1.0, z: 1.0 }));
                                                        }
                                                    }
                                                }
                                                let scale_id = egui::Id::new(("drag_scale", entity));
                                                if drag_started {
                                                    if let Ok(old) = world.get::<&ae_core::ecs::Scale>(entity) {
                                                        ctx.data_mut(|d| d.insert_temp(scale_id, [old.x, old.y, old.z]));
                                                    }
                                                }
                                                if changed {
                                                    let new_s = ae_core::ecs::Scale { x: sx, y: sy, z: sz };
                                                    if is_dragging {
                                                        ui_actions.push(EngineUiAction::LiveUpdateScale(entity, new_s));
                                                    } else {
                                                        let old_scale = ctx.data(|d| d.get_temp::<[f32; 3]>(scale_id))
                                                            .map(|arr| ae_core::ecs::Scale { x: arr[0], y: arr[1], z: arr[2] })
                                                            .unwrap_or_else(|| {
                                                                if let Ok(s) = world.get::<&ae_core::ecs::Scale>(entity) { *s } else { new_s }
                                                            });
                                                        ui_actions.push(EngineUiAction::ModifyScale(entity, old_scale, new_s));
                                                    }
                                                }
                                                if drag_stopped {
                                                    let new_s = ae_core::ecs::Scale { x: sx, y: sy, z: sz };
                                                    if let Some(arr) = ctx.data(|d| d.get_temp::<[f32; 3]>(scale_id)) {
                                                        let old_scale = ae_core::ecs::Scale { x: arr[0], y: arr[1], z: arr[2] };
                                                        if old_scale.x != new_s.x || old_scale.y != new_s.y || old_scale.z != new_s.z {
                                                            ui_actions.push(EngineUiAction::ModifyScale(entity, old_scale, new_s));
                                                        }
                                                    }
                                                    ctx.data_mut(|d| d.remove::<[f32; 3]>(scale_id));
                                                }
                                            });
                                    });
                                });
                            }

                            // --- APPEARANCE (Color Picker) ---
                            ui.group(|ui| {
                                ui.set_width(ui.available_width());
                                ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 8.0);
                                ui.label(egui::RichText::new("Appearance").strong().color(egui::Color32::WHITE));
                                let mut color = if let Ok(c) = world.get::<&ae_core::ecs::Color>(entity) {
                                    [c.r, c.g, c.b, c.a]
                                } else {
                                    [0.3, 0.3, 0.3, 1.0] // Default Dark Gray
                                };

                                ui.horizontal(|ui| {
                                    ui.label("Object Color:");
                                    let res = ui.color_edit_button_rgba_unmultiplied(&mut color);

                                    // Hex Input Field
                                    ui.add_space(5.0);
                                    ui.label("Hex:");
                                    let hex_res = ui.add(egui::TextEdit::singleline(inspector_color_hex).desired_width(65.0));

                                    if res.changed() {
                                        // Capture old color for undo
                                        let old_color = if let Ok(c) = world.get::<&ae_core::ecs::Color>(entity) {
                                            Some(*c)
                                        } else {
                                            Some(ae_core::ecs::Color { r: 0.3, g: 0.3, b: 0.3, a: 1.0 })
                                        };
                                        // Update component from color picker
                                        let new_color = ae_core::ecs::Color {
                                            r: color[0], g: color[1], b: color[2], a: color[3]
                                        };
                                        ui_actions.push(EngineUiAction::ModifyColor(entity, old_color.unwrap_or(ae_core::ecs::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }), new_color));

                                        // Sync Hex String
                                        *inspector_color_hex = format!("#{:02x}{:02x}{:02x}",
                                            (color[0] * 255.0) as u8,
                                            (color[1] * 255.0) as u8,
                                            (color[2] * 255.0) as u8);
                                    } else if hex_res.changed() {
                                        // Manual Hex Parsing (RRGGBB format)
                                        let clean_hex = inspector_color_hex.trim_start_matches('#');
                                        if clean_hex.len() == 6 {
                                            if let Ok(rgb) = u32::from_str_radix(clean_hex, 16) {
                                                let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
                                                let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
                                                let b = (rgb & 0xFF) as f32 / 255.0;

                                                let old_color = if let Ok(c) = world.get::<&ae_core::ecs::Color>(entity) {
                                                    *c
                                                } else { ae_core::ecs::Color { r: 0.3, g: 0.3, b: 0.3, a: 1.0 } };
                                                let new_color = ae_core::ecs::Color { r, g, b, a: 1.0 };

                                                ui_actions.push(EngineUiAction::ModifyColor(entity, old_color, new_color));
                                            }
                                        }
                                    }
                                });

                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label("Add to Palette:");
                                    if ui.button("✚").on_hover_text("Save selected color to palette").clicked() {
                                        if !saved_swatches.contains(&color) && saved_swatches.len() < 22 {
                                            saved_swatches.push(color);
                                        }
                                    }
                                    if ui.button("🗑").on_hover_text("Clear palette").clicked() {
                                        saved_swatches.clear();
                                    }
                                });

                                // --- SWATCH GRID (New: Inside Inspector) ---
                                if !saved_swatches.is_empty() {
                                    ui.add_space(4.0);
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                                        for &swatch in saved_swatches.iter() {
                                            let swatch_size = egui::vec2(14.0, 14.0);
                                            let (rect, res) = ui.allocate_at_least(swatch_size, egui::Sense::click());

                                            // Glow/Hover Effect
                                            let color32 = egui::Color32::from_rgba_unmultiplied(
                                                (swatch[0] * 255.0) as u8,
                                                (swatch[1] * 255.0) as u8,
                                                (swatch[2] * 255.0) as u8,
                                                (swatch[3] * 255.0) as u8,
                                            );

                                            if res.hovered() {
                                                // Glow effect: Expland rect slightly and add white border
                                                let glow_rect = rect.expand(1.5);
                                                ui.painter().rect_filled(glow_rect, 2.0, color32);
                                                ui.painter().rect_stroke(glow_rect, 2.0, egui::Stroke::new(1.5, egui::Color32::WHITE), egui::StrokeKind::Outside);
                                            } else {
                                                ui.painter().rect_filled(rect, 2.0, color32);
                                            }

                                            if res.clicked() {
                                                let old_color = if let Ok(c) = world.get::<&ae_core::ecs::Color>(entity) {
                                                    *c
                                                } else { ae_core::ecs::Color { r: 0.3, g: 0.3, b: 0.3, a: 1.0 } };
                                                let new_color = ae_core::ecs::Color { r: swatch[0], g: swatch[1], b: swatch[2], a: swatch[3] };

                                                ui_actions.push(EngineUiAction::ModifyColor(entity, old_color, new_color));
                                            }
                                        }
                                    });
                                }
                            });

                            // --- TEXTURE & MATERIAL SECTION ---
                            Self::draw_texture_section(ui, world, entity, textures, ui_actions);

                            if let Ok(light) = world.get::<&ae_core::ecs::Light>(entity) {
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label("Lighting Settings");
                                    ui.horizontal(|ui| {
                                        ui.label("Color:");
                                        let mut edit_color = light.color.clone();
                                        let res = ui.color_edit_button_rgb(&mut edit_color);
                                        if res.changed() {
                                            ui_actions.push(EngineUiAction::ModifyLightColor(entity, light.color, edit_color));
                                        }
                                    });
                                });
                            }

                            // --- RIGIDBODY SECTION (Dynamic) ---
                            Self::draw_rigidbody_section(ui, world, entity, ui_actions);

                            // --- COLLIDER SECTION (Dynamic) ---
                            Self::draw_collider_section(ui, world, entity, ui_actions);

                            // --- CHARACTER CONTROLLER SECTION ---
                            Self::draw_character_controller_section(ui, world, entity, ui_actions);

                            // --- AUDIO SOURCE SECTION ---
                            if let Ok(source) = world.get::<&ae_audio::AudioSource>(entity) {
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 8.0);
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("🔊 AudioSource").strong().color(egui::Color32::WHITE));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button("🗑").on_hover_text("Remove AudioSource").clicked() {
                                                ui_actions.push(EngineUiAction::RemoveAudioSource(entity));
                                            }
                                        });
                                    });
                                    ui.separator();

                                    let mut updated = (*source).clone();
                                    let mut changed = false;

                                    ui.horizontal(|ui| {
                                        ui.label("Sound Path:");
                                        if ui.text_edit_singleline(&mut updated.sound_path).changed() {
                                            changed = true;
                                        }
                                        if ui.button("📁").on_hover_text("Pick sound file (.wav, .ogg, .mp3)").clicked() {
                                            if let Some(path) = rfd::FileDialog::new()
                                                .add_filter("Audio File", &["wav", "ogg", "mp3", "flac"])
                                                .pick_file()
                                            {
                                                updated.sound_path = path.to_string_lossy().to_string();
                                                changed = true;
                                            }
                                        }
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Volume:");
                                        if ui.add(egui::Slider::new(&mut updated.volume, 0.0..=2.0).text("Gain")).changed() {
                                            changed = true;
                                        }
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Pitch:");
                                        if ui.add(egui::Slider::new(&mut updated.pitch, 0.1..=3.0).text("Speed")).changed() {
                                            changed = true;
                                        }
                                    });

                                    if ui.checkbox(&mut updated.is_spatial, "3D Spatial Audio").on_hover_text("Enable 3D distance falloff & stereo panning").changed() {
                                        changed = true;
                                    }
                                    if ui.checkbox(&mut updated.looping, "Loop Sound").on_hover_text("Repeat sound when reaching EOF").changed() {
                                        changed = true;
                                    }
                                    if ui.checkbox(&mut updated.play_on_start, "Play on Start").on_hover_text("Auto-start sound playback when spawned").changed() {
                                        changed = true;
                                    }

                                    if updated.is_spatial {
                                        ui.horizontal(|ui| {
                                            ui.label("Min Dist:");
                                            if ui.add(egui::DragValue::new(&mut updated.min_distance).speed(0.5).range(0.1..=100.0)).changed() {
                                                changed = true;
                                            }
                                            ui.label("Max Dist:");
                                            if ui.add(egui::DragValue::new(&mut updated.max_distance).speed(1.0).range(1.0..=1000.0)).changed() {
                                                changed = true;
                                            }
                                        });
                                    }

                                    if changed {
                                        ui_actions.push(EngineUiAction::ModifyAudioSource(entity, updated));
                                    }
                                });
                            }

                            // --- AUDIO LISTENER SECTION ---
                            if world.get::<&ae_audio::AudioListener>(entity).is_ok() {
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("👂 AudioListener").strong().color(egui::Color32::WHITE));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button("🗑").on_hover_text("Remove AudioListener").clicked() {
                                                ui_actions.push(EngineUiAction::RemoveAudioListener(entity));
                                            }
                                        });
                                    });
                                    ui.label("Active 3D spatial ear position.");
                                });
                            }

                            // --- PLAYER TAG SECTION ---
                            Self::draw_player_tag_section(ui, world, entity, ui_actions);

                            // --- HIERARCHY / PARENTING SECTION ---
                            Self::draw_parenting_section(ui, world, entity, ui_actions);

                            // --- LOD GROUP SECTION ---
                            if let Ok(lod) = world.get::<&ae_core::ecs::LodGroup>(entity) {
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 8.0);
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("📊 LOD Group").strong().color(egui::Color32::WHITE));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button("🗑").on_hover_text("Remove LOD Group").clicked() {
                                                ui_actions.push(EngineUiAction::RemoveLodGroup(entity));
                                            }
                                        });
                                    });
                                    ui.separator();

                                    let cam_pos = cgmath::Vector3::new(camera.position.x, camera.position.y, camera.position.z);
                                    let p_world = if let Ok(gt) = world.get::<&ae_core::ecs::GlobalTransform>(entity) {
                                        cgmath::Vector3::new(gt.0.w.x, gt.0.w.y, gt.0.w.z)
                                    } else if let Ok(pos) = world.get::<&ae_core::ecs::Position>(entity) {
                                        cgmath::Vector3::new(pos.x, pos.y, pos.z)
                                    } else {
                                        cgmath::Vector3::new(0.0, 0.0, 0.0)
                                    };
                                    let dist = (p_world - cam_pos).magnitude();

                                    let active_lod = if dist < lod.threshold_1 {
                                        "LOD 0 (High Detail)"
                                    } else if dist < lod.threshold_2 {
                                        "LOD 1 (Medium Detail)"
                                    } else {
                                        "LOD 2 (Low Detail)"
                                    };

                                    ui.colored_label(egui::Color32::from_rgb(77, 163, 255), format!("Distance: {:.1} units", dist));
                                    ui.colored_label(egui::Color32::GREEN, format!("Active Mesh: {}", active_lod));

                                    ui.separator();

                                    let mut get_model_name = |handle: ae_renderer::asset::AssetHandle| -> String {
                                        if let Some(asset) = models.get(handle) {
                                            std::path::Path::new(&asset.source_path)
                                                .file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or(&asset.source_path)
                                                .to_string()
                                        } else {
                                            format!("Unknown Model ({:?})", handle)
                                        }
                                    };

                                    // LOD 0
                                    ui.horizontal(|ui| {
                                        ui.label("LOD 0 (High):");
                                        let current_name = get_model_name(lod.lod_0);
                                        #[allow(deprecated)]
                                        egui::ComboBox::from_id_salt(egui::Id::new(("lod0_combo", entity)))
                                            .selected_text(current_name)
                                            .show_ui(ui, |ui| {
                                                for (handle, _) in models.iter() {
                                                    let name = get_model_name(handle);
                                                    if ui.selectable_label(handle == lod.lod_0, name).clicked() {
                                                        ui_actions.push(EngineUiAction::ModifyLodModel(entity, 0, Some(handle)));
                                                    }
                                                }
                                            });
                                    });

                                    // LOD 1
                                    ui.horizontal(|ui| {
                                        ui.label("LOD 1 (Med):");
                                        let current_name = lod.lod_1.map(&mut get_model_name).unwrap_or_else(|| "None".to_string());
                                        #[allow(deprecated)]
                                        egui::ComboBox::from_id_salt(egui::Id::new(("lod1_combo", entity)))
                                            .selected_text(current_name)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_label(lod.lod_1.is_none(), "None").clicked() {
                                                    ui_actions.push(EngineUiAction::ModifyLodModel(entity, 1, None));
                                                }
                                                for (handle, _) in models.iter() {
                                                    let name = get_model_name(handle);
                                                    if ui.selectable_label(Some(handle) == lod.lod_1, name).clicked() {
                                                        ui_actions.push(EngineUiAction::ModifyLodModel(entity, 1, Some(handle)));
                                                    }
                                                }
                                            });
                                    });

                                    // LOD 2
                                    ui.horizontal(|ui| {
                                        ui.label("LOD 2 (Low):");
                                        let current_name = lod.lod_2.map(&mut get_model_name).unwrap_or_else(|| "None".to_string());
                                        #[allow(deprecated)]
                                        egui::ComboBox::from_id_salt(egui::Id::new(("lod2_combo", entity)))
                                            .selected_text(current_name)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_label(lod.lod_2.is_none(), "None").clicked() {
                                                    ui_actions.push(EngineUiAction::ModifyLodModel(entity, 2, None));
                                                }
                                                for (handle, _) in models.iter() {
                                                    let name = get_model_name(handle);
                                                    if ui.selectable_label(Some(handle) == lod.lod_2, name).clicked() {
                                                        ui_actions.push(EngineUiAction::ModifyLodModel(entity, 2, Some(handle)));
                                                    }
                                                }
                                            });
                                    });

                                    ui.separator();

                                    let mut t1 = lod.threshold_1;
                                    let mut t2 = lod.threshold_2;
                                    ui.horizontal(|ui| {
                                        ui.label("LOD 0->1 Dist:");
                                        if ui.add(egui::DragValue::new(&mut t1).speed(0.5).range(0.1..=t2)).changed() {
                                            ui_actions.push(EngineUiAction::ModifyLodThresholds(entity, t1, t2));
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("LOD 1->2 Dist:");
                                        if ui.add(egui::DragValue::new(&mut t2).speed(0.5).range(t1..=2000.0)).changed() {
                                            ui_actions.push(EngineUiAction::ModifyLodThresholds(entity, t1, t2));
                                        }
                                    });
                                });
                            }

                            // --- BOTTOM ACTION BUTTONS (ADD COMPONENT & SAVE PREFAB) ---
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                Self::draw_add_component_button(ui, world, entity, ui_actions);
                                if ui.button("💾 Save as Prefab").on_hover_text("Save selected entity and its components as a reusable .aeprefab asset").clicked() {
                                    let ent_name = world.get::<&ae_core::ecs::Name>(entity).map(|n| n.0.clone()).unwrap_or_else(|_| "Entity".to_string());
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Aeon Prefab", &["aeprefab"])
                                        .set_file_name(format!("{}.aeprefab", ent_name))
                                        .save_file()
                                    {
                                        ui_actions.push(EngineUiAction::SaveEntityAsPrefab(entity, path));
                                    }
                                }
                            });
                        } else {
                            *selected_entity = None;
                        }
                    } else {
                        ui.label("No object selected. Select an object from the list on the left.");
                    }
                    });
                });
            });
        Some(response.response.rect)
    }

    /// Renders the Texture & Material inspector panel section.
    /// Shows active sprite/texture reference, handle metadata, and provides interactive buttons
    /// for picking a texture file from disk or removing/assigning a texture.
    pub fn draw_texture_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 8.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("🖼️ Texture & Material")
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if world.get::<&ae_core::ecs::SpriteId>(entity).is_ok() {
                        if ui
                            .button("🗑")
                            .on_hover_text("Remove Texture from Entity")
                            .clicked()
                        {
                            ui_actions.push(EngineUiAction::RemoveTextureFromEntity(entity));
                        }
                    }
                });
            });
            ui.separator();

            if let Ok(sprite_ref) = world.get::<&ae_core::ecs::SpriteId>(entity) {
                if let Some(asset) = textures.get(sprite_ref.0) {
                    let file_name = std::path::Path::new(&asset.source_path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| asset.source_path.clone());

                    ui.horizontal(|ui| {
                        ui.label("Path:");
                        ui.label(
                            egui::RichText::new(&file_name)
                                .color(egui::Color32::LIGHT_BLUE)
                                .strong(),
                        )
                        .on_hover_text(&asset.source_path);
                    });

                    let max_dim = asset.width.max(asset.height);
                    let mip_levels = if max_dim > 0 { max_dim.ilog2() + 1 } else { 1 };
                    ui.horizontal(|ui| {
                        ui.label("Info:");
                        ui.label(
                            egui::RichText::new(format!(
                                "{} x {} px | sRGB | Mipmaps: {}",
                                asset.width, asset.height, mip_levels
                            ))
                            .color(egui::Color32::GREEN)
                            .strong(),
                        );
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Status:");
                        ui.label(
                            egui::RichText::new("Texture Attached")
                                .color(egui::Color32::GREEN)
                                .strong(),
                        );
                    });
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui
                        .button("📁 Change Texture")
                        .on_hover_text("Browse disk for .png, .jpg, .tga file to change texture")
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Texture Image", &["png", "jpg", "jpeg", "tga", "bmp"])
                            .pick_file()
                        {
                            ui_actions.push(EngineUiAction::AssignTextureToEntity(
                                entity,
                                path.to_string_lossy().to_string(),
                            ));
                        }
                    }
                });

                // --- TILING & SAMPLER CONTROLS ---
                ui.separator();
                ui.label(
                    egui::RichText::new("🧱 Tiling & Sampler Settings")
                        .strong()
                        .color(egui::Color32::WHITE),
                );

                ui.horizontal(|ui| {
                    ui.label("Wrap U:");
                    ui.label(
                        egui::RichText::new("Repeat")
                            .color(egui::Color32::LIGHT_GREEN)
                            .strong(),
                    )
                    .on_hover_text("Horizontal texture coordinate repeating");
                });

                ui.horizontal(|ui| {
                    ui.label("Wrap V:");
                    ui.label(
                        egui::RichText::new("Repeat")
                            .color(egui::Color32::LIGHT_GREEN)
                            .strong(),
                    )
                    .on_hover_text("Vertical texture coordinate repeating");
                });

                ui.horizontal(|ui| {
                    ui.label("Anisotropy:");
                    ui.label(
                        egui::RichText::new("16x")
                            .color(egui::Color32::GOLD)
                            .strong(),
                    )
                    .on_hover_text("16x Anisotropic filtering for oblique surface clarity");
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("No Texture Assigned");
                    if ui
                        .button("➕ Add Texture")
                        .on_hover_text("Browse disk for .png, .jpg file to assign texture")
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Texture Image", &["png", "jpg", "jpeg", "tga", "bmp"])
                            .pick_file()
                        {
                            ui_actions.push(EngineUiAction::AssignTextureToEntity(
                                entity,
                                path.to_string_lossy().to_string(),
                            ));
                        }
                    }
                });
            }
        });
    }
}