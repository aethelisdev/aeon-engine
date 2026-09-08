// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Preferences Module
//!
//! Provides modular card renderers for editor settings including snapping, history limits,
//! physics simulation rate, and live runtime hot reload.

pub mod history;
pub mod panel;
pub mod physics;
pub mod runtime;
pub mod snapping;
pub mod types;

pub use panel::build_editor_tab;
pub use types::SNAP_MODE_OPTIONS;