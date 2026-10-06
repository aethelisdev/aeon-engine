// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Property Rows & Discrete Inspector Widgets
//!
//! Provides structural property rows, boolean toggle checkboxes, dropdown selection pills,
//! and section divider headers on [`UiScope`].
//!

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::{WidgetResponse, split_label_id};
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetId, WidgetRole,
};

impl<'a> UiScope<'a> {
    /// Emits a standardized horizontal property row with a left-aligned label and a left-aligned control slot.
    ///
    /// Ensures consistent padding, row height, and visual rhythm across editor property inspectors.
    /// If `label` contains `##` (e.g. `"Offset##collider"`), only the prefix (`"Offset"`) is displayed.
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left.
    /// * `f` - Child scope closure emitting the left-aligned control widget(s).
    pub fn property_row<F>(&mut self, label: &str, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        self.property_row_styled(label, None, f)
    }

    /// Emits a standardized horizontal property row with an optional fixed-width label container and left-aligned control slot.
    ///
    /// Setting `label_width` allows multiple property rows across an inspector card to align their control inputs into a neat vertical column.
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left.
    /// * `label_width` - Optional explicit width in physical pixels allocated for the label column.
    /// * `f` - Child scope closure emitting the left-aligned control widget(s).
    pub fn property_row_styled<F>(
        &mut self,
        label: &str,
        label_width: Option<f32>,
        f: F,
    ) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let (visible_label, _) = split_label_id(label);
        let mut row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::FlexStart)
            .height(24.0)
            .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
            .gap(4.0);
        row_style.min_height = Some(24.0);

        self.container_named("PropertyRow", row_style, |row| {
            // Left-aligned label
            let label_id = row.tree.create_node();
            if let Some(node) = row.tree.get_mut(label_id) {
                node.set_name("PropertyLabel");
                node.set_text(visible_label);
                node.font_size = 11.0;
                node.line_height = 14.0;
                node.text_align = TextAlign::Left;
                node.text_color = Color::rgba(0.70, 0.72, 0.78, 1.0);
                let mut label_style = Style::new().flex_shrink(0.0);
                if let Some(w) = label_width {
                    label_style.width = Some(w);
                }
                node.set_style(label_style);
            }
            let _ = row.tree.add_child(row.parent, label_id);

            // Left-aligned control slot container
            let slot_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::FlexStart)
                .flex_grow(1.0)
                .gap(3.0);

            row.container_named("PropertySlot", slot_style, f);
        })
    }

    /// Emits an immediate two-way bound property checkbox row.
    ///
    /// Toggles `*checked` in place when clicked, returning a [`WidgetResponse`]
    /// indicating whether the value was modified (`.changed()`).
    ///
    /// # Arguments
    /// * `label` - Property description displayed on the left (supports `##` separator).
    /// * `checked` - Mutable reference to the bound boolean state.
    pub fn property_checkbox(&mut self, label: &str, checked: &mut bool) -> WidgetResponse {
        let tag = self.tag_for(label);
        let mut response = WidgetResponse::default();

        let row_id = self.property_row(label, |slot| {
            let check_id = slot.tree.create_node();
            if let Some(node) = slot.tree.get_mut(check_id) {
                node.set_name("PropertyCheckbox");
                node.interactive = true;
                node.role = WidgetRole::Checkbox;
                node.tag = tag;
                node.cursor = Some(WidgetCursor::Pointer);
                node.font_size = 12.0;
                node.line_height = 14.0;
                node.text_align = TextAlign::Center;

                let (bg_color, text_color, symbol) = if *checked {
                    (
                        Color::rgba(0.0, 0.65, 0.85, 0.25),
                        Color::rgba(0.0, 0.90, 1.0, 1.0),
                        "✓",
                    )
                } else {
                    (
                        Color::rgba(0.12, 0.13, 0.16, 0.90),
                        Color::rgba(0.40, 0.42, 0.48, 1.0),
                        "",
                    )
                };

                node.set_text(symbol);
                node.text_color = text_color;
                node.set_style(
                    Style::new()
                        .width(18.0)
                        .height(18.0)
                        .background(bg_color)
                        .border(
                            1.0,
                            if *checked {
                                Color::rgba(0.0, 0.80, 1.0, 0.80)
                            } else {
                                Color::rgba(0.24, 0.26, 0.32, 0.80)
                            },
                        )
                        .border_radius(3.0)
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center),
                );
            }
            let _ = slot.tree.add_child(slot.parent, check_id);

            let (clicked, hovered, _) = slot.check_interaction(check_id);
            let mut changed = false;
            if clicked {
                *checked = !*checked;
                changed = true;
            }

            response = WidgetResponse::new(check_id, clicked, hovered, changed);
        });

        if response.id == WidgetId::default() {
            response.id = row_id;
        }
        response
    }

    /// Emits an interactive property dropdown selection trigger row.
    ///
    /// Renders the currently selected option string and returns a [`WidgetResponse`]
    /// indicating whether the combo box trigger was clicked to open a popup menu.
    ///
    /// # Arguments
    /// * `label` - Property description displayed on the left (supports `##` separator).
    /// * `current_option` - Display text of the active selection.
    pub fn property_dropdown(&mut self, label: &str, current_option: &str) -> WidgetResponse {
        let tag = self.tag_for(label);
        let mut response = WidgetResponse::default();

        let row_id = self.property_row(label, |slot| {
            let combo_id = slot.tree.create_node();
            if let Some(node) = slot.tree.get_mut(combo_id) {
                node.set_name("PropertyDropdownTrigger");
                node.interactive = true;
                node.role = WidgetRole::Button;
                node.tag = tag;
                node.cursor = Some(WidgetCursor::Pointer);
                node.set_text(format!("{} ▾", current_option));
                node.font_size = 11.0;
                node.line_height = 14.0;
                node.text_align = TextAlign::Left;
                node.text_color = Color::rgba(0.85, 0.88, 0.94, 1.0);
                let mut trigger_style = Style::new()
                    .height(20.0)
                    .background(Color::rgba(0.12, 0.13, 0.16, 0.95))
                    .border(1.0, Color::rgba(0.24, 0.26, 0.32, 0.80))
                    .border_radius(3.0)
                    .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
                    .align_items(AlignItems::Center);
                trigger_style.min_width = Some(80.0);
                node.set_style(trigger_style);
            }
            let _ = slot.tree.add_child(slot.parent, combo_id);

            let (clicked, hovered, _) = slot.check_interaction(combo_id);
            response = WidgetResponse::new(combo_id, clicked, hovered, false);
        });

        if response.id == WidgetId::default() {
            response.id = row_id;
        }
        response
    }

    /// Emits a clean, uppercase section divider header for structuring inspector cards and panels.
    ///
    /// # Arguments
    /// * `title` - Section heading title string (supports `##` separator).
    pub fn property_section(&mut self, title: &str) -> WidgetId {
        let (visible_title, _) = split_label_id(title);
        let tag = self.tag_for(title);
        let section_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(section_id) {
            node.set_name("PropertySectionHeader");
            node.interactive = true;
            node.tag = tag;
            node.set_text(visible_title);
            node.font_size = 9.5;
            node.line_height = 14.0;
            node.text_align = TextAlign::Left;
            node.text_color = Color::rgba(0.50, 0.53, 0.60, 1.0);
            node.set_style(
                Style::new()
                    .padding_insets(Insets::new(8.0, 4.0, 4.0, 4.0))
                    .border(0.5, Color::rgba(1.0, 1.0, 1.0, 0.05)),
            );
        }
        let _ = self.tree.add_child(self.parent, section_id);
        section_id
    }
}