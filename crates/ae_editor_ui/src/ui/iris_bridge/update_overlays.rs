// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Top-Level Shell and Transient Overlays Builder
//!
//! Constructs floating modal dialogues, context menus, cascading dropdown popups,
//! and drag preview indicators directly into the top-level Shell [`UiTree`] and [`OverlayTree`].
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).
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

/// Parameters required to construct top-level floating overlays and modal dialogues.
///
/// Groups geometry, occlusion flags, and cursor state into a single cohesive descriptor,
/// enforcing clean subsystem boundaries and zero Clippy argument warnings.
#[derive(Debug, Clone, Copy)]
pub struct ShellOverlayContext {
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
    /// Builds all transient floating overlays and modals into the Shell tree and registers them with [`OverlayTree`].
    ///
    /// Ensures that floating menus, dropdowns, and modal dialogs are never clipped by dock panel
    /// boundaries or GPU scissor rectangles.
    pub(crate) fn build_shell_overlays(
        &mut self,
        root: WidgetId,
        params: &mut OverlayUpdateParams<'_>,
        ctx: ShellOverlayContext,
    ) {
        let screen_width = ctx.screen_width;
        let screen_height = ctx.screen_height;
        let cursor = ctx.cursor;
        let is_cursor_occluded = ctx.is_cursor_occluded;

        let elapsed_secs = self.start_time.elapsed().as_secs_f32();

        self.overlay_tree.tree_mut().clear();
        self.overlay_tree.close();
        let _overlay_root = self.overlay_tree.ensure_root(screen_width, screen_height);

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
            let _overlay_root = self.overlay_tree.ensure_root(screen_width, screen_height);
            let (_pref_id, card_rect, content_rect, max_scroll_y) = build_preferences_dialog(
                self.overlay_tree.tree_mut(),
                preferences::PreferencesParams {
                    screen_width,
                    screen_height,
                    window_pos: self.preferences.pos,
                    active_tab: self.preferences.tab,
                    active_dropdown: self.preferences.dropdown,
                    dropdown_trigger_rect: self.preferences.dropdown_trigger_rect,
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
            self.overlay_tree.open(
                super::overlay_tree::ActiveOverlay::Modal(
                    super::overlay_tree::ModalKind::Preferences,
                ),
                super::overlay_tree::OverlayDismissPolicy::ExplicitOnly,
            );
            self.overlay_tree.set_overlay_bounds(card_rect);
        }

        // 2. About Modal Dialog
        if params.dialogs.show_about {
            let pending_events = std::mem::take(&mut self.modals.pending_interaction_events);
            let hovered_id = self.overlay_tree.tree().hit_test_layered(cursor);
            build_about_dialog(
                self.overlay_tree.tree_mut(),
                screen_width,
                screen_height,
                cursor,
                &pending_events,
                hovered_id,
            );
            self.overlay_tree.open(
                super::overlay_tree::ActiveOverlay::Modal(super::overlay_tree::ModalKind::About),
                super::overlay_tree::OverlayDismissPolicy::ExplicitOnly,
            );
            let card_rect = Rect::new(
                ((screen_width - super::about::ABOUT_DIALOG_WIDTH) * 0.5).max(0.0),
                ((screen_height - super::about::ABOUT_DIALOG_HEIGHT) * 0.5).max(0.0),
                super::about::ABOUT_DIALOG_WIDTH,
                super::about::ABOUT_DIALOG_HEIGHT,
            );
            self.overlay_tree.set_overlay_bounds(card_rect);
        }

        // 3. Delete Confirmation Modal
        if let Some(target_path) = params.dialogs.delete_target {
            build_delete_modal(
                self.overlay_tree.tree_mut(),
                target_path,
                screen_width,
                screen_height,
                cursor,
            );
            self.overlay_tree.open(
                super::overlay_tree::ActiveOverlay::Modal(
                    super::overlay_tree::ModalKind::DeleteConfirmation,
                ),
                super::overlay_tree::OverlayDismissPolicy::ExplicitOnly,
            );
            let card_rect = Rect::new(
                ((screen_width - modals::DELETE_MODAL_WIDTH) * 0.5).max(0.0),
                ((screen_height - modals::DELETE_MODAL_HEIGHT) * 0.5).max(0.0),
                modals::DELETE_MODAL_WIDTH,
                modals::DELETE_MODAL_HEIGHT,
            );
            self.overlay_tree.set_overlay_bounds(card_rect);
        }

        // 4. New Folder Modal
        if let Some(parent_path) = params.dialogs.new_folder_parent {
            build_new_folder_modal(
                self.overlay_tree.tree_mut(),
                modals::FolderModalParams {
                    parent_path,
                    input_text: &mut self.modals.new_folder_input.buffer,
                    screen_width,
                    screen_height,
                    cursor_pos: cursor,
                },
            );
            self.overlay_tree.open(
                super::overlay_tree::ActiveOverlay::Modal(
                    super::overlay_tree::ModalKind::NewFolder,
                ),
                super::overlay_tree::OverlayDismissPolicy::ExplicitOnly,
            );
            let card_rect = Rect::new(
                ((screen_width - modals::INPUT_MODAL_WIDTH) * 0.5).max(0.0),
                ((screen_height - modals::INPUT_MODAL_HEIGHT) * 0.5).max(0.0),
                modals::INPUT_MODAL_WIDTH,
                modals::INPUT_MODAL_HEIGHT,
            );
            self.overlay_tree.set_overlay_bounds(card_rect);
        }

        // 5. Rename Modal
        if let Some((target_path, is_folder)) = params.dialogs.rename_target {
            build_rename_modal(
                self.overlay_tree.tree_mut(),
                modals::RenameModalParams {
                    target_path,
                    input_text: &mut self.modals.rename_input.buffer,
                    is_folder,
                    screen_width,
                    screen_height,
                    cursor_pos: cursor,
                },
            );
            self.overlay_tree.open(
                super::overlay_tree::ActiveOverlay::Modal(super::overlay_tree::ModalKind::Rename),
                super::overlay_tree::OverlayDismissPolicy::ExplicitOnly,
            );
            let card_rect = Rect::new(
                ((screen_width - modals::INPUT_MODAL_WIDTH) * 0.5).max(0.0),
                ((screen_height - modals::INPUT_MODAL_HEIGHT) * 0.5).max(0.0),
                modals::INPUT_MODAL_WIDTH,
                modals::INPUT_MODAL_HEIGHT,
            );
            self.overlay_tree.set_overlay_bounds(card_rect);
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
            let overlay_root = self
                .overlay_tree
                .tree_mut()
                .root()
                .unwrap_or_else(|| self.overlay_tree.tree_mut().create_root().unwrap_or(root));
            let hier_tree = self
                .panels
                .get(crate::ui::panel_layout::PanelId::Hierarchy.id_str())
                .and_then(|p| p.tree());
            let mut hierarchy_overlay_actions = Vec::new();
            let menu_rects = hierarchy::build_hierarchy_overlays(
                self.overlay_tree.tree_mut(),
                overlay_root,
                &hier_params,
                hier_tree,
                &mut hierarchy_overlay_actions,
            );
            self.hierarchy
                .interactions
                .actions
                .append(&mut hierarchy_overlay_actions);
            if self.hierarchy.is_add_menu_open || self.hierarchy.active_context_menu.is_some() {
                self.overlay_tree.open(
                    super::overlay_tree::ActiveOverlay::HierarchyContextMenu,
                    super::overlay_tree::OverlayDismissPolicy::ClickOutside,
                );
                if !menu_rects.is_empty() {
                    let mut min_x = menu_rects[0].x;
                    let mut min_y = menu_rects[0].y;
                    let mut max_x = menu_rects[0].right();
                    let mut max_y = menu_rects[0].bottom();
                    for r in &menu_rects[1..] {
                        min_x = min_x.min(r.x);
                        min_y = min_y.min(r.y);
                        max_x = max_x.max(r.right());
                        max_y = max_y.max(r.bottom());
                    }
                    self.overlay_tree
                        .set_overlay_bounds(Rect::from_min_max(min_x, min_y, max_x, max_y));
                }
            }
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
                let overlay_root =
                    self.overlay_tree.tree_mut().root().unwrap_or_else(|| {
                        self.overlay_tree.tree_mut().create_root().unwrap_or(root)
                    });

                let mut inspector_overlay_actions = Vec::new();
                let bounds_opt = {
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
                        blink_caret: (self.start_time.elapsed().as_millis() / 500)
                            .is_multiple_of(2),
                    };

                    let insp_tree = self
                        .panels
                        .get(crate::ui::panel_layout::PanelId::Inspector.id_str())
                        .and_then(|p| p.tree());
                    inspector::build_inspector_overlays(
                        self.overlay_tree.tree_mut(),
                        overlay_root,
                        &insp_params,
                        insp_tree,
                        &mut inspector_overlay_actions,
                    )
                };

                let is_add_open = self.inspector.is_add_menu_open;
                let is_dropdown_open = self.inspector.active_dropdown.is_some();
                self.inspector
                    .actions
                    .append(&mut inspector_overlay_actions);

                if let Some(bounds) = bounds_opt {
                    let overlay_kind = if is_add_open {
                        super::overlay_tree::ActiveOverlay::InspectorAddComponent
                    } else if is_dropdown_open {
                        super::overlay_tree::ActiveOverlay::InspectorDropdown
                    } else {
                        super::overlay_tree::ActiveOverlay::InspectorColorPicker
                    };
                    self.overlay_tree.open(
                        overlay_kind,
                        super::overlay_tree::OverlayDismissPolicy::ClickOutside,
                    );
                    self.overlay_tree.set_overlay_bounds(bounds);
                }
            }
        }

