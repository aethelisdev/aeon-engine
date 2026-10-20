// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Dropdown Popup Renderer
//!
//! Renders top-layer floating popup menus for Camera Projection Modes and Shading Modes
//! using standardized [`ComboboxPopupBuilder`].

use super::types::{
    CameraViewOrientation, TAG_POPUP_CAMERA_BASE, TAG_POPUP_SHADING_BASE, ViewportHudAction,
    ViewportHudDropdownId, ViewportHudParams, ViewportHudState,
};
use ae_renderer::camera::{Camera, ProjectionMode};
use irisui::prelude::*;

/// Renders active floating popup menus in the Viewport HUD.
pub fn render_viewport_hud_dropdown_popup(
    scope: &mut UiScope<'_>,
    active_dd: ViewportHudDropdownId,
    params: &ViewportHudParams<'_>,
    hud: &mut ViewportHudState,
) {
    let cam_orient = params.camera_orientation();
    let (popup_w, popup_tag_base, items, selected_index) = match active_dd {
        ViewportHudDropdownId::CameraMode => {
            let selected = match cam_orient {
                CameraViewOrientation::Perspective => Some(0),
                CameraViewOrientation::Orthographic => Some(1),
                CameraViewOrientation::Top => Some(2),
                CameraViewOrientation::Front => Some(3),
                CameraViewOrientation::Right => Some(4),
                CameraViewOrientation::TwoDimensional => None,
            };
            (
                150.0,
                TAG_POPUP_CAMERA_BASE,
                vec![
                    ("🎥", "Perspective"),
                    ("📐", "Orthographic"),
                    ("📐", "Top"),
                    ("📐", "Front"),
                    ("📐", "Right"),
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
            (
                124.0,
                TAG_POPUP_SHADING_BASE,
                vec![("💡", "Lit"), ("🕸", "Wireframe")],
                selected,
            )
        }
    };

    let popup_x = match active_dd {
        ViewportHudDropdownId::CameraMode => 8.0,
        ViewportHudDropdownId::ShadingMode => 8.0 + cam_orient.width() + 1.0,
    };
    let popup_y = 6.0 + 32.0 + 2.0;

    scope.dropdown_menu_card(popup_x, popup_y, popup_w, |menu| {
        for (idx, (icon, label)) in items.into_iter().enumerate() {
            let is_selected = selected_index == Some(idx);
            let indicator = if is_selected { Some("✓") } else { None };
            let tag = popup_tag_base + idx as u64;
            menu.dropdown_item(tag, icon, label, indicator, true)
                .on_click(|| {
                    hud.dispatch(ViewportHudAction::SelectDropdownItem(active_dd, idx));
                    hud.dispatch(ViewportHudAction::ToggleDropdown(None));
                });
        }
    });
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
                0 => Some(ViewportHudAction::SetCameraTransform {
                    pitch: cgmath::Rad(-0.4),
                    yaw: cgmath::Rad(0.6),
                    position: cgmath::Point3::new(
                        target.x + d * 0.7,
                        target.y + d * 0.5,
                        target.z + d * 0.7,
                    ),
                    mode: Some(ProjectionMode::Perspective),
                }),
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
                    position: cgmath::Point3::new(target.x + d, target.y, target.z),
                    mode: Some(ProjectionMode::Orthographic),
                }),
                _ => None,
            }
        }
        ViewportHudDropdownId::ShadingMode => match index {
            0 => Some(ViewportHudAction::SetWireframe(false)),
            1 => Some(ViewportHudAction::SetWireframe(true)),
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

        let mut hud = ViewportHudState::default();
        render_viewport_hud_dropdown_popup(
            &mut UiScope::new(&mut tree, root),
            ViewportHudDropdownId::CameraMode,
            &params,
            &mut hud,
        );

        // Verify that hit_test_target finds the dropdown items
        let hit_item1 = tree
            .hit_test_target(Point::new(20.0, 75.0))
            .expect("Orthographic item must be hit");
        assert_eq!(hit_item1.layer, UiLayer::Popup);
        assert_eq!(hit_item1.role, WidgetRole::DropdownItem);
        assert_eq!(hit_item1.tag, TAG_POPUP_CAMERA_BASE + 1);

        // Verify action resolution for Orthographic (index 1)
        assert_eq!(
            resolve_viewport_hud_dropdown_action(ViewportHudDropdownId::CameraMode, 1, &camera),
            Some(ViewportHudAction::SetCameraMode(
                ProjectionMode::Orthographic
            ))
        );

        // Verify action resolution for Perspective (index 0) restores 3D orbit angle
        match resolve_viewport_hud_dropdown_action(ViewportHudDropdownId::CameraMode, 0, &camera) {
            Some(ViewportHudAction::SetCameraTransform {
                mode, pitch, yaw, ..
            }) => {
                assert_eq!(mode, Some(ProjectionMode::Perspective));
                assert!((pitch.0 - (-0.4)).abs() < 1e-4);
                assert!((yaw.0 - 0.6).abs() < 1e-4);
            }
            other => panic!(
                "Expected SetCameraTransform for Perspective, got {:?}",
                other
            ),
        }

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

    #[test]
    fn test_camera_mode_dropdown_docked_offset_positioning_and_text_collection() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");
        let viewport_rect = Rect::new(260.0, 32.0, 800.0, 600.0);

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
            viewport_rect,
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Select,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(280.0, 50.0),
            active_dropdown: Some(ViewportHudDropdownId::CameraMode),
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        let mut hud = ViewportHudState::default();
        render_viewport_hud_dropdown_popup(
            &mut UiScope::new(&mut tree, root),
            ViewportHudDropdownId::CameraMode,
            &params,
            &mut hud,
        );

        // Verify popup is positioned relative to viewport container at x = 8.0, y = 40.0
        let popup_node = tree
            .iter()
            .find(|(_, n)| n.role == WidgetRole::DropdownPopup)
            .map(|(_, n)| n)
            .expect("DropdownPopup node must be in the UI tree");

        assert_eq!(
            popup_node.computed_rect.x, 8.0,
            "Popup must be anchored at local x = 8.0 beneath floating toolbar"
        );
        assert_eq!(
            popup_node.computed_rect.y, 40.0,
            "Popup must be anchored below the toolbar at y = 40.0"
        );

        // Verify that hit testing finds the Perspective item (tag TAG_POPUP_CAMERA_BASE)
        let hit_item0 = tree
            .hit_test_target(Point::new(20.0, popup_node.computed_rect.y + 12.0))
            .expect("Perspective item must be hit");
        assert_eq!(hit_item0.tag, TAG_POPUP_CAMERA_BASE);

        // Verify that text sections are collected without being clipped by parent's clip_children
        let sections = irisui::text::collect_text_sections(&tree);
        let labels: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();
        assert!(
            labels.contains(&"Perspective"),
            "Perspective label must be collected and not culled: {:?}",
            labels
        );
        assert!(
            labels.contains(&"Orthographic"),
            "Orthographic label must be collected: {:?}",
            labels
        );
    }
}