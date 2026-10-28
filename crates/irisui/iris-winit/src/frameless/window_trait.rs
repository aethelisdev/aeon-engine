// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Window Platform Interface Abstraction
//!
//! Defines the decoupled [`WindowInterface`] trait to allow hardware-independent
//! and testable frameless window management without direct binding to concrete windowing objects.

use super::geometry::ResizeDirection;
use iris_core::geometry::Size;
use iris_core::node::WidgetCursor;
use std::fmt;
use std::sync::Arc;
use winit::window::{CursorIcon, Window};

/// Describes an error encountered during an interactive platform window operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowOpError {
    /// The window server or display backend rejected or failed to process the request.
    PlatformRejected,
    /// The requested operation is unsupported on the current platform backend.
    Unsupported,
}

impl fmt::Display for WindowOpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlatformRejected => write!(f, "Platform rejected the window operation"),
            Self::Unsupported => {
                write!(f, "Window operation is unsupported by the platform backend")
            }
        }
    }
}

impl std::error::Error for WindowOpError {}

/// Platform window interface required by frameless window hosting and drag-resizing.
///
/// Decouples window manipulation (cursor changes, interactive dragging, resizing, state toggling)
/// from the concrete [`winit::window::Window`] type, enabling headless unit testing and
/// pure software verification.
pub trait WindowInterface {
    /// Sets the native platform cursor icon.
    ///
    /// # Parameters
    /// - `icon`: Native platform cursor representation.
    fn set_cursor(&mut self, icon: CursorIcon);

    /// Sets the cursor icon using an Iris UI hardware-independent [`WidgetCursor`].
    ///
    /// Automatically converts the [`WidgetCursor`] into the corresponding native platform
    /// [`CursorIcon`] representation via [`crate::cursor::widget_cursor_to_winit`].
    #[inline]
    fn set_widget_cursor(&mut self, cursor: WidgetCursor) {
        self.set_cursor(crate::cursor::widget_cursor_to_winit(cursor));
    }

    /// Initiates an interactive window drag-resize operation along the specified boundary direction.
    ///
    /// Returns `Ok(())` if the windowing server successfully accepted the resize drag request,
    /// or `Err(WindowOpError)` if the operation was rejected, unsupported, or failed.
    fn drag_resize_window(&mut self, direction: ResizeDirection) -> Result<(), WindowOpError>;

    /// Initiates an interactive window move/drag operation from client area interaction.
    ///
    /// Returns `Ok(())` if the windowing server initiated the move, or `Err(WindowOpError)` otherwise.
    fn drag_window(&mut self) -> Result<(), WindowOpError>;

    /// Returns whether the platform window is currently maximized.
    fn is_maximized(&self) -> bool;

    /// Toggles the maximized state of the window between restored and maximized.
    fn toggle_maximize(&mut self);

    /// Minimizes the window to the system taskbar or dock.
    fn minimize(&mut self);

    /// Requests that the window be closed.
    fn close(&mut self);

    /// Returns the logical dimensions of the client area in logical points.
    ///
    /// For native Winit windows, calculates the logical dimensions from [`Window::inner_size`]
    /// divided by the platform [`Window::scale_factor`].
    /// Headless mock implementors return their configured virtual dimensions.
    fn logical_size(&self) -> Size;

    /// Returns the active HighDPI scale factor for this window.
    ///
    /// For native Winit windows, returns [`Window::scale_factor`], guaranteed to be positive
    /// and finite (falling back to `1.0` if non-positive or non-finite).
    fn scale_factor(&self) -> f64;
}

impl WindowInterface for Window {
    #[inline]
    fn set_cursor(&mut self, icon: CursorIcon) {
        Window::set_cursor(self, icon);
    }

    #[inline]
    fn drag_resize_window(&mut self, direction: ResizeDirection) -> Result<(), WindowOpError> {
        Window::drag_resize_window(self, direction.to_winit())
            .map_err(|_| WindowOpError::PlatformRejected)
    }

