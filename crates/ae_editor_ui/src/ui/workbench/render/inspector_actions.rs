// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Panel Action Handlers
//!
//! Dispatches ECS component mutations, numeric input edits, color adjustments,
//! transform resets, and combobox selections triggered by the Iris UI Inspector panel.

pub mod dropdowns;
pub mod numbers;
pub mod transform;

use crate::ui::iris_bridge::inspector::color_picker_popup::handle_color_edit_action;
use crate::ui::iris_bridge::inspector::{ComponentCheckboxId, InspectorAction};
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::state::EngineUi;

pub(crate) use dropdowns::handle_select_dropdown;
pub(crate) use numbers::handle_set_number_value;

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
                    ui_actions.push(EngineUiAction::SaveEntityAsPrefab(
                        entity,
                        std::path::PathBuf::from("assets/prefabs/prefab.json"),
                    ));
                }
                InspectorAction::StartNumberEdit(entity, num_id) => {
                    let comp_name = num_id.component_name();
                    let registry = ae_core::registry::ComponentRegistry::global();
                    if let Some(handler) = registry.get_by_name(comp_name)
                        && let Some(old_bytes) = handler.capture(world, entity)
                    {
                        self.iris_overlay.inspector.edit_start_snapshot =
                            Some((entity, comp_name, old_bytes));
                    }
                }
                InspectorAction::SetNumberValue(entity, num_id, val) => {
                    if self.iris_overlay.inspector.edit_start_snapshot.is_none() {
                        let comp_name = num_id.component_name();
                        let registry = ae_core::registry::ComponentRegistry::global();
                        if let Some(handler) = registry.get_by_name(comp_name)
                            && let Some(old_bytes) = handler.capture(world, entity)
                        {
                            self.iris_overlay.inspector.edit_start_snapshot =
                                Some((entity, comp_name, old_bytes));
                        }
                    }
                    handle_set_number_value(world, entity, num_id, val, &mut self.inspector_euler);
                }
                InspectorAction::CommitNumberEdit(entity, num_id) => {
                    let comp_name = num_id.component_name();
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
                InspectorAction::SetTextValue(entity, text_id, val) => {
                    let comp_name = text_id.component_name();
                    let registry = ae_core::registry::ComponentRegistry::global();
                    let old_bytes = registry
                        .get_by_name(comp_name)
                        .and_then(|h| h.capture(world, entity));
                    match text_id {
                        crate::ui::iris_bridge::inspector::InspectorTextInputId::UiTextContent => {
                            if let Ok(mut t) = world.get::<&mut ae_core::ecs::UiText>(entity) {
                                t.text = val;
                            }
                        }
                        crate::ui::iris_bridge::inspector::InspectorTextInputId::UiTextInputPlaceholder => {
                            if let Ok(mut t) = world.get::<&mut ae_core::ecs::UiTextInput>(entity) {
                                t.placeholder = val;
                            }
                        }
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
                InspectorAction::SelectDropdown(entity, dd_id, opt_idx) => {
                    let comp_name = dd_id.component_name();
                    let registry = ae_core::registry::ComponentRegistry::global();
                    let old_bytes = registry
                        .get_by_name(comp_name)
                        .and_then(|h| h.capture(world, entity));
                    handle_select_dropdown(world, entity, dd_id, opt_idx);
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
                InspectorAction::ToggleCheckbox(entity, cb_id) => {
                    let comp_name = cb_id.component_name();
                    let registry = ae_core::registry::ComponentRegistry::global();
                    let old_bytes = registry
                        .get_by_name(comp_name)
                        .and_then(|h| h.capture(world, entity));
                    if let ComponentCheckboxId::ColliderIsSensor = cb_id
                        && let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity)
                    {
                        c.is_sensor = !c.is_sensor;
                    } else if let ComponentCheckboxId::AudioLoop = cb_id
                        && let Ok(mut a) = world.get::<&mut ae_audio::AudioSource>(entity)
                    {
                        a.looping = !a.looping;
                    } else if let ComponentCheckboxId::AudioSpatial = cb_id
                        && let Ok(mut a) = world.get::<&mut ae_audio::AudioSource>(entity)
                    {
                        a.is_spatial = !a.is_spatial;
                    } else if let ComponentCheckboxId::AudioPlayOnStart = cb_id
                        && let Ok(mut a) = world.get::<&mut ae_audio::AudioSource>(entity)
                    {
                        a.play_on_start = !a.play_on_start;
                    } else if let ComponentCheckboxId::UiVisible = cb_id
                        && let Ok(mut u) = world.get::<&mut ae_core::ecs::UiElement>(entity)
                    {
                        u.visible = !u.visible;
                    } else if let ComponentCheckboxId::UiInteractable = cb_id
                        && let Ok(mut b) = world.get::<&mut ae_core::ecs::UiButton>(entity)
                    {
                        b.is_enabled = !b.is_enabled;
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
                _ => {}
            }
        }
    }
}