// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Workbench Subsystem
//!
//! Orchestrates the editor lifecycle, window events, persistent UI state, and WGPU rendering.

pub mod events;
pub mod render;
pub mod state;

pub use render::EditorUiRenderParams;
pub use state::{EngineUi, SceneDialogAction};