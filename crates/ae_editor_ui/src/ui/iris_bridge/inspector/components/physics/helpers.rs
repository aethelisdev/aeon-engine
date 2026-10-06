// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Common UI Widget and Card Render Helpers
//!
//! Provides reusable helper routines for drawing headers, numeric input pills,
//! dropdown comboboxes, and checkboxes inside Iris UI Inspector component cards.

use crate::ui::iris_bridge::inspector::tags::{
    encode_component_checkbox_tag, encode_component_delete_tag, encode_inspector_dropdown_tag,
    encode_inspector_number_input_tag,
};
use crate::ui::iris_bridge::inspector::types::{
    ComponentCheckboxId, InspectorDropdownId, InspectorNumberInputId,
};
use irisui::prelude::*;

/// Parameters for rendering a component card header.
#[derive(Debug, Clone, Copy)]
pub struct ComponentHeaderProps {
    /// Optional GPU SDF atlas icon coordinates (`[u_min, v_min, u_max, layer]`).
    pub atlas_icon: Option<[f32; 4]>,
    /// Fallback unicode icon when no atlas icon is present.
    pub icon: &'static str,
    /// Human-readable title of the component.
    pub display_title: &'static str,
    /// Color accent for the title and icon.
    pub header_color: Color,
    /// Unique component identifier name.
    pub component_name: &'static str,
}

/// Renders a standardized component inspector card header in a declarative [`UiScope`].
///
/// Features hardware texture atlas quad icon rendering, title typography,
/// and a top-right deletion action button tagged with [`encode_component_delete_tag`]
/// for zero-search direct hit-testing.
///
/// # Arguments
/// * `scope` - Active declarative UI scope for the card container.
/// * `props` - Title, icon coordinates, and visual accent styling configuration.
/// * `is_delete_hovered` - Whether the cursor is currently hovering over the delete action button.
///
/// Returns the allocated [`WidgetId`] of the component deletion button container.
pub fn build_declarative_card_header(
    scope: &mut UiScope<'_>,
    props: ComponentHeaderProps,
    is_delete_hovered: bool,
) -> WidgetId {
    let header_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .height(20.0)
        .padding_insets(Insets::new(0.0, 2.0, 2.0, 2.0));

    let mut del_btn_id = WidgetId::default();

    scope.container_named("CardHeader", header_style, |header| {
        let title_group_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(5.0);

        header.container_named("TitleGroup", title_group_style, |group| {
            if let Some(uv) = props.atlas_icon {
                group.icon_named("CardAtlasIcon", uv, props.header_color, 14.0);
            } else {
                group.label_styled_passive(
                    "CardTextIcon",
                    props.icon,
                    12.0,
                    props.header_color,
                    TextAlign::Left,
                    Style::new().width(15.0).height(20.0),
                );
            }

            group.label_styled_passive(
                "CardTitle",
                props.display_title,
                11.5,
                props.header_color,
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });

        let del_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .width(16.0)
            .height(16.0)
            .border_radius(3.0)
            .background(if is_delete_hovered {
                Color::rgba(0.35, 0.12, 0.15, 0.85)
            } else {
                Color::rgba(0.12, 0.13, 0.16, 0.5)
            });

        let del_tag = encode_component_delete_tag(props.component_name);
        del_btn_id = header.container_tagged(
            "ComponentDeleteBtn",
            del_style,
            WidgetRole::Button,
            del_tag,
            |btn| {
                btn.label_styled_passive(
                    "DeleteGlyph",
                    "×",
                    12.0,
                    if is_delete_hovered {
                        Color::WHITE
                    } else {
                        Color::rgba(0.70, 0.72, 0.78, 1.0)
                    },
                    TextAlign::Center,
                    Style::new().width(16.0).height(16.0),
                );
            },
        );
    });

    del_btn_id
}

/// Parameter descriptor for rendering a declarative combobox property row.
#[derive(Debug, Clone, Copy)]
pub struct DeclarativeComboboxRowParams {
    /// Associated inspector dropdown identifier for semantic tag hit-testing.
    pub dropdown_id: InspectorDropdownId,
    /// Optional property description label (e.g. `Some("Shape:")`).
    pub label: Option<&'static str>,
    /// Width allocated for the description label in physical pixels.
    pub label_w: f32,
    /// Currently selected option string displayed on the combobox pill.
    pub selected_text: &'static str,
    /// Whether the associated dropdown popup menu is currently active.
    pub is_open: bool,
    /// Whether the mouse cursor is currently over the combobox button.
    pub is_hovered: bool,
    /// Width of the combobox trigger button in physical pixels.
    pub combo_w: f32,
}

/// Renders a compact property combobox row with a label and dropdown trigger button in [`UiScope`].
///
/// Automatically uses the hardware texture atlas chevron icons (`ICON_CHEVRON_UP` when open,
/// `ICON_CHEVRON_DOWN` when closed) and tags the button with [`encode_inspector_dropdown_tag`]
/// for O(1) semantic hit-testing.
///
/// # Arguments
/// * `scope` - Active declarative UI scope for the parent row or card.
/// * `params` - Configuration descriptor for label, option text, state, and dimensions.
///
/// Returns the allocated [`WidgetId`] of the combobox button container.
pub fn render_declarative_combobox_row(
    scope: &mut UiScope<'_>,
    params: DeclarativeComboboxRowParams,
) -> WidgetId {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(22.0)
        .gap(6.0);

    let mut combo_id = WidgetId::default();

    scope.container_named("ComboboxRow", row_style, |row| {
        if let Some(lbl) = params.label {
            row.label_styled_passive(
                "ComboLabel",
                lbl,
                11.0,
                Color::rgba(0.620, 0.635, 0.678, 1.0),
                TextAlign::Left,
                Style::new().width(params.label_w).height(22.0),
            );
        }

        let (bg, border) = if params.is_open {
            (
                Color::rgba(0.118, 0.125, 0.145, 1.0),
                Color::rgba(0.353, 0.376, 0.439, 0.95),
            )
        } else if params.is_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
            )
        };

        let pill_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .width(params.combo_w)
            .height(22.0)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);

        let combo_tag = encode_inspector_dropdown_tag(params.dropdown_id);
        combo_id = row.container_tagged(
            "ComboPill",
            pill_style,
            WidgetRole::Button,
            combo_tag,
            |pill| {
                pill.label_styled_passive(
                    "ComboText",
                    params.selected_text,
                    10.5,
                    if params.is_open {
                        Color::WHITE
                    } else {
                        Color::rgba(0.886, 0.894, 0.918, 1.0)
                    },
                    TextAlign::Left,
                    Style::new().flex_grow(1.0),
                );

                let chevron_uv = if params.is_open {
                    crate::ui::iris_bridge::icons::ICON_CHEVRON_UP
                } else {
                    crate::ui::iris_bridge::icons::ICON_CHEVRON_DOWN
                };
                let chevron_color = if params.is_open {
                    Color::WHITE
                } else {
                    Color::rgba(0.70, 0.72, 0.78, 0.9)
                };
                pill.icon_named("ComboChevron", chevron_uv, chevron_color, 9.0);
            },
        );
    });

    combo_id
}

