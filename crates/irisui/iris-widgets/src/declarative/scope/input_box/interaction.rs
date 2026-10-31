// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Text Input Keyboard & Pointer Interaction Subsystem
//!
//! Provides cursor indexing, UTF-8 boundary navigation, text selection management,
//! and keyboard event dispatching for declarative text input fields.

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
    let mut cum_x = 0.0;
    for (idx, ch) in text.char_indices() {
        cum_x += measure_text_advance(ch.encode_utf8(&mut [0; 4]), font_size);
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
    /// Whether the underlying text buffer content was altered during this frame.
    pub changed: bool,
    /// Whether the user submitted the input (e.g. by pressing `KeyCode::Enter`).
    pub submitted: bool,
    /// Whether keyboard focus should be cleared (e.g. via `Enter` or `Escape`).
    pub should_clear_focus: bool,
}

/// Processes incoming control keys, navigation arrows, backspace/delete, and character input,
/// including native clipboard integration (Ctrl+A, Ctrl+C, Ctrl+X, Ctrl+V).
///
/// Manages selection overwrites, cursor movements, and UTF-8 string mutations in-place.
///
/// # Arguments
/// * `text` - Mutable reference to the underlying text buffer.
/// * `cursor_byte_idx` - Mutable reference to the current caret byte offset.
/// * `is_all_selected` - Mutable reference to the all-selected highlight state.
/// * `key_pressed` - Whether a physical keyboard key was pressed this frame.
/// * `key` - Optional physical key code received from the event loop.
/// * `incoming_text` - Optional textual characters typed during the current frame.
/// * `ctrl_held` - Whether the Control modifier was held during the key event.
///
/// # Returns
/// A [`TextInputEventResult`] indicating changes, submissions, and focus clear requests.
pub fn process_text_input_keys(
    text: &mut String,
    cursor_byte_idx: &mut usize,
    is_all_selected: &mut bool,
    key_pressed: bool,
    key: Option<KeyCode>,
    incoming_text: Option<&str>,
    ctrl_held: bool,
) -> TextInputEventResult {
    let mut result = TextInputEventResult::default();

    *cursor_byte_idx = (*cursor_byte_idx).min(text.len());
    while *cursor_byte_idx > 0 && !text.is_char_boundary(*cursor_byte_idx) {
        *cursor_byte_idx -= 1;
    }

    // 1. Autonomous Desktop Clipboard Shortcuts (Ctrl+A, Ctrl+C, Ctrl+X, Ctrl+V)
    if key_pressed && ctrl_held {
        let ch = match key {
            Some(KeyCode::Character(c)) => Some(c.to_ascii_lowercase()),
            _ => incoming_text.and_then(|s| s.chars().next().map(|c| c.to_ascii_lowercase())),
        };

        match ch {
            Some('a') => {
                if !text.is_empty() {
                    *is_all_selected = true;
                    *cursor_byte_idx = text.len();
                    result.changed = true;
                }
                return result;
            }
            Some('c') => {
                if !text.is_empty() {
                    crate::input::copy_to_clipboard(text);
                }
                return result;
            }
            Some('x') => {
                if !text.is_empty() {
                    crate::input::copy_to_clipboard(text);
                    text.clear();
                    *cursor_byte_idx = 0;
                    *is_all_selected = false;
                    result.changed = true;
                }
                return result;
            }
            Some('v') => {
                if let Some(pasted) = crate::input::get_clipboard_text() {
                    let filtered: String = pasted
                        .chars()
                        .filter(|c| !c.is_control() || *c == ' ')
                        .collect();
                    if !filtered.is_empty() {
                        if *is_all_selected {
                            text.clear();
                            text.push_str(&filtered);
                            *cursor_byte_idx = text.len();
                            *is_all_selected = false;
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

    let is_control_key = key_pressed
        && key.is_some_and(|k| {
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

    if is_control_key && let Some(k) = key {
        match k {
            KeyCode::ArrowLeft => {
                if *is_all_selected {
                    *is_all_selected = false;
                    *cursor_byte_idx = 0;
                } else {
                    *cursor_byte_idx = prev_char_boundary(text, *cursor_byte_idx);
                }
            }
            KeyCode::ArrowRight => {
                if *is_all_selected {
                    *is_all_selected = false;
                    *cursor_byte_idx = text.len();
                } else {
                    *cursor_byte_idx = next_char_boundary(text, *cursor_byte_idx);
                }
            }
            KeyCode::Home => {
                *is_all_selected = false;
                *cursor_byte_idx = 0;
            }
            KeyCode::End => {
                *is_all_selected = false;
                *cursor_byte_idx = text.len();
            }
            KeyCode::Backspace => {
                if *is_all_selected {
                    text.clear();
                    *cursor_byte_idx = 0;
                    *is_all_selected = false;
                    result.changed = true;
                } else if *cursor_byte_idx > 0 {
                    let prev = prev_char_boundary(text, *cursor_byte_idx);
                    text.drain(prev..*cursor_byte_idx);
                    *cursor_byte_idx = prev;
                    result.changed = true;
                }
            }
            KeyCode::Delete => {
                if *is_all_selected {
                    text.clear();
                    *cursor_byte_idx = 0;
                    *is_all_selected = false;
                    result.changed = true;
                } else if *cursor_byte_idx < text.len() {
                    let next = next_char_boundary(text, *cursor_byte_idx);
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
    } else if key_pressed && let Some(incoming) = incoming_text {
        let filtered: String = incoming.chars().filter(|c| !c.is_control()).collect();
        if !filtered.is_empty() {
            if *is_all_selected {
                text.clear();
                text.push_str(&filtered);
                *cursor_byte_idx = text.len();
                *is_all_selected = false;
                result.changed = true;
            } else {
                let idx = (*cursor_byte_idx).min(text.len());
                text.insert_str(idx, &filtered);
                *cursor_byte_idx += filtered.len();
                result.changed = true;
            }
        }
    }

    result
}