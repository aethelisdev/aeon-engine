// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Top-Level Shell Dialogs and Transient Popups Subsystem
//!
//! Directly constructs modal dialogues, right-click context menus, cascading dropdown cards,
//! and asset drag-drop previews into the primary unified Shell [`UiTree`] elevated layers
//! ([`UiLayer::Modal`], [`UiLayer::Popup`], and [`UiLayer::Tooltip`]).
//!
//! Eliminates separate overlay trees and coordinate mapping wrappers in full compliance
//! with Phase 5.1.5 of the unified retained UI tree architecture.
//!

use super::about::build_about_dialog;
use super::hierarchy;
use super::inspector;
use super::modals::{
    self, build_delete_modal, build_loading_overlay, build_new_folder_modal, build_rename_modal,
};
use super::preferences::{self, build_preferences_dialog};
use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;

/// Geometry and environment parameters required to construct top-level floating overlays and modal dialogues.
#[derive(Debug, Clone, Copy)]
pub struct ShellDialogsContext {
    /// Total width of the application viewport in physical pixels.
    pub screen_width: f32,
    /// Total height of the application viewport in physical pixels.
    pub screen_height: f32,
    /// Current mouse cursor position in global window coordinates.
    pub cursor: Point,
    /// Docking framework content bounds between top menubar and bottom status bar.
    pub workspace_rect: Rect,
    /// Whether the cursor is currently occluded by an active modal, dialog, or floating window.
    pub is_cursor_occluded: bool,
}

