// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dockable panel descriptor, lifecycle binding, and registry creation for the Iris UI docking system.
//!
//! Provides [`EditorDockPanel`] to bridge strongly-typed [`PanelId`] definitions into the generic
//! [`DockPanel`] interface, supporting localized dirty state tracking, custom rendering callbacks,
//! and registry-driven redraw polling.
//!

use crate::ui::panel_layout::PanelId;
use irisui::prelude::*;

/// Thread-safe closure delegate evaluating whether a dock panel is dirty.
pub type PanelDirtyDelegate = std::sync::Arc<dyn Fn() -> bool + Send + Sync>;

/// Thread-safe closure callback executing UI tree reconstruction for a dock panel.
pub type PanelRenderCallback = std::sync::Arc<dyn Fn(&mut UiTree, WidgetId, Rect) + Send + Sync>;

/// Result of evaluating a panel's retained lifecycle update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelLifecycleResult {
    /// Panel was marked dirty; UI tree was cleared and fully reconstructed.
    Rebuilt,
    /// Panel was clean but dock bounds changed; performed zero-allocation in-place flexbox relayout.
    RelayoutInPlace,
    /// Panel was clean and dock bounds unchanged; zero CPU work performed (sleeping).
    Unchanged,
}

/// Standard dockable panel implementor for built-in editor panels registered in [`PanelRegistry`].
///
/// Wraps a strongly typed [`crate::ui::panel_layout::PanelId`] and provides metadata,
/// localized dirty tracking, custom rendering callbacks, an independently retained [`UiTree`],
/// and zero-allocation in-place layout updates for docking split resize events.
pub struct EditorDockPanel {
    id: String,
    title: String,
    panel_id: PanelId,
    dirty: std::sync::atomic::AtomicBool,
    dirty_delegate: Option<PanelDirtyDelegate>,
    render_callback: Option<PanelRenderCallback>,
    tree: UiTree,
    last_bounds: Option<Rect>,
}

impl std::fmt::Debug for EditorDockPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorDockPanel")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("panel_id", &self.panel_id)
            .field("is_dirty", &self.is_dirty())
            .field("has_dirty_delegate", &self.dirty_delegate.is_some())
            .field("has_render_callback", &self.render_callback.is_some())
            .field("tree_nodes", &self.tree.len())
            .field("last_bounds", &self.last_bounds)
            .finish()
    }
}

impl EditorDockPanel {
    /// Creates a new editor panel descriptor for the given [`crate::ui::panel_layout::PanelId`].
    ///
    /// Initialized with clean dirty status (`false`), empty delegation callbacks, and a fresh
    /// independent [`UiTree`] and [`LayoutEngine`].
    pub fn new(panel_id: PanelId) -> Self {
        Self {
            id: panel_id.id_str().to_string(),
            title: panel_id.title().to_string(),
            panel_id,
            dirty: std::sync::atomic::AtomicBool::new(false),
            dirty_delegate: None,
            render_callback: None,
            tree: UiTree::new(),
            last_bounds: None,
        }
    }

    /// Returns the associated [`crate::ui::panel_layout::PanelId`].
    pub fn panel_id(&self) -> PanelId {
        self.panel_id
    }

    /// Returns an immutable reference to the independently retained [`UiTree`] owned by this panel.
    pub fn tree(&self) -> &UiTree {
        &self.tree
    }

    /// Returns a mutable reference to the independently retained [`UiTree`] owned by this panel.
    pub fn tree_mut(&mut self) -> &mut UiTree {
        &mut self.tree
    }

    /// Returns the last allocated physical bounds rectangle for this panel.
    pub fn last_bounds(&self) -> Option<Rect> {
        self.last_bounds
    }

    /// Sets or updates the cached physical bounds allocated to this panel.
    pub fn set_last_bounds(&mut self, bounds: Rect) {
        self.last_bounds = Some(bounds);
    }

    /// Clears the cached bounds rectangle when the panel becomes inactive or hidden in dock.
    ///
    /// Ensures subsequent compositing passes and typography collection ignore this panel,
    /// preventing overlapping phantom render passes across tab switches.
    pub fn clear_bounds(&mut self) {
        self.last_bounds = None;
    }

    /// Clears the retained UI tree state when invalidation occurs.
    ///
    /// Resets the internal arena and frees widget slots, preparing the panel for
    /// a clean reconstruction without lingering dirty states or memory leaks.
    pub fn clear_tree(&mut self) {
        self.tree.clear();
    }

