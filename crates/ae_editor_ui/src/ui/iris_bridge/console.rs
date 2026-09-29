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
pub use rows::{
    CONSOLE_ROW_HEIGHT, CONSOLE_ROW_LINE_HEIGHT, build_console_rows, render_console_empty_notice,
    render_console_row,
};
pub use types::{
    CONSOLE_TAG_AUTOSCROLL, CONSOLE_TAG_CLEAR, CONSOLE_TAG_FILTER_ALL, CONSOLE_TAG_FILTER_DEBUG,
    CONSOLE_TAG_FILTER_ERROR, CONSOLE_TAG_FILTER_INFO, CONSOLE_TAG_FILTER_WARN,
    CONSOLE_TAG_PANEL_ROOT, CONSOLE_TAG_ROW, CONSOLE_TAG_SCROLLBAR_THUMB,
    CONSOLE_TAG_SCROLLBAR_TRACK, CONSOLE_TAG_SEARCH_CLEAR, CONSOLE_TAG_SEARCH_INPUT,
    CONSOLE_TAG_TOOLBAR, CONSOLE_TAG_VIEWPORT, ConsoleAction, ConsoleFilterLevel, ConsoleLogCounts,
    ConsoleLogLevel, ConsolePanelParams, ConsolePanelState, ConsoleToolbarAction,
    ConsoleToolbarCursor, evaluate_console_toolbar_click, evaluate_console_toolbar_cursor,
    is_console_tag,
};