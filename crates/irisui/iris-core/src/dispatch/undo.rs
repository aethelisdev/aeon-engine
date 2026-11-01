// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Text Undo and Redo History Stack (`iris-core::dispatch::undo`)
//!
//! Provides bounded, zero-leak undo and redo history tracking with smart keystroke coalescing,
//! word-boundary segmentation, and selection preservation for interactive text editing sessions.

use std::collections::VecDeque;
use std::time::Instant;

/// Default capacity limit for undo snapshots to prevent unbounded heap memory growth.
pub const DEFAULT_MAX_UNDO_DEPTH: usize = 64;

/// Maximum idle duration between keystrokes before creating a new undo checkpoint (in milliseconds).
pub const TEXT_UNDO_COALESCE_TIMEOUT_MS: u128 = 400;

/// Determines whether a character constitutes a word boundary for typing coalescing.
///
/// Whitespace, punctuation, and bracket characters demarcate logical editing units,
/// causing the coalescing window to finalize so that pressing Undo restores the previous word.
#[inline]
pub fn is_word_boundary(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '.' | ','
                | ';'
                | ':'
                | '!'
                | '?'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '/'
                | '\\'
                | '"'
                | '\''
                | '<'
                | '>'
                | '='
                | '+'
                | '-'
                | '*'
                | '%'
                | '&'
                | '|'
                | '^'
        )
}

/// Classification of text editing mutations used for intelligent undo/redo coalescing.
///
/// Consecutive keystrokes of the same action kind occurring within the coalescing time
/// window are grouped into a single logical undo checkpoint. A transition between kinds
/// immediately seals the active sequence and establishes a new snapshot.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UndoActionKind {
    /// Incremental character typing.
    Insert,
    /// Backward or forward character deletion (Backspace or Delete).
    Delete,
    /// Discrete atomic mutation (Clipboard Paste, Cut, Selection Replacement, or Clear).
    Atomic,
}

/// Immutable snapshot representing the state of an interactive text editor at a discrete point in time.
///
/// Captures the complete text content, caret byte offset, and active selection range to allow
/// exact historical restoration during undo and redo operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextSnapshot {
    /// Full text buffer at the time of the snapshot.
    pub text: String,
    /// Byte index of the editing cursor within the buffer.
    pub cursor_byte_idx: usize,
    /// Active selection range `(start_byte_idx, end_byte_idx)`, if any was present.
    pub selection: Option<(usize, usize)>,
}

impl TextSnapshot {
    /// Constructs a new text snapshot with the specified buffer, cursor index, and selection.
    ///
    /// # Parameters
    /// - `text`: String buffer content to capture.
    /// - `cursor_byte_idx`: Caret byte index within the buffer.
    /// - `selection`: Optional selection span `(start, end)` where `start <= end`.
    pub fn new(
        text: impl Into<String>,
        cursor_byte_idx: usize,
        selection: Option<(usize, usize)>,
    ) -> Self {
        Self {
            text: text.into(),
            cursor_byte_idx,
            selection,
        }
    }
}

/// Bounded undo and redo history coordinator with smart keystroke coalescing.
///
/// Manages double-ended queues of historical [`TextSnapshot`] entries, enforcing a strict
/// maximum capacity (`max_depth`) to prevent memory leaks during long-running application sessions.
/// Keystrokes are automatically grouped based on typing cadence (~400ms idle threshold) and
/// word boundaries, while discrete actions (pasting, selection clearing) form atomic checkpoints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextUndoStack {
    /// FIFO/LIFO double-ended queue storing historical snapshots for undo operations.
    undo_stack: VecDeque<TextSnapshot>,
    /// FIFO/LIFO double-ended queue storing reverted snapshots for redo operations.
    redo_stack: VecDeque<TextSnapshot>,
    /// Timestamp recording when the last editing mutation occurred.
    last_action_time: Option<Instant>,
    /// Classification of the last recorded editing mutation.
    last_action_kind: Option<UndoActionKind>,
    /// Indicates whether the previous mutation constituted a word boundary character.
    last_action_was_boundary: bool,
    /// Maximum capacity of the undo history queue.
    max_depth: usize,
}

impl Default for TextUndoStack {
    fn default() -> Self {
        Self::new()
    }
}

