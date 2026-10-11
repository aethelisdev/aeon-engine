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
    /// Active text selection range: `(start_byte_idx, end_byte_idx)` where `start <= end`.
    pub selection: Option<(usize, usize)>,
}

/// Thread-safe in-process text clipboard storage for desktop editing operations.
static TEXT_CLIPBOARD: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// Copies a text slice to the global in-process text clipboard.
pub fn copy_to_clipboard(text: &str) {
    if let Ok(mut lock) = TEXT_CLIPBOARD.lock() {
        *lock = Some(text.to_string());
    }
}

/// Retrieves the active text string from the global in-process text clipboard if present.
#[must_use]
pub fn get_clipboard_text() -> Option<String> {
    TEXT_CLIPBOARD.lock().ok().and_then(|lock| lock.clone())
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
            selection: None,
        }
    }

    /// Returns a borrowed slice of the active text string buffer.
    #[must_use]
    #[inline]
    pub fn text(&self) -> &str {
        &self.buffer
    }

    /// Selects the entire text buffer range for highlight and replacement.
    pub fn select_all(&mut self) {
        if !self.buffer.is_empty() {
            self.selection = Some((0, self.buffer.len()));
            self.cursor_byte_idx = self.buffer.len();
        }
    }

    /// Clears any active text selection range.
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Returns whether a non-empty text selection range is currently active.
    #[must_use]
    pub fn has_selection(&self) -> bool {
        self.selection
            .is_some_and(|(s, e)| s < e && e <= self.buffer.len())
    }

    /// Returns the currently selected substring slice if a valid range exists.
    #[must_use]
    pub fn selected_text(&self) -> Option<&str> {
        self.selection.and_then(|(s, e)| {
            if s <= e
                && e <= self.buffer.len()
                && self.buffer.is_char_boundary(s)
                && self.buffer.is_char_boundary(e)
            {
                Some(&self.buffer[s..e])
            } else {
                None
            }
        })
    }

    /// Deletes the currently selected text range and sets the cursor to the start index.
    ///
    /// Returns `true` if a non-empty selection was deleted.
    pub fn delete_selection(&mut self) -> bool {
        if let Some((s, e)) = self.selection.take()
            && s < e
            && e <= self.buffer.len()
            && self.buffer.is_char_boundary(s)
            && self.buffer.is_char_boundary(e)
        {
            self.buffer.drain(s..e);
            self.cursor_byte_idx = s;
            return true;
        }
        false
    }

    /// Copies the currently selected text into the global in-process clipboard.
    pub fn copy_selection(&self) -> Option<String> {
        let text = self.selected_text().map(|s| s.to_string());
        if let Some(ref t) = text {
            copy_to_clipboard(t);
        }
        text
    }

    /// Cuts the currently selected text into the global in-process clipboard and deletes it.
    pub fn cut_selection(&mut self) -> Option<String> {
        let text = self.copy_selection();
        self.delete_selection();
        text
    }

    /// Pastes the given text string at the current cursor position, replacing any selection.
    pub fn paste(&mut self, text: &str) {
        self.insert_str(text);
    }

    /// Clears the text buffer, reset cursor, and clears selection.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor_byte_idx = 0;
        self.selection = None;
        self.ime_preedit = None;
        self.ime_cursor = None;
    }

    /// Replaces the entire buffer with new text, resetting cursor to the end.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.buffer = text.into();
        self.cursor_byte_idx = self.buffer.len();
        self.selection = None;
        self.ime_preedit = None;
        self.ime_cursor = None;
    }

    /// Moves the cursor to the start of the buffer and clears selection (Home key).
    pub fn move_home(&mut self) {
        self.clear_selection();
        self.cursor_byte_idx = 0;
    }

    /// Moves the cursor to the end of the buffer and clears selection (End key).
    pub fn move_end(&mut self) {
        self.clear_selection();
        self.cursor_byte_idx = self.buffer.len();
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
    /// Replaces active selection if one exists.
    pub fn insert_str(&mut self, text: &str) {
        self.delete_selection();
        self.cursor_byte_idx = self.cursor_byte_idx.min(self.buffer.len());
        while !self.buffer.is_char_boundary(self.cursor_byte_idx) && self.cursor_byte_idx > 0 {
            self.cursor_byte_idx -= 1;
        }
        self.buffer.insert_str(self.cursor_byte_idx, text);
        self.cursor_byte_idx += text.len();
        self.selection = None;
    }

    /// Deletes the previous UTF-8 character before the cursor (Backspace), or deletes selection if active.
    pub fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }
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

    /// Deletes the next UTF-8 character after the cursor (Delete), or deletes selection if active.
    pub fn delete(&mut self) {
        if self.delete_selection() {
            return;
        }
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

    /// Moves cursor one character left safely across multi-byte UTF-8 boundaries and clears selection.
    pub fn move_left(&mut self) {
        self.clear_selection();
        if self.cursor_byte_idx > 0 {
            self.cursor_byte_idx = self.buffer[..self.cursor_byte_idx]
                .char_indices()
                .last()
                .map(|(idx, _)| idx)
                .unwrap_or(0);
        }
    }

    /// Moves cursor one character right safely across multi-byte UTF-8 boundaries and clears selection.
    pub fn move_right(&mut self) {
        self.clear_selection();
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

    #[test]
    fn test_text_input_state_selection_and_clipboard() {
        let mut state = TextInputState::new("Hello World");
        assert!(!state.has_selection());

        state.select_all();
        assert!(state.has_selection());
        assert_eq!(state.selected_text(), Some("Hello World"));

        let copied = state.copy_selection();
        assert_eq!(copied, Some("Hello World".to_string()));
        assert_eq!(get_clipboard_text(), Some("Hello World".to_string()));

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
}