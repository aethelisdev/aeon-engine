// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Retained-mode Baked UI Geometry and Hardware Batching Buffer Management.
//!
//! Provides `BakedUiGeometry` to aggregate all SDF containers into Layer 0 (1st Draw Call)
//! and all 2D texture array views into Layer 1 (2nd Draw Call), completely eliminating
//! per-card pipeline state changes and reducing draw calls to a constant 2.
//!

use irisui::prelude::*;
use irisui::wgpu_backend::{ExternalTextureQuadInstance, QuadInstance, TextureQuadInstance};
use std::collections::HashMap;

/// Pre-allocated hardware geometry buffers for 100% batched Iris UI rendering.
/// Segregates UI elements into discrete rendering passes to guarantee exactly
/// 2 WGPU draw calls regardless of how many cards or complex widgets are present:
/// - Layer 0: SDF Quads (`DrawCommand::DrawSdfQuads`)
/// - Layer 1: 2D Texture Array Quads (`DrawCommand::DrawTexture`)
/// - Layer 2: External Texture Quads (`DrawCommand::DrawExternalTexture`)
#[derive(Debug, Clone, Default)]
pub struct BakedUiGeometry {
    /// Layer 0: All SDF quads (containers, panel bodies, card backgrounds, borders, shadows, badges).
    pub sdf_instances: Vec<QuadInstance>,
    /// Layer 1: All 2D Texture Array quads (vector icons, folder glyphs, asset thumbnails).
    pub texture_instances: Vec<TextureQuadInstance>,
    /// Layer 2: All External Texture quads (viewports, custom render targets).
    pub external_texture_quads: Vec<(ExternalTextureId, ExternalTextureQuadInstance)>,
    /// Fast direct lookup mapping `WidgetId` to its primary `sdf_instances` index.
    pub node_to_sdf_idx: HashMap<WidgetId, usize>,
}

impl BakedUiGeometry {
    /// Constructs a new `BakedUiGeometry` instance with pre-reserved default capacities.
    /// Pre-allocating capacity avoids reallocations during initial panel population.
    pub fn new() -> Self {
        Self {
            sdf_instances: Vec::with_capacity(512),
            texture_instances: Vec::with_capacity(256),
            external_texture_quads: Vec::with_capacity(8),
            node_to_sdf_idx: HashMap::with_capacity(512),
        }
    }

    /// Resets all buffers for baking while strictly retaining existing heap capacity.
    /// Conforms to Rule 9 by ensuring zero memory reallocations during per-frame UI passes.
    pub fn clear_retaining_capacity(&mut self) {
        self.sdf_instances.clear();
        self.texture_instances.clear();
        self.external_texture_quads.clear();
        self.node_to_sdf_idx.clear();
    }

    /// Bakes the UI tree geometry into decoupled hardware vertex instance buffers.
    pub fn bake(
        &mut self,
        tree: &UiTree,
        root: WidgetId,
        frame_pacing: Option<&ae_core::telemetry::FrameRingBuffer>,
    ) {
        super::engine::bake_ui_geometry(self, tree, root, frame_pacing);
    }

    /// Mutates an existing SDF quad instance in-place at the given baking index.
    /// Enables O(1) hover and selection visual updates directly inside the vertex instance
    /// buffer without re-traversing the tree or rebuilding draw commands.
    pub fn update_card_style(&mut self, baking_quad_idx: usize, style: &Style) {
        if let Some(quad) = self.sdf_instances.get_mut(baking_quad_idx) {
            let bg_linear = (style
                .background_color
                .with_alpha(style.background_color.a * style.opacity))
            .to_linear()
            .to_array();
            let border_linear = style.border.color.to_linear().to_array();

            quad.color = bg_linear;
            quad.border_color = border_linear;
            quad.border_width = style.border.width.to_array();
            quad.corner_radii = style.corner_radii.to_array();
        }
    }

    /// Emits batched hardware draw commands into the target `DrawCommandList`.
    /// Guarantees that:
    /// 1. Exactly one `DrawCommand::DrawSdfQuads` is emitted for all SDF geometry.
    /// 2. Exactly one `DrawCommand::DrawTexture` is emitted for all 2D texture array geometry.
    /// 3. One `DrawCommand::DrawExternalTexture` is emitted per unique external texture.
    pub fn apply_to_command_list(&self, command_list: &mut DrawCommandList) {
        command_list.commands.clear();
        command_list.quads.clear();
        command_list.texture_quads.clear();
        command_list.external_texture_quads.clear();

        // Layer 0: External Texture Quads (3D/2D Viewport scene render target)
        // Must be rendered first as the canvas layer so that all dock panels,
        // Gizmo tool buttons, icons, and overlays render cleanly on top.
        for (id, quad) in &self.external_texture_quads {
            let instance_index = command_list.external_texture_quads.len() as u32;
            command_list.external_texture_quads.push(*quad);
            command_list
                .commands
                .push(DrawCommand::DrawExternalTexture {
                    id: *id,
                    instance_index,
                });
        }

        // Layer 1: SDF Quads (all panels, containers, card backgrounds, borders, Gizmo button boxes)
        if !self.sdf_instances.is_empty() {
            command_list.quads.extend_from_slice(&self.sdf_instances);
            command_list.commands.push(DrawCommand::DrawSdfQuads {
                start: 0,
                count: self.sdf_instances.len() as u32,
            });
        }

        // Layer 2: 2D Texture Array Quads (UI icons, Gizmo button glyphs, folder logos, thumbnails)
        if !self.texture_instances.is_empty() {
            command_list
                .texture_quads
                .extend_from_slice(&self.texture_instances);
            command_list.commands.push(DrawCommand::DrawTexture {
                start: 0,
                count: self.texture_instances.len() as u32,
            });
        }
    }
}