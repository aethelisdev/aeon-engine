// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Autonomous subtree invalidation, atomic leaf purging, and boundary layout resolution.
//!
//! Provides granular, per-leaf rebuilding capabilities for complex retained UI hierarchies,
//! docking layouts, and compound widgets, eliminating the need for full UI tree reconstruction
//! (`UiTree::clear()`) and global layout solver passes during routine localized updates.

use super::UiScope;
use iris_core::{UiTree, WidgetId};
use iris_layout::{LayoutEngine, LayoutError};
use iris_text::TextSystem;

/// Autonomous coordinator managing isolated subtree invalidation and layout resolution.
///
/// Designed for scenarios where the rendering pass needs mutable access to state that may also own
/// the tree or layout systems, or where rendering cannot be enclosed in a single synchronous closure.
///
/// # Lifecycle:
/// 1. [`Self::begin`]:
///    - Untracks old child layout nodes from Taffy (`layout_engine.untrack_subtree`).
///    - Recursively purges arena descendants from the generational tree (`tree.clear_children`).
/// 2. User/Caller renders child widgets under `leaf_id` into a [`UiScope`].
/// 3. [`Self::finish`]:
///    - Shapes and measures typography exclusively within the subtree (`text_system.measure_subtree_text`).
///    - Computes boundary layout in $O(M)$ time (`layout_engine.compute_boundary_subtree`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubtreeSession {
    leaf_id: WidgetId,
}

impl SubtreeSession {
    /// Begins an isolated subtree rebuild on the specified `leaf_id`.
    ///
    /// Untracks previous child layout nodes from Taffy and clears all descendants under `leaf_id`
    /// while preserving `leaf_id` and its boundary properties in place.
    pub fn begin(tree: &mut UiTree, layout_engine: &mut LayoutEngine, leaf_id: WidgetId) -> Self {
        layout_engine.untrack_subtree(tree, leaf_id);
        let _ = tree.clear_children(leaf_id);
        Self { leaf_id }
    }

    /// Returns the target boundary leaf identifier.
    #[inline]
    #[must_use]
    pub fn leaf_id(&self) -> WidgetId {
        self.leaf_id
    }

    /// Finalizes the subtree rebuild by measuring text typography and recomputing boundary layout.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if layout computation encounters invalid geometry.
    pub fn finish(
        self,
        tree: &mut UiTree,
        layout_engine: &mut LayoutEngine,
        text_system: &mut TextSystem,
    ) -> Result<(), LayoutError> {
        text_system.measure_subtree_text(tree, self.leaf_id);
        layout_engine.compute_boundary_subtree(tree, self.leaf_id)
    }

    /// Atomically untracks, clears, renders, measures typography, and recomputes layout for an isolated boundary subtree.
    ///
    /// Coordinates the atomic invalidation, generational arena clearing, fresh widget synthesis,
    /// typography measurement, and boundary layout resolution for a single boundary container.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if boundary layout computation encounters invalid geometry.
    pub fn rebuild<F>(
        tree: &mut UiTree,
        layout_engine: &mut LayoutEngine,
        text_system: &mut TextSystem,
        boundary_id: WidgetId,
        render_fn: F,
    ) -> Result<(), LayoutError>
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        rebuild_leaf_subtree(tree, layout_engine, text_system, boundary_id, render_fn)
    }
}

