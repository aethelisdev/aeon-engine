// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use iris_core::event::KeyCode;

use super::{GutterMetrics, GutterStyle, MIN_GUTTER_WIDTH};
use crate::controls::multiline::state::MultilineTextState;

#[test]
fn test_vertical_navigation_preferred_column_memory() {
    // Construct 3 lines with varying lengths:
    // Line 0: "012345678901234" (length 15)
    // Line 1: "0123"            (length 4)
    // Line 2: "01234567890123456789" (length 20)
    let text = "012345678901234\n0123\n01234567890123456789";
    let mut state = MultilineTextState::new(text);

    // Position cursor at Line 0, Col 10
    state.set_cursor(0, 10);
    assert_eq!(state.cursor, (0, 10));
    assert_eq!(state.preferred_column, None);

    // 1. Move Down to Line 1 (only 4 chars).
    // Cursor should clamp to 4, but preferred_column must record 10!
    assert!(state.move_down());
    assert_eq!(state.cursor, (1, 4));
    assert_eq!(state.preferred_column, Some(10));

    // 2. Move Down to Line 2 (20 chars).
    // Cursor should restore to preferred_column 10!
    assert!(state.move_down());
    assert_eq!(state.cursor, (2, 10));
    assert_eq!(state.preferred_column, Some(10));

    // 3. Move Up back to Line 1 (4 chars).
    // Cursor should clamp to 4 again, while remembering 10.
    assert!(state.move_up());
    assert_eq!(state.cursor, (1, 4));
    assert_eq!(state.preferred_column, Some(10));

    // 4. Move Up back to Line 0 (15 chars).
    // Cursor should restore to 10.
    assert!(state.move_up());
    assert_eq!(state.cursor, (0, 10));
    assert_eq!(state.preferred_column, Some(10));

    // 5. Horizontal movement clears preferred_column
    assert!(state.move_left());
    assert_eq!(state.cursor, (0, 9));
    assert_eq!(state.preferred_column, None);

    // 6. Move Down with new column (9)
    assert!(state.move_down());
    assert_eq!(state.cursor, (1, 4));
    assert_eq!(state.preferred_column, Some(9));

    // Move Down to Line 2 -> restores to 9
    assert!(state.move_down());
    assert_eq!(state.cursor, (2, 9));
    assert_eq!(state.preferred_column, Some(9));
}

#[test]
fn test_vertical_navigation_boundaries() {
    let mut state = MultilineTextState::new("short\nlonger line\nend");

    // Move up from line 0 at col 3 -> moves to col 0
    state.set_cursor(0, 3);
    assert!(state.move_up());
    assert_eq!(state.cursor, (0, 0));

    // Move up from (0, 0) -> cannot move up
    assert!(!state.move_up());
    assert_eq!(state.cursor, (0, 0));

    // Move down from last line at col 0 -> moves to end of last line
    state.set_cursor(2, 0);
    assert!(state.move_down());
    assert_eq!(state.cursor, (2, 3));

    // Move down when already at end of last line -> cannot move down
    assert!(!state.move_down());
    assert_eq!(state.cursor, (2, 3));
}

#[test]
fn test_horizontal_navigation_and_wrapping() {
    let mut state = MultilineTextState::new("ab\ncd");

    state.set_cursor(0, 0);
    assert!(!state.move_left()); // Cannot move left at (0, 0)

    assert!(state.move_right()); // (0, 1)
    assert_eq!(state.cursor, (0, 1));

    assert!(state.move_right()); // (0, 2) - end of line 0
    assert_eq!(state.cursor, (0, 2));

    // Wrap to next line
    assert!(state.move_right()); // (1, 0)
    assert_eq!(state.cursor, (1, 0));

    // Wrap back to previous line
    assert!(state.move_left()); // (0, 2)
    assert_eq!(state.cursor, (0, 2));

    state.move_home();
    assert_eq!(state.cursor, (0, 0));

    state.move_end();
    assert_eq!(state.cursor, (0, 2));
}

#[test]
fn test_insert_newline_enter_key() {
    let mut state = MultilineTextState::new("hello world");
    state.set_cursor(0, 5); // After "hello", before " world"

    state.insert_newline();
    assert_eq!(state.text, "hello\n world");
    assert_eq!(state.cursor, (1, 0));
    assert_eq!(state.preferred_column, None);
    assert_eq!(state.line_count(), 2);
    assert_eq!(state.line_str(0), Some("hello"));
    assert_eq!(state.line_str(1), Some(" world"));
}

#[test]
fn test_page_up_and_page_down() {
    let mut state = MultilineTextState::new("0\n1\n2\n3\n4\n5\n6\n7\n8\n9");
    assert_eq!(state.line_count(), 10);

    state.set_cursor(0, 0);
    assert!(state.page_down(4));
    assert_eq!(state.cursor, (4, 0));

    assert!(state.page_down(4));
    assert_eq!(state.cursor, (8, 0));

    // Clamps to max line while preserving column 0
    assert!(state.page_down(4));
    assert_eq!(state.cursor, (9, 0));

    assert!(state.page_up(4));
    assert_eq!(state.cursor, (5, 0));

    assert!(state.page_up(10));
    assert_eq!(state.cursor, (0, 0));
}

