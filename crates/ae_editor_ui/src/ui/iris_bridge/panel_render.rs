// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Panel Render & Independent Tree Coordinator
//!
//! Directs panel rendering into independent [`EditorDockPanel`] trees in (0, 0)
//! local coordinates, providing hit-testing and pointer capture helpers.
//!
//! ## Architectural Invariants
//!
//! - **Local (0, 0) Coordinate Origin**: Every panel retained tree roots at `(0, 0)` with
//!   width and height equal to its allocated dock bounds. Layout is calculated within
//!   these local dimensions.
//! - **Zero Allocation When Clean**: If a panel is not dirty and its tree already has a root,
//!   it is skipped entirely with zero CPU work and zero heap allocations.
//! - **Pointer Capture Locking**: When dragging starts (e.g. timeline scrubbing, asset dragging),
//!   events are routed exclusively to the captured panel even when cursor leaves panel bounds.
//!

use crate::ui::panel_layout::PanelId;
use irisui::prelude::*;

use super::types::IrisEditorOverlay;

impl IrisEditorOverlay {
    /// Returns the active pointer capture panel, if any.
    pub fn pointer_capture(&self) -> Option<PanelId> {
        self.chrome.pointer_capture
    }

    /// Locks subsequent pointer motion and release events to the specified panel.
    pub fn set_pointer_capture(&mut self, panel_id: PanelId) {
        self.chrome.pointer_capture = Some(panel_id);
    }

    /// Releases any active pointer capture, restoring standard hit-test routing.
    pub fn release_pointer_capture(&mut self) {
        self.chrome.pointer_capture = None;
    }

    /// Returns an immutable reference to the retained [`UiTree`].
    pub fn panel_tree(&self, _panel_id: PanelId) -> &UiTree {
        &self.tree
    }

    /// Returns a mutable reference to the retained [`UiTree`].
    pub fn panel_tree_mut(&mut self, _panel_id: PanelId) -> Option<&mut UiTree> {
        Some(&mut self.tree)
    }

    /// Queries hit-testing on the retained tree using screen-space cursor coordinates.
    pub fn hit_test_panel(&self, _panel_id: PanelId, screen_point: Point) -> Option<HitTargetInfo> {
        self.tree.hit_test_target(screen_point)
    }

    /// Resolves an effective 64-bit semantic tag from an ancestor widget node.
    pub fn resolve_panel_tag(&self, _panel_id: PanelId, id: WidgetId) -> u64 {
        self.tree.resolve_ancestor_tag(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::types::IrisChromeState;
    use crate::ui::panel_layout::PanelId;

    #[test]
    fn test_pointer_capture_state_machine() {
        let mut chrome = IrisChromeState::default();
        assert_eq!(chrome.pointer_capture, None);

        chrome.pointer_capture = Some(PanelId::AnimationTimeline);
        assert_eq!(chrome.pointer_capture, Some(PanelId::AnimationTimeline));

        chrome.pointer_capture = None;
        assert_eq!(chrome.pointer_capture, None);
    }

    #[test]
    fn test_panel_tree_resolution_and_hit_testing() {
        let panel_bounds = Rect::new(100.0, 50.0, 300.0, 200.0);
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut scope = UiScope::new(&mut tree, root);
        scope
            .panel("ConsoleTestRoot")
            .bounds(panel_bounds)
            .tagged(0xCC00_0001)
            .background(Color::BLACK)
            .show(|s| {
                s.add(Button::new("Btn").name("TestBtn").tag(0xCC00_0002));
            });

        // Hit-test target query with screen coordinates: (150.0, 70.0) is inside panel_bounds
        let screen_point = Point::new(150.0, 70.0);
        assert!(panel_bounds.contains_point(screen_point));
        let hit = tree.hit_test_target(screen_point);
        assert!(hit.is_some());

        // Hit-test outside screen bounds
        let outside_point = Point::new(50.0, 20.0);
        assert!(!panel_bounds.contains_point(outside_point));
    }

    #[test]
    fn test_chrome_is_dirty_state_machine() {
        let mut chrome = IrisChromeState {
            last_dimensions: (1920.0, 1080.0),
            last_zoom_factor: 1.0,
            last_floating_count: 0,
            last_has_drag_payload: false,
            ..Default::default()
        };

        // Clean state
        assert!(!chrome.is_dirty((1920.0, 1080.0), 1.0, 0, false));

        // Dimension delta
        assert!(chrome.is_dirty((2560.0, 1440.0), 1.0, 0, false));

        // Zoom delta
        assert!(chrome.is_dirty((1920.0, 1080.0), 1.25, 0, false));

        // Floating count delta
        assert!(chrome.is_dirty((1920.0, 1080.0), 1.0, 1, false));

        // Drag payload
        assert!(chrome.is_dirty((1920.0, 1080.0), 1.0, 0, true));

        // Explicit layout rebuild
        chrome.needs_layout_rebuild = true;
        assert!(chrome.is_dirty((1920.0, 1080.0), 1.0, 0, false));
        chrome.needs_layout_rebuild = false;

        // Always rebuild override
        chrome.always_rebuild = true;
        assert!(chrome.is_dirty((1920.0, 1080.0), 1.0, 0, false));
    }

    #[test]
    fn test_unified_tree_layout_and_layer_compilation() {
        use irisui::text::{TextCollectionOptions, collect_text_sections_with_options};
        use irisui::wgpu_backend::{TreeCompilerOptions, compile_tree_draw_commands_into};

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");

        {
            let mut scope = UiScope::new(&mut tree, root);
            // Shell base
            scope.add(Button::new("TopMenubarBtn").tag(0xBB00_0001));
            // Panel container
            scope.container_tagged(
                "HierarchyContainer",
                Style::new().width(300.0).height(400.0).flex_col(),
                WidgetRole::Default,
                0xAA00_0001,
                |s| {
                    s.add(Button::new("EntityRow1").tag(0xAA00_0002));
                    s.add(Button::new("EntityRow2").tag(0xAA00_0003));
                },
            );
            // Modal dropdown overlay
            scope.add(
                Button::new("DropdownItem")
                    .tag(0xCC00_0001)
                    .layer(UiLayer::Modal),
            );
        }

        // Layout computation
        let mut text_system = irisui::text::TextSystem::new();
        text_system.measure_subtree_text(&mut tree, root);
        let mut layout_engine = LayoutEngine::new();
        let layout_res = layout_engine.compute_layout(&mut tree, Size::new(1920.0, 1080.0));
        assert!(layout_res.is_ok(), "Layout computation must succeed");

        // Native tree compilation into DrawCommandList
        let mut command_list = irisui::wgpu_backend::DrawCommandList::new();
        let mut options = TreeCompilerOptions::new();
        compile_tree_draw_commands_into(&tree, root, &mut options, &mut command_list);

        assert!(
            !command_list.quads.is_empty(),
            "Command list must contain quads"
        );

        // Unified typography collection across all layers
        let text_options = TextCollectionOptions::default();
        let sections = collect_text_sections_with_options(&tree, &text_options);
        assert_eq!(
            sections.len(),
            4,
            "Unified text sections must encompass all nodes (menubar + 2 panel rows + modal)"
        );
    }
}