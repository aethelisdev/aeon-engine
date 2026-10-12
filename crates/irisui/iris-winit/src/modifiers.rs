// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Modifier Keys State Management
//!
//! Tracks and provides ergonomic queries for keyboard modifier keys (Shift, Ctrl, Alt, Super/Meta)
//! across window events.
//!

use winit::event::Modifiers;
use winit::keyboard::ModifiersState as WinitModifiersState;

/// Retains active keyboard modifier keys state during window interactions.
///
/// Provides cross-platform querying methods for primary modifiers (`Shift`, `Control`, `Alt`, `Super`),
/// as well as ergonomic OS-aware shortcuts like [`ModifiersState::command`] which maps to `Super` on macOS
/// and `Control` on Windows/Linux.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct ModifiersState {
    /// Whether the Shift modifier key is held down.
    pub shift: bool,
    /// Whether the Control modifier key is held down.
    pub ctrl: bool,
    /// Whether the Alt / Option modifier key is held down.
    pub alt: bool,
    /// Whether the Super / Meta / Windows / Command modifier key is held down.
    pub super_key: bool,
}

impl ModifiersState {
    /// Creates an empty modifier state with no active keys held.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            super_key: false,
        }
    }

    /// Constructs a modifier state from a Winit [`Modifiers`] event wrapper.
    #[must_use]
    #[inline]
    pub fn from_winit(modifiers: &Modifiers) -> Self {
        Self::from_winit_state(modifiers.state())
    }

    /// Constructs a modifier state directly from Winit's internal [`WinitModifiersState`].
    #[must_use]
    #[inline]
    pub fn from_winit_state(state: WinitModifiersState) -> Self {
        Self {
            shift: state.shift_key(),
            ctrl: state.control_key(),
            alt: state.alt_key(),
            super_key: state.super_key(),
        }
    }

    /// Returns `true` if the Shift key is actively held.
    #[must_use]
    #[inline]
    pub const fn shift(&self) -> bool {
        self.shift
    }

    /// Returns `true` if the Control key is actively held.
    #[must_use]
    #[inline]
    pub const fn ctrl(&self) -> bool {
        self.ctrl
    }

    /// Returns `true` if the Alt / Option key is actively held.
    #[must_use]
    #[inline]
    pub const fn alt(&self) -> bool {
        self.alt
    }

    /// Returns `true` if the Super / Meta / Windows / Command key is actively held.
    #[must_use]
    #[inline]
    pub const fn super_key(&self) -> bool {
        self.super_key
    }

    /// Returns `true` if the platform primary command key is held down.
    ///
    /// Resolves to `super_key` on macOS and `ctrl` on Windows, Linux, and all other target platforms.
    #[must_use]
    #[inline]
    pub const fn command(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.super_key
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.ctrl
        }
    }

    /// Returns `true` if any modifier key is currently pressed.
    #[must_use]
    #[inline]
    pub const fn any(&self) -> bool {
        self.shift || self.ctrl || self.alt || self.super_key
    }

    /// Returns `true` if no modifier keys are pressed.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        !self.any()
    }
}