#[test]
fn test_handle_key_dispatch() {
    let mut state = MultilineTextState::new("foo\nbar");
    state.set_cursor(0, 0);

    // Down arrow
    let outcome = state.handle_key(KeyCode::ArrowDown, false, false);
    assert!(outcome.consumed);
    assert!(outcome.cursor_moved);
    assert!(!outcome.text_changed);
    assert_eq!(state.cursor, (1, 0));

    // Enter key
    let outcome = state.handle_key(KeyCode::Enter, false, false);
    assert!(outcome.consumed);
    assert!(outcome.text_changed);
    assert!(outcome.cursor_moved);
    assert_eq!(state.cursor, (2, 0));
    assert_eq!(state.line_count(), 3);

    // Backspace key
    let outcome = state.handle_key(KeyCode::Backspace, false, false);
    assert!(outcome.consumed);
    assert!(outcome.text_changed);
    assert_eq!(state.line_count(), 2);
}

#[test]
fn test_gutter_metrics_and_formatting() {
    // 5 lines -> minimum 2 digits, clamped to MIN_GUTTER_WIDTH (36.0)
    let metrics = GutterMetrics::compute(5, 8.0, 8.0);
    assert_eq!(metrics.total_lines, 5);
    assert_eq!(metrics.digits, 2);
    assert_eq!(metrics.width, MIN_GUTTER_WIDTH);

    assert_eq!(metrics.format_line_number(0), "1");
    assert_eq!(metrics.format_line_number(4), "5");

    // 150 lines -> 3 digits
    let metrics_large = GutterMetrics::compute(150, 8.0, 8.0);
    assert_eq!(metrics_large.digits, 3);
    assert_eq!(metrics_large.format_line_number(0), "1");
    assert_eq!(metrics_large.format_line_number(9), "10");
    assert_eq!(metrics_large.format_line_number(99), "100");

    // Very large line count where raw width exceeds MIN_GUTTER_WIDTH
    let metrics_huge = GutterMetrics::compute(10_000, 10.0, 10.0);
    assert_eq!(metrics_huge.digits, 5);
    assert_eq!(metrics_huge.width, 5.0 * 10.0 + 20.0); // 70.0 > 36.0
}

#[test]
fn test_shortcuts_select_all_undo_redo() {
    let mut state = MultilineTextState::new("hello world");
    state.set_cursor(0, 5);

    // Ctrl + A -> Select All
    let outcome = state.handle_key(KeyCode::Character('a'), true, false);
    assert!(outcome.consumed);
    assert_eq!(state.selection, Some(((0, 0), (0, 11))));
    assert_eq!(state.selected_text(), Some("hello world".to_string()));

    // Typing while having selection replaces selected text
    state.insert_str("foo");
    assert_eq!(state.text, "foo");
    assert_eq!(state.selection, None);

    // Ctrl + Z -> Undo
    let outcome = state.handle_key(KeyCode::Character('z'), true, false);
    assert!(outcome.consumed);
    assert!(outcome.text_changed);
    assert_eq!(state.text, "hello world");

    // Ctrl + Y -> Redo
    let outcome = state.handle_key(KeyCode::Character('y'), true, false);
    assert!(outcome.consumed);
    assert!(outcome.text_changed);
    assert_eq!(state.text, "foo");

    // Arrow key clears selection
    state.set_selection((0, 0), (0, 2));
    assert!(state.selection.is_some());
    let outcome = state.handle_key(KeyCode::ArrowRight, false, false);
    assert!(outcome.consumed);
    assert!(state.selection.is_none());

    // Shift + Right extends selection
    state.set_cursor(0, 1);
    let outcome = state.handle_key(KeyCode::ArrowRight, false, true);
    assert!(outcome.consumed);
    assert_eq!(state.selection, Some(((0, 1), (0, 2))));
}

#[test]
fn test_gutter_visible_line_range() {
    // 100 lines, line_height 20.0, viewport 100.0 (5 visible lines), overscan 2
    // scroll_y = 0.0 -> visible 0..5, with overscan 0..7
    let range = GutterMetrics::visible_line_range(0.0, 100.0, 20.0, 100, 2);
    assert_eq!(range, 0..7);

    // scroll_y = 200.0 (line 10) -> visible lines 10..15, with overscan 8..17
    let range_scrolled = GutterMetrics::visible_line_range(200.0, 100.0, 20.0, 100, 2);
    assert_eq!(range_scrolled, 8..17);

    // Zero line height edge case
    assert_eq!(
        GutterMetrics::visible_line_range(0.0, 100.0, 0.0, 100, 2),
        0..0
    );
}

#[test]
fn test_gutter_style_defaults() {
    let style = GutterStyle::default();
    assert_eq!(style.padding_left, 8.0);
    assert_eq!(style.padding_right, 8.0);
}