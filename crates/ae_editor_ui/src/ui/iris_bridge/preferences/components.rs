// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Declarative UI Components
//!
//! Reusable, type-safe, hardware-accelerated declarative building blocks for
//! the Preferences dialog tabs using [`UiScope`].

use super::types::{PreferencesToggleId, encode_section_tag, encode_toggle_tag};
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
    let tag = encode_section_tag(sec_id);
    let mut header_resp = WidgetResponse::default();
    let card_bg = scope.theme().card_bg;
    let card_border = scope.theme().card_border;
    let header_bg = scope.theme().card_header_bg;
    let hover_bg = header_bg.lerp(Color::WHITE, 0.08);
    let radii = if is_collapsed {
        CornerRadii::all(5.0)
    } else {
        CornerRadii::new(5.0, 5.0, 0.0, 0.0)
    };

    scope.container_named(
        "PrefSectionCard",
        Style::new()
            .flex_col()
            .background(card_bg)
            .border(1.0, card_border)
            .border_radius(6.0)
            .margin_insets(Insets::new(0.0, 0.0, 14.0, 0.0)),
        |card| {
            // Section Header Bar
            header_resp = card.container_tagged_resp(
                "PrefSectionHeader",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .height(34.0)
                    .padding_insets(Insets::new(0.0, 12.0, 0.0, 12.0))
                    .background(header_bg)
                    .hover_background(hover_bg)
                    .corner_radii(radii),
                WidgetRole::Button,
                tag,
                |hdr| {
                    let chevron = if is_collapsed { "▶ " } else { "▼ " };
                    hdr.label_styled_passive(
                        "PrefSectionChevron",
                        chevron,
                        10.0,
                        Color::rgba(0.55, 0.58, 0.68, 1.0),
                        TextAlign::Left,
                        Style::new().margin_insets(Insets::new(0.0, 8.0, 0.0, 0.0)),
                    );
                    hdr.label_styled_hover_passive(
                        "PrefSectionTitle",
                        title,
                        12.5,
                        (Color::rgba(0.92, 0.94, 0.98, 1.0), Color::WHITE),
                        TextAlign::Left,
                        Style::new().flex_grow(1.0),
                    );
                },
            );

            // Collapsible Content Body
            if !is_collapsed {
                card.divider(card.theme().card_border);
                card.container_named(
                    "PrefSectionBody",
                    Style::new()
                        .flex_col()
                        .gap(2.0)
                        .padding_insets(Insets::new(8.0, 12.0, 10.0, 12.0)),
                    content,
                );
            }
        },
    );

    header_resp
}

/// Emits an interactive checkbox toggle row.
///
/// Returns the [`WidgetResponse`] of the checkbox container.
pub fn pref_toggle_row(
    scope: &mut UiScope<'_>,
    toggle_id: PreferencesToggleId,
    label: &'static str,
    is_checked: bool,
) -> WidgetResponse {
    let tag = encode_toggle_tag(toggle_id);

    scope.container_tagged_resp(
        "PrefToggleRow",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(26.0)
            .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
            .margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0))
            .border_radius(4.0)
            .background(Color::TRANSPARENT)
            .hover_background(Color::rgba(0.18, 0.20, 0.28, 0.40)),
        WidgetRole::Checkbox,
        tag,
        |row| {
            let box_bg = if is_checked {
                Color::rgba(0.0, 0.70, 0.85, 1.0)
            } else {
                row.theme().button_idle_bg
            };
            let box_border = if is_checked {
                Color::rgba(0.0, 0.85, 1.0, 1.0)
            } else {
                row.theme().button_border
            };

            row.container_named(
                "PrefCheckboxIndicator",
                Style::new()
                    .width(16.0)
                    .height(16.0)
                    .background(box_bg)
                    .border(1.0, box_border)
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .margin_insets(Insets::new(0.0, 8.0, 0.0, 0.0)),
                |ind| {
                    if is_checked {
                        ind.label_styled_passive(
                            "PrefCheckMark",
                            "✓",
                            11.0,
                            Color::rgba(0.05, 0.06, 0.08, 1.0),
                            TextAlign::Center,
                            Style::new(),
                        );
                    }
                },
            );

            // Label text
            row.label_styled_hover_passive(
                "PrefToggleLabel",
                label,
                11.5,
                (Color::rgba(0.80, 0.83, 0.90, 1.0), Color::WHITE),
                TextAlign::Left,
                Style::new().flex_grow(1.0),
            );
        },
    )
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