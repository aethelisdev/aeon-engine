// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Root Orchestrator
//!
//! Dispatches Viewport RTT surface compositing, floating HUD toolbar, 3D orientation
//! compass, camera telemetry HUD, and Play Mode overlays purely through [`UiScope`]
//! with zero imperative `let` statements.

use super::camera_hud;
use super::compass;
use super::play_hud;
use super::toolbar;
use super::types::ViewportHudState;
use crate::ui::iris_bridge::viewport_texture::VIEWPORT_TEXTURE_ID;
use crate::ui::panel_registry::EditorPanelContext;
use crate::ui::types::{EditorFrameData, EngineUiAction};
use ae_renderer::camera::Camera;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

#[inline]
fn camera_hud_key(camera: &Camera, viewport_rect: Rect) -> ([i32; 3], i32, i32, Rect) {
    (
        [
            (camera.position.x * 10.0).round() as i32,
            (camera.position.y * 10.0).round() as i32,
            (camera.position.z * 10.0).round() as i32,
        ],
        camera.pitch.0.to_degrees().round() as i32,
        camera.yaw.0.to_degrees().round() as i32,
        viewport_rect,
    )
}

#[inline]
fn compass_key(camera: &Camera, viewport_rect: Rect) -> (i32, i32, Rect) {
    (
        (camera.pitch.0 * 1000.0).round() as i32,
        (camera.yaw.0 * 1000.0).round() as i32,
        viewport_rect,
    )
}

impl ViewportHudState {
    /// Entrypoint invoked directly by [`irisui::workbench::ScopedPanelRegistry`].
    pub(crate) fn draw_panel(&mut self, scope: &mut UiScope<'_>, ctx: &mut EditorPanelContext<'_>) {
        if ctx.data.has_viewport_texture
            && ctx.data.viewport_rect.width >= 1.0
            && ctx.data.viewport_rect.height >= 1.0
        {
            scope.external_texture(
                VIEWPORT_TEXTURE_ID,
                ctx.data.viewport_rect.width,
                ctx.data.viewport_rect.height,
            );
        }
        self.camera_angles = (ctx.data.camera.pitch.0, ctx.data.camera.yaw.0);
        self.camera_pos = (
            ctx.data.camera.position.x,
            ctx.data.camera.position.y,
            ctx.data.camera.position.z,
        );
        self.last_camera_hud_key = Some(camera_hud_key(ctx.data.camera, ctx.data.viewport_rect));
        self.last_compass_key = Some(compass_key(ctx.data.camera, ctx.data.viewport_rect));

        build_viewport_hud_scope(scope, &ctx.data, ctx.cursor_pos, ctx.action_bus);
    }

    /// In-place volatile canvas refresh invoked on `InPlaceRefresh` frames.
    pub(crate) fn refresh_panel(
        &mut self,
        scope: &mut UiScope<'_>,
        ctx: &mut EditorPanelContext<'_>,
    ) {
        self.camera_angles = (ctx.data.camera.pitch.0, ctx.data.camera.yaw.0);
        self.camera_pos = (
            ctx.data.camera.position.x,
            ctx.data.camera.position.y,
            ctx.data.camera.position.z,
        );
        if !ctx.data.is_editing || ctx.data.is_2d_mode {
            return;
        }
        if self.last_camera_hud_key != Some(camera_hud_key(ctx.data.camera, ctx.data.viewport_rect))
            && camera_hud::update_camera_hud(scope, ctx.data.camera)
        {
            self.last_camera_hud_key =
                Some(camera_hud_key(ctx.data.camera, ctx.data.viewport_rect));
        }
        if self.last_compass_key != Some(compass_key(ctx.data.camera, ctx.data.viewport_rect))
            && compass::update_compass_canvas(scope, ctx.data.camera)
        {
            self.last_compass_key = Some(compass_key(ctx.data.camera, ctx.data.viewport_rect));
        }
    }
}

