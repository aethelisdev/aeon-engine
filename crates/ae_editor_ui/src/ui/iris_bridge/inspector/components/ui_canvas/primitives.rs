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

use super::super::super::context::{PropComboMapConfig, PropComboRowLayout};
use super::super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::super::tags::encode_inspector_text_input_tag;
use super::super::super::types::{ComponentCategory, InspectorTextInputId};
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
        let Ok(mut p) = ctx.world.get::<&mut ae_core::ecs::UiPanel>(ctx.entity) else {
            return;
        };

        scope.card_panel("UiPanelCard", |card| {
            self.render_header(card);

            ctx.prop_f32(
                card,
                self.component_name(),
                "Border Width:",
                &mut p.border_width,
                0.0..=50.0,
                0.5,
            );

            ctx.prop_f32(
                card,
                self.component_name(),
                "Corner Radius:",
                &mut p.corner_radius,
                0.0..=100.0,
                0.5,
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
        let Ok(mut t) = ctx.world.get::<&mut ae_core::ecs::UiText>(ctx.entity) else {
            return;
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
        } else if t.text.is_empty() {
            "Empty text...".to_string()
        } else {
            t.text.clone()
        };

        let text_col = if is_editing {
            Color::WHITE
        } else if t.text.is_empty() {
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

        scope.card_panel("UiTextCard", |card| {
            self.render_header(card);

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
                    row.theme().text_muted,
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
            ctx.prop_f32(
                card,
                self.component_name(),
                "Font Size:",
                &mut t.font_size,
                6.0..=128.0,
                0.5,
            );

            // Row 3: Text Alignment Dropdown
            ctx.prop_combobox_mapped(
                card,
                self.component_name(),
                PropComboMapConfig::new(
                    "Align:",
                    match t.alignment {
                        UiTextAlignment::Left => 0,
                        UiTextAlignment::Center => 1,
                        UiTextAlignment::Right => 2,
                    },
                    &["Left", "Center", "Right"],
                    PropComboRowLayout::new(52.0, 80.0),
                ),
                &mut t.alignment,
                |idx| match idx {
                    0 => UiTextAlignment::Left,
                    1 => UiTextAlignment::Center,
                    _ => UiTextAlignment::Right,
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
        let Ok(mut pb) = ctx
            .world
            .get::<&mut ae_core::ecs::UiProgressBar>(ctx.entity)
        else {
            return;
        };

        let min = pb.min;
        let max = pb.max;
        let val_range = min..=max;
        let frac = if max > min {
            ((pb.value - min) / (max - min)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        scope.card_panel("UiProgressBarCard", |card| {
            self.render_header(card);

            ctx.prop_f32(
                card,
                self.component_name(),
                "Min:",
                &mut pb.min,
                -100_000.0..=100_000.0,
                1.0,
            );

            ctx.prop_f32(
                card,
                self.component_name(),
                "Max:",
                &mut pb.max,
                -100_000.0..=100_000.0,
                1.0,
            );

            ctx.prop_f32(
                card,
                self.component_name(),
                "Value:",
                &mut pb.value,
                val_range,
                1.0,
            );

            // Visual Mini Progress Bar track + fill
            let track_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .height(10.0)
                .background(card.theme().scrollbar_track)
                .border(1.0, card.theme().card_border)
                .border_radius(card.theme().corner_radius_sm);

            card.container_named("MiniBarTrack", track_style, |track| {
                if frac > 0.001 {
                    let fill_w = ((ctx.card_w - 20.0) * frac).max(2.0);
                    let fill_style = Style::new()
                        .width(fill_w)
                        .height(10.0)
                        .background(track.theme().accent)
                        .border_radius(track.theme().corner_radius_sm);
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
        let Ok(mut b) = ctx.world.get::<&mut ae_core::ecs::UiButton>(ctx.entity) else {
            return;
        };

        scope.card_panel("UiButtonCard", |card| {
            self.render_header(card);

            ctx.prop_checkbox(card, self.component_name(), "Enabled", &mut b.is_enabled);
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

    fn render_card(&self, scope: &mut UiScope<'_>, _ctx: &mut ComponentRenderContext<'_>) {
        let Ok(img) = _ctx.world.get::<&ae_core::ecs::UiImage>(_ctx.entity) else {
            return;
        };

        scope.card_panel("UiImageCard", |card| {
            self.render_header(card);

            let mode_str = match img.slice_mode {
                ae_core::ecs::UiSliceMode::Stretch => "Slice Mode: Stretch",
                ae_core::ecs::UiSliceMode::Fit => "Slice Mode: Fit",
                ae_core::ecs::UiSliceMode::NineSlice => "Slice Mode: NineSlice",
                ae_core::ecs::UiSliceMode::Tile => "Slice Mode: Tile",
            };
            card.label_styled_passive(
                "UiImageMode",
                mode_str,
                11.0,
                card.theme().text_primary,
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiImage::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::types::{InspectorAction, InspectorPanelParams};

    fn create_test_context<'a>(
        world: &'a hecs::World,
        entity: hecs::Entity,
        params: &'a InspectorPanelParams<'a>,
        actions: &'a mut Vec<InspectorAction>,
    ) -> ComponentRenderContext<'a> {
        ComponentRenderContext::with_actions(entity, world, params, actions, 0.0, 0.0, 260.0)
    }

    #[test]
    fn test_ui_panel_render_and_in_place_mutation() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::UiPanel {
            border_width: 2.0,
            corner_radius: 4.0,
            ..Default::default()
        },));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut actions = Vec::new();
        let mut ctx = create_test_context(&world, entity, &params, &mut actions);

        {
            let mut scope = UiScope::new(&mut tree, root);
            UiPanelHandler.render_card(&mut scope, &mut ctx);
        }

        // Verify direct in-place mutation
        {
            let mut p = world.get::<&mut ae_core::ecs::UiPanel>(entity).unwrap();
            p.border_width = 5.0;
            p.corner_radius = 8.0;
        }
        let panel = world.get::<&ae_core::ecs::UiPanel>(entity).unwrap();
        assert_eq!(panel.border_width, 5.0);
        assert_eq!(panel.corner_radius, 8.0);
    }

    #[test]
    fn test_ui_button_toggle_mutation() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::UiButton {
            is_enabled: true,
            ..Default::default()
        },));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut actions = Vec::new();
        let mut ctx = create_test_context(&world, entity, &params, &mut actions);

        {
            let mut scope = UiScope::new(&mut tree, root);
            UiButtonHandler.render_card(&mut scope, &mut ctx);
        }

        {
            let mut b = world.get::<&mut ae_core::ecs::UiButton>(entity).unwrap();
            b.is_enabled = false;
        }
        let btn = world.get::<&ae_core::ecs::UiButton>(entity).unwrap();
        assert!(!btn.is_enabled);
    }
}