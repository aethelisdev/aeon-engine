// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Action Dispatcher
//!
//! Processes pending inspector actions dispatched by interactive UI components,
//! executing mutations on the ECS world and issuing Undo/Redo actions.

use crate::ui::iris_bridge::inspector::InspectorAction;
use crate::ui::iris_bridge::inspector::color_picker_popup::handle_color_edit_action;
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::state::EngineUi;

impl EngineUi {
    /// Dispatches all pending Inspector panel actions to ECS entities and UI state.
    pub fn process_inspector_actions(
        &mut self,
        world: &hecs::World,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        for action in self.iris_overlay.take_inspector_actions() {
            match action {
                InspectorAction::RenameEntity(entity, name) => {
                    let old_name = world
                        .get::<&ae_core::ecs::Name>(entity)
                        .map(|n| n.0.clone())
                        .unwrap_or_default();
                    ui_actions.push(EngineUiAction::ModifyName(entity, old_name, name));
                }
                InspectorAction::ResetTransform(entity, axis) => {
                    self.handle_reset_transform(world, ui_actions, entity, axis);
                }
                InspectorAction::StartColorEdit(..)
                | InspectorAction::LiveSetObjectColor(..)
                | InspectorAction::CommitColorEdit(..) => {
                    handle_color_edit_action(
                        &mut self.iris_overlay.inspector.color_edit_start,
                        &mut self.iris_overlay.inspector.hsv,
                        &mut self.inspector_color_hex,
                        world,
                        ui_actions,
                        action,
                    );
                }
                InspectorAction::SetObjectColor(entity, col) => {
                    self.iris_overlay.inspector.color_edit_start = None;
                    self.handle_set_object_color(world, ui_actions, entity, col);
                }
                InspectorAction::AddColorToPalette(col) => {
                    if let Some(entity) = self.selected_entity {
                        let arr = if col == irisui::prelude::Color::TRANSPARENT {
                            world
                                .get::<&ae_core::ecs::Color>(entity)
                                .map(|c| [c.r, c.g, c.b, c.a])
                                .unwrap_or([0.60, 0.75, 0.95, 1.0])
                        } else {
                            [col.r, col.g, col.b, col.a]
                        };
                        if !self.saved_swatches.contains(&arr) && self.saved_swatches.len() < 28 {
                            self.saved_swatches.push(arr);
                        }
                    }
                }
                InspectorAction::ClearCustomPalette => {
                    self.saved_swatches.clear();
                }
                InspectorAction::RemoveColorFromPalette(idx) => {
                    if idx < self.saved_swatches.len() {
                        self.saved_swatches.remove(idx);
                    }
                }
                InspectorAction::RemoveComponent(entity, comp_name) => {
                    ui_actions.push(EngineUiAction::RemoveComponent(entity, comp_name));
                }
                InspectorAction::AddComponent(entity, comp_name) => {
                    ui_actions.push(EngineUiAction::AddComponent(entity, comp_name));
                }
                InspectorAction::SaveAsPrefab(entity) => {
                    let raw_name = world
                        .get::<&ae_core::ecs::Name>(entity)
                        .map(|n| n.0.clone())
                        .unwrap_or_else(|_| "prefab".to_string());
                    let clean_name = raw_name
                        .trim()
                        .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
                    let safe_name = if clean_name.is_empty() {
                        "prefab".to_string()
                    } else {
                        clean_name
                    };
                    ui_actions.push(EngineUiAction::SaveEntityAsPrefab(
                        entity,
                        std::path::PathBuf::from(format!(
                            "assets/prefabs/{}.prefab.json",
                            safe_name
                        )),
                    ));
                }
                InspectorAction::StartComponentEdit(entity, comp_name, explicit_bytes) => {
                    if self.iris_overlay.inspector.edit_start_snapshot.is_none() {
                        let old_bytes = explicit_bytes.or_else(|| {
                            let registry = ae_core::registry::ComponentRegistry::global();
                            registry
                                .get_by_name(comp_name)
                                .and_then(|h| h.capture(world, entity))
                        });
                        if let Some(bytes) = old_bytes {
                            self.iris_overlay.inspector.edit_start_snapshot =
                                Some((entity, comp_name, bytes));
                        }
                    }
                }
                InspectorAction::CommitComponentEdit(entity, comp_name) => {
                    if let Some((snap_entity, snap_comp_name, old_bytes)) =
                        self.iris_overlay.inspector.edit_start_snapshot.take()
                        && snap_entity == entity
                        && snap_comp_name == comp_name
                    {
                        let registry = ae_core::registry::ComponentRegistry::global();
                        if let Some(handler) = registry.get_by_name(comp_name)
                            && let Some(new_bytes) = handler.capture(world, entity)
                            && old_bytes != new_bytes
                        {
                            ui_actions.push(EngineUiAction::CommitComponentModify(
                                entity, comp_name, old_bytes, new_bytes,
                            ));
                        }
                    }
                }
                InspectorAction::CommitComponentModify(entity, comp_name, old_bytes, new_bytes) => {
                    self.iris_overlay.inspector.edit_start_snapshot = None;
                    if old_bytes != new_bytes {
                        ui_actions.push(EngineUiAction::CommitComponentModify(
                            entity, comp_name, old_bytes, new_bytes,
                        ));
                    }
                }
                InspectorAction::SyncInspectorEuler(rx, ry, rz) => {
                    self.inspector_euler = [rx, ry, rz];
                }
                InspectorAction::ResetPhysMatPreset(entity) => {
                    let comp_name = "PhysicsMaterial";
                    let registry = ae_core::registry::ComponentRegistry::global();
                    let old_bytes = registry
                        .get_by_name(comp_name)
                        .and_then(|h| h.capture(world, entity));
                    let surf = world
                        .get::<&ae_core::ecs::PhysicsMaterial>(entity)
                        .map(|m| m.surface_type)
                        .unwrap_or(ae_core::ecs::SurfaceType::Default);
                    if let Ok(mut m) = world.get::<&mut ae_core::ecs::PhysicsMaterial>(entity) {
                        *m = ae_core::ecs::PhysicsMaterial::from_preset(surf);
                    }
                    let new_bytes = registry
                        .get_by_name(comp_name)
                        .and_then(|h| h.capture(world, entity));
                    if let (Some(old), Some(new)) = (old_bytes, new_bytes)
                        && old != new
                    {
                        ui_actions.push(EngineUiAction::CommitComponentModify(
                            entity, comp_name, old, new,
                        ));
                    }
                }
                InspectorAction::PickAudioFile(entity) => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Audio Files", &["wav", "ogg", "mp3", "flac"])
                        .pick_file()
                    {
                        let path_str = path.to_string_lossy().to_string();
                        if let Ok(mut a) = world.get::<&mut ae_audio::AudioSource>(entity) {
                            a.sound_path = path_str;
                            a.is_playing = true;
                        }
                    }
                }
                InspectorAction::ToggleAudioPlayback(entity) => {
                    if let Ok(mut a) = world.get::<&mut ae_audio::AudioSource>(entity) {
                        a.is_playing = !a.is_playing;
                    }
                }
                InspectorAction::Unparent(entity) => {
                    ui_actions.push(EngineUiAction::UnparentEntity(entity));
                }
                InspectorAction::OpenAddComponentMenu(_) => {
                    self.iris_overlay.inspector.is_add_menu_open = true;
                    self.iris_overlay.inspector.active_submenu = None;
                    self.iris_overlay.notifier.tag_all();
                    self.iris_overlay.chrome.needs_layout_rebuild = true;
                }
                InspectorAction::CloseAddComponentMenu => {
                    self.iris_overlay.inspector.is_add_menu_open = false;
                    self.iris_overlay.inspector.active_submenu = None;
                    self.iris_overlay.notifier.tag_all();
                    self.iris_overlay.chrome.needs_layout_rebuild = true;
                }
                InspectorAction::OpenAddSubmenu(category) => {
                    self.iris_overlay.inspector.active_submenu = Some(category);
                    self.iris_overlay.notifier.tag_all();
                    self.iris_overlay.chrome.needs_layout_rebuild = true;
                }
                InspectorAction::ToggleColorPicker => {
                    self.iris_overlay.inspector.is_color_picker_open =
                        !self.iris_overlay.inspector.is_color_picker_open;
                    self.iris_overlay.notifier.tag_all();
                    self.iris_overlay.chrome.needs_layout_rebuild = true;
                }
                InspectorAction::FocusHexInput => {
                    if let Some(entity) = self.selected_entity {
                        let c = world
                            .get::<&ae_core::ecs::Color>(entity)
                            .map(|col| [col.r, col.g, col.b, col.a])
                            .unwrap_or([0.60, 0.75, 0.95, 1.0]);
                        let hex = format!(
                            "#{:02x}{:02x}{:02x}",
                            (c[0] * 255.0).round() as u8,
                            (c[1] * 255.0).round() as u8,
                            (c[2] * 255.0).round() as u8
                        );
                        self.iris_overlay.inspector.hex_buffer = Some((entity, hex));
                        self.iris_overlay.notifier.tag_all();
                        self.iris_overlay.chrome.needs_layout_rebuild = true;
                    }
                }
                InspectorAction::CloseAddSubmenu => {
                    self.iris_overlay.inspector.active_submenu = None;
                    self.iris_overlay.notifier.tag_all();
                    self.iris_overlay.chrome.needs_layout_rebuild = true;
                }
            }
        }
    }
}