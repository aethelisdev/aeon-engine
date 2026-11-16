// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Dropdown Popup Renderer
//!
//! Renders top-layer floating popup menus for Camera Projection Modes and Shading Modes
//! using standardized [`ComboboxPopupBuilder`].

use super::types::{
    CameraViewOrientation, TAG_POPUP_CAMERA_BASE, TAG_POPUP_SHADING_BASE, TAG_POPUP_SNAP_GRID_BASE,
    ViewportHudAction, ViewportHudDropdownId, ViewportHudParams, ViewportHudState,
};
use ae_renderer::camera::{Camera, ProjectionMode};
use irisui::prelude::*;
use irisui::widgets::declarative::scope::PopupPlacement;

const CAMERA_MODE_ITEMS: &[(&str, &str)] = &[
    ("🎥", "Perspective"),
    ("📐", "Orthographic"),
    ("📐", "Top"),
    ("📐", "Front"),
    ("📐", "Right"),
];

const SHADING_MODE_ITEMS: &[(&str, &str)] = &[("💡", "Lit"), ("🕸", "Wireframe")];

const GRID_SNAP_ITEMS: &[(&str, &str)] = &[
    ("📏", "0.05m"),
    ("📏", "0.1m"),
    ("📏", "0.25m"),
    ("📏", "0.5m"),
    ("📏", "1.0m"),
    ("📏", "5.0m"),
    ("📏", "10.0m"),
];

const fn dropdown_width(dd: ViewportHudDropdownId) -> f32 {
    match dd {
        ViewportHudDropdownId::CameraMode => 150.0,
        ViewportHudDropdownId::ShadingMode => 124.0,
        ViewportHudDropdownId::GridSnap => 110.0,
    }
}

const fn dropdown_tag_base(dd: ViewportHudDropdownId) -> u64 {
    match dd {
        ViewportHudDropdownId::CameraMode => TAG_POPUP_CAMERA_BASE,
        ViewportHudDropdownId::ShadingMode => TAG_POPUP_SHADING_BASE,
        ViewportHudDropdownId::GridSnap => TAG_POPUP_SNAP_GRID_BASE,
    }
}

const fn dropdown_items(dd: ViewportHudDropdownId) -> &'static [(&'static str, &'static str)] {
    match dd {
        ViewportHudDropdownId::CameraMode => CAMERA_MODE_ITEMS,
        ViewportHudDropdownId::ShadingMode => SHADING_MODE_ITEMS,
        ViewportHudDropdownId::GridSnap => GRID_SNAP_ITEMS,
    }
}

fn dropdown_selected_index(
    dd: ViewportHudDropdownId,
    params: &ViewportHudParams<'_>,
) -> Option<usize> {
    match dd {
        ViewportHudDropdownId::CameraMode => match params.camera_orientation() {
            CameraViewOrientation::Perspective => Some(0),
            CameraViewOrientation::Orthographic => Some(1),
            CameraViewOrientation::Top => Some(2),
            CameraViewOrientation::Front => Some(3),
            CameraViewOrientation::Right => Some(4),
            CameraViewOrientation::TwoDimensional => None,
        },
        ViewportHudDropdownId::ShadingMode => Some(usize::from(params.wireframe_enabled)),
        ViewportHudDropdownId::GridSnap => match params.snapping.grid_size {
            s if (s - 0.05).abs() < 0.001 => Some(0),
            s if (s - 0.1).abs() < 0.001 => Some(1),
            s if (s - 0.25).abs() < 0.001 => Some(2),
            s if (s - 0.5).abs() < 0.001 => Some(3),
            s if (s - 1.0).abs() < 0.001 => Some(4),
            s if (s - 5.0).abs() < 0.001 => Some(5),
            s if (s - 10.0).abs() < 0.001 => Some(6),
            _ => None,
        },
    }
}

fn dropdown_fallback_rect(dd: ViewportHudDropdownId, params: &ViewportHudParams<'_>) -> Rect {
    match dd {
        ViewportHudDropdownId::CameraMode => Rect::new(
            params.viewport_rect.x + 8.0,
            params.viewport_rect.y + 6.0,
            params.camera_orientation().width(),
            32.0,
        ),
        ViewportHudDropdownId::ShadingMode => Rect::new(
            params.viewport_rect.x + 8.0 + params.camera_orientation().width() + 1.0,
            params.viewport_rect.y + 6.0,
            params.shading_mode().width(),
            32.0,
        ),
        ViewportHudDropdownId::GridSnap => Rect::new(
            params.viewport_rect.x
                + 8.0
                + params.view_modes_group_width()
                + 8.0
                + 140.0
                + 8.0
                + 32.0
                + 8.0
                + 34.0,
            params.viewport_rect.y + 6.0,
            50.0,
            32.0,
        ),
    }
}

