// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Text Input Drag-to-Select Data Model (`iris-widgets::declarative::tests::drag_selection_tests`)
//!
//! Validates `TextInputState` continuous mouse drag-selection, forward/backward selection bounds,
//! single-click cancellation, UTF-8 cluster safety, arrow key cursor collapse, and replacement behavior.

use crate::input::TextInputState;

#[test]
fn test_text_input_state_drag_select_right() {
    let mut state = TextInputState::new("Hello World");
    assert_eq!(state.cursor_byte_idx, 11);
    assert!(!state.is_drag_selecting);
    assert_eq!(state.selection_range(), None);

    // Mouse pressed at letter index 2 ('l')
    state.start_drag_selection(2);
    assert!(state.is_drag_selecting);
    assert_eq!(state.selection_anchor, Some(2));
    assert_eq!(state.cursor_byte_idx, 2);
    assert_eq!(state.selection_range(), None);

    // Drag rightwards to index 7 ('o')
    state.update_drag_selection(7);
    assert!(state.is_drag_selecting);
    assert_eq!(state.cursor_byte_idx, 7);
    assert_eq!(state.selection_range(), Some((2, 7)));
    assert_eq!(state.selected_text(), Some("llo W"));

    // Conclude mouse drag gesture
    state.end_drag_selection();
    assert!(!state.is_drag_selecting);
    assert_eq!(state.selection_anchor, None);
    assert_eq!(state.selection_range(), Some((2, 7)));
    assert_eq!(state.cursor_byte_idx, 7);
    assert_eq!(state.selected_text(), Some("llo W"));
}

#[test]
fn test_text_input_state_drag_select_left() {
    let mut state = TextInputState::new("Hello World");

    // Mouse pressed at letter index 8 ('r')
    state.start_drag_selection(8);
    assert!(state.is_drag_selecting);
    assert_eq!(state.selection_anchor, Some(8));
    assert_eq!(state.cursor_byte_idx, 8);
    assert_eq!(state.selection_range(), None);

    // Drag leftwards backwards to index 3 ('l')
    state.update_drag_selection(3);
    assert!(state.is_drag_selecting);
    assert_eq!(state.cursor_byte_idx, 3);
    // Selection bounds must be strictly normalized with start <= end
    assert_eq!(state.selection_range(), Some((3, 8)));
    assert_eq!(state.selected_text(), Some("lo Wo"));

    // Mouse released
    state.end_drag_selection();
    assert!(!state.is_drag_selecting);
    assert_eq!(state.selection_anchor, None);
    assert_eq!(state.selection_range(), Some((3, 8)));
    assert_eq!(state.cursor_byte_idx, 3);
}

#[test]
fn test_text_input_state_drag_select_single_click_cancellation() {
    let mut state = TextInputState::new("Hello World");
    state.select_all();
    assert!(state.has_selection());

    // Single click: mouse down at index 4, mouse up at index 4 without movement
    state.start_drag_selection(4);
    assert!(state.is_drag_selecting);
    assert_eq!(state.cursor_byte_idx, 4);
    assert_eq!(state.selection_range(), None);

    state.end_drag_selection();
    assert!(!state.is_drag_selecting);
    assert_eq!(state.selection_anchor, None);
    assert_eq!(state.selection_range(), None);
    assert!(!state.has_selection());
    assert_eq!(state.cursor_byte_idx, 4);
}

#[test]
fn test_text_input_state_drag_select_cached_advances() {
    let mut state = TextInputState::new("Engine");
    // Advances: 'E'(10), 'n'(20), 'g'(30), 'i'(40), 'n'(50), 'e'(60)
    let advances = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0];

    // Start at letter 'n' (advance near 20 -> index 1)
    state.start_drag_selection(1);
    assert_eq!(state.cursor_byte_idx, 1);

    // Drag to advance near 50 ('n' -> index 5)
    state.update_drag_selection_from_cached_advances(&advances, 52.0);
    assert_eq!(state.cursor_byte_idx, 5);
    assert_eq!(state.selection_range(), Some((1, 5)));
    assert_eq!(state.selected_text(), Some("ngin"));

    state.end_drag_selection();
    assert_eq!(state.selection_range(), Some((1, 5)));
}

