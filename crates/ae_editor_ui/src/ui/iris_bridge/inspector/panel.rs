// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Scene Inspector panel layout assembly and empty-selection renderer.

use super::add_menu;
use super::appearance;
use super::color_picker_popup;
use super::footer;
use super::header;
use super::registry::{ComponentInspectorHandler, ComponentRenderContext, InspectorRegistry};
use super::transform;
use super::types::InspectorPanelParams;
use super::ui_transform;
use irisui::prelude::*;

/// Semantic tag for the Inspector root scroll panel container.
pub const TAG_INSPECTOR_PANEL_ROOT: u64 = 0xDAFF_0000;

/// Semantic tag for the Inspector scrollable cards container viewport.
pub const TAG_INSPECTOR_CARDS_CONTAINER: u64 = 0xDAFF_0001;

/// Builds the complete Scene Inspector layout and returns any direct [`crate::ui::types::EngineUiAction`] emitted.
pub fn build_inspector_panel(
    scope: &mut UiScope<'_>,
    params: &InspectorPanelParams<'_>,
    session: &mut super::types::InspectorSessionRefs<'_>,
) -> Option<crate::ui::types::EngineUiAction> {
    scope
        .panel("InspectorPanelRoot")
        .tagged(TAG_INSPECTOR_PANEL_ROOT)
        .bounds(params.panel_rect)
        .background_themed(|t| t.panel_bg)
        .show(
            |panel_scope| match params.selected_entity.filter(|e| params.world.contains(*e)) {
                Some(entity) => render_inspected_entity(panel_scope, entity, params, session),
                None => {
                    render_empty_selection_view(panel_scope, params);
                    None
                }
            },
        )
        .inner
}

/// Renders the full inspected entity view combining header, scrollable cards, and footer controls.
fn render_inspected_entity(
    scope: &mut UiScope<'_>,
    entity: hecs::Entity,
    params: &InspectorPanelParams<'_>,
    session: &mut super::types::InspectorSessionRefs<'_>,
) -> Option<crate::ui::types::EngineUiAction> {
    combine_card_actions(
        header::build_entity_header_modify_action(
            scope,
            entity,
            &params.entity_display_name(entity),
            session.inspected_name,
            params.panel_rect.width,
        ),
        scope
            .scroll_area(
                "InspectorCardsContainer",
                TAG_INSPECTOR_CARDS_CONTAINER,
                None,
                |cards_scope| {
                    render_inspector_cards(
                        cards_scope,
                        entity,
                        params,
                        session.saved_swatches,
                        session.is_color_picker_open,
                    )
                },
            )
            .inner
            .or_else(|| {
                footer::build_inspector_footer(
                    scope,
                    params,
                    session.is_add_menu_open,
                    session.active_submenu,
                )
            }),
    )
}

/// Renders all registered and primary component inspection cards in a declarative flex column.
fn render_inspector_cards(
    scope: &mut UiScope<'_>,
    entity: hecs::Entity,
    params: &InspectorPanelParams<'_>,
    saved_swatches: &mut Vec<[f32; 4]>,
    is_color_picker_open: &mut bool,
) -> Option<crate::ui::types::EngineUiAction> {
    render_context_cards(
        scope,
        &mut ComponentRenderContext::new(
            entity,
            params.world,
            params,
            6.0,
            0.0,
            params.panel_rect.width - 12.0,
        ),
        saved_swatches,
        is_color_picker_open,
    )
}

/// Evaluates primary cards followed by dynamic registry cards for the active rendering context.
fn render_context_cards(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    saved_swatches: &mut Vec<[f32; 4]>,
    is_color_picker_open: &mut bool,
) -> Option<crate::ui::types::EngineUiAction> {
    combine_card_actions(
        render_primary_cards(scope, ctx, saved_swatches, is_color_picker_open),
        render_registry_cards(scope, ctx).or_else(|| {
            super::components::hierarchy::draw_parent_card_if_present(scope, ctx.world, ctx.entity)
        }),
    )
}

