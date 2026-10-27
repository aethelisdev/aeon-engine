// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Dual-Layout Eliminator & Boundary Synchronization
//!
//! Autonomous synchronization pipeline eliminating visual jitter between coarse initial
//! tree passes and granular Taffy CSS flexbox boundary solvers.

use iris_core::{DirtyFlags, UiTree, WidgetId};
use iris_layout::{LayoutEngine, LayoutError};
use iris_text::TextSystem;
use iris_widgets::declarative::{SubtreeSession, UiScope};

/// Summary metrics captured following an automated layout synchronization pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LayoutSyncMetrics {
    /// Number of active docking leaf containers successfully shaped and resolved via Taffy.
    pub synchronized_count: usize,

    /// Number of leaf containers skipped due to non-positive bounds (e.g. collapsed splitters).
    pub skipped_count: usize,
}

/// Autonomous coordinator synchronizing docking leaf containers with Taffy flexbox solvers.
///
/// In complex multi-branch GUI architectures, initial full-shell construction passes
/// often compute preliminary container boundaries. If subsequent partial updates (Branch 2)
/// evaluate Taffy typography and box models without identical alignment in Branch 1,
/// slight visual jumping (1–5px jitter) occurs across widget hierarchies.
///
/// `DualLayoutEliminator` guarantees mathematical coordinate invariance between
/// Branch 1 (Full Shell) and Branch 2 (Partial Subtree Rebuild) by immediately
/// shaping typography and evaluating Taffy boundary geometry across all active leaves.
pub struct DualLayoutEliminator;

impl DualLayoutEliminator {
    /// Synchronizes a collection of dock leaf containers with Taffy boundary layouts.
    ///
    /// For every leaf container:
    /// 1. Inspects container geometry; skips collapsed/zero-area containers gracefully.
    /// 2. Performs subpixel typography shaping via [`TextSystem::measure_subtree_text`].
    /// 3. Solves the Taffy boundary layout via [`LayoutEngine::compute_boundary_subtree`].
    /// 4. Seals the leaf as an invalidation boundary ([`UiTree::set_boundary`]).
    /// 5. Marks the container for repainting ([`DirtyFlags::PAINT`]).
    ///
    /// # Generics
    /// - `TabId`: The application's unique dock panel identifier type.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if Taffy encounters invalid internal layout geometry.
    pub fn synchronize_all_leaves<TabId>(
        tree: &mut UiTree,
        layout_engine: &mut LayoutEngine,
        text_system: &mut TextSystem,
        leaf_containers: &[(TabId, WidgetId)],
    ) -> Result<LayoutSyncMetrics, LayoutError> {
        let mut metrics = LayoutSyncMetrics::default();

        for &(_, leaf_id) in leaf_containers {
            let Some(node) = tree.get(leaf_id) else {
                metrics.skipped_count += 1;
                continue;
            };

            let rect = node.computed_rect;
            if rect.width <= 0.0 || rect.height <= 0.0 {
                // Collapsed splitter or zero-size container: skip Taffy pass
                metrics.skipped_count += 1;
                continue;
            }

            text_system.measure_subtree_text(tree, leaf_id);
            layout_engine.compute_boundary_subtree(tree, leaf_id)?;

            tree.set_boundary(leaf_id, true);
            tree.mark_dirty(leaf_id, DirtyFlags::PAINT);
            metrics.synchronized_count += 1;
        }

        Ok(metrics)
    }

    /// Atomically rebuilds an individual leaf panel in-place during Branch 2 invalidation.
    ///
    /// Executes an isolated, generational subtree reconstruction cycle:
    /// 1. Untracks previous child layout nodes from Taffy and purges descendant nodes.
    /// 2. Renders fresh declarative widgets under `leaf_id` via `render_fn`.
    /// 3. Shapes typography and resolves boundary Taffy flexbox geometry in $O(M)$ time.
    /// 4. Re-seals the leaf as an Invalidation Boundary and flags it for repainting.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if boundary layout computation encounters invalid geometry.
    pub fn rebuild_leaf_panel<F>(
        tree: &mut UiTree,
        layout_engine: &mut LayoutEngine,
        text_system: &mut TextSystem,
        leaf_id: WidgetId,
        render_fn: F,
    ) -> Result<(), LayoutError>
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        SubtreeSession::rebuild(tree, layout_engine, text_system, leaf_id, render_fn)?;

        tree.set_boundary(leaf_id, true);
        tree.mark_dirty(leaf_id, DirtyFlags::PAINT);
        Ok(())
    }
}