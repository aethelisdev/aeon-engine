// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Themed Card Containers
//!
//! Provides styled card grouping components on [`UiScope`]: titled property cards,
//! custom header cards, and bordered inspector section containers.
//!

use super::super::core::UiScope;
use super::generic::InnerResponse;
use iris_core::{Color, Insets, Style};

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
        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
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

            card_scope.container_named("CardHeader", header_style, |header_scope| {
                header_scope.label_styled_passive(
                    "CardHeaderTitle",
                    title,
                    11.5,
                    Color::rgba(0.886, 0.894, 0.918, 1.0),
                    iris_core::TextAlign::Left,
                    Style::new().height(20.0),
                );
            });

            // Card body column
            card_scope.column(f).inner
        });
        InnerResponse::new(resp.response, resp.inner)
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
        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.082, 0.086, 0.100, 0.95))
            .border(1.0, Color::rgba(0.15, 0.16, 0.20, 0.85))
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