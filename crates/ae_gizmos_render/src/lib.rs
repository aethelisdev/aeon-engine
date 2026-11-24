// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Aeon Gizmos Render Crate
//!
//! WGPU pipeline and [`ae_renderer::render::OverlayRenderer`] implementations
//! for 3D transform gizmos, collider debug wireframes, and 3D entity billboards.

pub mod billboard_renderer;
pub mod debug_renderer;
pub mod gizmo_renderer;

pub use billboard_renderer::{BillboardPipeline, BillboardPrepareParams, BillboardSystem};
pub use debug_renderer::DebugRenderer;
pub use gizmo_renderer::{GizmoOverlayPrepareParams, GizmoRenderer};