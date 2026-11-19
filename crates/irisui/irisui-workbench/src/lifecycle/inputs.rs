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
/// (`winit`), docking implementations (`irisui-dock`), and application domain types.
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

    /// Monotonic domain revision counter reflecting external entity / model mutations (Undo/Redo commands, etc.).
    pub domain_revision: u64,

    /// Monotonic theme configuration revision counter reflecting styling, token, or color palette switches.
    pub theme_revision: u64,

    /// Deterministic geometry hash of active floating windows (`[Rect]`), detecting move/resize changes
    /// even when `floating_count` remains constant.
    pub floating_geometry_hash: u64,

    /// Whether any anchored pop-up, context menu, or flyout card is currently open.
    pub is_popup_active: bool,

    /// Monotonic revision or state bitfield for external embedded surfaces / viewport textures.
    pub surface_revision: u64,

    /// Monotonic revision for host-owned overlay/dialog tabs or custom chrome state.
    pub overlay_revision: u64,
}

impl LifecycleInputs {
    /// Creates a new lifecycle input descriptor with mandatory geometric parameters.
    ///
    /// Optional flags (floating count, menu/modal state, dock interaction, volatile updates, domain revision, theme revision)
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
            domain_revision: 0,
            theme_revision: 0,
            floating_geometry_hash: 0,
            is_popup_active: false,
            surface_revision: 0,
            overlay_revision: 0,
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

    /// Combines an external menu activity signal via logical OR with the workbench state.
    #[inline]
    #[must_use]
    pub const fn or_menu_active(mut self, active: bool) -> Self {
        self.is_menu_active = self.is_menu_active || active;
        self
    }

    /// Sets whether a modal dialog or blocker backdrop is active.
    #[inline]
    #[must_use]
    pub const fn with_modal_active(mut self, active: bool) -> Self {
        self.is_modal_active = active;
        self
    }

    /// Combines an external modal activity signal via logical OR with the workbench state.
    #[inline]
    #[must_use]
    pub const fn or_modal_active(mut self, active: bool) -> Self {
        self.is_modal_active = self.is_modal_active || active;
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

    /// Sets the external domain data transaction revision counter.
    #[inline]
    #[must_use]
    pub const fn with_domain_revision(mut self, revision: u64) -> Self {
        self.domain_revision = revision;
        self
    }

    /// Sets the application theme configuration revision counter.
    ///
    /// Changing the theme revision triggers a full shell reconstruction to re-style
    /// all dock chrome, statusbars, menus, and container surfaces.
    #[inline]
    #[must_use]
    pub const fn with_theme_revision(mut self, revision: u64) -> Self {
        self.theme_revision = revision;
        self
    }

    /// Sets the deterministic geometry hash of active floating windows.
    #[inline]
    #[must_use]
    pub const fn with_floating_geometry_hash(mut self, hash: u64) -> Self {
        self.floating_geometry_hash = hash;
        self
    }

    /// Computes and sets [`Self::floating_geometry_hash`] directly from an iterator of floating
    /// window rectangles with zero heap allocation.
    #[inline]
    #[must_use]
    pub fn with_floating_rects(
        mut self,
        rects: impl IntoIterator<Item = irisui_core::Rect>,
    ) -> Self {
        self.floating_geometry_hash = Self::hash_floating_rects(rects);
        self
    }

    /// Builds [`LifecycleInputs`] directly from a [`irisui_dock::DockState`], capturing
    /// dock revision, floating window count and geometry hash, and active splitter/tab drag state.
    #[inline]
    #[must_use]
    pub fn from_dock<TabId>(
        dock: &irisui_dock::DockState<TabId>,
        dimensions: (f32, f32),
        scale_factor: f32,
        theme_revision: u64,
    ) -> Self {
        Self::new(dimensions, scale_factor, dock.revision())
            .with_floating_count(dock.floating_windows.len())
            .with_floating_rects(dock.floating_windows.iter().map(|w| w.rect))
            .with_dock_interacting(dock.active_splitter.is_some() || dock.active_drag.is_some())
            .with_theme_revision(theme_revision)
    }

    /// Sets whether any anchored pop-up, context menu, or flyout is active.
    #[inline]
    #[must_use]
    pub const fn with_popup_active(mut self, active: bool) -> Self {
        self.is_popup_active = active;
        self
    }

    /// Combines an external pop-up activity signal via logical OR with the workbench state.
    #[inline]
    #[must_use]
    pub const fn or_popup_active(mut self, active: bool) -> Self {
        self.is_popup_active = self.is_popup_active || active;
        self
    }

    /// Sets the revision counter for external embedded surfaces / viewport textures.
    #[inline]
    #[must_use]
    pub const fn with_surface_revision(mut self, revision: u64) -> Self {
        self.surface_revision = revision;
        self
    }

    /// Sets the revision counter for host-owned overlay or dialog state.
    #[inline]
    #[must_use]
    pub const fn with_overlay_revision(mut self, revision: u64) -> Self {
        self.overlay_revision = revision;
        self
    }

    /// Computes an FNV-1a 64-bit hash over floating window rectangles so position/size changes
    /// are detected deterministically with zero heap allocation.
    #[inline]
    #[must_use]
    pub fn hash_floating_rects(rects: impl IntoIterator<Item = irisui_core::Rect>) -> u64 {
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x100000001b3;
        let mut hash = FNV_OFFSET;
        for r in rects {
            for bits in [
                r.x.to_bits(),
                r.y.to_bits(),
                r.width.to_bits(),
                r.height.to_bits(),
            ] {
                hash ^= u64::from(bits);
                hash = hash.wrapping_mul(FNV_PRIME);
            }
        }
        hash
    }
}