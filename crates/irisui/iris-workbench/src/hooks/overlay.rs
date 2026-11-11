// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Volatile Hardware Overlay Hooks & Interactive Overlay Layer
//!
//! Provides traits, anchors, and autonomous coordinators for rendering dynamic, per-frame
//! interactive overlay graphics (HUD toolbars, dropdown popovers, color pickers, gizmos,
//! telemetry oscilloscopes) on top of workbench panels without layout collisions.

use crate::action_bus::ActionBus;
use iris_core::event::{KeyCode, UiEvent};
use iris_core::geometry::{Point, Rect};
use iris_widgets::canvas::ImmediateCanvas;
use iris_widgets::declarative::UiScope;

/// Extension contract for emitting volatile hardware draw commands over workbench panels.
///
/// Implementors hook into the per-frame painting pass without mutating or dirtying
/// the persistent retained widget tree. Drawing directly onto the [`ImmediateCanvas`]
/// bypasses layout computation passes and submits hardware primitives straight
/// to the GPU compiler.
///
/// # Blanket Implementation
///
/// Any closure with signature `FnMut(&mut ImmediateCanvas<'_>, Rect) + Send + Sync`
/// automatically implements [`CustomOverlayHook`].
pub trait CustomOverlayHook: Send + Sync {
    /// Renders volatile immediate draw commands onto the canvas within the given container bounds.
    ///
    /// # Arguments
    /// * `canvas` - Immediate drawing context emitting primitives directly to the GPU compiler.
    /// * `bounds` - Resolved bounding rectangle of the host container or viewport panel.
    fn render_overlay(&mut self, canvas: &mut ImmediateCanvas<'_>, bounds: Rect);
}

impl<F> CustomOverlayHook for F
where
    F: FnMut(&mut ImmediateCanvas<'_>, Rect) + Send + Sync,
{
    #[inline]
    fn render_overlay(&mut self, canvas: &mut ImmediateCanvas<'_>, bounds: Rect) {
        (self)(canvas, bounds);
    }
}

impl CustomOverlayHook for Box<dyn CustomOverlayHook> {
    #[inline]
    fn render_overlay(&mut self, canvas: &mut ImmediateCanvas<'_>, bounds: Rect) {
        (**self).render_overlay(canvas, bounds);
    }
}

/// Strongly typed 64-bit identifier distinguishing overlay hooks within the workbench.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OverlayId(pub u64);

impl OverlayId {
    /// Creates a new strongly typed overlay identifier.
    #[must_use]
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Extracts the raw 64-bit integer value.
    #[must_use]
    #[inline]
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl From<u64> for OverlayId {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for OverlayId {
    type Target = u64;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Anchoring and placement geometry policy for an overlay layer.
///
/// Determines how the overlay bounding rectangle is mathematically resolved within
/// the parent workspace or viewport container without hardcoded layout heuristics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverlayAnchor {
    /// Explicit absolute rectangle in logical window coordinates.
    Absolute(Rect),
    /// Positioned relative to the container with an offset `(dx, dy)` and explicit size `(w, h)`.
    Offset {
        /// Offset in logical pixels from container top-left corner.
        offset: Point,
        /// Explicit dimensions `(width, height)` in logical pixels.
        size: (f32, f32),
    },
    /// Centered within the parent workspace or container.
    Centered {
        /// Explicit dimensions `(width, height)` in logical pixels.
        size: (f32, f32),
    },
    /// Spans the entire container or workspace area.
    Full,
}

impl OverlayAnchor {
    /// Resolves the concrete screen-space rectangle within the provided workspace bounds.
    #[must_use]
    #[inline]
    pub fn resolve_rect(&self, workspace: Rect) -> Rect {
        match *self {
            Self::Absolute(rect) => rect,
            Self::Offset { offset, size } => Rect::new(
                workspace.x + offset.x,
                workspace.y + offset.y,
                size.0,
                size.1,
            ),
            Self::Centered { size } => {
                let x = workspace.x + ((workspace.width - size.0) * 0.5).max(0.0);
                let y = workspace.y + ((workspace.height - size.1) * 0.5).max(0.0);
                Rect::new(x, y, size.0, size.1)
            }
            Self::Full => workspace,
        }
    }
}

/// Extension contract for volatile, interactive overlay layers (HUD toolbars, dropdowns, color pickers).
///
/// Enables panels to inject floating layers over the workbench workspace without
/// polluting the underlying retained widget tree or causing layout boundary collisions.
pub trait OverlayHook<AppState, Action: 'static + Send + Sync>: Send + Sync {
    /// Returns the unique 64-bit identifier of this overlay hook.
    fn id(&self) -> OverlayId;

    /// Evaluates whether this overlay is currently active and should be presented.
    ///
    /// Inactive overlays are skipped during hit-testing, event processing, and rendering.
    fn is_active(&self, _state: &AppState) -> bool {
        true
    }

    /// Returns the resolved bounding rectangle of the overlay within the given workspace.
    ///
    /// When `Some(rect)`, the workbench uses this rectangle for pointer occlusion and clipping.
    /// When `None`, the overlay manages its own unbounded or portal layout.
    fn bounds(&self, _workspace: Rect, _state: &AppState) -> Option<Rect> {
        None
    }

    /// Evaluates whether a screen-space coordinate falls within this overlay's interactive area.
    ///
    /// Returning `true` captures pointer events and prevents interaction pass-through
    /// to underlying 3D viewports or docked panels.
    fn wants_pointer_at(&self, point: Point, workspace: Rect, state: &AppState) -> bool {
        self.bounds(workspace, state)
            .is_some_and(|b| b.contains_point(point))
    }

    /// Declaratively renders the overlay into the given [`UiScope`].
    ///
    /// # Arguments
    /// * `scope` - Declarative UI scope scoped to the overlay layer or root container.
    /// * `workspace` - Bounding rectangle of the available workspace.
    /// * `state` - Mutable reference to the application domain state.
    /// * `action_bus` - Decoupled action bus collecting emitted application actions.
    fn render(
        &mut self,
        scope: &mut UiScope<'_>,
        workspace: Rect,
        state: &mut AppState,
        action_bus: &mut ActionBus<Action>,
    );

    /// Processes an input event, returning `true` if the event was consumed by the overlay.
    fn handle_event(
        &mut self,
        _event: &UiEvent,
        _workspace: Rect,
        _state: &mut AppState,
        _action_bus: &mut ActionBus<Action>,
    ) -> bool {
        false
    }

    /// Whether clicking outside of [`Self::bounds`] should automatically dismiss this overlay.
    fn dismiss_on_outside_click(&self) -> bool {
        false
    }

    /// Whether pressing the `Escape` key should automatically dismiss this overlay.
    fn dismiss_on_escape(&self) -> bool {
        false
    }

    /// Callback invoked when the overlay is dismissed (e.g. by outside click or Escape key).
    fn on_dismiss(&mut self, _state: &mut AppState, _action_bus: &mut ActionBus<Action>) {}
}

impl<AppState, Action: 'static + Send + Sync> OverlayHook<AppState, Action>
    for Box<dyn OverlayHook<AppState, Action>>
{
    #[inline]
    fn id(&self) -> OverlayId {
        (**self).id()
    }

    #[inline]
    fn is_active(&self, state: &AppState) -> bool {
        (**self).is_active(state)
    }

    #[inline]
    fn bounds(&self, workspace: Rect, state: &AppState) -> Option<Rect> {
        (**self).bounds(workspace, state)
    }

    #[inline]
    fn wants_pointer_at(&self, point: Point, workspace: Rect, state: &AppState) -> bool {
        (**self).wants_pointer_at(point, workspace, state)
    }

    #[inline]
    fn render(
        &mut self,
        scope: &mut UiScope<'_>,
        workspace: Rect,
        state: &mut AppState,
        action_bus: &mut ActionBus<Action>,
    ) {
        (**self).render(scope, workspace, state, action_bus);
    }

    #[inline]
    fn handle_event(
        &mut self,
        event: &UiEvent,
        workspace: Rect,
        state: &mut AppState,
        action_bus: &mut ActionBus<Action>,
    ) -> bool {
        (**self).handle_event(event, workspace, state, action_bus)
    }

    #[inline]
    fn dismiss_on_outside_click(&self) -> bool {
        (**self).dismiss_on_outside_click()
    }

    #[inline]
    fn dismiss_on_escape(&self) -> bool {
        (**self).dismiss_on_escape()
    }

    #[inline]
    fn on_dismiss(&mut self, state: &mut AppState, action_bus: &mut ActionBus<Action>) {
        (**self).on_dismiss(state, action_bus);
    }
}

/// Type alias for user-supplied declarative overlay render closure.
pub type OverlayRenderFn<AppState, Action> =
    Box<dyn FnMut(&mut UiScope<'_>, Rect, &mut AppState, &mut ActionBus<Action>) + Send + Sync>;

/// Type alias for user-supplied overlay event interceptor closure.
pub type OverlayEventFn<AppState, Action> =
    Box<dyn FnMut(&UiEvent, Rect, &mut AppState, &mut ActionBus<Action>) -> bool + Send + Sync>;

/// Type alias for user-supplied overlay dismissal notification closure.
pub type OverlayDismissFn<AppState, Action> =
    Box<dyn FnMut(&mut AppState, &mut ActionBus<Action>) + Send + Sync>;

/// Type alias for dynamic active predicate closure evaluated against application state.
pub type OverlayActiveFn<AppState> = Box<dyn Fn(&AppState) -> bool + Send + Sync>;

/// Fluent builder and concrete implementation of [`OverlayHook`] powered by closures.
pub struct ClosureOverlayHook<AppState, Action: 'static + Send + Sync> {
    id: OverlayId,
    anchor: Option<OverlayAnchor>,
    active_fn: Option<OverlayActiveFn<AppState>>,
    render_fn: OverlayRenderFn<AppState, Action>,
    event_fn: Option<OverlayEventFn<AppState, Action>>,
    dismiss_on_outside_click: bool,
    dismiss_on_escape: bool,
    on_dismiss_fn: Option<OverlayDismissFn<AppState, Action>>,
}

impl<AppState, Action: 'static + Send + Sync> ClosureOverlayHook<AppState, Action> {
    /// Creates a new closure-driven overlay hook with the given identifier and render closure.
    pub fn new<F>(id: impl Into<OverlayId>, render_fn: F) -> Self
    where
        F: FnMut(&mut UiScope<'_>, Rect, &mut AppState, &mut ActionBus<Action>)
            + Send
            + Sync
            + 'static,
    {
        Self {
            id: id.into(),
            anchor: None,
            active_fn: None,
            render_fn: Box::new(render_fn),
            event_fn: None,
            dismiss_on_outside_click: false,
            dismiss_on_escape: false,
            on_dismiss_fn: None,
        }
    }

    /// Configures the placement anchoring policy for this overlay.
    #[must_use]
    pub fn with_anchor(mut self, anchor: OverlayAnchor) -> Self {
        self.anchor = Some(anchor);
        self
    }

    /// Configures a dynamic predicate determining whether this overlay is active.
    #[must_use]
    pub fn with_active<P>(mut self, predicate: P) -> Self
    where
        P: Fn(&AppState) -> bool + Send + Sync + 'static,
    {
        self.active_fn = Some(Box::new(predicate));
        self
    }

    /// Configures an event handler closure intercepting inputs before docked panels.
    #[must_use]
    pub fn with_event_handler<E>(mut self, event_fn: E) -> Self
    where
        E: FnMut(&UiEvent, Rect, &mut AppState, &mut ActionBus<Action>) -> bool
            + Send
            + Sync
            + 'static,
    {
        self.event_fn = Some(Box::new(event_fn));
        self
    }

    /// Configures whether clicking outside of resolved bounds dismisses this overlay.
    #[must_use]
    pub fn with_dismiss_on_outside_click(mut self, dismiss: bool) -> Self {
        self.dismiss_on_outside_click = dismiss;
        self
    }

    /// Configures whether pressing the Escape key dismisses this overlay.
    #[must_use]
    pub fn with_dismiss_on_escape(mut self, dismiss: bool) -> Self {
        self.dismiss_on_escape = dismiss;
        self
    }

    /// Configures a callback invoked when the overlay is dismissed.
    #[must_use]
    pub fn with_on_dismiss<D>(mut self, on_dismiss: D) -> Self
    where
        D: FnMut(&mut AppState, &mut ActionBus<Action>) + Send + Sync + 'static,
    {
        self.on_dismiss_fn = Some(Box::new(on_dismiss));
        self
    }
}

impl<AppState, Action: 'static + Send + Sync> OverlayHook<AppState, Action>
    for ClosureOverlayHook<AppState, Action>
{
    #[inline]
    fn id(&self) -> OverlayId {
        self.id
    }

    #[inline]
    fn is_active(&self, state: &AppState) -> bool {
        self.active_fn.as_ref().is_none_or(|f| f(state))
    }

    #[inline]
    fn bounds(&self, workspace: Rect, _state: &AppState) -> Option<Rect> {
        self.anchor.map(|a| a.resolve_rect(workspace))
    }

    #[inline]
    fn render(
        &mut self,
        scope: &mut UiScope<'_>,
        workspace: Rect,
        state: &mut AppState,
        action_bus: &mut ActionBus<Action>,
    ) {
        (self.render_fn)(scope, workspace, state, action_bus);
    }

    #[inline]
    fn handle_event(
        &mut self,
        event: &UiEvent,
        workspace: Rect,
        state: &mut AppState,
        action_bus: &mut ActionBus<Action>,
    ) -> bool {
        self.event_fn
            .as_mut()
            .is_some_and(|f| f(event, workspace, state, action_bus))
    }

    #[inline]
    fn dismiss_on_outside_click(&self) -> bool {
        self.dismiss_on_outside_click
    }

    #[inline]
    fn dismiss_on_escape(&self) -> bool {
        self.dismiss_on_escape
    }

    #[inline]
    fn on_dismiss(&mut self, state: &mut AppState, action_bus: &mut ActionBus<Action>) {
        if let Some(ref mut f) = self.on_dismiss_fn {
            f(state, action_bus);
        }
    }
}

/// Autonomous coordinator and registry managing volatile interactive overlay hooks.
pub struct OverlayHost<AppState, Action: 'static + Send + Sync> {
    hooks: Vec<Box<dyn OverlayHook<AppState, Action>>>,
}

impl<AppState, Action: 'static + Send + Sync> Default for OverlayHost<AppState, Action> {
    fn default() -> Self {
        Self::new()
    }
}

impl<AppState, Action: 'static + Send + Sync> OverlayHost<AppState, Action> {
    /// Creates an empty overlay host with no registered hooks.
    #[must_use]
    pub const fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Creates an overlay host pre-allocated to hold `capacity` hooks.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            hooks: Vec::with_capacity(capacity),
        }
    }

    /// Registers an overlay hook into the host.
    ///
    /// If an overlay hook with the same [`OverlayId`] already exists, it is replaced.
    pub fn register(&mut self, hook: impl OverlayHook<AppState, Action> + 'static) {
        let id = hook.id();
        if let Some(existing) = self.hooks.iter_mut().find(|h| h.id() == id) {
            *existing = Box::new(hook);
        } else {
            self.hooks.push(Box::new(hook));
        }
    }

    /// Registers a boxed overlay hook into the host.
    pub fn register_boxed(&mut self, hook: Box<dyn OverlayHook<AppState, Action>>) {
        let id = hook.id();
        if let Some(existing) = self.hooks.iter_mut().find(|h| h.id() == id) {
            *existing = hook;
        } else {
            self.hooks.push(hook);
        }
    }

    /// Removes an overlay hook by its unique identifier.
    pub fn remove(&mut self, id: OverlayId) -> Option<Box<dyn OverlayHook<AppState, Action>>> {
        let idx = self.hooks.iter().position(|h| h.id() == id)?;
        Some(self.hooks.remove(idx))
    }

    /// Evaluates whether an overlay hook with the specified identifier is registered.
    #[must_use]
    pub fn contains(&self, id: OverlayId) -> bool {
        self.hooks.iter().any(|h| h.id() == id)
    }

    /// Evaluates whether the specified overlay hook is registered and actively presented.
    #[must_use]
    pub fn is_active(&self, id: OverlayId, state: &AppState) -> bool {
        self.hooks
            .iter()
            .find(|h| h.id() == id)
            .is_some_and(|h| h.is_active(state))
    }

    /// Evaluates whether any registered overlay hook is currently active.
    #[must_use]
    pub fn has_active_overlays(&self, state: &AppState) -> bool {
        self.hooks.iter().any(|h| h.is_active(state))
    }

    /// Evaluates whether a screen-space coordinate falls within any active overlay's interactive area.
    ///
    /// Evaluates registered hooks in top-to-bottom (reverse Z-order) sequence.
    #[must_use]
    pub fn wants_pointer_at(&self, point: Point, workspace: Rect, state: &AppState) -> bool {
        self.hooks
            .iter()
            .rev()
            .any(|h| h.is_active(state) && h.wants_pointer_at(point, workspace, state))
    }

    /// Dispatches input events to active overlay hooks in reverse Z-order.
    ///
    /// Handles outside-click dismissals, Escape key cancellations, and custom event handlers.
    /// Returns `true` if an overlay consumed the event.
    pub fn handle_event(
        &mut self,
        event: &UiEvent,
        workspace: Rect,
        state: &mut AppState,
        action_bus: &mut ActionBus<Action>,
    ) -> bool {
        // 1. Check Escape key dismissal across active overlays
        if let UiEvent::KeyboardInput {
            key: KeyCode::Escape,
            is_pressed: true,
            ..
        } = event
        {
            for hook in self.hooks.iter_mut().rev() {
                if hook.is_active(state) && hook.dismiss_on_escape() {
                    hook.on_dismiss(state, action_bus);
                    return true;
                }
            }
        }

        // 2. Check outside-click dismissal on MouseDown
        if let UiEvent::MouseDown { point, .. } = event {
            for hook in self.hooks.iter_mut().rev() {
                if hook.is_active(state)
                    && hook.dismiss_on_outside_click()
                    && !hook.wants_pointer_at(*point, workspace, state)
                {
                    hook.on_dismiss(state, action_bus);
                }
            }
        }

        // 3. Delegate event handling to active overlays
        for hook in self.hooks.iter_mut().rev() {
            if hook.is_active(state) && hook.handle_event(event, workspace, state, action_bus) {
                return true;
            }
        }

        false
    }

    /// Declaratively renders all active overlay hooks into the given UI scope.
    pub fn render(
        &mut self,
        scope: &mut UiScope<'_>,
        workspace: Rect,
        state: &mut AppState,
        action_bus: &mut ActionBus<Action>,
    ) {
        for hook in &mut self.hooks {
            if hook.is_active(state) {
                hook.render(scope, workspace, state, action_bus);
            }
        }
    }

    /// Returns the number of registered overlay hooks.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.hooks.len()
    }

    /// Evaluates whether no overlay hooks are currently registered.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }

    /// Clears all registered overlay hooks from the host.
    #[inline]
    pub fn clear(&mut self) {
        self.hooks.clear();
    }
}