// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Transform gizmo core types, screen scaling parameters, and CPU interaction state.

use super::space::GizmoSpace;
use cgmath::{Quaternion, Vector3};

/// The functional interaction mode of the 3D viewport gizmo system.
///
/// Supports selection-only mode (`Select`) where gizmo handles are suppressed for an
/// uncluttered viewport, as well as classic transform modes (`Translate`, `Rotate`, `Scale`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GizmoMode {
    /// Pure selection tool (Q): selects entities without drawing transform axis handles.
    #[default]
    Select,
    /// Translation tool (W): draws 3-axis arrows and planar quads for spatial movement.
    Translate,
    /// Rotation tool (E): draws 3-axis rotation rings for angular orientation.
    Rotate,
    /// Scaling tool (R): draws 3-axis cuboids and a central uniform scaling cube.
    Scale,
}

/// Active or hovered axis state.
/// - `None`: No axis is active or hovered.
/// - `Screen`: Camera-facing outer view rotation ring.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveAxis {
    /// No axis is hovered or actively manipulated.
    None,
    /// Primary X axis handle.
    X,
    /// Primary Y axis handle.
    Y,
    /// Primary Z axis handle.
    Z,
    /// Unconstrained center handle (uniform scale or screen-plane drag).
    Free,
    /// Camera-facing outer screen-space rotation ring.
    Screen,
    /// Dual-axis XY planar translation quad.
    PlaneXY,
    /// Dual-axis XZ planar translation quad.
    PlaneXZ,
    /// Dual-axis YZ planar translation quad.
    PlaneYZ,
}

/// Parameters for keeping gizmo constant-size on screen across perspective/ortho modes.
#[derive(Clone, Copy, Debug)]
pub struct GizmoScreenParams {
    /// Viewport logical/physical height in pixels (clamped to `>= 1.0` when projecting).
    pub viewport_height_px: f32,
    /// Perspective vertical field of view in degrees.
    pub camera_fovy_deg: f32,
    /// Target handle length in pixels.
    pub axis_length_px: f32,
    /// Interactive raycast pick tolerance radius in pixels.
    pub pick_radius_px: f32,
    /// Active camera projection mode (Perspective vs Orthographic).
    pub camera_mode: ae_core::camera::ProjectionMode,
    /// Orthographic vertical span scale in world units.
    pub ortho_scale: f32,
}

impl GizmoScreenParams {
    /// Calculates the axis length in world units for the given camera distance.
    pub fn axis_length_world(&self, camera_distance: f32) -> f32 {
        let h = self.viewport_height_px.max(1.0);
        match self.camera_mode {
            ae_core::camera::ProjectionMode::Perspective => {
                let d = camera_distance.max(1e-6);
                let fovy_rad = self.camera_fovy_deg.to_radians();
                let tan_half_fov = (fovy_rad * 0.5).tan();
                self.axis_length_px * 2.0 * d * tan_half_fov / h
            }
            ae_core::camera::ProjectionMode::Orthographic => {
                self.axis_length_px * self.ortho_scale / h
            }
        }
    }

    /// Calculates the pick (raycast) radius in world units for the given camera distance.
    pub fn pick_radius_world(&self, camera_distance: f32) -> f32 {
        let h = self.viewport_height_px.max(1.0);
        match self.camera_mode {
            ae_core::camera::ProjectionMode::Perspective => {
                let d = camera_distance.max(1e-6);
                let fovy_rad = self.camera_fovy_deg.to_radians();
                let tan_half_fov = (fovy_rad * 0.5).tan();
                self.pick_radius_px * 2.0 * d * tan_half_fov / h
            }
            ae_core::camera::ProjectionMode::Orthographic => {
                self.pick_radius_px * self.ortho_scale / h
            }
        }
    }
}

/// CPU-side 3D Transform Gizmo state coordinator.
///
/// Owns selection/hover/drag state and camera basis without any GPU handles.
/// Delegates intersection math to `picking`, drag deltas to `translate`/`rotate`,
/// and handle mesh generation to `geometry`/`rotation_geometry`.
pub struct TransformGizmo {
    /// Current manipulation tool (`Select`, `Translate`, `Rotate`, `Scale`).
    pub mode: GizmoMode,
    /// Coordinate space for gizmo axes (`World` or `Local`).
    pub space: GizmoSpace,
    /// Handle axis currently undergoing drag manipulation.
    pub active_axis: ActiveAxis,
    /// Handle axis currently hovered by the cursor.
    pub hovered_axis: ActiveAxis,
    /// Whether a handle drag transaction is actively in progress.
    pub is_dragging: bool,
    /// World-space initial ray-plane hit point recorded when dragging began.
    pub drag_start_world: Vector3<f32>,
    /// Unit vector from gizmo center to initial rotation hit point.
    pub drag_start_vector: Vector3<f32>,
    /// Unit vector from gizmo center to current rotation hit point.
    pub drag_current_vector: Vector3<f32>,
    pub(crate) drag_gizmo_pos: Vector3<f32>,
    /// Normal vector of the active drag constraint plane.
    pub drag_plane_normal: Vector3<f32>,
    /// Current world-space mouse hit position used for guidelines and rubber-band visuals.
    pub drag_current_hit: Vector3<f32>,
    pub(crate) drag_scale: f32,
    pub(crate) drag_scale_factor: f32,
    /// Orthonormal camera right vector used to orient view-facing gizmo rings.
    pub cam_right: Vector3<f32>,
    /// Orthonormal camera up vector used to orient view-facing gizmo rings.
    pub cam_up: Vector3<f32>,
    /// Normalized camera forward direction vector.
    pub cam_forward: Vector3<f32>,
    /// World-space camera eye position for back-facing arc culling.
    pub cam_pos: Vector3<f32>,
    /// Selected entity rotation quaternion, used to orient axes in `GizmoSpace::Local`.
    pub entity_rotation: Quaternion<f32>,
}