    /// Updates root container constraints and recomputes flexbox layout in-place without node allocation.
    ///
    /// When dock splitters resize the panel, this method updates the root node's width and height
    /// styles and executes [`LayoutEngine::compute_layout`] on the existing arena. Returns `true`
    /// if in-place relayout succeeded, or `false` if the tree has no root node.
    pub fn update_layout(&mut self, layout_engine: &mut LayoutEngine, new_bounds: Rect) -> bool {
        let Some(root) = self.tree.root() else {
            return false;
        };

        let mut scope = UiScope::new(&mut self.tree, root);
        scope.configure_container(
            &self.title,
            Style::new()
                .width(new_bounds.width)
                .height(new_bounds.height),
            false,
        );

        let layout_result = layout_engine.compute_layout(
            &mut self.tree,
            Size::new(new_bounds.width, new_bounds.height),
        );

        if layout_result.is_ok() {
            let mut finish_scope = UiScope::new(&mut self.tree, root);
            finish_scope.finish_layout(Rect::new(0.0, 0.0, new_bounds.width, new_bounds.height));
            self.last_bounds = Some(new_bounds);
            true
        } else {
            false
        }
    }

    /// Evaluates and advances the panel's retained lifecycle contract.
    ///
    /// 1. If [`Self::is_dirty`] is `true`: records the target bounds. If a custom
    ///    [`Self::render_callback`] is configured, reconstructs its subtree and clears dirty.
    ///    For standard engine panels, leaves tree generation and dirty clearance exclusively to
    ///    `render_dock_panel_into_tree`. Returns [`PanelLifecycleResult::Rebuilt`].
    /// 2. If clean but `last_bounds != Some(new_bounds)`: executes [`Self::update_layout`]
    ///    in-place with zero widget node allocations. Returns [`PanelLifecycleResult::RelayoutInPlace`].
    /// 3. If clean and bounds are identical: skips execution with zero CPU/GPU overhead.
    ///    Returns [`PanelLifecycleResult::Unchanged`].
    pub fn update_lifecycle(
        &mut self,
        layout_engine: &mut LayoutEngine,
        new_bounds: Rect,
    ) -> PanelLifecycleResult {
        if self.is_dirty() {
            self.last_bounds = Some(new_bounds);

            let callback_opt = self.render_callback.as_ref().map(std::sync::Arc::clone);
            if let Some(callback) = callback_opt {
                self.clear_tree();
                self.clear_dirty();
                if let Ok(root) = self.tree.create_root() {
                    callback(&mut self.tree, root, new_bounds);
                    let _ = layout_engine.compute_layout(
                        &mut self.tree,
                        Size::new(new_bounds.width, new_bounds.height),
                    );
                    let mut finish_scope = UiScope::new(&mut self.tree, root);
                    finish_scope.finish_layout(Rect::new(
                        0.0,
                        0.0,
                        new_bounds.width,
                        new_bounds.height,
                    ));
                }
            }

            PanelLifecycleResult::Rebuilt
        } else if self.last_bounds != Some(new_bounds) {
            if self.tree.root().is_some() && self.update_layout(layout_engine, new_bounds) {
                PanelLifecycleResult::RelayoutInPlace
            } else {
                self.set_dirty(true);
                self.last_bounds = Some(new_bounds);
                PanelLifecycleResult::Rebuilt
            }
        } else {
            PanelLifecycleResult::Unchanged
        }
    }

    /// Sets whether this dock panel is marked dirty for the next redraw cycle.
    pub fn set_dirty(&self, dirty: bool) {
        self.dirty
            .store(dirty, std::sync::atomic::Ordering::Relaxed);
    }

    /// Marks the panel as dirty, requesting a redraw on the next frame.
    pub fn mark_dirty(&self) {
        self.set_dirty(true);
    }

    /// Clears the dirty flag on this panel, resetting it to a clean state.
    pub fn clear_dirty(&self) {
        self.set_dirty(false);
    }

    /// Performs a hardware hit-test query on this panel's retained UI tree using (0, 0) local coordinates.
    pub fn hit_test_local(&self, local_point: Point) -> Option<HitTargetInfo> {
        self.tree.hit_test_target(local_point)
    }

    /// Resolves the effective 64-bit semantic tag from an ancestor widget node in this panel's tree.
    pub fn resolve_ancestor_tag(&self, id: WidgetId) -> u64 {
        self.tree.resolve_ancestor_tag(id)
    }

    /// Configures an optional closure or state delegate to evaluate whether the panel is dirty.
    ///
    /// When specified, [`DockPanel::is_dirty`] delegates to this callback rather than the internal atomic flag.
    pub fn with_dirty_delegate<F>(mut self, delegate: F) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        self.dirty_delegate = Some(std::sync::Arc::new(delegate));
        self
    }

    /// Configures an optional custom rendering callback invoked when [`DockPanel::render`] is called.
    pub fn with_render_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UiTree, WidgetId, Rect) + Send + Sync + 'static,
    {
        self.render_callback = Some(std::sync::Arc::new(callback));
        self
    }
}