/// Atomically untracks, clears, renders, measures typography, and recomputes layout for an isolated leaf subtree.
///
/// Unites the core framework systems ([`UiTree`], [`LayoutEngine`], and [`TextSystem`])
/// to provide a high-level, single-call declarative entry point for partial UI reconstruction.
///
/// # Lifecycle:
/// 1. Untracks previous child layout nodes from Taffy layout storage.
/// 2. Recursively purges all children under `leaf_id` from the [`UiTree`] arena while keeping `leaf_id` intact.
/// 3. Populates fresh child widgets under `leaf_id` via the provided `render_fn` closure.
/// 4. Shapes and measures intrinsic text typography exclusively within the leaf subtree.
/// 5. Recomputes layout geometry in $O(M)$ time via [`LayoutEngine::compute_boundary_subtree`].
///
/// # Errors
/// Returns [`LayoutError`] if `leaf_id` is not tracked in the tree or has invalid boundary dimensions.
pub fn rebuild_leaf_subtree<F>(
    tree: &mut UiTree,
    layout_engine: &mut LayoutEngine,
    text_system: &mut TextSystem,
    leaf_id: WidgetId,
    render_fn: F,
) -> Result<(), LayoutError>
where
    F: FnOnce(&mut UiScope<'_>),
{
    let session = SubtreeSession::begin(tree, layout_engine, leaf_id);
    {
        let mut scope = UiScope::new(tree, leaf_id);
        render_fn(&mut scope);
    }
    session.finish(tree, layout_engine, text_system)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Button;
    use iris_core::{Rect, Size};

    #[test]
    fn test_rebuild_leaf_subtree_isolates_invalidation_and_preserves_siblings() {
        let mut tree = UiTree::new();
        let mut layout_engine = LayoutEngine::new();
        let mut text_system = TextSystem::new();

        let root = tree.create_root().expect("root widget");
        UiScope::new(&mut tree, root).finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));

        let mut menubar_btn_ids = Vec::new();
        let mut panel_b_btn_ids = Vec::new();
        let mut old_panel_a_btn_ids = Vec::new();

        // 1. Build initial tree structure with Menubar, Panel A, and Panel B
        let (menubar_id, panel_a_id, panel_b_id) = {
            let mut root_scope = UiScope::new(&mut tree, root);

            // Menubar container at top
            let resp_menu = root_scope.panel("menubar").show(|menu_scope| {
                let b1 = menu_scope.add(Button::new("File").tag(101));
                let b2 = menu_scope.add(Button::new("Edit").tag(102));
                let b3 = menu_scope.add(Button::new("View").tag(103));
                menubar_btn_ids.extend([b1.id, b2.id, b3.id]);
            });

            // Panel A leaf
            let resp_pa = root_scope.panel("panel_a").show(|pa_scope| {
                let b1 = pa_scope.add(Button::new("A_Action1").tag(201));
                let b2 = pa_scope.add(Button::new("A_Action2").tag(202));
                old_panel_a_btn_ids.extend([b1.id, b2.id]);
            });

            // Panel B leaf
            let resp_pb = root_scope.panel("panel_b").show(|pb_scope| {
                let b1 = pb_scope.add(Button::new("B_Prop1").tag(301));
                let b2 = pb_scope.add(Button::new("B_Prop2").tag(302));
                panel_b_btn_ids.extend([b1.id, b2.id]);
            });

            (
                resp_menu.response.id,
                resp_pa.response.id,
                resp_pb.response.id,
            )
        };

        // Set explicit boundary rects on parent containers declaratively
        UiScope::new(&mut tree, menubar_id).finish_layout(Rect::new(0.0, 0.0, 1920.0, 30.0));
        UiScope::new(&mut tree, panel_a_id).finish_layout(Rect::new(0.0, 30.0, 800.0, 600.0));
        UiScope::new(&mut tree, panel_b_id).finish_layout(Rect::new(800.0, 30.0, 800.0, 600.0));

        // 2. Initial full layout computation
        assert!(
            layout_engine
                .compute_layout(&mut tree, Size::new(1920.0, 1080.0))
                .is_ok()
        );

        // Snapshot initial state of untouched containers
        let menubar_rects_before: Vec<Rect> = menubar_btn_ids
            .iter()
            .map(|id| tree.get(*id).unwrap().computed_rect)
            .collect();
        let panel_b_rects_before: Vec<Rect> = panel_b_btn_ids
            .iter()
            .map(|id| tree.get(*id).unwrap().computed_rect)
            .collect();

        assert_eq!(menubar_rects_before.len(), 3);
        assert_eq!(panel_b_rects_before.len(), 2);

        // Verify old panel A children are tracked in layout engine
        for id in &old_panel_a_btn_ids {
            assert!(layout_engine.is_tracked(*id));
        }

        // 3. Rebuild ONLY Panel A leaf in place
        let rebuild_result = rebuild_leaf_subtree(
            &mut tree,
            &mut layout_engine,
            &mut text_system,
            panel_a_id,
            |pa_scope| {
                pa_scope.add(Button::new("A_Rebuilt1").tag(401));
                pa_scope.add(Button::new("A_Rebuilt2").tag(402));
                pa_scope.add(Button::new("A_Rebuilt3").tag(403));
            },
        );
        assert!(rebuild_result.is_ok(), "rebuild_leaf_subtree must succeed");

        // 4. Verify Panel A was reconstructed
        let panel_a_node = tree.get(panel_a_id).expect("panel_a node exists");
        assert_eq!(panel_a_node.children.len(), 3);

        // Verify old children of Panel A were completely purged from tree and untracked from layout
        for old_id in &old_panel_a_btn_ids {
            assert!(
                tree.get(*old_id).is_none(),
                "Old node must be removed from arena"
            );
            assert!(
                !layout_engine.is_tracked(*old_id),
                "Old node must be untracked from layout engine"
            );
        }

        // Verify new children of Panel A have valid non-zero layout bounds
        for new_id in &panel_a_node.children {
            let child = tree.get(*new_id).expect("new child exists");
            assert!(child.computed_rect.width > 0.0);
            assert!(child.computed_rect.height > 0.0);
            assert!(layout_engine.is_tracked(*new_id));
        }

        // 5. CRITICAL: Verify Menubar and Panel B are 100% UNTOUCHED
        let menubar_node = tree.get(menubar_id).expect("menubar exists");
        assert_eq!(menubar_node.children, menubar_btn_ids);
        for (i, id) in menubar_btn_ids.iter().enumerate() {
            let rect = tree.get(*id).unwrap().computed_rect;
            assert_eq!(rect, menubar_rects_before[i]);
        }

        let panel_b_node = tree.get(panel_b_id).expect("panel_b exists");
        assert_eq!(panel_b_node.children, panel_b_btn_ids);
        for (i, id) in panel_b_btn_ids.iter().enumerate() {
            let rect = tree.get(*id).unwrap().computed_rect;
            assert_eq!(rect, panel_b_rects_before[i]);
        }
    }

    #[test]
    fn test_rebuild_leaf_subtree_handles_invalid_boundary() {
        let mut tree = UiTree::new();
        let mut layout_engine = LayoutEngine::new();
        let mut text_system = TextSystem::new();

        // Non-existent widget id
        let invalid_id = WidgetId::default();
        let result = rebuild_leaf_subtree(
            &mut tree,
            &mut layout_engine,
            &mut text_system,
            invalid_id,
            |scope| {
                scope.add(Button::new("InvalidBoundaryFallback").tag(901));
            },
        );
        assert!(
            result.is_err(),
            "Non-existent boundary node must return Err"
        );

        // Zero-dimension node
        let root = tree.create_root().unwrap();
        UiScope::new(&mut tree, root).finish_layout(Rect::new(0.0, 0.0, 0.0, 0.0));
        let result = rebuild_leaf_subtree(
            &mut tree,
            &mut layout_engine,
            &mut text_system,
            root,
            |scope| {
                scope.add(Button::new("ZeroDimensionFallback").tag(902));
            },
        );
        assert!(
            result.is_err(),
            "Zero-dimension boundary node must return Err"
        );
    }

    #[test]
    fn test_subtree_session_rebuild_preserves_container_style_and_geometry() {
        use iris_core::{Color, Style};

        let mut tree = UiTree::new();
        let mut layout_engine = LayoutEngine::new();
        let mut text_system = TextSystem::new();

        let root = tree.create_root().expect("root widget node must exist");
        let initial_rect = Rect::new(100.0, 50.0, 320.0, 480.0);
        let custom_bg = Color::from_u8(25, 25, 30, 255);

        let mut scope = UiScope::new(&mut tree, root);
        let res = scope.container(
            Style::new()
                .width(320.0)
                .height(480.0)
                .background(custom_bg)
                .flex_col(),
            |s| {
                s.label("Old Content");
            },
        );
        let leaf = res.response.id;

        UiScope::new(&mut tree, leaf).finish_layout(initial_rect);

        SubtreeSession::rebuild(&mut tree, &mut layout_engine, &mut text_system, leaf, |s| {
            s.label("Fresh Content 1");
            s.label("Fresh Content 2");
        })
        .expect("subtree rebuild via SubtreeSession::rebuild");

        let leaf_node_after = tree.get(leaf).expect("leaf node persists");
        assert_eq!(leaf_node_after.computed_rect, initial_rect);
        assert_eq!(leaf_node_after.style.background_color, custom_bg);
    }
}