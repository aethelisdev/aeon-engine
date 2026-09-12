// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dockable panel abstraction and registry subsystem for Iris UI.
//!
//! Provides the [`DockPanel`] trait and [`PanelRegistry`] collection for hosting,
//! querying, downcasting, and rendering independent UI panes into docking layouts.

use iris_core::{Rect, UiEvent, UiTree, WidgetId};
use std::any::Any;
use std::collections::HashMap;

/// Lifecycle, interaction, and rendering interface for a dockable user interface panel.
/// Implementors define their unique identifier, user-facing title, and UI reconstruction logic.
/// Panels are registered with [`PanelRegistry`] and rendered into dockable host panes.
pub trait DockPanel: Send + Sync + 'static {
    /// Returns the unique alphanumeric string identifier for this panel.
    /// This identifier must remain stable across frames for layout persistence and tab matching.
    fn id(&self) -> &str;

    /// Returns the user-facing title rendered on the tab header for this panel.
    fn title(&self) -> &str;

    /// Renders the panel contents into the specified [`UiTree`] under `parent`.
    /// The `bounds` parameter defines the available physical rectangle allocated to this panel
    /// by the docking layout engine.
    fn render(&mut self, tree: &mut UiTree, parent: WidgetId, bounds: Rect);

    /// Determines whether the panel requires a UI rebuild on the current frame.
    /// Implementations that rely on continuous telemetry, animation, or external events
    /// can return `true`. Static or retained panels can return `false` when unmodified.
    /// Defaults to `true`.
    fn is_dirty(&self) -> bool {
        true
    }

    /// Optional callback to handle interactive UI events targeted at this panel.
    /// Returns `true` if the event was consumed and should not propagate further.
    /// Defaults to `false`.
    fn on_event(&mut self, _event: &UiEvent) -> bool {
        false
    }

    /// Returns this panel as a `&dyn Any` reference for downcasting to concrete types.
    fn as_any(&self) -> &dyn Any;

    /// Returns this panel as a `&mut dyn Any` reference for downcasting to concrete types.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Central registry and lifecycle manager for dockable panels.
/// Maintains a collection of [`DockPanel`] instances indexed by their unique string identifiers.
/// Preserves insertion order while providing $O(1)$ fast lookup and type-safe downcasting.
#[derive(Default)]
pub struct PanelRegistry {
    /// Order-preserving panel storage.
    panels: Vec<Box<dyn DockPanel>>,
    /// Fast identifier lookup mapping panel IDs to vector indices.
    id_to_index: HashMap<String, usize>,
}

