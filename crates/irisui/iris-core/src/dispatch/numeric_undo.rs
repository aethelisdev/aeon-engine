// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Numeric Widget Undo & Redo History (`iris-core::dispatch::numeric_undo`)
//!
//! Provides a bounded, zero-engine-dependency undo/redo history stack ([`NumericUndoStack`])
//! for scalar and vector numeric input controls (`NumericDrag`, `DragVec2`, `DragVec3`,
//! property sliders, and inline numeric text submissions).
//!
//! Tracks pre-drag baseline values during pointer scrubbing gestures, post-`Enter` or
//! focus-loss expression evaluations, and vector reset operations so `Ctrl+Z`, `Ctrl+Y`,
//! and `Ctrl+Shift+Z` shortcuts restore committed numeric values directly in-place during
//! the next declarative UI pass.

use std::collections::VecDeque;

use crate::dispatch::undo::DEFAULT_MAX_UNDO_DEPTH;

/// Represents a discrete, reversible numeric value transition recorded by a UI widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumericUndoEntry {
    /// A single scalar value transition bound to a widget's 64-bit semantic `tag`.
    Scalar {
        /// Semantic 64-bit tag of the scalar numeric widget or vector axis pill.
        tag: u64,
        /// Value prior to the drag gesture or inline expression submission.
        old_val: f32,
        /// Value committed at the end of the drag gesture or inline expression submission.
        new_val: f32,
    },
    /// A 3-component vector value transition (such as clicking the `🔄` reset button on a Vec3 row).
    Vec3 {
        /// Base semantic 64-bit tag of the 3D vector property row.
        base_tag: u64,
        /// 3-component `[x, y, z]` array prior to the transition.
        old_val: [f32; 3],
        /// 3-component `[x, y, z]` array committed by the transition.
        new_val: [f32; 3],
    },
}

impl NumericUndoEntry {
    /// Returns a concrete widget node tag in the [`crate::tree::UiTree`] that can be resolved
    /// via [`crate::tree::UiTree::find_node_by_tag`] to mark the owning panel dirty upon undo/redo.
    ///
    /// For [`Self::Scalar`], returns `tag` directly. For [`Self::Vec3`], returns `base_tag.wrapping_add(1)`
    /// which corresponds to the X-axis numeric pill node in the tree.
    #[inline]
    #[must_use]
    pub const fn target_node_tag(&self) -> u64 {
        match *self {
            Self::Scalar { tag, .. } => tag,
            Self::Vec3 { base_tag, .. } => base_tag.wrapping_add(1),
        }
    }
}

/// Pending numeric value restoration scheduled by an `undo()` or `redo()` invocation,
/// awaiting consumption by the target widget during the next declarative render pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PendingNumericRestore {
    /// Restores a single scalar value for the matching semantic `tag`.
    Scalar {
        /// Target semantic 64-bit tag.
        tag: u64,
        /// Restored floating-point value.
        val: f32,
    },
    /// Restores a 3-element float array for the matching `base_tag`.
    Vec3 {
        /// Target 3D vector property row base semantic tag.
        base_tag: u64,
        /// Restored `[x, y, z]` floating-point array.
        val: [f32; 3],
    },
}

/// Autonomous, bounded undo/redo stack for numeric UI widgets (`NumericDrag`, `DragVec2`, `DragVec3`, sliders).
///
/// Coordinates three numeric interaction lifecycles:
/// 1. **Continuous Pointer Scrubbing**: [`Self::begin_drag`] captures the initial pre-drag scalar
///    when a drag starts; [`Self::commit_drag`] records the `(start_val -> final_val)` transition
///    when the pointer button is released.
/// 2. **Inline Expression Submission (`Enter` / Focus Loss)**: [`Self::push_scalar`] records the
///    `(old_val -> new_val)` transition when an inline text edit evaluates to a new number.
/// 3. **Vector Reset Action**: [`Self::push_vec3`] records the 3-axis `[x, y, z]` transition when
///    a vector reset button is activated.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericUndoStack {
    /// Chronological ring buffer of committed numeric transitions available for `Ctrl+Z`.
    undo_stack: VecDeque<NumericUndoEntry>,
    /// LIFO stack of reverted numeric transitions available for `Ctrl+Y` / `Ctrl+Shift+Z`.
    redo_stack: Vec<NumericUndoEntry>,
    /// Maximum number of entries retained in `undo_stack`.
    max_depth: usize,
    /// Baseline `(tag, start_val)` captured at the inception of an active pointer drag gesture.
    active_drag: Option<(u64, f32)>,
    /// Pending restoration emitted by [`Self::undo`] or [`Self::redo`] to be applied on the next frame.
    pending: Option<PendingNumericRestore>,
}

