// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Dropdown Option Tables & Direct Action Mapping
//!
//! Defines option entries and pure resolvers mapping Camera Mode, Shading Mode, and Grid Snap
//! selections directly to strongly-typed [`EngineUiAction`] commands without fallback rect hacks.

use super::types::{
    CameraViewOrientation, TAG_POPUP_CAMERA_GROUP, TAG_POPUP_SHADING_GROUP,
    TAG_POPUP_SNAP_GRID_GROUP,
};
use crate::ui::types::EngineUiAction;
use ae_camera::{Camera, ProjectionMode};
use ae_editor::snapping::SnapSettings;
use irisui::prelude::AnchoredSelectOption;

/// Option rows displayed inside the Viewport Camera Mode dropdown.
pub const CAMERA_MODE_OPTIONS: &[AnchoredSelectOption<'_>] = &[
    AnchoredSelectOption::new(TAG_POPUP_CAMERA_GROUP.indexed(0).raw(), "🎥", "Perspective"),
    AnchoredSelectOption::new(
        TAG_POPUP_CAMERA_GROUP.indexed(1).raw(),
        "📐",
        "Orthographic",
    ),
    AnchoredSelectOption::new(TAG_POPUP_CAMERA_GROUP.indexed(2).raw(), "📐", "Top"),
    AnchoredSelectOption::new(TAG_POPUP_CAMERA_GROUP.indexed(3).raw(), "📐", "Front"),
    AnchoredSelectOption::new(TAG_POPUP_CAMERA_GROUP.indexed(4).raw(), "📐", "Right"),
];

/// Option rows displayed inside the Viewport Shading Mode dropdown.
pub const SHADING_MODE_OPTIONS: &[AnchoredSelectOption<'_>] = &[
    AnchoredSelectOption::new(TAG_POPUP_SHADING_GROUP.indexed(0).raw(), "💡", "Lit"),
    AnchoredSelectOption::new(TAG_POPUP_SHADING_GROUP.indexed(1).raw(), "🕸", "Wireframe"),
];

/// Option rows displayed inside the Viewport Grid Snap dropdown.
pub const GRID_SNAP_OPTIONS: &[AnchoredSelectOption<'_>] = &[
    AnchoredSelectOption::new(TAG_POPUP_SNAP_GRID_GROUP.indexed(0).raw(), "📏", "0.05m"),
    AnchoredSelectOption::new(TAG_POPUP_SNAP_GRID_GROUP.indexed(1).raw(), "📏", "0.1m"),
    AnchoredSelectOption::new(TAG_POPUP_SNAP_GRID_GROUP.indexed(2).raw(), "📏", "0.25m"),
    AnchoredSelectOption::new(TAG_POPUP_SNAP_GRID_GROUP.indexed(3).raw(), "📏", "0.5m"),
    AnchoredSelectOption::new(TAG_POPUP_SNAP_GRID_GROUP.indexed(4).raw(), "📏", "1.0m"),
    AnchoredSelectOption::new(TAG_POPUP_SNAP_GRID_GROUP.indexed(5).raw(), "📏", "5.0m"),
    AnchoredSelectOption::new(TAG_POPUP_SNAP_GRID_GROUP.indexed(6).raw(), "📏", "10.0m"),
];

const GRID_SNAP_SIZES: [f32; 7] = [0.05, 0.1, 0.25, 0.5, 1.0, 5.0, 10.0];

/// Returns the active checkmark index for the Camera Mode dropdown.
#[inline]
#[must_use]
pub fn camera_mode_selected_index(camera: &Camera, is_2d: bool) -> Option<usize> {
    match CameraViewOrientation::from_camera(camera, is_2d) {
        CameraViewOrientation::Perspective => Some(0),
        CameraViewOrientation::Orthographic => Some(1),
        CameraViewOrientation::Top => Some(2),
        CameraViewOrientation::Front => Some(3),
        CameraViewOrientation::Right => Some(4),
        CameraViewOrientation::TwoDimensional => None,
    }
}

