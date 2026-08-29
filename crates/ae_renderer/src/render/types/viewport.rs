// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport & Frame Telemetry Metrics
//!
//! Viewport rectangle hit-testing, generic overlay rendering interfaces, and per-frame draw call telemetry.

/// ViewportRect represents screen boundaries of the 3D Viewport.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ViewportRect {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl ViewportRect {
    /// Returns true if logical coordinates (x, y) fall strictly inside the viewport boundaries.
    #[inline]
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }
}

/// Viewport descriptor for sub-region rendering.
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub position: [f32; 2],
    pub size: [f32; 2],
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            size: [800.0, 600.0],
        }
    }
}

/// Generic overlay renderer trait for drawing editor overlays into the main render pass.
/// Systems that need to draw overlays (e.g. gizmos, debug lines) implement this trait.
/// RenderState calls `draw_overlay()` without knowing the concrete type, achieving full
/// decoupling between the render module and editor subsystems.
pub trait OverlayRenderer {
    /// Draw the overlay into an already-active render pass.
    /// Implementors should have already prepared their GPU state (uniforms, vertex data)
    /// via a separate `prepare()` call before this is invoked.
    fn draw_overlay<'a>(&'a self, queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'a>);
}

/// Render error type for surface acquisition failures.
#[derive(Debug)]
pub enum RenderError {
    SurfaceLost,
    OutOfMemory,
    Other(String),
}

/// Live per-frame rendering and geometry metrics collected during render passes.
/// Tracks total GPU draw calls, batched calls, instanced calls, compute passes,
/// culled meshes, triangles, vertices, and rendered instance count for real-time profiling.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FrameRenderStats {
    pub draw_calls: u32,
    pub batched_draw_calls: u32,
    pub instanced_draw_calls: u32,
    pub dispatched_compute: u32,
    pub culled_meshes: u32,
    pub triangles: u64,
    pub vertices: u64,
    pub entities_rendered: u32,
}

impl FrameRenderStats {
    /// Converts `FrameRenderStats` into an `ae_core::telemetry::DrawCallBreakdown` struct.
    pub fn to_breakdown(&self) -> ae_core::telemetry::DrawCallBreakdown {
        ae_core::telemetry::DrawCallBreakdown {
            total_draw_calls: self.draw_calls,
            batched_draw_calls: self.batched_draw_calls,
            instanced_draw_calls: self.instanced_draw_calls,
            dispatched_compute: self.dispatched_compute,
            culled_meshes: self.culled_meshes,
            triangles: self.triangles,
            vertices: self.vertices,
        }
    }
}