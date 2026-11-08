// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Text Input Keyboard & Pointer Interaction Subsystem
//!
//! Provides cursor indexing, UTF-8 boundary navigation, text selection management,
//! and keyboard event dispatching for declarative text input fields.

use iris_core::dispatch::undo::{TextSnapshot, TextUndoStack, UndoActionKind};
use iris_core::event::KeyCode;
use iris_text::measure_text_advance;

/// Calculates the target cursor character byte index within `text` closest to `click_rel_x`.
///
/// Uses precomputed glyph clusters and $O(\log N)$ binary search to resolve
/// the target caret byte offset accurately without redundant string slicing.
///
/// # Arguments
/// * `text` - The underlying string content being inspected.
/// * `click_rel_x` - Horizontal click position relative to the left edge of the text origin.
/// * `font_size` - Active font size in physical pixels for glyph advance measurements.
///
/// # Returns
/// Valid UTF-8 byte offset within `text` where the caret should be positioned.
pub fn find_cursor_idx_at_x(text: &str, click_rel_x: f32, font_size: f32) -> usize {
    if click_rel_x <= 0.0 || text.is_empty() {
        return 0;
    }
    let mut clusters: Vec<(usize, f32)> = Vec::with_capacity(text.len().min(64));
    for (idx, ch) in text.char_indices() {
        let prefix_len = idx + ch.len_utf8();
        let cum_x = measure_text_advance(&text[..prefix_len], font_size);
        clusters.push((idx, cum_x));
    }
    iris_text::hit_test_from_cached_clusters(&clusters, text.len(), click_rel_x)
}

/// Finds the nearest preceding valid UTF-8 character boundary before `cursor_idx`.
///
/// # Arguments
/// * `text` - Source string buffer.
/// * `cursor_idx` - Current byte offset.
///
/// # Returns
/// Preceding character boundary byte offset, or `0` if at the start.
#[inline]
pub fn prev_char_boundary(text: &str, cursor_idx: usize) -> usize {
    if cursor_idx == 0 {
        return 0;
    }
    let mut prev = cursor_idx - 1;
    while prev > 0 && !text.is_char_boundary(prev) {
        prev -= 1;
    }
    prev
}

/// Finds the nearest succeeding valid UTF-8 character boundary after `cursor_idx`.
///
/// # Arguments
/// * `text` - Source string buffer.
/// * `cursor_idx` - Current byte offset.
///
/// # Returns
/// Next character boundary byte offset, or `text.len()` if at the end.
#[inline]
pub fn next_char_boundary(text: &str, cursor_idx: usize) -> usize {
    if cursor_idx >= text.len() {
        return text.len();
    }
    let mut next = cursor_idx + 1;
    while next < text.len() && !text.is_char_boundary(next) {
        next += 1;
    }
    next
}

/// Results of processing keyboard navigation and text mutation events for a text input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextInputEventResult {
    /// Whether the interaction was consumed by this text input.
    pub consumed: bool,
    /// Whether the underlying text buffer content was altered during this frame.
    pub changed: bool,
    /// Whether the user submitted the input (e.g. by pressing `KeyCode::Enter`).
    pub submitted: bool,
    /// Whether keyboard focus should be cleared (e.g. via `Enter` or `Escape`).
    pub should_clear_focus: bool,
}

/// Rules governing which characters are admitted during keyboard input and clipboard paste operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextInputFilter {
    /// Accepts all printable non-control characters and spaces (default for text boxes).
    #[default]
    All,
    /// Strictly limits input to numeric digits and basic arithmetic expression operators (`0-9`, `.`, `-`, `+`, `*`, `/`, `(`, `)`, space).
    NumericExpression,
}

impl TextInputFilter {
    /// Evaluates whether a given character passes the active filter rule.
    #[inline]
    #[must_use]
    pub fn is_allowed(self, c: char) -> bool {
        match self {
            Self::All => !c.is_control() || c == ' ',
            Self::NumericExpression => {
                c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | '*' | '/' | '(' | ')' | ' ')
            }
        }
    }
}