impl IrisEditorOverlay {
    /// Builds all transient floating popups, context menus, and modal dialogs directly into the primary Shell tree.
    ///
    /// Every constructed element is tagged with appropriate [`UiLayer::Modal`] or [`UiLayer::Popup`]
    /// attributes, allowing Iris UI's native multi-layer compositor and hit-testing engine to process
    /// them with zero coordinate transformations and zero auxiliary tree overhead.
    pub(crate) fn build_shell_dialogs(
        &mut self,
        root: WidgetId,
        params: &mut OverlayUpdateParams<'_>,
        ctx: ShellDialogsContext,
    ) {
        let screen_width = ctx.screen_width;
        let screen_height = ctx.screen_height;
        let cursor = ctx.cursor;
        let is_cursor_occluded = ctx.is_cursor_occluded;
        let elapsed_secs = self.start_time.elapsed().as_secs_f32();

        // 1. Preferences Dialog
        if params.dialogs.show_preferences {
            let blink_caret = (self.start_time.elapsed().as_millis() % 1060) < 530;
            let caret_buf;
            let active_number_input =
                if let Some((t, ref s, is_all_selected)) = self.preferences.active_number_input {
                    let text_with_caret = if !is_all_selected && blink_caret {
                        caret_buf = format!("{}|", s);
                        caret_buf.as_str()
                    } else {
                        s.as_str()
                    };
                    Some((t, text_with_caret, is_all_selected))
                } else {
                    None
                };

            let (_pref_id, card_rect, content_rect, max_scroll_y) = build_preferences_dialog(
                &mut self.tree,
                preferences::PreferencesParams {
                    screen_width,
                    screen_height,
                    window_pos: self.preferences.pos,
                    active_tab: self.preferences.tab,
                    collapsed_sections: &self.preferences.collapsed_sections,
                    active_number_input,
                    blink_caret,
                    cursor_pos: cursor,
                    zoom_factor: params.context.zoom_factor,
                    graphics_settings: params.preferences.graphics_settings,
                    snapping_settings: params.preferences.snapping_settings,
                    editor_config: params.preferences.editor_config,
                    enable_live_updates: params.context.enable_live_updates,
                    enabled_modules: params.preferences.enabled_modules,
                    events: &self.preferences.pending_interaction_events,
                },
                &mut self.preferences.interactions.actions,
            );
            self.preferences.pending_interaction_events.clear();
            self.preferences.card_rect = Some(card_rect);
            self.preferences.content_rect = Some(content_rect);
            self.preferences.max_scroll_y = max_scroll_y;
        }

        // 2. About Modal Dialog
        if params.dialogs.show_about {
            let pending_events = std::mem::take(&mut self.modals.pending_interaction_events);
            let hovered_id = self.tree.hit_test_layered(cursor);
            build_about_dialog(
                &mut self.tree,
                screen_width,
                screen_height,
                cursor,
                &pending_events,
                hovered_id,
            );
        }

        // 3. Delete Confirmation Modal
        if let Some(target_path) = params.dialogs.delete_target {
            build_delete_modal(
                &mut self.tree,
                target_path,
                screen_width,
                screen_height,
                cursor,
            );
        }

        // 4. New Folder Modal
        if let Some(parent_path) = params.dialogs.new_folder_parent {
            build_new_folder_modal(
                &mut self.tree,
                modals::FolderModalParams {
                    parent_path,
                    input_text: &mut self.modals.new_folder_input.buffer,
                    screen_width,
                    screen_height,
                    cursor_pos: cursor,
                },
            );
        }

        // 5. Rename Modal
        if let Some((target_path, is_folder)) = params.dialogs.rename_target {
            build_rename_modal(
                &mut self.tree,
                modals::RenameModalParams {
                    target_path,
                    input_text: &mut self.modals.rename_input.buffer,
                    is_folder,
                    screen_width,
                    screen_height,
                    cursor_pos: cursor,
                },
            );
        }

        // 6. Asset Loading Splash
        if params.dialogs.is_loading_assets {
            build_loading_overlay(
                &mut self.tree,
                modals::LoadingOverlayParams {
                    screen_width,
                    screen_height,
                    time_secs: elapsed_secs,
                },
            );
        }

        // 7. Hierarchy Add Menu and Context Menu
        if let Some(hier_rect) = params.panel_rects.hierarchy {
            let hier_params = hierarchy::HierarchyPanelParams {
                panel_rect: hier_rect,
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                search_query: self.hierarchy.search_input.text(),
                is_editing: params.context.is_editing,
                is_2d: params.context.is_2d_mode,
                active_submenu: self.hierarchy.active_submenu,
                active_sub_submenu: self.hierarchy.active_sub_submenu,
                is_add_menu_open: self.hierarchy.is_add_menu_open,
                active_context_menu: self.hierarchy.active_context_menu,
                cursor_pos: cursor,
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
                focus_state: &self.focus_state,
                cursor_byte_idx: self.hierarchy.search_input.cursor_byte_idx,
                ctrl_held: self.chrome.ctrl_held,
                shift_held: self.chrome.shift_held,
            };
            let hier_tree = self
                .panels
                .get(crate::ui::panel_layout::PanelId::Hierarchy.id_str())
                .and_then(|p| p.tree());
            let mut hierarchy_overlay_actions = Vec::new();
            hierarchy::build_hierarchy_overlays(
                &mut self.tree,
                root,
                &hier_params,
                hier_tree,
                &mut hierarchy_overlay_actions,
            );
            self.hierarchy
                .interactions
                .actions
                .append(&mut hierarchy_overlay_actions);
        }

        // 8. Inspector Add Component Cascading Menu, Dropdowns, and Color Picker
        if let Some(insp_rect) = params.panel_rects.inspector {
            let num_input_ref = self
                .inspector
                .active_number_input
                .as_ref()
                .filter(|session| Some(session.entity) == params.scene.selected_entity)
                .map(|session| super::inspector::types::ActiveNumberInputState {
                    id: session.id,
                    buffer: session.buffer.as_str(),
                    cursor_idx: session.cursor_idx,
                    is_all_selected: session.is_all_selected,
                });
            let text_input_ref = self
                .inspector
                .active_text_input
                .as_ref()
                .filter(|(ent, _, _)| Some(*ent) == params.scene.selected_entity)
                .map(|(_, id, s)| (*id, s.as_str()));
            let rename_buf_ref = self
                .inspector
                .rename_input
                .as_ref()
                .filter(|(ent, _)| Some(*ent) == params.scene.selected_entity)
                .map(|(_, input)| input.text());
            let hex_buf_ref = self
                .inspector
                .hex_buffer
                .as_ref()
                .filter(|(ent, _)| Some(*ent) == params.scene.selected_entity)
                .map(|(_, s)| s.as_str());

            let is_inspector_popup_open = self.inspector.is_add_menu_open
                || self.inspector.active_dropdown.is_some()
                || self.inspector.is_color_picker_open;

            if is_inspector_popup_open {
                let mut inspector_overlay_actions = Vec::new();
                let insp_params = inspector::types::InspectorPanelParams {
                    panel_rect: insp_rect,
                    world: params.scene.world,
                    selected_entity: params.scene.selected_entity,
                    inspector_euler: params.panel_data.inspector_euler,
                    inspector_color_hex: params.panel_data.inspector_color_hex,
                    saved_swatches: params.panel_data.saved_swatches,
                    cursor_pos: cursor,
                    active_dropdown: self.inspector.active_dropdown,
                    active_submenu: self.inspector.active_submenu,
                    is_add_menu_open: self.inspector.is_add_menu_open,
                    is_color_picker_open: self.inspector.is_color_picker_open,
                    active_number_input: num_input_ref,
                    active_text_input: text_input_ref,
                    active_rename_buffer: rename_buf_ref,
                    is_rename_all_selected: self.inspector.rename_is_all_selected
                        && rename_buf_ref.is_some(),
                    active_hex_buffer: hex_buf_ref,
                    inspector_hsv: self.inspector.hsv,
                    blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
                };

                let insp_tree = self
                    .panels
                    .get(crate::ui::panel_layout::PanelId::Inspector.id_str())
                    .and_then(|p| p.tree());
                inspector::build_inspector_overlays(
                    &mut self.tree,
                    root,
                    &insp_params,
                    insp_tree,
                    &mut inspector_overlay_actions,
                );

                self.inspector
                    .actions
                    .append(&mut inspector_overlay_actions);
            }
        }

        // 9. Asset Drag & Viewport Drop Overlays
        if let Some(payload) = &params.panel_data.asset_browser.drag_payload {
            super::assets::build_asset_drag_overlays(
                &mut self.tree,
                root,
                payload,
                cursor,
                params.viewport.viewport_rect,
                params.viewport.camera,
                params.context.is_2d_mode,
            );
        }

        // 10. Native Dock Tab Overflow Dropdown Menu
        if let Some((leaf_id, anchor_rect)) = self.chrome.active_dock_overflow
            && let Some(ref mut frame) = self.chrome.native_dock_frame
        {
            super::native_dock::build_native_dock_overflow_menu(
                &mut self.tree,
                root,
                super::native_dock::NativeDockOverflowMenuParams {
                    leaf_id,
                    anchor_rect,
                    layout_state: params.context.layout_state,
                    cursor_pos: cursor,
                    is_cursor_occluded,
                },
                frame,
            );
        }
    }
}