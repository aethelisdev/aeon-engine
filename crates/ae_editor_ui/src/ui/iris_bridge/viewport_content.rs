// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Viewport image, game UI, and editor HUD composition within native Iris panel layers.

use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use super::viewport_hud::{self, ViewportHudParams};
use super::viewport_texture::VIEWPORT_TEXTURE_ID;
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds viewport content (3D scene texture quad and interactive HUD) under the owning panel.
    ///
    /// Constructs the viewport layout tree declaratively via [`UiScope::viewport_surface`]: root container,
    /// hardware 3D RTT texture quad, and the complete interactive HUD overlay.
    pub(crate) fn build_viewport_content(
        &mut self,
        scope: &mut UiScope<'_>,
        viewport_rect: Rect,
        params: &OverlayUpdateParams<'_>,
    ) {
        if viewport_rect.width <= 20.0 || viewport_rect.height <= 20.0 {
            return;
        }

        let texture_hook = params
            .viewport
            .has_viewport_texture
            .then(|| ExternalTextureHook::new(VIEWPORT_TEXTURE_ID.0, viewport_rect));

        let cursor_pos = self.cursor_pos();
        let active_dropdown = self.viewport_hud.dropdown;
        let hud = &mut self.viewport_hud;

        scope.viewport_surface(texture_hook, Some("Rendering viewport..."), |vp_scope| {
            // Viewport HUD Overlays (Gizmo, toolbar, camera info, projection modes)
            viewport_hud::build_viewport_hud_scope(
                vp_scope,
                &ViewportHudParams {
                    viewport_rect,
                    camera: params.viewport.camera,
                    wireframe_enabled: params.viewport.wireframe_enabled,
                    gizmo_mode: params.viewport.gizmo_mode,
                    gizmo_space: params.viewport.gizmo_space,
                    snapping: params.preferences.snapping_settings,
                    cursor_pos,
                    active_dropdown,
                    selected_entity: params.scene.selected_entity,
                    world: params.scene.world,
                    is_editing: params.context.is_editing,
                    is_2d: params.context.is_2d_mode,
                },
                hud,
            );
        });

        hud.camera_angles = (params.viewport.camera.pitch.0, params.viewport.camera.yaw.0);
        hud.camera_pos = (
            params.viewport.camera.position.x,
            params.viewport.camera.position.y,
            params.viewport.camera.position.z,
        );
    }
}