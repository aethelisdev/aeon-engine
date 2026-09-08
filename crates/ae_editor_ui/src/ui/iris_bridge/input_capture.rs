// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Explicit keyboard focus and popup capture queries for native window event routing.

use super::IrisEditorOverlay;

impl IrisEditorOverlay {
    /// Reports modal or popup capture independently of ordinary panel hover.
    /// Used by docking and scene picking so a menu cannot activate an obscured control.
    pub fn has_popup_or_modal(&self) -> bool {
        self.active_menu.is_some()
            || self.about_targets.is_some()
            || self.preferences_targets.is_some()
            || self.delete_targets.is_some()
            || self.new_folder_targets.is_some()
            || self.rename_targets.is_some()
            || self.loading_targets.is_some()
            || self.assets_preview_modal.is_some()
            || self.assets_context_menu.is_some()
            || self.viewport_hud_dropdown.is_some()
            || self.hierarchy_is_add_menu_open
            || self.hierarchy_active_context_menu.is_some()
            || self.inspector_is_add_menu_open
            || self.inspector_active_dropdown.is_some()
            || self.inspector_is_color_picker_open
            || self.preferences_dropdown.is_some()
    }

    /// Returns true while text entry or a modal owns keyboard input, suppressing scene shortcuts.
    pub fn wants_keyboard_input(&self) -> bool {
        self.has_popup_or_modal()
            || self.hierarchy_is_search_focused
            || self.console_is_search_focused
            || self.assets_is_search_focused
            || self.viewport_is_search_focused
            || self.inspector_active_number_input.is_some()
            || self.inspector_rename_buffer.is_some()
            || self.inspector_hex_buffer.is_some()
            || self.active_number_input.is_some()
    }
}