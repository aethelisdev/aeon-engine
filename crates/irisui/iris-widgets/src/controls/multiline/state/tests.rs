// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::MultilineTextState;

#[test]
fn test_multiline_state_empty_initialization() {
    let state = MultilineTextState::new("");
    assert_eq!(state.line_count(), 1);
    assert_eq!(state.line_starts, vec![0]);
    assert_eq!(state.cursor, (0, 0));
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
    assert_eq!(state.line_str(0), Some(""));
    assert_eq!(state.line_char_count(0), 0);
    assert_eq!(state.line_byte_range(0), Some(0..0));
    assert_eq!(state.line_content_byte_range(0), Some(0..0));
    assert_eq!(state.cursor_byte_offset(), 0);
    assert_eq!(state.byte_offset_to_cursor(0), (0, 0));
}

#[test]
fn test_multiline_state_single_line_no_newline() {
    let state = MultilineTextState::new("hello");
    assert_eq!(state.line_count(), 1);
    assert_eq!(state.line_starts, vec![0]);
    assert_eq!(state.line_str(0), Some("hello"));
    assert_eq!(state.line_raw_str(0), Some("hello"));
    assert_eq!(state.line_char_count(0), 5);
    assert_eq!(state.line_byte_range(0), Some(0..5));
    assert_eq!(state.line_content_byte_range(0), Some(0..5));
}

#[test]
fn test_multiline_state_newlines_line_starts_topology() {
    let state = MultilineTextState::new("line1\nline2\nline3");
    assert_eq!(state.line_count(), 3);
    assert_eq!(state.line_starts, vec![0, 6, 12]);

    assert_eq!(state.line_str(0), Some("line1"));
    assert_eq!(state.line_str(1), Some("line2"));
    assert_eq!(state.line_str(2), Some("line3"));

    assert_eq!(state.line_raw_str(0), Some("line1\n"));
    assert_eq!(state.line_raw_str(1), Some("line2\n"));
    assert_eq!(state.line_raw_str(2), Some("line3"));

    assert_eq!(state.line_byte_range(0), Some(0..6));
    assert_eq!(state.line_content_byte_range(0), Some(0..5));
    assert_eq!(state.line_byte_range(1), Some(6..12));
    assert_eq!(state.line_content_byte_range(1), Some(6..11));
    assert_eq!(state.line_byte_range(2), Some(12..17));
    assert_eq!(state.line_content_byte_range(2), Some(12..17));
}

#[test]
fn test_multiline_state_consecutive_newlines() {
    let state = MultilineTextState::new("\n\n\n");
    assert_eq!(state.line_count(), 4);
    assert_eq!(state.line_starts, vec![0, 1, 2, 3]);

    assert_eq!(state.line_str(0), Some(""));
    assert_eq!(state.line_str(1), Some(""));
    assert_eq!(state.line_str(2), Some(""));
    assert_eq!(state.line_str(3), Some(""));

    assert_eq!(state.line_char_count(0), 0);
    assert_eq!(state.line_char_count(1), 0);
    assert_eq!(state.line_char_count(2), 0);
    assert_eq!(state.line_char_count(3), 0);
}

#[test]
fn test_multiline_state_crlf_windows_newlines() {
    let state = MultilineTextState::new("alpha\r\nbeta\r\ngamma");
    assert_eq!(state.line_count(), 3);
    assert_eq!(state.line_starts, vec![0, 7, 13]);

    assert_eq!(state.line_str(0), Some("alpha"));
    assert_eq!(state.line_str(1), Some("beta"));
    assert_eq!(state.line_str(2), Some("gamma"));

    assert_eq!(state.line_char_count(0), 5);
    assert_eq!(state.line_char_count(1), 4);
    assert_eq!(state.line_char_count(2), 5);

    assert_eq!(state.line_content_byte_range(0), Some(0..5));
    assert_eq!(state.line_byte_range(0), Some(0..7));
}

#[test]
fn test_multiline_state_utf8_turkish_and_emojis() {
    let text = "Şemsiye\nÖrdek 🦆\nÇiçek";
    let state = MultilineTextState::new(text);

    assert_eq!(state.line_count(), 3);
    assert_eq!(state.line_str(0), Some("Şemsiye"));
    assert_eq!(state.line_char_count(0), 7);

    assert_eq!(state.line_str(1), Some("Ördek 🦆"));
    assert_eq!(state.line_char_count(1), 7);

    assert_eq!(state.line_str(2), Some("Çiçek"));
    assert_eq!(state.line_char_count(2), 5);

    // Byte offset conversions
    // Line 1: 'Ö' (2 bytes), 'r'(1), 'd'(1), 'e'(1), 'k'(1), ' '(1), '🦆'(4)
    let line1_start = state.line_starts[1];
    assert_eq!(state.char_col_to_byte_offset(1, 0), line1_start);
    assert_eq!(state.char_col_to_byte_offset(1, 1), line1_start + 2); // After 'Ö'
    assert_eq!(state.char_col_to_byte_offset(1, 5), line1_start + 6); // Space
    assert_eq!(state.char_col_to_byte_offset(1, 6), line1_start + 7); // Before '🦆'
    assert_eq!(
        state.char_col_to_byte_offset(1, 7),
        line1_start + "Ördek 🦆".len()
    );

    // Roundtrip test
    for col in 0..=7 {
        let byte = state.char_col_to_byte_offset(1, col);
        let (l, c) = state.byte_offset_to_cursor(byte);
        assert_eq!(l, 1);
        assert_eq!(c, col);
    }
}

