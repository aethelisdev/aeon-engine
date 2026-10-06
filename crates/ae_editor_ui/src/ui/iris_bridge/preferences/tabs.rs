// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Tab Renderers
//!
//! Submodules providing modular content builders for all Preferences sections.

pub mod addons;
pub mod editor;
pub mod experimental;
pub mod general;
pub mod graphics;
pub mod input;
pub mod keymap;
pub mod modules_tab;
pub mod navigation;
pub mod system;

pub use addons::build_addons_tab;
pub use editor::{SNAP_MODE_OPTIONS, build_editor_tab};
pub use experimental::build_experimental_tab;
pub use general::build_general_tab;
pub use graphics::build_graphics_tab;
pub use input::build_input_tab;
pub use keymap::build_keymap_tab;
pub use modules_tab::build_modules_tab;
pub use navigation::build_navigation_tab;
pub use system::build_system_tab;