    #[inline]
    fn drag_window(&mut self) -> Result<(), WindowOpError> {
        Window::drag_window(self).map_err(|_| WindowOpError::PlatformRejected)
    }

    #[inline]
    fn is_maximized(&self) -> bool {
        Window::is_maximized(self)
    }

    #[inline]
    fn toggle_maximize(&mut self) {
        let is_max = self.is_maximized();
        self.set_maximized(!is_max);
    }

    #[inline]
    fn minimize(&mut self) {
        self.set_minimized(true);
    }

    #[inline]
    fn close(&mut self) {
        self.set_visible(false);
    }

    #[inline]
    fn scale_factor(&self) -> f64 {
        let scale = Window::scale_factor(self);
        if scale > 0.0 && scale.is_finite() {
            scale
        } else {
            1.0
        }
    }

    #[inline]
    fn logical_size(&self) -> Size {
        let phys = Window::inner_size(self);
        let factor = self.scale_factor();
        Size::new(
            (phys.width as f64 / factor) as f32,
            (phys.height as f64 / factor) as f32,
        )
    }
}

impl WindowInterface for &Window {
    #[inline]
    fn set_cursor(&mut self, icon: CursorIcon) {
        Window::set_cursor(self, icon);
    }

    #[inline]
    fn drag_resize_window(&mut self, direction: ResizeDirection) -> Result<(), WindowOpError> {
        Window::drag_resize_window(self, direction.to_winit())
            .map_err(|_| WindowOpError::PlatformRejected)
    }

    #[inline]
    fn drag_window(&mut self) -> Result<(), WindowOpError> {
        Window::drag_window(self).map_err(|_| WindowOpError::PlatformRejected)
    }

    #[inline]
    fn is_maximized(&self) -> bool {
        Window::is_maximized(self)
    }

    #[inline]
    fn toggle_maximize(&mut self) {
        let is_max = self.is_maximized();
        self.set_maximized(!is_max);
    }

    #[inline]
    fn minimize(&mut self) {
        self.set_minimized(true);
    }

    #[inline]
    fn close(&mut self) {
        self.set_visible(false);
    }

    #[inline]
    fn scale_factor(&self) -> f64 {
        let scale = Window::scale_factor(self);
        if scale > 0.0 && scale.is_finite() {
            scale
        } else {
            1.0
        }
    }

    #[inline]
    fn logical_size(&self) -> Size {
        let phys = Window::inner_size(self);
        let factor = self.scale_factor();
        Size::new(
            (phys.width as f64 / factor) as f32,
            (phys.height as f64 / factor) as f32,
        )
    }
}

impl WindowInterface for Arc<Window> {
    #[inline]
    fn set_cursor(&mut self, icon: CursorIcon) {
        Window::set_cursor(self, icon);
    }

    #[inline]
    fn drag_resize_window(&mut self, direction: ResizeDirection) -> Result<(), WindowOpError> {
        Window::drag_resize_window(self, direction.to_winit())
            .map_err(|_| WindowOpError::PlatformRejected)
    }

    #[inline]
    fn drag_window(&mut self) -> Result<(), WindowOpError> {
        Window::drag_window(self).map_err(|_| WindowOpError::PlatformRejected)
    }

    #[inline]
    fn is_maximized(&self) -> bool {
        Window::is_maximized(self)
    }

    #[inline]
    fn toggle_maximize(&mut self) {
        let is_max = self.is_maximized();
        self.set_maximized(!is_max);
    }

    #[inline]
    fn minimize(&mut self) {
        self.set_minimized(true);
    }

    #[inline]
    fn close(&mut self) {
        self.set_visible(false);
    }

    #[inline]
    fn scale_factor(&self) -> f64 {
        let scale = Window::scale_factor(self);
        if scale > 0.0 && scale.is_finite() {
            scale
        } else {
            1.0
        }
    }

    #[inline]
    fn logical_size(&self) -> Size {
        let phys = Window::inner_size(self);
        let factor = self.scale_factor();
        Size::new(
            (phys.width as f64 / factor) as f32,
            (phys.height as f64 / factor) as f32,
        )
    }
}

