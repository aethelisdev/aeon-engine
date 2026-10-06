// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen UI Primitives Inspector Cards
//!
//! Provides declarative handlers for core UI Designer widgets:
//! - `UiElement`
//! - `UiPanel`
//! - `UiText`
//! - `UiProgressBar`
//! - `UiButton`
//! - `UiImage`

use super::super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::super::tags::encode_inspector_text_input_tag;
use super::super::super::types::{
    ComponentCategory, ComponentCheckboxId, InspectorDropdownId, InspectorNumberInputId,
    InspectorTextInputId,
};
use super::super::physics::helpers::{
    ComponentHeaderProps, DeclarativeComboboxRowParams, DeclarativeNumericRowParams,
    build_declarative_card_header, render_declarative_checkbox_row,
    render_declarative_combobox_row, render_declarative_numeric_row,
};
use ae_core::ui::UiTextAlignment;
use irisui::prelude::*;

/// Inspector handler for `UiElement`.
///
/// Handled as the primary 2D Screen Transform (RectTransform) card at the top of the Inspector.
pub struct UiElementHandler;

impl ComponentInspectorHandler for UiElementHandler {
    fn component_name(&self) -> &'static str {
        "UiElement"
    }

    fn display_title(&self) -> &'static str {
        "2D Screen UI Element"
    }

    fn icon(&self) -> &'static str {
        "📐"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, _world: &hecs::World, _entity: hecs::Entity) -> bool {
        // Rendered as the primary 2D Screen Transform card at the top of the Inspector (replacing 3D Transform)
        false
    }

    fn render_card(&self, _scope: &mut UiScope<'_>, _ctx: &mut ComponentRenderContext<'_>) {}

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiElement::default());
    }
}

/// Inspector handler for `🔲 UiPanel` background container component.
pub struct UiPanelHandler;

impl ComponentInspectorHandler for UiPanelHandler {
    fn component_name(&self) -> &'static str {
        "UiPanel"
    }

    fn display_title(&self) -> &'static str {
        "UI Background Panel"
    }

    fn icon(&self) -> &'static str {
        "🔲"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiPanel>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let (border_w, radius) = if let Ok(p) = ctx.world.get::<&ae_core::ecs::UiPanel>(ctx.entity)
        {
            (p.border_width, p.corner_radius)
        } else {
            (1.0, 4.0)
        };

        let get_edit = |id| {
            ctx.params
                .active_number_input
                .filter(|s| s.id == id)
                .map(|s| s.to_edit_state(ctx.params.blink_caret))
        };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("UiPanelCard", card_style, |card| {
            build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: None,
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                false,
            );

            // Row 1: Border Width
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::UiBorderWidth,
                    label: "Border Width",
                    val: border_w,
                    label_w: 80.0,
                    box_w: 60.0,
                    unit: Some("px"),
                    edit_state: get_edit(InspectorNumberInputId::UiBorderWidth),
                    is_hovered: false,
                },
            );

            // Row 2: Corner Radius
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::UiCornerRadius,
                    label: "Corner Radius",
                    val: radius,
                    label_w: 80.0,
                    box_w: 60.0,
                    unit: Some("px"),
                    edit_state: get_edit(InspectorNumberInputId::UiCornerRadius),
                    is_hovered: false,
                },
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiPanel::default());
    }
}

/// Inspector handler for `🔤 UiText` label component.
pub struct UiTextHandler;

impl ComponentInspectorHandler for UiTextHandler {
    fn component_name(&self) -> &'static str {
        "UiText"
    }

    fn display_title(&self) -> &'static str {
        "UI Text Label"
    }

    fn icon(&self) -> &'static str {
        "🔤"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiText>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let (txt, font_size, alignment) =
            if let Ok(t) = ctx.world.get::<&ae_core::ecs::UiText>(ctx.entity) {
                (t.text.clone(), t.font_size, t.alignment)
            } else {
                ("Label".to_string(), 14.0, UiTextAlignment::Left)
            };

        let get_edit = |id| {
            ctx.params
                .active_number_input
                .filter(|s| s.id == id)
                .map(|s| s.to_edit_state(ctx.params.blink_caret))
        };

        let is_editing = matches!(
            ctx.params.active_text_input,
            Some((InspectorTextInputId::UiTextContent, _))
        );

        let display_text = if is_editing {
            let buf = match ctx.params.active_text_input {
                Some((InspectorTextInputId::UiTextContent, b)) => b,
                _ => "",
            };
            if ctx.params.blink_caret {
                format!("{}|", buf)
            } else {
                buf.to_string()
            }
        } else if txt.is_empty() {
            "Empty text...".to_string()
        } else {
            txt.clone()
        };

        let text_col = if is_editing {
            Color::WHITE
        } else if txt.is_empty() {
            Color::rgba(0.45, 0.48, 0.55, 1.0)
        } else {
            Color::rgba(0.886, 0.894, 0.918, 1.0)
        };

        let (bg, border_col) = if is_editing {
            (
                Color::rgba(0.118, 0.125, 0.145, 1.0),
                Color::rgba(0.0, 0.85, 1.0, 0.95),
            )
        } else {
            (
                Color::rgba(0.125, 0.133, 0.153, 0.98),
                Color::rgba(0.180, 0.192, 0.227, 0.85),
            )
        };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        let align_str = match alignment {
            UiTextAlignment::Left => "Left",
            UiTextAlignment::Center => "Center",
            UiTextAlignment::Right => "Right",
        };

        let is_align_open =
            ctx.params.active_dropdown == Some(InspectorDropdownId::UiTextAlignment);

        scope.container_named("UiTextCard", card_style, |card| {
            build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: None,
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                false,
            );

            // Row 1: Text string interactive box
            let text_row_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .height(22.0)
                .gap(4.0);

            card.container_named("TextRow", text_row_style, |row| {
                row.label_styled_passive(
                    "UiTextLabelPrefix",
                    "Text",
                    11.0,
                    Color::rgba(0.620, 0.635, 0.678, 1.0),
                    TextAlign::Left,
                    Style::new().width(42.0).height(22.0),
                );

                let box_style = Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .flex_grow(1.0)
                    .height(22.0)
                    .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
                    .background(bg)
                    .border(1.0, border_col)
                    .border_radius(4.0);

                let text_tag = encode_inspector_text_input_tag(InspectorTextInputId::UiTextContent);
                row.container_tagged(
                    "UiTextBox",
                    box_style,
                    WidgetRole::TextInput,
                    text_tag,
                    |tb| {
                        tb.label_styled_passive(
                            "UiTextBoxText",
                            &display_text,
                            11.0,
                            text_col,
                            TextAlign::Left,
                            Style::new().flex_grow(1.0).height(22.0),
                        );
                    },
                );
            });

            // Row 2: Font Size
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::UiFontSize,
                    label: "Font Size",
                    val: font_size,
                    label_w: 80.0,
                    box_w: 60.0,
                    unit: Some("pt"),
                    edit_state: get_edit(InspectorNumberInputId::UiFontSize),
                    is_hovered: false,
                },
            );

            // Row 3: Text Alignment Dropdown
            render_declarative_combobox_row(
                card,
                DeclarativeComboboxRowParams {
                    dropdown_id: InspectorDropdownId::UiTextAlignment,
                    label: Some("Align"),
                    label_w: 52.0,
                    selected_text: align_str,
                    is_open: is_align_open,
                    is_hovered: false,
                    combo_w: 80.0,
                },
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiText::default());
    }
}