/// Renders whichever Viewport HUD dropdown tag is currently active in [`iris_core::UiTree`].
pub fn render_active_viewport_hud_dropdown_popup(
    scope: &mut UiScope<'_>,
    params: &ViewportHudParams<'_>,
    hud: &mut ViewportHudState,
) {
    [
        ViewportHudDropdownId::CameraMode,
        ViewportHudDropdownId::ShadingMode,
        ViewportHudDropdownId::GridSnap,
    ]
    .into_iter()
    .for_each(|dd| render_viewport_hud_dropdown_popup(scope, dd, params, hud));
}

/// Renders an active floating popup menu in the Viewport HUD using scoped popup lifecycle.
pub fn render_viewport_hud_dropdown_popup(
    scope: &mut UiScope<'_>,
    active_dd: ViewportHudDropdownId,
    params: &ViewportHudParams<'_>,
    hud: &mut ViewportHudState,
) {
    scope.scoped_popup_anchored_to_tag(
        "ViewportHudDropdown",
        active_dd.tag(),
        dropdown_fallback_rect(active_dd, params),
        dropdown_width(active_dd),
        PopupPlacement::Below,
        |menu| {
            dropdown_items(active_dd)
                .iter()
                .enumerate()
                .for_each(|(idx, &(icon, label))| {
                    menu.dropdown_item_autoclose(
                        dropdown_tag_base(active_dd) + idx as u64,
                        icon,
                        label,
                        (dropdown_selected_index(active_dd, params) == Some(idx)).then_some("✓"),
                        true,
                    )
                    .on_click(|| {
                        hud.dispatch(ViewportHudAction::SelectDropdownItem(active_dd, idx));
                    });
                });
        },
    );
}

