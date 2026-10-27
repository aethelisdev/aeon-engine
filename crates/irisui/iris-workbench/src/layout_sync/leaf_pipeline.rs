// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # High-Level Dock Leaf Synchronization Pipeline
//!
//! Declarative helper routines connecting active [`DockChromeFrame`] instances
//! directly into the [`DualLayoutEliminator`] boundary synchronization pass.

use iris_core::{UiTree, WidgetId};
use iris_dock::DockChromeFrame;
use iris_layout::{LayoutEngine, LayoutError};
use iris_text::TextSystem;

use super::dual_layout::{DualLayoutEliminator, LayoutSyncMetrics};

/// Synchronizes all leaf containers captured in an active [`DockChromeFrame`].
///
/// Designed to be called immediately after [`DockArea::show`](iris_dock::DockArea::show)
/// during Branch 1 (Full Shell Rebuild). Ensures that all newly minted dock leaves
/// have their typography measured and their Taffy flexbox boundary layout solved
/// before the first frame is rendered, eliminating subsequent Branch 2 coordinate divergence.
///
/// # Generics
/// - `TabId`: The application's unique dock panel identifier type.
///
/// # Errors
/// Returns [`LayoutError`] if Taffy encounters invalid internal layout geometry.
pub fn synchronize_dock_frame<TabId: Copy>(
    tree: &mut UiTree,
    layout_engine: &mut LayoutEngine,
    text_system: &mut TextSystem,
    frame: &DockChromeFrame<TabId>,
) -> Result<LayoutSyncMetrics, LayoutError> {
    DualLayoutEliminator::synchronize_all_leaves(
        tree,
        layout_engine,
        text_system,
        &frame.leaf_containers,
    )
}

/// Synchronizes a raw slice of `(TabId, WidgetId)` leaf containers with the Taffy layout engine.
///
/// # Generics
/// - `TabId`: The application's unique dock panel identifier type.
///
/// # Errors
/// Returns [`LayoutError`] if Taffy encounters invalid internal layout geometry.
pub fn synchronize_dock_leaves<TabId: Copy>(
    tree: &mut UiTree,
    layout_engine: &mut LayoutEngine,
    text_system: &mut TextSystem,
    leaves: &[(TabId, WidgetId)],
) -> Result<LayoutSyncMetrics, LayoutError> {
    DualLayoutEliminator::synchronize_all_leaves(tree, layout_engine, text_system, leaves)
}