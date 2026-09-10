// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular UI subsystems and view builders for Aeon Launcher.

pub mod builder;
pub mod new_project;
pub mod recent_projects;
pub mod sidebar;
pub mod types;

pub use builder::build_launcher_ui;
pub use new_project::build_new_project_view;
pub use recent_projects::build_recent_projects_view;
pub use sidebar::build_sidebar;
pub use types::{LauncherAction, LauncherTab, LauncherUiState, ViewLayoutContext};