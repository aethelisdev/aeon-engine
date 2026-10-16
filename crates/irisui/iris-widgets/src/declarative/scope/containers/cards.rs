// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Themed Card Containers
//!
//! Provides styled card grouping components on [`UiScope`]: titled property cards,
//! custom header cards, and bordered inspector section containers.
//!

use super::super::core::UiScope;
use super::generic::InnerResponse;
use iris_core::{Insets, Style};

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
        let theme = self.tree.theme();
        let card_style = Style::new()
            .flex_col()
            .background(theme.panel_bg)
            .border(1.0, theme.card_bg)
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        let resp = self.container_named(name, card_style, |card_scope| {
            // Header bar
            let header_style = Style::new()
                .flex_row()
                .align_items(iris_core::AlignItems::Center)
                .height(20.0)
                .gap(6.0);

            let title_color = card_scope.theme().text_primary;
            card_scope.container_named("CardHeader", header_style, |header_scope| {
                header_scope.label_styled_passive(
                    "CardHeaderTitle",
                    title,
                    11.5,
                    title_color,
                    iris_core::TextAlign::Left,
                    Style::new().height(20.0),
                );
            });

            // Card body column
            card_scope.column(f).inner
        });
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
            .background(theme.panel_bg)
            .border(1.0, theme.card_bg)
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);
        self.container_named(name, card_style, f)
    }

    /// Emits an elevated, bordered card container conforming to flexbox layout.
    ///
    /// Automatically applies standard dark styling, header label, and places body contents into
    /// a vertical flex column.
    ///
    /// # Arguments
    /// * `title` - Card heading string.
    /// * `f` - Child scope closure emitting the body content.
    pub fn card<R, F>(&mut self, title: &str, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.card_named("Card", title, f)
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
            .background(theme.panel_bg)
            .border(1.0, theme.card_border)
            .border_radius(6.0)
            .padding(8.0)
            .gap(6.0);

        let resp = self.container(card_style, |card_scope| {
            let header_style = Style::new()
                .flex_row()
                .align_items(iris_core::AlignItems::Center)
                .justify_content(iris_core::JustifyContent::SpaceBetween)
                .padding_insets(Insets::new(0.0, 2.0, 2.0, 2.0));

            let hr = card_scope.container(header_style, header_fn).inner;
            let br = card_scope.column(body_fn).inner;
            (hr, br)
        });
        InnerResponse::new(resp.response, resp.inner)
    }
}