/// Parameter descriptor for rendering a declarative numeric property row.
#[derive(Debug, Clone, Copy)]
pub struct DeclarativeNumericRowParams<'a> {
    /// Associated inspector number input identifier for semantic tag hit-testing.
    pub input_id: InspectorNumberInputId,
    /// Human-readable property title displayed on the left.
    pub label: &'static str,
    /// Floating-point scalar value to display when not actively editing.
    pub val: f32,
    /// Width of the left title label column.
    pub label_w: f32,
    /// Width of the numeric pill container.
    pub box_w: f32,
    /// Optional physical unit suffix string.
    pub unit: Option<&'static str>,
    /// Active keyboard editing buffer and caret state, if currently focused.
    pub edit_state: Option<NumericInputEditState<'a>>,
    /// Whether the mouse cursor is currently hovering over the input pill.
    pub is_hovered: bool,
}

/// Renders a compact numeric property input row in a declarative [`UiScope`] with optional custom display text.
///
/// Supports interactive inline keyboard editing state with text buffer selection
/// and blinking cursor caret, as well as an optional physical unit suffix (e.g. `m/s`).
/// Tags the container box with [`encode_inspector_number_input_tag`] for zero-search direct hit-testing.
///
/// # Arguments
/// * `scope` - Active declarative UI scope.
/// * `params` - Numerical property parameters descriptor.
/// * `custom_text` - Optional custom formatted string to override default scalar display (e.g. `"45°"`, `"50"`).
///
/// Returns the allocated [`WidgetId`] of the numeric input pill container.
pub fn render_declarative_numeric_row_custom(
    scope: &mut UiScope<'_>,
    params: DeclarativeNumericRowParams<'_>,
    custom_text: Option<&str>,
) -> WidgetId {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(22.0)
        .gap(4.0);

    let mut box_id = WidgetId::default();

    scope.container_named("NumericRow", row_style, |row| {
        row.label_styled_passive(
            "NumLbl",
            params.label,
            11.0,
            Color::rgba(0.620, 0.635, 0.678, 1.0),
            TextAlign::Left,
            Style::new().width(params.label_w).height(22.0),
        );

        let is_editing = params.edit_state.is_some();
        let (bg, border_col) = if is_editing {
            (
                Color::rgba(0.180, 0.190, 0.220, 1.0),
                Color::rgba(0.85, 0.88, 0.98, 0.95),
            )
        } else if params.is_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
            )
        };

        let pill_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .width(params.box_w)
            .height(22.0)
            .background(bg)
            .border(1.0, border_col)
            .border_radius(4.0);

        let display_str = if let Some(s) = params.edit_state {
            let buf = s.buffer;
            let cursor = s.cursor_idx.min(buf.len());
            let (left, right) = buf.split_at(cursor);
            if s.is_all_selected {
                buf.to_string()
            } else if s.blink_caret {
                format!("{}|{}", left, right)
            } else {
                buf.to_string()
            }
        } else if let Some(txt) = custom_text {
            txt.to_string()
        } else {
            format!("{:.2}", params.val)
        };

        let input_tag = encode_inspector_number_input_tag(params.input_id);
        box_id = row.container_tagged(
            "NumPillBox",
            pill_style,
            WidgetRole::TextInput,
            input_tag,
            |pill| {
                pill.label_styled_passive(
                    "NumVal",
                    display_str,
                    10.5,
                    if is_editing {
                        Color::WHITE
                    } else if params.is_hovered {
                        Color::rgba(0.95, 0.96, 0.98, 1.0)
                    } else {
                        Color::rgba(0.886, 0.894, 0.918, 1.0)
                    },
                    TextAlign::Center,
                    Style::new().width(params.box_w).height(22.0),
                );
            },
        );

        if let Some(unit_str) = params.unit {
            row.label_styled_passive(
                "NumUnit",
                unit_str,
                11.0,
                Color::rgba(0.620, 0.635, 0.678, 1.0),
                TextAlign::Left,
                Style::new().width(35.0).height(22.0),
            );
        }
    });

    box_id
}

