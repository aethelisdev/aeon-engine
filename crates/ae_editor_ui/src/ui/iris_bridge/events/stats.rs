// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interaction and scrolling subsystem for the Performance Stats & Telemetry panel overlay.
//!
//! Evaluates clicks and toggles directly via hardware hit-testing ([`UiTree::hit_test_target`])
//! and semantic tags without retaining coordinate rectangles.
//!

use crate::ui::iris_bridge::hierarchy::is_hierarchy_tag;
use crate::ui::iris_bridge::stats::{
    STATS_TAG_TOGGLE_GRID, STATS_TAG_TOGGLE_WIREFRAME, StatsPanelAction, is_stats_tag,
};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles mouse clicks and checkbox toggles for the Performance Stats panel.
    ///
    /// Evaluates target elements directly via hardware hit-testing on the [`UiTree`]
    /// matching against [`STATS_TAG_TOGGLE_WIREFRAME`] and [`STATS_TAG_TOGGLE_GRID`].
    pub(crate) fn handle_stats_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let _ = self.stats.last_rect?;
        let mut result = IrisOverlayEventResult::default();

        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos();

            if let Some(hit) = self.tree.hit_test_target(click_point) {
                let effective_tag = resolve_ancestor_tag(&self.tree, hit.id);

                if effective_tag == STATS_TAG_TOGGLE_WIREFRAME {
                    self.stats.actions.push(StatsPanelAction::ToggleWireframe);
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                    result.consumed = true;
                    return Some(result);
                }

                if effective_tag == STATS_TAG_TOGGLE_GRID {
                    self.stats.actions.push(StatsPanelAction::ToggleGrid);
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                    result.consumed = true;
                    return Some(result);
                }

                if is_stats_tag(effective_tag) {
                    result.consumed = true;
                    return Some(result);
                }
            }
        }

        None
    }

    /// Handles mouse wheel scrolling for docked panels (Stats, Hierarchy, Preferences, Inspector).
    pub(crate) fn handle_panel_mouse_wheel(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let WindowEvent::MouseWheel { delta, .. } = event else {
            return None;
        };

        let mut result = IrisOverlayEventResult::default();
        let delta_y = match delta {
            winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 24.0,
            winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
        };

        let cursor = self.cursor_pos();

        // 1. Stats and Hierarchy Panel Scrolling via hardware hit-test
        if let Some(hit) = self.tree.hit_test_target(cursor) {
            let effective_tag = resolve_ancestor_tag(&self.tree, hit.id);
            if is_stats_tag(effective_tag) {
                self.stats.scroll_y =
                    (self.stats.scroll_y - delta_y).clamp(0.0, self.stats.max_scroll);
                self.chrome.needs_layout_rebuild = true;
                result.consumed = true;
                return Some(result);
            }
            if is_hierarchy_tag(effective_tag) {
                self.hierarchy.scroll_y =
                    (self.hierarchy.scroll_y - delta_y).clamp(0.0, self.hierarchy.max_scroll);
                self.chrome.needs_layout_rebuild = true;
                result.consumed = true;
                return Some(result);
            }
        }

        // 3. Preferences Dialog Scrolling
        if let Some(ref targets) = self.preferences.targets
            && targets.card_rect.contains_point(cursor)
        {
            self.preferences.scroll_y = (self.preferences.scroll_y - delta_y).max(0.0);
            result.consumed = true;
            return Some(result);
        }

        // 4. Inspector Panel Scrolling
        if let Some(ref targets) = self.inspector.targets
            && targets.scroll_container_rect.contains_point(cursor)
        {
            self.inspector.scroll_y = (self.inspector.scroll_y - delta_y).max(0.0);
            result.consumed = true;
            return Some(result);
        }

        None
    }
}

/// Traverses up the widget hierarchy starting from `start_id` to locate the first non-zero semantic tag.
///
/// Ensures that mouse clicks on inner child elements (e.g. checkmark text or inner checkbox container)
/// correctly resolve to the composite parent control's semantic tag.
fn resolve_ancestor_tag(
    tree: &irisui::prelude::UiTree,
    start_id: irisui::prelude::WidgetId,
) -> u64 {
    let mut curr = Some(start_id);
    while let Some(id) = curr {
        if let Some(node) = tree.get(id) {
            if node.tag != 0 {
                return node.tag;
            }
            curr = node.parent;
        } else {
            break;
        }
    }
    0
}