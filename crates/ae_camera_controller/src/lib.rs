// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Aeon Camera Controller Crate
//!
//! Modular camera control strategies cleanly decoupled from WGPU and editor UI:
//! - [`free_fly`]: Editor 6-DoF WASD/QE navigation, mouse-look, orthographic pan, scroll zoom, and target focus.
//! - [`follow`]: Runtime 3D spring-arm orbit and 2D orthographic planar follow camera.

pub mod follow;
pub mod free_fly;

pub use follow::{FollowCameraConfig, update_follow_camera};
pub use free_fly::{
    EditorCameraConfig, FreeCameraInput, apply_mouse_look, apply_scroll_zoom,
    focus_camera_on_entity, focus_camera_on_point, pan_orthographic_camera, step_free_camera,
};