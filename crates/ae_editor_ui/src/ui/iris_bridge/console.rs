// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console & Logger Iris UI Module
//!
//! Provides a hardware-accelerated, high-performance GPU SDF Developer Console
//! and logging telemetry viewer for the Aeon Engine Editor.
//!

pub mod events;
pub mod panel;
pub mod rows;
#[cfg(test)]
mod tests;
pub mod types;

pub use events::{handle_console_click, handle_console_scroll};
pub use panel::{CONSOLE_TOOLBAR_HEIGHT, build_console_panel};
pub use rows::{CONSOLE_ROW_HEIGHT, build_console_rows};
pub use types::{ConsoleAction, ConsoleFilterLevel, ConsolePanelParams, ConsolePanelTargets};