/// Renders a compact numeric property input row in a declarative [`UiScope`].
///
/// Supports interactive inline keyboard editing state with text buffer selection
/// and blinking cursor caret, as well as an optional physical unit suffix (e.g. `m/s`).
///
/// # Arguments
/// * `scope` - Active declarative UI scope.
/// * `params` - Numerical property parameters descriptor.
///
/// Returns the allocated [`WidgetId`] of the numeric input pill container.
pub fn render_declarative_numeric_row(
    scope: &mut UiScope<'_>,
    params: DeclarativeNumericRowParams<'_>,
) -> WidgetId {
    render_declarative_numeric_row_custom(scope, params, None)
}

/// Renders a standardized boolean toggle checkbox row in a declarative [`UiScope`].
///
/// Displays a 14x14 pixel checkbox box with an accent border, checkmark glyph (`✓`),
/// and a companion descriptive label. Tags the checkbox box with [`encode_component_checkbox_tag`]
/// for direct O(1) hit-testing.
///
/// # Arguments
/// * `scope` - Active declarative UI scope.
/// * `cb_id` - Semantic component checkbox identifier.
/// * `label` - Human-readable label displayed next to the checkbox.
/// * `is_checked` - Current boolean state of the property.
/// * `is_hovered` - Whether the mouse cursor is currently hovering over the checkbox.
///
/// Returns the allocated [`WidgetId`] of the checkbox container box.
pub fn render_declarative_checkbox_row(
    scope: &mut UiScope<'_>,
    cb_id: ComponentCheckboxId,
    label: &'static str,
    is_checked: bool,
    is_hovered: bool,
) -> WidgetId {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(20.0)
        .gap(8.0);

    let mut box_id = WidgetId::default();

    scope.container_named("CheckboxRow", row_style, |row| {
        let (bg, border) = if is_checked {
            (
                Color::rgba(0.20, 0.28, 0.38, 1.0),
                Color::rgba(0.40, 0.55, 0.75, 0.95),
            )
        } else if is_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
            )
        };

        let box_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .width(14.0)
            .height(14.0)
            .background(bg)
            .border(1.0, border)
            .border_radius(3.0);

        let cb_tag = encode_component_checkbox_tag(cb_id);
        box_id = row.container_tagged(
            "CheckboxBox",
            box_style,
            WidgetRole::Checkbox,
            cb_tag,
            |box_scope| {
                if is_checked {
                    box_scope.label_styled_passive(
                        "Checkmark",
                        "✓",
                        10.0,
                        Color::WHITE,
                        TextAlign::Center,
                        Style::new().width(14.0).height(14.0),
                    );
                }
            },
        );

        row.label_styled_passive(
            "CheckboxLabel",
            label,
            11.0,
            Color::rgba(0.620, 0.635, 0.678, 1.0),
            TextAlign::Left,
            Style::new().flex_grow(1.0),
        );
    });

    box_id
}