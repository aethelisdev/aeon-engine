// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Viewport image, game UI, and editor HUD composition within native Iris panel layers.

use super::types::OverlayUpdateParams;
use super::viewport_hud::{self, ViewportHudParams, ViewportHudState};
use super::viewport_texture::VIEWPORT_TEXTURE_ID;
use irisui::prelude::*;

impl ViewportHudState {
    /// Builds viewport content (3D scene texture quad and interactive HUD) under the owning panel.
    ///
    /// Constructs the viewport layout tree declaratively via [`UiScope::viewport_surface`]: root container,
    /// hardware 3D RTT texture quad, and the complete interactive HUD overlay.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        viewport_rect: Rect,
        params: &OverlayUpdateParams<'_>,
        cursor_pos: Point,
    ) {
        if viewport_rect.width <= 20.0 || viewport_rect.height <= 20.0 {
            return;
        }

        scope.viewport_surface(
            params.viewport.has_viewport_texture.then(|| {
                ExternalTextureView::new(
                    VIEWPORT_TEXTURE_ID.0,
                    viewport_rect.width,
                    viewport_rect.height,
                )
                .with_aspect_ratio(AspectRatio::Stretch)
            }),
            Some("Rendering viewport..."),
            |vp_scope| {
                viewport_hud::build_viewport_hud_scope(
                    vp_scope,
                    &ViewportHudParams {
                        viewport_rect,
                        camera: params.viewport.camera,
                        wireframe_enabled: params.viewport.wireframe_enabled,
                        gizmo_mode: params.viewport.gizmo_mode,
                        gizmo_space: params.viewport.gizmo_space,
                        snapping: params.preferences.snapping_settings,
                        cursor_pos: vp_scope.resolved_cursor_pos(cursor_pos),
                        selected_entity: params.scene.selected_entity,
                        world: params.scene.world,
                        is_editing: params.context.is_editing,
                        is_2d: params.context.is_2d_mode,
                    },
                    self,
                );

                params
                    .panel_data
                    .asset_browser
                    .drag_payload
                    .as_ref()
                    .into_iter()
                    .for_each(|payload| {
                        super::assets::build_asset_drag_overlays_scope(
                            vp_scope,
                            payload,
                            vp_scope.resolved_cursor_pos(cursor_pos),
                            viewport_rect,
                            params.viewport.camera,
                            params.context.is_2d_mode,
                        );
                    });
            },
        );

        self.camera_angles = (params.viewport.camera.pitch.0, params.viewport.camera.yaw.0);
        self.camera_pos = (
            params.viewport.camera.position.x,
            params.viewport.camera.position.y,
            params.viewport.camera.position.z,
        );
    }
}