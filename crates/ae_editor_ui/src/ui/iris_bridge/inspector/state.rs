// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector State & Edit Sessions
//!
//! Encapsulates the persistent interactive state, entity rename edit sessions,
//! active popups, and authoritative color/rotation caches for the Scene Inspector panel overlay.

use crate::ui::iris_bridge::inspector::types::ComponentCategory;

/// Persistent interactive state for the Scene Inspector panel overlay.
#[derive(Debug, Default)]
pub struct InspectorPanelState {
    /// Whether `➕ Add Component` menu is open in Inspector.
    pub is_add_menu_open: bool,
    /// Currently open category submenu in Add Component menu.
    pub active_submenu: Option<ComponentCategory>,
    /// Live entity rename TextInputState session if currently focused.
    pub rename_input: Option<(hecs::Entity, irisui::prelude::TextInputState)>,
    /// Whether the active entity rename buffer has its full text selected in blue.
    pub rename_is_all_selected: bool,
    /// Live HSV color cache: `[hue (0..360), saturation (0..1), value (0..1)]`.
    pub hsv: [f32; 3],
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
        if let Some((ent, input)) = self.rename_input.take()
            && Some(ent) == selected_entity
        {
            self.rename_input = Some((ent, input));
        }
    }
}