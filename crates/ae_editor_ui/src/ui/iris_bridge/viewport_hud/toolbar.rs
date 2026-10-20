// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Floating Toolbar Builder
//!
//! Renders the top-left floating glass toolbar containing camera projection modes,
//! shading mode selector, Q/W/E/R gizmo operation buttons, and coordinate space toggle.

use super::types::{
    TAG_GIZMO_ROTATE, TAG_GIZMO_SCALE, TAG_GIZMO_SELECT, TAG_GIZMO_SPACE_TOGGLE,
    TAG_GIZMO_TRANSLATE, TAG_VIEWPORT_SHADING_MODE, ViewportHudAction, ViewportHudDropdownId,
    ViewportHudParams, ViewportHudState,
};
use crate::ui::iris_bridge::icons::{
    ICON_LOCAL, ICON_ROTATE, ICON_SCALE, ICON_SELECT, ICON_TRANSLATE, ICON_WORLD,
};
use ae_editor::gizmo::{GizmoMode, GizmoSpace};
use irisui::prelude::*;

/// Builds the top-left floating viewport toolbar with projection, shading, gizmo mode, and coordinate space controls.
///
/// Built entirely using the generic declarative [`Toolbar`] and [`ToolbarGroup`] container widgets
/// without manual rect arithmetic or external action vector bags. Dispatches interactions
/// directly onto the editor's [`ViewportHudState`] action queue via fluent `.on_click` closures.
pub fn build_viewport_toolbar(
    scope: &mut UiScope<'_>,
    params: &ViewportHudParams<'_>,
    hud: &mut ViewportHudState,
) {
    let cam = params.camera_orientation();
    let sh = params.shading_mode();
    let is_cam_open =
        !params.is_2d && params.active_dropdown == Some(ViewportHudDropdownId::CameraMode);
    let is_sh_open =
        !params.is_2d && params.active_dropdown == Some(ViewportHudDropdownId::ShadingMode);

    Toolbar::floating("ViewportToolbar", 8.0, 6.0)
        .width(params.toolbar_total_width())
        .transparent()
        .gap(8.0)
        .show(scope, |tb| {
            // ── 1. View Modes Container (Camera Mode + 1px Vertical Divider + Shading Mode) ──
            tb.group_styled(
                ToolbarGroup::new()
                    .width(params.view_modes_group_width())
                    .gap(0.0)
                    .background(Color::rgba(0.08, 0.09, 0.12, 0.92))
                    .border_color(Color::rgba(0.24, 0.26, 0.32, 0.85))
                    .corner_radius(4.0),
                |group| {
                    group
                        .mode_button_tagged(
                            cam.icon_uv(),
                            cam.label(),
                            cam.tag(),
                            is_cam_open,
                            cam.width(),
                            cam.corner_radii(),
                        )
                        .on_click(|| {
                            if !params.is_2d {
                                hud.dispatch(ViewportHudAction::ToggleDropdown(if is_cam_open {
                                    None
                                } else {
                                    Some(ViewportHudDropdownId::CameraMode)
                                }));
                            }
                        });

                    if !params.is_2d {
                        group.divider();

                        group
                            .mode_button_tagged(
                                Some(sh.icon_uv()),
                                sh.label(),
                                TAG_VIEWPORT_SHADING_MODE,
                                is_sh_open,
                                sh.width(),
                                sh.corner_radii(),
                            )
                            .on_click(|| {
                                hud.dispatch(ViewportHudAction::ToggleDropdown(if is_sh_open {
                                    None
                                } else {
                                    Some(ViewportHudDropdownId::ShadingMode)
                                }));
                            });
                    }
                },
            );

            // ── 2. Gizmo Tool Buttons: 4 Large 32×32 Square Boxes with 4px gap ──
            let gizmo_modes = [
                (GizmoMode::Select, ICON_SELECT, TAG_GIZMO_SELECT),
                (GizmoMode::Translate, ICON_TRANSLATE, TAG_GIZMO_TRANSLATE),
                (GizmoMode::Rotate, ICON_ROTATE, TAG_GIZMO_ROTATE),
                (GizmoMode::Scale, ICON_SCALE, TAG_GIZMO_SCALE),
            ];

            tb.group_styled(ToolbarGroup::new().gap(4.0).transparent(), |gg| {
                for (mode, uv, tag) in gizmo_modes {
                    let is_selected = params.gizmo_mode == mode;
                    gg.selectable_tool_tagged(is_selected, uv, tag)
                        .on_click(|| hud.dispatch(ViewportHudAction::SetGizmoMode(mode)));
                }
            });

            // ── 3. Coordinate Space Box (World / Local) — 32×32 Square Button ──
            let is_local = params.gizmo_space == GizmoSpace::Local;
            let space_uv = if is_local { ICON_LOCAL } else { ICON_WORLD };
            tb.toggle_tool_tagged(is_local, space_uv, TAG_GIZMO_SPACE_TOGGLE)
                .on_click(|| hud.dispatch(ViewportHudAction::ToggleGizmoSpace));
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::viewport_hud::TAG_VIEWPORT_CAMERA_MODE;
    use ae_editor::gizmo::{GizmoMode, GizmoSpace};
    use ae_editor::snapping::SnapSettings;
    use ae_renderer::camera::{Camera, ProjectionMode};
    use hecs::World;

    /// Verifies that the Camera Mode dropdown trigger is registered with `TAG_VIEWPORT_CAMERA_MODE`
    /// even when the camera is in `ProjectionMode::Orthographic`.
    #[test]
    fn test_toolbar_camera_mode_dropdown_trigger_registered_in_orthographic_mode() {
        let mut tree = UiTree::new();
        let parent_id = tree.create_root().expect("Root node creation failed");
        let camera = Camera {
            position: cgmath::Point3::new(0.0, 5.0, 10.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ProjectionMode::Orthographic,
            ortho_scale: 10.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        };
        let snapping = SnapSettings::default();
        let world = World::new();

        let params = ViewportHudParams {
            viewport_rect: Rect::new(0.0, 0.0, 1280.0, 720.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Select,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(20.0, 15.0),
            active_dropdown: None,
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        let mut hud = ViewportHudState::default();
        build_viewport_toolbar(&mut UiScope::new(&mut tree, parent_id), &params, &mut hud);
        let mut scope = UiScope::new(&mut tree, parent_id);
        scope.finish_layout(params.viewport_rect);

        // Verify that CameraMode dropdown trigger node with tag exists and has positive computed bounds
        let cam_node = tree
            .iter()
            .find(|(_, node)| node.tag == TAG_VIEWPORT_CAMERA_MODE);
        assert!(
            cam_node.is_some(),
            "CameraMode dropdown node must be present in Orthographic mode"
        );
        let (_, node) = cam_node.unwrap();
        assert!(node.computed_rect.width > 0.0 && node.computed_rect.height > 0.0);
    }

    /// Verifies that the Camera Mode dropdown trigger is registered with `TAG_VIEWPORT_CAMERA_MODE`
    /// when the camera is in `ProjectionMode::Perspective`.
    #[test]
    fn test_toolbar_camera_mode_dropdown_trigger_registered_in_perspective_mode() {
        let mut tree = UiTree::new();
        let parent_id = tree.create_root().expect("Root node creation failed");
        let camera = Camera {
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
        };
        let snapping = SnapSettings::default();
        let world = World::new();

        let params = ViewportHudParams {
            viewport_rect: Rect::new(0.0, 0.0, 1280.0, 720.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Select,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(20.0, 15.0),
            active_dropdown: None,
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        let mut hud = ViewportHudState::default();
        build_viewport_toolbar(&mut UiScope::new(&mut tree, parent_id), &params, &mut hud);
        let mut scope = UiScope::new(&mut tree, parent_id);
        scope.finish_layout(params.viewport_rect);

        let cam_node = tree
            .iter()
            .find(|(_, node)| node.tag == TAG_VIEWPORT_CAMERA_MODE);
        assert!(
            cam_node.is_some(),
            "CameraMode dropdown node must be present in Perspective mode"
        );
        let (_, node) = cam_node.unwrap();
        assert!(node.computed_rect.width > 0.0 && node.computed_rect.height > 0.0);
    }

    /// Verifies that in 2D mode (`is_2d == true`), the 3D Camera Mode dropdown trigger
    /// is suppressed and does NOT register any interactive trigger.
    #[test]
    fn test_toolbar_mode_2d_suppresses_camera_mode_dropdown() {
        let mut tree = UiTree::new();
        let parent_id = tree.create_root().expect("Root node creation failed");
        let camera = Camera {
            position: cgmath::Point3::new(0.0, 0.0, 10.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ProjectionMode::Orthographic,
            ortho_scale: 10.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        };
        let snapping = SnapSettings::default();
        let world = World::new();

        let params = ViewportHudParams {
            viewport_rect: Rect::new(0.0, 0.0, 1280.0, 720.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Select,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(20.0, 15.0),
            active_dropdown: None,
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: true,
        };

        let mut hud = ViewportHudState::default();
        build_viewport_toolbar(&mut UiScope::new(&mut tree, parent_id), &params, &mut hud);

        let cam_node = tree
            .iter()
            .find(|(_, node)| node.tag == TAG_VIEWPORT_CAMERA_MODE);
        assert!(
            cam_node.is_none(),
            "CameraMode dropdown trigger must be suppressed in 2D mode"
        );

        let sh_node = tree
            .iter()
            .find(|(_, node)| node.tag == TAG_VIEWPORT_SHADING_MODE);
        assert!(
            sh_node.is_none(),
            "ShadingMode dropdown trigger must be suppressed in 2D mode"
        );
    }

    /// Verifies that clicking gizmo mode and space toggle buttons on the toolbar
    /// dispatches [`ViewportHudAction`] directly into [`ViewportHudState::actions`].
    #[test]
    fn test_toolbar_button_click_dispatches_actions() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");
        let camera = Camera {
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
        };
        let snapping = SnapSettings::default();
        let world = World::new();

        let params = ViewportHudParams {
            viewport_rect: Rect::new(0.0, 0.0, 1280.0, 720.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Select,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(20.0, 15.0),
            active_dropdown: None,
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        // 1. Initial build & layout
        let mut hud = ViewportHudState::default();
        build_viewport_toolbar(&mut UiScope::new(&mut tree, root), &params, &mut hud);
        let mut scope = UiScope::new(&mut tree, root);
        scope.finish_layout(params.viewport_rect);

        // Find translate button node
        let (translate_id, translate_rect) = tree
            .iter()
            .find(|(_, n)| n.tag == TAG_GIZMO_TRANSLATE)
            .map(|(id, n)| (id, n.computed_rect))
            .expect("Translate gizmo button node must exist");

        // 2. Simulate click on Translate button
        let click_pt = Point::new(translate_rect.x + 4.0, translate_rect.y + 4.0);
        tree.dispatch_event(&UiEvent::MouseDown {
            point: click_pt,
            button: MouseButton::Left,
        });
        tree.dispatch_event(&UiEvent::MouseUp {
            point: click_pt,
            button: MouseButton::Left,
        });

        assert_eq!(tree.event_state.frame_clicked_id, Some(translate_id));

        // 3. Rebuild toolbar with active click event state
        let mut rebuild_tree = UiTree::new();
        let rebuild_root = rebuild_tree.create_root().expect("Root creation failed");
        rebuild_tree.event_state = tree.event_state;

        build_viewport_toolbar(
            &mut UiScope::new(&mut rebuild_tree, rebuild_root),
            &params,
            &mut hud,
        );

        assert!(
            hud.actions
                .contains(&ViewportHudAction::SetGizmoMode(GizmoMode::Translate)),
            "Clicking Translate button must dispatch ViewportHudAction::SetGizmoMode(Translate): {:?}",
            hud.actions
        );
    }
}