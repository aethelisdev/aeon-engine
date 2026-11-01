// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Text Undo and Redo History Integration (`controls::text_input::undo`)
//!
//! Re-exports core undo/redo models from [`iris_core::dispatch::undo`] and provides
//! widget-level integration helpers for interactive text controls.

pub use iris_core::dispatch::undo::{
    DEFAULT_MAX_UNDO_DEPTH, TEXT_UNDO_COALESCE_TIMEOUT_MS, TextSnapshot, TextUndoStack,
    UndoActionKind, is_word_boundary,
};

#[cfg(test)]
mod tests {
    use crate::input::TextInputState;

    #[test]
    fn test_text_input_state_undo_redo_integration() {
        let mut state = TextInputState::new("Hello");
        assert!(!state.can_undo());

        state.undo_stack.checkpoint(state.create_snapshot());
        state.buffer = "Hello World".to_string();
        state.cursor_byte_idx = 11;
        assert!(state.can_undo());

        assert!(state.undo());
        assert_eq!(state.buffer, "Hello");
        assert_eq!(state.cursor_byte_idx, 5);
        assert!(state.can_redo());

        assert!(state.redo());
        assert_eq!(state.buffer, "Hello World");
        assert_eq!(state.cursor_byte_idx, 11);
    }
}