impl WindowInterface for &Arc<Window> {
    #[inline]
    fn set_cursor(&mut self, icon: CursorIcon) {
        Window::set_cursor(self, icon);
    }

    #[inline]
    fn drag_resize_window(&mut self, direction: ResizeDirection) -> Result<(), WindowOpError> {
        Window::drag_resize_window(self, direction.to_winit())
            .map_err(|_| WindowOpError::PlatformRejected)
    }

    #[inline]
    fn drag_window(&mut self) -> Result<(), WindowOpError> {
        Window::drag_window(self).map_err(|_| WindowOpError::PlatformRejected)
    }

    #[inline]
    fn is_maximized(&self) -> bool {
        Window::is_maximized(self)
    }

    #[inline]
    fn toggle_maximize(&mut self) {
        let is_max = self.is_maximized();
        self.set_maximized(!is_max);
    }

    #[inline]
    fn minimize(&mut self) {
        self.set_minimized(true);
    }

    #[inline]
    fn close(&mut self) {
        self.set_visible(false);
    }

    #[inline]
    fn scale_factor(&self) -> f64 {
        let scale = Window::scale_factor(self);
        if scale > 0.0 && scale.is_finite() {
            scale
        } else {
            1.0
        }
    }

    #[inline]
    fn logical_size(&self) -> Size {
        let phys = Window::inner_size(self);
        let factor = self.scale_factor();
        Size::new(
            (phys.width as f64 / factor) as f32,
            (phys.height as f64 / factor) as f32,
        )
    }
}

impl<W: WindowInterface + ?Sized> WindowInterface for &mut W {
    #[inline]
    fn set_cursor(&mut self, icon: CursorIcon) {
        (**self).set_cursor(icon);
    }

    #[inline]
    fn set_widget_cursor(&mut self, cursor: WidgetCursor) {
        (**self).set_widget_cursor(cursor);
    }

    #[inline]
    fn drag_resize_window(&mut self, direction: ResizeDirection) -> Result<(), WindowOpError> {
        (**self).drag_resize_window(direction)
    }

    #[inline]
    fn drag_window(&mut self) -> Result<(), WindowOpError> {
        (**self).drag_window()
    }

    #[inline]
    fn is_maximized(&self) -> bool {
        (**self).is_maximized()
    }

    #[inline]
    fn toggle_maximize(&mut self) {
        (**self).toggle_maximize();
    }

    #[inline]
    fn minimize(&mut self) {
        (**self).minimize();
    }

    #[inline]
    fn close(&mut self) {
        (**self).close();
    }

    #[inline]
    fn logical_size(&self) -> Size {
        (**self).logical_size()
    }

    #[inline]
    fn scale_factor(&self) -> f64 {
        (**self).scale_factor()
    }
}

impl<W: WindowInterface + ?Sized> WindowInterface for Box<W> {
    #[inline]
    fn set_cursor(&mut self, icon: CursorIcon) {
        (**self).set_cursor(icon);
    }

    #[inline]
    fn set_widget_cursor(&mut self, cursor: WidgetCursor) {
        (**self).set_widget_cursor(cursor);
    }

    #[inline]
    fn drag_resize_window(&mut self, direction: ResizeDirection) -> Result<(), WindowOpError> {
        (**self).drag_resize_window(direction)
    }

    #[inline]
    fn drag_window(&mut self) -> Result<(), WindowOpError> {
        (**self).drag_window()
    }

    #[inline]
    fn is_maximized(&self) -> bool {
        (**self).is_maximized()
    }

    #[inline]
    fn toggle_maximize(&mut self) {
        (**self).toggle_maximize();
    }

    #[inline]
    fn minimize(&mut self) {
        (**self).minimize();
    }

    #[inline]
    fn close(&mut self) {
        (**self).close();
    }

    #[inline]
    fn logical_size(&self) -> Size {
        (**self).logical_size()
    }

    #[inline]
    fn scale_factor(&self) -> f64 {
        (**self).scale_factor()
    }
}