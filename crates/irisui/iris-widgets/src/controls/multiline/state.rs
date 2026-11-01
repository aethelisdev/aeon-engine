// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Multi-line Text State Data Model (`controls::multiline::state`)
//!
//! Autonomous data structure managing multi-line text buffers, zero-allocation line start caching,
//! 2D character-based cursor coordinate translation, and UTF-8 safe editing primitives.

use std::ops::Range;

#[cfg(test)]
mod tests;

/// Clamps an arbitrary byte index to the nearest preceding valid UTF-8 character boundary.
#[inline]
fn clamp_to_char_boundary(slice: &str, mut idx: usize) -> usize {
    if idx >= slice.len() {
        return slice.len();
    }
    while idx > 0 && !slice.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

/// Encapsulated state and cached topological layout metadata for a multi-line text editing buffer.
///
/// Tracks raw text, logical 2D cursor coordinates, sticky vertical navigation columns,
/// cached line start byte offsets, and vertical viewport scrolling.
#[derive(Clone, Debug, PartialEq)]
pub struct MultilineTextState {
    /// Full UTF-8 text buffer managed by the multi-line editor.
    pub text: String,
    /// Current logical cursor position as `(line_index, column_index)` where both indices
    /// are 0-based character offsets (Unicode scalar counts, not raw bytes).
    pub cursor: (usize, usize),
    /// Visual character column remembered during vertical navigation across varying line lengths.
    pub preferred_column: Option<usize>,
    /// Cached byte offsets where each line begins within `text`.
    ///
    /// Invariant: Always contains at least one element (`line_starts[0] == 0`).
    /// The length of this vector is strictly equal to [`Self::line_count`].
    pub line_starts: Vec<usize>,
    /// Vertical scroll offset in logical display pixels/points.
    pub scroll_y: f32,
}

impl Default for MultilineTextState {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl MultilineTextState {
    /// Constructs a new multi-line text editing state initialized with the given text.
    ///
    /// Cursor is positioned at `(0, 0)` with no preferred column and zero scroll offset.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let mut state = Self {
            text,
            cursor: (0, 0),
            preferred_column: None,
            line_starts: Vec::new(),
            scroll_y: 0.0,
        };
        state.rebuild_line_starts();
        state
    }

    /// Rebuilds the cached `line_starts` table in $O(N)$ time by scanning newline delimiters.
    ///
    /// Invariant: `line_starts` is guaranteed to contain at least one entry (`line_starts[0] == 0`).
    /// Also validates and clamps the cursor within legal bounds.
    pub fn rebuild_line_starts(&mut self) {
        self.line_starts.clear();
        self.line_starts.push(0);

        for (idx, _) in self.text.match_indices('\n') {
            self.line_starts.push(idx + 1);
        }

        self.clamp_cursor();
    }

    /// Returns the total number of lines in the buffer.
    ///
    /// Invariant: Guaranteed to be at least 1, even for empty strings.
    #[inline]
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Returns `true` if the text buffer is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns the byte length of the text buffer.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Returns the absolute byte range for the specified line index, including any trailing newline.
    ///
    /// Returns `None` if `line_idx >= self.line_count()`.
    #[must_use]
    pub fn line_byte_range(&self, line_idx: usize) -> Option<Range<usize>> {
        if line_idx >= self.line_starts.len() {
            return None;
        }

        let start = self.line_starts[line_idx];
        let end = if line_idx + 1 < self.line_starts.len() {
            self.line_starts[line_idx + 1]
        } else {
            self.text.len()
        };

        Some(start..end)
    }

    /// Returns the absolute byte range for the specified line index, excluding trailing `\n` or `\r\n`.
    ///
    /// Returns `None` if `line_idx >= self.line_count()`.
    #[must_use]
    pub fn line_content_byte_range(&self, line_idx: usize) -> Option<Range<usize>> {
        let mut range = self.line_byte_range(line_idx)?;
        let slice = &self.text[range.clone()];

        if slice.ends_with("\r\n") {
            range.end = range.end.saturating_sub(2);
        } else if slice.ends_with('\n') {
            range.end = range.end.saturating_sub(1);
        }

        Some(range)
    }

    /// Returns a string slice of the specified line including trailing newline delimiters.
    ///
    /// Returns `None` if `line_idx >= self.line_count()`.
    #[must_use]
    pub fn line_raw_str(&self, line_idx: usize) -> Option<&str> {
        let range = self.line_byte_range(line_idx)?;
        Some(&self.text[range])
    }

    /// Returns a string slice of the specified line excluding trailing `\n` or `\r\n`.
    ///
    /// Returns `None` if `line_idx >= self.line_count()`.
    #[must_use]
    pub fn line_str(&self, line_idx: usize) -> Option<&str> {
        let range = self.line_content_byte_range(line_idx)?;
        Some(&self.text[range])
    }

    /// Returns the character count (Unicode scalar values) of the specified line, excluding newlines.
    ///
    /// Returns 0 if `line_idx >= self.line_count()`.
    #[must_use]
    pub fn line_char_count(&self, line_idx: usize) -> usize {
        self.line_str(line_idx).map_or(0, |l| l.chars().count())
    }

    /// Converts a 2D logical position `(line_idx, char_col)` into an absolute byte offset in `self.text`.
    ///
    /// Handles multi-byte UTF-8 character sequences safely and clamps to line boundaries.
    #[must_use]
    pub fn char_col_to_byte_offset(&self, line_idx: usize, char_col: usize) -> usize {
        if self.line_starts.is_empty() {
            return 0;
        }

        let clamped_line = line_idx.min(self.line_starts.len().saturating_sub(1));
        let line_start = self.line_starts[clamped_line];
        let content = self.line_str(clamped_line).unwrap_or("");

        for (current_char_count, (char_byte_offset, _)) in content.char_indices().enumerate() {
            if current_char_count == char_col {
                return line_start + char_byte_offset;
            }
        }

        line_start + content.len()
    }

    /// Returns the absolute byte offset corresponding to the current [`Self::cursor`].
    #[inline]
    #[must_use]
    pub fn cursor_byte_offset(&self) -> usize {
        self.char_col_to_byte_offset(self.cursor.0, self.cursor.1)
    }

    /// Converts an absolute byte offset into a 2D logical cursor `(line_idx, char_col)`.
    ///
    /// Uses binary search on cached `line_starts` in $O(\log L)$ time, followed by
    /// character iteration over the target line.
    #[must_use]
    pub fn byte_offset_to_cursor(&self, byte_offset: usize) -> (usize, usize) {
        if self.line_starts.is_empty() {
            return (0, 0);
        }

        let clamped_byte = byte_offset.min(self.text.len());
        let line_idx = match self.line_starts.binary_search(&clamped_byte) {
            Ok(exact) => exact,
            Err(insertion_idx) => insertion_idx.saturating_sub(1),
        };

        let line_start = self.line_starts[line_idx];
        let line_slice = self.line_str(line_idx).unwrap_or("");
        let rel_byte = clamped_byte
            .saturating_sub(line_start)
            .min(line_slice.len());
        let safe_rel_byte = clamp_to_char_boundary(line_slice, rel_byte);
        let char_col = line_slice[..safe_rel_byte].chars().count();

        (line_idx, char_col)
    }

    /// Clamps the cursor to ensure it stays within valid line and character column bounds.
    pub fn clamp_cursor(&mut self) {
        if self.line_starts.is_empty() {
            self.line_starts.push(0);
        }

        let max_line = self.line_starts.len().saturating_sub(1);
        let line = self.cursor.0.min(max_line);
        let line_chars = self.line_char_count(line);
        let col = self.cursor.1.min(line_chars);

        self.cursor = (line, col);
    }

    /// Sets the cursor to the specified `(line, col)`, automatically clamping to valid bounds.
    pub fn set_cursor(&mut self, line: usize, col: usize) {
        self.cursor = (line, col);
        self.clamp_cursor();
    }

    /// Positions the cursor at the very beginning of the document `(0, 0)`.
    ///
    /// Clears any cached [`Self::preferred_column`].
    pub fn cursor_to_start(&mut self) {
        self.cursor = (0, 0);
        self.preferred_column = None;
    }

    /// Positions the cursor at the very end of the document.
    ///
    /// Clears any cached [`Self::preferred_column`].
    pub fn cursor_to_end(&mut self) {
        let last_line = self.line_starts.len().saturating_sub(1);
        let last_col = self.line_char_count(last_line);
        self.cursor = (last_line, last_col);
        self.preferred_column = None;
    }

    /// Returns the current zero-based line index of the cursor.
    #[inline]
    #[must_use]
    pub fn cursor_line(&self) -> usize {
        self.cursor.0
    }

    /// Returns the current zero-based character column index of the cursor.
    #[inline]
    #[must_use]
    pub fn cursor_col(&self) -> usize {
        self.cursor.1
    }

    /// Replaces the entire buffer text, rebuilds line topology, and resets navigation state.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.rebuild_line_starts();
        self.clamp_cursor();
        self.preferred_column = None;
    }

    /// Clears the text buffer, resetting cursor and scroll offsets to zero.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = (0, 0);
        self.preferred_column = None;
        self.line_starts.clear();
        self.line_starts.push(0);
        self.scroll_y = 0.0;
    }

    /// Inserts a single character at the current cursor position.
    ///
    /// Automatically advances cursor line and column coordinates, rebuilds `line_starts`,
    /// and resets [`Self::preferred_column`].
    pub fn insert_char(&mut self, c: char) {
        let byte_offset = self.cursor_byte_offset();
        self.text.insert(byte_offset, c);

        let new_cursor = if c == '\n' {
            (self.cursor.0 + 1, 0)
        } else {
            (self.cursor.0, self.cursor.1 + 1)
        };

        self.cursor = new_cursor;
        self.rebuild_line_starts();
        self.preferred_column = None;
    }

    /// Inserts a string slice at the current cursor position.
    ///
    /// Handles arbitrary text containing single or multiple newlines (`\n`), updates cursor
    /// coordinates to the end of the inserted slice, rebuilds `line_starts`, and resets
    /// [`Self::preferred_column`].
    pub fn insert_str(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }

        let byte_offset = self.cursor_byte_offset();
        self.text.insert_str(byte_offset, s);

        let newline_count = s.chars().filter(|&c| c == '\n').count();
        let new_cursor = if newline_count == 0 {
            (self.cursor.0, self.cursor.1 + s.chars().count())
        } else {
            let last_line_chars = s
                .rsplit('\n')
                .next()
                .map_or(0, |l| l.trim_end_matches('\r').chars().count());
            (self.cursor.0 + newline_count, last_line_chars)
        };

        self.cursor = new_cursor;
        self.rebuild_line_starts();
        self.preferred_column = None;
    }

    /// Deletes the character immediately preceding the cursor (Backspace behavior).
    ///
    /// If the cursor is at the beginning of a line (`col == 0`), deletes the preceding
    /// newline delimiter and merges the current line with the previous line.
    ///
    /// Returns `true` if a character or newline was deleted, or `false` if already at `(0, 0)`.
    pub fn delete_char_before_cursor(&mut self) -> bool {
        if self.cursor == (0, 0) {
            return false;
        }

        if self.cursor.1 == 0 {
            let prev_line = self.cursor.0.saturating_sub(1);
            let prev_content_end = self.line_content_byte_range(prev_line).map_or(0, |r| r.end);
            let curr_line_start = self.line_starts[self.cursor.0];
            let prev_line_chars = self.line_char_count(prev_line);

            self.text.drain(prev_content_end..curr_line_start);
            self.cursor = (prev_line, prev_line_chars);
            self.rebuild_line_starts();
            self.preferred_column = None;
            true
        } else {
            let prev_char_byte = self.char_col_to_byte_offset(self.cursor.0, self.cursor.1 - 1);
            let curr_char_byte = self.cursor_byte_offset();

            self.text.drain(prev_char_byte..curr_char_byte);
            self.cursor = (self.cursor.0, self.cursor.1 - 1);
            self.rebuild_line_starts();
            self.preferred_column = None;
            true
        }
    }

    /// Deletes the character immediately following the cursor (Delete key behavior).
    ///
    /// If the cursor is at the end of a line, deletes the trailing newline delimiter
    /// and merges the next line into the current line.
    ///
    /// Returns `true` if a character or newline was deleted, or `false` if at the end of the buffer.
    pub fn delete_char_after_cursor(&mut self) -> bool {
        let cur_byte = self.cursor_byte_offset();
        if cur_byte >= self.text.len() {
            return false;
        }

        let line_chars = self.line_char_count(self.cursor.0);
        if self.cursor.1 >= line_chars {
            if self.cursor.0 + 1 >= self.line_starts.len() {
                return false;
            }

            let next_line_start = self.line_starts[self.cursor.0 + 1];
            self.text.drain(cur_byte..next_line_start);
            self.rebuild_line_starts();
            self.preferred_column = None;
            true
        } else {
            let next_char_len = self.text[cur_byte..]
                .chars()
                .next()
                .map_or(0, char::len_utf8);

            if next_char_len == 0 {
                return false;
            }

            self.text.drain(cur_byte..(cur_byte + next_char_len));
            self.rebuild_line_starts();
            self.preferred_column = None;
            true
        }
    }
}