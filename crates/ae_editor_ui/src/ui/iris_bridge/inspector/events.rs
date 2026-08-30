// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Event Handling and Hit-Testing
//!
//! Dispatches mouse clicks, number input edits, color palette picks,
//! and component life-cycle commands with zero allocations.

use super::types::{InspectorAction, InspectorPanelTargets};
use irisui::prelude::*;

/// Handles mouse click input over the Inspector panel.
pub fn handle_inspector_click(
    pos: Point,
    button: MouseButton,
    targets: &InspectorPanelTargets,
    out_actions: &mut Vec<InspectorAction>,
) -> bool {
    if button != MouseButton::Left {
        return false;
    }

    // 1. Check Submenu items inside open Add Component menu
    if let Some(sub_rect) = targets.active_submenu_rect
        && sub_rect.contains_point(pos)
    {
        for &(comp_name, item_rect) in &targets.submenu_components {
            if item_rect.contains_point(pos) {
                out_actions.push(InspectorAction::AddComponent(comp_name));
                out_actions.push(InspectorAction::CloseAddComponentMenu);
                return true;
            }
        }
        return true;
    }

    // 2. Check Categories inside open Add Component menu
    if let Some(add_rect) = targets.active_add_menu_rect
        && add_rect.contains_point(pos)
    {
        for &(cat, item_rect) in &targets.add_menu_categories {
            if item_rect.contains_point(pos) {
                out_actions.push(InspectorAction::OpenAddSubmenu(cat));
                return true;
            }
        }
        return true;
    }

    // 3. Check Active Dropdown items
    if let Some(popup_rect) = targets.active_dropdown_popup_rect
        && popup_rect.contains_point(pos)
    {
        for &(_opt_idx, item_rect) in &targets.dropdown_items {
            if item_rect.contains_point(pos) {
                // Resolved in parent handler
                return true;
            }
        }
        return true;
    }

    // 4. Check Active Color Picker Popup items
    if let Some(picker_rect) = targets.color_picker_popup_rect
        && picker_rect.contains_point(pos)
    {
        if let Some(close_rect) = targets.color_picker_close_btn_rect
            && close_rect.contains_point(pos)
        {
            out_actions.push(InspectorAction::ToggleColorPicker);
            return true;
        }

        // SV Box and Hue Bar drag/clicks are intercepted in parent events handler
        return true;
    }

    // 5. `➕ Add Component` Button
    if targets.add_component_btn_rect.contains_point(pos) {
        if targets.active_add_menu_rect.is_some() {
            out_actions.push(InspectorAction::CloseAddComponentMenu);
        } else {
            out_actions.push(InspectorAction::OpenAddComponentMenu(pos));
        }
        return true;
    }

    // 6. `💾 Save as Prefab` Button
    if targets.save_prefab_btn_rect.contains_point(pos) {
        out_actions.push(InspectorAction::SaveAsPrefab);
        return true;
    }

    // 7. Name Input Box
    if targets.name_input_rect.contains_point(pos) {
        out_actions.push(InspectorAction::FocusRename);
        return true;
    }

    // 8. Transform Reset Buttons
    for &(axis_type, btn_rect) in &targets.transform_reset_btns {
        if btn_rect.contains_point(pos) {
            out_actions.push(InspectorAction::ResetTransform(axis_type));
            return true;
        }
    }

    // 9. Component Trash/Delete Buttons
    for &(comp_name, btn_rect) in &targets.component_delete_btns {
        if btn_rect.contains_point(pos) {
            out_actions.push(InspectorAction::RemoveComponent(comp_name));
            return true;
        }
    }

    // 10. Dropdown Trigger Combo Boxes
    for &(dd_id, combo_rect, _) in &targets.dropdowns {
        if combo_rect.contains_point(pos) {
            out_actions.push(InspectorAction::SelectDropdown(dd_id, 0));
            return true;
        }
    }

    // 11. Component Checkboxes
    for &(cb_id, cb_rect, _) in &targets.checkboxes {
        if cb_rect.contains_point(pos) {
            out_actions.push(InspectorAction::ToggleCheckbox(cb_id));
            return true;
        }
    }

    // 12. Object Color Swatch Box (Opens floating Color Picker Popup)
    if let Some(swatch_rect) = targets.color_swatch_rect
        && swatch_rect.contains_point(pos)
    {
        out_actions.push(InspectorAction::ToggleColorPicker);
        return true;
    }

    // 13. Hex Input Box (Focuses for typing)
    if let Some(hex_rect) = targets.hex_input_rect
        && hex_rect.contains_point(pos)
    {
        out_actions.push(InspectorAction::FocusHexInput);
        return true;
    }

    // 14. Add to Palette Button
    if let Some(add_pal_rect) = targets.add_palette_btn_rect
        && add_pal_rect.contains_point(pos)
    {
        out_actions.push(InspectorAction::AddColorToPalette(Color::TRANSPARENT));
        return true;
    }

    // 15. Clear Palette Button
    if let Some(clr_pal_rect) = targets.clear_palette_btn_rect
        && clr_pal_rect.contains_point(pos)
    {
        out_actions.push(InspectorAction::ClearCustomPalette);
        return true;
    }

    // 16. Preset Reset Button
    if let Some(preset_rect) = targets.preset_btn_rect
        && preset_rect.contains_point(pos)
    {
        out_actions.push(InspectorAction::ResetPhysMatPreset);
        return true;
    }

    // 17. Palette Swatch Pills
    for &(_idx, sw_rect, col) in &targets.palette_swatches {
        if sw_rect.contains_point(pos) {
            out_actions.push(InspectorAction::SetObjectColor(col));
            return true;
        }
    }

    // 18. If clicking outside menus while an Add Menu is open, dismiss it
    if targets.active_add_menu_rect.is_some() || targets.active_dropdown_popup_rect.is_some() {
        out_actions.push(InspectorAction::CloseAddComponentMenu);
        return true;
    }

    false
}