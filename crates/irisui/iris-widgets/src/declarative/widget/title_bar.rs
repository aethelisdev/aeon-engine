// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Frameless Window Title Bar Widget (`TitleBar`)
//!
//! Provides the declarative [`TitleBar`] widget builder implementing [`Widget`],
//! standard window management action routing ([`TitleBarAction`]), and fluent scope integration
//! for frameless desktop application windows without engine-layer coupling.

use std::ops::Deref;

use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, Theme, WidgetCursor,
    WidgetResponse, WidgetRole,
};

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::hash_label_with_seed;
use crate::declarative::widget::Widget;

/// Magic seed constant used when deriving default window control button tags.
pub const TITLE_BAR_CONTROL_SEED: u64 = 0x5449_544C_4542_4152; // "TITLEBAR"

/// Compile-time constant for the red hover background on the close button (`#E81123`).
const CLOSE_HOVER_BG: Color = Color::new(
    0xE8 as f32 / 255.0,
    0x11 as f32 / 255.0,
    0x23 as f32 / 255.0,
    1.0,
);

/// Compile-time constant for the subtle translucent hover background on control buttons.
const CONTROL_HOVER_BG: Color = Color::rgba(1.0, 1.0, 1.0, 0.12);

/// Derives the persistent semantic tag for the minimize window button.
///
/// Falls back to [`TITLE_BAR_CONTROL_SEED`] when `base_tag == 0`.
///
/// # Arguments
/// * `base_tag` - Root persistent tag assigned to the parent title bar.
#[inline]
#[must_use]
pub fn title_bar_minimize_tag(base_tag: u64) -> u64 {
    let seed = if base_tag != 0 {
        base_tag
    } else {
        TITLE_BAR_CONTROL_SEED
    };
    hash_label_with_seed(seed, "TitleBarMinimize")
}

/// Derives the persistent semantic tag for the maximize or restore window button.
///
/// Falls back to [`TITLE_BAR_CONTROL_SEED`] when `base_tag == 0`.
///
/// # Arguments
/// * `base_tag` - Root persistent tag assigned to the parent title bar.
#[inline]
#[must_use]
pub fn title_bar_maximize_tag(base_tag: u64) -> u64 {
    let seed = if base_tag != 0 {
        base_tag
    } else {
        TITLE_BAR_CONTROL_SEED
    };
    hash_label_with_seed(seed, "TitleBarMaximize")
}

/// Derives the persistent semantic tag for the close window button.
///
/// Falls back to [`TITLE_BAR_CONTROL_SEED`] when `base_tag == 0`.
///
/// # Arguments
/// * `base_tag` - Root persistent tag assigned to the parent title bar.
#[inline]
#[must_use]
pub fn title_bar_close_tag(base_tag: u64) -> u64 {
    let seed = if base_tag != 0 {
        base_tag
    } else {
        TITLE_BAR_CONTROL_SEED
    };
    hash_label_with_seed(seed, "TitleBarClose")
}

/// Window management action triggered by title bar control buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TitleBarAction {
    /// Request to minimize the host window into the taskbar or dock.
    Minimize,
    /// Request to toggle the host window between maximized and restored bounds.
    Maximize,
    /// Request to close the host window / terminate the application session.
    Close,
}

/// Comprehensive interaction response returned by evaluating an autonomous [`TitleBar`] widget.
///
/// Implements [`Deref<Target = WidgetResponse>`] for transparent access to generic
/// interaction flags (`clicked()`, `hovered()`) while providing direct access to window actions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TitleBarResponse {
    /// Underlying generic widget response for the root title bar container.
    pub response: WidgetResponse,
    /// Window management action clicked in this frame, if any.
    pub action: Option<TitleBarAction>,
}

impl Deref for TitleBarResponse {
    type Target = WidgetResponse;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl TitleBarResponse {
    /// Returns the window management action clicked in this frame, if any.
    #[inline]
    #[must_use]
    pub const fn action(&self) -> Option<TitleBarAction> {
        self.action
    }

    /// Returns `true` if the minimize window button was clicked in this frame.
    #[inline]
    #[must_use]
    pub const fn is_minimize(&self) -> bool {
        matches!(self.action, Some(TitleBarAction::Minimize))
    }

    /// Returns `true` if the maximize or restore window button was clicked in this frame.
    #[inline]
    #[must_use]
    pub const fn is_maximize(&self) -> bool {
        matches!(self.action, Some(TitleBarAction::Maximize))
    }

    /// Returns `true` if the close window button was clicked in this frame.
    #[inline]
    #[must_use]
    pub const fn is_close(&self) -> bool {
        matches!(self.action, Some(TitleBarAction::Close))
    }

    /// Returns `true` if the draggable title bar surface was clicked (excluding control buttons).
    ///
    /// Ideal for triggering host window dragging or double-click maximize gestures.
    #[inline]
    #[must_use]
    pub fn drag_area_clicked(&self) -> bool {
        self.action.is_none() && self.response.clicked()
    }
}

/// Configurable declarative builder for frameless window title bars.
///
/// Adheres strictly to the declarative builder pattern, providing autonomous layout of
/// leading application icon, window title, middle drag/spacer zone, and window control buttons.
#[derive(Debug, Clone, Copy)]
pub struct TitleBar<'a> {
    title: &'a str,
    icon: Option<&'a str>,
    height: f32,
    is_maximized: bool,
    tag: u64,
    background: Option<Color>,
    border_color: Option<Color>,
    text_color: Option<Color>,
    show_controls: bool,
    show_minimize: bool,
    show_maximize: bool,
    show_close: bool,
}

