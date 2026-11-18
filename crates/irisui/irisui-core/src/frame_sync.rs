// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Frame Lifecycle & GPU Surface Sync Signals (`irisui-core::frame_sync`)
//!
//! Pure data descriptors bridging workbench/lifecycle evaluation with GPU rendering backends
//! without introducing GPU crate dependencies into workbench crates.

/// Lifecycle refresh classification consumed by GPU surface renderers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceRefreshMode {
    /// Full shell or granular panel subtree rebuild requiring static/volatile typography sync.
    StructuralRebuild {
        /// Whether the entire UI shell was rebuilt and requires a full quad upload.
        is_full_shell: bool,
    },
    /// Incremental live canvas/telemetry refresh eligible for in-place boundary quad patching.
    InPlaceRefresh,
    /// Steady idle frame with no structural rebuilds; only pending paint/hover patches run.
    Dormant,
}

/// Decoupled frame invalidation summary synchronized onto GPU render surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IrisSurfaceFrameFlags {
    /// Structural rebuild vs incremental refresh classification for the frame.
    pub mode: SurfaceRefreshMode,
    /// Whether caret blink phase flipped this frame and requires static text preparation.
    pub caret_blink_redraw: bool,
    /// Aggregate text preparation requested for this frame.
    pub needs_text_prepare: bool,
    /// High-frequency volatile text layer preparation requested for this frame.
    pub volatile_needs_text_prepare: bool,
    /// Whether retained quad draw commands must be recompiled or boundary-patched.
    pub needs_quad_reupload: bool,
}