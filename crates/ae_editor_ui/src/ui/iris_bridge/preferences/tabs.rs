// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Tab Renderers
//!
//! Submodules providing modular content builders for all Preferences sections.

pub mod editor;
pub mod general;
pub mod graphics;
pub mod info_tabs;
pub mod modules_tab;

pub use editor::{SNAP_MODE_OPTIONS, build_editor_tab};
pub use general::build_general_tab;
pub use graphics::build_graphics_tab;
pub use info_tabs::build_info_tab;
pub use modules_tab::build_modules_tab;