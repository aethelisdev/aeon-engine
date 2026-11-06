// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Themed Card Containers
//!
//! Provides styled card grouping components on [`UiScope`]: titled property cards,
//! custom header cards, action buttons, and bordered inspector section containers.

use super::super::core::UiScope;
use super::generic::InnerResponse;
use crate::declarative::types::WidgetResponse;
use iris_core::{AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetRole};

/// Response returned by declarative card header widgets.
///
/// Encapsulates interaction status for the header row and its optional action button.
#[derive(Debug, Clone, Copy)]
pub struct CardHeaderResponse {
    /// Interaction response corresponding to the card header container row.
    pub response: WidgetResponse,
    /// Interaction response corresponding to the optional action button.
    pub action_response: Option<WidgetResponse>,
    /// Whether the action button (e.g., delete/close) was clicked during the active frame.
    pub action_clicked: bool,
}

impl CardHeaderResponse {
    /// Returns `true` if the card header was clicked during the active frame.
    #[inline]
    #[must_use]
    pub fn clicked(&self) -> bool {
        self.response.clicked()
    }

    /// Returns `true` if the mouse cursor is currently hovering over the card header.
    #[inline]
    #[must_use]
    pub fn hovered(&self) -> bool {
        self.response.hovered()
    }

    /// Returns `true` if the header action button was clicked during the active frame.
    #[inline]
    #[must_use]
    pub const fn action_clicked(&self) -> bool {
        self.action_clicked
    }

    /// Invokes the provided closure if the action button was clicked in this frame.
    ///
    /// Enables pure declarative event handling without requiring intermediate `let` statements.
    #[inline]
    pub fn on_action(self, f: impl FnOnce()) -> Self {
        if self.action_clicked {
            f();
        }
        self
    }
}

/// Response returned when emitting an autonomous card container.
///
/// Encapsulates the outer container frame response, header response, optional action click,
/// and the result value produced by the body closure.
#[derive(Debug, Clone)]
pub struct CardResponse<R> {
    /// Interaction response corresponding to the outer card container frame.
    pub response: WidgetResponse,
    /// Interaction response corresponding to the header row.
    pub header_response: WidgetResponse,
    /// Whether the card's header action button (if configured) was clicked.
    pub action_clicked: bool,
    /// Return value of the inner body scope closure.
    pub inner: R,
}

impl<R> CardResponse<R> {
    /// Returns `true` if the outer card container was clicked during the active frame.
    #[inline]
    #[must_use]
    pub fn clicked(&self) -> bool {
        self.response.clicked()
    }

    /// Returns `true` if the mouse cursor is currently hovering over the outer card container.
    #[inline]
    #[must_use]
    pub fn hovered(&self) -> bool {
        self.response.hovered()
    }

    /// Returns `true` if the card header row was clicked during the active frame.
    #[inline]
    #[must_use]
    pub fn header_clicked(&self) -> bool {
        self.header_response.clicked()
    }

    /// Returns `true` if the mouse cursor is hovering over the card header row.
    #[inline]
    #[must_use]
    pub fn header_hovered(&self) -> bool {
        self.header_response.hovered()
    }

    /// Returns `true` if the header action button was clicked during the active frame.
    #[inline]
    #[must_use]
    pub const fn action_clicked(&self) -> bool {
        self.action_clicked
    }

    /// Invokes the provided closure if the action button was clicked in this frame.
    ///
    /// Enables pure declarative event handling without requiring intermediate `let` statements.
    #[inline]
    pub fn on_action(self, f: impl FnOnce()) -> Self {
        if self.action_clicked {
            f();
        }
        self
    }

    /// Returns an immutable reference to the inner result value.
    #[inline]
    pub const fn inner(&self) -> &R {
        &self.inner
    }

    /// Consumes the response and extracts the inner result value.
    #[inline]
    pub fn into_inner(self) -> R {
        self.inner
    }
}

/// Configuration parameters for rendering a styled card container.
#[derive(Debug, Clone, Copy, Default)]
pub struct CardOptions<'a> {
    /// Optional unicode icon prepended to the title.
    pub icon: Option<&'a str>,
    /// Optional accent color for title and icon.
    pub color: Option<Color>,
    /// Optional action button label (e.g. `"×"` or `"🗑"`).
    pub action_label: Option<&'a str>,
    /// Optional border radius override.
    pub border_radius: Option<f32>,
}

impl<'a> CardOptions<'a> {
    /// Creates a new default [`CardOptions`].
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            icon: None,
            color: None,
            action_label: None,
            border_radius: None,
        }
    }
}

