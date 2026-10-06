// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Scene Inspector panel layout assembly and empty-selection renderer.
//!

use super::add_menu;
use super::appearance;
use super::color_picker_popup;
use super::dropdown_popup;
use super::footer;
use super::header;
use super::registry::{ComponentRenderContext, InspectorRegistry};
use super::transform;
use super::types::InspectorPanelParams;
use super::ui_transform;
use crate::ui::iris_bridge::theme::*;
use irisui::prelude::*;

/// Builds the complete Scene Inspector layout and returns root handle.
pub fn build_inspector_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
) {
    let padding_x = 6.0;
    let card_w = params.panel_rect.width - padding_x * 2.0;

    let mut scope = UiScope::with_tagged_interactions(tree, parent_id, &[], params.hovered_tag);

    let panel_style = Style::new()
        .flex_col()
        .background(ELEVATION_1_PANEL)
        .clip_children(true);

    scope.container_tagged(
        "InspectorPanelRoot",
        panel_style,
        WidgetRole::Default,
        super::tags::TAG_INSPECTOR_PANEL_ROOT,
        |panel_scope| {
            // 1. Check if an entity is selected
            let Some(entity) = params.selected_entity.filter(|e| params.world.contains(*e)) else {
                render_empty_selection_view(panel_scope, params);
                panel_scope.finish_layout(params.panel_rect);
                return;
            };

            // 2. Top Entity Name Header
            header::build_entity_header(panel_scope, entity, params);

            // 3. Scrollable Cards Container (Single Declarative Flex Column)
            let mut ctx = ComponentRenderContext::new(
                entity,
                params.world,
                params,
                params.panel_rect.x + padding_x,
                params.panel_rect.y,
                card_w,
            );

            let container_style = Style::new()
                .flex_col()
                .flex_grow(1.0)
                .gap(6.0)
                .width(card_w)
                .margin_insets(Insets::new(0.0, padding_x, 0.0, padding_x))
                .clip_children(true)
                .scroll_offset_y(params.scroll_y);

            panel_scope.container_named(
                "InspectorCardsContainer",
                container_style,
                |cards_scope| {
                    // 4. Primary Transform / Layout Card
                    let is_ui_element = ctx
                        .world
                        .get::<&ae_core::ecs::UiElement>(ctx.entity)
                        .is_ok();
                    if is_ui_element {
                        ui_transform::build_ui_transform_card(cards_scope, &mut ctx);
                    } else {
                        transform::build_transform_card(cards_scope, &mut ctx);
                        appearance::build_appearance_card(cards_scope, &mut ctx);
                    }

                    // 5. Extensible Registry Component Cards
                    let registry = InspectorRegistry::global();
                    for handler in registry.handlers() {
                        if handler.has_component(ctx.world, ctx.entity) {
                            handler.render_card(cards_scope, &mut ctx);
                        }
                    }
                },
            );

            // 6. Bottom Action Bar (Fixed at bottom)
            footer::build_inspector_footer(panel_scope, params);

            // 7. Single unified layout pass for the entire inspector panel
            panel_scope.finish_layout(params.panel_rect);
        },
    );

    // 8. Cascading `➕ Add Component` Floating Dropdown Menu (Z-Order Top)
    add_menu::build_add_component_menu(tree, parent_id, params);

    // 9. Floating ComboBox Dropdown Popup (Z-Order Topmost)
    dropdown_popup::build_inspector_dropdown_popup(tree, parent_id, params);

    // 10. Floating Color Picker Popup (Z-Order Topmost)
    color_picker_popup::build_color_picker_popup(tree, parent_id, params);
}

/// Renders the empty state placeholder when no entity is selected in the editor,
/// styled identically to the Material & Surface Studio empty card via declarative [`UiScope`].
fn render_empty_selection_view(scope: &mut UiScope<'_>, params: &InspectorPanelParams<'_>) {
    let center_style = Style::new()
        .flex_col()
        .flex_grow(1.0)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .width(params.panel_rect.width)
        .padding_insets(Insets::new(12.0, 12.0, 12.0, 12.0));

    scope.container_named("InspectorEmptyCenter", center_style, |center| {
        let card_style = Style::new()
            .flex_col()
            .width(params.panel_rect.width - 24.0)
            .background(Color::rgba(0.09, 0.10, 0.12, 0.95))
            .border(1.0, Color::rgba(0.18, 0.20, 0.25, 0.80))
            .border_radius(8.0)
            .padding_insets(Insets::new(14.0, 14.0, 14.0, 14.0))
            .gap(4.0);

        center.container_named("EmptySelectionCard", card_style, |card| {
            let icon_row = Style::new()
                .flex_row()
                .justify_content(JustifyContent::Center)
                .height(30.0);
            card.container_named("EmptyIconRow", icon_row, |r| {
                r.icon_named(
                    "EmptyWorldIcon",
                    crate::ui::iris_bridge::icons::ICON_WORLD,
                    Color::rgba(0.0, 0.85, 1.0, 0.65),
                    28.0,
                );
            });

            card.label_styled_passive(
                "EmptySelectionTitle",
                "No Entity Selected",
                12.0,
                Color::rgba(0.90, 0.92, 0.95, 1.0),
                TextAlign::Left,
                Style::new().height(18.0),
            );

            card.label_styled_passive(
                "EmptySelectionDesc1",
                "Select an entity in the hierarchy or viewport",
                10.5,
                Color::rgba(0.55, 0.58, 0.64, 1.0),
                TextAlign::Left,
                Style::new().height(16.0),
            );

            card.label_styled_passive(
                "EmptySelectionDesc2",
                "to inspect and edit components.",
                10.5,
                Color::rgba(0.55, 0.58, 0.64, 1.0),
                TextAlign::Left,
                Style::new().height(16.0),
            );
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_empty_selection_view_renders_material_style_card() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let world = hecs::World::new();

        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 600.0),
            world: &world,
            selected_entity: None,
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            scroll_y: 0.0,
            active_dropdown: None,
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_number_input: None,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
            hovered_tag: None,
        };

        let mut scope = UiScope::new(&mut tree, root);
        render_empty_selection_view(&mut scope, &params);
        scope.finish_layout(params.panel_rect);

        // Find EmptySelectionCard in tree
        let mut card_found = false;
        let mut icon_found = false;
        let mut title_found = false;

        for (_, node) in tree.iter() {
            if node.name.as_deref() == Some("EmptySelectionCard") {
                card_found = true;
                assert_eq!(
                    node.style.background_color,
                    Color::rgba(0.09, 0.10, 0.12, 0.95)
                );
                assert_eq!(node.style.corner_radii.top_left, 8.0);
            }
            if node.name.as_deref() == Some("EmptyWorldIcon") {
                icon_found = true;
                assert_eq!(node.texture_tint, Some(Color::rgba(0.0, 0.85, 1.0, 0.65)));
            }
            if node.name.as_deref() == Some("EmptySelectionTitle") {
                title_found = true;
                assert_eq!(node.text.as_deref(), Some("No Entity Selected"));
            }
        }

        assert!(card_found, "EmptySelectionCard node must be created");
        assert!(icon_found, "EmptyWorldIcon node must be created");
        assert!(title_found, "EmptySelectionTitle node must be created");
    }
}