// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Modal Dialogue Focus & Tab Navigation Cycle
//!
//! Provides deterministic cyclic keyboard focus navigation between interactive buttons,
//! text inputs, and custom controls within active modal dialogue cards.

use iris_widgets::controls::focus::FocusTarget;

/// Identifies an interactive control target within an active modal dialogue card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogFocusItem {
    /// Text input buffer or field at the given index.
    Input(u32),
    /// Secondary dismissal / Cancel button in the footer or header.
    Cancel,
    /// Primary affirmative / Confirm / Submit / Close button.
    Confirm,
    /// Custom widget identified by a 64-bit semantic tag.
    Custom(u64),
}

impl DialogFocusItem {
    /// Converts this item into a 64-bit semantic tag for widget highlight integration.
    ///
    /// Semantic Tag Space:
    /// - `Confirm`: `0xF0C0_0000_C011_F111`
    /// - `Cancel`: `0xF0C0_0000_CA11_CE01`
    /// - `Input(idx)`: `0xF0C0_1000_0000_0000 | (idx as u64)`
    /// - `Custom(tag)`: `tag`
    #[must_use]
    #[inline]
    pub const fn to_semantic_tag(self) -> u64 {
        match self {
            Self::Confirm => 0xF0C0_0000_C011_F111,
            Self::Cancel => 0xF0C0_0000_CA11_CE01,
            Self::Input(idx) => 0xF0C0_1000_0000_0000 | (idx as u64),
            Self::Custom(tag) => tag,
        }
    }

    /// Converts this item into an [`iris_widgets::controls::focus::FocusTarget`].
    #[must_use]
    #[inline]
    pub const fn to_focus_target(self) -> FocusTarget {
        FocusTarget::Tag(self.to_semantic_tag())
    }
}

/// Deterministic cyclic focus state machine managing keyboard navigation across modal controls.
///
/// Coordinates `Tab` (forward) and `Shift+Tab` (reverse) focus traversal without allocations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DialogFocusCycle {
    /// Ordered list of focusable interactive items within the active modal.
    targets: Vec<DialogFocusItem>,
    /// Currently focused item, if any.
    current: Option<DialogFocusItem>,
}

impl DialogFocusCycle {
    /// Creates a new empty focus cycle with no registered targets.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            targets: Vec::new(),
            current: None,
        }
    }

    /// Creates a focus cycle populated with the given slice of focusable targets.
    ///
    /// The first target in the slice is immediately marked as focused.
    #[must_use]
    pub fn with_targets(targets: impl Into<Vec<DialogFocusItem>>) -> Self {
        let targets = targets.into();
        let current = targets.first().copied();
        Self { targets, current }
    }

    /// Registers focusable targets and sets the initial focus target.
    pub fn set_targets(&mut self, targets: Vec<DialogFocusItem>, initial: Option<DialogFocusItem>) {
        self.targets = targets;
        self.current = initial.or_else(|| self.targets.first().copied());
    }

    /// Returns a shared slice of all focusable targets in navigation order.
    #[must_use]
    #[inline]
    pub fn targets(&self) -> &[DialogFocusItem] {
        &self.targets
    }

    /// Returns the currently focused control item, if any.
    #[must_use]
    #[inline]
    pub const fn current(&self) -> Option<DialogFocusItem> {
        self.current
    }

    /// Sets the focused control target directly.
    pub fn set_current(&mut self, item: DialogFocusItem) {
        if self.targets.contains(&item) {
            self.current = Some(item);
        }
    }

    /// Evaluates whether the specified item currently owns keyboard focus.
    #[must_use]
    #[inline]
    pub fn is_focused(&self, item: DialogFocusItem) -> bool {
        self.current == Some(item)
    }

    /// Cycles focus through the registered targets.
    ///
    /// If `reverse` is `false`, navigates forward (`Tab`).
    /// If `reverse` is `true`, navigates backward (`Shift+Tab`).
    ///
    /// Returns the newly focused item, or `None` if the cycle has no targets.
    pub fn cycle(&mut self, reverse: bool) -> Option<DialogFocusItem> {
        if self.targets.is_empty() {
            self.current = None;
            return None;
        }

        let next_idx = match self.current {
            Some(curr) => match self.targets.iter().position(|&t| t == curr) {
                Some(idx) => {
                    if reverse {
                        if idx == 0 {
                            self.targets.len() - 1
                        } else {
                            idx - 1
                        }
                    } else {
                        (idx + 1) % self.targets.len()
                    }
                }
                None => {
                    if reverse {
                        self.targets.len() - 1
                    } else {
                        0
                    }
                }
            },
            None => {
                if reverse {
                    self.targets.len() - 1
                } else {
                    0
                }
            }
        };

        let next_item = self.targets[next_idx];
        self.current = Some(next_item);
        Some(next_item)
    }

    /// Navigates focus to the next control target (`Tab`).
    #[inline]
    pub fn focus_next(&mut self) -> Option<DialogFocusItem> {
        self.cycle(false)
    }

    /// Navigates focus to the previous control target (`Shift+Tab`).
    #[inline]
    pub fn focus_prev(&mut self) -> Option<DialogFocusItem> {
        self.cycle(true)
    }

    /// Clears any currently active focus selection.
    #[inline]
    pub fn clear(&mut self) {
        self.current = None;
    }
}