impl<'a> TitleBar<'a> {
    /// Creates a new [`TitleBar`] builder with the specified title string.
    ///
    /// # Arguments
    /// * `title` - Title text displayed in the header.
    #[inline]
    #[must_use]
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            icon: None,
            height: 32.0,
            is_maximized: false,
            tag: 0,
            background: None,
            border_color: None,
            text_color: None,
            show_controls: true,
            show_minimize: true,
            show_maximize: true,
            show_close: true,
        }
    }

    /// Sets the optional application icon text or symbol displayed to the left of the title.
    ///
    /// # Arguments
    /// * `icon` - Icon glyph or emoji displayed before the window title.
    #[inline]
    #[must_use]
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the height of the title bar in logical points. Defaults to 32.0 px.
    ///
    /// # Arguments
    /// * `height` - Vertical height constraint in logical pixels.
    #[inline]
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets whether the host window is currently maximized, updating the maximize/restore icon.
    ///
    /// # Arguments
    /// * `is_maximized` - Current window maximized state.
    #[inline]
    #[must_use]
    pub fn maximized(mut self, is_maximized: bool) -> Self {
        self.is_maximized = is_maximized;
        self
    }

    /// Assigns a 64-bit persistent semantic tag for interaction tracking and event routing.
    ///
    /// # Arguments
    /// * `tag` - Custom non-zero semantic tag identifier.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Overrides the background fill color of the title bar strip.
    ///
    /// # Arguments
    /// * `background` - Solid background color.
    #[inline]
    #[must_use]
    pub fn background(mut self, background: Color) -> Self {
        self.background = Some(background);
        self
    }

    /// Overrides the outline border color of the title bar strip.
    ///
    /// # Arguments
    /// * `border_color` - Solid outline border color.
    #[inline]
    #[must_use]
    pub fn border_color(mut self, border_color: Color) -> Self {
        self.border_color = Some(border_color);
        self
    }

    /// Overrides the primary text color of the title and icon.
    ///
    /// # Arguments
    /// * `text_color` - Solid text display color.
    #[inline]
    #[must_use]
    pub fn text_color(mut self, text_color: Color) -> Self {
        self.text_color = Some(text_color);
        self
    }

    /// Sets whether window management control buttons are rendered. Defaults to `true`.
    ///
    /// # Arguments
    /// * `show` - Master toggle for rendering window control buttons on the right.
    #[inline]
    #[must_use]
    pub fn show_controls(mut self, show: bool) -> Self {
        self.show_controls = show;
        self
    }

    /// Sets whether the minimize button is visible. Defaults to `true`.
    ///
    /// # Arguments
    /// * `show` - Whether the minimize button is displayed.
    #[inline]
    #[must_use]
    pub fn show_minimize(mut self, show: bool) -> Self {
        self.show_minimize = show;
        self
    }

    /// Sets whether the maximize/restore button is visible. Defaults to `true`.
    ///
    /// # Arguments
    /// * `show` - Whether the maximize button is displayed.
    #[inline]
    #[must_use]
    pub fn show_maximize(mut self, show: bool) -> Self {
        self.show_maximize = show;
        self
    }

    /// Sets whether the close button is visible. Defaults to `true`.
    ///
    /// # Arguments
    /// * `show` - Whether the close button is displayed.
    #[inline]
    #[must_use]
    pub fn show_close(mut self, show: bool) -> Self {
        self.show_close = show;
        self
    }

    /// Renders the title bar into the given scope and returns [`TitleBarResponse`].
    ///
    /// # Arguments
    /// * `scope` - Target UI scope.
    #[inline]
    pub fn show(self, scope: &mut UiScope<'_>) -> TitleBarResponse {
        scope.add(self)
    }
}

impl<'a> Widget for TitleBar<'a> {
    type Response = TitleBarResponse;

    fn ui(self, scope: &mut UiScope<'_>) -> Self::Response {
        let theme = scope.theme();
        let bg = self.background.unwrap_or(theme.toolbar_bg);
        let border_color = self.border_color.unwrap_or(theme.toolbar_border);
        let text_color = self.text_color.unwrap_or(theme.text_primary);
        let base_tag = resolve_base_tag(scope, self.tag, self.title);

        let bar_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .height(self.height)
            .background(bg)
            .border(1.0, border_color);

        let container_resp = scope.container_tagged(
            "TitleBar",
            bar_style,
            WidgetRole::Default,
            base_tag,
            |bar| {
                render_left_section(bar, self.icon, self.title, text_color);
                bar.empty_box_passive_named(
                    "TitleBarSpacer",
                    Style::new().flex_grow(1.0).height(self.height),
                );
                self.show_controls
                    .then(|| {
                        render_controls_section(
                            bar,
                            ControlsParams {
                                base_tag,
                                height: self.height,
                                is_maximized: self.is_maximized,
                                show_minimize: self.show_minimize,
                                show_maximize: self.show_maximize,
                                show_close: self.show_close,
                                theme: &theme,
                            },
                        )
                    })
                    .flatten()
            },
        );

        TitleBarResponse {
            response: container_resp.response,
            action: container_resp.inner,
        }
    }
}

