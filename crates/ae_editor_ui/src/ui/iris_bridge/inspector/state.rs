// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector State & Dirty Tracking
//!
//! Encapsulates the persistent interactive state, input edit sessions, and localized
//! retained-mode dirty tracking for the Scene Inspector panel overlay.
//!

use crate::ui::iris_bridge::inspector::types::{
    ComponentCategory, InspectorAction, InspectorColorDragMode, InspectorDropdownId,
    InspectorNumberDragState, InspectorNumberInputSession, InspectorTextInputId,
};
use crate::ui::iris_bridge::types::{OverlayUpdateParams, PanelInteractionState};

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
    /// Active continuous horizontal mouse drag state for Inspector numeric fields.
    pub drag_number: Option<InspectorNumberDragState>,
    /// Active entity component pre-edit snapshot captured when an Inspector edit starts: `(entity, component_name, old_data)`.
    pub edit_start_snapshot: Option<(hecs::Entity, &'static str, Vec<u8>)>,
    /// Pre-edit color snapshot captured when color picker dragging or editing begins: `(entity, start_color)`.
    pub color_edit_start: Option<(hecs::Entity, ae_core::ecs::Color)>,
    /// Live entity rename text buffer if currently focused: `(entity, buffer)`.
    pub rename_buffer: Option<(hecs::Entity, String)>,
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
    /// Last recorded selected entity for detecting Inspector invalidation.
    pub last_selected_entity: Option<hecs::Entity>,
    /// Active ECS entity being inspected in the panel.
    pub inspected_entity: Option<hecs::Entity>,
    /// Current display name of the inspected entity.
    pub inspected_entity_name: String,
    /// Local cache of saved palette colors for swatch interactions.
    pub saved_swatches: Vec<[f32; 4]>,
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
    /// Evaluates whether the Inspector panel requires an in-place repaint.
    ///
    /// Inspects whether the active ECS scene entity selection has changed, or if
    /// an active text input, number drag, color editing session, or flyout menu is in progress.
    pub fn is_dirty(&self, params: &OverlayUpdateParams<'_>) -> bool {
        self.last_selected_entity != params.scene.selected_entity
            || self.active_number_input.is_some()
            || self.active_text_input.is_some()
            || self.drag_number.is_some()
            || self.is_color_picker_open
            || self.is_add_menu_open
    }

    /// Synchronizes internal cached snapshot values against active frame parameters.
    pub fn sync_dirty(&mut self, params: &OverlayUpdateParams<'_>) {
        self.last_selected_entity = params.scene.selected_entity;
    }

    /// Evaluates `is_dirty` and automatically updates snapshot caches if dirty.
    ///
    /// Returns `true` if the panel state changed and requires redraw tagging.
    pub fn check_and_sync_dirty(&mut self, params: &OverlayUpdateParams<'_>) -> bool {
        let dirty = self.is_dirty(params);
        if dirty {
            self.sync_dirty(params);
        }
        dirty
    }
}