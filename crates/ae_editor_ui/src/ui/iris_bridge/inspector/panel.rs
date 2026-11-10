// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Scene Inspector panel layout assembly and empty-selection renderer.

use super::add_menu;
use super::appearance;
use super::color_picker_popup;
use super::footer;
use super::header;
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
        params.world,
        entity,
        params.inspector_color_hex,
        saved_swatches,
        is_color_picker_open,
    )
}

/// Evaluates primary cards followed by dynamic registry cards for the active rendering context.
fn render_context_cards(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
    hex_str: &str,
    saved_swatches: &mut Vec<[f32; 4]>,
    is_color_picker_open: &mut bool,
) -> Option<crate::ui::types::EngineUiAction> {
    combine_card_actions(
        render_primary_cards(
            scope,
            world,
            entity,
            hex_str,
            saved_swatches,
            is_color_picker_open,
        ),
        render_registry_cards(scope, world, entity).or_else(|| {
            super::components::hierarchy::draw_parent_card_if_present(scope, world, entity)
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
    world: &hecs::World,
    entity: hecs::Entity,
    hex_str: &str,
    saved_swatches: &mut Vec<[f32; 4]>,
    is_color_picker_open: &mut bool,
) -> Option<crate::ui::types::EngineUiAction> {
    if world.get::<&ae_core::ecs::UiElement>(entity).is_ok() {
        ui_transform::build_ui_transform_card(scope, world, entity);
        header::component_delete_action(scope, entity, ui_transform::UI_ELEMENT_COMPONENT_NAME)
    } else {
        transform::build_transform_card(scope, world, entity);
        appearance::build_appearance_card(
            scope,
            world,
            entity,
            hex_str,
            saved_swatches,
            is_color_picker_open,
        )
    }
}

/// Built-in component inspector card descriptor pairing canonical component name with its pure render function.
struct BuiltinComponentCard {
    /// Canonical component type name matching the card header action tag.
    name: &'static str,
    /// Pure rendering function pointer drawing the component card if present on entity.
    draw: fn(&mut UiScope<'_>, &hecs::World, hecs::Entity),
}

/// Ordered pipeline of all built-in attachable ECS component inspector cards.
///
/// Preserves deterministic inspection hierarchy layout order matching registration.
const BUILTIN_COMPONENT_CARDS: [BuiltinComponentCard; 31] = [
    // 1. Physics Components
    BuiltinComponentCard {
        name: "RigidBody",
        draw: super::components::draw_rigidbody_card,
    },
    BuiltinComponentCard {
        name: "Collider",
        draw: super::components::draw_collider_card,
    },
    BuiltinComponentCard {
        name: "PhysicsMaterial",
        draw: super::components::draw_physics_material_card,
    },
    // 2. Character & Gameplay Components
    BuiltinComponentCard {
        name: "CharacterController",
        draw: super::components::draw_character_controller_card,
    },
    BuiltinComponentCard {
        name: "CharacterAction",
        draw: super::components::draw_character_action_card,
    },
    BuiltinComponentCard {
        name: "PlayerTag",
        draw: super::components::draw_player_tag_card,
    },
    BuiltinComponentCard {
        name: "Velocity",
        draw: super::components::draw_velocity_card,
    },
    BuiltinComponentCard {
        name: "Rotator",
        draw: super::components::draw_rotator_card,
    },
    BuiltinComponentCard {
        name: "MovingPlatform",
        draw: super::components::draw_moving_platform_card,
    },
    BuiltinComponentCard {
        name: "TriggerZone",
        draw: super::components::draw_trigger_zone_card,
    },
    BuiltinComponentCard {
        name: "DestructibleTarget",
        draw: super::components::draw_destructible_target_card,
    },
    // 3. Rendering, LOD & Illumination Components
    BuiltinComponentCard {
        name: "Light",
        draw: super::components::draw_light_card,
    },
    BuiltinComponentCard {
        name: "ModelMesh",
        draw: super::components::draw_model_mesh_card,
    },
    BuiltinComponentCard {
        name: "Shape",
        draw: super::components::draw_shape_card,
    },
    BuiltinComponentCard {
        name: "LodGroup",
        draw: super::components::draw_lod_group_card,
    },
    // 4. Skeletal Animation Components
    BuiltinComponentCard {
        name: "AnimationPlayer",
        draw: super::components::draw_animation_player_card,
    },
    // 5. Audio Components
    BuiltinComponentCard {
        name: "AudioSource",
        draw: super::components::draw_audio_source_card,
    },
    BuiltinComponentCard {
        name: "AudioListener",
        draw: super::components::draw_audio_listener_card,
    },
    // 6. In-Game 2D UI & HUD Components
    BuiltinComponentCard {
        name: "UiElement",
        draw: super::components::draw_ui_element_card,
    },
    BuiltinComponentCard {
        name: "UiPanel",
        draw: super::components::draw_ui_panel_card,
    },
    BuiltinComponentCard {
        name: "UiText",
        draw: super::components::draw_ui_text_card,
    },
    BuiltinComponentCard {
        name: "UiProgressBar",
        draw: super::components::draw_ui_progress_bar_card,
    },
    BuiltinComponentCard {
        name: "UiButton",
        draw: super::components::draw_ui_button_card,
    },
    BuiltinComponentCard {
        name: "UiImage",
        draw: super::components::draw_ui_image_card,
    },
    BuiltinComponentCard {
        name: "UiSlider",
        draw: super::components::draw_ui_slider_card,
    },
    BuiltinComponentCard {
        name: "UiCheckbox",
        draw: super::components::draw_ui_checkbox_card,
    },
    BuiltinComponentCard {
        name: "UiTextInput",
        draw: super::components::draw_ui_text_input_card,
    },
    BuiltinComponentCard {
        name: "UiLayoutGroup",
        draw: super::components::draw_ui_layout_group_card,
    },
    BuiltinComponentCard {
        name: "PlayerHealthBarTag",
        draw: super::components::draw_player_health_bar_tag_card,
    },
    BuiltinComponentCard {
        name: "ScoreDisplayTag",
        draw: super::components::draw_score_display_tag_card,
    },
    BuiltinComponentCard {
        name: "ReticleTag",
        draw: super::components::draw_reticle_tag_card,
    },
];

/// Renders all non-hierarchy component inspector cards via a direct pure function pipeline
/// and returns the first [`crate::ui::types::EngineUiAction::RemoveComponent`] whose card delete
/// button was clicked in this frame.
///
/// Every visible card is unconditionally evaluated: [`Option::or_else`] preserves the first
/// triggered action while guaranteeing that remaining cards are rendered without early returns.
pub fn render_registry_cards(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
) -> Option<crate::ui::types::EngineUiAction> {
    BUILTIN_COMPONENT_CARDS.iter().fold(None, |action, card| {
        (card.draw)(scope, world, entity);
        action.or_else(|| header::component_delete_action(scope, entity, card.name))
    })
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
            inspector_hsv: [0.0, 0.0, 1.0],
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

    #[test]
    fn test_render_registry_cards_pure_pipeline_renders_and_detects_delete_action() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut world = hecs::World::new();
        let entity = world.spawn((
            ae_core::ecs::RigidBody::default(),
            ae_core::ecs::Light::default(),
        ));

        {
            let mut scope = UiScope::new(&mut tree, root);
            let action = render_registry_cards(&mut scope, &world, entity);
            assert!(
                action.is_none(),
                "No delete action should be emitted initially"
            );
        }

        assert!(
            tree.find_node_by_name("RigidBodyCard").is_some(),
            "RigidBodyCard must be rendered by pure pipeline"
        );
        assert!(
            tree.find_node_by_name("LightCard").is_some(),
            "LightCard must be rendered by pure pipeline"
        );

        // Click delete button for RigidBody
        let delete_tag = UiScope::action_tag("RigidBody");
        tree.event_state.frame_clicked_tag = Some(delete_tag);

        {
            let mut scope = UiScope::new(&mut tree, root);
            let action = render_registry_cards(&mut scope, &world, entity);
            assert!(
                matches!(
                    action,
                    Some(crate::ui::types::EngineUiAction::RemoveComponent(
                        ent,
                        "RigidBody"
                    )) if ent == entity
                ),
                "Delete action must emit EngineUiAction::RemoveComponent for clicked card"
            );
        }
    }
}