impl TextUndoStack {
    /// Creates a new, empty undo history stack with the default maximum capacity ([`DEFAULT_MAX_UNDO_DEPTH`]).
    pub fn new() -> Self {
        Self::with_max_depth(DEFAULT_MAX_UNDO_DEPTH)
    }

    /// Creates a new, empty undo history stack configured with a custom maximum capacity.
    ///
    /// # Parameters
    /// - `max_depth`: Maximum number of undo snapshots retained before discarding oldest entries.
    pub fn with_max_depth(max_depth: usize) -> Self {
        let effective_depth = max_depth.max(1);
        Self {
            undo_stack: VecDeque::with_capacity(effective_depth.min(128)),
            redo_stack: VecDeque::new(),
            last_action_time: None,
            last_action_kind: None,
            last_action_was_boundary: false,
            max_depth: effective_depth,
        }
    }

    /// Returns the maximum undo history capacity limit.
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Updates the maximum undo history capacity limit, discarding oldest entries if needed.
    pub fn set_max_depth(&mut self, max_depth: usize) {
        self.max_depth = max_depth.max(1);
        while self.undo_stack.len() > self.max_depth {
            self.undo_stack.pop_front();
        }
        while self.redo_stack.len() > self.max_depth {
            self.redo_stack.pop_front();
        }
    }

