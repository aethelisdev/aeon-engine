// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Floating Toolbar Builder
//!
//! Renders the top-left floating glass toolbar containing camera projection modes,
//! shading mode selector, Q/W/E/R gizmo operation buttons, and coordinate space toggle.

use super::types::{
    ViewportHudAction, ViewportHudDropdownId, ViewportHudParams, ViewportHudTargets,
};
use crate::ui::iris_bridge::icons::{
    ICON_CUBE, ICON_LIGHT, ICON_LOCAL, ICON_ROTATE, ICON_SCALE, ICON_SELECT, ICON_TRANSLATE,
    ICON_WIREFRAME, ICON_WORLD,
};
use ae_editor::gizmo::{GizmoMode, GizmoSpace};
use ae_renderer::camera::ProjectionMode;
use irisui::prelude::*;

/// Builds the top-left floating viewport toolbar with projection, shading, gizmo mode, and coordinate space controls.
/// Configured with 32×32 pixel square tool buttons with subtle rounded corners (`border_radius: 4.0`)
/// providing clear click targets, spacious icon framing, and distinct modular grouping.
pub fn build_viewport_toolbar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &ViewportHudParams<'_>,
    targets: &mut ViewportHudTargets,
) {
    let box_y = params.viewport_rect.y + 6.0;
    let box_h = 32.0;
    let corner_radius = 4.0;
    let group_gap = 8.0;

    let mut cur_x = params.viewport_rect.x + 8.0;
    // ── 1. View Modes Box: Mode Indicator / Projection Dropdown & Shading ──
    let is_persp = params.camera.mode == ProjectionMode::Perspective;
    let is_top = !is_persp && params.camera.pitch.0 < -1.5;
    let is_front = !is_persp && params.camera.pitch.0.abs() < 0.1 && params.camera.yaw.0 > 1.5;
    let is_right =
        !is_persp && params.camera.pitch.0.abs() < 0.1 && params.camera.yaw.0.abs() < 0.1;

    let (camera_label, cam_w) = if params.is_2d {
        ("2D", 44.0)
    } else if is_persp {
        ("Perspective", 98.0)
    } else if is_top {
        ("Top", 54.0)
    } else if is_front {
        ("Front", 58.0)
    } else if is_right {
        ("Right", 58.0)
    } else {
        ("Orthographic", 98.0)
    };

    let (sh_icon_uv, shading_label, sh_w) = if params.wireframe_enabled {
        (ICON_WIREFRAME, "Wireframe", 94.0)
    } else {
        (ICON_LIGHT, "Lit", 54.0)
    };

    let view_box_w = if params.is_2d {
        cam_w
    } else {
        cam_w + 1.0 + sh_w
    };
    let view_box_rect = Rect::new(cur_x, box_y, view_box_w, box_h);

    let view_box_id = tree.create_node();
    if let Some(node) = tree.get_mut(view_box_id) {
        node.set_name("ViewModesBox");
        node.computed_rect = view_box_rect;
        node.style = Style::new()
            .background(Color::rgba(0.12, 0.13, 0.16, 0.92))
            .border(1.0, Color::rgba(0.24, 0.26, 0.32, 0.85))
            .corner_radii(CornerRadii::all(corner_radius))
            .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35));
    }
    let _ = tree.add_child(parent_id, view_box_id);

    // 1.1 Camera Projection Mode Dropdown Button (In 3D) or Static 2D Badge (In 2D)
    let is_cam_open =
        !params.is_2d && params.active_dropdown == Some(ViewportHudDropdownId::CameraMode);
    let cam_rect = Rect::new(cur_x, box_y, cam_w, box_h);
    let is_cam_hover = !params.is_2d && cam_rect.contains_point(params.cursor_pos);

    let cam_btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(cam_btn_id) {
        node.set_name(if params.is_2d {
            "Mode2DBadge"
        } else {
            "CameraModeBtn"
        });
        node.computed_rect = cam_rect;
        let bg = if !params.is_2d && (is_cam_open || is_cam_hover) {
            Color::rgba(0.20, 0.23, 0.30, 0.90)
        } else {
            Color::TRANSPARENT
        };
        let radii = if params.is_2d {
            CornerRadii::all(corner_radius)
        } else {
            CornerRadii::new(corner_radius, 0.0, 0.0, corner_radius)
        };
        node.style = Style::new().background(bg).corner_radii(radii);
    }
    let _ = tree.add_child(view_box_id, cam_btn_id);

    let cam_text_color = if !params.is_2d && (is_cam_open || is_cam_hover) {
        Color::rgba(1.0, 1.0, 1.0, 1.0)
    } else {
        Color::rgba(0.85, 0.88, 0.94, 1.0)
    };

    if !params.is_2d && is_persp {
        let cube_icon_size = 18.0;
        let cube_icon_x = cur_x + 6.0;
        let cube_icon_y = box_y + (box_h - cube_icon_size) * 0.5;
        let cube_id = tree.create_node();
        if let Some(node) = tree.get_mut(cube_id) {
            node.set_name("PerspectiveCubeIcon");
            node.computed_rect =
                Rect::new(cube_icon_x, cube_icon_y, cube_icon_size, cube_icon_size);
            node.set_texture_uv(ICON_CUBE);
            node.set_texture_tint(cam_text_color);
        }
        let _ = tree.add_child(cam_btn_id, cube_id);

        let cam_txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(cam_txt_id) {
            node.set_name("CameraModeText");
            node.set_text(camera_label);
            node.font_size = 11.0;
            node.line_height = box_h;
            node.text_align = TextAlign::Center;
            node.text_color = cam_text_color;
            node.computed_rect = Rect::new(cur_x + 25.0, box_y, cam_w - 25.0, box_h);
        }
        let _ = tree.add_child(cam_btn_id, cam_txt_id);
    } else {
        let cam_txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(cam_txt_id) {
            node.set_name(if params.is_2d {
                "Mode2DText"
            } else {
                "CameraModeText"
            });
            node.set_text(camera_label);
            node.font_size = 11.0;
            node.line_height = box_h;
            node.text_align = TextAlign::Center;
            node.text_color = cam_text_color;
            node.computed_rect = cam_rect;
        }
        let _ = tree.add_child(cam_btn_id, cam_txt_id);
    }

    // Only in 3D mode: register CameraMode dropdown trigger and Shading Mode button
    if !params.is_2d {
        targets
            .dropdown_triggers
            .push((ViewportHudDropdownId::CameraMode, cam_rect));

        // Divider between Camera and Shading
        add_divider(tree, view_box_id, cur_x + cam_w, box_y + 5.0, box_h - 10.0);

        // 1.2 Shading Mode Button
        let sh_x = cur_x + cam_w + 1.0;
        let is_sh_open = params.active_dropdown == Some(ViewportHudDropdownId::ShadingMode);
        let sh_rect = Rect::new(sh_x, box_y, sh_w, box_h);
        let is_sh_hover = sh_rect.contains_point(params.cursor_pos);

        let sh_btn_id = tree.create_node();
        if let Some(node) = tree.get_mut(sh_btn_id) {
            node.set_name("ShadingModeBtn");
            node.computed_rect = sh_rect;
            let bg = if is_sh_open || is_sh_hover {
                Color::rgba(0.20, 0.23, 0.30, 0.90)
            } else {
                Color::TRANSPARENT
            };
            node.style = Style::new().background(bg).corner_radii(CornerRadii::new(
                0.0,
                corner_radius,
                corner_radius,
                0.0,
            ));
        }
        let _ = tree.add_child(view_box_id, sh_btn_id);

        let sh_text_color = if is_sh_open || is_sh_hover {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.85, 0.88, 0.94, 1.0)
        };

        let sh_icon_size = 18.0;
        let sh_icon_x = sh_x + 6.0;
        let sh_icon_y = box_y + (box_h - sh_icon_size) * 0.5;
        let sh_icon_id = tree.create_node();
        if let Some(node) = tree.get_mut(sh_icon_id) {
            node.set_name("ShadingModeIcon");
            node.computed_rect = Rect::new(sh_icon_x, sh_icon_y, sh_icon_size, sh_icon_size);
            node.set_texture_uv(sh_icon_uv);
            node.set_texture_tint(sh_text_color);
        }
        let _ = tree.add_child(sh_btn_id, sh_icon_id);

        let sh_txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(sh_txt_id) {
            node.set_name("ShadingModeText");
            node.set_text(shading_label);
            node.font_size = 11.0;
            node.line_height = box_h;
            node.text_align = TextAlign::Center;
            node.text_color = sh_text_color;
            node.computed_rect = Rect::new(sh_x + 25.0, box_y, sh_w - 25.0, box_h);
        }
        let _ = tree.add_child(sh_btn_id, sh_txt_id);
        targets
            .dropdown_triggers
            .push((ViewportHudDropdownId::ShadingMode, sh_rect));
    }

    cur_x += view_box_w + group_gap;

    // ── 2. Gizmo Tool Buttons: 4 Large 32×32 Square Boxes with Subtle Rounded Corners ──
    let btn_size = 32.0;
    let btn_gap = 4.0;

    let gizmo_modes = [
        (GizmoMode::Select, ICON_SELECT),
        (GizmoMode::Translate, ICON_TRANSLATE),
        (GizmoMode::Rotate, ICON_ROTATE),
        (GizmoMode::Scale, ICON_SCALE),
    ];

    for (mode, uv) in gizmo_modes {
        let is_selected = params.gizmo_mode == mode;
        let g_rect = Rect::new(cur_x, box_y, btn_size, btn_size);
        let is_hover = g_rect.contains_point(params.cursor_pos);

        let (bg, border_color) = if is_selected {
            (
                Color::rgba(0.06, 0.46, 0.92, 1.0),
                Color::rgba(0.25, 0.60, 1.0, 0.95),
            )
        } else if is_hover {
            (
                Color::rgba(0.20, 0.23, 0.30, 0.95),
                Color::rgba(0.35, 0.40, 0.50, 0.90),
            )
        } else {
            (
                Color::rgba(0.12, 0.13, 0.16, 0.92),
                Color::rgba(0.24, 0.26, 0.32, 0.85),
            )
        };

        let g_id = tree.create_node();
        if let Some(node) = tree.get_mut(g_id) {
            node.set_name("GizmoBtn");
            node.computed_rect = g_rect;
            node.style = Style::new()
                .background(bg)
                .border(1.0, border_color)
                .corner_radii(CornerRadii::all(corner_radius))
                .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35));
        }
        let _ = tree.add_child(parent_id, g_id);

        let icon_size = 22.0;
        let icon_x = g_rect.x + (btn_size - icon_size) * 0.5;
        let icon_y = g_rect.y + (btn_size - icon_size) * 0.5;
        let icon_rect = Rect::new(icon_x, icon_y, icon_size, icon_size);

        let g_icon = tree.create_node();
        if let Some(node) = tree.get_mut(g_icon) {
            node.set_name("GizmoBtnIcon");
            node.computed_rect = icon_rect;
            node.set_texture_uv(uv);
            let tint = if is_selected {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else if is_hover {
                Color::rgba(0.95, 0.98, 1.0, 1.0)
            } else {
                Color::rgba(0.80, 0.84, 0.90, 0.90)
            };
            node.set_texture_tint(tint);
        }
        let _ = tree.add_child(g_id, g_icon);

        targets
            .buttons
            .push((ViewportHudAction::SetGizmoMode(mode), g_rect));

        cur_x += btn_size + btn_gap;
    }

    cur_x += group_gap - btn_gap;

    // ── 3. Coordinate Space Box (World / Local) — 32×32 Square Button ──
    let is_local = params.gizmo_space == GizmoSpace::Local;
    let space_rect = Rect::new(cur_x, box_y, btn_size, btn_size);
    let is_space_hover = space_rect.contains_point(params.cursor_pos);

    let (bg, border_color) = if is_space_hover {
        (
            Color::rgba(0.20, 0.23, 0.30, 0.95),
            Color::rgba(0.35, 0.40, 0.50, 0.90),
        )
    } else {
        (
            Color::rgba(0.12, 0.13, 0.16, 0.92),
            Color::rgba(0.24, 0.26, 0.32, 0.85),
        )
    };

    let space_box_id = tree.create_node();
    if let Some(node) = tree.get_mut(space_box_id) {
        node.set_name("GizmoSpaceBox");
        node.computed_rect = space_rect;
        node.style = Style::new()
            .background(bg)
            .border(1.0, border_color)
            .corner_radii(CornerRadii::all(corner_radius))
            .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35));
    }
    let _ = tree.add_child(parent_id, space_box_id);

    let space_uv = if is_local { ICON_LOCAL } else { ICON_WORLD };
    let space_icon_size = 22.0;
    let space_icon_x = cur_x + (btn_size - space_icon_size) * 0.5;
    let space_icon_y = box_y + (btn_size - space_icon_size) * 0.5;

    let space_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(space_icon_id) {
        node.set_name("GizmoSpaceIcon");
        node.computed_rect =
            Rect::new(space_icon_x, space_icon_y, space_icon_size, space_icon_size);
        node.set_texture_uv(space_uv);
        let icon_tint = if is_local || is_space_hover {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.85, 0.88, 0.94, 1.0)
        };
        node.set_texture_tint(icon_tint);
    }
    let _ = tree.add_child(space_box_id, space_icon_id);
    targets
        .buttons
        .push((ViewportHudAction::ToggleGizmoSpace, space_rect));
}

