// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Common UI Widget and Card Render Helpers
//!
//! Provides reusable helper routines for drawing standardized headers
//! inside Iris UI Inspector component cards.

use crate::ui::iris_bridge::inspector::tags::encode_component_delete_tag;
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
        .padding_insets(Insets::new(0.0, 2.0, 0.0, 2.0));

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
            })
            .hover_background(header.theme().danger);

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