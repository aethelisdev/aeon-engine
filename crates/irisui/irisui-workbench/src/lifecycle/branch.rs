// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Lifecycle Execution Branches
//!
//! Categorization of frame execution into three distinct architectural paths:
//! 1. Full shell & docking topology reconstruction (Branch 1).
//! 2. Granular partial leaf panel subtree rebuild (Branch 2).
//! 3. Retained in-place volatile draw refresh (Branch 3).
//! 4. Dormant steady-state skipping when zero updates are needed.

/// Distinct architectural execution branch evaluated for the current UI frame.
///
/// Desktop IDE shells cannot afford to rebuild their entire widget hierarchy,
/// dock layout, and Taffy flexbox graph on every frame. `LifecycleBranch` separates
/// coarse structural mutations from fine-grained leaf content updates and volatile
/// zero-retained-mutation rendering passes.
///
/// # Generics
/// - `TabId`: The application's unique dock panel identifier type (e.g. `PanelId`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleBranch<TabId> {
    /// Branch 1: Structural topology mutation requiring a full shell reconstruction.
    ///
    /// Triggered by window resizing, DPI scale factor alterations, dock splitter
    /// movements, tab docking/undocking, modal backdrop openings/closings, or
    /// explicit theme preset switches. The UI tree is cleared and reconstructed
    /// from root downward.
    FullShellRebuild,

    /// Branch 2: Granular partial rebuild of one or more dirty leaf containers.
    ///
    /// The root shell, top menu bar, bottom status bar, and all sibling docking
    /// panels remain fully intact. Only the specified leaf panels are rebuilt
    /// in-place inside their existing isolation boundaries with zero root tree destruction
    /// and $O(M)$ partial Taffy flexbox layout.
    GranularPanels(Vec<TabId>),

    /// Branch 3: Retained in-place volatile draw refresh with zero DOM mutations.
    ///
    /// The retained widget tree and layout engine are not modified at all.
    /// Only volatile canvas primitives (HUDs, compasses, telemetry graphs) and
    /// in-place reactive text nodes are refreshed directly onto the draw command stream.
    InPlaceRefresh,

    /// Steady-state dormant frame where zero visual or structural elements changed.
    ///
    /// Rendering pipelines may choose to skip command buffer emission and present
    /// the previous swapchain frame or hold steady at minimal energy consumption.
    Dormant,
}

impl<TabId> LifecycleBranch<TabId> {
    /// Returns `true` if this branch represents a full shell topology reconstruction.
    #[inline]
    #[must_use]
    pub const fn is_shell_rebuild(&self) -> bool {
        matches!(self, Self::FullShellRebuild)
    }

    /// Returns `true` if this branch represents a granular partial panel rebuild.
    #[inline]
    #[must_use]
    pub const fn is_granular(&self) -> bool {
        matches!(self, Self::GranularPanels(_))
    }

    /// Returns `true` if this branch represents an in-place volatile draw refresh.
    #[inline]
    #[must_use]
    pub const fn is_in_place(&self) -> bool {
        matches!(self, Self::InPlaceRefresh)
    }

    /// Returns `true` if this branch is completely dormant with zero work required.
    #[inline]
    #[must_use]
    pub const fn is_dormant(&self) -> bool {
        matches!(self, Self::Dormant)
    }

    /// Returns a slice of tab identifiers that require granular rebuilds.
    ///
    /// If the branch is not [`LifecycleBranch::GranularPanels`], returns an empty slice.
    #[inline]
    #[must_use]
    pub fn dirty_panels(&self) -> &[TabId] {
        match self {
            Self::GranularPanels(panels) => panels.as_slice(),
            _ => &[],
        }
    }
}