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
    /// Constructs the viewport layout tree declaratively via [`UiScope`]: root container,
    /// hardware 3D RTT texture quad, and the complete interactive HUD overlay.
    pub(crate) fn build_viewport_content(
        &mut self,
        parent: WidgetId,
        viewport_rect: Rect,
        params: &OverlayUpdateParams<'_>,
    ) {
        if viewport_rect.width <= 20.0 || viewport_rect.height <= 20.0 {
            return;
        }

        let root_style = Style::new()
            .position_absolute()
            .left(viewport_rect.x)
            .top(viewport_rect.y)
            .width(viewport_rect.width)
            .height(viewport_rect.height)
            .background(Color::TRANSPARENT)
            .clip_children(true);

        let mut scope = UiScope::new(&mut self.tree, parent);
        let root = scope.container_passive(root_style, |vp_scope| {
            if params.viewport.has_viewport_texture {
                vp_scope.external_texture(
                    VIEWPORT_TEXTURE_ID,
                    viewport_rect.width,
                    viewport_rect.height,
                );
            } else {
                let ph_style = Style::new()
                    .position_absolute()
                    .left(0.0)
                    .top(0.0)
                    .width(viewport_rect.width)
                    .height(viewport_rect.height)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center);
                vp_scope.container_passive(ph_style, |ph| {
                    ph.label(
                        "Rendering viewport...",
                        14.0,
                        Color::hex("#888888"),
                        TextAlign::Center,
                    );
                });
            }
        });

        // 3. Viewport HUD Overlays (Gizmo, toolbar, camera info, projection modes)
        let cursor_pos = self.cursor_pos();
        let active_dropdown = self.viewport_hud.dropdown;
        viewport_hud::build_viewport_hud(
            &mut self.tree,
            root,
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
        );
        self.viewport_hud.camera_angles =
            (params.viewport.camera.pitch.0, params.viewport.camera.yaw.0);
        self.viewport_hud.is_active = true;
    }
}