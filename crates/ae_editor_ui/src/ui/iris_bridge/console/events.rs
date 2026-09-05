// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Console Interactive Event Handlers
//!
//! Processes mouse clicks, toolbar actions, filter switches, and mouse wheel
//! scrolling within the Developer Console panel.
//!

use super::types::{ConsoleAction, ConsoleFilterLevel, ConsolePanelTargets};
use irisui::prelude::Point;

/// Handles mouse click events over registered console panel interactive targets.
pub fn handle_console_click(
    targets: &ConsolePanelTargets,
    click_pos: Point,
) -> Option<ConsoleAction> {
    if !targets.panel_rect.contains_point(click_pos) {
        return None;
    }

    if targets.clear_btn_rect.contains_point(click_pos) {
        return Some(ConsoleAction::ClearLogs);
    }

    if targets.filter_all_rect.contains_point(click_pos) {
        return Some(ConsoleAction::SetFilter(ConsoleFilterLevel::All));
    }

    if targets.filter_error_rect.contains_point(click_pos) {
        return Some(ConsoleAction::SetFilter(ConsoleFilterLevel::Error));
    }

    if targets.filter_warn_rect.contains_point(click_pos) {
        return Some(ConsoleAction::SetFilter(ConsoleFilterLevel::Warn));
    }

    if targets.filter_info_rect.contains_point(click_pos) {
        return Some(ConsoleAction::SetFilter(ConsoleFilterLevel::Info));
    }

    if targets.filter_debug_rect.contains_point(click_pos) {
        return Some(ConsoleAction::SetFilter(ConsoleFilterLevel::Debug));
    }

    if targets.autoscroll_toggle_rect.contains_point(click_pos) {
        return Some(ConsoleAction::ToggleAutoScroll);
    }

    if targets
        .search_clear_btn_rect
        .is_some_and(|r| r.contains_point(click_pos))
    {
        return Some(ConsoleAction::ClearSearch);
    }

    if targets.search_input_rect.contains_point(click_pos) {
        return Some(ConsoleAction::FocusSearch);
    }

    None
}

/// Handles mouse wheel scrolling over the console viewport.
/// When the user manually scrolls up, automatic scroll-to-bottom is paused.
/// If scrolled all the way to the bottom, auto-scroll is resumed.
pub fn handle_console_scroll(
    targets: &ConsolePanelTargets,
    cursor_pos: Point,
    scroll_delta_y: f32,
    current_scroll_y: &mut f32,
    auto_scroll: &mut bool,
) -> bool {
    if !targets.panel_rect.contains_point(cursor_pos) {
        return false;
    }

    // scroll_delta_y is positive when scrolling up, negative when scrolling down
    let scroll_step = scroll_delta_y * 24.0;
    let new_scroll = (*current_scroll_y - scroll_step).clamp(0.0, targets.max_scroll_y);

    *current_scroll_y = new_scroll;

    if new_scroll < targets.max_scroll_y - 12.0 {
        // User scrolled up away from bottom
        *auto_scroll = false;
    } else {
        // User scrolled to or near bottom
        *auto_scroll = true;
    }

    true
}