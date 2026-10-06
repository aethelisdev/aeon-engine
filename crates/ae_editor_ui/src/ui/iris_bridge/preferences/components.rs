// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Declarative UI Components
//!
//! Reusable, type-safe, hardware-accelerated declarative building blocks for
//! the Preferences dialog tabs using [`UiScope`].

use super::types::{
    PreferencesDropdownId, PreferencesToggleId, encode_dropdown_tag, encode_section_tag,
    encode_toggle_tag,
};
use irisui::prelude::*;

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
        col.divider(Color::rgba(0.20, 0.22, 0.30, 0.70));
        col.empty_box_passive_named("PrefHeadingSpacer", Style::new().height(8.0));
    });
}

/// Emits a collapsible card section with a toggleable header and scoped content body.
pub fn pref_section_card<F>(
    scope: &mut UiScope<'_>,
    sec_id: &'static str,
    title: &'static str,
    is_collapsed: bool,
    hovered_tag: Option<u64>,
    content: F,
) where
    F: FnOnce(&mut UiScope<'_>),
{
    let tag = encode_section_tag(sec_id);
    let is_header_hovered = hovered_tag == Some(tag);

    scope.container_named(
        "PrefSectionCard",
        Style::new()
            .flex_col()
            .background(Color::rgba(0.10, 0.11, 0.14, 0.95))
            .border(1.0, Color::rgba(0.20, 0.23, 0.30, 0.85))
            .border_radius(6.0)
            .margin_insets(Insets::new(0.0, 0.0, 14.0, 0.0)),
        |card| {
            // Section Header Bar
            let header_bg = if is_header_hovered {
                Color::rgba(0.16, 0.18, 0.24, 0.90)
            } else {
                Color::rgba(0.12, 0.13, 0.17, 0.85)
            };

            card.container_tagged(
                "PrefSectionHeader",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .height(34.0)
                    .padding_insets(Insets::new(0.0, 12.0, 0.0, 12.0))
                    .background(header_bg)
                    .border_radius(5.0),
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
                    hdr.label_styled_passive(
                        "PrefSectionTitle",
                        title,
                        12.5,
                        Color::rgba(0.92, 0.94, 0.98, 1.0),
                        TextAlign::Left,
                        Style::new().flex_grow(1.0),
                    );
                },
            );

            // Collapsible Content Body
            if !is_collapsed {
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
}

/// Emits an interactive checkbox toggle row.
pub fn pref_toggle_row(
    scope: &mut UiScope<'_>,
    toggle_id: PreferencesToggleId,
    label: &'static str,
    is_checked: bool,
    hovered_tag: Option<u64>,
) {
    let tag = encode_toggle_tag(toggle_id);
    let is_hovered = hovered_tag == Some(tag);

    scope.container_tagged(
        "PrefToggleRow",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(26.0)
            .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
            .margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0))
            .border_radius(4.0)
            .background(if is_hovered {
                Color::rgba(0.18, 0.20, 0.28, 0.40)
            } else {
                Color::TRANSPARENT
            }),
        WidgetRole::Checkbox,
        tag,
        |row| {
            // Checkbox visual indicator box
            let box_bg = if is_checked {
                Color::rgba(0.0, 0.70, 0.85, 1.0)
            } else if is_hovered {
                Color::rgba(0.18, 0.20, 0.28, 1.0)
            } else {
                Color::rgba(0.11, 0.12, 0.16, 1.0)
            };

            row.container_named(
                "PrefCheckboxIndicator",
                Style::new()
                    .width(16.0)
                    .height(16.0)
                    .background(box_bg)
                    .border(1.0, Color::rgba(0.25, 0.30, 0.42, 1.0))
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
            row.label_styled_passive(
                "PrefToggleLabel",
                label,
                11.5,
                if is_hovered {
                    Color::rgba(1.0, 1.0, 1.0, 1.0)
                } else {
                    Color::rgba(0.80, 0.83, 0.90, 1.0)
                },
                TextAlign::Left,
                Style::new().flex_grow(1.0),
            );
        },
    );
}

/// Emits an interactive dropdown ComboBox row.
pub fn pref_dropdown_row(
    scope: &mut UiScope<'_>,
    dropdown_id: PreferencesDropdownId,
    label: &'static str,
    current_label: &str,
    is_open: bool,
    hovered_tag: Option<u64>,
) {
    let tag = encode_dropdown_tag(dropdown_id);
    let is_hovered = hovered_tag == Some(tag);

    scope.container_named(
        "PrefDropdownRow",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(28.0)
            .margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0)),
        |row| {
            // Label
            row.label_styled_passive(
                "PrefDropdownLabel",
                label,
                11.5,
                Color::rgba(0.78, 0.82, 0.90, 1.0),
                TextAlign::Left,
                Style::new().width(170.0),
            );

            // ComboBox Trigger Button
            let btn_bg = if is_open {
                Color::rgba(0.18, 0.22, 0.32, 0.95)
            } else if is_hovered {
                Color::rgba(0.16, 0.18, 0.24, 0.90)
            } else {
                Color::rgba(0.12, 0.13, 0.17, 0.85)
            };

            let border_color = if is_open {
                Color::rgba(0.0, 0.75, 1.0, 0.90)
            } else {
                Color::rgba(0.22, 0.25, 0.34, 0.70)
            };

            let combo_text = format!("{} ▾", current_label);

            row.container_tagged(
                "PrefComboBox",
                Style::new()
                    .width(180.0)
                    .height(24.0)
                    .background(btn_bg)
                    .border(1.0, border_color)
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0)),
                WidgetRole::Button,
                tag,
                |btn| {
                    btn.label_styled_passive(
                        "PrefComboText",
                        combo_text,
                        11.5,
                        Color::rgba(0.90, 0.92, 0.96, 1.0),
                        TextAlign::Left,
                        Style::new().flex_grow(1.0),
                    );
                },
            );
        },
    );
}