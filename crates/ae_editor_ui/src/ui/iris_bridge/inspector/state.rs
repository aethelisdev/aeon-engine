// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector State & Dirty Tracking
//!
//! Encapsulates the persistent interactive state, input edit sessions, and localized
//! retained-mode dirty tracking for the Scene Inspector panel overlay.

use crate::ui::iris_bridge::inspector::types::{
    ComponentCategory, InspectorAction, InspectorColorDragMode, InspectorDropdownId,
    InspectorNumberDragState, InspectorNumberInputSession, InspectorTextInputId,
};
use crate::ui::iris_bridge::types::PanelInteractionState;

/// Persistent interactive state for the Scene Inspector panel overlay.
#[derive(Debug, Default)]
pub struct InspectorPanelState {
    /// Common panel interaction state (scroll_y, search, actions).
    pub interactions: PanelInteractionState<(), InspectorAction>,
    /// Whether `➕ Add Component` menu is open in Inspector.
    pub is_add_menu_open: bool,
    /// Currently open category submenu in Add Component menu.
    pub active_submenu: Option<ComponentCategory>,
    /// Currently open dropdown in Inspector.
    pub active_dropdown: Option<InspectorDropdownId>,
    /// Currently active number input editing session in Inspector.
    pub active_number_input: Option<InspectorNumberInputSession>,
    /// Currently active string text input editing state in Inspector: `(entity, id, buffer)`.
    pub active_text_input: Option<(hecs::Entity, InspectorTextInputId, String)>,
    /// Currently active `TextInputState` session for Inspector string text fields.
    pub active_text_input_state: Option<(
        hecs::Entity,
        InspectorTextInputId,
        irisui::prelude::TextInputState,
    )>,
    /// Active continuous horizontal mouse drag state for Inspector numeric fields.
    pub drag_number: Option<InspectorNumberDragState>,
    /// Active entity component pre-edit snapshot captured when an Inspector edit starts: `(entity, component_name, old_data)`.
    pub edit_start_snapshot: Option<(hecs::Entity, &'static str, Vec<u8>)>,
    /// Pre-edit color snapshot captured when color picker dragging or editing begins: `(entity, start_color)`.
    pub color_edit_start: Option<(hecs::Entity, ae_core::ecs::Color)>,
    /// Live entity rename TextInputState session if currently focused.
    pub rename_input: Option<(hecs::Entity, irisui::prelude::TextInputState)>,
    /// Whether the active entity rename buffer has its full text selected in blue.
    pub rename_is_all_selected: bool,
    /// Live HEX color text input editing buffer if currently focused: `(entity, buffer)`.
    pub hex_buffer: Option<(hecs::Entity, String)>,
    /// Live HSV color cache: `[hue (0..360), saturation (0..1), value (0..1)]`.
    pub hsv: [f32; 3],
    /// Active mouse dragging mode on the 2D HSV color picker.
    pub color_drag_mode: Option<InspectorColorDragMode>,
    /// Whether the floating Color Picker popup is currently open.
    pub is_color_picker_open: bool,
    /// Active ECS entity being inspected in the panel.
    pub inspected_entity: Option<hecs::Entity>,
    /// Current display name of the inspected entity.
    pub inspected_entity_name: String,
    /// Authoritative inspector Euler angles [rx, ry, rz] in degrees.
    pub inspector_euler: [f32; 3],
    /// Local cache of saved palette colors for swatch interactions.
    pub saved_swatches: Vec<[f32; 4]>,
    /// Last observed transform snapshot of the inspected entity for reactive $O(1)$ ECS change tracking.
    pub last_transform_snapshot: Option<InspectorTransformSnapshot>,
}

/// Compact snapshot of an inspected entity's 3D and 2D transform components for $O(1)$ reactive dirty tracking.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct InspectorTransformSnapshot {
    /// 3D position [x, y, z].
    pub position: Option<[f32; 3]>,
    /// 3D rotation quaternion [x, y, z, w].
    pub rotation: Option<[f32; 4]>,
    /// 3D scale [x, y, z].
    pub scale: Option<[f32; 3]>,
    /// 2D UI element offset and size if inspected entity is a 2D canvas element.
    pub ui_element: Option<([f32; 2], [f32; 2])>,
}