        // 10. Asset Drag & Viewport Drop Overlays
        if let Some(payload) = &params.panel_data.asset_browser.drag_payload {
            let overlay_root = self
                .overlay_tree
                .tree_mut()
                .root()
                .unwrap_or_else(|| self.overlay_tree.tree_mut().create_root().unwrap_or(root));

            super::assets::build_asset_drag_overlays(
                self.overlay_tree.tree_mut(),
                overlay_root,
                payload,
                cursor,
                params.viewport.viewport_rect,
                params.viewport.camera,
                params.context.is_2d_mode,
            );
        }

        // 11. Top Menubar Dropdown Popup handled autonomously by UiScope::menu_bar

        // 12. Native Dock Tab Overflow Dropdown Menu
        if let Some((leaf_id, anchor_rect)) = self.chrome.active_dock_overflow
            && let Some(ref mut frame) = self.chrome.native_dock_frame
        {
            let overlay_root = self
                .overlay_tree
                .tree_mut()
                .root()
                .unwrap_or_else(|| self.overlay_tree.tree_mut().create_root().unwrap_or(root));

            super::native_dock::build_native_dock_overflow_menu(
                self.overlay_tree.tree_mut(),
                overlay_root,
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

        // 13. Finalize OverlayTree Text Pre-Measurement and Full Screen Layout Computation
        if self.overlay_tree.is_open()
            && let Some(r) = self.overlay_tree.tree().root()
        {
            self.text_system
                .measure_subtree_text(self.overlay_tree.tree_mut(), r);
            self.overlay_tree
                .compute_layout(Size::new(screen_width, screen_height));
        }
    }
}