#[test]
fn test_text_input_state_drag_select_utf8_clusters() {
    // "Ağaç" -> 'A' (0..1), 'ğ' (1..3), 'a' (3..4), 'ç' (4..6)
    let mut state = TextInputState::new("Ağaç");
    let clusters = [(0, 10.0), (1, 24.0), (3, 34.0), (4, 48.0)];

    // Drag from 'A' start to after 'ç'
    state.start_drag_selection(0);
    state.update_drag_selection_from_cached_clusters(&clusters, 46.0);
    assert_eq!(state.cursor_byte_idx, 6);
    assert_eq!(state.selection_range(), Some((0, 6)));
    assert_eq!(state.selected_text(), Some("Ağaç"));

    // Drag backwards to between 'A' and 'ğ'
    state.update_drag_selection_from_cached_clusters(&clusters, 12.0);
    assert_eq!(state.cursor_byte_idx, 1);
    assert_eq!(state.selection_range(), Some((0, 1)));
    assert_eq!(state.selected_text(), Some("A"));

    state.end_drag_selection();
    assert_eq!(state.selection_range(), Some((0, 1)));
}

#[test]
fn test_text_input_state_drag_select_arrow_collapse() {
    let mut state = TextInputState::new("Industrial Framework");
    state.set_selection_range(11, 20); // "Framework"
    assert_eq!(state.selection_range(), Some((11, 20)));
    assert_eq!(state.selected_text(), Some("Framework"));

    // Press Left arrow: collapses cursor to start boundary (11) and clears selection
    state.move_left();
    assert_eq!(state.cursor_byte_idx, 11);
    assert_eq!(state.selection_range(), None);
    assert!(!state.has_selection());

    // Select again
    state.set_selection_range(0, 10); // "Industrial"
    assert_eq!(state.selection_range(), Some((0, 10)));

    // Press Right arrow: collapses cursor to end boundary (10) and clears selection
    state.move_right();
    assert_eq!(state.cursor_byte_idx, 10);
    assert_eq!(state.selection_range(), None);
    assert!(!state.has_selection());
}

#[test]
fn test_text_input_state_drag_select_replace_and_delete() {
    let mut state = TextInputState::new("Hello World");
    state.set_selection_range(6, 11); // "World"
    assert_eq!(state.selected_text(), Some("World"));

    // Type new text over selection: replaces selected text and places cursor after insertion
    state.insert_str("Rust");
    assert_eq!(state.buffer, "Hello Rust");
    assert_eq!(state.cursor_byte_idx, 10);
    assert_eq!(state.selection_range(), None);
    assert!(!state.has_selection());

    // Select "Hello" and delete
    state.set_selection_range(0, 5);
    assert_eq!(state.selected_text(), Some("Hello"));
    assert!(state.delete_selection());
    assert_eq!(state.buffer, " Rust");
    assert_eq!(state.cursor_byte_idx, 0);
    assert_eq!(state.selection_range(), None);
}

#[test]
fn test_text_input_state_cancel_and_arbitrary_range() {
    let mut state = TextInputState::new("Autonomous Retained UI");

    // In-progress drag cancelled
    state.start_drag_selection(11);
    state.update_drag_selection(19);
    assert!(state.is_drag_selecting);
    assert!(state.has_selection());

    state.cancel_drag_selection();
    assert!(!state.is_drag_selecting);
    assert_eq!(state.selection_anchor, None);
    assert_eq!(state.selection_range(), None);

    // Set inverted range (end before start): should normalize with min <= max
    state.set_selection_range(19, 11);
    assert_eq!(state.selection_range(), Some((11, 19)));
    assert_eq!(state.selected_text(), Some("Retained"));

    // Set identical range: collapses to None
    state.set_selection_range(5, 5);
    assert_eq!(state.selection_range(), None);
}