/// Resolves a dispatched [`ViewportHudAction`] from a selected dropdown option index.
#[must_use]
pub fn resolve_viewport_hud_dropdown_action(
    dropdown_id: ViewportHudDropdownId,
    index: usize,
    camera: &Camera,
) -> Option<ViewportHudAction> {
    match dropdown_id {
        ViewportHudDropdownId::CameraMode => match index {
            0 => Some(ViewportHudAction::SetCameraTransform {
                pitch: cgmath::Rad(-0.4),
                yaw: cgmath::Rad(0.6),
                position: cgmath::Point3::new(
                    camera.target.x + 7.0,
                    camera.target.y + 5.0,
                    camera.target.z + 7.0,
                ),
                mode: Some(ProjectionMode::Perspective),
            }),
            1 => Some(ViewportHudAction::SetCameraMode(
                ProjectionMode::Orthographic,
            )),
            2 => Some(ViewportHudAction::SetCameraTransform {
                pitch: cgmath::Rad(-std::f32::consts::FRAC_PI_2 + 0.001),
                yaw: cgmath::Rad(0.0),
                position: cgmath::Point3::new(
                    camera.target.x,
                    camera.target.y + 10.0,
                    camera.target.z,
                ),
                mode: Some(ProjectionMode::Orthographic),
            }),
            3 => Some(ViewportHudAction::SetCameraTransform {
                pitch: cgmath::Rad(0.0),
                yaw: cgmath::Rad(std::f32::consts::FRAC_PI_2),
                position: cgmath::Point3::new(
                    camera.target.x,
                    camera.target.y,
                    camera.target.z - 10.0,
                ),
                mode: Some(ProjectionMode::Orthographic),
            }),
            4 => Some(ViewportHudAction::SetCameraTransform {
                pitch: cgmath::Rad(0.0),
                yaw: cgmath::Rad(0.0),
                position: cgmath::Point3::new(
                    camera.target.x + 10.0,
                    camera.target.y,
                    camera.target.z,
                ),
                mode: Some(ProjectionMode::Orthographic),
            }),
            _ => None,
        },
        ViewportHudDropdownId::ShadingMode => match index {
            0 => Some(ViewportHudAction::SetWireframe(false)),
            1 => Some(ViewportHudAction::SetWireframe(true)),
            _ => None,
        },
        ViewportHudDropdownId::GridSnap => match index {
            0 => Some(ViewportHudAction::SetGridSnapSize(0.05)),
            1 => Some(ViewportHudAction::SetGridSnapSize(0.1)),
            2 => Some(ViewportHudAction::SetGridSnapSize(0.25)),
            3 => Some(ViewportHudAction::SetGridSnapSize(0.5)),
            4 => Some(ViewportHudAction::SetGridSnapSize(1.0)),
            5 => Some(ViewportHudAction::SetGridSnapSize(5.0)),
            6 => Some(ViewportHudAction::SetGridSnapSize(10.0)),
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
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        tree.open_popup(ViewportHudDropdownId::CameraMode.tag());
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
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        tree.open_popup(ViewportHudDropdownId::CameraMode.tag());
        let mut hud = ViewportHudState::default();
        render_viewport_hud_dropdown_popup(
            &mut UiScope::new(&mut tree, root),
            ViewportHudDropdownId::CameraMode,
            &params,
            &mut hud,
        );

        // Verify popup is anchored beneath the toolbar button inside docked viewport coordinates
        let popup_node = tree
            .iter()
            .find(|(_, n)| n.role == WidgetRole::DropdownPopup)
            .map(|(_, n)| n)
            .expect("DropdownPopup node must be in the UI tree");

        assert_eq!(
            popup_node.computed_rect.x,
            viewport_rect.x + 8.0,
            "Popup must be anchored beneath docked toolbar button at viewport_rect.x + 8.0"
        );
        assert_eq!(
            popup_node.computed_rect.y,
            viewport_rect.y + 40.0,
            "Popup must be anchored below docked toolbar at viewport_rect.y + 40.0"
        );

        // Verify that hit testing finds the Perspective item (tag TAG_POPUP_CAMERA_BASE)
        let hit_item0 = tree
            .hit_test_target(Point::new(
                viewport_rect.x + 20.0,
                popup_node.computed_rect.y + 12.0,
            ))
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

    /// Verifies that resolving dropdown actions for GridSnap correctly translates item indices
    /// into explicit grid snap sizes.
    #[test]
    fn test_resolve_grid_snap_dropdown_actions() {
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

        assert_eq!(
            resolve_viewport_hud_dropdown_action(ViewportHudDropdownId::GridSnap, 0, &camera),
            Some(ViewportHudAction::SetGridSnapSize(0.05))
        );
        assert_eq!(
            resolve_viewport_hud_dropdown_action(ViewportHudDropdownId::GridSnap, 4, &camera),
            Some(ViewportHudAction::SetGridSnapSize(1.0))
        );
        assert_eq!(
            resolve_viewport_hud_dropdown_action(ViewportHudDropdownId::GridSnap, 6, &camera),
            Some(ViewportHudAction::SetGridSnapSize(10.0))
        );
        assert_eq!(
            resolve_viewport_hud_dropdown_action(ViewportHudDropdownId::GridSnap, 7, &camera),
            None
        );
    }

    /// Regression test verifying docked Viewport HUD dropdown popups anchor beneath their
    /// toolbar button inside `params.viewport_rect` rather than top-left of the OS window.
    #[test]
    fn test_docked_viewport_hud_dropdown_anchors_inside_viewport_rect() {
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
        let vp_rect = Rect::new(260.0, 56.0, 800.0, 600.0);

        let params = ViewportHudParams {
            viewport_rect: vp_rect,
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Select,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(0.0, 0.0),
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        // Frame 1: build HUD inside a docked panel bounds so trigger button tag_rect is cached
        let mut hud = ViewportHudState::default();
        UiScope::new(&mut tree, root)
            .panel("Viewport")
            .bounds(vp_rect)
            .show(|panel_scope| {
                crate::ui::iris_bridge::viewport_hud::build_viewport_hud_scope(
                    panel_scope,
                    &params,
                    &mut hud,
                );
            });

        // Frame 2: open GridSnap dropdown and rebuild panel prior to layout
        tree.open_popup(ViewportHudDropdownId::GridSnap.tag());
        let _ = tree.clear_children(root);
        UiScope::new(&mut tree, root)
            .panel("Viewport")
            .bounds(vp_rect)
            .show(|panel_scope| {
                crate::ui::iris_bridge::viewport_hud::build_viewport_hud_scope(
                    panel_scope,
                    &params,
                    &mut hud,
                );
            });

        let snap_btn_rect = tree
            .resolve_anchor_rect(crate::ui::iris_bridge::viewport_hud::TAG_SNAP_GRID_DROPDOWN)
            .expect("Snap dropdown trigger rect must resolve");
        let popup_node = tree
            .iter()
            .map(|(_, n)| n)
            .find(|n| n.name.as_deref() == Some("ViewportHudDropdown"))
            .expect("ViewportHudDropdown popup node must exist");

        assert!(
            popup_node.computed_rect.x >= vp_rect.x
                && (popup_node.computed_rect.x - snap_btn_rect.x).abs() < 1.0,
            "Dropdown x ({}) must align with docked trigger button x ({}) inside viewport_rect.x ({})",
            popup_node.computed_rect.x,
            snap_btn_rect.x,
            vp_rect.x
        );
        assert!(
            (popup_node.computed_rect.y - (snap_btn_rect.bottom() + 2.0)).abs() < 1.0,
            "Dropdown y ({}) must open right below docked trigger bottom ({})",
            popup_node.computed_rect.y,
            snap_btn_rect.bottom()
        );
    }
}