impl Default for TransformGizmo {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformGizmo {
    /// Creates a new CPU-side `TransformGizmo` initialized to `Translate` + `Local` space.
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: GizmoMode::Translate,
            space: GizmoSpace::Local,
            active_axis: ActiveAxis::None,
            hovered_axis: ActiveAxis::None,
            is_dragging: false,
            drag_start_world: Vector3::new(0.0, 0.0, 0.0),
            drag_start_vector: Vector3::new(1.0, 0.0, 0.0),
            drag_current_vector: Vector3::new(1.0, 0.0, 0.0),
            drag_gizmo_pos: Vector3::new(0.0, 0.0, 0.0),
            drag_plane_normal: Vector3::new(0.0, 0.0, 1.0),
            drag_current_hit: Vector3::new(0.0, 0.0, 0.0),
            drag_scale: 1.0,
            drag_scale_factor: 1.0,
            cam_right: Vector3::unit_x(),
            cam_up: Vector3::unit_y(),
            cam_forward: -Vector3::unit_z(),
            cam_pos: Vector3::new(0.0, 0.0, 5.0),
            entity_rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        }
    }

    /// Updates the camera orthonormal basis and eye position used for view-aligned rings.
    pub fn set_camera_basis(
        &mut self,
        right: Vector3<f32>,
        up: Vector3<f32>,
        forward: Vector3<f32>,
        pos: Vector3<f32>,
    ) {
        self.cam_right = right;
        self.cam_up = up;
        self.cam_forward = forward;
        self.cam_pos = pos;
    }

    /// Returns `true` if the gizmo is currently being dragged.
    #[must_use]
    pub fn dragging_active(&self) -> bool {
        self.is_dragging
    }

    /// Returns the currently active drag axis (or `ActiveAxis::None`).
    #[must_use]
    pub fn active_axis(&self) -> ActiveAxis {
        self.active_axis
    }

    /// Checks if the given axis handle is allowed in the current gizmo mode.
    #[must_use]
    pub fn is_handle_allowed(&self, axis: ActiveAxis) -> bool {
        match self.mode {
            GizmoMode::Select => false,
            GizmoMode::Translate => matches!(
                axis,
                ActiveAxis::X
                    | ActiveAxis::Y
                    | ActiveAxis::Z
                    | ActiveAxis::Free
                    | ActiveAxis::PlaneXY
                    | ActiveAxis::PlaneXZ
                    | ActiveAxis::PlaneYZ
            ),
            GizmoMode::Rotate => matches!(
                axis,
                ActiveAxis::X | ActiveAxis::Y | ActiveAxis::Z | ActiveAxis::Screen
            ),
            GizmoMode::Scale => matches!(
                axis,
                ActiveAxis::X | ActiveAxis::Y | ActiveAxis::Z | ActiveAxis::Free
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::camera::ProjectionMode;

    #[test]
    fn test_handle_allowed_matrix_and_select_rejection() {
        let mut gizmo = TransformGizmo::new();
        gizmo.mode = GizmoMode::Select;
        for axis in [
            ActiveAxis::None,
            ActiveAxis::X,
            ActiveAxis::Y,
            ActiveAxis::Z,
            ActiveAxis::Free,
            ActiveAxis::Screen,
            ActiveAxis::PlaneXY,
            ActiveAxis::PlaneXZ,
            ActiveAxis::PlaneYZ,
        ] {
            assert!(
                !gizmo.is_handle_allowed(axis),
                "Select mode must reject all handles"
            );
        }

        gizmo.mode = GizmoMode::Rotate;
        assert!(gizmo.is_handle_allowed(ActiveAxis::Screen));
        assert!(!gizmo.is_handle_allowed(ActiveAxis::PlaneXY));

        gizmo.mode = GizmoMode::Scale;
        assert!(gizmo.is_handle_allowed(ActiveAxis::Free));
        assert!(!gizmo.is_handle_allowed(ActiveAxis::Screen));
    }

    #[test]
    fn test_screen_params_clamps_degenerate_viewport_and_distance() {
        let persp = GizmoScreenParams {
            viewport_height_px: 0.0,
            camera_fovy_deg: 60.0,
            axis_length_px: 100.0,
            pick_radius_px: 10.0,
            camera_mode: ProjectionMode::Perspective,
            ortho_scale: 10.0,
        };
        let len = persp.axis_length_world(-5.0);
        let pick = persp.pick_radius_world(0.0);
        assert!(len.is_finite() && len > 0.0);
        assert!(pick.is_finite() && pick > 0.0);

        let ortho = GizmoScreenParams {
            camera_mode: ProjectionMode::Orthographic,
            viewport_height_px: 500.0,
            ortho_scale: 20.0,
            axis_length_px: 50.0,
            pick_radius_px: 10.0,
            camera_fovy_deg: 45.0,
        };
        assert!((ortho.axis_length_world(100.0) - 2.0).abs() < 1e-5);
        assert!((ortho.pick_radius_world(100.0) - 0.4).abs() < 1e-5);
    }
}