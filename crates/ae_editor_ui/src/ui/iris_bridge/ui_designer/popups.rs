// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Dropdown Popups
//!
//! Renders the Aspect Ratio selector dropdown and the `➕ Add Element` palette popup
//! with fixed column alignments, elevated borders, and hover feedback.
//!

use super::types::{
    CanvasAspectRatio, UiDesignerPanelParams, UiDesignerPanelTargets, UiElementType,
};
use irisui::prelude::*;

/// Builds the Aspect Ratio dropdown menu popup if open.
pub fn build_aspect_ratio_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &UiDesignerPanelParams<'_>,
    targets: &mut UiDesignerPanelTargets,
) {
    if !params.is_aspect_dropdown_open {
        return;
    }

    let anchor = match targets.btn_aspect {
        Some(r) => r,
        None => return,
    };

    let popup_w = 160.0;
    let item_h = 24.0;
    let options = [
        CanvasAspectRatio::Ratio16x9,
        CanvasAspectRatio::Ratio16x10,
        CanvasAspectRatio::Ratio4x3,
        CanvasAspectRatio::Ratio21x9,
    ];
    let popup_h = (options.len() as f32) * item_h + 8.0;

    let popup_rect = Rect::new(anchor.x, anchor.bottom() + 4.0, popup_w, popup_h);
    targets.aspect_popup_rect = Some(popup_rect);

    let popup_id = tree.create_node();
    if let Some(node) = tree.get_mut(popup_id) {
        node.set_name("AspectRatioPopup");
        node.computed_rect = popup_rect;
        node.style = Style::new()
            .background(Color::rgba(0.090, 0.095, 0.110, 0.98))
            .border(1.0, Color::rgba(0.0, 0.70, 0.90, 0.95))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, popup_id);

    for (idx, opt) in options.iter().enumerate() {
        let itm_y = popup_rect.y + 4.0 + (idx as f32) * item_h;
        let itm_rect = Rect::new(popup_rect.x + 4.0, itm_y, popup_w - 8.0, item_h);
        targets.aspect_dropdown_options.push((*opt, itm_rect));

        let is_selected = params.state.aspect_ratio == *opt;
        let is_hovered = itm_rect.contains_point(params.cursor_pos);

        let (bg, txt_col) = if is_selected {
            (
                Color::rgba(0.0, 0.35, 0.48, 0.95),
                Color::rgba(1.0, 1.0, 1.0, 1.0),
            )
        } else if is_hovered {
            (
                Color::rgba(0.16, 0.18, 0.22, 0.95),
                Color::rgba(0.90, 0.92, 0.96, 1.0),
            )
        } else {
            (
                Color::rgba(0.0, 0.0, 0.0, 0.0),
                Color::rgba(0.75, 0.78, 0.85, 1.0),
            )
        };

        let itm_id = tree.create_node();
        if let Some(node) = tree.get_mut(itm_id) {
            node.set_name("AspectOptionItem");
            node.computed_rect = itm_rect;
            node.style = Style::new().background(bg).border_radius(3.0);
        }
        let _ = tree.add_child(popup_id, itm_id);

        let txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(txt_id) {
            node.set_name("AspectOptionText");
            node.set_text(opt.label());
            node.font_size = 10.5;
            node.line_height = item_h;
            node.text_color = txt_col;
            node.computed_rect =
                Rect::new(itm_rect.x + 8.0, itm_rect.y, itm_rect.width - 12.0, item_h);
        }
        let _ = tree.add_child(itm_id, txt_id);
    }
}

/// Builds the `➕ Add Element` palette popup if open.
pub fn build_add_element_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &UiDesignerPanelParams<'_>,
    targets: &mut UiDesignerPanelTargets,
) {
    if !params.is_add_menu_open {
        return;
    }

    let anchor = match targets.btn_add_element {
        Some(r) => r,
        None => return,
    };

    let popup_w = 200.0;
    let item_h = 24.0;
    let elements = [
        UiElementType::Panel,
        UiElementType::ProgressBar,
        UiElementType::Text,
        UiElementType::Button,
        UiElementType::Image,
        UiElementType::Slider,
        UiElementType::Checkbox,
        UiElementType::TextInput,
        UiElementType::HealthBar,
        UiElementType::ScoreDisplay,
    ];
    let popup_h = (elements.len() as f32) * item_h + 8.0;

    let popup_rect = Rect::new(
        (anchor.right() - popup_w).max(params.panel_rect.x + 8.0),
        anchor.bottom() + 4.0,
        popup_w,
        popup_h,
    );
    targets.add_popup_rect = Some(popup_rect);

    let popup_id = tree.create_node();
    if let Some(node) = tree.get_mut(popup_id) {
        node.set_name("AddElementPopup");
        node.computed_rect = popup_rect;
        node.style = Style::new()
            .background(Color::rgba(0.090, 0.095, 0.110, 0.98))
            .border(1.0, Color::rgba(0.0, 0.70, 0.90, 0.95))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, popup_id);

    for (idx, elem) in elements.iter().enumerate() {
        let itm_y = popup_rect.y + 4.0 + (idx as f32) * item_h;
        let itm_rect = Rect::new(popup_rect.x + 4.0, itm_y, popup_w - 8.0, item_h);
        targets.add_menu_options.push((*elem, itm_rect));

        let is_hovered = itm_rect.contains_point(params.cursor_pos);
        let (bg, txt_col) = if is_hovered {
            (
                Color::rgba(0.0, 0.32, 0.44, 0.95),
                Color::rgba(1.0, 1.0, 1.0, 1.0),
            )
        } else {
            (
                Color::rgba(0.0, 0.0, 0.0, 0.0),
                Color::rgba(0.80, 0.83, 0.89, 1.0),
            )
        };

        let itm_id = tree.create_node();
        if let Some(node) = tree.get_mut(itm_id) {
            node.set_name("AddElementOption");
            node.computed_rect = itm_rect;
            node.style = Style::new().background(bg).border_radius(3.0);
        }
        let _ = tree.add_child(popup_id, itm_id);

        // Icon + Label
        let icon_str = elem.icon();
        let label_str = elem.label();

        let icon_id = tree.create_node();
        if let Some(node) = tree.get_mut(icon_id) {
            node.set_name("AddOptionIcon");
            node.set_text(icon_str);
            node.font_size = 11.0;
            node.line_height = item_h;
            node.text_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
            node.computed_rect = Rect::new(itm_rect.x + 6.0, itm_rect.y, 18.0, item_h);
        }
        let _ = tree.add_child(itm_id, icon_id);

        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("AddOptionLabel");
            node.set_text(label_str);
            node.font_size = 10.5;
            node.line_height = item_h;
            node.text_color = txt_col;
            node.computed_rect =
                Rect::new(itm_rect.x + 28.0, itm_rect.y, itm_rect.width - 32.0, item_h);
        }
        let _ = tree.add_child(itm_id, lbl_id);
    }
}