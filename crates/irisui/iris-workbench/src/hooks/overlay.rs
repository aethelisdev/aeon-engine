// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Volatile Hardware Overlay Hooks
//!
//! Provides traits and closures for rendering dynamic, per-frame overlay graphics
//! (telemetry oscilloscopes, 3D compass orientation gizmos, HUD reticles, audio meters)
//! on top of workbench panels via immediate-mode canvas command emission.

use iris_core::Rect;
use iris_widgets::canvas::ImmediateCanvas;

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