/// Builds viewport HUD overlays purely through a declarative [`UiScope`].
pub fn build_viewport_hud_scope(
    scope: &mut UiScope<'_>,
    frame: &EditorFrameData<'_>,
    cursor_pos: Point,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    if frame.viewport_rect.width < 20.0 || frame.viewport_rect.height < 20.0 {
        return;
    }

    if frame.is_editing {
        toolbar::build_viewport_toolbar(
            scope,
            frame.camera,
            frame.editor_state,
            frame.snapping_settings,
            frame.is_2d_mode,
            action_bus,
        );

        scope.when(!frame.is_2d_mode, |s| {
            compass::build_scene_navigation_compass(s, frame.camera);
            camera_hud::build_camera_hud(s, frame.camera);
        });
    } else {
        play_hud::build_play_hud(
            scope,
            frame.world,
            frame.viewport_rect,
            cursor_pos,
            frame.editor_state.primary_selected(),
            action_bus,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::viewport_hud::types::with_test_frame_data;
    use crate::ui::iris_bridge::viewport_hud::{
        TAG_CAMERA_HUD, TAG_COMPASS_CANVAS, TAG_GIZMO_TRANSLATE, TAG_PLAY_EXIT, TAG_PLAY_RESUME,
        TAG_VIEWPORT_CAMERA_MODE, TAG_VIEWPORT_SHADING_MODE,
    };
    use ae_ecs::World;
    use ae_editor::editor_state::EditorState;
    use ae_editor::gizmo::GizmoMode;
    use ae_editor::snapping::SnapSettings;
    use ae_renderer::camera::{Camera, ProjectionMode};

    fn make_test_camera() -> Camera {
        Camera {
            position: cgmath::Point3::new(0.0, 5.0, 10.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 10.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        }
    }

    #[test]
    fn test_build_viewport_hud_editing_3d() {
        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let world = World::new();
        let editor_state = EditorState::default();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root creation failed");

        let mut bus = ActionBus::new();
        with_test_frame_data(
            (Rect::new(0.0, 0.0, 800.0, 600.0), true, false),
            &camera,
            &editor_state,
            &snapping,
            &world,
            |frame| {
                build_viewport_hud_scope(
                    &mut UiScope::new(&mut tree, root),
                    frame,
                    Point::new(100.0, 100.0),
                    &mut bus,
                );
            },
        );

        let tags: Vec<u64> = tree.iter().map(|(_, n)| n.tag).collect();
        assert!(tags.contains(&TAG_VIEWPORT_CAMERA_MODE));
        assert!(tags.contains(&TAG_COMPASS_CANVAS));
        assert!(tags.contains(&TAG_CAMERA_HUD));
    }

    #[test]
    fn test_build_viewport_hud_editing_2d() {
        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let world = World::new();
        let editor_state = EditorState::default();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root creation failed");

        let mut bus = ActionBus::new();
        with_test_frame_data(
            (Rect::new(0.0, 0.0, 800.0, 600.0), true, true),
            &camera,
            &editor_state,
            &snapping,
            &world,
            |frame| {
                build_viewport_hud_scope(
                    &mut UiScope::new(&mut tree, root),
                    frame,
                    Point::new(100.0, 100.0),
                    &mut bus,
                );
            },
        );

        let tags: Vec<u64> = tree.iter().map(|(_, n)| n.tag).collect();
        assert!(!tags.contains(&TAG_VIEWPORT_CAMERA_MODE));
        assert!(!tags.contains(&TAG_VIEWPORT_SHADING_MODE));
        assert!(tags.contains(&TAG_GIZMO_TRANSLATE));
        assert!(!tags.contains(&TAG_COMPASS_CANVAS));
        assert!(!tags.contains(&TAG_CAMERA_HUD));
    }

    #[test]
    fn test_build_viewport_hud_play_mode() {
        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let mut world = World::new();
        let _player = world.spawn((
            ae_plugin_api::PlayerTag,
            ae_plugin_api::CharacterAction::default(),
        ));
        let _pause = world.spawn((ae_core::ui::PauseMenuUiTag,));
        let editor_state = EditorState::default();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root creation failed");

        let mut bus = ActionBus::new();
        with_test_frame_data(
            (Rect::new(0.0, 0.0, 800.0, 600.0), false, false),
            &camera,
            &editor_state,
            &snapping,
            &world,
            |frame| {
                build_viewport_hud_scope(
                    &mut UiScope::new(&mut tree, root),
                    frame,
                    Point::new(100.0, 100.0),
                    &mut bus,
                );
            },
        );

        let tags: Vec<u64> = tree.iter().map(|(_, n)| n.tag).collect();
        assert!(!tags.contains(&TAG_VIEWPORT_CAMERA_MODE));
        assert!(!tags.contains(&TAG_COMPASS_CANVAS));
        assert!(tags.contains(&TAG_PLAY_EXIT));
        assert!(tags.contains(&TAG_PLAY_RESUME));
    }

    #[test]
    fn test_build_viewport_hud_registers_active_dropdown_tag_and_outside_click() {
        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let world = World::new();
        let editor_state = EditorState {
            gizmo_mode: GizmoMode::Select,
            ..Default::default()
        };
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root creation failed");
        let viewport_rect = Rect::new(0.0, 0.0, 800.0, 600.0);

        tree.open_popup(TAG_VIEWPORT_CAMERA_MODE);
        let mut bus = ActionBus::new();
        with_test_frame_data(
            (viewport_rect, true, false),
            &camera,
            &editor_state,
            &snapping,
            &world,
            |frame| {
                let mut scope = UiScope::new(&mut tree, root);
                build_viewport_hud_scope(&mut scope, frame, Point::new(100.0, 100.0), &mut bus);
                scope.finish_layout(viewport_rect);
            },
        );

        assert!(tree.is_popup_tag_active(TAG_VIEWPORT_CAMERA_MODE));
        assert!(tree.has_open_popup());
        assert!(
            tree.iter()
                .any(|(_, n)| n.layer == UiLayer::Popup && n.role == WidgetRole::DropdownPopup)
        );

        tree.dispatch_event(&UiEvent::MouseDown {
            point: Point::new(700.0, 500.0),
            button: MouseButton::Left,
        });
        assert!(!tree.has_open_popup());
        assert!(!tree.is_popup_tag_active(TAG_VIEWPORT_CAMERA_MODE));

        tree.clear();
        let root2 = tree.create_root().expect("Root2");
        with_test_frame_data(
            (viewport_rect, true, false),
            &camera,
            &editor_state,
            &snapping,
            &world,
            |frame| {
                let mut scope = UiScope::new(&mut tree, root2);
                build_viewport_hud_scope(&mut scope, frame, Point::new(100.0, 100.0), &mut bus);
            },
        );
        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.layer == UiLayer::Popup && n.role == WidgetRole::DropdownPopup),
            "Closed popup should not render after outside click"
        );
    }
}