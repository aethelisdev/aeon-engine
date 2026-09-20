// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Settings and Preferences Modal System
//!
//! Provides comprehensive builders and styling structures for tabbed dialogs,
//! collapsible setting sections, and standardized setting property rows.

pub mod dialog;
pub mod row;
pub mod section;
pub mod types;

pub use dialog::TabbedDialogBuilder;
pub use row::SettingRowBuilder;
pub use section::SettingSectionBuilder;
pub use types::{
    SettingRowFrame, SettingRowStyle, SettingSectionFrame, SettingSectionStyle, TabbedDialogFrame,
    TabbedDialogStyle, TabbedDialogTab,
};