// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Modal and Dialog Interaction Responses
//!
//! Strongly-typed outcomes capturing user dismissals, confirmations, and actions
//! emitted during modal dialogue processing.

/// Specific cause or user gesture that prompted a modal dialogue dismissal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModalDismissReason {
    /// The user pressed the keyboard Escape key.
    EscapeKey,

    /// The user clicked the semi-transparent backdrop scrim outside the modal card bounds.
    BackdropClick,

    /// The user clicked the standard `✕` close button in the modal card header.
    CloseButton,

    /// The user clicked an explicit cancel button inside the modal dialog.
    CancelButton,
}

/// Evaluation outcome returned when rendering an interactive modal dialogue.
///
/// # Generics
/// - `Action`: Application-specific action payload emitted on user confirmation (e.g. enum, struct).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalOutcome<Action = ()> {
    /// The modal was dismissed via a standard dismiss gesture (Escape, backdrop click, close button).
    Dismissed(ModalDismissReason),

    /// A child control inside the modal emitted an explicit application action (e.g. Save, Delete, Confirm).
    Action(Action),

    /// The modal remains actively open and interactive with no terminating action.
    Active,
}

impl<Action> ModalOutcome<Action> {
    /// Returns `true` if the modal was dismissed by any dismiss gesture.
    #[inline]
    #[must_use]
    pub const fn is_dismissed(&self) -> bool {
        matches!(self, Self::Dismissed(_))
    }

    /// Returns `true` if the modal produced an affirmative application action.
    #[inline]
    #[must_use]
    pub const fn is_action(&self) -> bool {
        matches!(self, Self::Action(_))
    }

    /// Returns `true` if the modal remains active and open.
    #[inline]
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Converts this outcome into an optional application action if present.
    #[inline]
    pub fn action(self) -> Option<Action> {
        match self {
            Self::Action(a) => Some(a),
            _ => None,
        }
    }

    /// Returns the dismissal reason if dismissed.
    #[inline]
    #[must_use]
    pub const fn dismiss_reason(&self) -> Option<ModalDismissReason> {
        match self {
            Self::Dismissed(reason) => Some(*reason),
            _ => None,
        }
    }
}