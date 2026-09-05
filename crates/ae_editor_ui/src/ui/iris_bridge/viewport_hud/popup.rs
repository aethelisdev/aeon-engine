// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Dropdown Popup Renderer
//!
//! Renders top-layer floating popup menus for Camera Projection Modes and Shading Modes.

use super::types::{
    ViewportHudAction, ViewportHudDropdownId, ViewportHudParams, ViewportHudTargets,
};
use ae_renderer::camera::ProjectionMode;
use irisui::prelude::*;

/// Renders active floating popup menus in the Viewport HUD.
pub fn render_viewport_hud_dropdown_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    active_dd: ViewportHudDropdownId,
    params: &ViewportHudParams<'_>,
    targets: &mut ViewportHudTargets,
) {
    let Some(&(_, btn_rect)) = targets
        .dropdown_triggers
        .iter()
        .find(|(id, _)| *id == active_dd)
    else {
        return;
    };

    let (items_count, options): (usize, Vec<(String, ViewportHudAction, bool)>) = match active_dd {
        ViewportHudDropdownId::CameraMode => {
            let is_persp = params.camera.mode == ProjectionMode::Perspective;
            let is_top = !is_persp && params.camera.pitch.0 < -1.5;
            let is_front =
                !is_persp && params.camera.pitch.0.abs() < 0.1 && params.camera.yaw.0 > 1.5;
            let is_right =
                !is_persp && params.camera.pitch.0.abs() < 0.1 && params.camera.yaw.0.abs() < 0.1;
            let is_ortho = !is_persp && !is_top && !is_front && !is_right;

            let target = params.camera.target;
            let d = 10.0;

            let opts = vec![
                (
                    "Perspective".to_string(),
                    ViewportHudAction::SetCameraMode(ProjectionMode::Perspective),
                    is_persp,
                ),
                (
                    "📐 Orthographic".to_string(),
                    ViewportHudAction::SetCameraMode(ProjectionMode::Orthographic),
                    is_ortho,
                ),
                (
                    "📐 Top".to_string(),
                    ViewportHudAction::SetCameraTransform {
                        pitch: cgmath::Rad(-std::f32::consts::FRAC_PI_2 + 0.001),
                        yaw: cgmath::Rad(0.0),
                        position: cgmath::Point3::new(target.x, target.y + d, target.z),
                        mode: Some(ProjectionMode::Orthographic),
                    },
                    is_top,
                ),
                (
                    "📐 Front".to_string(),
                    ViewportHudAction::SetCameraTransform {
                        pitch: cgmath::Rad(0.0),
                        yaw: cgmath::Rad(std::f32::consts::FRAC_PI_2),
                        position: cgmath::Point3::new(target.x, target.y, target.z - d),
                        mode: Some(ProjectionMode::Orthographic),
                    },
                    is_front,
                ),
                (
                    "📐 Right".to_string(),
                    ViewportHudAction::SetCameraTransform {
                        pitch: cgmath::Rad(0.0),
                        yaw: cgmath::Rad(0.0),
                        position: cgmath::Point3::new(target.x + d, target.y, target.z),
                        mode: Some(ProjectionMode::Orthographic),
                    },
                    is_right,
                ),
            ];
            (opts.len(), opts)
        }
        ViewportHudDropdownId::ShadingMode => {
            let opts = vec![
                (
                    "Lit".to_string(),
                    ViewportHudAction::ToggleWireframe,
                    !params.wireframe_enabled,
                ),
                (
                    "Wireframe".to_string(),
                    ViewportHudAction::ToggleWireframe,
                    params.wireframe_enabled,
                ),
            ];
            (opts.len(), opts)
        }
    };

    let popup_h = (items_count as f32) * 22.0 + 4.0;
    let popup_rect = Rect::new(
        btn_rect.x,
        btn_rect.y + btn_rect.height + 2.0,
        btn_rect.width.max(130.0),
        popup_h,
    );
    targets.active_dropdown_popup_rect = Some(popup_rect);

    let popup_id = tree.create_node();
    if let Some(node) = tree.get_mut(popup_id) {
        node.set_name("ViewportHudPopup");
        node.computed_rect = popup_rect;
        node.style = Style::new()
            .background(Color::rgba(0.07, 0.08, 0.11, 0.98))
            .border(1.0, Color::rgba(0.24, 0.28, 0.38, 0.70))
            .border_radius(4.0)
            .box_shadow(0.0, 6.0, 16.0, Color::rgba(0.0, 0.0, 0.0, 0.75));
    }
    let _ = tree.add_child(parent_id, popup_id);

    for (idx, (label, action, is_selected)) in options.into_iter().enumerate() {
        let item_y = popup_rect.y + 2.0 + (idx as f32) * 22.0;
        let item_rect = Rect::new(popup_rect.x + 2.0, item_y, popup_rect.width - 4.0, 20.0);
        let is_hovered = item_rect.contains_point(params.cursor_pos);

        let item_id = tree.create_node();
        if let Some(node) = tree.get_mut(item_id) {
            node.set_name("ViewportHudPopupItem");
            node.computed_rect = item_rect;
            let bg = if is_selected {
                Color::rgba(0.0, 0.35, 0.45, 0.85)
            } else if is_hovered {
                Color::rgba(0.20, 0.23, 0.32, 0.95)
            } else {
                Color::rgba(0.0, 0.0, 0.0, 0.0)
            };
            node.style = Style::new().background(bg).border_radius(3.0);
        }
        let _ = tree.add_child(popup_id, item_id);

        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("ViewportHudItemText");
            node.set_text(&label);
            node.font_size = 11.0;
            node.line_height = 20.0;
            node.text_color = if is_selected {
                Color::rgba(0.0, 0.90, 1.0, 1.0)
            } else if is_hovered {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::rgba(0.85, 0.88, 0.95, 1.0)
            };
            node.computed_rect =
                Rect::new(item_rect.x + 6.0, item_rect.y, item_rect.width - 12.0, 20.0);
        }
        let _ = tree.add_child(item_id, lbl_id);

        targets
            .active_dropdown_items
            .push((action, item_rect, label));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_editor::gizmo::{GizmoMode, GizmoSpace};
    use ae_editor::snapping::SnapSettings;
    use ae_renderer::camera::{Camera, ProjectionMode};
    use hecs::World;

    /// Tests that the camera mode dropdown provides Perspective, Orthographic, and orthogonal presets
    /// that properly enforce `ProjectionMode::Orthographic`.
    #[test]
    fn test_camera_mode_dropdown_options_and_orthographic_mode() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
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
            viewport_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Select,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(0.0, 0.0),
            active_dropdown: Some(ViewportHudDropdownId::CameraMode),
            selected_entity: None,
            world: &world,
            is_editing: true,
        };

        let mut targets = ViewportHudTargets::default();
        targets.dropdown_triggers.push((
            ViewportHudDropdownId::CameraMode,
            Rect::new(10.0, 10.0, 100.0, 30.0),
        ));

        render_viewport_hud_dropdown_popup(
            &mut tree,
            root,
            ViewportHudDropdownId::CameraMode,
            &params,
            &mut targets,
        );

        assert_eq!(targets.active_dropdown_items.len(), 5);

        let labels: Vec<&str> = targets
            .active_dropdown_items
            .iter()
            .map(|(_, _, l)| l.as_str())
            .collect();
        assert_eq!(
            labels,
            vec![
                "Perspective",
                "📐 Orthographic",
                "📐 Top",
                "📐 Front",
                "📐 Right"
            ]
        );

        // Verify that Orthographic sets camera mode
        assert_eq!(
            targets.active_dropdown_items[1].0,
            ViewportHudAction::SetCameraMode(ProjectionMode::Orthographic)
        );

        // Verify that Top, Front, Right include mode: Some(ProjectionMode::Orthographic)
        for idx in [2, 3, 4] {
            match &targets.active_dropdown_items[idx].0 {
                ViewportHudAction::SetCameraTransform { mode, .. } => {
                    assert_eq!(*mode, Some(ProjectionMode::Orthographic));
                }
                other => panic!(
                    "Expected SetCameraTransform action at index {}, got {:?}",
                    idx, other
                ),
            }
        }
    }
}