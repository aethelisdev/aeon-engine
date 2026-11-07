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
        ui_actions.append(&mut self.iris_overlay.pending_actions);
        for action in self.iris_overlay.take_inspector_actions() {
            match action {
                InspectorAction::RenameEntity(entity, name) => {
                    let old_name = world
                        .get::<&ae_core::ecs::Name>(entity)
                        .map(|n| n.0.clone())
                        .unwrap_or_default();
                    ui_actions.push(EngineUiAction::ModifyName(entity, old_name, name));
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
                    let old_col = world
                        .get::<&ae_core::ecs::Color>(entity)
                        .map(|c| *c)
                        .unwrap_or(ae_core::ecs::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        });
                    let new_col = ae_core::ecs::Color {
                        r: col.r,
                        g: col.g,
                        b: col.b,
                        a: col.a,
                    };
                    let r = (col.r.clamp(0.0, 1.0) * 255.0) as u8;
                    let g = (col.g.clamp(0.0, 1.0) * 255.0) as u8;
                    let b = (col.b.clamp(0.0, 1.0) * 255.0) as u8;
                    self.inspector_color_hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
                    let (h, s, v) =
                        irisui::prelude::HsvColorPickerState::rgb_to_hsv(col.r, col.g, col.b);
                    self.iris_overlay.inspector.hsv = [h, s, v];
                    ui_actions.push(EngineUiAction::ModifyColor(entity, old_col, new_col));
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