/// Contextual payload describing incoming keyboard and text interaction parameters.
///
/// Groups event states into a single descriptor to avoid parameter bloat.
#[derive(Clone, Copy, Debug)]
pub struct TextInputKeyContext<'a> {
    /// Whether a keyboard key was pressed down this frame.
    pub key_pressed: bool,
    /// Physical key code received from the event loop, if any.
    pub key: Option<KeyCode>,
    /// Optional textual string representation produced by the key event.
    pub incoming_text: Option<&'a str>,
    /// Whether the Control (or Command on macOS) modifier key was held.
    pub ctrl_held: bool,
    /// Whether the Shift modifier key was held.
    pub shift_held: bool,
    /// Input character filter mode applied to incoming typing and pasted clipboard strings.
    pub filter: TextInputFilter,
    /// Whether clipboard shortcuts (`Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V`) are enabled.
    pub clipboard_shortcuts_enabled: bool,
    /// Whether autonomous undo/redo shortcuts (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`) are enabled.
    pub undo_shortcuts_enabled: bool,
}

impl<'a> Default for TextInputKeyContext<'a> {
    #[inline]
    fn default() -> Self {
        Self {
            key_pressed: false,
            key: None,
            incoming_text: None,
            ctrl_held: false,
            shift_held: false,
            filter: TextInputFilter::All,
            clipboard_shortcuts_enabled: true,
            undo_shortcuts_enabled: true,
        }
    }
}

impl<'a> TextInputKeyContext<'a> {
    /// Creates a new keyboard event context with explicit parameters, default filter, and shortcuts enabled.
    #[inline]
    pub const fn new(
        key_pressed: bool,
        key: Option<KeyCode>,
        incoming_text: Option<&'a str>,
        ctrl_held: bool,
        shift_held: bool,
    ) -> Self {
        Self {
            key_pressed,
            key,
            incoming_text,
            ctrl_held,
            shift_held,
            filter: TextInputFilter::All,
            clipboard_shortcuts_enabled: true,
            undo_shortcuts_enabled: true,
        }
    }