/// Fluent builder for assembling and customizing card containers.
pub struct CardBuilder<'a, 'b> {
    scope: &'a mut UiScope<'b>,
    name: Option<String>,
    title: String,
    icon: Option<String>,
    color: Option<Color>,
    action: Option<String>,
    border_radius: Option<f32>,
}

impl<'a, 'b> CardBuilder<'a, 'b> {
    /// Constructs a new [`CardBuilder`] with the specified heading title.
    #[inline]
    pub fn new(scope: &'a mut UiScope<'b>, title: impl Into<String>) -> Self {
        Self {
            scope,
            name: None,
            title: title.into(),
            icon: None,
            color: None,
            action: None,
            border_radius: None,
        }
    }

    /// Sets the node debug name for the card container.
    #[inline]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets a unicode icon prepended to the card title.
    #[inline]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the accent color for the title and icon.
    #[inline]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Alias for [`Self::color`].
    #[inline]
    pub fn accent(self, color: Color) -> Self {
        self.color(color)
    }

    /// Sets an action button label (e.g. `"×"` or `"🗑"`).
    #[inline]
    pub fn action(mut self, action_label: impl Into<String>) -> Self {
        self.action = Some(action_label.into());
        self
    }

    /// Overrides the container border radius. Defaults to 6.0px.
    #[inline]
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(radius);
        self
    }

    /// Emits the configured card and executes the child body closure.
    pub fn show<R>(self, f: impl FnOnce(&mut UiScope<'_>) -> R) -> CardResponse<R> {
        let name = self.name.unwrap_or_else(|| "Card".to_string());
        let options = CardOptions {
            icon: self.icon.as_deref(),
            color: self.color,
            action_label: self.action.as_deref(),
            border_radius: self.border_radius,
        };
        self.scope
            .card_with_action_internal(name, &self.title, options, f)
    }
}

impl<'a> UiScope<'a> {
    /// Emits a named elevated, bordered card container conforming to flexbox layout.
    ///
    /// The card expands according to its parent container's layout constraints without requiring
    /// hardcoded physical coordinates or triggering premature layout subtree evaluations.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier assigned to the card container node.
    /// * `title` - Card heading string rendered in a styled header row.
    /// * `f` - Child scope closure emitting the body content.
    pub fn card_named<R, F>(
        &mut self,
        name: impl Into<String>,
        title: &str,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let resp = self.card_with_action_internal(name, title, CardOptions::new(), f);
        InnerResponse::new(resp.response, resp.inner)
    }