    /// Returns `true` if one or more snapshots are available to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if one or more snapshots are available to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Returns the count of historical snapshots currently present on the undo stack.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Returns the count of reverted snapshots currently present on the redo stack.
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clears both undo and redo stacks and resets active coalescing state.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.last_action_time = None;
        self.last_action_kind = None;
        self.last_action_was_boundary = false;
    }

    /// Returns a reference to the top snapshot on the undo stack without popping it.
    pub fn peek_undo(&self) -> Option<&TextSnapshot> {
        self.undo_stack.back()
    }

    /// Returns a reference to the top snapshot on the redo stack without popping it.
    pub fn peek_redo(&self) -> Option<&TextSnapshot> {
        self.redo_stack.back()
    }

    /// Records an editing mutation into the history, applying intelligent coalescing logic.
    ///
    /// If the incoming edit matches the previous action kind, occurs within the idle timeout
    /// ([`TEXT_UNDO_COALESCE_TIMEOUT_MS`]), and does not cross a word boundary, it coalesces
    /// with the in-progress sequence without pushing a duplicate snapshot. Otherwise, the provided
    /// pre-edit snapshot is sealed onto the undo stack and the redo stack is cleared.
    ///
    /// # Parameters
    /// - `pre_edit_snapshot`: State of the text buffer before the current mutation occurred.
    /// - `action_kind`: Classification of the editing mutation being applied.
    /// - `boundary_char`: Optional character being typed or deleted that may trigger a word boundary.
    pub fn record_edit(
        &mut self,
        pre_edit_snapshot: TextSnapshot,
        action_kind: UndoActionKind,
        boundary_char: Option<char>,
    ) {
        let now = Instant::now();
        let is_same_kind = self.last_action_kind == Some(action_kind);
        let is_timed_out = self.last_action_time.is_none_or(|last| {
            now.duration_since(last).as_millis() >= TEXT_UNDO_COALESCE_TIMEOUT_MS
        });
        let is_boundary = boundary_char.is_some_and(is_word_boundary);

        let should_push_checkpoint = action_kind == UndoActionKind::Atomic
            || !is_same_kind
            || is_timed_out
            || is_boundary
            || self.last_action_was_boundary;

        if should_push_checkpoint {
            self.push_undo_entry(pre_edit_snapshot);
        }

        self.redo_stack.clear();
        self.last_action_time = Some(now);
        self.last_action_kind = Some(action_kind);
        self.last_action_was_boundary = is_boundary;
    }

    /// Forces a discrete checkpoint snapshot onto the undo stack without coalescing.
    ///
    /// Clears the redo stack and resets coalescing timers, guaranteeing that the supplied
    /// snapshot forms an independent historical undo step.
    ///
    /// # Parameters
    /// - `snapshot`: The discrete snapshot to push onto the undo stack.
    pub fn checkpoint(&mut self, snapshot: TextSnapshot) {
        self.push_undo_entry(snapshot);
        self.redo_stack.clear();
        self.last_action_time = None;
        self.last_action_kind = None;
        self.last_action_was_boundary = false;
    }

    /// Reverts to the previous snapshot on the undo stack, saving the current state onto the redo stack.
    ///
    /// Returns the restored [`TextSnapshot`] containing the historical text buffer, caret position,
    /// and selection range, or `None` if the undo stack is empty.
    ///
    /// # Parameters
    /// - `current_text`: The current text buffer before undoing.
    /// - `current_cursor`: The current caret byte offset before undoing.
    /// - `current_selection`: The current selection range before undoing.
    pub fn undo(
        &mut self,
        current_text: impl Into<String>,
        current_cursor: usize,
        current_selection: Option<(usize, usize)>,
    ) -> Option<TextSnapshot> {
        let current_snapshot = TextSnapshot::new(current_text, current_cursor, current_selection);
        self.undo_with_snapshot(current_snapshot)
    }

    /// Reverts to the previous snapshot using an explicit current [`TextSnapshot`].
    ///
    /// # Parameters
    /// - `current`: Snapshot of the active editor state prior to executing undo.
    pub fn undo_with_snapshot(&mut self, current: TextSnapshot) -> Option<TextSnapshot> {
        let previous = self.undo_stack.pop_back()?;
        self.push_redo_entry(current);
        self.last_action_time = None;
        self.last_action_kind = None;
        self.last_action_was_boundary = false;
        Some(previous)
    }

    /// Reverts to the previous snapshot, mutating the supplied text buffer, caret offset,
    /// and selection range in place.
    ///
    /// Returns `true` if an undo snapshot was available and applied, or `false` if the stack was empty.
    pub fn undo_in_place(
        &mut self,
        text: &mut String,
        cursor: &mut usize,
        selection: &mut Option<(usize, usize)>,
    ) -> bool {
        if let Some(restored) = self.undo(text.as_str(), *cursor, *selection) {
            *text = restored.text;
            *cursor = restored.cursor_byte_idx;
            *selection = restored.selection;
            true
        } else {
            false
        }
    }

    /// Re-applies the next reverted snapshot on the redo stack, saving the current state onto the undo stack.
    ///
    /// Returns the restored [`TextSnapshot`] or `None` if the redo stack is empty.
    ///
    /// # Parameters
    /// - `current_text`: The current text buffer before redoing.
    /// - `current_cursor`: The current caret byte offset before redoing.
    /// - `current_selection`: The current selection range before redoing.
    pub fn redo(
        &mut self,
        current_text: impl Into<String>,
        current_cursor: usize,
        current_selection: Option<(usize, usize)>,
    ) -> Option<TextSnapshot> {
        let current_snapshot = TextSnapshot::new(current_text, current_cursor, current_selection);
        self.redo_with_snapshot(current_snapshot)
    }

    /// Re-applies the next reverted snapshot using an explicit current [`TextSnapshot`].
    ///
    /// # Parameters
    /// - `current`: Snapshot of the active editor state prior to executing redo.
    pub fn redo_with_snapshot(&mut self, current: TextSnapshot) -> Option<TextSnapshot> {
        let next = self.redo_stack.pop_back()?;
        self.push_undo_entry(current);
        self.last_action_time = None;
        self.last_action_kind = None;
        self.last_action_was_boundary = false;
        Some(next)
    }

    /// Re-applies the next reverted snapshot, mutating the supplied text buffer, caret offset,
    /// and selection range in place.
    ///
    /// Returns `true` if a redo snapshot was available and applied, or `false` if the stack was empty.
    pub fn redo_in_place(
        &mut self,
        text: &mut String,
        cursor: &mut usize,
        selection: &mut Option<(usize, usize)>,
    ) -> bool {
        if let Some(restored) = self.redo(text.as_str(), *cursor, *selection) {
            *text = restored.text;
            *cursor = restored.cursor_byte_idx;
            *selection = restored.selection;
            true
        } else {
            false
        }
    }

    /// Internal helper appending a snapshot to the undo stack while enforcing the max depth bound.
    fn push_undo_entry(&mut self, snapshot: TextSnapshot) {
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(snapshot);
    }

    /// Internal helper appending a snapshot to the redo stack while enforcing the max depth bound.
    fn push_redo_entry(&mut self, snapshot: TextSnapshot) {
        if self.redo_stack.len() >= self.max_depth {
            self.redo_stack.pop_front();
        }
        self.redo_stack.push_back(snapshot);
    }
}

#[cfg(test)]
mod tests;