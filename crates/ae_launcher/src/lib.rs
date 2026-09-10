// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! `ae_launcher` - Lightweight Project Hub and Profile Launcher for Aeon Engine.
//!
//! Provides an ultra-fast, standalone project manager built on Iris UI for launching Aeon Engine
//! with 2D or 3D dimension profiles. When an engine instance is spawned,
//! the launcher process terminates, releasing all allocated system resources.
//!

pub mod app;
pub mod icon;
pub mod icons;
pub mod project;
pub mod spawner;
pub mod ui;

#[cfg(test)]
mod tests;

pub use app::LauncherApp;
pub use icon::load_icon_from_memory;
pub use icons::{ICON_CUBE, ICON_FOLDER, ICON_PLUS, ICON_WIREFRAME, ICON_WORLD};
pub use project::{ProjectConfig, ProjectRegistry};
pub use spawner::{find_engine_binary, launch_engine_and_exit};
pub use ui::{LauncherAction, LauncherTab, LauncherUiState, build_launcher_ui};