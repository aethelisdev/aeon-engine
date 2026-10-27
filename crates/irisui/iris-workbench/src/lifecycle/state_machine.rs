// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 3-Branch Lifecycle State Machine
//!
//! Autonomous tracking of coarse layout changes, fine-grained panel invalidations,
//! and volatile draw streams to guarantee zero-jitter frame execution.

use std::collections::HashSet;
use std::hash::Hash;

use super::branch::LifecycleBranch;
use super::inputs::LifecycleInputs;

/// 3-Branch lifecycle state machine governing application shell frame reconstruction.
///
/// In modern desktop IDEs, frame execution must be dynamically partitioned into:
/// 1. **Full Shell Rebuild (Branch 1)**: Triggered only on genuine topology mutations
///    (window resizing, scale factor changes, dock tab docking/undocking, modal backdrop
///    state changes, or active splitter dragging).
/// 2. **Granular Subtree Invalidation (Branch 2)**: Partial in-place rebuild of dirty leaf
///    panels using isolated boundaries without modifying the root shell or sibling panels.
/// 3. **In-Place Volatile Draw Refresh (Branch 3)**: High-frequency telemetry and canvas
///    refreshes with zero retained tree mutations and zero layout dirty flags.
/// 4. **Dormant Steady-State**: Zero mutations, allowing the frame to hold steady.
///
/// # Generics
/// - `TabId`: The application's unique dock panel identifier type (e.g. `PanelId`).
#[derive(Debug, Clone)]
pub struct WorkbenchLifecycle<TabId: Eq + Hash + Clone> {
    /// Screen dimensions from the previous evaluation pass, or `None` on initial boot.
    last_dimensions: Option<(f32, f32)>,

    /// Monitor DPI scale factor from the previous evaluation pass.
    last_scale_factor: f32,

    /// Docking layout revision counter from the previous evaluation pass.
    last_dock_revision: u64,

    /// Number of floating windows from the previous evaluation pass.
    last_floating_count: usize,

    /// Whether any top menu was open during the previous evaluation pass.
    last_menu_active: bool,

    /// Whether any modal dialog was open during the previous evaluation pass.
    last_modal_active: bool,

    /// Whether any drag-and-drop operation was active during the previous evaluation pass.
    last_drag_active: bool,

    /// Explicit request to force a full shell reconstruction on the next turn.
    force_shell_rebuild: bool,

    /// Developer debug override to force full shell reconstruction every frame.
    always_rebuild_shell: bool,

    /// Set of panel identifiers that have been explicitly marked dirty for partial rebuild.
    dirty_tabs: HashSet<TabId>,
}

impl<TabId: Eq + Hash + Clone> Default for WorkbenchLifecycle<TabId> {
    fn default() -> Self {
        Self::new()
    }
}

