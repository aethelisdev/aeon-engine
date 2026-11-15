// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Menu Bar & Floating Dropdown Primitives
//!
//! Pure module root coordinating top application menu bar controls, floating dropdown
//! popup cards, selectable menu item rows, cascading submenus, and anchored popups.

pub mod anchored;
pub mod bar_controls;
pub mod dropdown;
pub mod submenu;
pub mod types;

pub use types::{CascadingPopupConfig, DropdownIcon, PopupExt, PopupPlacement};