// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Multi-line Text Navigation & Editing Rules (`controls::multiline::navigation`)
//!
//! Subsystem implementing 2D cursor navigation, sticky column memory (`preferred_column`),
//! newline creation, keyboard navigation event routing, and line gutter formatting.

use std::ops::Range;

use iris_core::Color;
use iris_core::event::KeyCode;

use super::state::MultilineTextState;

#[cfg(test)]
mod tests;

/// Default horizontal padding in logical pixels applied to both sides of the line gutter.
pub const DEFAULT_GUTTER_PADDING: f32 = 8.0;

/// Minimum digit reservation in the gutter to prevent horizontal layout jitter for small documents.
pub const MIN_GUTTER_DIGITS: usize = 2;

/// Minimum gutter width in logical pixels to ensure clean spacing and separation from code text.
pub const MIN_GUTTER_WIDTH: f32 = 36.0;

/// Results of dispatching keyboard navigation and editing events in a multi-line text editor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MultilineNavOutcome {
    /// Whether the key event was recognized and consumed by the multi-line editor.
    pub consumed: bool,
    /// Whether the underlying text buffer content was mutated (e.g. newline inserted or char deleted).
    pub text_changed: bool,
    /// Whether the cursor coordinates or sticky column memory changed.
    pub cursor_moved: bool,
}

/// Visual styling and color parameters for rendering the line number gutter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GutterStyle {
    /// Left horizontal padding in logical pixels.
    pub padding_left: f32,
    /// Right horizontal padding in logical pixels.
    pub padding_right: f32,
    /// Text color for standard line numbers.
    pub text_color: Color,
    /// Highlight text color for the line currently containing the caret.
    pub active_line_color: Color,
    /// Background color of the gutter strip.
    pub background_color: Color,
    /// Border divider line color separating the gutter from the editing canvas.
    pub divider_color: Color,
}

impl Default for GutterStyle {
    fn default() -> Self {
        Self {
            padding_left: DEFAULT_GUTTER_PADDING,
            padding_right: DEFAULT_GUTTER_PADDING,
            text_color: Color::from_u8(110, 115, 125, 255),
            active_line_color: Color::from_u8(220, 225, 235, 255),
            background_color: Color::from_u8(24, 25, 30, 255),
            divider_color: Color::from_u8(42, 44, 52, 255),
        }
    }
}

/// Precomputed layout metrics and formatting helpers for the multi-line line number gutter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GutterMetrics {
    /// Total number of lines in the document.
    pub total_lines: usize,
    /// Number of character glyph columns reserved for line digits.
    pub digits: usize,
    /// Total computed width in logical pixels including left and right padding.
    pub width: f32,
    /// Left horizontal padding in logical pixels.
    pub padding_left: f32,
    /// Right horizontal padding in logical pixels.
    pub padding_right: f32,
}

impl GutterMetrics {
    /// Computes gutter layout metrics based on line count, font metrics, and padding.
    #[must_use]
    pub fn compute(total_lines: usize, char_width: f32, padding_x: f32) -> Self {
        let lines_clamped = total_lines.max(1);
        let calculated_digits = (lines_clamped as f64).log10().floor() as usize + 1;
        let digits = calculated_digits.max(MIN_GUTTER_DIGITS);
        let raw_width = digits as f32 * char_width + (2.0 * padding_x);
        let width = raw_width.max(MIN_GUTTER_WIDTH);

        Self {
            total_lines,
            digits,
            width,
            padding_left: padding_x,
            padding_right: padding_x,
        }
    }

    /// Formats a zero-based line index into a 1-based string for centered or aligned gutter rendering.
    #[must_use]
    pub fn format_line_number(&self, line_idx: usize) -> String {
        format!("{}", line_idx + 1)
    }

    /// Computes the range of zero-based line indices visible within the viewport under vertical scrolling.
    ///
    /// Includes extra overscan lines above and below to ensure flicker-free smooth scrolling.
    #[must_use]
    pub fn visible_line_range(
        scroll_y: f32,
        viewport_height: f32,
        line_height: f32,
        total_lines: usize,
        overscan: usize,
    ) -> Range<usize> {
        if line_height <= 0.0 || total_lines == 0 {
            return 0..0;
        }

        let first_line = (scroll_y / line_height).floor().max(0.0) as usize;
        let start = first_line.saturating_sub(overscan);

        let last_line = ((scroll_y + viewport_height) / line_height).ceil().max(0.0) as usize;
        let end = (last_line + overscan).min(total_lines);

        start.min(end)..end
    }
}

