// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Settings and Preferences Modal System
//!
//! Provides comprehensive builders and styling structures for tabbed dialogs
//! and collapsible setting sections.

pub mod dialog;
pub mod section;
pub mod types;

pub use dialog::TabbedDialogBuilder;
pub use section::SettingSectionBuilder;
pub use types::{
    SettingSectionFrame, SettingSectionStyle, TabbedDialogFrame, TabbedDialogStyle, TabbedDialogTab,
};