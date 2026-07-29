// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Gizmo core types and state — the coordinator struct lives here.
use cgmath::{Quaternion, Vector3};

/// The functional mode of the gizmo.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
}

/// Active or hovered axis state.
/// - `None`: No axis is active or hovered.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveAxis {
    None,
    X,
    Y,
    Z,
    Free,
    PlaneXY,
    PlaneXZ,
    PlaneYZ,
}

/// Parameters for keeping gizmo constant-size on screen across perspective/ortho modes.
#[derive(Clone, Copy, Debug)]
pub struct GizmoScreenParams {
    pub viewport_height_px: f32,
    pub camera_fovy_deg: f32,
    pub axis_length_px: f32,
    pub pick_radius_px: f32,
    pub camera_mode: ae_core::camera::ProjectionMode,
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

/// The main Gizmo system — holds state and GPU resources.
/// Acts as a **coordinator**: delegates math to `math.rs`, intersection to `picking.rs`,
/// drag logic to `translate.rs`/`rotate.rs`, and rendering to `render.rs`.
pub struct GizmoSystem {
    pub mode: GizmoMode,
    /// Coordinate space for gizmo axes — World or Local.
    pub space: super::space::GizmoSpace,
    pub active_axis: ActiveAxis,
    pub hovered_axis: ActiveAxis,
    pub is_dragging: bool,

    // Drag state
    pub drag_start_world: Vector3<f32>,
    pub drag_start_vector: Vector3<f32>,
    pub drag_current_vector: Vector3<f32>,
    pub(crate) drag_gizmo_pos: Vector3<f32>,
    pub(crate) drag_plane_normal: Vector3<f32>,
    /// The current world position of the mouse during a drag or scale operation.
    /// Used for drawing guidelines and rubber-band effect lines.
    pub drag_current_hit: Vector3<f32>,
    pub(crate) drag_scale: f32,
    pub(crate) drag_scale_factor: f32,
    pub(crate) cam_right: std::cell::Cell<Vector3<f32>>,
    pub(crate) cam_up: std::cell::Cell<Vector3<f32>>,
    /// The selected entity's rotation quaternion, used to orient gizmo axes in Local space.
    /// Updated by the engine each frame before gizmo rendering and input processing.
    pub entity_rotation: Quaternion<f32>,

    // GPU Resources
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) rotate_vertex_buffer: wgpu::Buffer,
    pub(crate) scale_vertex_buffer: wgpu::Buffer,
    pub(crate) interaction_vertex_buffer: wgpu::Buffer,
    pub(crate) ring_vertex_buffer: wgpu::Buffer,
    pub(crate) uniform_buffer: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) mesh_pipeline: wgpu::RenderPipeline,
    pub(crate) line_pipeline: wgpu::RenderPipeline,
    pub(crate) num_vertices: u32,
    pub(crate) num_rotate_vertices: u32,
    pub(crate) num_scale_vertices: u32,
}

impl GizmoSystem {
    /// Returns `true` if the gizmo is currently being dragged.
    pub fn dragging_active(&self) -> bool {
        self.is_dragging
    }

    /// Returns the currently active drag axis (or `ActiveAxis::None`).
    pub fn active_axis(&self) -> ActiveAxis {
        self.active_axis
    }

    /// Checks if the given axis handle is allowed in the current gizmo mode.
    pub fn is_handle_allowed(&self, axis: ActiveAxis) -> bool {
        match self.mode {
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
            GizmoMode::Rotate => matches!(axis, ActiveAxis::X | ActiveAxis::Y | ActiveAxis::Z),
            GizmoMode::Scale => matches!(
                axis,
                ActiveAxis::X | ActiveAxis::Y | ActiveAxis::Z | ActiveAxis::Free
            ),
        }
    }
}