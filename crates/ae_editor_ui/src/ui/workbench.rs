// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Workbench & Framework-Gap Quarantine
//!
//! Pure module root consolidating workbench orchestration, overlay action dispatch,
//! and transitional framework-gap scaffolding inside [`crate::ui::workbench`]
//! until eliminated into `irisui`.

pub mod bridge_params;
pub mod bridge_types;
pub mod icons;
pub mod panels;
pub mod render;
pub mod state;

pub use panels::{EditorPanelContext, EditorPanelScope, create_editor_panel_registry};
pub use render::EditorUiRenderParams;
pub use state::{EngineUi, SceneDialogAction};