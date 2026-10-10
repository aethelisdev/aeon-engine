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

use super::compositor;
use super::dock_panel::EditorDockPanel;
use super::types::{IrisEditorOverlay, OverlayUpdateParams};

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

    /// Returns an immutable reference to the independently retained [`UiTree`] owned by a panel.
    ///
    /// If the panel is registered and has an active tree, returns its tree; otherwise falls back
    /// to the shell tree [`Self::tree`].
    pub fn panel_tree(&self, panel_id: PanelId) -> &UiTree {
        if let Some(panel) = self.panels.get(panel_id.id_str())
            && let Some(tree) = panel.tree()
            && tree.root().is_some()
        {
            tree
        } else {
            &self.tree
        }
    }

    /// Queries hit-testing on a panel's retained tree using screen-space cursor coordinates.
    ///
    /// Automatically converts screen coordinates to panel-local `(0, 0)` coordinates
    /// using `screen_to_panel_local` before evaluating `UiTree::hit_test_target`.
    pub fn hit_test_panel(&self, panel_id: PanelId, screen_point: Point) -> Option<HitTargetInfo> {
        if let Some(panel) = self.panels.get(panel_id.id_str())
            && let Some(tree) = panel.tree()
            && let Some(bounds) = panel.bounds()
            && tree.root().is_some()
        {
            if !compositor::is_point_in_panel(screen_point, bounds) {
                return None;
            }
            let local_point = compositor::screen_to_panel_local(screen_point, bounds);
            tree.hit_test_target(local_point)
        } else {
            None
        }
    }

    /// Resolves an effective 64-bit semantic tag from an ancestor widget node in a panel's tree.
    pub fn resolve_panel_tag(&self, panel_id: PanelId, id: WidgetId) -> u64 {
        if let Some(panel) = self.panels.get(panel_id.id_str())
            && let Some(tree) = panel.tree()
            && tree.root().is_some()
        {
            tree.resolve_ancestor_tag(id)
        } else {
            self.tree.resolve_ancestor_tag(id)
        }
    }

    /// Renders a tool panel into its independently retained [`EditorDockPanel::tree`].
    ///
    /// Evaluates whether the panel is dirty or unbuilt. If rebuild is needed, extracts
    /// the tree, clears it, constructs the panel content using local `(0, 0)` coordinates,
    /// recomputes layout, and stores it back into [`Self::panels`].
    pub(crate) fn render_dock_panel_into_tree(
        &mut self,
        panel_id: PanelId,
        params: &OverlayUpdateParams<'_>,
    ) {
        let rect_opt = match panel_id {
            PanelId::Viewport => None,
            PanelId::Hierarchy => params.panel_rects.hierarchy,
            PanelId::Inspector => params.panel_rects.inspector,
            PanelId::Console => params.panel_rects.console,
            PanelId::Assets => params.panel_rects.assets,
            PanelId::MaterialEditor => params.panel_rects.material,
            PanelId::AnimationTimeline => params.panel_rects.timeline,
            PanelId::UiDesigner => params.panel_rects.ui_designer,
            PanelId::Stats => params.panel_rects.stats,
        };

        let Some(panel_bounds) = rect_opt else {
            if let Some(panel) = self
                .panels
                .get_downcast_mut::<EditorDockPanel>(panel_id.id_str())
            {
                panel.clear_bounds();
            }
            return;
        };

        if panel_bounds.width <= 20.0 || panel_bounds.height <= 20.0 {
            if let Some(panel) = self
                .panels
                .get_downcast_mut::<EditorDockPanel>(panel_id.id_str())
            {
                panel.clear_bounds();
            }
            return;
        }

        let is_dirty = self
            .panels
            .get_downcast_mut::<EditorDockPanel>(panel_id.id_str())
            .is_some_and(|p| {
                p.is_dirty()
                    || self.notifier.is_dirty(panel_id)
                    || self.notifier.is_global_dirty()
                    || p.tree().root().is_none()
                    || p.last_bounds() != Some(panel_bounds)
            });

        if !is_dirty {
            return;
        }

        let mut tree = if let Some(panel) = self
            .panels
            .get_downcast_mut::<EditorDockPanel>(panel_id.id_str())
        {
            std::mem::take(panel.tree_mut())
        } else {
            UiTree::new()
        };

        tree.clear();
        if let Ok(root) = tree.create_root() {
            // Directly build into the panel's isolated tree without mem::swap shims
            self.render_panel_by_id(panel_id, &mut tree, root, params);

            // Pre-measure all text nodes to populate intrinsic content_size
            self.text_system.measure_subtree_text(&mut tree, root);

            // Compute Taffy layout for the panel's local coordinates (0, 0, width, height)
            let mut layout_engine = LayoutEngine::new();
            let _ = layout_engine.compute_layout(
                &mut tree,
                Size::new(panel_bounds.width, panel_bounds.height),
            );
        }

        if let Some(panel) = self
            .panels
            .get_downcast_mut::<EditorDockPanel>(panel_id.id_str())
        {
            *panel.tree_mut() = tree;
            panel.clear_dirty();
            panel.set_last_bounds(panel_bounds);
        }
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
        let mut panel = EditorDockPanel::new(PanelId::Console);
        panel.set_last_bounds(panel_bounds);

        let tree = panel.tree_mut();
        let root = tree.create_root().expect("root node");
        let mut scope = UiScope::new(tree, root);
        scope.panel_tagged(
            "ConsoleTestRoot",
            Rect::new(0.0, 0.0, 300.0, 200.0),
            0xCC00_0001,
            Color::BLACK,
            |s| {
                s.button_named_tagged("TestBtn", "Btn", 0xCC00_0002);
            },
        );

        // Hit-test local query with coordinate transformation: (150.0, 70.0) -> local (50.0, 20.0)
        let screen_point = Point::new(150.0, 70.0);
        let local_point = compositor::screen_to_panel_local(screen_point, panel_bounds);
        let hit = panel.hit_test_local(local_point);
        assert!(hit.is_some());

        // Hit-test outside screen bounds
        let outside_point = Point::new(50.0, 20.0);
        assert!(!compositor::is_point_in_panel(outside_point, panel_bounds));
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
    fn test_dock_panels_layout_computation_and_three_layer_compositing() {
        use crate::ui::iris_bridge::compositor::TreeCompositor;
        use irisui::text::TextCollectionOptions;

        // 1. Construct panel and build nodes in local (0, 0) space
        let panel_bounds = Rect::new(100.0, 50.0, 300.0, 400.0);
        let mut panel = EditorDockPanel::new(PanelId::Hierarchy);
        panel.set_last_bounds(panel_bounds);

        let tree = panel.tree_mut();
        let root = tree.create_root().expect("root node");

        {
            let mut scope = UiScope::new(tree, root);
            scope.container_tagged(
                "HierarchyLocalContainer",
                Style::new().width(300.0).height(400.0).flex_col(),
                WidgetRole::Default,
                0xAA00_0001,
                |s| {
                    let _ = s.button_tagged("EntityRow1", 0xAA00_0002);
                    let _ = s.button_tagged("EntityRow2", 0xAA00_0003);
                },
            );
        }

        // Before compute_layout, local nodes have zero layout dimensions
        let root_node_before = tree.get(root).unwrap();
        assert_eq!(root_node_before.computed_rect, Rect::ZERO);

        // 2. Perform layout computation with LayoutEngine
        let mut text_system = irisui::text::TextSystem::new();
        text_system.measure_subtree_text(tree, root);
        let mut layout_engine = LayoutEngine::new();
        let layout_res =
            layout_engine.compute_layout(tree, Size::new(panel_bounds.width, panel_bounds.height));
        assert!(layout_res.is_ok(), "Layout computation must succeed");

        // After compute_layout, nodes have non-zero local computed bounds
        let root_node_after = tree.get(root).unwrap();
        assert!(
            root_node_after.computed_rect.width >= 300.0,
            "Root node must have computed width after layout: got {}",
            root_node_after.computed_rect.width
        );
        assert!(
            root_node_after.computed_rect.height >= 400.0,
            "Root node must have computed height after layout: got {}",
            root_node_after.computed_rect.height
        );

        // 3. Test multi-tree layer composition via TreeCompositor
        let mut shell_tree = UiTree::new();
        let shell_root = shell_tree.create_root().unwrap();
        {
            let mut scope = UiScope::new(&mut shell_tree, shell_root);
            let _ = scope.button_tagged("TopMenubarBtn", 0xBB00_0001);
            scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 30.0));
        }

        let mut overlay_tree = UiTree::new();
        let overlay_root = overlay_tree.create_root().unwrap();
        {
            let mut scope = UiScope::new(&mut overlay_tree, overlay_root);
            let _ = scope.button_tagged("DropdownItem", 0xCC00_0001);
            scope.finish_layout(Rect::new(50.0, 30.0, 150.0, 200.0));
        }

        let mut compositor = TreeCompositor::new();

        // Layer 0: Shell commands
        let mut shell_commands = irisui::wgpu_backend::DrawCommandList::new();
        let mut shell_options = TreeCompilerOptions::new();
        compile_tree_draw_commands_into(
            &shell_tree,
            shell_root,
            &mut shell_options,
            &mut shell_commands,
        );
        compositor.append_shell_commands(shell_commands);

        // Layer 1: Docked panel tree with hardware scissor and offset
        compositor.append_panel_tree(tree, panel_bounds, None, None, false);

        // Layer 2: Overlay tree at topmost Z-order
        compositor.append_overlay_tree(&overlay_tree, None, None, false);

        // Verify compositor layers and commands
        assert_eq!(compositor.composited_layer_count, 1);
        assert!(
            !compositor.command_list.quads.is_empty(),
            "Composited command list must contain SDF quads"
        );

        // Verify hardware scissor was pushed for panel bounds (100, 50, 300, 400)
        let has_scissor = compositor.command_list.commands.iter().any(|cmd| {
            matches!(
                cmd,
                irisui::wgpu_backend::DrawCommand::SetScissor {
                    x: 100,
                    y: 50,
                    width: 300,
                    height: 400
                }
            )
        });
        assert!(
            has_scissor,
            "TreeCompositor must configure hardware scissor matching panel bounds"
        );

        // 4. Verify composited typography collection across all 3 layers
        let text_options = TextCollectionOptions::default();
        let docked_panels = [(tree as &UiTree, panel_bounds)];
        let sections = TreeCompositor::collect_composited_text_sections(
            &shell_tree,
            &docked_panels,
            &overlay_tree,
            &text_options,
        );

        // Shell has 1 text, panel has 2 texts, overlay has 1 text = 4 sections
        assert_eq!(
            sections.len(),
            4,
            "Composited text sections must encompass all active trees (shell + panel + overlay)"
        );

        // Panel text must be translated by panel_bounds origin (+100.0, +50.0)
        let panel_section = sections
            .iter()
            .find(|s| s.text == "EntityRow1")
            .expect("EntityRow1 must be found");
        assert!(
            panel_section.bounds.x >= 100.0,
            "Panel text bounds.x must be offset by panel_bounds.x (>= 100.0), got {}",
            panel_section.bounds.x
        );
        assert!(
            panel_section.bounds.y >= 50.0,
            "Panel text bounds.y must be offset by panel_bounds.y (>= 50.0), got {}",
            panel_section.bounds.y
        );
    }
}