impl MultilineTextState {
    /// Moves the cursor vertically up by one line.
    ///
    /// Maintains sticky [`Self::preferred_column`] memory: if the target line is shorter than
    /// the preferred column, the cursor clamps to the end of the line, but the original column
    /// is remembered and restored when subsequently moving onto a longer line.
    ///
    /// If the cursor is on the first line (`line == 0`), moves the cursor to column 0.
    ///
    /// Returns `true` if the cursor moved.
    pub fn move_up(&mut self) -> bool {
        if self.cursor.0 > 0 {
            if self.preferred_column.is_none() {
                self.preferred_column = Some(self.cursor.1);
            }

            let target_line = self.cursor.0 - 1;
            let target_len = self.line_char_count(target_line);
            let target_col = self
                .preferred_column
                .unwrap_or(self.cursor.1)
                .min(target_len);

            self.cursor = (target_line, target_col);
            true
        } else if self.cursor.1 > 0 {
            self.preferred_column = None;
            self.cursor.1 = 0;
            true
        } else {
            false
        }
    }

    /// Moves the cursor vertically down by one line.
    ///
    /// Maintains sticky [`Self::preferred_column`] memory: if the target line is shorter than
    /// the preferred column, the cursor clamps to the end of the line, but the original column
    /// is remembered and restored when subsequently moving onto a longer line.
    ///
    /// If the cursor is on the last line, moves the cursor to the end of that line.
    ///
    /// Returns `true` if the cursor moved.
    pub fn move_down(&mut self) -> bool {
        let line_count = self.line_count();
        if self.cursor.0 + 1 < line_count {
            if self.preferred_column.is_none() {
                self.preferred_column = Some(self.cursor.1);
            }

            let target_line = self.cursor.0 + 1;
            let target_len = self.line_char_count(target_line);
            let target_col = self
                .preferred_column
                .unwrap_or(self.cursor.1)
                .min(target_len);

            self.cursor = (target_line, target_col);
            true
        } else {
            let last_line_len = self.line_char_count(self.cursor.0);
            if self.cursor.1 < last_line_len {
                self.preferred_column = None;
                self.cursor.1 = last_line_len;
                true
            } else {
                false
            }
        }
    }

