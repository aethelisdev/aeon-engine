// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Change detection, event notification, and selective invalidation engine for Iris UI.
//!
//! Provides [`UiNotifier`] to tag specific panels or the entire workbench as dirty,
//! enabling coarse-grained, retained selective redraws with zero per-frame CPU waste when idle.

use std::collections::HashSet;
use std::hash::Hash;

/// Selective invalidation and redraw notification engine.
///
/// Tracks dirty flags on a per-panel basis as well as global workbench-level layout dirty state.
/// When no panels are marked dirty, the UI pipeline remains completely asleep, bypassing
/// layout tree reconstruction and avoiding flickering or unwanted panel erasure.
///
/// Generic over the panel identifier type `P: Eq + Hash + Copy + Send + Sync + 'static`,
/// guaranteeing strongly-typed, zero-heap-allocation per-frame selective panel invalidation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiNotifier<P: Eq + Hash + Copy + Send + Sync + 'static> {
    /// Set of panel identifiers that have requested a redraw on the current frame.
    dirty_panels: HashSet<P>,
    /// Global invalidation flag indicating that the entire UI shell, window size, or dock layout changed.
    global_dirty: bool,
    /// Registered domain-bound panel identifiers that consume external model or entity data.
    domain_panels: HashSet<P>,
    /// Monotonic revision counter of domain transactions, enabling $O(1)$ synchronization.
    domain_revision: u64,
    /// Whether dirty panel tags automatically escalate to global dirty state in standalone environments.
    auto_propagate_panels: bool,
    /// Tracks pending dirty panel invalidations that were recorded after layout evaluation in the active frame.
    unconsumed_dirty: bool,
}

impl<P: Eq + Hash + Copy + Send + Sync + 'static> Default for UiNotifier<P> {
    fn default() -> Self {
        Self {
            dirty_panels: HashSet::new(),
            global_dirty: true,
            domain_panels: HashSet::new(),
            domain_revision: 0,
            auto_propagate_panels: true,
            unconsumed_dirty: false,
        }
    }
}