/// Resolves the root semantic tag for the title bar, falling back to label derivation or control seed.
#[inline]
fn resolve_base_tag(scope: &UiScope<'_>, tag: u64, title: &str) -> u64 {
    if tag != 0 {
        tag
    } else {
        let derived = scope.tag_for(title);
        if derived != 0 {
            derived
        } else {
            TITLE_BAR_CONTROL_SEED
        }
    }
}

/// Re-export of [`TitleBarBuilder`] from the declarative scope module.
pub use crate::declarative::scope::title_bar::TitleBarBuilder;

/// Renders the leading left section of the title bar containing an optional icon and title text.
fn render_left_section(bar: &mut UiScope<'_>, icon: Option<&str>, title: &str, text_color: Color) {
    let left_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .gap(8.0)
        .padding_insets(Insets::new(0.0, 10.0, 0.0, 8.0))
        .flex_shrink(0.0);

    bar.container_passive_named("TitleBarLeft", left_style, |left| {
        if let Some(icon_glyph) = icon {
            left.label_styled_passive(
                "TitleBarIcon",
                icon_glyph,
                13.0,
                text_color,
                TextAlign::Left,
                Style::new(),
            );
        }

        left.label_styled_passive(
            "TitleBarTitle",
            title,
            12.0,
            text_color,
            TextAlign::Left,
            Style::new(),
        );
    });
}

/// Parameter container for rendering the window management control buttons.
struct ControlsParams<'a> {
    base_tag: u64,
    height: f32,
    is_maximized: bool,
    show_minimize: bool,
    show_maximize: bool,
    show_close: bool,
    theme: &'a Theme,
}

/// Renders the window management control buttons (minimize, maximize, close) in the title bar.
fn render_controls_section(
    bar: &mut UiScope<'_>,
    params: ControlsParams<'_>,
) -> Option<TitleBarAction> {
    let right_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .flex_shrink(0.0)
        .height(params.height);

    let controls_resp = bar.container_passive_named("TitleBarControls", right_style, |right| {
        let min_action = if params.show_minimize {
            let min_tag = title_bar_minimize_tag(params.base_tag);
            let min_resp = title_bar_button(
                right,
                min_tag,
                "WindowMinimizeButton",
                "—",
                false,
                params.height,
                params.theme,
            );
            min_resp.clicked().then_some(TitleBarAction::Minimize)
        } else {
            None
        };

        let max_action = if params.show_maximize {
            let max_tag = title_bar_maximize_tag(params.base_tag);
            let max_glyph = if params.is_maximized { "❐" } else { "□" };
            let max_resp = title_bar_button(
                right,
                max_tag,
                "WindowMaximizeButton",
                max_glyph,
                false,
                params.height,
                params.theme,
            );
            max_resp.clicked().then_some(TitleBarAction::Maximize)
        } else {
            None
        };

        let close_action = if params.show_close {
            let close_tag = title_bar_close_tag(params.base_tag);
            let close_resp = title_bar_button(
                right,
                close_tag,
                "WindowCloseButton",
                "✕",
                true,
                params.height,
                params.theme,
            );
            close_resp.clicked().then_some(TitleBarAction::Close)
        } else {
            None
        };

        min_action.or(max_action).or(close_action)
    });
    controls_resp.inner
}

/// Helper function to emit standard window management control buttons.
fn title_bar_button(
    scope: &mut UiScope<'_>,
    tag: u64,
    node_name: &'static str,
    label: &'static str,
    is_close: bool,
    height: f32,
    theme: &Theme,
) -> WidgetResponse {
    let node_id = scope.tree_mut().create_node();
    if let Some(node) = scope.tree_mut().get_mut(node_id) {
        node.set_name(node_name);
        node.set_role(WidgetRole::Button);
        node.tag = tag;
        node.interactive = true;
        node.cursor = Some(WidgetCursor::Pointer);
        node.set_text(label);
        node.font_size = 11.0;
        node.line_height = 13.0;
        node.text_align = TextAlign::Center;
        node.text_color = theme.text_secondary;
        node.hover_text_color = Some(Color::WHITE);

        let hover_bg = if is_close {
            CLOSE_HOVER_BG
        } else {
            CONTROL_HOVER_BG
        };

        node.set_style(
            Style::new()
                .width(44.0)
                .height(height)
                .background(Color::TRANSPARENT)
                .hover_background(hover_bg)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .flex_shrink(0.0),
        );
    }
    let parent = scope.parent();
    let _ = scope.tree_mut().add_child(parent, node_id);
    let (clicked, hovered, _) = scope.check_interaction(node_id);
    WidgetResponse::new(node_id, clicked, hovered, false)
}