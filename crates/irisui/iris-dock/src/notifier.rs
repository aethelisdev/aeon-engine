// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Change detection, event notification, and selective invalidation engine for Iris UI.
//!
//! Provides [`UiNotifier`] to tag specific panels or the entire workbench as dirty,
//! enabling coarse-grained, retained selective redraws with zero per-frame CPU waste when idle.

use std::collections::HashSet;

/// Selective invalidation and redraw notification engine.
/// Tracks dirty flags on a per-panel basis as well as global workbench-level layout dirty state.
/// When no panels are marked dirty, the UI pipeline remains completely asleep, bypassing
/// layout tree reconstruction and avoiding flickering or unwanted panel erasure.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiNotifier {
    /// Set of panel identifiers that have requested a redraw on the current frame.
    dirty_panels: HashSet<String>,
    /// Global invalidation flag indicating that the entire UI shell, window size, or dock layout changed.
    global_dirty: bool,
}

impl UiNotifier {
    /// Initializes an empty UI notifier with zero dirty panels and global dirty state set to `true`
    /// to guarantee an initial first-frame full bake.
    pub fn new() -> Self {
        Self {
            dirty_panels: HashSet::new(),
            global_dirty: true,
        }
    }

    /// Initializes an empty UI notifier in a completely clean/sleeping state.
    /// Useful in unit tests or when explicit invalidation triggers are required.
    pub fn clean() -> Self {
        Self {
            dirty_panels: HashSet::new(),
            global_dirty: false,
        }
    }

    /// Tags a specific panel for a selective redraw on the next frame.
    /// Only the specified panel's contents will be reconstructed, leaving other panels and the UI shell untouched.
    pub fn tag_redraw(&mut self, panel_id: &str) {
        self.dirty_panels.insert(panel_id.to_string());
    }

    /// Tags the entire UI shell and all panels for a full global reconstruction.
    /// Should be called upon window resize, display scaling / DPI change, or dock tree structural modifications.
    pub fn tag_all(&mut self) {
        self.global_dirty = true;
    }

    /// Queries whether a specific panel has been marked dirty for redraw.
    /// If [`Self::is_global_dirty`] is `true`, this method always returns `true` because
    /// a global invalidation rebuilds all panels.
    pub fn is_dirty(&self, panel_id: &str) -> bool {
        self.global_dirty || self.dirty_panels.contains(panel_id)
    }

    /// Returns `true` if any panel or the global UI shell requires a redraw on this frame.
    /// When this returns `false`, the UI coordinator can enter a zero-cost sleep state,
    /// directly replaying the existing GPU command stream without any tree allocations.
    pub fn is_any_dirty(&self) -> bool {
        self.global_dirty || !self.dirty_panels.is_empty()
    }

    /// Returns `true` if the global layout is dirty (requiring a full shell and panel reconstruction).
    pub fn is_global_dirty(&self) -> bool {
        self.global_dirty
    }

    /// Clears the dirty flag for an individual panel once its redraw pass has completed.
    pub fn clear_panel(&mut self, panel_id: &str) {
        self.dirty_panels.remove(panel_id);
    }

    /// Clears all dirty flags, returning the notifier to a clean sleep state.
    /// Typically invoked at the end of the frame once all dirty commands have been dispatched to the GPU.
    pub fn clear_all(&mut self) {
        self.dirty_panels.clear();
        self.global_dirty = false;
    }

    /// Returns an immutable reference to the set of currently dirty panel identifiers.
    pub fn dirty_panels(&self) -> &HashSet<String> {
        &self.dirty_panels
    }

    /// Polls all panels registered in a [`crate::panel::PanelRegistry`] and marks any panel
    /// reporting [`crate::panel::DockPanel::is_dirty`] as requiring a redraw.
    /// This allows self-contained panels (such as telemetries, charts, or animated widgets)
    /// to trigger reactive UI bakes based on their own internal state.
    pub fn poll_registry(&mut self, registry: &crate::panel::PanelRegistry) {
        for panel in registry.iter() {
            if panel.is_dirty() {
                self.tag_redraw(panel.id());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notifier_initial_state_is_globally_dirty_for_initial_bake() {
        let notifier = UiNotifier::new();
        assert!(notifier.is_global_dirty());
        assert!(notifier.is_any_dirty());
        assert!(notifier.is_dirty("hierarchy"));
        assert!(notifier.is_dirty("stats"));
    }

    #[test]
    fn test_notifier_clean_initialization() {
        let notifier = UiNotifier::clean();
        assert!(!notifier.is_global_dirty());
        assert!(!notifier.is_any_dirty());
        assert!(!notifier.is_dirty("inspector"));
        assert!(notifier.dirty_panels().is_empty());
    }

    #[test]
    fn test_notifier_tag_and_check_single_panel() {
        let mut notifier = UiNotifier::clean();
        assert!(!notifier.is_any_dirty());

        notifier.tag_redraw("stats");
        assert!(notifier.is_any_dirty());
        assert!(notifier.is_dirty("stats"));
        assert!(!notifier.is_dirty("inspector"));
        assert!(!notifier.is_global_dirty());
        assert_eq!(notifier.dirty_panels().len(), 1);

        notifier.clear_panel("stats");
        assert!(!notifier.is_any_dirty());
        assert!(!notifier.is_dirty("stats"));
    }

    #[test]
    fn test_notifier_tag_all_and_clear_all() {
        let mut notifier = UiNotifier::clean();
        notifier.tag_redraw("console");
        assert!(!notifier.is_global_dirty());

        notifier.tag_all();
        assert!(notifier.is_global_dirty());
        assert!(notifier.is_any_dirty());
        assert!(notifier.is_dirty("any_random_panel"));

        notifier.clear_all();
        assert!(!notifier.is_global_dirty());
        assert!(!notifier.is_any_dirty());
        assert!(!notifier.is_dirty("console"));
    }

    #[test]
    fn test_notifier_poll_registry() {
        use crate::panel::{DockPanel, PanelRegistry};
        use iris_core::{Rect, UiTree, WidgetId};
        use std::any::Any;

        struct TestPanel {
            id: String,
            dirty: bool,
        }

        impl DockPanel for TestPanel {
            fn id(&self) -> &str {
                &self.id
            }
            fn title(&self) -> &str {
                "Test"
            }
            fn render(&mut self, _tree: &mut UiTree, _parent: WidgetId, _bounds: Rect) {}
            fn is_dirty(&self) -> bool {
                self.dirty
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }

        let mut registry = PanelRegistry::default();
        registry.register(TestPanel {
            id: "clean_panel".to_string(),
            dirty: false,
        });
        registry.register(TestPanel {
            id: "active_panel".to_string(),
            dirty: true,
        });

        let mut notifier = UiNotifier::clean();
        notifier.poll_registry(&registry);

        assert!(!notifier.is_dirty("clean_panel"));
        assert!(notifier.is_dirty("active_panel"));
        assert!(notifier.is_any_dirty());
    }
}