    /// Moves the cursor horizontally left by one character.
    ///
    /// If at the beginning of a line (`col == 0`), wraps to the end of the preceding line.
    /// Resets [`Self::preferred_column`].
    ///
    /// Returns `true` if the cursor moved.
    pub fn move_left(&mut self) -> bool {
        self.preferred_column = None;

        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
            true
        } else if self.cursor.0 > 0 {
            let prev_line = self.cursor.0 - 1;
            let prev_len = self.line_char_count(prev_line);
            self.cursor = (prev_line, prev_len);
            true
        } else {
            false
        }
    }

    /// Moves the cursor horizontally right by one character.
    ///
    /// If at the end of a line, wraps to the beginning of the succeeding line.
    /// Resets [`Self::preferred_column`].
    ///
    /// Returns `true` if the cursor moved.
    pub fn move_right(&mut self) -> bool {
        self.preferred_column = None;

        let cur_len = self.line_char_count(self.cursor.0);
        if self.cursor.1 < cur_len {
            self.cursor.1 += 1;
            true
        } else if self.cursor.0 + 1 < self.line_count() {
            self.cursor = (self.cursor.0 + 1, 0);
            true
        } else {
            false
        }
    }

    /// Moves the cursor to the beginning of the current line (`col == 0`).
    ///
    /// Resets [`Self::preferred_column`].
    pub fn move_home(&mut self) {
        self.preferred_column = None;
        self.cursor.1 = 0;
    }

    /// Moves the cursor to the end of the current line.
    ///
    /// Resets [`Self::preferred_column`].
    pub fn move_end(&mut self) {
        self.preferred_column = None;
        self.cursor.1 = self.line_char_count(self.cursor.0);
    }

    /// Moves the cursor vertically up by `page_size` lines, preserving [`Self::preferred_column`].
    pub fn page_up(&mut self, page_size: usize) -> bool {
        if self.cursor.0 == 0 && self.cursor.1 == 0 {
            return false;
        }

        if self.preferred_column.is_none() {
            self.preferred_column = Some(self.cursor.1);
        }

        let target_line = self.cursor.0.saturating_sub(page_size);
        let target_len = self.line_char_count(target_line);
        let target_col = self
            .preferred_column
            .unwrap_or(self.cursor.1)
            .min(target_len);

        self.cursor = (target_line, target_col);
        true
    }

    /// Moves the cursor vertically down by `page_size` lines, preserving [`Self::preferred_column`].
    pub fn page_down(&mut self, page_size: usize) -> bool {
        let max_line = self.line_count().saturating_sub(1);
        let last_line_len = self.line_char_count(max_line);

        if self.cursor.0 == max_line && self.cursor.1 >= last_line_len {
            return false;
        }

        if self.preferred_column.is_none() {
            self.preferred_column = Some(self.cursor.1);
        }

        let target_line = (self.cursor.0 + page_size).min(max_line);
        let target_len = self.line_char_count(target_line);
        let target_col = self
            .preferred_column
            .unwrap_or(self.cursor.1)
            .min(target_len);

        self.cursor = (target_line, target_col);
        true
    }

    /// Inserts a newline delimiter at the current cursor position (Enter key action).
    ///
    /// Splits the current line, creates a new line below, positions the cursor at `(line + 1, 0)`,
    /// and resets [`Self::preferred_column`].
    pub fn insert_newline(&mut self) {
        self.insert_char('\n');
    }

    /// Synchronizes selection anchor before a navigation movement.
    fn sync_selection_anchor(&mut self, shift: bool) {
        if shift && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !shift {
            self.selection_anchor = None;
            self.clear_selection();
        }
    }

    /// Updates active selection range according to anchor and current cursor.
    fn finalize_shift_selection(&mut self, shift: bool) {
        if !shift {
            return;
        }
        if let Some(anchor) = self.selection_anchor {
            if anchor != self.cursor {
                self.set_selection(anchor, self.cursor);
            } else {
                self.clear_selection();
            }
        }
    }

    /// Dispatches a virtual keyboard navigation or editing key event.
    ///
    /// Automatically routes arrow keys, Home, End, Enter, Backspace, Delete,
    /// and clipboard/history shortcuts (`Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V`, `Ctrl+Z`, `Ctrl+Y`).
    pub fn handle_key(&mut self, key: KeyCode, ctrl: bool, shift: bool) -> MultilineNavOutcome {
        let mut outcome = MultilineNavOutcome::default();

        if ctrl {
            if (shift && matches!(key, KeyCode::Character('z') | KeyCode::Character('Z')))
                || matches!(key, KeyCode::Character('y') | KeyCode::Character('Y'))
            {
                outcome.consumed = true;
                if self.redo() {
                    outcome.text_changed = true;
                    outcome.cursor_moved = true;
                }
                return outcome;
            }

            match key {
                KeyCode::Character('a') | KeyCode::Character('A') => {
                    self.select_all();
                    outcome.consumed = true;
                    outcome.cursor_moved = true;
                    return outcome;
                }
                KeyCode::Character('c') | KeyCode::Character('C') => {
                    outcome.consumed = true;
                    let _ = self.copy_selection_to_clipboard();
                    return outcome;
                }
                KeyCode::Character('x') | KeyCode::Character('X') => {
                    outcome.consumed = true;
                    if self.cut_selection_to_clipboard() {
                        outcome.text_changed = true;
                        outcome.cursor_moved = true;
                    }
                    return outcome;
                }
                KeyCode::Character('v') | KeyCode::Character('V') => {
                    outcome.consumed = true;
                    if self.paste_from_clipboard() {
                        outcome.text_changed = true;
                        outcome.cursor_moved = true;
                    }
                    return outcome;
                }
                KeyCode::Character('z') | KeyCode::Character('Z') => {
                    outcome.consumed = true;
                    if self.undo() {
                        outcome.text_changed = true;
                        outcome.cursor_moved = true;
                    }
                    return outcome;
                }
                _ => {}
            }
        }

        let handle_move = |state: &mut Self, move_fn: fn(&mut Self) -> bool| -> bool {
            state.sync_selection_anchor(shift);
            let moved = move_fn(state);
            state.finalize_shift_selection(shift);
            moved
        };

        match key {
            KeyCode::ArrowUp => {
                outcome.consumed = true;
                outcome.cursor_moved = handle_move(self, Self::move_up);
            }
            KeyCode::ArrowDown => {
                outcome.consumed = true;
                outcome.cursor_moved = handle_move(self, Self::move_down);
            }
            KeyCode::ArrowLeft => {
                outcome.consumed = true;
                outcome.cursor_moved = handle_move(self, Self::move_left);
            }
            KeyCode::ArrowRight => {
                outcome.consumed = true;
                outcome.cursor_moved = handle_move(self, Self::move_right);
            }
            KeyCode::Home => {
                outcome.consumed = true;
                self.sync_selection_anchor(shift);
                if ctrl {
                    self.cursor_to_start();
                } else {
                    self.move_home();
                }
                self.finalize_shift_selection(shift);
                outcome.cursor_moved = true;
            }
            KeyCode::End => {
                outcome.consumed = true;
                self.sync_selection_anchor(shift);
                if ctrl {
                    self.cursor_to_end();
                } else {
                    self.move_end();
                }
                self.finalize_shift_selection(shift);
                outcome.cursor_moved = true;
            }
            KeyCode::Enter => {
                self.delete_selection();
                outcome.consumed = true;
                self.insert_newline();
                outcome.text_changed = true;
                outcome.cursor_moved = true;
            }
            KeyCode::Backspace => {
                outcome.consumed = true;
                if self.delete_char_before_cursor() {
                    outcome.text_changed = true;
                    outcome.cursor_moved = true;
                }
            }
            KeyCode::Delete => {
                outcome.consumed = true;
                if self.delete_char_after_cursor() {
                    outcome.text_changed = true;
                }
            }
            _ => {}
        }

        outcome
    }
}