    /// Sets the character filter applied to typed input and clipboard paste operations.
    #[inline]
    #[must_use]
    pub const fn with_filter(mut self, filter: TextInputFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Sets whether clipboard keyboard shortcuts (`Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V`) are enabled.
    #[inline]
    #[must_use]
    pub const fn with_clipboard_shortcuts(mut self, enabled: bool) -> Self {
        self.clipboard_shortcuts_enabled = enabled;
        self
    }

    /// Sets whether undo and redo keyboard shortcuts (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`) are enabled.
    #[inline]
    #[must_use]
    pub const fn with_undo_shortcuts(mut self, enabled: bool) -> Self {
        self.undo_shortcuts_enabled = enabled;
        self
    }

    /// Sets whether both clipboard and undo/redo keyboard shortcuts are enabled.
    #[inline]
    #[must_use]
    pub const fn with_shortcuts(mut self, enabled: bool) -> Self {
        self.clipboard_shortcuts_enabled = enabled;
        self.undo_shortcuts_enabled = enabled;
        self
    }
}

/// Processes incoming control keys, navigation arrows, backspace/delete, character input,
/// autonomous undo/redo shortcuts (Ctrl+Z, Ctrl+Y, Ctrl+Shift+Z), and clipboard integration
/// (Ctrl+A, Ctrl+C, Ctrl+X, Ctrl+V) with word-boundary coalescing.
///
/// Manages selection overwrites, cursor movements, and UTF-8 string mutations in-place.
///
/// # Arguments
/// * `text` - Mutable reference to the underlying text buffer.
/// * `cursor_byte_idx` - Mutable reference to the current caret byte offset.
/// * `selection` - Mutable reference to the optional active selection range `(start, end)` where `start < end`.
/// * `ctx` - Contextual keyboard input parameters (`key_pressed`, `key`, `incoming_text`, `ctrl_held`, `shift_held`).
/// * `undo_stack` - Mutable reference to the active session's undo/redo history stack.
///
/// # Returns
/// A [`TextInputEventResult`] indicating consumption, changes, submissions, and focus clear requests.
pub fn process_text_input_keys(
    text: &mut String,
    cursor_byte_idx: &mut usize,
    selection: &mut Option<(usize, usize)>,
    ctx: TextInputKeyContext<'_>,
    undo_stack: &mut TextUndoStack,
) -> TextInputEventResult {
    let mut result = TextInputEventResult::default();

    *cursor_byte_idx = (*cursor_byte_idx).min(text.len());
    while *cursor_byte_idx > 0 && !text.is_char_boundary(*cursor_byte_idx) {
        *cursor_byte_idx -= 1;
    }

    if let Some((s, e)) = selection {
        *s = (*s).min(text.len());
        while *s > 0 && !text.is_char_boundary(*s) {
            *s -= 1;
        }
        *e = (*e).min(text.len());
        while *e > 0 && !text.is_char_boundary(*e) {
            *e -= 1;
        }
        if *s >= *e {
            *selection = None;
        }
    }

    // 1. Autonomous Desktop Keyboard Shortcuts (Ctrl+Z, Ctrl+Y, Ctrl+Shift+Z, Ctrl+A, Ctrl+C, Ctrl+X, Ctrl+V)
    if ctx.key_pressed && ctx.ctrl_held {
        let ch = match ctx.key {
            Some(KeyCode::Character(c)) => Some(c.to_ascii_lowercase()),
            _ => ctx
                .incoming_text
                .and_then(|s| s.chars().next().map(|c| c.to_ascii_lowercase())),
        };

        // Autonomous Undo: Ctrl+Z (without Shift)
        if ctx.undo_shortcuts_enabled && !ctx.shift_held && ch == Some('z') {
            result.consumed = true;
            if undo_stack.undo_in_place(text, cursor_byte_idx, selection) {
                result.changed = true;
            }
            return result;
        }

        // Autonomous Redo: Ctrl+Y (Windows/Linux) or Ctrl+Shift+Z (macOS / Modern IDE)
        if ctx.undo_shortcuts_enabled
            && ((!ctx.shift_held && ch == Some('y')) || (ctx.shift_held && ch == Some('z')))
        {
            result.consumed = true;
            if undo_stack.redo_in_place(text, cursor_byte_idx, selection) {
                result.changed = true;
            }
            return result;
        }

        if ctx.clipboard_shortcuts_enabled {
            match ch {
                Some('a') => {
                    result.consumed = true;
                    if !text.is_empty() {
                        *selection = Some((0, text.len()));
                        *cursor_byte_idx = text.len();
                        result.changed = true;
                    }
                    return result;
                }
                Some('c') => {
                    result.consumed = true;
                    if !text.is_empty() {
                        let copied = if let Some((s, e)) = *selection {
                            &text[s..e]
                        } else {
                            text.as_str()
                        };
                        crate::input::copy_to_clipboard(copied);
                    }
                    return result;
                }
                Some('x') => {
                    result.consumed = true;
                    if !text.is_empty() {
                        undo_stack.record_edit(
                            TextSnapshot::new(text.clone(), *cursor_byte_idx, *selection),
                            UndoActionKind::Atomic,
                            None,
                        );
                        if let Some((s, e)) = selection.take() {
                            crate::input::copy_to_clipboard(&text[s..e]);
                            text.drain(s..e);
                            *cursor_byte_idx = s;
                            result.changed = true;
                        } else {
                            crate::input::copy_to_clipboard(text);
                            text.clear();
                            *cursor_byte_idx = 0;
                            result.changed = true;
                        }
                    }
                    return result;
                }
                Some('v') => {
                    result.consumed = true;
                    if let Some(pasted) = crate::input::get_clipboard_text() {
                        let filtered: String = pasted
                            .chars()
                            .filter(|&c| ctx.filter.is_allowed(c))
                            .collect();
                        if !filtered.is_empty() {
                            undo_stack.record_edit(
                                TextSnapshot::new(text.clone(), *cursor_byte_idx, *selection),
                                UndoActionKind::Atomic,
                                None,
                            );
                            if let Some((s, e)) = selection.take() {
                                text.drain(s..e);
                                text.insert_str(s, &filtered);
                                *cursor_byte_idx = s + filtered.len();
                                result.changed = true;
                            } else {
                                let idx = (*cursor_byte_idx).min(text.len());
                                text.insert_str(idx, &filtered);
                                *cursor_byte_idx += filtered.len();
                                result.changed = true;
                            }
                        }
                    }
                    return result;
                }
                _ => {}
            }
        }
    }

    let is_control_key = ctx.key_pressed
        && ctx.key.is_some_and(|k| {
            matches!(
                k,
                KeyCode::ArrowLeft
                    | KeyCode::ArrowRight
                    | KeyCode::Home
                    | KeyCode::End
                    | KeyCode::Backspace
                    | KeyCode::Delete
                    | KeyCode::Enter
                    | KeyCode::Escape
            )
        });

    if is_control_key && let Some(k) = ctx.key {
        result.consumed = true;
        match k {
            KeyCode::ArrowLeft => {
                if let Some((s, _)) = selection.take() {
                    *cursor_byte_idx = s;
                } else {
                    *cursor_byte_idx = prev_char_boundary(text, *cursor_byte_idx);
                }
            }
            KeyCode::ArrowRight => {
                if let Some((_, e)) = selection.take() {
                    *cursor_byte_idx = e;
                } else {
                    *cursor_byte_idx = next_char_boundary(text, *cursor_byte_idx);
                }
            }
            KeyCode::Home => {
                *selection = None;
                *cursor_byte_idx = 0;
            }
            KeyCode::End => {
                *selection = None;
                *cursor_byte_idx = text.len();
            }
            KeyCode::Backspace => {
                if let Some((s, e)) = selection.take() {
                    undo_stack.record_edit(
                        TextSnapshot::new(text.clone(), *cursor_byte_idx, Some((s, e))),
                        UndoActionKind::Atomic,
                        None,
                    );
                    text.drain(s..e);
                    *cursor_byte_idx = s;
                    result.changed = true;
                } else if *cursor_byte_idx > 0 {
                    let prev = prev_char_boundary(text, *cursor_byte_idx);
                    let deleted_char = text[prev..*cursor_byte_idx].chars().next();
                    undo_stack.record_edit(
                        TextSnapshot::new(text.clone(), *cursor_byte_idx, None),
                        UndoActionKind::Delete,
                        deleted_char,
                    );
                    text.drain(prev..*cursor_byte_idx);
                    *cursor_byte_idx = prev;
                    result.changed = true;
                }
            }
            KeyCode::Delete => {
                if let Some((s, e)) = selection.take() {
                    undo_stack.record_edit(
                        TextSnapshot::new(text.clone(), *cursor_byte_idx, Some((s, e))),
                        UndoActionKind::Atomic,
                        None,
                    );
                    text.drain(s..e);
                    *cursor_byte_idx = s;
                    result.changed = true;
                } else if *cursor_byte_idx < text.len() {
                    let next = next_char_boundary(text, *cursor_byte_idx);
                    let deleted_char = text[*cursor_byte_idx..next].chars().next();
                    undo_stack.record_edit(
                        TextSnapshot::new(text.clone(), *cursor_byte_idx, None),
                        UndoActionKind::Delete,
                        deleted_char,
                    );
                    text.drain(*cursor_byte_idx..next);
                    result.changed = true;
                }
            }
            KeyCode::Enter => {
                result.submitted = true;
                result.should_clear_focus = true;
            }
            KeyCode::Escape => {
                result.should_clear_focus = true;
            }
            _ => {}
        }
    } else if ctx.key_pressed
        && !ctx.ctrl_held
        && let Some(incoming) = ctx.incoming_text
    {
        let filtered: String = incoming
            .chars()
            .filter(|&c| ctx.filter.is_allowed(c))
            .collect();
        if !filtered.is_empty() {
            result.consumed = true;
            let boundary_char = filtered.chars().next();
            if let Some((s, e)) = selection.take() {
                undo_stack.record_edit(
                    TextSnapshot::new(text.clone(), *cursor_byte_idx, Some((s, e))),
                    UndoActionKind::Atomic,
                    boundary_char,
                );
                text.drain(s..e);
                text.insert_str(s, &filtered);
                *cursor_byte_idx = s + filtered.len();
                result.changed = true;
            } else {
                undo_stack.record_edit(
                    TextSnapshot::new(text.clone(), *cursor_byte_idx, None),
                    UndoActionKind::Insert,
                    boundary_char,
                );
                let idx = (*cursor_byte_idx).min(text.len());
                text.insert_str(idx, &filtered);
                *cursor_byte_idx += filtered.len();
                result.changed = true;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcuts_disabled_in_context() {
        let mut text = "hello".to_string();
        let mut cur = 5;
        let mut sel = None;
        let mut undo = TextUndoStack::new();

        undo.checkpoint(TextSnapshot::new("hello", 5, None));

        // When undo shortcuts are disabled, Ctrl+Z is ignored
        let ctx =
            TextInputKeyContext::new(true, Some(KeyCode::Character('z')), Some("z"), true, false)
                .with_undo_shortcuts(false);
        let res = process_text_input_keys(&mut text, &mut cur, &mut sel, ctx, &mut undo);
        assert!(!res.consumed);
        assert!(!res.changed);

        // When clipboard shortcuts are disabled, Ctrl+A is ignored
        let ctx =
            TextInputKeyContext::new(true, Some(KeyCode::Character('a')), Some("a"), true, false)
                .with_clipboard_shortcuts(false);
        let res = process_text_input_keys(&mut text, &mut cur, &mut sel, ctx, &mut undo);
        assert!(!res.consumed);
        assert_eq!(sel, None);
    }
}