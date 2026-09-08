// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Viewport HUD Subsystem
//!
//! Provides the root orchestration entry points for building 3D Viewport Toolbar,
//! Orientation Compass, Camera HUD, Billboard Icons, and Play Mode overlays in Iris UI.
//!

pub mod billboards;
pub mod builder;
pub mod camera_hud;
pub mod compass;
pub mod play_hud;
pub mod popup;
pub mod toolbar;
pub mod types;

pub use builder::build_viewport_hud;
pub use types::{ViewportHudAction, ViewportHudDropdownId, ViewportHudParams, ViewportHudTargets};