    /// Emits a styled card panel container using active theme surface and border tokens.
    ///
    /// Ideal for inspector component cards, tool drawers, and grouped property panels.
    pub fn card_panel<R, F>(&mut self, name: impl Into<String>, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let theme = self.tree.theme();
        let card_style = Style::new()
            .flex_col()
            .background(theme.card_bg)
            .border(1.0, theme.card_border)
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);
        self.container_named(name, card_style, f)
    }

    /// Emits a standardized card header bar with title, optional icon, and optional accent color.
    pub fn card_header(
        &mut self,
        title: &str,
        icon: Option<&str>,
        color: Option<Color>,
    ) -> WidgetResponse {
        self.card_header_internal(title, icon, color, None).response
    }

    /// Emits a standardized card header bar with title, optional icon, accent color, and an interactive action button.
    pub fn card_header_with_action(
        &mut self,
        title: &str,
        icon: Option<&str>,
        color: Option<Color>,
        action_label: &str,
    ) -> CardHeaderResponse {
        self.card_header_internal(title, icon, color, Some(action_label))
    }

    fn card_header_internal(
        &mut self,
        title: &str,
        icon: Option<&str>,
        color: Option<Color>,
        action_label: Option<&str>,
    ) -> CardHeaderResponse {
        let theme = self.tree.theme();
        let accent_color = color.unwrap_or(theme.accent);
        let header_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .height(20.0)
            .padding_insets(Insets::new(0.0, 2.0, 0.0, 2.0));

        let resp = self.container_named("CardHeader", header_style, |header_scope| {
            let title_group_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .gap(5.0);

            header_scope.container_named("TitleGroup", title_group_style, |group| {
                if let Some(ic) = icon {
                    group.label_styled_passive(
                        "CardTextIcon",
                        ic,
                        12.0,
                        accent_color,
                        TextAlign::Left,
                        Style::new().width(15.0).height(20.0),
                    );
                }

                group.label_styled_passive(
                    "CardTitle",
                    title,
                    11.5,
                    accent_color,
                    TextAlign::Left,
                    Style::new().height(20.0),
                );
            });

            if let Some(act) = action_label {
                let tag = header_scope.tag_for(act);
                let btn_style = Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .width(16.0)
                    .height(16.0)
                    .border_radius(theme.corner_radius_sm)
                    .background(Color::rgba(0.12, 0.13, 0.16, 0.5))
                    .hover_background(theme.danger);

                let btn_resp = header_scope.container_tagged(
                    "CardActionButton",
                    btn_style,
                    WidgetRole::Button,
                    tag,
                    |btn| {
                        btn.label_styled_passive(
                            "CardActionGlyph",
                            act,
                            12.0,
                            theme.text_secondary,
                            TextAlign::Center,
                            Style::new().width(16.0).height(16.0),
                        );
                    },
                );
                Some(btn_resp.response)
            } else {
                None
            }
        });

        let (action_response, action_clicked) = match resp.inner {
            Some(btn_r) => {
                let clicked = btn_r.clicked();
                (Some(btn_r), clicked)
            }
            None => (None, false),
        };

        CardHeaderResponse {
            response: resp.response,
            action_response,
            action_clicked,
        }
    }

    /// Emits an elevated, bordered card container conforming to flexbox layout with standardized header.
    ///
    /// # Arguments
    /// * `title` - Card heading string.
    /// * `icon` - Optional unicode icon (e.g. `"📐"`, `"📦"`).
    /// * `color` - Optional accent color for title and icon. Defaults to theme accent.
    /// * `f` - Child scope closure emitting the body content.
    pub fn card<R, F>(
        &mut self,
        title: &str,
        icon: Option<&str>,
        color: Option<Color>,
        f: F,
    ) -> CardResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let options = CardOptions {
            icon,
            color,
            action_label: None,
            border_radius: None,
        };
        self.card_with_action_internal("Card", title, options, f)
    }

    /// Emits a card container with an interactive action button (e.g., delete/close) in its header.
    ///
    /// The returned [`CardResponse`] contains `.action_clicked` and provides fluent `.on_action(...)` chaining,
    /// enabling pure declarative action dispatch without intermediate variable plumbing.
    pub fn card_with_action<R, F>(
        &mut self,
        title: &str,
        icon: Option<&str>,
        color: Option<Color>,
        action_label: &str,
        f: F,
    ) -> CardResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let options = CardOptions {
            icon,
            color,
            action_label: Some(action_label),
            border_radius: None,
        };
        self.card_with_action_internal("Card", title, options, f)
    }

    /// Emits a simplified card container with just a title and body.
    pub fn card_simple<R, F>(&mut self, title: &str, f: F) -> CardResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.card(title, None, None, f)
    }

    /// Returns a fluent [`CardBuilder`] to configure icons, accent colors, actions, and custom frames.
    #[inline]
    pub fn card_builder<'s>(&'s mut self, title: impl Into<String>) -> CardBuilder<'s, 'a> {
        CardBuilder::new(self, title)
    }

    fn card_with_action_internal<R, F>(
        &mut self,
        name: impl Into<String>,
        title: &str,
        options: CardOptions<'_>,
        f: F,
    ) -> CardResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let theme = self.tree.theme();
        let radius = options.border_radius.unwrap_or(6.0);
        let card_style = Style::new()
            .flex_col()
            .background(theme.card_bg)
            .border(1.0, theme.card_border)
            .border_radius(radius)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        let card_resp = self.container_named(name, card_style, |card_scope| {
            let header_resp = card_scope.card_header_internal(
                title,
                options.icon,
                options.color,
                options.action_label,
            );
            let inner = card_scope.column(f).inner;
            (header_resp, inner)
        });

        let (header_response, inner) = card_resp.inner;
        CardResponse {
            response: card_resp.response,
            header_response: header_response.response,
            action_clicked: header_response.action_clicked,
            inner,
        }
    }

    /// Emits an elevated, bordered card container allowing custom header and body closures.
    ///
    /// # Arguments
    /// * `header_fn` - Closure rendering the custom card title row.
    /// * `body_fn` - Closure rendering the primary card contents.
    pub fn card_custom<H, B, HR, BR>(&mut self, header_fn: H, body_fn: B) -> InnerResponse<(HR, BR)>
    where
        H: FnOnce(&mut UiScope<'_>) -> HR,
        B: FnOnce(&mut UiScope<'_>) -> BR,
    {
        let theme = self.tree.theme();
        let card_style = Style::new()
            .flex_col()
            .background(theme.card_bg)
            .border(1.0, theme.card_border)
            .border_radius(6.0)
            .padding(8.0)
            .gap(6.0);

        let resp = self.container(card_style, |card_scope| {
            let header_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .padding_insets(Insets::new(0.0, 2.0, 2.0, 2.0));

            let hr = card_scope.container(header_style, header_fn).inner;
            let br = card_scope.column(body_fn).inner;
            (hr, br)
        });
        InnerResponse::new(resp.response, resp.inner)
    }
}