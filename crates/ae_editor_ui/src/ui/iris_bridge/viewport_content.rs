// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Viewport image, game UI, and editor HUD composition within native Iris panel layers.

use super::types::OverlayUpdateParams;
use super::viewport_hud::{self, ViewportHudParams, ViewportHudState};
use super::viewport_texture::VIEWPORT_TEXTURE_ID;
use crate::ui::types::EngineUiAction;
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
        action_bus: &mut irisui::workbench::ActionBus<EngineUiAction>,
    ) {
        self.reset_live_cache();
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
                    action_bus,
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

    /// Updates camera HUD text and compass lines in place during `InPlaceRefresh` frames.
    pub(crate) fn refresh_live(
        &mut self,
        scope: &mut UiScope<'_>,
        cam: &ae_renderer::camera::Camera,
        vp_rect: Rect,
    ) {
        self.refresh_camera_hud_live(
            scope,
            cam,
            (
                [
                    (cam.position.x * 10.0).round() as i32,
                    (cam.position.y * 10.0).round() as i32,
                    (cam.position.z * 10.0).round() as i32,
                ],
                cam.pitch.0.to_degrees().round() as i32,
                cam.yaw.0.to_degrees().round() as i32,
                vp_rect,
            ),
        );
        self.refresh_compass_live(
            scope,
            cam,
            (
                (cam.pitch.0 * 1000.0).round() as i32,
                (cam.yaw.0 * 1000.0).round() as i32,
                vp_rect,
            ),
        );
    }

    #[inline]
    fn refresh_camera_hud_live(
        &mut self,
        scope: &mut UiScope<'_>,
        cam: &ae_renderer::camera::Camera,
        key: ([i32; 3], i32, i32, Rect),
    ) {
        if self.last_camera_hud_key != Some(key) {
            self.last_camera_hud_key = Some(key);
            viewport_hud::camera_hud::update_camera_hud(scope, cam);
        }
    }

    #[inline]
    fn refresh_compass_live(
        &mut self,
        scope: &mut UiScope<'_>,
        cam: &ae_renderer::camera::Camera,
        key: (i32, i32, Rect),
    ) {
        if self.last_compass_key != Some(key) {
            self.last_compass_key = Some(key);
            viewport_hud::compass::update_compass_canvas(scope, cam);
        }
    }
}