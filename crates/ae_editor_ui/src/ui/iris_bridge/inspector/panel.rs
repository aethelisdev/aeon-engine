// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Scene Inspector panel layout assembly and empty-selection renderer.

use super::add_menu;
use super::appearance;
use super::color_picker_popup;
use super::footer;
use super::header;
use super::registry::{ComponentRenderContext, InspectorRegistry};
use super::transform;
use super::types::InspectorPanelParams;
use super::ui_transform;
use irisui::prelude::*;

/// Builds the complete Scene Inspector layout and returns any direct [`crate::ui::types::EngineUiAction`] emitted.
pub fn build_inspector_panel(
    scope: &mut UiScope<'_>,
    params: &InspectorPanelParams<'_>,
    actions: &mut Vec<super::types::InspectorAction>,
    saved_swatches: &mut Vec<[f32; 4]>,
    is_color_picker_open: &mut bool,
) -> Option<crate::ui::types::EngineUiAction> {
    let padding_x = 6.0;
    let card_w = params.panel_rect.width - padding_x * 2.0;

    scope
        .panel("InspectorPanelRoot")
        .tagged(super::tags::TAG_INSPECTOR_PANEL_ROOT)
        .bounds(params.panel_rect)
        .background_themed(|t| t.panel_bg)
        .show(|panel_scope| {
            // 1. Check if an entity is selected
            let Some(entity) = params.selected_entity.filter(|e| params.world.contains(*e)) else {
                render_empty_selection_view(panel_scope, params);
                return None;
            };

            // 2. Top Entity Name Header
            let mut name = params.entity_display_name(entity).to_string();
            header::build_entity_header(panel_scope, &mut name, params.panel_rect.width).on_change(
                || {
                    actions.push(super::types::InspectorAction::RenameEntity(
                        entity,
                        name.clone(),
                    ));
                },
            );

            // 3. Scrollable Cards Container (Single Declarative Flex Column)
            let mut ctx = ComponentRenderContext::with_actions(
                entity,
                params.world,
                params,
                actions,
                padding_x,
                0.0,
                card_w,
            );

            panel_scope
                .scroll_area(
                    "InspectorCardsContainer",
                    super::tags::TAG_INSPECTOR_CARDS_CONTAINER,
                    None,
                    |cards_scope| {
                        combine_card_actions(
                            render_primary_cards(
                                cards_scope,
                                &mut ctx,
                                saved_swatches,
                                is_color_picker_open,
                            ),
                            {
                                render_registry_cards(cards_scope, &mut ctx);
                                super::components::hierarchy::draw_parent_card_if_present(
                                    cards_scope,
                                    ctx.world,
                                    ctx.entity,
                                )
                            },
                        )
                    },
                )
                .inner
                .or(footer::build_inspector_footer(panel_scope, params, actions))
        })
        .inner
}

/// Builds top-level floating overlays (Color Picker, Add Component, and ComboBox Dropdowns)
/// directly into the designated overlay or portal tree, returning `(Option<Rect>, Option<EngineUiAction>)`.
pub fn build_inspector_overlays_with_action(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    actions: &mut Vec<super::types::InspectorAction>,
    is_color_picker_open: &mut bool,
) -> (Option<Rect>, Option<crate::ui::types::EngineUiAction>) {
    if params.is_add_menu_open {
        (
            add_menu::build_add_component_menu(tree, parent_id, params, actions),
            None,
        )
    } else if params.is_color_picker_open {
        color_picker_popup::build_color_picker_popup_with_action(
            tree,
            parent_id,
            params,
            is_color_picker_open,
        )
    } else {
        (None, None)
    }
}

/// Builds top-level floating overlays (Color Picker, Add Component, and ComboBox Dropdowns)
/// directly into the designated overlay or portal tree, preventing panel boundary clipping (Mine 2).
///
/// Returns the screen-space bounding rectangle of the constructed overlay if one was active.
pub fn build_inspector_overlays(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    actions: &mut Vec<super::types::InspectorAction>,
    is_color_picker_open: &mut bool,
) -> Option<Rect> {
    build_inspector_overlays_with_action(tree, parent_id, params, actions, is_color_picker_open).0
}

/// Combines two optional UI actions, evaluating both child branches.
#[inline]
pub fn combine_card_actions(
    primary: Option<crate::ui::types::EngineUiAction>,
    hierarchy: Option<crate::ui::types::EngineUiAction>,
) -> Option<crate::ui::types::EngineUiAction> {
    primary.or(hierarchy)
}

/// Renders the primary transform or UI layout card followed by the appearance card.
pub fn render_primary_cards(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    saved_swatches: &mut Vec<[f32; 4]>,
    is_color_picker_open: &mut bool,
) -> Option<crate::ui::types::EngineUiAction> {
    if ctx
        .world
        .get::<&ae_core::ecs::UiElement>(ctx.entity)
        .is_ok()
    {
        ui_transform::build_ui_transform_card(scope, ctx);
        None
    } else {
        transform::build_transform_card(scope, ctx);
        appearance::build_appearance_card(scope, ctx, saved_swatches, is_color_picker_open)
    }
}

/// Renders the dynamically registered non-hierarchy component inspector cards.
pub fn render_registry_cards(scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
    for handler in InspectorRegistry::global().handlers() {
        if handler.category() != super::types::ComponentCategory::Hierarchy
            && handler.has_component(ctx.world, ctx.entity)
        {
            handler.render_card(scope, ctx);
        }
    }
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
            .background(center.theme().card_bg)
            .border(1.0, center.theme().card_border)
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
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut scope = UiScope::new(&mut tree, root);
        render_empty_selection_view(&mut scope, &params);
        scope.finish_layout(params.panel_rect);

        // Verify EmptySelectionCard, EmptyWorldIcon, and EmptySelectionTitle exist in tree
        assert_eq!(
            tree.node_named("EmptySelectionCard")
                .expect("EmptySelectionCard node must be created")
                .style
                .background_color,
            tree.theme().card_bg
        );
        assert_eq!(
            tree.node_named("EmptySelectionCard")
                .expect("EmptySelectionCard node must be created")
                .style
                .corner_radii
                .top_left,
            8.0
        );
        assert_eq!(
            tree.node_named("EmptyWorldIcon")
                .expect("EmptyWorldIcon node must be created")
                .texture_tint,
            Some(Color::rgba(0.0, 0.85, 1.0, 0.65))
        );
        assert_eq!(
            tree.node_named("EmptySelectionTitle")
                .expect("EmptySelectionTitle node must be created")
                .text
                .as_deref(),
            Some("No Entity Selected")
        );
    }
}