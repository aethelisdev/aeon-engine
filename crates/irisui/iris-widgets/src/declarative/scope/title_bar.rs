// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Title Bar Scope Extension & Builder
//!
//! Provides the fluent [`title_bar`](UiScope::title_bar) entry point on [`UiScope`]
//! and the [`TitleBarBuilder`] for constructing and attaching autonomous frameless window title bars.

use super::core::UiScope;
use crate::declarative::widget::title_bar::{TitleBar, TitleBarAction, TitleBarResponse};
use iris_core::Color;

/// Fluent scope builder for configuring and emitting an autonomous [`TitleBar`].
///
/// Encapsulates builder state during title bar layout emission, providing chained configuration
/// of leading icon glyph, window title text, vertical bar height, maximized window state,
/// background and border colors, and window management controls visibility.
///
/// # Architecture & Lifetime
/// * `'s`: Mutable borrow lifetime of the active parent [`UiScope`].
/// * `'tree`: Lifetime of the underlying generational node arena [`iris_core::UiTree`].
/// * `'title`: Lifetime of string slices passed for the window title and leading icon.
///
/// # Integration
/// Evaluated either via [`.controls(...)`](TitleBarBuilder::controls) to handle window actions
/// with an inline callback, or via [`.show()`](TitleBarBuilder::show) returning a [`TitleBarResponse`].
pub struct TitleBarBuilder<'s, 'tree, 'title> {
    scope: &'s mut UiScope<'tree>,
    title_bar: TitleBar<'title>,
}

impl<'s, 'tree, 'title> TitleBarBuilder<'s, 'tree, 'title> {
    /// Creates a new [`TitleBarBuilder`] attached to the given UI scope.
    ///
    /// Initializes builder defaults with a standard height of 32.0 px, full window control
    /// buttons enabled (minimize, maximize, close), and unmaximized window state.
    ///
    /// # Arguments
    /// * `scope` - Active mutable UI scope receiving the emitted title bar container.
    /// * `title` - Title text displayed in the header strip.
    #[inline]
    #[must_use]
    pub fn new(scope: &'s mut UiScope<'tree>, title: &'title str) -> Self {
        Self {
            scope,
            title_bar: TitleBar::new(title),
        }
    }

    /// Sets the optional application icon text or symbol displayed to the left of the title.
    ///
    /// # Arguments
    /// * `icon` - Icon glyph, symbol, or emoji displayed before the window title label.
    #[inline]
    #[must_use]
    pub fn icon(mut self, icon: &'title str) -> Self {
        self.title_bar = self.title_bar.icon(icon);
        self
    }

