// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen UI Primitives Inspector Cards
//!
//! Provides handlers for core UI Designer widgets:
//! - `UiElement`
//! - `UiPanel`
//! - `UiText`
//! - `UiProgressBar`
//! - `UiButton`
//! - `UiImage`

use super::super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::super::types::{
    ComboboxRowParams, CompactNumericRowParams, ComponentCategory, ComponentCheckboxId,
    InspectorDropdownId, InspectorNumberInputId, InspectorTextInputId,
};
use super::super::physics::{
    render_checkbox_row, render_combobox_row, render_component_header, render_numeric_row_compact,
};
use ae_core::ui::UiTextAlignment;
use irisui::prelude::*;

/// Inspector handler for `UiElement`.
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

    fn render_card(
        &self,
        _tree: &mut UiTree,
        _parent_id: WidgetId,
        _ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        0.0
    }

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

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let (border_w, radius) = if let Ok(p) = ctx.world.get::<&ae_core::ecs::UiPanel>(ctx.entity)
        {
            (p.border_width, p.corner_radius)
        } else {
            (1.0, 4.0)
        };

        let padding = 8.0;
        let row_h = 22.0;
        let row_gap = 4.0;
        let card_h = 24.0 + 2.0 * (row_h + row_gap) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("UiPanelCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
                .border(1.0, Color::rgba(0.16, 0.19, 0.26, 0.80))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_id);

        render_component_header(
            tree,
            card_id,
            ctx,
            self.icon(),
            self.display_title(),
            self.header_color(),
            self.component_name(),
        );

        let mut cur_y = ctx.base_y + padding + 22.0;

        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Border Width",
                input_id: InspectorNumberInputId::UiBorderWidth,
                val: border_w,
                row_y: cur_y,
                label_w: 80.0,
                box_w: 60.0,
                unit: Some("px"),
            },
        );
        cur_y += row_h + row_gap;

        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Corner Radius",
                input_id: InspectorNumberInputId::UiCornerRadius,
                val: radius,
                row_y: cur_y,
                label_w: 80.0,
                box_w: 60.0,
                unit: Some("px"),
            },
        );

        card_h
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

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let (txt, font_size, alignment) =
            if let Ok(t) = ctx.world.get::<&ae_core::ecs::UiText>(ctx.entity) {
                (t.text.clone(), t.font_size, t.alignment)
            } else {
                ("Label".to_string(), 14.0, UiTextAlignment::Left)
            };

        let padding = 8.0;
        let row_h = 22.0;
        let row_gap = 4.0;
        let card_h = 24.0 + 3.0 * (row_h + row_gap) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("UiTextCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
                .border(1.0, Color::rgba(0.16, 0.19, 0.26, 0.80))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_id);

        render_component_header(
            tree,
            card_id,
            ctx,
            self.icon(),
            self.display_title(),
            self.header_color(),
            self.component_name(),
        );

        let mut cur_y = ctx.base_y + padding + 22.0;

        // Interactive Text String Input Field
        let lbl_w = 42.0;
        let input_w = (ctx.card_w - padding * 2.0 - lbl_w - 6.0).max(60.0);
        let box_rect = Rect::new(ctx.base_x + padding + lbl_w + 4.0, cur_y, input_w, row_h);

        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("UiTextLabelPrefix");
            node.set_text("Text");
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
            node.computed_rect = Rect::new(ctx.base_x + padding, cur_y, lbl_w, row_h);
        }
        let _ = tree.add_child(card_id, lbl_id);

        let is_editing = matches!(
            ctx.params.active_text_input,
            Some((InspectorTextInputId::UiTextContent, _))
        );
        let is_hovered = box_rect.contains_point(ctx.params.cursor_pos);

        let (bg, border_col) = if is_editing {
            (
                Color::rgba(0.118, 0.125, 0.145, 1.0),
                Color::rgba(0.0, 0.85, 1.0, 0.95),
            )
        } else if is_hovered {
            (
                Color::rgba(0.157, 0.169, 0.200, 1.0),
                Color::rgba(0.235, 0.247, 0.286, 0.95),
            )
        } else {
            (
                Color::rgba(0.125, 0.133, 0.153, 0.98),
                Color::rgba(0.180, 0.192, 0.227, 0.85),
            )
        };

        let box_id = tree.create_node();
        if let Some(node) = tree.get_mut(box_id) {
            node.set_name("UiTextBox");
            node.computed_rect = box_rect;
            node.style = Style::new()
                .background(bg)
                .border(1.0, border_col)
                .border_radius(4.0);
        }
        let _ = tree.add_child(card_id, box_id);

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

        let txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(txt_id) {
            node.set_name("UiTextBoxText");
            node.set_text(display_text);
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_align = TextAlign::Left;
            node.text_color = text_col;
            node.computed_rect =
                Rect::new(box_rect.x + 6.0, box_rect.y, box_rect.width - 12.0, row_h);
        }
        let _ = tree.add_child(box_id, txt_id);

        ctx.targets
            .text_inputs
            .push((InspectorTextInputId::UiTextContent, box_rect, txt));
        cur_y += row_h + row_gap;

        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Font Size",
                input_id: InspectorNumberInputId::UiFontSize,
                val: font_size,
                row_y: cur_y,
                label_w: 80.0,
                box_w: 60.0,
                unit: Some("pt"),
            },
        );
        cur_y += row_h + row_gap;

        let (align_str, _align_idx) = match alignment {
            UiTextAlignment::Left => ("Left", 0),
            UiTextAlignment::Center => ("Center", 1),
            UiTextAlignment::Right => ("Right", 2),
        };

        render_combobox_row(
            tree,
            card_id,
            ctx,
            ComboboxRowParams {
                label: "Align",
                selected_text: align_str,
                dropdown_id: InspectorDropdownId::UiTextAlignment,
                label_w: 52.0,
                row_y: cur_y,
            },
        );

        card_h
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

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 2.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("UiProgressBarCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
                .border(1.0, Color::rgba(0.16, 0.19, 0.26, 0.80))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_id);

        render_component_header(
            tree,
            card_id,
            ctx,
            self.icon(),
            self.display_title(),
            self.header_color(),
            self.component_name(),
        );

        let cur_y = ctx.base_y + padding + 22.0;

        let (val, min, max) = ctx
            .world
            .get::<&ae_core::ecs::UiProgressBar>(ctx.entity)
            .map(|bar| (bar.value, bar.min, bar.max))
            .unwrap_or((50.0, 0.0, 100.0));

        let lbl1_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl1_id) {
            node.set_name("ProgressBarStats");
            node.set_text(format!("Value: {:.0} / {:.0}", val, max));
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.85, 0.88, 0.95, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                cur_y,
                ctx.card_w - padding * 2.0,
                row_h,
            );
        }
        let _ = tree.add_child(card_id, lbl1_id);

        // Visual Mini Progress Bar
        let bar_w = ctx.card_w - padding * 2.0;
        let bar_h = 10.0;
        let bar_rect = Rect::new(ctx.base_x + padding, cur_y + row_h + 2.0, bar_w, bar_h);

        let track_id = tree.create_node();
        if let Some(node) = tree.get_mut(track_id) {
            node.set_name("MiniBarTrack");
            node.computed_rect = bar_rect;
            node.style = Style::new()
                .background(Color::rgba(0.15, 0.18, 0.25, 0.90))
                .border(1.0, Color::rgba(0.25, 0.30, 0.40, 0.70))
                .border_radius(3.0);
        }
        let _ = tree.add_child(card_id, track_id);

        let frac = if max > min {
            ((val - min) / (max - min)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        if frac > 0.001 {
            let fill_w = (bar_w * frac).max(2.0);
            let fill_id = tree.create_node();
            if let Some(node) = tree.get_mut(fill_id) {
                node.set_name("MiniBarFill");
                node.computed_rect = Rect::new(bar_rect.x, bar_rect.y, fill_w, bar_h);
                node.style = Style::new()
                    .background(Color::rgba(0.15, 0.65, 1.0, 0.95))
                    .border_radius(3.0);
            }
            let _ = tree.add_child(track_id, fill_id);
        }

        card_h
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

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let is_enabled = ctx
            .world
            .get::<&ae_core::ecs::UiButton>(ctx.entity)
            .map(|b| b.is_enabled)
            .unwrap_or(true);

        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 1.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("UiButtonCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
                .border(1.0, Color::rgba(0.16, 0.19, 0.26, 0.80))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_id);

        render_component_header(
            tree,
            card_id,
            ctx,
            self.icon(),
            self.display_title(),
            self.header_color(),
            self.component_name(),
        );

        let cur_y = ctx.base_y + padding + 22.0;
        render_checkbox_row(
            tree,
            card_id,
            ctx,
            "Enabled",
            ComponentCheckboxId::UiInteractable,
            is_enabled,
            cur_y,
        );

        card_h
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

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let slice_mode = if let Ok(img) = ctx.world.get::<&ae_core::ecs::UiImage>(ctx.entity) {
            format!("{:?}", img.slice_mode)
        } else {
            "Stretch".to_string()
        };

        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 1.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("UiImageCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
                .border(1.0, Color::rgba(0.16, 0.19, 0.26, 0.80))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_id);

        render_component_header(
            tree,
            card_id,
            ctx,
            self.icon(),
            self.display_title(),
            self.header_color(),
            self.component_name(),
        );

        let cur_y = ctx.base_y + padding + 22.0;
        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("UiImageMode");
            node.set_text(format!("Slice Mode: {}", slice_mode));
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.886, 0.894, 0.918, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                cur_y,
                ctx.card_w - padding * 2.0,
                row_h,
            );
        }
        let _ = tree.add_child(card_id, lbl_id);

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiImage::default());
    }
}