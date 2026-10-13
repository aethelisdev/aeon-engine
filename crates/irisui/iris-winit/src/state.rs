// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Stateful Winit Window Integration
//!
//! Tracks platform cursor position in logical points, maintains HighDPI scale factor,
//! tracks active modifier keys, and orchestrates event translation for Iris UI.
//!

use crate::adapter::winit_to_ui_event;
use crate::modifiers::ModifiersState;
use iris_core::event::UiEvent;
use iris_core::geometry::Point;
use winit::event::WindowEvent;

/// Encapsulates platform window state and coordinates DPI-aware UI event dispatch.
///
/// Converts raw physical window coordinates to logical UI space using the active DPI scale factor,
/// maintains real-time cursor presence, and updates keyboard modifier flags.
///
/// # Examples
///
/// ```
/// use iris_winit::state::WinitState;
///
/// let mut state = WinitState::new(1.0);
/// assert_eq!(state.scale_factor(), 1.0);
/// assert!(state.cursor_pos().is_none());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct WinitState {
    /// Active cursor coordinate in logical UI points, or `None` if the cursor is outside the window.
    cursor_pos: Option<Point>,
    /// HighDPI scale factor applied to physical pixel inputs.
    scale_factor: f64,
    /// Real-time modifier key states (Shift, Control, Alt, Super).
    modifiers: ModifiersState,
    /// Whether the host platform window currently holds window focus.
    focused: bool,
}

impl WinitState {
    /// Creates a new `WinitState` with the specified HighDPI scale factor.
    ///
    /// If `scale_factor <= 0.0`, defaults to `1.0` to prevent division-by-zero errors.
    #[must_use]
    #[inline]
    pub fn new(scale_factor: f64) -> Self {
        let factor = if scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        Self {
            cursor_pos: None,
            scale_factor: factor,
            modifiers: ModifiersState::new(),
            focused: true,
        }
    }

    /// Returns the current HighDPI scale factor as an `f32`.
    #[must_use]
    #[inline]
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor as f32
    }

    /// Returns the current HighDPI scale factor as an `f64`.
    #[must_use]
    #[inline]
    pub const fn scale_factor_f64(&self) -> f64 {
        self.scale_factor
    }

    /// Updates the window HighDPI scale factor.
    ///
    /// If `scale_factor <= 0.0`, defaults to `1.0`.
    #[inline]
    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = if scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
    }

    /// Returns the active mouse cursor coordinate in logical UI points, if inside the window bounds.
    #[must_use]
    #[inline]
    pub const fn cursor_pos(&self) -> Option<Point> {
        self.cursor_pos
    }

    /// Returns true if an active mouse cursor coordinate is tracked within the window bounds.
    #[must_use]
    #[inline]
    pub const fn has_pointer(&self) -> bool {
        self.cursor_pos.is_some()
    }

    /// Explicitly updates or clears the active cursor position.
    #[inline]
    pub fn set_cursor_pos(&mut self, cursor_pos: Option<Point>) {
        self.cursor_pos = cursor_pos;
    }

    /// Returns the current keyboard modifier key states.
    #[must_use]
    #[inline]
    pub const fn modifiers(&self) -> ModifiersState {
        self.modifiers
    }

    /// Returns whether the window currently has operating system focus.
    #[must_use]
    #[inline]
    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    /// Updates state from a Winit [`WindowEvent`] and converts it into an Iris UI [`UiEvent`] if applicable.
    ///
    /// Automatically tracks scale factor changes, cursor movement and departure, focus changes,
    /// and keyboard modifiers.
    pub fn on_window_event(&mut self, event: &WindowEvent) -> Option<UiEvent> {
        match event {
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.set_scale_factor(*scale_factor);
                None
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = ModifiersState::from_winit(modifiers);
                None
            }
            WindowEvent::Focused(focused) => {
                self.focused = *focused;
                None
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor_pos = None;
                None
            }
            WindowEvent::CursorMoved { position, .. } => {
                let logical_x = (position.x / self.scale_factor) as f32;
                let logical_y = (position.y / self.scale_factor) as f32;
                let point = Point::new(logical_x, logical_y);
                self.cursor_pos = Some(point);
                Some(UiEvent::MouseMove { point })
            }
            WindowEvent::MouseInput { .. } | WindowEvent::MouseWheel { .. } => {
                let p = self.cursor_pos.unwrap_or(Point::new(0.0, 0.0));
                winit_to_ui_event(event, p)
            }
            WindowEvent::KeyboardInput { .. } | WindowEvent::Ime(_) => {
                let p = self.cursor_pos.unwrap_or(Point::new(0.0, 0.0));
                winit_to_ui_event(event, p)
            }
            _ => None,
        }
    }
}