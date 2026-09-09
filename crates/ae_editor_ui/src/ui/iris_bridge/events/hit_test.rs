// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Hit-testing and overlay boundary verification subsystem for Iris UI editor components.

use crate::ui::iris_bridge::types::IrisEditorOverlay;
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Returns true if the coordinate is over an active floating modal dialog, Preferences window, or menubar dropdown.
    /// When true, underlying dock splitters, tabs, and panel controls MUST NOT receive click or drag interactions.
    pub fn is_point_over_modal_or_dropdown(&self, point: Point) -> bool {
        if point.y <= Self::MENUBAR_HEIGHT {
            return true;
        }
        // Floating dropdown popup from menubar has highest z-order
        if let Some(dd_rect) = self.dropdown_rect
            && dd_rect.contains_point(point)
        {
            return true;
        }
        if self
            .about_targets
            .as_ref()
            .is_some_and(|t| t.dialog_rect.contains_point(point))
            || self
                .delete_targets
                .as_ref()
                .is_some_and(|t| t.dialog_rect.contains_point(point))
            || self
                .new_folder_targets
                .as_ref()
                .is_some_and(|t| t.dialog_rect.contains_point(point))
            || self
                .rename_targets
                .as_ref()
                .is_some_and(|t| t.dialog_rect.contains_point(point))
            || self.loading_targets.as_ref().is_some_and(|t| {
                t.card_rect.contains_point(point) || t.scrim_rect.contains_point(point)
            })
            || self
                .assets_targets
                .as_ref()
                .and_then(|a| a.preview_modal.as_ref())
                .is_some_and(|m| m.dialog_rect.contains_point(point))
            || self
                .assets_targets
                .as_ref()
                .and_then(|a| a.context_menu.as_ref())
                .is_some_and(|c| c.card_rect.contains_point(point))
        {
            return true;
        }
        if let Some(ref targets) = self.preferences_targets
            && (targets.card_rect.contains_point(point)
                || targets
                    .active_dropdown_popup_rect
                    .is_some_and(|r| r.contains_point(point)))
        {
            return true;
        }
        false
    }

    /// Returns true if the given coordinate is over the menubar, status bar, active dropdown/modal,
    /// or active interactive editor panels (respecting floating window occlusion).
    pub fn is_point_over_overlay(&self, point: Point) -> bool {
        // 1. Top Menubar and active modal dialogs (About, Preferences, Delete, Rename, Loading, Dropdowns)
        // These always have the absolute highest z-order above everything else.
        if point.y <= Self::MENUBAR_HEIGHT {
            return true;
        }
        // Floating dropdown popup from menubar has highest z-order above docked panels and modals
        if let Some(dd_rect) = self.dropdown_rect
            && dd_rect.contains_point(point)
        {
            return true;
        }
        if self.about_targets.is_some()
            || self.delete_targets.is_some()
            || self.new_folder_targets.is_some()
            || self.rename_targets.is_some()
            || self.loading_targets.is_some()
            || self.assets_preview_modal.is_some()
        {
            return true;
        }
        if let Some(ref targets) = self.preferences_targets
            && (targets.card_rect.contains_point(point)
                || targets
                    .active_dropdown_popup_rect
                    .is_some_and(|r| r.contains_point(point)))
        {
            return true;
        }

        // 2. Viewport HUD targets (toolbar buttons, dropdowns, compass, billboard icons)
        // Even when the 3D Viewport is detached into a floating window, its HUD buttons remain interactive.
        if let Some(ref hud) = self.viewport_hud_targets {
            if let Some(dd_rect) = hud.active_dropdown_popup_rect
                && dd_rect.contains_point(point)
            {
                return true;
            }
            if hud.buttons.iter().any(|(_, r)| r.contains_point(point))
                || hud
                    .dropdown_triggers
                    .iter()
                    .any(|(_, r)| r.contains_point(point))
                || hud
                    .compass_knobs
                    .iter()
                    .any(|(_, r)| r.contains_point(point))
                || hud
                    .billboard_icons
                    .iter()
                    .any(|(_, r)| r.contains_point(point))
            {
                return true;
            }
        }

        // 3. Occlusion Check: If the point is inside an active floating window,
        // all underlying docked panels (UI Designer, Hierarchy, Stats, Console, Assets,
        // Timeline, Material, Inspector) and the bottom status bar are occluded and
        // MUST NOT claim the point!
        let is_occluded_by_floating = self
            .floating_window_rects
            .iter()
            .any(|r| r.contains_point(point));
        if is_occluded_by_floating {
            return false;
        }

        // 4. Background Docked Panels (only tested when NOT occluded by floating windows)
        if let Some(ref targets) = self.hierarchy_targets {
            if let Some(sub2_rect) = targets.active_sub_submenu_rect
                && sub2_rect.contains_point(point)
            {
                return true;
            }
            if let Some(sub_rect) = targets.active_submenu_rect
                && sub_rect.contains_point(point)
            {
                return true;
            }
            if let Some(add_rect) = targets.active_add_menu_rect
                && add_rect.contains_point(point)
            {
                return true;
            }
            if let Some((_, menu_rect, _, _)) = targets.active_context_menu
                && menu_rect.contains_point(point)
            {
                return true;
            }
            if targets.panel_rect.contains_point(point) {
                return true;
            }
        }
        if let Some(ref targets) = self.stats_targets
            && targets.panel_rect.contains_point(point)
        {
            return true;
        }
        if let Some(ref targets) = self.console_targets
            && targets.panel_rect.contains_point(point)
        {
            return true;
        }
        if let Some(ref targets) = self.assets_targets {
            if let Some(ref cm) = targets.context_menu
                && cm.card_rect.contains_point(point)
            {
                return true;
            }
            if targets.panel_rect.contains_point(point) {
                return true;
            }
        }
        if let Some(ref targets) = self.timeline_targets
            && targets.panel_rect.contains_point(point)
        {
            return true;
        }
        if let Some(ref targets) = self.material_targets
            && targets.panel_rect.contains_point(point)
        {
            return true;
        }
        if let Some(ref targets) = self.ui_designer_targets {
            if let Some(ref popup_r) = targets.aspect_popup_rect
                && popup_r.contains_point(point)
            {
                return true;
            }
            if let Some(ref popup_r) = targets.add_popup_rect
                && popup_r.contains_point(point)
            {
                return true;
            }
            if targets.panel_rect.contains_point(point) {
                return true;
            }
        }
        if let Some(ref targets) = self.inspector_targets {
            if let Some(picker_rect) = targets.color_picker_popup_rect
                && picker_rect.contains_point(point)
            {
                return true;
            }
            if let Some(sub_rect) = targets.active_submenu_rect
                && sub_rect.contains_point(point)
            {
                return true;
            }
            if let Some(add_rect) = targets.active_add_menu_rect
                && add_rect.contains_point(point)
            {
                return true;
            }
            if targets.scroll_container_rect.contains_point(point)
                || targets.add_component_btn_rect.contains_point(point)
                || targets.save_prefab_btn_rect.contains_point(point)
                || targets.name_input_rect.contains_point(point)
            {
                return true;
            }
        }
        if self.screen_height > Self::STATUS_BAR_HEIGHT
            && point.y >= (self.screen_height - Self::STATUS_BAR_HEIGHT)
        {
            return true;
        }
        false
    }
}