impl PanelRegistry {
    /// Initializes an empty panel registry with zero pre-allocated panel slots.
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
            id_to_index: HashMap::new(),
        }
    }

    /// Registers a new [`DockPanel`] instance, taking ownership.
    /// If a panel with the same identifier already exists, it is replaced in-place,
    /// preserving its position in the insertion order.
    pub fn register<P: DockPanel>(&mut self, panel: P) {
        self.register_boxed(Box::new(panel));
    }

    /// Registers a boxed [`DockPanel`] instance into the registry.
    pub fn register_boxed(&mut self, panel: Box<dyn DockPanel>) {
        let id = panel.id().to_string();
        if let Some(&index) = self.id_to_index.get(&id) {
            self.panels[index] = panel;
        } else {
            let index = self.panels.len();
            self.id_to_index.insert(id, index);
            self.panels.push(panel);
        }
    }

    /// Returns a reference to the panel with the specified identifier, if present.
    pub fn get(&self, id: &str) -> Option<&dyn DockPanel> {
        let &index = self.id_to_index.get(id)?;
        self.panels.get(index).map(|p| &**p)
    }

    /// Returns a mutable reference to the panel with the specified identifier, if present.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut (dyn DockPanel + 'static)> {
        let &index = self.id_to_index.get(id)?;
        self.panels.get_mut(index).map(|p| &mut **p)
    }

    /// Attempts to retrieve and downcast an immutable reference to a concrete panel type.
    /// Returns `None` if the panel does not exist or if the requested type does not match.
    pub fn get_downcast<T: 'static>(&self, id: &str) -> Option<&T> {
        self.get(id)?.as_any().downcast_ref::<T>()
    }

    /// Attempts to retrieve and downcast a mutable reference to a concrete panel type.
    /// Returns `None` if the panel does not exist or if the requested type does not match.
    pub fn get_downcast_mut<T: 'static>(&mut self, id: &str) -> Option<&mut T> {
        self.get_mut(id)?.as_any_mut().downcast_mut::<T>()
    }

    /// Checks whether a panel with the given identifier is registered.
    pub fn contains(&self, id: &str) -> bool {
        self.id_to_index.contains_key(id)
    }

    /// Returns the total number of registered panels.
    pub fn len(&self) -> usize {
        self.panels.len()
    }

    /// Returns `true` if no panels are registered.
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty()
    }

    /// Returns an iterator over immutable references to all registered panels in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &dyn DockPanel> {
        self.panels.iter().map(|p| &**p)
    }

    /// Returns an iterator over mutable references to all boxed panels in insertion order.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn DockPanel>> {
        self.panels.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPanel {
        panel_id: String,
        panel_title: String,
        render_count: usize,
        last_bounds: Option<Rect>,
    }

    impl MockPanel {
        fn new(id: &str, title: &str) -> Self {
            Self {
                panel_id: id.to_string(),
                panel_title: title.to_string(),
                render_count: 0,
                last_bounds: None,
            }
        }
    }

    impl DockPanel for MockPanel {
        fn id(&self) -> &str {
            &self.panel_id
        }

        fn title(&self) -> &str {
            &self.panel_title
        }

        fn render(&mut self, tree: &mut UiTree, parent: WidgetId, bounds: Rect) {
            self.render_count += 1;
            self.last_bounds = Some(bounds);

            let child = tree.create_node();
            let _ = tree.add_child(parent, child);
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn test_panel_registration_and_retrieval() {
        let mut registry = PanelRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register(MockPanel::new("hierarchy", "Scene Hierarchy"));
        registry.register(MockPanel::new("stats", "Engine Stats"));

        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());
        assert!(registry.contains("hierarchy"));
        assert!(registry.contains("stats"));
        assert!(!registry.contains("inspector"));

        let panel = registry.get("hierarchy").expect("Panel must exist");
        assert_eq!(panel.id(), "hierarchy");
        assert_eq!(panel.title(), "Scene Hierarchy");
        assert!(panel.is_dirty());
    }

    #[test]
    fn test_panel_downcasting() {
        let mut registry = PanelRegistry::new();
        registry.register(MockPanel::new("mock", "Test Mock"));

        // Immutable downcast
        let mock_ref = registry.get_downcast::<MockPanel>("mock");
        assert!(mock_ref.is_some());
        assert_eq!(mock_ref.unwrap().render_count, 0);

        // Mutable downcast
        let mock_mut = registry.get_downcast_mut::<MockPanel>("mock");
        assert!(mock_mut.is_some());
        mock_mut.unwrap().render_count = 42;

        let recheck = registry.get_downcast::<MockPanel>("mock").unwrap();
        assert_eq!(recheck.render_count, 42);
    }

    #[test]
    fn test_panel_render_invocation() {
        let mut registry = PanelRegistry::new();
        registry.register(MockPanel::new("viewport", "3D Viewport"));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root creation failed");
        let bounds = Rect::new(10.0, 20.0, 800.0, 600.0);

        let panel = registry.get_mut("viewport").expect("Panel must exist");
        panel.render(&mut tree, root, bounds);

        let mock = registry.get_downcast::<MockPanel>("viewport").unwrap();
        assert_eq!(mock.render_count, 1);
        assert_eq!(mock.last_bounds, Some(bounds));
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_duplicate_registration_overwrites_in_place() {
        let mut registry = PanelRegistry::new();
        registry.register(MockPanel::new("console", "Console v1"));
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.get("console").unwrap().title(), "Console v1");

        // Overwrite
        registry.register(MockPanel::new("console", "Console v2"));
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.get("console").unwrap().title(), "Console v2");
    }
}