impl<P: Eq + Hash + Copy + Send + Sync + 'static> UiNotifier<P> {
    /// Initializes an empty UI notifier with zero dirty panels and global dirty state set to `true`
    /// to guarantee an initial first-frame full bake.
    ///
    /// Enables `auto_propagate_panels` by default to ensure standalone or transition environments
    /// consistently trigger layout passes when individual panels request redrawing.
    pub fn new() -> Self {
        Self {
            dirty_panels: HashSet::new(),
            global_dirty: true,
            domain_panels: HashSet::new(),
            domain_revision: 0,
            auto_propagate_panels: true,
            unconsumed_dirty: false,
        }
    }

    /// Initializes an empty UI notifier in a completely clean/sleeping state.
    ///
    /// Disables `auto_propagate_panels` to allow precise granular invalidation testing.
    pub fn clean() -> Self {
        Self {
            dirty_panels: HashSet::new(),
            global_dirty: false,
            domain_panels: HashSet::new(),
            domain_revision: 0,
            auto_propagate_panels: false,
            unconsumed_dirty: false,
        }
    }

    /// Configures whether individual dirty panel tags automatically escalate to global dirty state.
    ///
    /// When enabled, any invocation of [`Self::tag_redraw`], [`Self::sync_domain_revision`],
    /// or [`Self::notify_domain_mutation`] sets [`Self::global_dirty`] to `true`.
    #[inline]
    pub fn set_auto_propagate_panels(&mut self, enabled: bool) {
        self.auto_propagate_panels = enabled;
    }

    /// Builder method configuring automatic panel invalidation escalation to global layout dirty state.
    #[inline]
    #[must_use]
    pub fn with_auto_propagate_panels(mut self, enabled: bool) -> Self {
        self.auto_propagate_panels = enabled;
        self
    }

    /// Tags a specific panel for a selective redraw on the next frame.
    ///
    /// Only the specified panel's contents will be reconstructed, leaving other panels and the UI shell untouched.
    /// When `auto_propagate_panels` is enabled, also sets global dirty and tracks unconsumed invalidation
    /// to guarantee execution across frame boundaries.
    /// Passes the panel identifier `panel_id` by copy without heap allocation.
    pub fn tag_redraw(&mut self, panel_id: P) {
        self.dirty_panels.insert(panel_id);
        if self.auto_propagate_panels {
            self.unconsumed_dirty = true;
        }
    }

    /// Tags the entire UI shell and all panels for a full global reconstruction.
    ///
    /// Should be called upon window resize, display scaling / DPI change, or dock tree structural modifications.
    pub fn tag_all(&mut self) {
        self.global_dirty = true;
    }

    /// Queries whether a specific panel has been marked dirty for redraw.
    ///
    /// If [`Self::is_global_dirty`] is `true`, this method always returns `true` because
    /// a global invalidation rebuilds all panels.
    pub fn is_dirty(&self, panel_id: P) -> bool {
        self.global_dirty || self.dirty_panels.contains(&panel_id)
    }

    /// Returns `true` if any panel or the global UI shell requires a redraw on this frame.
    ///
    /// When this returns `false`, the UI coordinator can enter a zero-cost sleep state,
    /// directly replaying the existing GPU command stream without any tree allocations.
    pub fn is_any_dirty(&self) -> bool {
        self.global_dirty || !self.dirty_panels.is_empty()
    }

    /// Returns `true` if the application shell itself (menubar, statusbar, window chrome) is marked dirty via [`Self::tag_all`].
    ///
    /// Distinct from [`Self::is_global_dirty`] which includes escalated panel dirty states in fallback modes.
    #[inline]
    #[must_use]
    pub fn is_shell_dirty(&self) -> bool {
        self.global_dirty
    }

    /// Returns `true` if the global layout is dirty (requiring a full shell and panel reconstruction).
    ///
    /// When `auto_propagate_panels` is active, also returns `true` whenever any panel is dirty.
    #[inline]
    #[must_use]
    pub fn is_global_dirty(&self) -> bool {
        self.global_dirty || (self.auto_propagate_panels && !self.dirty_panels.is_empty())
    }

    /// Clears the dirty flag for an individual panel once its redraw pass has completed.
    pub fn clear_panel(&mut self, panel_id: P) {
        self.dirty_panels.remove(&panel_id);
        if self.dirty_panels.is_empty() {
            self.unconsumed_dirty = false;
        }
    }

    /// Clears all dirty flags, returning the notifier to a clean sleep state.
    ///
    /// If unconsumed panel invalidations were registered after layout evaluation within the active frame,
    /// preserves dirty state so the subsequent frame executes a complete update cycle.
    pub fn clear_all(&mut self) {
        if self.auto_propagate_panels && self.unconsumed_dirty {
            self.unconsumed_dirty = false;
        } else {
            self.dirty_panels.clear();
            self.global_dirty = false;
            self.unconsumed_dirty = false;
        }
    }

    /// Returns an immutable reference to the set of currently dirty panel identifiers.
    pub fn dirty_panels(&self) -> &HashSet<P> {
        &self.dirty_panels
    }

    /// Polls all panels registered in a [`crate::panel::PanelRegistry`] and marks any panel
    /// reporting [`crate::panel::DockPanel::is_dirty`] as requiring a redraw.
    ///
    /// The `mapper` closure resolves each panel's string identifier to the strongly-typed panel key `P`.
    /// Registers a panel identifier as a domain data subscriber.
    ///
    /// Domain-bound panels automatically receive selective redraw invalidations whenever
    /// [`Self::notify_domain_mutation`] is invoked or [`Self::sync_domain_revision`] detects
    /// an external transaction revision delta (such as Undo/Redo operations).
    pub fn register_domain_panel(&mut self, panel_id: P) {
        self.domain_panels.insert(panel_id);
    }

    /// Registers multiple panel identifiers as domain data subscribers in a single call.
    pub fn register_domain_panels(&mut self, panels: impl IntoIterator<Item = P>) {
        for panel in panels {
            self.domain_panels.insert(panel);
        }
    }

    /// Unregisters a panel identifier from domain data subscription.
    ///
    /// Returns `true` if the panel was previously registered as a domain panel.
    pub fn unregister_domain_panel(&mut self, panel_id: &P) -> bool {
        self.domain_panels.remove(panel_id)
    }

    /// Returns `true` if the specified panel is registered as a domain data subscriber.
    #[must_use]
    pub fn is_domain_panel(&self, panel_id: &P) -> bool {
        self.domain_panels.contains(panel_id)
    }

    /// Returns an immutable reference to the set of registered domain data subscriber panels.
    #[must_use]
    pub fn domain_panels(&self) -> &HashSet<P> {
        &self.domain_panels
    }

    /// Returns the current monotonic domain revision counter tracked by this notifier.
    #[must_use]
    pub const fn domain_revision(&self) -> u64 {
        self.domain_revision
    }

    /// Flags all registered domain-bound panels for selective redraw on the next frame.
    ///
    /// Also increments the internal domain transaction revision counter via wrapping arithmetic.
    /// Desktop IDEs call this method directly whenever an Undo/Redo command or external entity
    /// transaction executes, guaranteeing that retained property inputs immediately refresh.
    pub fn notify_domain_mutation(&mut self) {
        self.domain_revision = self.domain_revision.wrapping_add(1);
        for &panel in &self.domain_panels {
            self.dirty_panels.insert(panel);
        }
        if self.auto_propagate_panels {
            self.unconsumed_dirty = true;
        }
    }

    /// Synchronizes this notifier with an external monotonic transaction revision counter.
    ///
    /// If `current_revision` differs from the tracked [`Self::domain_revision`], updates the internal
    /// revision and automatically marks all registered [`Self::domain_panels`] for selective redraw.
    /// Returns `true` if a revision mismatch triggered invalidations, or `false` if unchanged ($O(1)$ fast path).
    ///
    /// # Performance
    /// When idle or during consecutive un-mutated frames, this performs a single scalar comparison
    /// without heap allocation or collection iteration.
    pub fn sync_domain_revision(&mut self, current_revision: u64) -> bool {
        if self.domain_revision != current_revision {
            self.domain_revision = current_revision;
            for &panel in &self.domain_panels {
                self.dirty_panels.insert(panel);
            }
            if self.auto_propagate_panels {
                self.unconsumed_dirty = true;
            }
            true
        } else {
            false
        }
    }

    /// Polls all panels registered in a [`crate::panel::PanelRegistry`] and marks any panel
    /// reporting [`crate::panel::DockPanel::is_dirty`] as requiring a redraw.
    ///
    /// The `mapper` closure resolves each panel's string identifier to the strongly-typed panel key `P`.
    pub fn poll_registry<F>(&mut self, registry: &crate::panel::PanelRegistry, mut mapper: F)
    where
        F: FnMut(&str) -> Option<P>,
    {
        for panel in registry.iter() {
            if panel.is_dirty()
                && let Some(p) = mapper(panel.id())
            {
                self.tag_redraw(p);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notifier_initial_state_is_globally_dirty_for_initial_bake() {
        let notifier: UiNotifier<&'static str> = UiNotifier::new();
        assert!(notifier.is_global_dirty());
        assert!(notifier.is_any_dirty());
        assert!(notifier.is_dirty("hierarchy"));
        assert!(notifier.is_dirty("stats"));
    }

    #[test]
    fn test_notifier_clean_initialization() {
        let notifier: UiNotifier<&'static str> = UiNotifier::clean();
        assert!(!notifier.is_global_dirty());
        assert!(!notifier.is_any_dirty());
        assert!(!notifier.is_dirty("inspector"));
        assert!(notifier.dirty_panels().is_empty());
    }

    #[test]
    fn test_notifier_tag_and_check_single_panel() {
        let mut notifier: UiNotifier<&'static str> = UiNotifier::clean();
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
        let mut notifier: UiNotifier<&'static str> = UiNotifier::clean();
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

        let mut notifier: UiNotifier<&'static str> = UiNotifier::clean();
        notifier.poll_registry(&registry, |id| match id {
            "clean_panel" => Some("clean_panel"),
            "active_panel" => Some("active_panel"),
            _ => None,
        });

        assert!(!notifier.is_dirty("clean_panel"));
        assert!(notifier.is_dirty("active_panel"));
        assert!(notifier.is_any_dirty());
    }

    #[test]
    fn test_notifier_domain_panel_registration_and_mutation() {
        let mut notifier: UiNotifier<&'static str> = UiNotifier::clean();
        notifier.register_domain_panel("inspector");
        notifier.register_domain_panels(["hierarchy", "materials"]);

        assert!(notifier.is_domain_panel(&"inspector"));
        assert!(notifier.is_domain_panel(&"hierarchy"));
        assert!(notifier.is_domain_panel(&"materials"));
        assert!(!notifier.is_domain_panel(&"console"));
        assert_eq!(notifier.domain_panels().len(), 3);

        // Initially clean
        assert!(!notifier.is_dirty("inspector"));
        assert!(!notifier.is_dirty("hierarchy"));
        assert_eq!(notifier.domain_revision(), 0);

        // Fire domain mutation (simulating Undo/Redo or transaction)
        notifier.notify_domain_mutation();
        assert_eq!(notifier.domain_revision(), 1);
        assert!(notifier.is_dirty("inspector"));
        assert!(notifier.is_dirty("hierarchy"));
        assert!(notifier.is_dirty("materials"));
        assert!(!notifier.is_dirty("console"));

        // Clear after frame
        notifier.clear_all();
        assert!(!notifier.is_dirty("inspector"));

        // Unregister one panel
        assert!(notifier.unregister_domain_panel(&"materials"));
        assert!(!notifier.is_domain_panel(&"materials"));

        // Second mutation only invalidates registered remaining panels
        notifier.notify_domain_mutation();
        assert_eq!(notifier.domain_revision(), 2);
        assert!(notifier.is_dirty("inspector"));
        assert!(notifier.is_dirty("hierarchy"));
        assert!(!notifier.is_dirty("materials"));
    }

    #[test]
    fn test_notifier_domain_revision_synchronization() {
        let mut notifier: UiNotifier<&'static str> = UiNotifier::clean();
        notifier.register_domain_panel("inspector");

        // Sync with identical revision returns false (0.0% overhead, zero dirty)
        assert!(!notifier.sync_domain_revision(0));
        assert!(!notifier.is_dirty("inspector"));

        // Sync with incremented revision (e.g. Undo stack changed)
        assert!(notifier.sync_domain_revision(42));
        assert_eq!(notifier.domain_revision(), 42);
        assert!(notifier.is_dirty("inspector"));

        notifier.clear_panel("inspector");
        assert!(!notifier.is_dirty("inspector"));

        // Syncing with same revision again does not dirty
        assert!(!notifier.sync_domain_revision(42));
        assert!(!notifier.is_dirty("inspector"));
    }

    #[test]
    fn test_notifier_auto_propagate_panels_triggers_global_dirty() {
        let mut notifier: UiNotifier<&'static str> =
            UiNotifier::clean().with_auto_propagate_panels(true);
        assert!(!notifier.is_global_dirty());

        notifier.tag_redraw("inspector");
        assert!(notifier.is_global_dirty());
        assert!(notifier.is_any_dirty());
        assert!(notifier.is_dirty("inspector"));

        notifier.clear_all();
        // Since unconsumed_dirty was set when tag_redraw was called, first clear_all keeps global_dirty
        assert!(notifier.is_global_dirty());

        // Subsequent clear_all after bake resets cleanly
        notifier.clear_all();
        assert!(!notifier.is_global_dirty());
        assert!(!notifier.is_any_dirty());
    }

    #[test]
    fn test_notifier_domain_revision_auto_propagation() {
        let mut notifier: UiNotifier<&'static str> = UiNotifier::clean();
        notifier.set_auto_propagate_panels(true);
        notifier.register_domain_panel("hierarchy");

        assert!(!notifier.is_global_dirty());
        assert!(notifier.sync_domain_revision(100));
        assert!(notifier.is_global_dirty());
        assert!(notifier.is_dirty("hierarchy"));
    }
}