impl DockPanel for EditorDockPanel {
    fn id(&self) -> &str {
        &self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn render(&mut self, tree: &mut UiTree, parent: WidgetId, bounds: Rect) {
        if let Some(ref callback) = self.render_callback {
            callback(tree, parent, bounds);
        } else {
            let mut scope = UiScope::new(tree, parent);
            scope.container(Style::default(), |inner| {
                inner.label(&self.title, 13.0, Color::WHITE, TextAlign::Left);
            });
        }
    }

    fn is_dirty(&self) -> bool {
        if let Some(ref delegate) = self.dirty_delegate {
            delegate()
        } else {
            self.dirty.load(std::sync::atomic::Ordering::Relaxed)
        }
    }

    fn tree(&self) -> Option<&UiTree> {
        Some(&self.tree)
    }

    fn tree_mut(&mut self) -> Option<&mut UiTree> {
        Some(&mut self.tree)
    }

    fn bounds(&self) -> Option<Rect> {
        self.last_bounds
    }

    fn update_layout(&mut self, new_bounds: Rect) -> bool {
        let mut engine = LayoutEngine::new();
        self.update_layout(&mut engine, new_bounds)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Constructs the standard [`PanelRegistry`] populated with all built-in editor panels.
pub fn create_default_panel_registry() -> PanelRegistry {
    let mut registry = PanelRegistry::default();
    for &panel_id in PanelId::all() {
        registry.register(EditorDockPanel::new(panel_id));
    }
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_panel_registry_contains_all_panels() {
        let registry = create_default_panel_registry();
        assert_eq!(registry.len(), PanelId::all().len());

        for &panel_id in PanelId::all() {
            let id = panel_id.id_str();
            assert!(registry.contains(id));

            let panel = registry.get(id).expect("panel should exist");
            assert_eq!(panel.id(), id);
            assert_eq!(panel.title(), panel_id.title());

            let downcast = registry
                .get_downcast::<EditorDockPanel>(id)
                .expect("should downcast to EditorDockPanel");
            assert_eq!(downcast.panel_id(), panel_id);
        }
    }

    #[test]
    fn test_editor_dock_panel_render_and_events() {
        let mut panel = EditorDockPanel::new(PanelId::Inspector);
        let mut tree = UiTree::new();
        let parent = tree.create_root().expect("root widget");
        panel.render(&mut tree, parent, Rect::new(0.0, 0.0, 100.0, 100.0));
        assert!(tree.len() >= 2);
        assert!(!panel.is_dirty());

        panel.set_dirty(true);
        assert!(panel.is_dirty());

        panel.clear_dirty();
        assert!(!panel.is_dirty());
    }

    #[test]
    fn test_editor_dock_panel_delegates() {
        let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag_clone = flag.clone();

        let panel = EditorDockPanel::new(PanelId::Hierarchy)
            .with_dirty_delegate(move || flag_clone.load(std::sync::atomic::Ordering::Relaxed));

        assert!(!panel.is_dirty());
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
        assert!(panel.is_dirty());
    }

    #[test]
    fn test_editor_dock_panel_lifecycle_rebuilt_on_dirty() {
        let mut panel = EditorDockPanel::new(PanelId::Inspector).with_render_callback(
            |tree, parent, bounds| {
                let mut scope = UiScope::new(tree, parent);
                scope.container(
                    Style::new().width(bounds.width).height(bounds.height),
                    |c| {
                        c.label("Inspector", 12.0, Color::WHITE, TextAlign::Left);
                    },
                );
            },
        );
        let mut engine = LayoutEngine::new();
        panel.mark_dirty();
        assert!(panel.is_dirty());

        let bounds1 = Rect::new(0.0, 0.0, 300.0, 500.0);
        let res1 = panel.update_lifecycle(&mut engine, bounds1);
        assert_eq!(res1, PanelLifecycleResult::Rebuilt);
        assert!(!panel.is_dirty());
        assert_eq!(panel.last_bounds(), Some(bounds1));
        assert!(panel.tree().root().is_some());
        let initial_node_count = panel.tree().len();
        assert!(initial_node_count >= 2);

        // Mark dirty again and rebuild with new content: node count should stay consistent, NOT accumulate!
        panel.mark_dirty();
        let res2 = panel.update_lifecycle(&mut engine, bounds1);
        assert_eq!(res2, PanelLifecycleResult::Rebuilt);
        assert_eq!(panel.tree().len(), initial_node_count);
    }

    #[test]
    fn test_editor_dock_panel_standard_panel_preserves_dirty_without_fake_nodes() {
        let mut panel = EditorDockPanel::new(PanelId::Hierarchy);
        let mut engine = LayoutEngine::new();
        panel.mark_dirty();
        assert!(panel.is_dirty());

        let bounds1 = Rect::new(0.0, 0.0, 250.0, 400.0);
        let res1 = panel.update_lifecycle(&mut engine, bounds1);
        assert_eq!(res1, PanelLifecycleResult::Rebuilt);
        // Crucial invariant: dirty flag must NOT be prematurely cleared, and no dummy nodes created!
        assert!(panel.is_dirty());
        assert_eq!(panel.last_bounds(), Some(bounds1));
        assert_eq!(panel.tree().len(), 0);
    }

    #[test]
    fn test_editor_dock_panel_lifecycle_relayout_in_place_on_resize() {
        let mut panel = EditorDockPanel::new(PanelId::Hierarchy).with_render_callback(
            |tree, parent, bounds| {
                let mut scope = UiScope::new(tree, parent);
                scope.container(
                    Style::new().width(bounds.width).height(bounds.height),
                    |c| {
                        c.label("Hierarchy Node", 12.0, Color::WHITE, TextAlign::Left);
                    },
                );
            },
        );
        let mut engine = LayoutEngine::new();
        panel.mark_dirty();

        let initial_bounds = Rect::new(0.0, 0.0, 250.0, 400.0);
        assert_eq!(
            panel.update_lifecycle(&mut engine, initial_bounds),
            PanelLifecycleResult::Rebuilt
        );
        let node_count_before = panel.tree().len();

        // Dock resize event: bounds change from 250 to 350 width.
        // Data is clean: must perform zero-allocation in-place layout without rebuilding nodes!
        let resized_bounds = Rect::new(0.0, 0.0, 350.0, 400.0);
        let resize_res = panel.update_lifecycle(&mut engine, resized_bounds);
        assert_eq!(resize_res, PanelLifecycleResult::RelayoutInPlace);
        assert_eq!(
            panel.tree().len(),
            node_count_before,
            "Node count must not change on resize!"
        );
        assert_eq!(panel.last_bounds(), Some(resized_bounds));

        // Root computed rect must reflect the new width
        let root = panel.tree().root().expect("root must exist");
        let root_rect = panel.tree().get(root).expect("root node").computed_rect;
        assert_eq!(root_rect.width, 350.0);
        assert_eq!(root_rect.height, 400.0);
    }

    #[test]
    fn test_editor_dock_panel_lifecycle_unchanged_when_clean() {
        let mut panel =
            EditorDockPanel::new(PanelId::Console).with_render_callback(|tree, parent, bounds| {
                let mut scope = UiScope::new(tree, parent);
                scope.container(
                    Style::new().width(bounds.width).height(bounds.height),
                    |c| {
                        c.label("Console Output", 12.0, Color::WHITE, TextAlign::Left);
                    },
                );
            });
        let mut engine = LayoutEngine::new();
        panel.mark_dirty();

        let bounds = Rect::new(0.0, 0.0, 500.0, 200.0);
        assert_eq!(
            panel.update_lifecycle(&mut engine, bounds),
            PanelLifecycleResult::Rebuilt
        );

        // Next frame: no dirty flag, same bounds -> must be Unchanged (0 CPU)
        assert_eq!(
            panel.update_lifecycle(&mut engine, bounds),
            PanelLifecycleResult::Unchanged
        );
    }

    #[test]
    fn test_editor_dock_panel_custom_render_callback_and_in_place_relayout() {
        let panel = EditorDockPanel::new(PanelId::UiDesigner).with_render_callback(
            |tree, parent, bounds| {
                let mut scope = UiScope::new(tree, parent);
                scope.container(
                    Style::new()
                        .width(bounds.width)
                        .height(bounds.height)
                        .flex_col(),
                    |col| {
                        col.empty_box(Style::new().width(100.0).height(40.0));
                    },
                );
            },
        );

        let mut panel = panel;
        let mut engine = LayoutEngine::new();
        panel.mark_dirty();

        let bounds1 = Rect::new(0.0, 0.0, 400.0, 300.0);
        assert_eq!(
            panel.update_lifecycle(&mut engine, bounds1),
            PanelLifecycleResult::Rebuilt
        );
        assert_eq!(panel.tree().len(), 3);

        // Resize
        let bounds2 = Rect::new(0.0, 0.0, 600.0, 450.0);
        assert_eq!(
            panel.update_lifecycle(&mut engine, bounds2),
            PanelLifecycleResult::RelayoutInPlace
        );
        assert_eq!(panel.tree().len(), 3);

        let root_node = panel.tree().get(panel.tree().root().unwrap()).unwrap();
        assert_eq!(root_node.computed_rect.width, 600.0);
        assert_eq!(root_node.computed_rect.height, 450.0);
    }
}