// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Floating Toolbar Builder
//!
//! Renders the top-left floating glass toolbar containing camera projection modes,
//! shading mode selector, Q/W/E/R gizmo operation buttons, and coordinate space toggle.

use super::types::{
    TAG_GIZMO_ROTATE, TAG_GIZMO_SCALE, TAG_GIZMO_SELECT, TAG_GIZMO_SPACE_TOGGLE,
    TAG_GIZMO_TRANSLATE, TAG_VIEWPORT_CAMERA_MODE, TAG_VIEWPORT_SHADING_MODE,
    ViewportHudDropdownId, ViewportHudParams,
};
use crate::ui::iris_bridge::icons::{
    ICON_CUBE, ICON_LIGHT, ICON_LOCAL, ICON_ROTATE, ICON_SCALE, ICON_SELECT, ICON_TRANSLATE,
    ICON_WIREFRAME, ICON_WORLD,
};
use ae_editor::gizmo::{GizmoMode, GizmoSpace};
use ae_renderer::camera::ProjectionMode;
use irisui::prelude::*;

/// Builds the top-left floating viewport toolbar with projection, shading, gizmo mode, and coordinate space controls.
///
/// Built entirely using declarative [`UiScope`] containers, buttons, and flexbox distribution.
/// Configured with 32×32 pixel square tool buttons with subtle rounded corners (`border_radius: 4.0`)
/// providing clear click targets, spacious icon framing, and distinct modular grouping.
///
/// Positioned using declarative absolute layout ([`Style::position_absolute`], [`Style::left`], [`Style::top`]).
/// Interactions and mode switching are driven 100% by semantic tags dispatched through the UI tree.
pub fn build_viewport_toolbar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &ViewportHudParams<'_>,
) {
    let box_h = 32.0;
    let corner_radius = 4.0;

    let is_persp = params.camera.mode == ProjectionMode::Perspective;
    let is_top = !is_persp && params.camera.pitch.0 < -1.5;
    let is_front = !is_persp && params.camera.pitch.0.abs() < 0.1 && params.camera.yaw.0 > 1.5;
    let is_right =
        !is_persp && params.camera.pitch.0.abs() < 0.1 && params.camera.yaw.0.abs() < 0.1;

    let (camera_label, cam_w, cam_icon) = if params.is_2d {
        ("2D", 44.0, None)
    } else if is_persp {
        ("Perspective", 116.0, Some(ICON_CUBE))
    } else if is_top {
        ("Top", 54.0, None)
    } else if is_front {
        ("Front", 58.0, None)
    } else if is_right {
        ("Right", 58.0, None)
    } else {
        ("Orthographic", 116.0, None)
    };

    let (sh_icon_uv, shading_label, sh_w) = if params.wireframe_enabled {
        (ICON_WIREFRAME, "Wireframe", 94.0)
    } else {
        (ICON_LIGHT, "Lit", 54.0)
    };

    let is_cam_open =
        !params.is_2d && params.active_dropdown == Some(ViewportHudDropdownId::CameraMode);
    let is_sh_open =
        !params.is_2d && params.active_dropdown == Some(ViewportHudDropdownId::ShadingMode);

    let view_box_w = if params.is_2d {
        cam_w
    } else {
        cam_w + 1.0 + sh_w
    };

    let total_w = view_box_w + 8.0 + 140.0 + 8.0 + 32.0;

    let mut scope = UiScope::new(tree, parent_id);

    // Toolbar Root Container: Flex Row with 8px group gap, absolutely anchored top-left
    let tb_style = Style::new()
        .position_absolute()
        .left(8.0)
        .top(6.0)
        .width(total_w)
        .height(box_h)
        .flex_row()
        .align_items(AlignItems::Center)
        .gap(8.0);

    scope.container(tb_style, |tb| {
        // ── 1. View Modes Container (Camera Mode + 1px Vertical Divider + Shading Mode) ──
        let view_box_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .width(view_box_w)
            .height(box_h)
            .background(Color::rgba(0.12, 0.13, 0.16, 0.92))
            .border(1.0, Color::rgba(0.24, 0.26, 0.32, 0.85))
            .corner_radii(CornerRadii::all(corner_radius))
            .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35));

        tb.container(view_box_style, |vbox| {
            let cam_radii = if params.is_2d {
                CornerRadii::all(corner_radius)
            } else {
                CornerRadii::new(corner_radius, 0.0, 0.0, corner_radius)
            };
            let cam_tag = if params.is_2d {
                0
            } else {
                TAG_VIEWPORT_CAMERA_MODE
            };
            vbox.toolbar_mode_button(
                cam_icon,
                camera_label,
                cam_tag,
                is_cam_open,
                cam_w,
                cam_radii,
            );

            if !params.is_2d {
                // 1px Vertical Divider between Camera and Shading
                vbox.vertical_divider(box_h - 10.0, Color::rgba(1.0, 1.0, 1.0, 0.12));

                // Shading Mode Button
                vbox.toolbar_mode_button(
                    Some(sh_icon_uv),
                    shading_label,
                    TAG_VIEWPORT_SHADING_MODE,
                    is_sh_open,
                    sh_w,
                    CornerRadii::new(0.0, corner_radius, corner_radius, 0.0),
                );
            }
        });

        // ── 2. Gizmo Tool Buttons: 4 Large 32×32 Square Boxes with 4px gap ──
        let gizmo_modes = [
            (GizmoMode::Select, ICON_SELECT, TAG_GIZMO_SELECT),
            (GizmoMode::Translate, ICON_TRANSLATE, TAG_GIZMO_TRANSLATE),
            (GizmoMode::Rotate, ICON_ROTATE, TAG_GIZMO_ROTATE),
            (GizmoMode::Scale, ICON_SCALE, TAG_GIZMO_SCALE),
        ];

        let gizmo_group_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(4.0);

        tb.container(gizmo_group_style, |gg| {
            for (mode, uv, tag) in gizmo_modes {
                let is_selected = params.gizmo_mode == mode;
                gg.toolbar_icon_button(uv, tag, is_selected);
            }
        });

        // ── 3. Coordinate Space Box (World / Local) — 32×32 Square Button ──
        let is_local = params.gizmo_space == GizmoSpace::Local;
        let space_uv = if is_local { ICON_LOCAL } else { ICON_WORLD };
        tb.toolbar_icon_button(space_uv, TAG_GIZMO_SPACE_TOGGLE, is_local);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
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

        build_viewport_toolbar(&mut tree, parent_id, &params);
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

        build_viewport_toolbar(&mut tree, parent_id, &params);
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

        build_viewport_toolbar(&mut tree, parent_id, &params);

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
}