/// Returns the active checkmark index for the Shading Mode dropdown.
#[inline]
#[must_use]
pub fn shading_mode_selected_index(wireframe_enabled: bool) -> Option<usize> {
    Some(usize::from(wireframe_enabled))
}

/// Returns the active checkmark index for the Grid Snap dropdown.
#[inline]
#[must_use]
pub fn grid_snap_selected_index(snapping: &SnapSettings) -> Option<usize> {
    GRID_SNAP_SIZES
        .iter()
        .position(|&s| (snapping.grid_size - s).abs() < 0.001)
}

/// Translates a Camera Mode dropdown selection into a typed [`EngineUiAction`].
#[must_use]
pub fn resolve_camera_mode_action(index: usize, camera: &Camera) -> Option<EngineUiAction> {
    match index {
        0 => Some(EngineUiAction::SetCameraTransform {
            pitch: cgmath::Rad(-0.4),
            yaw: cgmath::Rad(0.6),
            position: cgmath::Point3::new(
                camera.target.x + 7.0,
                camera.target.y + 5.0,
                camera.target.z + 7.0,
            ),
            mode: Some(ProjectionMode::Perspective),
        }),
        1 => Some(EngineUiAction::SetCameraMode(ProjectionMode::Orthographic)),
        2 => Some(EngineUiAction::SetCameraTransform {
            pitch: cgmath::Rad(-std::f32::consts::FRAC_PI_2 + 0.001),
            yaw: cgmath::Rad(0.0),
            position: cgmath::Point3::new(camera.target.x, camera.target.y + 10.0, camera.target.z),
            mode: Some(ProjectionMode::Orthographic),
        }),
        3 => Some(EngineUiAction::SetCameraTransform {
            pitch: cgmath::Rad(0.0),
            yaw: cgmath::Rad(std::f32::consts::FRAC_PI_2),
            position: cgmath::Point3::new(camera.target.x, camera.target.y, camera.target.z - 10.0),
            mode: Some(ProjectionMode::Orthographic),
        }),
        4 => Some(EngineUiAction::SetCameraTransform {
            pitch: cgmath::Rad(0.0),
            yaw: cgmath::Rad(0.0),
            position: cgmath::Point3::new(camera.target.x + 10.0, camera.target.y, camera.target.z),
            mode: Some(ProjectionMode::Orthographic),
        }),
        _ => None,
    }
}

/// Translates a Shading Mode dropdown selection into [`EngineUiAction::SetWireframe`].
#[inline]
#[must_use]
pub const fn resolve_shading_mode_action(index: usize) -> Option<EngineUiAction> {
    match index {
        0 => Some(EngineUiAction::SetWireframe(false)),
        1 => Some(EngineUiAction::SetWireframe(true)),
        _ => None,
    }
}

