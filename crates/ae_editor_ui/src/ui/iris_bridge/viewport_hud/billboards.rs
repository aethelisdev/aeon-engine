// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 3D Viewport Billboard Icons Builder
//!
//! Viewport billboard icon badges (Light, Camera, Audio) have been migrated to the native
//! 3D forward overlay render pass in `ae_editor::billboard::BillboardSystem` to achieve
//! 0.0ms latency, native 144Hz+ synchronization, and preserve 2D Iris UI sleep mode.

use super::types::{ViewportHudParams, ViewportHudTargets};
use irisui::prelude::*;

/// Retained for architectural interface compatibility. Viewport billboard rendering
/// is handled with 0.0ms latency directly inside the 3D forward overlay pipeline.
pub fn build_billboard_icons(
    _tree: &mut UiTree,
    _parent_id: WidgetId,
    _params: &ViewportHudParams<'_>,
    _targets: &mut ViewportHudTargets,
) {
    // Migrated to 3D WGPU Overlay pass (ae_editor::billboard::BillboardSystem)
}