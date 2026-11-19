// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Declarative UI Components
//!
//! Reusable, type-safe, hardware-accelerated declarative building blocks for
//! the Preferences dialog tabs using [`UiScope`].

use super::types::encode_section_tag;
use irisui::prelude::*;
use irisui::widgets::declarative::ComboBoxResponse;
use irisui::widgets::declarative::scope::WrappedLabelDescriptor;

/// Emits the top section heading with title, descriptive subtitle, and horizontal rule.
pub fn pref_heading(scope: &mut UiScope<'_>, title: &'static str, subtitle: &'static str) {
    scope.column(|col| {
        col.label_styled_passive(
            "PrefHeading",
            title,
            17.0,
            Color::rgba(1.0, 1.0, 1.0, 1.0),
            TextAlign::Left,
            Style::new().margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
        );
        col.label_styled_passive_wrapped(
            "PrefSubtitle",
            subtitle,
            WrappedLabelDescriptor::new(
                11.5,
                Color::rgba(0.65, 0.68, 0.76, 1.0),
                Style::new()
                    .width(540.0)
                    .margin_insets(Insets::new(0.0, 0.0, 10.0, 0.0)),
            ),
        );
        col.divider(col.theme().divider);
        col.empty_box_passive_named("PrefHeadingSpacer", Style::new().height(8.0));
    });
}

/// Emits a collapsible card section with a toggleable header and scoped content body.
///
/// Returns the [`WidgetResponse`] of the section header button.
pub fn pref_section_card<F>(
    scope: &mut UiScope<'_>,
    sec_id: &'static str,
    title: &'static str,
    is_collapsed: bool,
    content: F,
) -> WidgetResponse
where
    F: FnOnce(&mut UiScope<'_>),
{
    scope
        .collapsing_card_builder(title)
        .tag(encode_section_tag(sec_id))
        .default_open(!is_collapsed)
        .show(content)
        .header_response
}

/// Emits an autonomous declarative combobox row inside the Preferences dialog using [`irisui::prelude::ComboBox`].
///
/// Binds label, selection text, options, and returns the [`irisui::prelude::ComboBoxResponse`] containing interaction
/// signals and selected index.
pub fn pref_combobox<'a>(
    scope: &mut UiScope<'_>,
    label: &'static str,
    selected_text: &'a str,
    options: &'a [&'a str],
) -> ComboBoxResponse {
    scope.add(
        irisui::prelude::ComboBox::new(selected_text, options)
            .label(label)
            .label_width(170.0)
            .width(180.0),
    )
}