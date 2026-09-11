// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Hardware UI Geometry Baking and Two-Pass Extraction Engine.
//!
//! Traverses the Iris `UiTree` hierarchically to extract and sort rendering instances
//! into Layer 0 (SDF quads) and Layer 1 (2D Texture Array quads), ensuring zero per-card
//! pipeline switches on the GPU.
//!

use super::geometry::BakedUiGeometry;
use ae_core::telemetry::FrameRingBuffer;
use irisui::prelude::*;
use irisui::wgpu_backend::{ExternalTextureQuadInstance, QuadInstance, TextureQuadInstance};

/// Bakes the complete UI tree into decoupled hardware vertex instance buffers.
/// Recursively walks the generational arena tree starting from `root`, mapping every
/// visible container, card background, border, icon, and oscilloscope canvas into
/// contiguous instance layers.
pub fn bake_ui_geometry(
    geometry: &mut BakedUiGeometry,
    tree: &UiTree,
    root: WidgetId,
    frame_pacing: Option<&FrameRingBuffer>,
) {
    geometry.clear_retaining_capacity();
    bake_node_recursive(geometry, tree, root, None, frame_pacing);
}

/// Recursive visitor that classifies and extracts vertex quad instances by hardware pipeline layer.
fn bake_node_recursive(
    geometry: &mut BakedUiGeometry,
    tree: &UiTree,
    current: WidgetId,
    clip_rect: Option<Rect>,
    frame_pacing: Option<&FrameRingBuffer>,
) {
    let Some(node) = tree.get(current) else {
        return;
    };
    if !node.visible {
        return;
    }

    let child_clip = if node.style.clip_children {
        match clip_rect {
            Some(existing) => Some(existing.intersect(node.computed_rect)),
            None => Some(node.computed_rect),
        }
    } else {
        clip_rect
    };

    // Layer 0: SDF Quad (container background, border, shadow, rounded corners)
    if node.computed_rect.width > 0.0
        && node.computed_rect.height > 0.0
        && (node.style.background_color.a > 0.0
            || node.style.border.width.top > 0.0
            || node.style.border.width.right > 0.0
            || node.style.border.width.bottom > 0.0
            || node.style.border.width.left > 0.0
            || node.style.box_shadow.is_some())
    {
        let quad_idx = geometry.sdf_instances.len();
        geometry.sdf_instances.push(QuadInstance::from_style(
            node.computed_rect,
            &node.style,
            clip_rect,
        ));
        geometry.node_to_sdf_idx.insert(current, quad_idx);
    }

    // Layer 1: 2D Texture Array Quad (vector icon, thumbnail view)
    if let Some(uv) = node.texture_uv
        && node.computed_rect.width > 0.0
        && node.computed_rect.height > 0.0
    {
        let tint = node.texture_tint.unwrap_or(Color::WHITE);
        let clip_arr = match clip_rect {
            Some(c) => [c.x, c.y, c.x + c.width, c.y + c.height],
            None => [0.0, 0.0, 0.0, 0.0],
        };
        geometry.texture_instances.push(TextureQuadInstance {
            rect: [
                node.computed_rect.x,
                node.computed_rect.y,
                node.computed_rect.width,
                node.computed_rect.height,
            ],
            uv_rect: uv,
            tint: [tint.r, tint.g, tint.b, tint.a],
            clip_rect: clip_arr,
        });
    }

    // Layer 2: External Texture Quad (viewport render target, video texture)
    if let Some(id) = node.external_texture
        && node.computed_rect.width > 0.0
        && node.computed_rect.height > 0.0
    {
        let tint = node.texture_tint.unwrap_or(Color::WHITE);
        let uv = node.texture_uv.unwrap_or([0.0, 0.0, 1.0, 1.0]);
        geometry.external_texture_quads.push((
            id,
            ExternalTextureQuadInstance::with_uv(node.computed_rect, uv, tint, clip_rect),
        ));
    }

    // Oscilloscope Telemetry Trace (appended to Layer 0 SDF instances)
    if node.role == WidgetRole::OscilloscopeCanvas
        && let Some(ring) = frame_pacing
    {
        let mut osc_list = DrawCommandList::new();
        crate::ui::iris_bridge::stats::append_oscilloscope_quads(
            &mut osc_list,
            node.computed_rect,
            ring,
        );
        geometry.sdf_instances.extend(osc_list.quads);
    }

    // Recursively process child nodes in strict hierarchy order
    for &child in &node.children {
        bake_node_recursive(geometry, tree, child, child_clip, frame_pacing);
    }
}