// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Viewport image, game UI, and editor HUD composition within native Iris panel layers.

use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use super::viewport_hud::{self, ViewportHudParams, ViewportHudTargets};
use super::viewport_texture::VIEWPORT_TEXTURE_ID;
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds viewport content (3D scene texture quad and interactive HUD) under the owning panel.
    pub(crate) fn build_viewport_content(
        &mut self,
        parent: WidgetId,
        viewport_rect: Rect,
        params: &OverlayUpdateParams<'_>,
    ) {
        if viewport_rect.width <= 20.0 || viewport_rect.height <= 20.0 {
            return;
        }

        // 1. Root container for the viewport canvas with clipping
        let root = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(root) {
            node.set_name("IrisViewportRoot");
            node.computed_rect = viewport_rect;
            let mut style = Style::new().background(Color::TRANSPARENT);
            style.clip_children = true;
            node.set_style(style);
        }
        let _ = self.tree.add_child(parent, root);

        // 2. Viewport 3D RTT Texture Quad or Placeholder
        if params.has_viewport_texture {
            let image_node = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(image_node) {
                node.set_name("IrisViewportImage");
                node.computed_rect = viewport_rect;
                node.external_texture = Some(VIEWPORT_TEXTURE_ID);
            }
            let _ = self.tree.add_child(root, image_node);
        } else {
            let placeholder_node = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(placeholder_node) {
                node.set_name("IrisViewportPlaceholder");
                node.computed_rect = viewport_rect;
                node.set_text("Rendering viewport...");
                node.set_text_properties(14.0, 18.0, Color::hex("#888888"), TextAlign::Center);
            }
            let _ = self.tree.add_child(root, placeholder_node);
        }

        // 3. Viewport HUD Overlays (Gizmo, toolbar, camera info, projection modes)
        let mut hud_targets = ViewportHudTargets::default();
        viewport_hud::build_viewport_hud(
            &mut self.tree,
            root,
            &ViewportHudParams {
                viewport_rect,
                camera: params.camera,
                wireframe_enabled: params.wireframe_enabled,
                gizmo_mode: params.gizmo_mode,
                gizmo_space: params.gizmo_space,
                snapping: params.snapping_settings,
                cursor_pos: self.cursor_pos,
                active_dropdown: self.viewport_hud_dropdown,
                selected_entity: params.selected_entity,
                world: params.world,
                is_editing: params.is_editing,
            },
            &mut hud_targets,
        );
        self.viewport_hud_targets = Some(hud_targets);
    }
}