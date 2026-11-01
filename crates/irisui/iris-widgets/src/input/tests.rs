// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Text Input Buffer & Caret Operations (`input::tests`)

use super::*;

#[test]
fn test_text_input_state_utf8_safety() {
    let mut state = TextInputState::new("Ağaç");
    assert_eq!(state.cursor_byte_idx, 6); // 1 + 2 + 1 + 2 = 6 bytes

    state.backspace();
    assert_eq!(state.buffer, "Ağa");
    assert_eq!(state.cursor_byte_idx, 4);

    state.move_left();
    state.insert_str("k");
    assert_eq!(state.buffer, "Ağka");
}

#[test]
fn test_text_input_state_ime() {
    let mut state = TextInputState::new("Hello ");
    state.set_ime_preedit("世界", Some((0, 2)));
    assert_eq!(state.display_text(), "Hello [世界]");

    state.commit_ime("世界");
    assert_eq!(state.display_text(), "Hello 世界");
    assert_eq!(state.buffer, "Hello 世界");
}

#[test]
fn test_text_input_state_selection_and_clipboard() {
    let _guard = CLIPBOARD_TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut state = TextInputState::new("Hello World");
    assert!(!state.has_selection());

    state.select_all();
    assert!(state.has_selection());
    assert_eq!(state.selected_text(), Some("Hello World"));

    let copied = state.copy_selection();
    assert_eq!(copied, Some("Hello World".to_owned()));
    assert_eq!(get_clipboard_text(), Some("Hello World".to_owned()));

    state.insert_str("Aeon");
    assert_eq!(state.buffer, "Aeon");
    assert!(!state.has_selection());

    state.move_home();
    assert_eq!(state.cursor_byte_idx, 0);

    state.move_end();
    assert_eq!(state.cursor_byte_idx, 4);

    state.paste(" Engine");
    assert_eq!(state.buffer, "Aeon Engine");
}

#[test]
fn test_text_input_state_clipboard_provider_integration() {
    use iris_core::platform::InMemoryClipboard;

    let clipboard = InMemoryClipboard::new();
    let mut state = TextInputState::new("Industrial Framework");

    state.select_all();
    let cut = state.cut_selection_with_provider(&clipboard);
    assert_eq!(cut, Some("Industrial Framework".to_owned()));
    assert_eq!(
        clipboard.get_text(),
        Some("Industrial Framework".to_owned())
    );
    assert_eq!(state.buffer, "");

    assert!(state.paste_from_provider(&clipboard));
    assert_eq!(state.buffer, "Industrial Framework");
}

#[test]
fn test_text_input_state_hit_testing() {
    let mut state = TextInputState::new("Engine");
    assert_eq!(state.cursor_byte_idx, 6);

    // Advances: 'E' (10), 'n' (20), 'g' (30), 'i' (40), 'n' (50), 'e' (60)
    let advances = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0];

    // Click at 0.0 -> start (0)
    state.set_cursor_from_cached_advances(&advances, 0.0);
    assert_eq!(state.cursor_byte_idx, 0);

    // Click at 12.0 (near 'n') -> index 1
    state.set_cursor_from_cached_advances(&advances, 12.0);
    assert_eq!(state.cursor_byte_idx, 1);

    // Click at 58.0 (near last 'e') -> index 6
    state.set_cursor_from_cached_advances(&advances, 58.0);
    assert_eq!(state.cursor_byte_idx, 6);

    // Test with explicit TextSystem
    let mut text_sys = iris_text::TextSystem::new();
    state.set_cursor_at_position(0.0, 14.0, &mut text_sys);
    assert_eq!(state.cursor_byte_idx, 0);
}

#[test]
fn test_text_input_state_undo_redo() {
    let mut state = TextInputState::new("Initial");
    assert!(!state.can_undo());
    assert!(!state.can_redo());

    state.undo_stack.checkpoint(state.create_snapshot());
    state.buffer = "Initial Edited".to_string();
    state.cursor_byte_idx = 14;

    assert!(state.can_undo());
    assert!(state.undo());
    assert_eq!(state.buffer, "Initial");
    assert_eq!(state.cursor_byte_idx, 7);

    assert!(state.can_redo());
    assert!(state.redo());
    assert_eq!(state.buffer, "Initial Edited");
    assert_eq!(state.cursor_byte_idx, 14);
}