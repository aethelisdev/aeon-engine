// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Application Status Bar Subsystem
//!
//! Root connection and export module for the editor bottom status and diagnostics bar.
//!

pub mod builder;
pub mod types;

pub use builder::build_bottom_status_bar;
pub use types::{STATUS_BAR_HEIGHT, StatusBarParams};