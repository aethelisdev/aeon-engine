// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Lifecycle Evaluation Inputs
//!
//! Declarative parameter container capturing the external system state necessary
//! to evaluate whether the application shell requires a full reconstruction,
//! granular panel refresh, or volatile draw refresh.

/// Environmental and interactive state passed into [`WorkbenchLifecycle::evaluate`](super::WorkbenchLifecycle::evaluate).
///
/// This structure decouples the lifecycle state machine from concrete windowing crates
/// (`winit`), docking implementations (`iris-dock`), and application domain types.
/// All metrics are expressed in standard primitive types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LifecycleInputs {
    /// Total viewport dimensions `(width, height)` in logical pixels.
    pub dimensions: (f32, f32),

    /// Current High-DPI monitor scale factor (e.g. 1.0 for standard displays, 2.0 for Retina/4K).
    pub scale_factor: f32,

    /// Structural revision counter from the docking tree (`DockState::revision()`).
    ///
    /// Incremented automatically by the docking engine whenever tabs are split, merged,
    /// closed, added, or reordered.
    pub dock_revision: u64,

    /// Number of actively instantiated floating / detached tool windows.
    pub floating_count: usize,

    /// Whether any top menu bar dropdown card or contextual pop-up is currently open.
    pub is_menu_active: bool,

    /// Whether any modal dialog, confirmation sheet, or blocker backdrop is currently visible.
    pub is_modal_active: bool,

    /// Whether the user is actively dragging a dock splitter, resizing a pane, or dragging a tab.
    pub is_dock_interacting: bool,

    /// Whether volatile HUDs, 3D compasses, or high-frequency telemetry graphs have pending frame updates.
    pub has_volatile_updates: bool,

    /// Whether any drag-and-drop operation is currently active in the workbench.
    pub is_drag_active: bool,
}

impl LifecycleInputs {
    /// Creates a new lifecycle input descriptor with mandatory geometric parameters.
    ///
    /// Optional flags (floating count, menu/modal state, dock interaction, volatile updates)
    /// default to zero / false and can be chained fluently.
    ///
    /// # Parameters
    /// - `dimensions`: `(width, height)` in logical pixels.
    /// - `scale_factor`: Monitor scale factor (usually >= 1.0).
    /// - `dock_revision`: Monotonic revision counter from `DockState::revision()`.
    #[inline]
    #[must_use]
    pub const fn new(dimensions: (f32, f32), scale_factor: f32, dock_revision: u64) -> Self {
        Self {
            dimensions,
            scale_factor,
            dock_revision,
            floating_count: 0,
            is_menu_active: false,
            is_modal_active: false,
            is_dock_interacting: false,
            has_volatile_updates: false,
            is_drag_active: false,
        }
    }

    /// Sets the number of detached floating windows.
    #[inline]
    #[must_use]
    pub const fn with_floating_count(mut self, count: usize) -> Self {
        self.floating_count = count;
        self
    }

    /// Sets whether any top menu bar dropdown or pop-up is active.
    #[inline]
    #[must_use]
    pub const fn with_menu_active(mut self, active: bool) -> Self {
        self.is_menu_active = active;
        self
    }

    /// Sets whether a modal dialog or blocker backdrop is active.
    #[inline]
    #[must_use]
    pub const fn with_modal_active(mut self, active: bool) -> Self {
        self.is_modal_active = active;
        self
    }

    /// Sets whether a docking splitter or tab drag interaction is actively occurring.
    #[inline]
    #[must_use]
    pub const fn with_dock_interacting(mut self, interacting: bool) -> Self {
        self.is_dock_interacting = interacting;
        self
    }

    /// Sets whether volatile elements (graphs, compass, HUD) have pending frame data.
    #[inline]
    #[must_use]
    pub const fn with_volatile_updates(mut self, updates: bool) -> Self {
        self.has_volatile_updates = updates;
        self
    }

    /// Sets whether any drag-and-drop operation is actively in flight across the workbench.
    #[inline]
    #[must_use]
    pub const fn with_drag_active(mut self, active: bool) -> Self {
        self.is_drag_active = active;
        self
    }
}