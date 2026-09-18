// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Dropdown Popup Renderer
//!
//! Renders top-layer floating popup menus for Camera Projection Modes and Shading Modes
//! using standardized [`ComboboxPopupBuilder`].

use super::types::{
    ViewportHudAction, ViewportHudDropdownId, ViewportHudParams, ViewportHudTargets,
};
use ae_renderer::camera::{Camera, ProjectionMode};
use irisui::prelude::*;
use irisui::widgets::{ComboboxPopupBuilder, ComboboxPopupStyle};

/// Renders active floating popup menus in the Viewport HUD.
pub fn render_viewport_hud_dropdown_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    active_dd: ViewportHudDropdownId,
    params: &ViewportHudParams<'_>,
    targets: &ViewportHudTargets,
) {
    let Some(&(_, btn_rect)) = targets
        .dropdown_triggers
        .iter()
        .find(|(id, _)| *id == active_dd)
    else {
        return;
    };

    let (labels, selected_index): (Vec<&str>, Option<usize>) = match active_dd {
        ViewportHudDropdownId::CameraMode => {
            let is_persp = params.camera.mode == ProjectionMode::Perspective;
            let is_top = !is_persp && params.camera.pitch.0 < -1.5;
            let is_front =
                !is_persp && params.camera.pitch.0.abs() < 0.1 && params.camera.yaw.0 > 1.5;
            let is_right =
                !is_persp && params.camera.pitch.0.abs() < 0.1 && params.camera.yaw.0.abs() < 0.1;
            let is_ortho = !is_persp && !is_top && !is_front && !is_right;

            let selected = if is_persp {
                Some(0)
            } else if is_ortho {
                Some(1)
            } else if is_top {
                Some(2)
            } else if is_front {
                Some(3)
            } else if is_right {
                Some(4)
            } else {
                None
            };
            (
                vec![
                    "Perspective",
                    "📐 Orthographic",
                    "📐 Top",
                    "📐 Front",
                    "📐 Right",
                ],
                selected,
            )
        }
        ViewportHudDropdownId::ShadingMode => {
            let selected = if params.wireframe_enabled {
                Some(1)
            } else {
                Some(0)
            };
            (vec!["Lit", "Wireframe"], selected)
        }
    };

    let style = ComboboxPopupStyle {
        background: Color::rgba(0.07, 0.08, 0.11, 0.98),
        border_width: 1.0,
        border_color: Color::rgba(0.24, 0.28, 0.38, 0.70),
        border_radius: 4.0,
        shadow_y: 6.0,
        shadow_blur: 16.0,
        shadow_color: Color::rgba(0.0, 0.0, 0.0, 0.75),
        item_idle_bg: Color::TRANSPARENT,
        item_hover_bg: Color::rgba(0.20, 0.23, 0.32, 0.95),
        item_selected_bg: Color::rgba(0.0, 0.35, 0.45, 0.85),
        text_idle_color: Color::rgba(0.85, 0.88, 0.95, 1.0),
        text_hover_color: Color::WHITE,
        text_selected_color: Color::rgba(0.0, 0.90, 1.0, 1.0),
        font_size: 11.0,
        row_height: 22.0,
        item_padding_x: 6.0,
    };

    ComboboxPopupBuilder::new(btn_rect)
        .name("ViewportHudPopup")
        .min_width(130.0)
        .items(&labels)
        .selected_index(selected_index)
        .cursor_pos(params.cursor_pos)
        .style(style)
        .build(tree, parent_id);
}

/// Resolves a dispatched [`ViewportHudAction`] from a selected dropdown option index.
#[must_use]
pub fn resolve_viewport_hud_dropdown_action(
    dropdown_id: ViewportHudDropdownId,
    index: usize,
    camera: &Camera,
) -> Option<ViewportHudAction> {
    match dropdown_id {
        ViewportHudDropdownId::CameraMode => {
            let target = camera.target;
            let d = 10.0;
            match index {
                0 => Some(ViewportHudAction::SetCameraMode(
                    ProjectionMode::Perspective,
                )),
                1 => Some(ViewportHudAction::SetCameraMode(
                    ProjectionMode::Orthographic,
                )),
                2 => Some(ViewportHudAction::SetCameraTransform {
                    pitch: cgmath::Rad(-std::f32::consts::FRAC_PI_2 + 0.001),
                    yaw: cgmath::Rad(0.0),
                    position: cgmath::Point3::new(target.x, target.y + d, target.z),
                    mode: Some(ProjectionMode::Orthographic),
                }),
                3 => Some(ViewportHudAction::SetCameraTransform {
                    pitch: cgmath::Rad(0.0),
                    yaw: cgmath::Rad(std::f32::consts::FRAC_PI_2),
                    position: cgmath::Point3::new(target.x, target.y, target.z - d),
                    mode: Some(ProjectionMode::Orthographic),
                }),
                4 => Some(ViewportHudAction::SetCameraTransform {
                    pitch: cgmath::Rad(0.0),
                    yaw: cgmath::Rad(0.0),
                    position: cgmath::Point3::new(target.x - d, target.y, target.z),
                    mode: Some(ProjectionMode::Orthographic),
                }),
                _ => None,
            }
        }
        ViewportHudDropdownId::ShadingMode => match index {
            0 | 1 => Some(ViewportHudAction::ToggleWireframe),
            _ => None,
        },
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
        let _ = tree.set_root(root);

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
            is_2d: false,
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
            &targets,
        );

        // Verify that hit_test_target finds the dropdown items
        let hit_item1 = tree
            .hit_test_target(Point::new(20.0, 75.0))
            .expect("Orthographic item must be hit");
        assert_eq!(hit_item1.layer, UiLayer::Popup);
        assert_eq!(hit_item1.role, WidgetRole::DropdownItem);
        assert_eq!(hit_item1.tag, 1);

        // Verify action resolution for Orthographic (index 1)
        assert_eq!(
            resolve_viewport_hud_dropdown_action(ViewportHudDropdownId::CameraMode, 1, &camera),
            Some(ViewportHudAction::SetCameraMode(
                ProjectionMode::Orthographic
            ))
        );

        // Verify action resolution for Top, Front, Right (indices 2, 3, 4)
        for idx in [2, 3, 4] {
            match resolve_viewport_hud_dropdown_action(
                ViewportHudDropdownId::CameraMode,
                idx,
                &camera,
            ) {
                Some(ViewportHudAction::SetCameraTransform { mode, .. }) => {
                    assert_eq!(mode, Some(ProjectionMode::Orthographic));
                }
                other => panic!(
                    "Expected SetCameraTransform action at index {}, got {:?}",
                    idx, other
                ),
            }
        }
    }
}