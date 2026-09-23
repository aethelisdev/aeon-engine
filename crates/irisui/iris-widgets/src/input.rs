// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Text input editing buffer, IME composition state, and UTF-8 cursor safety.

/// Encapsulated state and UTF-8 safe editing buffer for interactive text input widgets.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextInputState {
    /// Accumulated text string buffer.
    pub buffer: String,
    /// Byte index of the editing cursor.
    pub cursor_byte_idx: usize,
    /// In-progress uncommitted IME composition string.
    pub ime_preedit: Option<String>,
    /// In-progress IME selection/cursor range.
    pub ime_cursor: Option<(usize, usize)>,
}

impl TextInputState {
    /// Creates a new state initialized with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        let buffer = text.into();
        let cursor_byte_idx = buffer.len();
        Self {
            buffer,
            cursor_byte_idx,
            ime_preedit: None,
            ime_cursor: None,
        }
    }

    /// Sets the active in-progress IME preedit composition string.
    pub fn set_ime_preedit(&mut self, text: impl Into<String>, cursor: Option<(usize, usize)>) {
        let txt = text.into();
        if txt.is_empty() {
            self.ime_preedit = None;
            self.ime_cursor = None;
        } else {
            self.ime_preedit = Some(txt);
            self.ime_cursor = cursor;
        }
    }

    /// Clears any active in-progress IME preedit composition.
    pub fn clear_ime_preedit(&mut self) {
        self.ime_preedit = None;
        self.ime_cursor = None;
    }

    /// Commits finalized IME text into the buffer and clears preedit.
    pub fn commit_ime(&mut self, text: &str) {
        self.clear_ime_preedit();
        self.insert_str(text);
    }

    /// Formats the display string with in-progress IME preedit text inserted at cursor position.
    pub fn display_text(&self) -> String {
        if let Some(ref preedit) = self.ime_preedit {
            let (head, tail) = self
                .buffer
                .split_at(self.cursor_byte_idx.min(self.buffer.len()));
            format!("{}[{}]{}", head, preedit, tail)
        } else {
            self.buffer.clone()
        }
    }

    /// Inserts a character or substring at the current cursor position, maintaining UTF-8 boundary integrity.
    pub fn insert_str(&mut self, text: &str) {
        self.cursor_byte_idx = self.cursor_byte_idx.min(self.buffer.len());
        while !self.buffer.is_char_boundary(self.cursor_byte_idx) && self.cursor_byte_idx > 0 {
            self.cursor_byte_idx -= 1;
        }
        self.buffer.insert_str(self.cursor_byte_idx, text);
        self.cursor_byte_idx += text.len();
    }

    /// Deletes the previous UTF-8 character before the cursor (Backspace).
    pub fn backspace(&mut self) {
        if self.cursor_byte_idx == 0 || self.buffer.is_empty() {
            return;
        }
        self.cursor_byte_idx = self.cursor_byte_idx.min(self.buffer.len());
        let prev_boundary = self.buffer[..self.cursor_byte_idx]
            .char_indices()
            .last()
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        self.buffer.drain(prev_boundary..self.cursor_byte_idx);
        self.cursor_byte_idx = prev_boundary;
    }

    /// Deletes the next UTF-8 character after the cursor (Delete).
    pub fn delete(&mut self) {
        if self.cursor_byte_idx >= self.buffer.len() {
            return;
        }
        let next_boundary = self.buffer[self.cursor_byte_idx..]
            .char_indices()
            .nth(1)
            .map(|(idx, _)| self.cursor_byte_idx + idx)
            .unwrap_or(self.buffer.len());
        self.buffer.drain(self.cursor_byte_idx..next_boundary);
    }

    /// Moves cursor one character left safely across multi-byte UTF-8 boundaries.
    pub fn move_left(&mut self) {
        if self.cursor_byte_idx > 0 {
            self.cursor_byte_idx = self.buffer[..self.cursor_byte_idx]
                .char_indices()
                .last()
                .map(|(idx, _)| idx)
                .unwrap_or(0);
        }
    }

    /// Moves cursor one character right safely across multi-byte UTF-8 boundaries.
    pub fn move_right(&mut self) {
        if self.cursor_byte_idx < self.buffer.len() {
            self.cursor_byte_idx = self.buffer[self.cursor_byte_idx..]
                .char_indices()
                .nth(1)
                .map(|(idx, _)| self.cursor_byte_idx + idx)
                .unwrap_or(self.buffer.len());
        }
    }
}

#[cfg(test)]
mod tests {
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
}