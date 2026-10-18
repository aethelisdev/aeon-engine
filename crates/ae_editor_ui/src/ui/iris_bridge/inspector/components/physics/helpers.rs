// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Common UI Widget and Card Render Helpers
//!
//! Provides reusable helper routines for drawing headers, numeric input pills,
//! dropdown comboboxes, and checkboxes inside Iris UI Inspector component cards.

use crate::ui::iris_bridge::inspector::tags::{
    encode_component_checkbox_tag, encode_component_delete_tag, encode_inspector_dropdown_tag,
};
use crate::ui::iris_bridge::inspector::types::{
    ComponentCheckboxId, InspectorAction, InspectorDropdownId,
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
) {
    let header_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .height(20.0)
        .padding_insets(Insets::new(0.0, 2.0, 2.0, 2.0));

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
            .border_radius(header.theme().corner_radius_sm)
            .background(if is_delete_hovered {
                header.theme().danger
            } else {
                Color::rgba(0.12, 0.13, 0.16, 0.5)
            });

        header.container_tagged(
            "ComponentDeleteBtn",
            del_style,
            WidgetRole::Button,
            encode_component_delete_tag(props.component_name),
            |btn| {
                btn.label_styled_passive(
                    "DeleteGlyph",
                    "×",
                    12.0,
                    if is_delete_hovered {
                        btn.theme().text_primary
                    } else {
                        btn.theme().text_secondary
                    },
                    TextAlign::Center,
                    Style::new().width(16.0).height(16.0),
                );
            },
        );
    });
}

use crate::ui::iris_bridge::icons::{ICON_CHEVRON_DOWN, ICON_CHEVRON_UP};
use crate::ui::iris_bridge::inspector::registry::ComponentRenderContext;

/// Parameter descriptor for rendering an autonomous declarative combobox property row.
#[derive(Debug, Clone, Copy)]
pub struct DeclarativeComboboxRowParams<'a> {
    /// Associated inspector dropdown identifier for semantic tag hit-testing.
    pub dropdown_id: InspectorDropdownId,
    /// Optional property description label (e.g. `Some("Shape:")`).
    pub label: Option<&'a str>,
    /// Width allocated for the description label in physical pixels.
    pub label_w: f32,
    /// Currently selected option string displayed on the combobox pill.
    pub selected_text: &'a str,
    /// Width of the combobox trigger button in physical pixels.
    pub combo_w: f32,
    /// Available selectable option strings for the popup dropdown list.
    pub options: &'a [&'a str],
}

/// Renders a standardized autonomous dropdown combobox row in a declarative [`UiScope`].
///
/// Automatically delegates to the autonomous [`irisui::prelude::ComboBox`] widget which
/// manages open/close popup layer state internally, emitting [`InspectorAction::SelectDropdown`]
/// directly upon option selection.
pub fn render_declarative_combobox_row(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    params: DeclarativeComboboxRowParams<'_>,
) {
    let mut combo = irisui::prelude::ComboBox::new(params.selected_text, params.options)
        .tag(encode_inspector_dropdown_tag(params.dropdown_id))
        .width(params.combo_w)
        .indicator(|pill_scope, is_open| {
            let uv = if is_open {
                ICON_CHEVRON_UP
            } else {
                ICON_CHEVRON_DOWN
            };
            pill_scope.icon_named("ComboChevron", uv, Color::rgba(0.85, 0.88, 0.94, 0.9), 9.0);
        });
    if let Some(lbl) = params.label {
        combo = combo.label(lbl).label_width(params.label_w);
    }
    let resp = scope.add(combo);
    if resp.changed()
        && let Some(idx) = resp.selected_index()
    {
        ctx.push_action(InspectorAction::SelectDropdown(
            ctx.entity,
            params.dropdown_id,
            idx,
        ));
    }
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
) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(20.0)
        .gap(8.0);

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

        row.container_tagged(
            "CheckboxBox",
            box_style,
            WidgetRole::Checkbox,
            encode_component_checkbox_tag(cb_id),
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
            row.theme().text_muted,
            TextAlign::Left,
            Style::new().flex_grow(1.0),
        );
    });
}