/// Inspector handler for `📊 UiProgressBar`.
pub struct UiProgressBarHandler;

impl ComponentInspectorHandler for UiProgressBarHandler {
    fn component_name(&self) -> &'static str {
        "UiProgressBar"
    }

    fn display_title(&self) -> &'static str {
        "UI Progress Bar"
    }

    fn icon(&self) -> &'static str {
        "📊"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiProgressBar>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let (val, min, max) = ctx
            .world
            .get::<&ae_core::ecs::UiProgressBar>(ctx.entity)
            .map(|bar| (bar.value, bar.min, bar.max))
            .unwrap_or((50.0, 0.0, 100.0));

        let frac = if max > min {
            ((val - min) / (max - min)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        let mut del_btn_id = WidgetId::default();

        scope.container_named("UiProgressBarCard", card_style, |card| {
            del_btn_id = build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: None,
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                false,
            );

            let stats_text = format!("Value: {:.0} / {:.0}", val, max);
            card.label_styled_passive(
                "ProgressBarStats",
                &stats_text,
                11.0,
                Color::rgba(0.85, 0.88, 0.95, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );

            // Visual Mini Progress Bar track + fill
            let track_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .height(10.0)
                .background(Color::rgba(0.15, 0.18, 0.25, 0.90))
                .border(1.0, Color::rgba(0.25, 0.30, 0.40, 0.70))
                .border_radius(3.0);

            card.container_named("MiniBarTrack", track_style, |track| {
                if frac > 0.001 {
                    let fill_w = ((ctx.card_w - 20.0) * frac).max(2.0);
                    let fill_style = Style::new()
                        .width(fill_w)
                        .height(10.0)
                        .background(Color::rgba(0.15, 0.65, 1.0, 0.95))
                        .border_radius(3.0);
                    track.empty_box_passive_named("MiniBarFill", fill_style);
                }
            });
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiProgressBar::default());
    }
}

/// Inspector handler for `🔘 UiButton`.
pub struct UiButtonHandler;

impl ComponentInspectorHandler for UiButtonHandler {
    fn component_name(&self) -> &'static str {
        "UiButton"
    }

    fn display_title(&self) -> &'static str {
        "Interactive Button"
    }

    fn icon(&self) -> &'static str {
        "🔘"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.40, 0.75, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiButton>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let is_enabled = ctx
            .world
            .get::<&ae_core::ecs::UiButton>(ctx.entity)
            .map(|b| b.is_enabled)
            .unwrap_or(true);

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("UiButtonCard", card_style, |card| {
            build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: None,
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                false,
            );

            render_declarative_checkbox_row(
                card,
                ComponentCheckboxId::UiInteractable,
                "Enabled",
                is_enabled,
                false,
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiButton::default());
    }
}

/// Inspector handler for `🖼️ UiImage` sprite component.
pub struct UiImageHandler;

impl ComponentInspectorHandler for UiImageHandler {
    fn component_name(&self) -> &'static str {
        "UiImage"
    }

    fn display_title(&self) -> &'static str {
        "UI Sprite Image"
    }

    fn icon(&self) -> &'static str {
        "🖼️"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiImage>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let slice_mode = if let Ok(img) = ctx.world.get::<&ae_core::ecs::UiImage>(ctx.entity) {
            format!("{:?}", img.slice_mode)
        } else {
            "Stretch".to_string()
        };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("UiImageCard", card_style, |card| {
            build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: None,
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                false,
            );

            let mode_str = format!("Slice Mode: {}", slice_mode);
            card.label_styled_passive(
                "UiImageMode",
                &mode_str,
                11.0,
                Color::rgba(0.886, 0.894, 0.918, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiImage::default());
    }
}