impl<TabId: Eq + Hash + Clone> WorkbenchLifecycle<TabId> {
    /// Creates a new lifecycle state machine initialized for an initial cold start.
    ///
    /// The first evaluation pass is mathematically guaranteed to emit [`LifecycleBranch::FullShellRebuild`]
    /// because `last_dimensions` is initialized to `None`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_dimensions: None,
            last_scale_factor: 1.0,
            last_dock_revision: 0,
            last_floating_count: 0,
            last_menu_active: false,
            last_modal_active: false,
            last_drag_active: false,
            force_shell_rebuild: true,
            always_rebuild_shell: false,
            dirty_tabs: HashSet::new(),
        }
    }

    /// Explicitly flags the entire application shell as dirty, requiring a full reconstruction.
    ///
    /// Typically invoked on application theme switches, window visibility transitions,
    /// or complete project loading events.
    #[inline]
    pub fn mark_shell_dirty(&mut self) {
        self.force_shell_rebuild = true;
    }

    /// Marks a specific dock panel as dirty, requesting a granular partial subtree rebuild.
    ///
    /// If the shell itself is not dirty, only this panel (and any other concurrently marked
    /// panels) will be rebuilt in-place via Branch 2.
    #[inline]
    pub fn mark_tab_dirty(&mut self, tab: TabId) {
        self.dirty_tabs.insert(tab);
    }

    /// Marks multiple dock panels as dirty in a single batched operation.
    #[inline]
    pub fn mark_tabs_dirty(&mut self, tabs: impl IntoIterator<Item = TabId>) {
        self.dirty_tabs.extend(tabs);
    }

    /// Clears the dirty state for a specific panel.
    #[inline]
    pub fn clear_tab_dirty(&mut self, tab: &TabId) {
        self.dirty_tabs.remove(tab);
    }

    /// Clears all pending panel dirty marks.
    #[inline]
    pub fn clear_all_dirty_tabs(&mut self) {
        self.dirty_tabs.clear();
    }

    /// Checks whether a specific panel is currently marked as dirty.
    #[inline]
    #[must_use]
    pub fn is_tab_dirty(&self, tab: &TabId) -> bool {
        self.dirty_tabs.contains(tab)
    }

    /// Returns a reference to the set of currently dirty panel tabs.
    #[inline]
    #[must_use]
    pub const fn dirty_tabs(&self) -> &HashSet<TabId> {
        &self.dirty_tabs
    }

    /// Sets whether the shell must unconditionally rebuild on every frame (debug mode).
    #[inline]
    pub fn set_always_rebuild(&mut self, always: bool) {
        self.always_rebuild_shell = always;
    }

    /// Returns whether the shell is configured to unconditionally rebuild on every frame.
    #[inline]
    #[must_use]
    pub const fn is_always_rebuild(&self) -> bool {
        self.always_rebuild_shell
    }

    /// Resets the baseline state tracking, forcing a full reconstruction on the next turn.
    #[inline]
    pub fn reset_state(&mut self) {
        self.last_dimensions = None;
        self.last_dock_revision = 0;
        self.last_floating_count = 0;
        self.last_menu_active = false;
        self.last_modal_active = false;
        self.last_drag_active = false;
        self.force_shell_rebuild = true;
        self.dirty_tabs.clear();
    }

    /// Evaluates whether the application shell requires a full topology reconstruction.
    ///
    /// Checks structural invariants:
    /// - Explicit forced rebuild flags (`force_shell_rebuild` or `always_rebuild_shell`).
    /// - Initial cold start (`last_dimensions.is_none()`).
    /// - Viewport dimension alteration (`dimensions != last_dimensions`).
    /// - DPI scale factor change (`|scale_factor - last_scale_factor| > 1e-4`).
    /// - Dock layout structural revision counter bump (`dock_revision != last_dock_revision`).
    /// - Floating window count change (`floating_count != last_floating_count`).
    /// - Menu active state transition (`is_menu_active != last_menu_active`).
    /// - Modal dialog active state transition (`is_modal_active != last_modal_active`).
    /// - Drag-and-drop active state transition (`is_drag_active != last_drag_active`).
    /// - Active docking user interaction (`is_dock_interacting == true`).
    #[inline]
    #[must_use]
    pub fn is_shell_dirty(&self, inputs: &LifecycleInputs) -> bool {
        self.force_shell_rebuild
            || self.always_rebuild_shell
            || self.last_dimensions.is_none()
            || self.last_dimensions != Some(inputs.dimensions)
            || (self.last_scale_factor - inputs.scale_factor).abs() > 1e-4
            || self.last_dock_revision != inputs.dock_revision
            || self.last_floating_count != inputs.floating_count
            || self.last_menu_active != inputs.is_menu_active
            || self.last_modal_active != inputs.is_modal_active
            || self.last_drag_active != inputs.is_drag_active
            || inputs.is_dock_interacting
    }

    /// Non-mutating speculative query predicting the execution branch for the given inputs.
    ///
    /// Useful for telemetry, profiling pre-passes, or diagnostic logging.
    #[must_use]
    pub fn peek_branch(&self, inputs: &LifecycleInputs) -> LifecycleBranch<TabId> {
        if self.is_shell_dirty(inputs) {
            LifecycleBranch::FullShellRebuild
        } else if !self.dirty_tabs.is_empty() {
            let mut list: Vec<TabId> = self.dirty_tabs.iter().cloned().collect();
            list.shrink_to_fit();
            LifecycleBranch::GranularPanels(list)
        } else if inputs.has_volatile_updates {
            LifecycleBranch::InPlaceRefresh
        } else {
            LifecycleBranch::Dormant
        }
    }

    /// Evaluates the current frame, advances internal state trackers, and returns the execution branch.
    ///
    /// # State Synchronization Invariants
    /// - If [`LifecycleBranch::FullShellRebuild`] is returned, all pending `dirty_tabs` are cleared
    ///   because the full shell reconstruction rebuilds all panel leaf contents automatically.
    /// - If [`LifecycleBranch::GranularPanels`] is returned, the drained tabs are returned in the
    ///   branch and removed from the internal dirty set.
    /// - Internal tracking states (`last_dimensions`, `last_dock_revision`, `last_menu_active`, etc.)
    ///   are updated to match `inputs`.
    #[must_use]
    pub fn evaluate(&mut self, inputs: LifecycleInputs) -> LifecycleBranch<TabId> {
        let shell_dirty = self.is_shell_dirty(&inputs);

        // Advance baseline state tracking
        self.last_dimensions = Some(inputs.dimensions);
        self.last_scale_factor = inputs.scale_factor;
        self.last_dock_revision = inputs.dock_revision;
        self.last_floating_count = inputs.floating_count;
        self.last_menu_active = inputs.is_menu_active;
        self.last_modal_active = inputs.is_modal_active;
        self.last_drag_active = inputs.is_drag_active;
        self.force_shell_rebuild = false;

        if shell_dirty {
            // Full shell rebuild automatically reconstructs all leaf panels
            self.dirty_tabs.clear();
            LifecycleBranch::FullShellRebuild
        } else if !self.dirty_tabs.is_empty() {
            let panels: Vec<TabId> = self.dirty_tabs.drain().collect();
            LifecycleBranch::GranularPanels(panels)
        } else if inputs.has_volatile_updates {
            LifecycleBranch::InPlaceRefresh
        } else {
            LifecycleBranch::Dormant
        }
    }
}