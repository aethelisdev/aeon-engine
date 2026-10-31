// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Text input editing buffer, IME composition state, and UTF-8 cursor safety.

use std::sync::LazyLock;

use crate::clipboard::SystemClipboard;
use iris_core::platform::ClipboardProvider;

/// Global shared platform clipboard instance initialized lazily with operating system bindings.
static GLOBAL_SYSTEM_CLIPBOARD: LazyLock<SystemClipboard> = LazyLock::new(SystemClipboard::new);

/// Helper function ensuring an arbitrary byte index snaps to a valid UTF-8 boundary within a buffer slice.
#[inline]
fn clamp_to_char_boundary(buffer: &str, mut idx: usize) -> usize {
    if idx >= buffer.len() {
        return buffer.len();
    }
    while idx > 0 && !buffer.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

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
    /// Anchor byte index where a drag-selection gesture originated.
    pub selection_anchor: Option<usize>,
    /// Indicates whether a continuous mouse drag-selection gesture is in progress.
    pub is_drag_selecting: bool,
}

/// Copies a text slice to the shared system clipboard.
pub fn copy_to_clipboard(text: &str) {
    GLOBAL_SYSTEM_CLIPBOARD.set_text(text);
}

/// Retrieves the active text string from the shared system clipboard if present.
#[must_use]
pub fn get_clipboard_text() -> Option<String> {
    GLOBAL_SYSTEM_CLIPBOARD.get_text()
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
            selection_anchor: None,
            is_drag_selecting: false,
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
            self.selection_anchor = Some(0);
            self.cursor_byte_idx = self.buffer.len();
            self.is_drag_selecting = false;
        }
    }

    /// Clears any active text selection range and drag gesture metadata.
    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.selection_anchor = None;
        self.is_drag_selecting = false;
    }

    /// Returns the currently active text selection range `(start_byte_idx, end_byte_idx)` if present.
    ///
    /// The returned bounds are guaranteed to satisfy `start < end <= buffer.len()` along valid UTF-8
    /// character boundaries.
    ///
    /// # Returns
    /// An optional tuple `(start_byte_idx, end_byte_idx)` of the active selection.
    #[must_use]
    #[inline]
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection
            .filter(|(s, e)| s < e && *e <= self.buffer.len())
    }

    /// Initiates a continuous mouse drag-selection gesture anchored at the specified byte offset.
    ///
    /// Sets [`Self::cursor_byte_idx`] and [`Self::selection_anchor`] to the clamped character boundary,
    /// marks [`Self::is_drag_selecting`] as `true`, and clears any previous selection range.
    ///
    /// # Arguments
    /// * `anchor_idx` - Target byte index where the drag gesture begins.
    pub fn start_drag_selection(&mut self, anchor_idx: usize) {
        let safe_idx = clamp_to_char_boundary(&self.buffer, anchor_idx);
        self.cursor_byte_idx = safe_idx;
        self.selection_anchor = Some(safe_idx);
        self.selection = None;
        self.is_drag_selecting = true;
    }

    /// Updates the continuous drag-selection range as the mouse moves to a new character position.
    ///
    /// If a drag gesture is active ([`Self::is_drag_selecting`]), updates [`Self::cursor_byte_idx`]
    /// to `current_idx` and sets [`Self::selection`] to `(min(anchor, current), max(anchor, current))`.
    /// If `anchor == current_idx`, the selection range is collapsed to `None`.
    ///
    /// # Arguments
    /// * `current_idx` - Current byte offset under the mouse pointer.
    pub fn update_drag_selection(&mut self, current_idx: usize) {
        if !self.is_drag_selecting {
            return;
        }
        let safe_idx = clamp_to_char_boundary(&self.buffer, current_idx);
        self.cursor_byte_idx = safe_idx;

        if let Some(anchor) = self.selection_anchor {
            if anchor == safe_idx {
                self.selection = None;
            } else {
                let start = anchor.min(safe_idx);
                let end = anchor.max(safe_idx);
                self.selection = Some((start, end));
            }
        }
    }

    /// Updates the drag-selection range using precomputed character advance metrics.
    ///
    /// Computes the nearest character boundary using $O(\log N)$ binary search and updates
    /// the selection range relative to the active anchor.
    ///
    /// # Arguments
    /// * `glyph_advances` - Monotonically increasing horizontal character right-edge offsets.
    /// * `local_x` - Horizontal cursor coordinate relative to text origin.
    pub fn update_drag_selection_from_cached_advances(
        &mut self,
        glyph_advances: &[f32],
        local_x: f32,
    ) {
        if !self.is_drag_selecting {
            return;
        }
        let current_idx = iris_text::hit_test_from_cached_advances(glyph_advances, local_x);
        self.update_drag_selection(current_idx);
    }

    /// Updates the drag-selection range using precomputed glyph clusters for multi-byte UTF-8 and emoji safety.
    ///
    /// # Arguments
    /// * `clusters` - Slice of `(start_byte_idx, right_x)` pairs.
    /// * `local_x` - Horizontal cursor coordinate relative to text origin.
    pub fn update_drag_selection_from_cached_clusters(
        &mut self,
        clusters: &[(usize, f32)],
        local_x: f32,
    ) {
        if !self.is_drag_selecting {
            return;
        }
        let current_idx =
            iris_text::hit_test_from_cached_clusters(clusters, self.buffer.len(), local_x);
        self.update_drag_selection(current_idx);
    }

    /// Updates the drag-selection range by querying an explicit typography engine reference.
    ///
    /// # Arguments
    /// * `local_x` - Horizontal cursor coordinate relative to text origin.
    /// * `font_size` - Font size in physical pixels.
    /// * `text_system` - Active typography system.
    pub fn update_drag_selection_at_position(
        &mut self,
        local_x: f32,
        font_size: f32,
        text_system: &mut iris_text::TextSystem,
    ) {
        if !self.is_drag_selecting {
            return;
        }
        let current_idx = text_system.index_for_position(&self.buffer, font_size, local_x);
        self.update_drag_selection(current_idx);
    }

    /// Concludes an active drag-selection gesture (e.g. upon mouse button release).
    ///
    /// Marks [`Self::is_drag_selecting`] as `false`, clears the transient anchor, and normalizes
    /// any zero-width selection range to `None`.
    pub fn end_drag_selection(&mut self) {
        self.is_drag_selecting = false;
        self.selection_anchor = None;
        if let Some((s, e)) = self.selection
            && s >= e
        {
            self.selection = None;
        }
    }

    /// Cancels an in-progress drag selection gesture without retaining any selected range.
    pub fn cancel_drag_selection(&mut self) {
        self.clear_selection();
    }

    /// Explicitly configures an active selection range between two arbitrary byte boundaries.
    ///
    /// Safely clamps both bounds to valid UTF-8 character boundaries and normalizes `start <= end`.
    /// If `start == end`, the selection is cleared to `None`.
    ///
    /// # Arguments
    /// * `start` - First byte offset boundary.
    /// * `end` - Second byte offset boundary.
    pub fn set_selection_range(&mut self, start: usize, end: usize) {
        let safe_start = clamp_to_char_boundary(&self.buffer, start);
        let safe_end = clamp_to_char_boundary(&self.buffer, end);
        if safe_start == safe_end {
            self.selection = None;
        } else {
            let min_idx = safe_start.min(safe_end);
            let max_idx = safe_start.max(safe_end);
            self.selection = Some((min_idx, max_idx));
        }
        self.selection_anchor = None;
        self.is_drag_selecting = false;
    }

    /// Returns whether a non-empty text selection range is currently active.
    #[must_use]
    #[inline]
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

    /// Copies the currently selected text using the provided clipboard provider.
    ///
    /// # Arguments
    /// * `clipboard` - Provider implementing [`ClipboardProvider`].
    ///
    /// # Returns
    /// The copied text string if a selection was active and successfully extracted.
    pub fn copy_selection_with_provider(
        &self,
        clipboard: &impl ClipboardProvider,
    ) -> Option<String> {
        let text = self.selected_text().map(|s| s.to_string());
        if let Some(ref t) = text {
            clipboard.set_text(t);
        }
        text
    }

    /// Cuts the currently selected text using the provided clipboard provider and removes it from the buffer.
    ///
    /// # Arguments
    /// * `clipboard` - Provider implementing [`ClipboardProvider`].
    ///
    /// # Returns
    /// The cut text string if a selection was active and successfully deleted.
    pub fn cut_selection_with_provider(
        &mut self,
        clipboard: &impl ClipboardProvider,
    ) -> Option<String> {
        let text = self.copy_selection_with_provider(clipboard);
        self.delete_selection();
        text
    }

    /// Pastes text retrieved from the provided clipboard provider at the current cursor position.
    pub fn paste_from_provider(&mut self, clipboard: &impl ClipboardProvider) -> bool {
        if let Some(text) = clipboard.get_text() {
            self.paste(&text);
            true
        } else {
            false
        }
    }

    /// Copies the currently selected text into the system clipboard.
    pub fn copy_selection(&self) -> Option<String> {
        let text = self.selected_text().map(|s| s.to_string());
        if let Some(ref t) = text {
            copy_to_clipboard(t);
        }
        text
    }

    /// Cuts the currently selected text into the system clipboard and deletes it.
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

    /// Sets the cursor byte index from precomputed cumulative character advances.
    ///
    /// Clears active selection and resolves the caret offset using $O(\log N)$ binary search.
    ///
    /// # Arguments
    /// * `glyph_advances` - Slice of monotonically increasing character right-edge X positions.
    /// * `local_x` - Horizontal click position relative to text origin.
    pub fn set_cursor_from_cached_advances(&mut self, glyph_advances: &[f32], local_x: f32) {
        self.clear_selection();
        self.cursor_byte_idx = iris_text::hit_test_from_cached_advances(glyph_advances, local_x);
    }

    /// Sets the cursor byte index from precomputed glyph clusters `(start_byte_idx, right_x)`.
    ///
    /// Clears active selection and resolves multi-byte UTF-8 character boundaries.
    ///
    /// # Arguments
    /// * `clusters` - Slice of `(start_byte_idx, right_x)` pairs.
    /// * `local_x` - Horizontal click position relative to text origin.
    pub fn set_cursor_from_cached_clusters(&mut self, clusters: &[(usize, f32)], local_x: f32) {
        self.clear_selection();
        self.cursor_byte_idx =
            iris_text::hit_test_from_cached_clusters(clusters, self.buffer.len(), local_x);
    }

    /// Sets the cursor byte index by querying an explicit mutable typography engine reference.
    ///
    /// # Arguments
    /// * `local_x` - Horizontal click position relative to text origin.
    /// * `font_size` - Physical font size in pixels.
    /// * `text_system` - Active typography engine.
    pub fn set_cursor_at_position(
        &mut self,
        local_x: f32,
        font_size: f32,
        text_system: &mut iris_text::TextSystem,
    ) {
        self.clear_selection();
        self.cursor_byte_idx = text_system.index_for_position(&self.buffer, font_size, local_x);
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
    ///
    /// If an active selection is present, collapses the cursor to the selection start boundary.
    pub fn move_left(&mut self) {
        if let Some((start, _)) = self.selection.take() {
            self.clear_selection();
            self.cursor_byte_idx = start;
            return;
        }
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
    ///
    /// If an active selection is present, collapses the cursor to the selection end boundary.
    pub fn move_right(&mut self) {
        if let Some((_, end)) = self.selection.take() {
            self.clear_selection();
            self.cursor_byte_idx = end;
            return;
        }
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
}