// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Dual-Layout Eliminator & Boundary Synchronization
//!
//! Autonomous synchronization pipeline eliminating visual jitter between coarse initial
//! tree passes and granular Taffy CSS flexbox boundary solvers.

use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
use iris_core::{DirtyFlags, UiTree, WidgetId};
use iris_dock::{TabViewer, UiNotifier};
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

    /// Whether any layout subtree or leaf was reconstructed or synchronized, requiring text preparation.
    pub needs_text_prepare: bool,
}

pub use super::engines::SubtreeEngines;

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
///
/// Automatically tracks whether text preparation is required (`needs_text_prepare`)
/// when partial subtree rebuilds occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DualLayoutEliminator {
    needs_text_prepare: bool,
}

impl DualLayoutEliminator {
    /// Creates a new autonomous dual-layout eliminator instance.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            needs_text_prepare: false,
        }
    }

    /// Returns `true` if any partial subtree rebuild or layout pass triggered text preparation.
    #[inline]
    #[must_use]
    pub const fn needs_text_prepare(&self) -> bool {
        self.needs_text_prepare
    }

    /// Explicitly flags text preparation as required on the active frame.
    #[inline]
    pub fn mark_needs_text_prepare(&mut self) {
        self.needs_text_prepare = true;
    }

    /// Resets the text preparation flag to `false`.
    #[inline]
    pub fn clear_needs_text_prepare(&mut self) {
        self.needs_text_prepare = false;
    }

    /// Consumes and returns the current text preparation flag, resetting it to `false`.
    ///
    /// Useful for single-pass render loops to query and clear the flag atomically.
    #[inline]
    pub fn take_needs_text_prepare(&mut self) -> bool {
        let needs = self.needs_text_prepare;
        self.needs_text_prepare = false;
        needs
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
            tree.mark_dirty(leaf_id, DirtyFlags::PAINT | DirtyFlags::TEXT);
            metrics.synchronized_count += 1;
        }

        metrics.needs_text_prepare = metrics.synchronized_count > 0;
        Ok(metrics)
    }

    /// Atomically rebuilds an individual leaf panel in-place during Branch 2 invalidation.
    ///
    /// Executes an isolated, generational subtree reconstruction cycle:
    /// 1. Untracks previous child layout nodes from Taffy and purges descendant nodes.
    /// 2. Renders fresh declarative widgets under `leaf_id` via `render_fn`.
    /// 3. Shapes typography and resolves boundary Taffy flexbox geometry in $O(M)$ time.
    /// 4. Re-seals the leaf as an Invalidation Boundary and flags it for repainting and text preparation.
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
        tree.mark_dirty(leaf_id, DirtyFlags::PAINT | DirtyFlags::TEXT);
        Ok(())
    }

    /// Atomically rebuilds an individual leaf panel in-place during partial subtree invalidation.
    ///
    /// Executes an isolated, generational subtree reconstruction cycle:
    /// 1. Validates that the container node exists and has non-zero computed dimensions (skips collapsed leaves).
    /// 2. Untracks previous child layout nodes from Taffy and purges descendant nodes from [`UiTree`].
    /// 3. Renders fresh declarative widgets under `leaf_id` via `render_fn`.
    /// 4. Shapes typography and resolves boundary Taffy flexbox geometry in $O(M)$ time.
    /// 5. Re-seals the leaf as an Invalidation Boundary and flags it for repainting and text preparation.
    ///
    /// Returns `Ok(true)` if the panel was successfully reconstructed, or `Ok(false)` if the leaf was
    /// missing or collapsed (e.g. 0-width/0-height).
    ///
    /// # Errors
    /// Returns [`LayoutError`] if boundary layout computation encounters invalid geometry.
    pub fn rebuild_single_panel_leaf<F>(
        tree: &mut UiTree,
        layout_engine: &mut LayoutEngine,
        text_system: &mut TextSystem,
        leaf_id: WidgetId,
        render_fn: F,
    ) -> Result<bool, LayoutError>
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let Some(node) = tree.get(leaf_id) else {
            return Ok(false);
        };

        if node.computed_rect.width <= 0.0 || node.computed_rect.height <= 0.0 {
            return Ok(false);
        }

        SubtreeSession::rebuild(tree, layout_engine, text_system, leaf_id, render_fn)?;

        tree.set_boundary(leaf_id, true);
        tree.mark_dirty(leaf_id, DirtyFlags::PAINT | DirtyFlags::TEXT);
        Ok(true)
    }

    /// Atomically rebuilds an individual leaf panel in-place with a fallible render closure.
    ///
    /// Guarantees that any error returned by `render_fn` safely aborts layout computation
    /// and is propagated as a [`LayoutError::Taffy`].
    ///
    /// Returns `Ok(true)` if reconstructed, or `Ok(false)` if missing or zero-sized.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if boundary layout computation encounters invalid geometry or `render_fn` fails.
    pub fn rebuild_single_panel_leaf_fallible<E, F>(
        tree: &mut UiTree,
        layout_engine: &mut LayoutEngine,
        text_system: &mut TextSystem,
        leaf_id: WidgetId,
        render_fn: F,
    ) -> Result<bool, LayoutError>
    where
        F: FnOnce(&mut UiScope<'_>) -> Result<(), E>,
        E: std::fmt::Display,
    {
        let Some(node) = tree.get(leaf_id) else {
            return Ok(false);
        };

        if node.computed_rect.width <= 0.0 || node.computed_rect.height <= 0.0 {
            return Ok(false);
        }

        let mut render_err = None;
        SubtreeSession::rebuild(tree, layout_engine, text_system, leaf_id, |scope| {
            if let Err(err) = render_fn(scope) {
                render_err = Some(LayoutError::Taffy(err.to_string()));
            }
        })?;

        if let Some(err) = render_err {
            return Err(err);
        }

        tree.set_boundary(leaf_id, true);
        tree.mark_dirty(leaf_id, DirtyFlags::PAINT | DirtyFlags::TEXT);
        Ok(true)
    }

    /// Atomically rebuilds an individual leaf panel and records text preparation on this instance.
    pub fn rebuild_single_panel_leaf_instance<F>(
        &mut self,
        tree: &mut UiTree,
        layout_engine: &mut LayoutEngine,
        text_system: &mut TextSystem,
        leaf_id: WidgetId,
        render_fn: F,
    ) -> Result<bool, LayoutError>
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let rebuilt =
            Self::rebuild_single_panel_leaf(tree, layout_engine, text_system, leaf_id, render_fn)?;
        if rebuilt {
            self.needs_text_prepare = true;
        }
        Ok(rebuilt)
    }

    /// Atomically rebuilds an individual leaf panel and clears its dirty flag in [`UiNotifier`].
    ///
    /// If the leaf exists and has positive area, it is rebuilt and sealed as an Invalidation Boundary,
    /// its dirty state in `notifier` is cleared, and `needs_text_prepare` is flagged on this eliminator instance.
    pub fn rebuild_single_panel_leaf_notified<TabId, F>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        panel_id: TabId,
        leaf_id: WidgetId,
        render_fn: F,
    ) -> Result<bool, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
        F: FnOnce(&mut UiScope<'_>),
    {
        let rebuilt = Self::rebuild_single_panel_leaf(
            engines.tree,
            engines.layout_engine,
            engines.text_system,
            leaf_id,
            render_fn,
        )?;
        if rebuilt {
            notifier.clear_panel(panel_id);
            self.needs_text_prepare = true;
        }
        Ok(rebuilt)
    }

    /// Atomically rebuilds an individual leaf panel with a fallible closure and clears its flag in [`UiNotifier`].
    pub fn rebuild_single_panel_leaf_notified_fallible<TabId, E, F>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        panel_id: TabId,
        leaf_id: WidgetId,
        render_fn: F,
    ) -> Result<bool, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
        F: FnOnce(&mut UiScope<'_>) -> Result<(), E>,
        E: std::fmt::Display,
    {
        let rebuilt = Self::rebuild_single_panel_leaf_fallible(
            engines.tree,
            engines.layout_engine,
            engines.text_system,
            leaf_id,
            render_fn,
        )?;
        if rebuilt {
            notifier.clear_panel(panel_id);
            self.needs_text_prepare = true;
        }
        Ok(rebuilt)
    }

    /// Atomically rebuilds an individual registered dock panel in-place under `leaf_id`.
    ///
    /// Delegates directly to [`Self::rebuild_single_panel_leaf_fallible`] with [`WorkbenchPanelRegistry::draw`].
    /// Clears previous children, shapes typography, resolves boundary flexbox geometry, and marks the leaf dirty.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if boundary layout computation fails or registry drawing encounters an error.
    pub fn rebuild_panel<TabId, Context>(
        engines: &mut SubtreeEngines<'_>,
        registry: &mut WorkbenchPanelRegistry<TabId, Context>,
        context: &mut Context,
        panel_id: TabId,
        leaf_id: WidgetId,
    ) -> Result<bool, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
    {
        Self::rebuild_single_panel_leaf_fallible(
            engines.tree,
            engines.layout_engine,
            engines.text_system,
            leaf_id,
            |scope| registry.draw(&panel_id, scope, context),
        )
    }

    /// Atomically rebuilds an individual dock panel using an arbitrary [`TabViewer`] implementation.
    ///
    /// Clears previous child nodes, invokes [`TabViewer::ui`] inside [`SubtreeSession`],
    /// shapes typography, resolves boundary layout, seals the leaf as an Invalidation Boundary,
    /// and schedules repainting.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if boundary layout computation encounters invalid geometry.
    pub fn rebuild_panel_with_viewer<TabId, V>(
        engines: &mut SubtreeEngines<'_>,
        viewer: &mut V,
        panel_id: TabId,
        leaf_id: WidgetId,
    ) -> Result<bool, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
        V: TabViewer<TabId>,
    {
        Self::rebuild_single_panel_leaf_fallible(
            engines.tree,
            engines.layout_engine,
            engines.text_system,
            leaf_id,
            |scope| {
                viewer.ui(scope, &panel_id);
                Ok::<(), std::convert::Infallible>(())
            },
        )
    }

    /// Atomically rebuilds an individual registered dock panel and records text preparation on this instance.
    pub fn rebuild_panel_instance<TabId, Context>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        registry: &mut WorkbenchPanelRegistry<TabId, Context>,
        context: &mut Context,
        panel_id: TabId,
        leaf_id: WidgetId,
    ) -> Result<bool, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
    {
        let rebuilt = Self::rebuild_panel(engines, registry, context, panel_id, leaf_id)?;
        if rebuilt {
            self.needs_text_prepare = true;
        }
        Ok(rebuilt)
    }

    /// Rebuilds an individual registered dock panel in-place under `leaf_id`, clearing its dirty state in `notifier`.
    ///
    /// Delegates directly to [`Self::rebuild_panel`] and flags typography preparation on this eliminator instance.
    pub fn rebuild_single_panel_notified<TabId, Context>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        registry: &mut WorkbenchPanelRegistry<TabId, Context>,
        context: &mut Context,
        panel_id: TabId,
        leaf_id: WidgetId,
    ) -> Result<bool, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
    {
        let rebuilt = Self::rebuild_panel(engines, registry, context, panel_id, leaf_id)?;
        if rebuilt {
            notifier.clear_panel(panel_id);
            self.needs_text_prepare = true;
        }
        Ok(rebuilt)
    }

    /// Atomically rebuilds an individual leaf panel with a fallible render closure.
    ///
    /// Delegates directly to [`Self::rebuild_single_panel_leaf_fallible`].
    ///
    /// # Errors
    /// Returns [`LayoutError`] if the node is missing, geometry is invalid, or `render_fn` fails.
    pub fn rebuild_leaf_panel_fallible<E, F>(
        engines: &mut SubtreeEngines<'_>,
        leaf_id: WidgetId,
        render_fn: F,
    ) -> Result<bool, LayoutError>
    where
        F: FnOnce(&mut UiScope<'_>) -> Result<(), E>,
        E: std::fmt::Display,
    {
        Self::rebuild_single_panel_leaf_fallible(
            engines.tree,
            engines.layout_engine,
            engines.text_system,
            leaf_id,
            render_fn,
        )
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

            let rebuilt = Self::rebuild_panel(engines, registry, context, panel_id, leaf_id)?;
            notifier.clear_panel(panel_id);
            Ok(rebuilt)
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

        if rebuilt_count > 0 {
            self.needs_text_prepare = true;
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

    /// Maps physical dirty boundaries in the tree back to tab identifiers and flags them in `notifier`.
    ///
    /// Clears `tree.dirty_boundaries` after synchronization with zero redundant heap allocation.
    pub fn sync_dirty_boundaries<TabId, F>(
        tree: &mut UiTree,
        notifier: &mut UiNotifier<TabId>,
        mut tab_resolver: F,
    ) where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
        F: FnMut(WidgetId) -> Option<TabId>,
    {
        if tree.has_dirty_boundaries() {
            let mut boundaries = std::mem::take(&mut tree.dirty_boundaries);
            for boundary_id in &boundaries {
                if let Some(tab_id) = tab_resolver(*boundary_id) {
                    notifier.tag_redraw(tab_id);
                }
            }
            boundaries.clear();
            tree.dirty_boundaries = boundaries;
        }
    }

    /// Synchronizes dirty boundaries from `tree` and rebuilds all dirty docking leaf panels in $O(M)$ time.
    ///
    /// Clears physical dirty boundaries, synchronizes affected tabs into `notifier`,
    /// and invokes [`Self::rebuild_dirty_panels`].
    ///
    /// # Generics
    /// * `TabId` - The application's unique dock panel identifier type.
    /// * `Context` - Application state passed to registered declarative panel drawing routines.
    /// * `F` - Closure resolving a tab identifier to its active container [`WidgetId`].
    /// * `B` - Closure resolving a boundary container [`WidgetId`] back to its tab identifier.
    ///
    /// # Returns
    /// Number of panels successfully reconstructed.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if layout computation encounters invalid geometry.
    pub fn rebuild_dirty_panels_with_boundaries<TabId, Context, F, B>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        registry: &mut WorkbenchPanelRegistry<TabId, Context>,
        context: &mut Context,
        leaf_resolver: F,
        boundary_resolver: B,
    ) -> Result<usize, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
        F: FnMut(TabId) -> Option<WidgetId>,
        B: FnMut(WidgetId) -> Option<TabId>,
    {
        Self::sync_dirty_boundaries(engines.tree, notifier, boundary_resolver);
        self.rebuild_dirty_panels(engines, notifier, registry, context, leaf_resolver)
    }
}