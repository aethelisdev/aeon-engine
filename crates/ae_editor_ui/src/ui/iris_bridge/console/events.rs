// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Console Interactive Event Handlers
//!
//! Processes mouse clicks, toolbar actions, filter switches, and mouse wheel
//! scrolling within the Developer Console panel.
//!

use super::types::ConsoleAction;
use irisui::prelude::{ConsoleToolbarAction, Point, UiTree, evaluate_console_toolbar_click};

/// Handles mouse click events over the console panel using semantic tags.
///
/// Dispatches click actions via the semantic `UiTree::hit_test_target` resolver.
pub fn handle_console_click(tree: &UiTree, click_pos: Point) -> Option<ConsoleAction> {
    if let Some(action) = evaluate_console_toolbar_click(tree, click_pos) {
        return Some(match action {
            ConsoleToolbarAction::ClearLogs => ConsoleAction::ClearLogs,
            ConsoleToolbarAction::SetFilter(level) => ConsoleAction::SetFilter(level),
            ConsoleToolbarAction::ToggleAutoScroll => ConsoleAction::ToggleAutoScroll,
            ConsoleToolbarAction::FocusSearch => ConsoleAction::FocusSearch,
            ConsoleToolbarAction::ClearSearch => ConsoleAction::ClearSearch,
        });
    }

    None
}

/// Handles mouse wheel scrolling over the console viewport.
///
/// When the user manually scrolls up, automatic scroll-to-bottom is paused.
/// If scrolled all the way to the bottom, auto-scroll is resumed.
pub fn handle_console_scroll(
    scroll_delta_y: f32,
    max_scroll_y: f32,
    current_scroll_y: &mut f32,
    auto_scroll: &mut bool,
) -> bool {
    // scroll_delta_y is positive when scrolling up, negative when scrolling down
    let scroll_step = scroll_delta_y * 24.0;
    let base_scroll = if *auto_scroll {
        max_scroll_y
    } else {
        *current_scroll_y
    };
    let new_scroll = (base_scroll - scroll_step).clamp(0.0, max_scroll_y);

    *current_scroll_y = new_scroll;

    if new_scroll < max_scroll_y - 12.0 {
        // User scrolled up away from bottom
        *auto_scroll = false;
    } else {
        // User scrolled to or near bottom
        *auto_scroll = true;
    }

    true
}