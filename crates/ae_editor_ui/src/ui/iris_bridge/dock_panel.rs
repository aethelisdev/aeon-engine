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

/// Standard dockable panel implementor for built-in editor panels registered in [`PanelRegistry`].
///
/// Wraps a strongly typed [`crate::ui::panel_layout::PanelId`] and provides metadata,
/// localized dirty tracking, custom rendering callbacks, and lifecycle integration with
/// the Iris UI docking framework.
pub struct EditorDockPanel {
    id: String,
    title: String,
    panel_id: PanelId,
    dirty: std::sync::atomic::AtomicBool,
    dirty_delegate: Option<PanelDirtyDelegate>,
    render_callback: Option<PanelRenderCallback>,
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
            .finish()
    }
}

impl EditorDockPanel {
    /// Creates a new editor panel descriptor for the given [`crate::ui::panel_layout::PanelId`].
    ///
    /// Initialized with clean dirty status (`false`) and empty delegation callbacks.
    pub fn new(panel_id: PanelId) -> Self {
        Self {
            id: panel_id.id_str().to_string(),
            title: panel_id.title().to_string(),
            panel_id,
            dirty: std::sync::atomic::AtomicBool::new(false),
            dirty_delegate: None,
            render_callback: None,
        }
    }

    /// Returns the associated [`crate::ui::panel_layout::PanelId`].
    pub fn panel_id(&self) -> PanelId {
        self.panel_id
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
}