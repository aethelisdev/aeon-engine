// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Viewport HUD Subsystem
//!
//! Provides the root orchestration entry points for building 3D Viewport Toolbar,
//! Orientation Compass, Camera HUD, and Play Mode overlays in Iris UI.
//!

pub mod builder;
pub mod camera_hud;
pub mod compass;
pub mod play_hud;
pub mod popup;
pub mod toolbar;
pub mod types;

pub use builder::build_viewport_hud;
pub use types::{
    TAG_COMPASS_CANVAS, TAG_COMPASS_NEG_X, TAG_COMPASS_NEG_Y, TAG_COMPASS_NEG_Z, TAG_COMPASS_POS_X,
    TAG_COMPASS_POS_Y, TAG_COMPASS_POS_Z, TAG_GIZMO_ROTATE, TAG_GIZMO_SCALE, TAG_GIZMO_SELECT,
    TAG_GIZMO_SPACE_TOGGLE, TAG_GIZMO_TRANSLATE, TAG_PLAY_EXIT, TAG_PLAY_RESUME,
    TAG_VIEWPORT_CAMERA_MODE, TAG_VIEWPORT_SHADING_MODE, ViewportHudAction, ViewportHudDropdownId,
    ViewportHudParams, ViewportHudState,
};