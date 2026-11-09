// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Property Rows & Discrete Inspector Widgets
//!
//! Provides structural property rows, boolean toggle checkboxes, dropdown selection pills,
//! and section divider headers on [`UiScope`].

use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::input::ComboRowLayout;
use crate::declarative::types::{WidgetResponse, split_label_id};
use crate::declarative::widget::{ComboBox, ComboBoxResponse};
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetId, WidgetRole,
};

/// Composite interaction response returned by [`UiScope::prop_combobox_mapped_with_action`].
///
/// Combines the [`ComboBoxResponse`] of the dropdown selection widget with the [`WidgetResponse`]
/// of the adjacent action button (e.g. `"↺ Preset"`), exposing fluent `.on_action(...)` and
/// `.on_change(...)` callback chains without requiring intermediate `let` bindings in view code.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ComboBoxActionResponse {
    /// Interaction response from the dropdown combobox selector.
    pub combo: ComboBoxResponse,
    /// Interaction response from the trailing action button.
    pub action: WidgetResponse,
}

impl ComboBoxActionResponse {
    /// Returns `true` if either a new dropdown option was selected or the trailing action button was clicked.
    #[inline]
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.combo.changed() || self.action.clicked()
    }

    /// Returns `true` if the trailing action button was clicked in the current frame.
    #[inline]
    #[must_use]
    pub const fn action_clicked(&self) -> bool {
        self.action.clicked()
    }

    /// Invokes `f` when the trailing action button is clicked, returning `self` for fluent chaining.
    #[inline]
    pub fn on_action(self, f: impl FnOnce()) -> Self {
        if self.action.clicked() {
            f();
        }
        self
    }

    /// Invokes `f` when either the dropdown selection changes or the trailing action button is clicked.
    #[inline]
    pub fn on_change(self, f: impl FnOnce()) -> Self {
        if self.changed() {
            f();
        }
        self
    }
}

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
    pub fn property_row_styled_with_insets<F>(
        &mut self,
        label: &str,
        label_width: Option<f32>,
        margin_insets: Option<Insets>,
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
        if let Some(insets) = margin_insets {
            row_style.margin = insets;
        }

        let row_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(row_id) {
            node.set_name("PropertyRow");
            node.set_style(row_style);
        }
        let _ = self.tree.add_child(self.parent, row_id);

        let mut row = self.child_scope(row_id);

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
        row_id
    }

    /// Emits a standard property row container with custom label width.
    ///
    /// # Arguments
    /// * `label` - Descriptive property caption displayed on the left.
    /// * `label_width` - Optional explicit width in physical pixels allocated for the label column.
    /// * `f` - Child scope closure emitting the left-aligned control widget(s).
    #[inline]
    pub fn property_row_styled<F>(
        &mut self,
        label: &str,
        label_width: Option<f32>,
        f: F,
    ) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        self.property_row_styled_with_insets(label, label_width, None, f)
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
        self.prop_bool(label, checked)
    }

    /// Emits a declarative boolean checkbox property row modifying `checked` in-place.
    ///
    /// Renders a left-aligned property label and a custom styled checkbox indicator,
    /// toggling `*checked` upon click. Defaults to [`crate::declarative::widget::DEFAULT_PROPERTY_LABEL_WIDTH`].
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left.
    /// * `checked` - Mutable reference to the bound boolean state.
    pub fn prop_bool(&mut self, label: &str, checked: &mut bool) -> WidgetResponse {
        self.prop_bool_styled(label, checked, None)
    }

    /// Emits a declarative boolean checkbox property row with an explicit label column width.
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left.
    /// * `checked` - Mutable reference to the bound boolean state.
    /// * `label_width` - Optional column width in physical pixels allocated for the label column.
    pub fn prop_bool_styled(
        &mut self,
        label: &str,
        checked: &mut bool,
        label_width: Option<f32>,
    ) -> WidgetResponse {
        let tag = self.tag_for(label);
        let mut response = WidgetResponse::default();
        let mut slot_executed = false;

        let row_id = self.property_row_styled(label, label_width, |slot| {
            response = slot.checkbox_tagged("", tag, checked);
            slot_executed = true;
        });

        if !slot_executed {
            response = self.response(row_id);
        }
        response
    }

    /// Emits a declarative boolean checkbox property row modifying `checked` in-place.
    ///
    /// Alias for [`UiScope::prop_bool`].
    pub fn prop_checkbox(&mut self, label: &str, checked: &mut bool) -> WidgetResponse {
        self.prop_bool(label, checked)
    }

    /// Emits a declarative boolean checkbox property row with an explicit label column width.
    ///
    /// Alias for [`UiScope::prop_bool_styled`].
    pub fn prop_checkbox_styled(
        &mut self,
        label: &str,
        checked: &mut bool,
        label_width: Option<f32>,
    ) -> WidgetResponse {
        self.prop_bool_styled(label, checked, label_width)
    }

    /// Emits a declarative two-way bound dropdown combobox property row modifying `selected` index in-place.
    ///
    /// Renders a left-aligned property label defaulting to [`crate::declarative::widget::DEFAULT_PROPERTY_LABEL_WIDTH`], an interactive
    /// selection pill displaying the current option text, and autonomously manages the elevated [`iris_core::UiLayer::Popup`]
    /// dropdown menu. Mutates `*selected = idx` immediately upon selection and returns a [`ComboBoxResponse`].
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left (supports `##` separator).
    /// * `selected` - Mutable reference to the zero-based active option index modified in-place upon selection.
    /// * `options` - List of selectable option string labels.
    pub fn prop_combobox(
        &mut self,
        label: &str,
        selected: &mut usize,
        options: &[&str],
    ) -> ComboBoxResponse {
        self.prop_combobox_styled(label, selected, options, None, None)
    }

    /// Emits a declarative two-way bound dropdown combobox property row with custom column and pill widths.
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left (supports `##` separator).
    /// * `selected` - Mutable reference to the zero-based active option index modified in-place upon selection.
    /// * `options` - List of selectable option string labels.
    /// * `label_width` - Optional column width in physical pixels allocated for the label.
    /// * `combo_width` - Optional width in physical pixels for the dropdown pill button.
    pub fn prop_combobox_styled(
        &mut self,
        label: &str,
        selected: &mut usize,
        options: &[&str],
        label_width: Option<f32>,
        combo_width: Option<f32>,
    ) -> ComboBoxResponse {
        let selected_text = options.get(*selected).copied().unwrap_or("");
        let mut combo = ComboBox::new(selected_text, options).label(label);
        if let Some(lw) = label_width {
            combo = combo.label_width(lw);
        }
        if let Some(cw) = combo_width {
            combo = combo.width(cw);
        }
        let resp = self.add(combo);
        if let Some(idx) = resp.selected_index() {
            *selected = idx;
        }
        resp
    }

    /// Emits a declarative two-way bound dropdown combobox property row mapped to an arbitrary domain value or enum.
    ///
    /// Modifies `*target = map_fn(idx)` in-place when a new option item is selected, eliminating imperative
    /// `let` plumbing and variable unpacking in UI view callers. Defaults to standard [`ComboRowLayout::default`].
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left (supports `##` separator).
    /// * `current_idx` - Current zero-based index of the active option.
    /// * `options` - List of selectable option string labels.
    /// * `target` - Target domain value or enum modified in-place upon selection.
    /// * `map_fn` - Closure mapping the newly selected index to the domain target value.
    pub fn prop_combobox_mapped<T>(
        &mut self,
        label: &str,
        current_idx: usize,
        options: &[&str],
        target: &mut T,
        map_fn: impl FnOnce(usize) -> T,
    ) -> ComboBoxResponse {
        self.prop_combobox_mapped_styled(
            label,
            current_idx,
            options,
            ComboRowLayout::default(),
            target,
            map_fn,
        )
    }

    /// Emits a declarative two-way bound dropdown combobox property row mapped to an arbitrary domain value or enum with custom layout.
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left (supports `##` separator).
    /// * `current_idx` - Current zero-based index of the active option.
    /// * `options` - List of selectable option string labels.
    /// * `layout` - Column width layout for the label and pill button ([`ComboRowLayout`]).
    /// * `target` - Target domain value or enum modified in-place upon selection.
    /// * `map_fn` - Closure mapping the newly selected index to the domain target value.
    pub fn prop_combobox_mapped_styled<T>(
        &mut self,
        label: &str,
        current_idx: usize,
        options: &[&str],
        layout: ComboRowLayout,
        target: &mut T,
        map_fn: impl FnOnce(usize) -> T,
    ) -> ComboBoxResponse {
        let mut idx = current_idx;
        let resp = self.prop_combobox_styled(
            label,
            &mut idx,
            options,
            Some(layout.label_width),
            Some(layout.button_width),
        );
        if resp.changed() && idx != current_idx {
            *target = map_fn(idx);
        }
        resp
    }

    /// Emits a declarative two-way bound dropdown combobox property row paired with a trailing action button.
    ///
    /// Renders a horizontal row (`"SurfaceTypeRow"`, height `22.0`, gap `4.0`) containing the mapped
    /// dropdown combobox alongside an interactive action button (`"PresetBtn"`, width `62.0`, height `22.0`)
    /// displaying `action_label`. Returns a [`ComboBoxActionResponse`] supporting fluent `.on_action(...)`
    /// and `.on_change(...)` callback chains without requiring intermediate `let` bindings in view callers.
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left (supports `##` separator).
    /// * `current_idx` - Current zero-based index of the active option.
    /// * `options` - List of selectable option string labels.
    /// * `action_label` - Display text for the trailing action button (e.g. `"↺ Preset"`).
    /// * `target` - Target domain value or enum modified in-place upon dropdown selection.
    /// * `map_fn` - Closure mapping the newly selected index to the domain target value.
    pub fn prop_combobox_mapped_with_action<T>(
        &mut self,
        label: &str,
        current_idx: usize,
        options: &[&str],
        action_label: &str,
        target: &mut T,
        map_fn: impl FnOnce(usize) -> T,
    ) -> ComboBoxActionResponse {
        let row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(22.0)
            .gap(4.0);

        let mut combo_resp = ComboBoxResponse::default();
        let mut action_resp = WidgetResponse::default();

        self.container_named("SurfaceTypeRow", row_style, |row| {
            combo_resp = row.prop_combobox_mapped_styled(
                label,
                current_idx,
                options,
                ComboRowLayout::default(),
                target,
                map_fn,
            );

            let preset_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .width(62.0)
                .height(22.0)
                .background(Color::rgba(0.16, 0.17, 0.20, 0.95))
                .border(1.0, Color::rgba(0.24, 0.26, 0.30, 0.85))
                .border_radius(4.0);

            let action_tag = row.tag_for(action_label);
            let preset_container = row.container_tagged(
                "PresetBtn",
                preset_style,
                WidgetRole::Button,
                action_tag,
                |btn| {
                    btn.label_styled_passive(
                        "PresetText",
                        action_label,
                        10.5,
                        btn.theme().text_secondary,
                        TextAlign::Center,
                        Style::new().width(62.0).height(22.0),
                    );
                },
            );
            action_resp = preset_container.response;
        });

        ComboBoxActionResponse {
            combo: combo_resp,
            action: action_resp,
        }
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
        let mut slot_executed = false;

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
            slot_executed = true;
        });

        if !slot_executed {
            response = self.response(row_id);
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