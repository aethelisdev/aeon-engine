// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Workbench Rendering Subsystem
//!
//! Coordinates frame rendering across the native Iris UI GPU SDF dock host,
//! overlays, and action dispatchers for settings, hierarchy, and inspector mutations.
//!

pub mod draw;
pub mod inspector_actions;
pub mod iris_pass;
pub mod overlay_actions;
pub mod types;

pub use types::EditorUiRenderParams;