/// Builds top-level floating overlays (Color Picker, Add Component, and ComboBox Dropdowns)
/// directly into the designated [`UiScope`] overlay or portal view, returning `(Option<Rect>, Option<EngineUiAction>)`.
pub fn build_inspector_overlays_with_action(
    scope: &mut UiScope<'_>,
    params: &InspectorPanelParams<'_>,
    is_color_picker_open: &mut bool,
    is_add_menu_open: &mut bool,
    active_submenu: &mut Option<super::types::ComponentCategory>,
) -> (Option<Rect>, Option<crate::ui::types::EngineUiAction>) {
    if *is_add_menu_open {
        add_menu::draw_add_component_menu(scope, params, is_add_menu_open, active_submenu)
    } else if *is_color_picker_open {
        color_picker_popup::build_color_picker_popup_with_action(
            scope,
            params,
            is_color_picker_open,
        )
    } else {
        (None, None)
    }
}

/// Builds top-level floating overlays (Color Picker, Add Component, and ComboBox Dropdowns)
/// directly into the designated [`UiScope`] overlay or portal view, preventing panel boundary clipping (Mine 2).
///
/// Returns the screen-space bounding rectangle of the constructed overlay if one was active.
pub fn build_inspector_overlays(
    scope: &mut UiScope<'_>,
    params: &InspectorPanelParams<'_>,
    is_color_picker_open: &mut bool,
    is_add_menu_open: &mut bool,
    active_submenu: &mut Option<super::types::ComponentCategory>,
) -> Option<Rect> {
    build_inspector_overlays_with_action(
        scope,
        params,
        is_color_picker_open,
        is_add_menu_open,
        active_submenu,
    )
    .0
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
        header::component_delete_action(scope, ctx.entity, ui_transform::UI_ELEMENT_COMPONENT_NAME)
    } else {
        transform::build_transform_card(scope, ctx);
        appearance::build_appearance_card(scope, ctx, saved_swatches, is_color_picker_open)
    }
}

/// Renders the dynamically registered non-hierarchy component inspector cards and returns the
/// first [`crate::ui::types::EngineUiAction::RemoveComponent`] whose card delete button was
/// clicked in this frame.
///
/// Every visible card is always rendered: `Option::or` evaluates its argument eagerly, so the
/// fold never short-circuits the remaining cards after an action has been found.
pub fn render_registry_cards(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
) -> Option<crate::ui::types::EngineUiAction> {
    InspectorRegistry::global()
        .handlers()
        .iter()
        .fold(None, |action, handler| {
            action.or(render_registry_card(scope, ctx, handler.as_ref()))
        })
}

/// Renders one registry card when its component is present on the inspected entity (hierarchy
/// cards are drawn separately), then maps a click on its delete button into `RemoveComponent`.
fn render_registry_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    handler: &dyn ComponentInspectorHandler,
) -> Option<crate::ui::types::EngineUiAction> {
    (handler.category() != super::types::ComponentCategory::Hierarchy
        && handler.has_component(ctx.world, ctx.entity))
    .then(|| handler.render_card(scope, ctx))
    .and_then(|()| header::component_delete_action(scope, ctx.entity, handler.component_name()))
}

/// Renders the empty state placeholder when no entity is selected in the editor,
/// styled identically to the Material & Surface Studio empty card via declarative [`UiScope`].
fn render_empty_selection_view(scope: &mut UiScope<'_>, params: &InspectorPanelParams<'_>) {
    scope.container_named(
        "InspectorEmptyCenter",
        Style::new()
            .flex_col()
            .flex_grow(1.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .width(params.panel_rect.width)
            .padding_insets(Insets::new(12.0, 12.0, 12.0, 12.0)),
        |center| {
            center.container_named(
                "EmptySelectionCard",
                Style::new()
                    .flex_col()
                    .width(params.panel_rect.width - 24.0)
                    .background(center.theme().card_bg)
                    .border(1.0, center.theme().card_border)
                    .border_radius(8.0)
                    .padding_insets(Insets::new(14.0, 14.0, 14.0, 14.0))
                    .gap(4.0),
                |card| {
                    card.container_named(
                        "EmptyIconRow",
                        Style::new()
                            .flex_row()
                            .justify_content(JustifyContent::Center)
                            .height(30.0),
                        |r| {
                            r.icon_named(
                                "EmptyWorldIcon",
                                crate::ui::iris_bridge::icons::ICON_WORLD,
                                Color::rgba(0.0, 0.85, 1.0, 0.65),
                                28.0,
                            );
                        },
                    );

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
                },
            );
        },
    );
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