impl Default for NumericUndoStack {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl NumericUndoStack {
    /// Creates an empty [`NumericUndoStack`] with [`DEFAULT_MAX_UNDO_DEPTH`] capacity.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: Vec::new(),
            max_depth: DEFAULT_MAX_UNDO_DEPTH,
            active_drag: None,
            pending: None,
        }
    }

    /// Captures the initial baseline value `current_val` at the start of a pointer drag gesture for `tag`.
    ///
    /// Subsequent calls with the same `tag` during the same drag session are no-ops, preserving the
    /// original pre-drag value until [`Self::commit_drag`] is called on pointer release.
    #[inline]
    pub fn begin_drag(&mut self, tag: u64, current_val: f32) {
        if tag == 0 {
            return;
        }
        if self
            .active_drag
            .is_none_or(|(active_tag, _)| active_tag != tag)
        {
            self.active_drag = Some((tag, current_val));
        }
    }

    /// Finalizes an active pointer drag gesture for `tag`, pushing a [`NumericUndoEntry::Scalar`]
    /// onto the undo stack if `final_val` differs from the captured pre-drag value.
    ///
    /// Returns `true` if a non-trivial numeric change was recorded.
    #[inline]
    pub fn commit_drag(&mut self, tag: u64, final_val: f32) -> bool {
        self.commit_drag_with_start(tag, final_val).0
    }

    /// Finalizes an active pointer drag gesture for `tag`, pushing a [`NumericUndoEntry::Scalar`]
    /// onto the undo stack if `final_val` differs from the captured pre-drag value.
    ///
    /// Returns `(changed, Some(start_val))` if a drag was active and tracked, or `(false, None)` otherwise.
    #[inline]
    pub fn commit_drag_with_start(&mut self, tag: u64, final_val: f32) -> (bool, Option<f32>) {
        if let Some((active_tag, start_val)) = self.active_drag
            && active_tag == tag
        {
            self.active_drag = None;
            let changed = (final_val - start_val).abs() > f32::EPSILON;
            if changed {
                self.push_entry(NumericUndoEntry::Scalar {
                    tag,
                    old_val: start_val,
                    new_val: final_val,
                });
            }
            return (changed, Some(start_val));
        }
        (false, None)
    }

    /// Returns the captured starting value of the current active drag gesture for `tag`, if any.
    #[inline]
    #[must_use]
    pub const fn active_drag_start(&self, tag: u64) -> Option<f32> {
        if let Some((active_tag, start_val)) = self.active_drag
            && active_tag == tag
        {
            return Some(start_val);
        }
        None
    }

    /// Records an immediate scalar value transition `(old_val -> new_val)` for `tag`
    /// (e.g. when committing an inline numeric expression via `Enter` or clicking outside).
    #[inline]
    pub fn push_scalar(&mut self, tag: u64, old_val: f32, new_val: f32) {
        if tag == 0 || (new_val - old_val).abs() <= f32::EPSILON {
            return;
        }
        self.active_drag = None;
        self.push_entry(NumericUndoEntry::Scalar {
            tag,
            old_val,
            new_val,
        });
    }

    /// Records an immediate 3D vector transition `(old_val -> new_val)` for `base_tag`
    /// (e.g. when clicking the `🔄` reset button on a 3D vector property row).
    #[inline]
    pub fn push_vec3(&mut self, base_tag: u64, old_val: [f32; 3], new_val: [f32; 3]) {
        if base_tag == 0 {
            return;
        }
        let changed = (new_val[0] - old_val[0]).abs() > f32::EPSILON
            || (new_val[1] - old_val[1]).abs() > f32::EPSILON
            || (new_val[2] - old_val[2]).abs() > f32::EPSILON;
        if !changed {
            return;
        }
        self.active_drag = None;
        self.push_entry(NumericUndoEntry::Vec3 {
            base_tag,
            old_val,
            new_val,
        });
    }

    /// Pushes a [`NumericUndoEntry`] onto the bounded undo deque and clears the redo stack.
    #[inline]
    fn push_entry(&mut self, entry: NumericUndoEntry) {
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(entry);
        self.redo_stack.clear();
    }

    /// Pops the most recent numeric transition from the undo stack, moves it to the redo stack,
    /// schedules a [`PendingNumericRestore`] with `old_val`, and returns the target widget node tag.
    #[inline]
    pub fn undo(&mut self) -> Option<u64> {
        let entry = self.undo_stack.pop_back()?;
        let target_tag = entry.target_node_tag();
        self.pending = Some(match entry {
            NumericUndoEntry::Scalar { tag, old_val, .. } => {
                PendingNumericRestore::Scalar { tag, val: old_val }
            }
            NumericUndoEntry::Vec3 {
                base_tag, old_val, ..
            } => PendingNumericRestore::Vec3 {
                base_tag,
                val: old_val,
            },
        });
        self.redo_stack.push(entry);
        Some(target_tag)
    }

    /// Pops the most recently undone numeric transition from the redo stack, moves it back to the
    /// undo stack, schedules a [`PendingNumericRestore`] with `new_val`, and returns the target widget node tag.
    #[inline]
    pub fn redo(&mut self) -> Option<u64> {
        let entry = self.redo_stack.pop()?;
        let target_tag = entry.target_node_tag();
        self.pending = Some(match entry {
            NumericUndoEntry::Scalar { tag, new_val, .. } => {
                PendingNumericRestore::Scalar { tag, val: new_val }
            }
            NumericUndoEntry::Vec3 {
                base_tag, new_val, ..
            } => PendingNumericRestore::Vec3 {
                base_tag,
                val: new_val,
            },
        });
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(entry);
        Some(target_tag)
    }

    /// Consumes and returns a pending scalar restoration if it matches `tag`.
    #[inline]
    pub fn take_pending_scalar(&mut self, tag: u64) -> Option<f32> {
        if let Some(PendingNumericRestore::Scalar {
            tag: pending_tag,
            val,
        }) = self.pending
            && pending_tag == tag
        {
            self.pending = None;
            return Some(val);
        }
        None
    }

    /// Consumes and returns a pending 3D vector restoration if it matches `base_tag`.
    #[inline]
    pub fn take_pending_vec3(&mut self, base_tag: u64) -> Option<[f32; 3]> {
        if let Some(PendingNumericRestore::Vec3 {
            base_tag: pending_tag,
            val,
        }) = self.pending
            && pending_tag == base_tag
        {
            self.pending = None;
            return Some(val);
        }
        None
    }

    /// Returns `true` if at least one committed numeric transition can be undone.
    #[inline]
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if at least one reverted numeric transition can be redone.
    #[inline]
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drag_lifecycle_and_undo_redo() {
        let mut stack = NumericUndoStack::new();
        let tag = 42;

        // First frame of drag captures initial baseline (10.0)
        stack.begin_drag(tag, 10.0);
        // Subsequent drag frames do not overwrite baseline
        stack.begin_drag(tag, 15.0);
        stack.begin_drag(tag, 22.5);

        // Release commits (10.0 -> 25.0)
        assert!(stack.commit_drag(tag, 25.0));
        assert!(stack.can_undo());
        assert!(!stack.can_redo());

        // Undo schedules 10.0 for tag 42
        assert_eq!(stack.undo(), Some(tag));
        assert_eq!(stack.take_pending_scalar(tag), Some(10.0));
        assert_eq!(stack.take_pending_scalar(tag), None);
        assert!(stack.can_redo());

        // Redo schedules 25.0 for tag 42
        assert_eq!(stack.redo(), Some(tag));
        assert_eq!(stack.take_pending_scalar(tag), Some(25.0));
    }

    #[test]
    fn test_vec3_reset_undo_redo() {
        let mut stack = NumericUndoStack::new();
        let base_tag = 1000;

        stack.push_vec3(base_tag, [1.0, 2.0, 3.0], [0.0, 0.0, 0.0]);
        assert!(stack.can_undo());

        // target_node_tag() returns base_tag + 1 (the X axis pill tag)
        assert_eq!(stack.undo(), Some(1001));
        assert_eq!(stack.take_pending_vec3(base_tag), Some([1.0, 2.0, 3.0]));

        assert_eq!(stack.redo(), Some(1001));
        assert_eq!(stack.take_pending_vec3(base_tag), Some([0.0, 0.0, 0.0]));
    }
}