    /// Sets the height of the title bar in logical points. Defaults to 32.0 px.
    ///
    /// Affects both the outer strip container height and the height of control buttons.
    ///
    /// # Arguments
    /// * `height` - Vertical height constraint in logical pixels.
    #[inline]
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.title_bar = self.title_bar.height(height);
        self
    }

    /// Sets whether the host window is currently maximized, updating the maximize/restore icon.
    ///
    /// When `true`, the maximize button displays the restore glyph (`"❐"`).
    /// When `false`, it displays the single square maximize glyph (`"□"`).
    ///
    /// # Arguments
    /// * `is_maximized` - Current window maximized state from the window host.
    #[inline]
    #[must_use]
    pub fn maximized(mut self, is_maximized: bool) -> Self {
        self.title_bar = self.title_bar.maximized(is_maximized);
        self
    }

    /// Assigns an explicit 64-bit persistent semantic tag for interaction tracking and event routing.
    ///
    /// If omitted or set to 0, the title bar autonomously derives its tag from `scope.tag_for(title)`,
    /// falling back to [`TITLE_BAR_CONTROL_SEED`](crate::declarative::widget::TITLE_BAR_CONTROL_SEED).
    ///
    /// # Arguments
    /// * `tag` - Custom non-zero semantic tag identifier.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.title_bar = self.title_bar.tag(tag);
        self
    }

    /// Overrides the background fill color of the title bar strip.
    ///
    /// If not specified, defaults to [`Theme::toolbar_bg`](iris_core::Theme::toolbar_bg).
    ///
    /// # Arguments
    /// * `background` - Solid background fill color.
    #[inline]
    #[must_use]
    pub fn background(mut self, background: Color) -> Self {
        self.title_bar = self.title_bar.background(background);
        self
    }

    /// Overrides the outline border color of the title bar strip.
    ///
    /// If not specified, defaults to [`Theme::toolbar_border`](iris_core::Theme::toolbar_border).
    ///
    /// # Arguments
    /// * `border_color` - Solid outline border color.
    #[inline]
    #[must_use]
    pub fn border_color(mut self, border_color: Color) -> Self {
        self.title_bar = self.title_bar.border_color(border_color);
        self
    }

    /// Overrides the primary text color of the title and icon.
    ///
    /// If not specified, defaults to [`Theme::text_primary`](iris_core::Theme::text_primary).
    ///
    /// # Arguments
    /// * `text_color` - Solid text display color.
    #[inline]
    #[must_use]
    pub fn text_color(mut self, text_color: Color) -> Self {
        self.title_bar = self.title_bar.text_color(text_color);
        self
    }

    /// Sets whether the window management control buttons container is rendered.
    ///
    /// When set to `false`, hides minimize, maximize, and close buttons entirely. Defaults to `true`.
    ///
    /// # Arguments
    /// * `show` - Master visibility toggle for window management control buttons.
    #[inline]
    #[must_use]
    pub fn show_controls(mut self, show: bool) -> Self {
        self.title_bar = self.title_bar.show_controls(show);
        self
    }

    /// Sets whether the minimize button is visible. Defaults to `true`.
    ///
    /// # Arguments
    /// * `show` - Whether the minimize button (`"—"`) is displayed in the controls group.
    #[inline]
    #[must_use]
    pub fn show_minimize(mut self, show: bool) -> Self {
        self.title_bar = self.title_bar.show_minimize(show);
        self
    }

    /// Sets whether the maximize/restore button is visible. Defaults to `true`.
    ///
    /// Useful for fixed-size dialogs or utility windows where resizing is disabled.
    ///
    /// # Arguments
    /// * `show` - Whether the maximize/restore button is displayed in the controls group.
    #[inline]
    #[must_use]
    pub fn show_maximize(mut self, show: bool) -> Self {
        self.title_bar = self.title_bar.show_maximize(show);
        self
    }

    /// Sets whether the close button is visible. Defaults to `true`.
    ///
    /// # Arguments
    /// * `show` - Whether the close button (`"✕"`) is displayed in the controls group.
    #[inline]
    #[must_use]
    pub fn show_close(mut self, show: bool) -> Self {
        self.title_bar = self.title_bar.show_close(show);
        self
    }

    /// Evaluates the title bar and passes any triggered [`TitleBarAction`] to the provided closure.
    ///
    /// If a window control button was clicked during the active frame, executes `on_action(action)`
    /// immediately before returning the [`TitleBarResponse`].
    ///
    /// # Arguments
    /// * `on_action` - Callback executed when a window control button (minimize, maximize, close) is clicked.
    ///
    /// # Returns
    /// Complete [`TitleBarResponse`] with underlying container interaction flags.
    pub fn controls<F: FnMut(TitleBarAction)>(self, mut on_action: F) -> TitleBarResponse {
        let resp = self.scope.add(self.title_bar);
        if let Some(action) = resp.action {
            on_action(action);
        }
        resp
    }

    /// Evaluates and attaches the title bar without registering an action callback.
    ///
    /// The caller can inspect the returned [`TitleBarResponse`] directly via `.action()`,
    /// `.is_close()`, `.is_minimize()`, `.is_maximize()`, or `.drag_area_clicked()`.
    ///
    /// # Returns
    /// Complete [`TitleBarResponse`] representing the emitted title bar.
    #[inline]
    pub fn show(self) -> TitleBarResponse {
        self.scope.add(self.title_bar)
    }
}

impl<'a> UiScope<'a> {
    /// Returns a fluent [`TitleBarBuilder`] for emitting an autonomous frameless window title bar.
    ///
    /// The builder manages leading application icon display, title text, middle flex spacer,
    /// and standard window management buttons (minimize, maximize/restore, close).
    ///
    /// # Arguments
    /// * `title` - Title text displayed in the header strip.
    ///
    /// # Returns
    /// Chained [`TitleBarBuilder`] for configuring styling, visibility, and action callbacks.
    #[inline]
    #[must_use]
    pub fn title_bar<'s, 'title>(
        &'s mut self,
        title: &'title str,
    ) -> TitleBarBuilder<'s, 'a, 'title> {
        TitleBarBuilder::new(self, title)
    }
}