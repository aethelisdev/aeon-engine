// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Cursor Translation Subsystem
//!
//! Provides bidirectional mapping between Iris UI hardware-agnostic [`WidgetCursor`]
//! values and native platform [`winit::window::CursorIcon`] primitives.

use iris_core::node::WidgetCursor;
use winit::window::CursorIcon;

/// Converts an Iris UI [`WidgetCursor`] specification into a native Winit [`CursorIcon`].
///
/// Ensures 1:1 parity across common graphical user interface cursors, including directional
/// resizing, beam inputs, click pointers, crosshairs, and prohibited states.
///
/// # Examples
///
/// ```
/// use iris_core::node::WidgetCursor;
/// use iris_winit::cursor::widget_cursor_to_winit;
/// use winit::window::CursorIcon;
///
/// assert_eq!(widget_cursor_to_winit(WidgetCursor::Pointer), CursorIcon::Pointer);
/// assert_eq!(widget_cursor_to_winit(WidgetCursor::Text), CursorIcon::Text);
/// ```
#[must_use]
#[inline]
pub fn widget_cursor_to_winit(cursor: WidgetCursor) -> CursorIcon {
    match cursor {
        WidgetCursor::Default => CursorIcon::Default,
        WidgetCursor::Pointer => CursorIcon::Pointer,
        WidgetCursor::Text => CursorIcon::Text,
        WidgetCursor::Crosshair => CursorIcon::Crosshair,
        WidgetCursor::Grab => CursorIcon::Grab,
        WidgetCursor::Grabbing => CursorIcon::Grabbing,
        WidgetCursor::ColResize => CursorIcon::ColResize,
        WidgetCursor::RowResize => CursorIcon::RowResize,
        WidgetCursor::EwResize => CursorIcon::EwResize,
        WidgetCursor::NsResize => CursorIcon::NsResize,
        WidgetCursor::NeswResize => CursorIcon::NeswResize,
        WidgetCursor::NwseResize => CursorIcon::NwseResize,
        WidgetCursor::NotAllowed => CursorIcon::NotAllowed,
    }
}

/// Converts a native Winit [`CursorIcon`] into an Iris UI [`WidgetCursor`], if a direct equivalent exists.
///
/// Returns `None` for platform cursors that do not have an immediate counterpart in Iris UI's
/// retained hierarchy (such as `Progress`, `Wait`, `Help`, or `Cell`).
///
/// # Examples
///
/// ```
/// use iris_core::node::WidgetCursor;
/// use iris_winit::cursor::winit_to_widget_cursor;
/// use winit::window::CursorIcon;
///
/// assert_eq!(winit_to_widget_cursor(CursorIcon::Pointer), Some(WidgetCursor::Pointer));
/// assert_eq!(winit_to_widget_cursor(CursorIcon::Wait), None);
/// ```
#[must_use]
#[inline]
pub fn winit_to_widget_cursor(icon: CursorIcon) -> Option<WidgetCursor> {
    match icon {
        CursorIcon::Default => Some(WidgetCursor::Default),
        CursorIcon::Pointer => Some(WidgetCursor::Pointer),
        CursorIcon::Text => Some(WidgetCursor::Text),
        CursorIcon::Crosshair => Some(WidgetCursor::Crosshair),
        CursorIcon::Grab => Some(WidgetCursor::Grab),
        CursorIcon::Grabbing => Some(WidgetCursor::Grabbing),
        CursorIcon::ColResize => Some(WidgetCursor::ColResize),
        CursorIcon::RowResize => Some(WidgetCursor::RowResize),
        CursorIcon::EwResize => Some(WidgetCursor::EwResize),
        CursorIcon::NsResize => Some(WidgetCursor::NsResize),
        CursorIcon::NeswResize => Some(WidgetCursor::NeswResize),
        CursorIcon::NwseResize => Some(WidgetCursor::NwseResize),
        CursorIcon::NotAllowed => Some(WidgetCursor::NotAllowed),
        _ => None,
    }
}