/// Helper to render subtle vertical dividers between toolbar groups.
fn add_divider(tree: &mut UiTree, parent_id: WidgetId, x: f32, y: f32, h: f32) {
    let sep_id = tree.create_node();
    if let Some(node) = tree.get_mut(sep_id) {
        node.set_name("ToolbarDivider");
        node.style = Style::new().background(Color::rgba(1.0, 1.0, 1.0, 0.12));
        node.computed_rect = Rect::new(x, y, 1.0, h);
    }
    let _ = tree.add_child(parent_id, sep_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_editor::gizmo::{GizmoMode, GizmoSpace};
    use ae_editor::snapping::SnapSettings;
    use ae_renderer::camera::{Camera, ProjectionMode};
    use hecs::World;

    /// Verifies that the Camera Mode dropdown trigger is registered and clickable
    /// even when the camera is in `ProjectionMode::Orthographic`.
    #[test]
    fn test_toolbar_camera_mode_dropdown_trigger_registered_in_orthographic_mode() {
        let mut tree = UiTree::new();
        let parent_id = tree.create_node();
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

        let mut targets = ViewportHudTargets::default();
        build_viewport_toolbar(&mut tree, parent_id, &params, &mut targets);

        // Verify that CameraMode dropdown trigger is registered
        let cam_trigger = targets
            .dropdown_triggers
            .iter()
            .find(|(id, _)| *id == ViewportHudDropdownId::CameraMode);
        assert!(
            cam_trigger.is_some(),
            "CameraMode dropdown trigger must be present in Orthographic mode"
        );
        let (_, rect) = cam_trigger.unwrap();
        assert!(rect.width > 0.0 && rect.height > 0.0);
    }

    /// Verifies that the Camera Mode dropdown trigger is registered and clickable
    /// when the camera is in `ProjectionMode::Perspective`.
    #[test]
    fn test_toolbar_camera_mode_dropdown_trigger_registered_in_perspective_mode() {
        let mut tree = UiTree::new();
        let parent_id = tree.create_node();
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

        let mut targets = ViewportHudTargets::default();
        build_viewport_toolbar(&mut tree, parent_id, &params, &mut targets);

        let cam_trigger = targets
            .dropdown_triggers
            .iter()
            .find(|(id, _)| *id == ViewportHudDropdownId::CameraMode);
        assert!(
            cam_trigger.is_some(),
            "CameraMode dropdown trigger must be present in Perspective mode"
        );
        let (_, rect) = cam_trigger.unwrap();
        assert!(rect.width > 0.0 && rect.height > 0.0);
    }

    /// Verifies that in 2D mode (`is_2d == true`), the 3D Camera Mode dropdown trigger
    /// is suppressed and does NOT register any interactive trigger.
    #[test]
    fn test_toolbar_mode_2d_suppresses_camera_mode_dropdown() {
        let mut tree = UiTree::new();
        let parent_id = tree.create_node();
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

        let mut targets = ViewportHudTargets::default();
        build_viewport_toolbar(&mut tree, parent_id, &params, &mut targets);

        let cam_trigger = targets
            .dropdown_triggers
            .iter()
            .find(|(id, _)| *id == ViewportHudDropdownId::CameraMode);
        assert!(
            cam_trigger.is_none(),
            "CameraMode dropdown trigger must be suppressed in 2D mode"
        );

        let sh_trigger = targets
            .dropdown_triggers
            .iter()
            .find(|(id, _)| *id == ViewportHudDropdownId::ShadingMode);
        assert!(
            sh_trigger.is_none(),
            "ShadingMode dropdown trigger must be suppressed in 2D mode"
        );
    }
}