#[test]
fn test_multiline_state_cursor_clamping_and_movement() {
    let mut state = MultilineTextState::new("ab\ncdef\ng");

    state.set_cursor(100, 100);
    assert_eq!(state.cursor, (2, 1)); // Clamped to line 2, max col 1 ('g')

    state.cursor_to_start();
    assert_eq!(state.cursor, (0, 0));

    state.cursor_to_end();
    assert_eq!(state.cursor, (2, 1));

    state.set_cursor(1, 2);
    assert_eq!(state.cursor, (1, 2));
    assert_eq!(state.cursor_line(), 1);
    assert_eq!(state.cursor_col(), 2);
}

#[test]
fn test_multiline_state_insert_char_and_newlines() {
    let mut state = MultilineTextState::new("");

    state.insert_char('h');
    state.insert_char('i');
    assert_eq!(state.text, "hi");
    assert_eq!(state.cursor, (0, 2));
    assert_eq!(state.line_starts, vec![0]);

    state.insert_char('\n');
    assert_eq!(state.text, "hi\n");
    assert_eq!(state.cursor, (1, 0));
    assert_eq!(state.line_count(), 2);
    assert_eq!(state.line_starts, vec![0, 3]);

    state.insert_char('w');
    assert_eq!(state.text, "hi\nw");
    assert_eq!(state.cursor, (1, 1));
}

#[test]
fn test_multiline_state_insert_str_multiline() {
    let mut state = MultilineTextState::new("start ");
    state.cursor_to_end();

    state.insert_str("foo\nbar\nbaz");
    assert_eq!(state.text, "start foo\nbar\nbaz");
    assert_eq!(state.cursor, (2, 3));
    assert_eq!(state.line_count(), 3);
    assert_eq!(state.line_str(0), Some("start foo"));
    assert_eq!(state.line_str(1), Some("bar"));
    assert_eq!(state.line_str(2), Some("baz"));
}

#[test]
fn test_multiline_state_delete_char_before_cursor_backspace() {
    let mut state = MultilineTextState::new("line1\nline2");
    assert!(!state.delete_char_before_cursor()); // Cursor at (0, 0)

    // Move cursor to line 1 col 0 (start of "line2")
    state.set_cursor(1, 0);
    assert!(state.delete_char_before_cursor()); // Deletes newline, merges lines
    assert_eq!(state.text, "line1line2");
    assert_eq!(state.cursor, (0, 5));
    assert_eq!(state.line_count(), 1);
    assert_eq!(state.line_starts, vec![0]);

    // Backspace on multi-byte UTF-8: "Öz"
    state.set_text("Öz");
    state.cursor_to_end();
    assert_eq!(state.cursor, (0, 2));

    assert!(state.delete_char_before_cursor());
    assert_eq!(state.text, "Ö");
    assert_eq!(state.cursor, (0, 1));

    assert!(state.delete_char_before_cursor());
    assert_eq!(state.text, "");
    assert_eq!(state.cursor, (0, 0));
    assert!(!state.delete_char_before_cursor());
}

#[test]
fn test_multiline_state_delete_char_after_cursor_delete_key() {
    let mut state = MultilineTextState::new("ab\ncd");

    state.set_cursor(0, 1); // After 'a', before 'b'
    assert!(state.delete_char_after_cursor()); // Deletes 'b'
    assert_eq!(state.text, "a\ncd");
    assert_eq!(state.cursor, (0, 1));

    assert!(state.delete_char_after_cursor()); // At end of line 0, deletes '\n'
    assert_eq!(state.text, "acd");
    assert_eq!(state.cursor, (0, 1));
    assert_eq!(state.line_count(), 1);

    // Delete at end of document
    state.cursor_to_end();
    assert!(!state.delete_char_after_cursor());
}

#[test]
fn test_multiline_state_clear_and_set_text() {
    let mut state = MultilineTextState::new("hello\nworld");
    state.scroll_y = 42.0;
    state.clear();

    assert_eq!(state.text, "");
    assert_eq!(state.cursor, (0, 0));
    assert_eq!(state.line_starts, vec![0]);
    assert_eq!(state.scroll_y, 0.0);

    state.set_text("foo\nbar");
    assert_eq!(state.line_count(), 2);
    assert_eq!(state.line_str(0), Some("foo"));
    assert_eq!(state.line_str(1), Some("bar"));
}