// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Graphics Preferences Tab Subsystem
//!
//! Orchestrates the rendering of Shadows, Performance, Anti-Aliasing, Post-Processing,
//! Environment & Sky, and Procedural Clouds preference cards.

pub mod environment;
pub mod helpers;
pub mod panel;
pub mod performance;
pub mod popup;
pub mod shadows;
pub mod types;

pub use panel::build_graphics_tab;
pub use popup::render_graphics_dropdown_popup;
pub use types::*;