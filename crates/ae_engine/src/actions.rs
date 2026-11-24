// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Host-side editor UI action dispatcher and mutation handlers.
//!
//! Consumes [`ae_editor_ui::ui::EngineUiAction`] commands emitted by UI panels and executes
//! mutations directly against [`crate::engine::AeEngine`].

pub mod components;
pub mod dispatcher;
pub mod scene_io;
pub mod spawning;
pub mod system;

pub use dispatcher::process_ui_actions;