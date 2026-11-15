// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Action Hook Callback Type Aliases
//!
//! Encapsulates type aliases for decoupled action mapping closures triggered by
//! window-level interactions, file drops, zoom keybindings, and preferences shortcuts.

use iris_core::geometry::Point;
use std::path::PathBuf;

/// Type alias for external file drag-and-drop action handler closure.
pub type FileDropActionBuilder<Action> = Box<dyn Fn(Vec<PathBuf>, Point) -> Action + Send + Sync>;

/// Type alias for UI zoom alteration action handler closure.
pub type ZoomActionBuilder<Action> = Box<dyn Fn(f32) -> Action + Send + Sync>;

/// Type alias for preferences modal toggle action handler closure.
pub type PreferencesActionBuilder<Action> = Box<dyn Fn() -> Action + Send + Sync>;