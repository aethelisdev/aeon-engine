// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Top-Level Shell Overlays and Transient Popups Subsystem
//!
//! Handles remaining top-level floating overlays (asset drag-drop previews and dock tab overflow menus)
//! directly into the primary Shell [`UiTree`] elevated layers ([`UiLayer::Modal`], [`UiLayer::Popup`]).
//!
//! Modal dialogues are rendered autonomously by [`irisui::workbench::DialogHost`].
//! Preferences are managed via standard [`irisui::dock`] floating windows.
//! Contextual popups and cascading menus are rendered declaratively in widget [`UiScope`].

use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;

/// Geometry and environment parameters required to construct top-level floating overlays.
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
    /// Builds transient floating overlays (drag preview, tab overflow) directly into the primary Shell tree.
    ///
    /// Modal dialogs are rendered autonomously by `iris-workbench::DialogHost`.
    pub(crate) fn build_shell_dialogs(
        &mut self,
        root: WidgetId,
        params: &mut OverlayUpdateParams<'_>,
        ctx: ShellDialogsContext,
    ) {
        let cursor = ctx.cursor;
        let is_cursor_occluded = ctx.is_cursor_occluded;

        // 1. Asset Drag & Viewport Drop Overlays
        if let Some(payload) = &params.panel_data.asset_browser.drag_payload {
            let mut root_scope = UiScope::new(&mut self.tree, root);
            super::assets::build_asset_drag_overlays_scope(
                &mut root_scope,
                payload,
                cursor,
                params.viewport.viewport_rect,
                params.viewport.camera,
                params.context.is_2d_mode,
            );
        }

        // 2. Native Dock Tab Overflow Dropdown Menu
        if let Some((leaf_id, anchor_rect)) = self.chrome.active_dock_overflow
            && let Some(node) = params.context.layout_state.dock_state.tree.get(leaf_id)
            && let irisui::dock::DockNode::Leaf { tabs, active_tab } = node
            && !tabs.is_empty()
        {
            let style = irisui::dock::DockOverflowMenuStyle::default();
            let effective_cursor = if is_cursor_occluded {
                Point::new(-1000.0, -1000.0)
            } else {
                cursor
            };
            let registry = crate::ui::workbench::create_editor_panel_registry();
            self.chrome.active_overflow_frame = irisui::dock::build_dock_overflow_menu(
                &mut self.tree,
                root,
                irisui::dock::DockOverflowMenuParams {
                    leaf_id,
                    tabs,
                    active_tab: *active_tab,
                    anchor_rect,
                    cursor_pos: effective_cursor,
                },
                &registry.tab_viewer(),
                &style,
            );
        }
    }
}