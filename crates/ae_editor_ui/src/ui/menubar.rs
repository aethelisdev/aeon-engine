// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Application Menu Bar Subsystem
//!
//! Root connection and export module for the editor top horizontal menu bar and
//! floating popup dropdown overlays.
//!

pub mod builder;
pub mod events;
pub mod types;

// Re-exports for clean modular access
pub use builder::{
    DropdownBuildParams, TopMenuBarOutput, build_floating_dropdown, build_top_menu_bar,
};
pub use events::{MenuBarEventContext, handle_menubar_event};
pub use types::{
    ActiveMenu, DROPDOWN_WIDTH, DropdownAction, MENUBAR_HEIGHT, MenuBarState,
    TAG_ACTION_PLAY_PAUSE, TAG_MENU_EDIT, TAG_MENU_FILE, TAG_MENU_HELP, TAG_MENU_VIEW,
    TAG_MENU_WINDOW,
};