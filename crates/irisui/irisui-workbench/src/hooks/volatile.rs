// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Volatile In-Place Refresh Hooks (`Branch 3`)
//!
//! Provides engine-independent contracts for high-frequency, layout-bypassing UI updates
//! (telemetry oscilloscopes, camera HUD text, compasses, live meters) executed during
//! [`LifecycleBranch::InPlaceRefresh`](crate::lifecycle::LifecycleBranch::InPlaceRefresh).

use irisui_core::UiTree;

use super::overlay::OverlayId;

/// Result returned by a [`VolatileCanvasHook`] describing which GPU caches must refresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VolatileRefreshResult {
    /// Whether glyph shaping / text section preparation must run this frame.
    pub needs_text_prepare: bool,
    /// Whether retained quad vertex buffers must be re-uploaded this frame.
    pub needs_quad_reupload: bool,
}

impl VolatileRefreshResult {
    /// Returns a no-op result indicating nothing changed in-place.
    #[inline]
    #[must_use]
    pub const fn none() -> Self {
        Self {
            needs_text_prepare: false,
            needs_quad_reupload: false,
        }
    }

    /// Returns a result requiring only quad draw command / buffer refresh.
    #[inline]
    #[must_use]
    pub const fn quads_only() -> Self {
        Self {
            needs_text_prepare: false,
            needs_quad_reupload: true,
        }
    }

    /// Returns a result requiring both typography preparation and quad re-upload.
    #[inline]
    #[must_use]
    pub const fn text_and_quads() -> Self {
        Self {
            needs_text_prepare: true,
            needs_quad_reupload: true,
        }
    }

    /// Combines two refresh results using logical OR.
    #[inline]
    #[must_use]
    pub const fn merge(self, other: Self) -> Self {
        Self {
            needs_text_prepare: self.needs_text_prepare || other.needs_text_prepare,
            needs_quad_reupload: self.needs_quad_reupload || other.needs_quad_reupload,
        }
    }

    /// Returns `true` if either text preparation or quad upload is required.
    #[inline]
    #[must_use]
    pub const fn any(self) -> bool {
        self.needs_text_prepare || self.needs_quad_reupload
    }
}

/// Extension trait for high-frequency volatile updates executed without layout rebuilds.
pub trait VolatileCanvasHook<AppState>: Send + Sync {
    /// Unique identifier for replacing or unregistering this volatile hook.
    fn id(&self) -> OverlayId;

    /// Performs an in-place update on tagged retained nodes (e.g. canvas primitives or text).
    fn refresh_in_place(
        &mut self,
        tree: &mut UiTree,
        app_state: &mut AppState,
    ) -> VolatileRefreshResult;
}

/// Closure-backed implementation of [`VolatileCanvasHook`].
pub struct ClosureVolatileHook<F> {
    id: OverlayId,
    func: F,
}

impl<F> ClosureVolatileHook<F> {
    /// Wraps a closure with an [`OverlayId`] into a [`VolatileCanvasHook`].
    #[inline]
    #[must_use]
    pub const fn new(id: OverlayId, func: F) -> Self {
        Self { id, func }
    }
}

impl<AppState, F> VolatileCanvasHook<AppState> for ClosureVolatileHook<F>
where
    F: FnMut(&mut UiTree, &mut AppState) -> VolatileRefreshResult + Send + Sync,
{
    #[inline]
    fn id(&self) -> OverlayId {
        self.id
    }

    #[inline]
    fn refresh_in_place(
        &mut self,
        tree: &mut UiTree,
        app_state: &mut AppState,
    ) -> VolatileRefreshResult {
        (self.func)(tree, app_state)
    }
}

/// Registry storing ordered [`VolatileCanvasHook`] instances executed during `InPlaceRefresh`.
pub struct VolatileHookRegistry<AppState> {
    hooks: Vec<Box<dyn VolatileCanvasHook<AppState>>>,
}

impl<AppState> Default for VolatileHookRegistry<AppState> {
    fn default() -> Self {
        Self::new()
    }
}

impl<AppState> VolatileHookRegistry<AppState> {
    /// Creates an empty [`VolatileHookRegistry`].
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Registers a volatile hook, replacing any existing hook with the same [`OverlayId`].
    pub fn register(&mut self, hook: impl VolatileCanvasHook<AppState> + 'static) {
        let id = hook.id();
        if let Some(slot) = self.hooks.iter_mut().find(|existing| existing.id() == id) {
            *slot = Box::new(hook);
            return;
        }
        self.hooks.push(Box::new(hook));
    }

    /// Convenience helper registering a closure as a [`VolatileCanvasHook`].
    pub fn register_fn<F>(&mut self, id: OverlayId, func: F)
    where
        F: FnMut(&mut UiTree, &mut AppState) -> VolatileRefreshResult + Send + Sync + 'static,
    {
        self.register(ClosureVolatileHook::new(id, func));
    }

    /// Unregisters the volatile hook matching `id`. Returns `true` if a hook was removed.
    pub fn unregister(&mut self, id: OverlayId) -> bool {
        let before = self.hooks.len();
        self.hooks.retain(|h| h.id() != id);
        self.hooks.len() != before
    }

    /// Returns `true` when at least one volatile hook is registered.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }

    /// Returns the number of registered volatile hooks.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.hooks.len()
    }

    /// Executes all registered volatile hooks in registration order and merges their refresh flags.
    pub fn refresh_in_place(
        &mut self,
        tree: &mut UiTree,
        app_state: &mut AppState,
    ) -> VolatileRefreshResult {
        let mut combined = VolatileRefreshResult::none();
        for hook in &mut self.hooks {
            combined = combined.merge(hook.refresh_in_place(tree, app_state));
        }
        combined
    }
}