impl InspectorTransformSnapshot {
    /// Captures the transform state of an entity from the ECS world in $O(1)$ time with zero heap allocation.
    pub fn capture(world: &hecs::World, entity: hecs::Entity) -> Self {
        let position = world
            .get::<&ae_core::ecs::Position>(entity)
            .map(|p| [p.x, p.y, p.z])
            .ok();
        let rotation = world
            .get::<&ae_core::ecs::Rotation>(entity)
            .map(|r| [r.x, r.y, r.z, r.w])
            .ok();
        let scale = world
            .get::<&ae_core::ecs::Scale>(entity)
            .map(|s| [s.x, s.y, s.z])
            .ok();
        let ui_element = world
            .get::<&ae_core::ecs::UiElement>(entity)
            .map(|u| (u.offset, u.size))
            .ok();
        Self {
            position,
            rotation,
            scale,
            ui_element,
        }
    }
}

/// Traverses widget parent pointers to test whether `widget_id` is identical to or a descendant of `ancestor_id`.
#[inline]
pub fn is_widget_descendant_of(
    tree: &irisui::core::UiTree,
    mut widget_id: irisui::core::WidgetId,
    ancestor_id: irisui::core::WidgetId,
) -> bool {
    loop {
        if widget_id == ancestor_id {
            return true;
        }
        let Some(node) = tree.get(widget_id) else {
            break;
        };
        let Some(p) = node.parent else { break };
        widget_id = p;
    }
    false
}

impl std::ops::Deref for InspectorPanelState {
    type Target = PanelInteractionState<(), InspectorAction>;
    fn deref(&self) -> &Self::Target {
        &self.interactions
    }
}

impl std::ops::DerefMut for InspectorPanelState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.interactions
    }
}

/// Semantic alias for [`InspectorPanelState`].
pub type InspectorState = InspectorPanelState;

impl InspectorPanelState {
    /// Starts an active entity rename session with full selection highlight.
    pub fn start_rename(&mut self, entity: hecs::Entity, name: &str) {
        let mut input = irisui::prelude::TextInputState::new(name);
        input.select_all();
        self.rename_is_all_selected = true;
        self.rename_input = Some((entity, input));
    }

    /// Clears any active entity rename session.
    pub fn clear_rename(&mut self) {
        self.rename_input = None;
        self.rename_is_all_selected = false;
    }

    /// Commits any pending numeric, text, or rename edit sessions if the selected entity changed.
    pub fn commit_pending_edits_on_selection_change(
        &mut self,
        selected_entity: Option<hecs::Entity>,
    ) {
        if let Some(session) = self.active_number_input.take() {
            if Some(session.entity) != selected_entity {
                if let Ok(v) =
                    super::math_eval::evaluate_inspector_math(&session.buffer, session.initial_val)
                {
                    self.actions.push(InspectorAction::SetNumberValue(
                        session.entity,
                        session.id,
                        v,
                    ));
                    self.actions.push(InspectorAction::CommitNumberEdit(
                        session.entity,
                        session.id,
                    ));
                } else {
                    self.edit_start_snapshot = None;
                }
            } else {
                self.active_number_input = Some(session);
            }
        }
        if let Some((ent, id, buf)) = self.active_text_input.take() {
            if Some(ent) != selected_entity {
                self.actions
                    .push(InspectorAction::SetTextValue(ent, id, buf));
            } else {
                self.active_text_input = Some((ent, id, buf));
            }
        }
        if let Some((ent, input)) = self.rename_input.take() {
            if Some(ent) != selected_entity {
                let final_name = input.text().trim().to_string();
                if !final_name.is_empty() {
                    self.actions
                        .push(InspectorAction::RenameEntity(ent, final_name));
                }
            } else {
                self.rename_input = Some((ent, input));
            }
        }
    }
}