/// Translates a Grid Snap dropdown selection into [`EngineUiAction::UpdateSnapSettings`].
#[inline]
#[must_use]
pub fn resolve_grid_snap_action(index: usize, snapping: &SnapSettings) -> Option<EngineUiAction> {
    GRID_SNAP_SIZES.get(index).map(|&size| {
        EngineUiAction::UpdateSnapSettings(SnapSettings {
            grid_size: size,
            ..*snapping
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::viewport_hud::panel::build_viewport_hud_scope;
    use crate::ui::iris_bridge::viewport_hud::types::{
        TAG_SNAP_GRID_DROPDOWN, TAG_VIEWPORT_CAMERA_MODE, with_test_frame_data,
    };
    use ae_camera::{Camera, ProjectionMode};
    use ae_ecs::World;
    use ae_editor::editor_state::EditorState;
    use ae_editor::snapping::SnapSettings;
    use ae_gizmos::GizmoMode;
    use irisui::prelude::*;
    use irisui::workbench::ActionBus;

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
    fn test_camera_mode_dropdown_options_and_orthographic_mode() {
        let camera = make_test_camera();

        assert!(matches!(
            resolve_camera_mode_action(1, &camera),
            Some(EngineUiAction::SetCameraMode(ProjectionMode::Orthographic))
        ));

        match resolve_camera_mode_action(0, &camera) {
            Some(EngineUiAction::SetCameraTransform {
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

        for idx in [2, 3, 4] {
            match resolve_camera_mode_action(idx, &camera) {
                Some(EngineUiAction::SetCameraTransform { mode, .. }) => {
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
    fn test_resolve_grid_snap_dropdown_actions() {
        let snapping = SnapSettings::default();

        match resolve_grid_snap_action(0, &snapping) {
            Some(EngineUiAction::UpdateSnapSettings(s)) => {
                assert!((s.grid_size - 0.05).abs() < 1e-4)
            }
            other => panic!("Expected UpdateSnapSettings(0.05), got {:?}", other),
        }
        match resolve_grid_snap_action(4, &snapping) {
            Some(EngineUiAction::UpdateSnapSettings(s)) => {
                assert!((s.grid_size - 1.0).abs() < 1e-4)
            }
            other => panic!("Expected UpdateSnapSettings(1.0), got {:?}", other),
        }
        match resolve_grid_snap_action(6, &snapping) {
            Some(EngineUiAction::UpdateSnapSettings(s)) => {
                assert!((s.grid_size - 10.0).abs() < 1e-4)
            }
            other => panic!("Expected UpdateSnapSettings(10.0), got {:?}", other),
        }
        assert!(resolve_grid_snap_action(7, &snapping).is_none());
    }

    #[test]
    fn test_docked_viewport_hud_dropdown_anchors_inside_viewport_rect_without_fallback() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");

        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let world = World::new();
        let vp_rect = Rect::new(260.0, 56.0, 800.0, 600.0);

        let editor_state = EditorState {
            gizmo_mode: GizmoMode::Select,
            ..Default::default()
        };

        let mut bus = ActionBus::new();

        // Frame 1: build HUD inside docked panel bounds so trigger rects are cached by layout
        with_test_frame_data(
            (vp_rect, true, false),
            &camera,
            &editor_state,
            &snapping,
            &world,
            |frame| {
                UiScope::new(&mut tree, root)
                    .panel("Viewport")
                    .bounds(vp_rect)
                    .show(|panel_scope| {
                        build_viewport_hud_scope(panel_scope, frame, Point::ZERO, &mut bus);
                    });
            },
        );

        let cam_btn_rect = tree
            .resolve_anchor_rect(TAG_VIEWPORT_CAMERA_MODE)
            .expect("Camera dropdown trigger rect must resolve");
        assert!(cam_btn_rect.x >= vp_rect.x);

        // Frame 2: open GridSnap dropdown cleanly via UiTree popup state
        tree.open_popup(TAG_SNAP_GRID_DROPDOWN);
        let _ = tree.clear_children(root);
        with_test_frame_data(
            (vp_rect, true, false),
            &camera,
            &editor_state,
            &snapping,
            &world,
            |frame| {
                UiScope::new(&mut tree, root)
                    .panel("Viewport")
                    .bounds(vp_rect)
                    .show(|panel_scope| {
                        build_viewport_hud_scope(panel_scope, frame, Point::ZERO, &mut bus);
                    });
            },
        );

        let snap_btn_rect = tree
            .resolve_anchor_rect(TAG_SNAP_GRID_DROPDOWN)
            .expect("Snap dropdown trigger rect must resolve");
        let popup_node = tree
            .iter()
            .map(|(_, n)| n)
            .find(|n| n.name.as_deref() == Some("GridSnapDropdown"))
            .expect("GridSnapDropdown popup node must exist");

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
        assert!(
            irisui::text::collect_text_sections(&tree)
                .iter()
                .any(|s| s.text.as_ref() == "1.0m"),
            "Dropdown option labels must be collected cleanly"
        );
    }
}