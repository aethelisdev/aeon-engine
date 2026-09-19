// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Subsystem Widgets (`iris-widgets::console`)
//!
//! Provides the complete widget suite for the hardware-accelerated Developer Console:
//! - Semantic hit-testing tags, severity filters, actions, and cursor resolvers.
//! - Header toolbar builder with Clear, filter tabs, search box with caret, and auto-scroll.
//! - Log entry row builder with zebra striping, level badges, timestamps, and target tags.
//! - Empty state notice placeholder builder.
//!

pub mod rows;
pub mod toolbar;
pub mod types;

pub use rows::{ConsoleEmptyNoticeBuilder, ConsoleRowBuilder, ConsoleRowFrame, ConsoleRowStyle};
pub use toolbar::{ConsoleToolbarBuilder, ConsoleToolbarFrame, ConsoleToolbarStyle};
pub use types::{
    CONSOLE_TAG_AUTOSCROLL, CONSOLE_TAG_CLEAR, CONSOLE_TAG_FILTER_ALL, CONSOLE_TAG_FILTER_DEBUG,
    CONSOLE_TAG_FILTER_ERROR, CONSOLE_TAG_FILTER_INFO, CONSOLE_TAG_FILTER_WARN,
    CONSOLE_TAG_SEARCH_CLEAR, CONSOLE_TAG_SEARCH_INPUT, ConsoleFilterLevel, ConsoleLogCounts,
    ConsoleLogLevel, ConsoleToolbarAction, ConsoleToolbarCursor, evaluate_console_toolbar_click,
    evaluate_console_toolbar_cursor,
};