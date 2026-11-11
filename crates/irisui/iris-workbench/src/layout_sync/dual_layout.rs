// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Dual-Layout Eliminator & Boundary Synchronization
//!
//! Autonomous synchronization pipeline eliminating visual jitter between coarse initial
//! tree passes and granular Taffy CSS flexbox boundary solvers.

use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
use iris_core::{DirtyFlags, UiTree, WidgetId};
use iris_dock::UiNotifier;
use iris_layout::{LayoutEngine, LayoutError};
use iris_text::TextSystem;
use iris_widgets::declarative::{SubtreeSession, UiScope};
use std::fmt::Debug;
use std::hash::Hash;

/// Summary metrics captured following an automated layout synchronization pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LayoutSyncMetrics {
    /// Number of active docking leaf containers successfully shaped and resolved via Taffy.
    pub synchronized_count: usize,

    /// Number of leaf containers skipped due to non-positive bounds (e.g. collapsed splitters).
    pub skipped_count: usize,
}

/// Mutable references to core framework rendering and layout engines.
///
/// Bundles [`UiTree`], [`LayoutEngine`], and [`TextSystem`] into a single ergonomic descriptor
/// to ensure clean architecture and prevent argument proliferation across layout sync passes.
pub struct SubtreeEngines<'a> {
    /// Generational UI widget tree.
    pub tree: &'a mut UiTree,
    /// Flexbox boundary solver layout engine.
    pub layout_engine: &'a mut LayoutEngine,
    /// Text measurement and font layout system.
    pub text_system: &'a mut TextSystem,
}

impl<'a> SubtreeEngines<'a> {
    /// Constructs a new bundle of framework layout engines.
    #[inline]
    pub fn new(
        tree: &'a mut UiTree,
        layout_engine: &'a mut LayoutEngine,
        text_system: &'a mut TextSystem,
    ) -> Self {
        Self {
            tree,
            layout_engine,
            text_system,
        }
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DualLayoutEliminator;

impl DualLayoutEliminator {
    /// Creates a new autonomous dual-layout eliminator instance.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

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

    /// Granularly rebuilds all dirty docking leaf panels flagged in [`UiNotifier`].
    ///
    /// Evaluates all pending panel redraw tags, resolves each panel's docking leaf container
    /// via `leaf_resolver`, untracks stale child layout nodes, declaratively synthesizes fresh
    /// widgets via [`WorkbenchPanelRegistry::draw`], shapes typography, and recomputes boundary
    /// Taffy flexbox models in $O(M)$ time.
    ///
    /// Clears processed dirty panel tags in [`UiNotifier`], marks reconstructed leaves as
    /// sealed Invalidation Boundaries ([`UiTree::set_boundary`]), and schedules repainting
    /// ([`DirtyFlags::PAINT`]).
    ///
    /// Employs a zero-allocation stack buffer for standard workloads (up to 32 dirty panels).
    ///
    /// # Generics
    /// * `TabId` - The application's unique dock panel identifier type.
    /// * `Context` - Application state passed to registered declarative panel drawing routines.
    /// * `F` - Closure resolving a tab identifier to its active container [`WidgetId`].
    ///
    /// # Returns
    /// Number of panels successfully reconstructed.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if layout computation encounters invalid geometry.
    pub fn rebuild_dirty_panels<TabId, Context, F>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        registry: &mut WorkbenchPanelRegistry<TabId, Context>,
        context: &mut Context,
        mut leaf_resolver: F,
    ) -> Result<usize, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
        F: FnMut(TabId) -> Option<WidgetId>,
    {
        if !notifier.is_any_dirty() {
            return Ok(0);
        }

        // Zero-allocation stack buffer holding up to 32 dirty panel tags
        let mut stack_dirty = [None; 32];
        let mut count = 0;
        let mut heap_dirty: Option<Vec<TabId>> = None;

        for &panel_id in notifier.dirty_panels() {
            if count < 32 {
                stack_dirty[count] = Some(panel_id);
                count += 1;
            } else {
                if heap_dirty.is_none() {
                    let mut v = Vec::with_capacity(notifier.dirty_panels().len());
                    for item in stack_dirty.iter().flatten() {
                        v.push(*item);
                    }
                    heap_dirty = Some(v);
                }
                if let Some(v) = &mut heap_dirty {
                    v.push(panel_id);
                }
            }
        }

        let mut rebuilt_count = 0;

        let mut process_panel = |panel_id: TabId| -> Result<bool, LayoutError> {
            let Some(leaf_id) = leaf_resolver(panel_id) else {
                notifier.clear_panel(panel_id);
                return Ok(false);
            };

            let Some(node) = engines.tree.get(leaf_id) else {
                notifier.clear_panel(panel_id);
                return Ok(false);
            };

            if node.computed_rect.width <= 0.0 || node.computed_rect.height <= 0.0 {
                notifier.clear_panel(panel_id);
                return Ok(false);
            }

            SubtreeSession::rebuild(
                engines.tree,
                engines.layout_engine,
                engines.text_system,
                leaf_id,
                |scope| {
                    let _ = registry.draw(&panel_id, scope, context);
                },
            )?;

            engines.tree.set_boundary(leaf_id, true);
            engines.tree.mark_dirty(leaf_id, DirtyFlags::PAINT);
            notifier.clear_panel(panel_id);
            Ok(true)
        };

        if let Some(heap_list) = heap_dirty {
            for panel_id in heap_list {
                if process_panel(panel_id)? {
                    rebuilt_count += 1;
                }
            }
        } else {
            for panel_id in stack_dirty[..count].iter().flatten().copied() {
                if process_panel(panel_id)? {
                    rebuilt_count += 1;
                }
            }
        }

        Ok(rebuilt_count)
    }

    /// Granularly rebuilds all dirty docking leaf panels flagged in [`UiNotifier`] without requiring an instance.
    #[inline]
    pub fn rebuild_dirty_panels_direct<TabId, Context, F>(
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        registry: &mut WorkbenchPanelRegistry<TabId, Context>,
        context: &mut Context,
        leaf_resolver: F,
    ) -> Result<usize, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
        F: FnMut(TabId) -> Option<WidgetId>,
    {
        Self::new().rebuild_dirty_panels(engines, notifier, registry, context, leaf_resolver)
    }
}