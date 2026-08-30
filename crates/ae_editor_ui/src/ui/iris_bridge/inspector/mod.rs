// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector and Component Property Editor
//!
//! Orchestrates the inspection, modification, reflection, and prefab export
//! of selected ECS entities via 100% hardware-accelerated Iris UI GPU SDF.

pub mod add_menu;
pub mod appearance;
pub mod components;
pub mod dropdown_popup;
pub mod events;
pub mod footer;
pub mod header;
pub mod registry;
pub mod transform;
pub mod types;

pub use events::handle_inspector_click;
pub use registry::{ComponentInspectorHandler, ComponentRenderContext, InspectorRegistry};
pub use types::{
    ComponentCategory, ComponentCheckboxId, InspectorAction, InspectorDropdownId,
    InspectorNumberInputId, InspectorPanelParams, InspectorPanelTargets, TransformAxisType,
};

use irisui::prelude::*;

/// Builds the complete Scene Inspector layout and returns root handle.
pub fn build_inspector_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    targets: &mut InspectorPanelTargets,
) {
    targets.transform_reset_btns.clear();
    targets.number_inputs.clear();
    targets.palette_swatches.clear();
    targets.dropdowns.clear();
    targets.checkboxes.clear();
    targets.component_delete_btns.clear();

    let padding_x = 6.0;
    let base_x = params.panel_rect.x + padding_x;
    let card_w = params.panel_rect.width - padding_x * 2.0;

    // Panel Base Container
    let panel_root_id = tree.create_node();
    if let Some(node) = tree.get_mut(panel_root_id) {
        node.set_name("InspectorPanelRoot");
        node.computed_rect = params.panel_rect;
        node.style = Style::new().background(Color::rgba(0.075, 0.078, 0.090, 0.98));
    }
    let _ = tree.add_child(parent_id, panel_root_id);

    // 1. Check if an entity is selected
    let Some(entity) = params.selected_entity else {
        render_empty_selection_view(tree, panel_root_id, params, base_x, card_w);
        return;
    };

    if !params.world.contains(entity) {
        render_empty_selection_view(tree, panel_root_id, params, base_x, card_w);
        return;
    }

    let mut cur_y = params.panel_rect.y + 4.0;

    // 2. Top Entity Name Header
    let (header_h, _) =
        header::build_entity_header(tree, panel_root_id, entity, params, targets, cur_y);
    cur_y += header_h;

    // 3. Scrollable Cards Container
    let footer_h = 32.0;
    let list_h = (params.panel_rect.bottom() - footer_h - cur_y).max(40.0);
    let scroll_rect = Rect::new(base_x, cur_y, card_w, list_h);
    targets.scroll_container_rect = scroll_rect;

    let container_id = tree.create_node();
    if let Some(node) = tree.get_mut(container_id) {
        node.set_name("InspectorCardsContainer");
        node.computed_rect = scroll_rect;
        node.style = Style::new().clip_children(true);
    }
    let _ = tree.add_child(panel_root_id, container_id);

    let mut content_y = cur_y - params.scroll_y;
    let card_gap = 6.0;

    let mut ctx = ComponentRenderContext {
        entity,
        world: params.world,
        params,
        targets,
        base_x,
        base_y: content_y,
        card_w,
    };

    // 4. Transform Card
    ctx.base_y = content_y;
    let t_h = transform::build_transform_card(tree, container_id, &mut ctx);
    content_y += t_h + card_gap;

    // 5. Appearance Card
    ctx.base_y = content_y;
    let a_h = appearance::build_appearance_card(tree, container_id, &mut ctx);
    content_y += a_h + card_gap;

    // 6. Extensible Registry Component Cards
    let registry = InspectorRegistry::global();
    for handler in registry.handlers() {
        if handler.has_component(params.world, entity) {
            ctx.base_y = content_y;
            let comp_h = handler.render_card(tree, container_id, &mut ctx);
            content_y += comp_h + card_gap;
        }
    }

    // 7. Bottom Action Bar (Fixed at bottom)
    footer::build_inspector_footer(tree, panel_root_id, params, targets);

    // 8. Cascading `➕ Add Component` Floating Dropdown Menu (Z-Order Top)
    add_menu::build_add_component_menu(tree, parent_id, params, targets);

    // 9. Floating ComboBox Dropdown Popup (Z-Order Topmost)
    dropdown_popup::build_inspector_dropdown_popup(tree, parent_id, params, targets);
}

/// Renders the empty state placeholder when no entity is selected in the editor.
fn render_empty_selection_view(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    base_x: f32,
    card_w: f32,
) {
    let msg_id = tree.create_node();
    if let Some(node) = tree.get_mut(msg_id) {
        node.set_name("InspectorEmptySelection");
        node.set_text(
            "No Entity Selected\nSelect an entity in the Hierarchy to view and edit components.",
        );
        node.font_size = 11.0;
        node.line_height = 18.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.54, 0.56, 0.60, 0.90);
        node.computed_rect = Rect::new(
            base_x + 10.0,
            params.panel_rect.y + 40.0,
            card_w - 20.0,
            60.0,
        );
    }
    let _ = tree.add_child(parent_id, msg_id);
}