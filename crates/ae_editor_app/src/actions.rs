// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Host-side editor UI action dispatcher and I/O adapters.
//!
//! Consumes [`ae_editor_ui::ui::EngineUiAction`] commands emitted by UI panels and wires
//! [`ae_scene`], [`ae_editor`], [`ae_assets`], and OS dialogs onto [`crate::session::EditorSession`].

pub mod asset_import;
pub mod dispatcher;
pub mod scene_io;
pub mod spawning;

pub use dispatcher::process_ui_actions;