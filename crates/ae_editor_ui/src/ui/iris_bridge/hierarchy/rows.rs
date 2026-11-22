// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Tree Adapter & Declarative Row Renderer
//!
//! Connects ECS `hecs::World` entities to `irisui-widgets`'s swappable
//! [`TreeDataSource`] / [`TreeModel`] / [`UiScope::virtual_tree`] pipeline with
//! zero consumer-side `HashMap` flattening, zero per-frame allocations, and zero `let` plumbing.

use super::types::{
    HIERARCHY_TAG_FOLDOUT_BASE, HIERARCHY_TAG_ROW_BASE, HIERARCHY_TAG_SCROLLBAR_THUMB,
    HIERARCHY_TAG_SCROLLBAR_TRACK, HIERARCHY_TAG_VIEWPORT, make_eye_tag,
};
use crate::ui::iris_bridge::icons::*;
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;
use std::fmt::Write as _;

/// Physical height in pixels for an individual Scene Hierarchy tree row.
pub const HIERARCHY_ROW_HEIGHT: f32 = 24.0;

/// Vertical gap between consecutive tree rows in physical pixels (`0.0` for  striping).
pub const HIERARCHY_ROW_GAP: f32 = 0.0;

/// Total vertical stride for a single row including row gap (`24.0 px`).
pub const HIERARCHY_ROW_STRIDE: f32 = HIERARCHY_ROW_HEIGHT + HIERARCHY_ROW_GAP;

/// Swappable [`TreeDataSource`] adapter projecting an ECS [`hecs::World`] scene graph
/// into Iris UI's generic cached [`TreeModel`].
pub struct SceneHierarchySource<'a>(pub &'a hecs::World);

impl TreeDataSource for SceneHierarchySource<'_> {
    type Id = hecs::Entity;

    fn node_key(&self, id: Self::Id) -> u64 {
        id.to_bits().get()
    }

    fn id_from_key(&self, key: u64) -> Option<Self::Id> {
        hecs::Entity::from_bits(key)
    }

    fn collect_nodes(&self, collector: &mut TreeNodeCollector<Self::Id>) {
        collector.reserve(self.0.len() as usize);
        for ent_ref in self.0.iter() {
            if ent_ref.get::<&ae_core::ui::PauseMenuUiTag>().is_some() {
                continue;
            }
            collector.push_node(
                ent_ref.entity(),
                ent_ref
                    .get::<&ae_core::ecs::Parent>()
                    .and_then(|p| self.0.contains(p.0).then_some(p.0)),
            );
        }
    }

    fn search_text(&self, id: Self::Id, out: &mut String) {
        match self.0.get::<&ae_core::ecs::Name>(id) {
            Ok(name) => out.push_str(&name.0),
            Err(_) => _ = write!(out, "entity {id:?}"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum EntityIcon {
    Texture([f32; 4], Color),
    Text(&'static str),
}

#[inline]
const fn selection_accent(is_selected: bool) -> Color {
    if is_selected {
        Color::rgba(0.0, 0.95, 1.0, 1.0)
    } else {
        Color::WHITE
    }
}

#[inline]
const fn folder_icon(is_selected: bool) -> EntityIcon {
    EntityIcon::Texture(
        ICON_FOLDER,
        if is_selected {
            Color::rgba(0.0, 0.95, 1.0, 1.0)
        } else {
            Color::rgba(0.96, 0.97, 1.0, 0.92)
        },
    )
}

#[inline]
fn normalize_light_tint(rgb: [f32; 3], max_channel: f32) -> Color {
    if max_channel > 0.01 {
        Color::rgba(
            (rgb[0] / max_channel).max(0.3),
            (rgb[1] / max_channel).max(0.3),
            (rgb[2] / max_channel).max(0.3),
            1.0,
        )
    } else {
        Color::rgba(1.0, 0.88, 0.35, 1.0)
    }
}

#[inline]
fn light_icon_tint(rgb: [f32; 3], is_selected: bool) -> Color {
    if is_selected {
        Color::rgba(0.0, 0.95, 1.0, 1.0)
    } else if (rgb[0] - 1.0).abs() < 0.05
        && (rgb[1] - 1.0).abs() < 0.05
        && (rgb[2] - 1.0).abs() < 0.05
    {
        Color::rgba(1.0, 0.88, 0.35, 1.0)
    } else {
        normalize_light_tint(rgb, rgb[0].max(rgb[1]).max(rgb[2]))
    }
}

#[inline]
fn accent_or_slate(is_selected: bool) -> Color {
    if is_selected {
        Color::rgba(0.0, 0.95, 1.0, 1.0)
    } else {
        Color::rgba(0.40, 0.75, 1.0, 1.0)
    }
}

fn resolve_entity_icon(world: &hecs::World, entity: hecs::Entity, is_selected: bool) -> EntityIcon {
    world.entity(entity).ok().map_or_else(
        || folder_icon(is_selected),
        |ent_ref| resolve_component_icon(&ent_ref, is_selected),
    )
}

fn resolve_component_icon(ent_ref: &hecs::EntityRef<'_>, is_selected: bool) -> EntityIcon {
    ent_ref
        .get::<&ae_core::ecs::Light>()
        .map(|light| EntityIcon::Texture(ICON_LIGHT, light_icon_tint(light.color, is_selected)))
        .or_else(|| {
            (ent_ref.get::<&ae_audio::AudioListener>().is_some()
                || ent_ref.get::<&ae_core::camera::Camera>().is_some())
            .then(|| EntityIcon::Texture(ICON_CAMERA, accent_or_slate(is_selected)))
        })
        .or_else(|| {
            ent_ref.get::<&ae_core::ecs::Shape>().map(|shape| {
                EntityIcon::Texture(
                    match *shape {
                        ae_core::ecs::Shape::Sphere => ICON_SPHERE,
                        _ => ICON_CUBE,
                    },
                    selection_accent(is_selected),
                )
            })
        })
        .or_else(|| {
            ent_ref
                .get::<&ae_core::ecs::ModelId>()
                .map(|_| EntityIcon::Texture(ICON_CUBE, selection_accent(is_selected)))
        })
        .or_else(|| {
            ent_ref
                .get::<&ae_audio::AudioSource>()
                .map(|_| EntityIcon::Texture(ICON_AUDIO, accent_or_slate(is_selected)))
        })
        .unwrap_or_else(|| resolve_glyph_or_folder_icon(ent_ref, is_selected))
}

fn resolve_glyph_or_folder_icon(ent_ref: &hecs::EntityRef<'_>, is_selected: bool) -> EntityIcon {
    if ent_ref.get::<&ae_core::ecs::UiPanel>().is_some() {
        EntityIcon::Text("🔲")
    } else if ent_ref.get::<&ae_core::ecs::PlayerHealthBarTag>().is_some() {
        EntityIcon::Text("❤️")
    } else if ent_ref.get::<&ae_core::ecs::ScoreDisplayTag>().is_some() {
        EntityIcon::Text("⭐")
    } else if ent_ref.get::<&ae_core::ecs::ReticleTag>().is_some()
        || ent_ref.get::<&ae_core::ecs::DestructibleTarget>().is_some()
    {
        EntityIcon::Text("🎯")
    } else if ent_ref.get::<&ae_core::ecs::UiProgressBar>().is_some() {
        EntityIcon::Text("📊")
    } else if ent_ref.get::<&ae_core::ecs::UiButton>().is_some() {
        EntityIcon::Text("🔘")
    } else if ent_ref.get::<&ae_core::ecs::UiText>().is_some() {
        EntityIcon::Text("🔤")
    } else if ent_ref.get::<&ae_core::ecs::UiImage>().is_some() {
        EntityIcon::Text("🖼️")
    } else if ent_ref.get::<&ae_core::ecs::UiSlider>().is_some() {
        EntityIcon::Text("🎚️")
    } else if ent_ref.get::<&ae_core::ecs::UiCheckbox>().is_some() {
        EntityIcon::Text("☑️")
    } else if ent_ref.get::<&ae_core::ecs::UiTextInput>().is_some() {
        EntityIcon::Text("📝")
    } else if ent_ref.get::<&ae_core::ecs::PlayerTag>().is_some() {
        EntityIcon::Text("🎮")
    } else if ent_ref.get::<&ae_core::ecs::Rotator>().is_some() {
        EntityIcon::Text("🔄")
    } else if ent_ref.get::<&ae_core::ecs::MovingPlatform>().is_some() {
        EntityIcon::Text("🚡")
    } else if ent_ref.get::<&ae_core::ecs::TriggerZone>().is_some() {
        EntityIcon::Text("⚡")
    } else if ent_ref.get::<&ae_core::ecs::CharacterAction>().is_some() {
        EntityIcon::Text("🔫")
    } else if ent_ref.get::<&ae_core::ecs::SpriteId>().is_some() {
        EntityIcon::Text("🖼")
    } else {
        folder_icon(is_selected)
    }
}

fn hierarchy_virtual_tree_config(
    selected_entity: Option<hecs::Entity>,
) -> VirtualTreeConfig<hecs::Entity> {
    VirtualTreeConfig::new(
        "HierarchyRowsViewport",
        "HierarchyEntityRow",
        HIERARCHY_TAG_VIEWPORT,
        (HIERARCHY_TAG_SCROLLBAR_TRACK, HIERARCHY_TAG_SCROLLBAR_THUMB),
        HIERARCHY_TAG_ROW_BASE,
        HIERARCHY_TAG_FOLDOUT_BASE,
    )
    .with_empty_notice("Scene is empty")
    .with_filtered_empty_notice("No matching entities found")
    .with_default_expanded(true)
    .with_external_selected(selected_entity)
    .with_tree_config(
        TreeViewConfig::default()
            .with_name("HierarchyEntityRow")
            .with_row_height(HIERARCHY_ROW_HEIGHT)
            .with_row_gap(HIERARCHY_ROW_GAP)
            .with_indent(
                TreeIndentConfig::default()
                    .with_base_padding(6.0)
                    .with_indent_step(18.0)
                    .with_line_arm_length(8.0)
                    .with_line_arm_gap(4.0)
                    .with_line_color(Color::rgba(0.20, 0.55, 0.90, 0.85)),
            )
            .with_style(
                TreeRowStyle::dark_default()
                    .with_zebra_striping(
                        Color::rgba(0.045, 0.050, 0.065, 0.70),
                        Color::rgba(0.080, 0.090, 0.120, 0.90),
                    )
                    .with_selected_bg(Color::rgba(0.0, 0.35, 0.50, 0.75))
                    .with_selected_border(Color::rgba(0.0, 0.85, 1.0, 0.80))
                    .with_hover_bg(Color::rgba(0.14, 0.18, 0.26, 0.65))
                    .with_hover_border(Color::rgba(0.24, 0.32, 0.45, 0.50))
                    .with_border_radius(4.0),
            ),
    )
}

/// Renders the virtualized, frustum-culled Scene Hierarchy tree list declaratively via [`UiScope::virtual_tree_retained`].
///
/// Returns the total unfiltered entity row count rendered by the retained tree cache.
pub fn build_hierarchy_rows(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    revision: u64,
    search_query: &str,
    selected_entity: Option<hecs::Entity>,
    action_bus: &mut ActionBus<EngineUiAction>,
) -> usize {
    scope
        .virtual_tree_retained(
            &SceneHierarchySource(world),
            revision,
            world.len() as usize,
            search_query,
            &hierarchy_virtual_tree_config(selected_entity),
            action_bus,
        )
        .on_selection(|bus, event| match event {
            TreeSelectionEvent::Primary(sel) => bus.emit(EngineUiAction::SelectEntity(sel)),
            TreeSelectionEvent::Secondary(id) => bus.emit(EngineUiAction::SelectEntity(Some(id))),
        })
        .context_menu(|menu, bus, entity| render_row_context_menu(menu, entity, bus))
        .render_row(|row_s, bus, entry, master_idx, is_selected| {
            render_hierarchy_row_contents(row_s, world, entry.id, master_idx, is_selected, bus);
        })
        .total_count
}

fn render_row_context_menu(
    menu: &mut UiScope<'_>,
    entity: hecs::Entity,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    menu.context_menu_item_with_icon("🗑", "Delete Entity", None)
        .on_click(|| {
            action_bus.emit(EngineUiAction::SelectEntity(Some(entity)));
            action_bus.emit(EngineUiAction::DeleteSelected);
        });
    menu.context_menu_item_with_icon("👁", "Toggle Visibility", None)
        .on_click(|| {
            action_bus.emit(EngineUiAction::ToggleVisibility(entity));
        });
}

fn render_hierarchy_row_contents(
    row_s: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
    master_idx: usize,
    is_selected: bool,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    render_entity_icon(
        row_s,
        resolve_entity_icon(world, entity, is_selected),
        is_selected,
    );
    render_entity_label(
        row_s,
        world,
        entity,
        row_text_color(
            is_selected,
            world.get::<&ae_core::ecs::Hidden>(entity).is_err(),
        ),
    );
    render_eye_button(
        row_s,
        entity,
        master_idx,
        is_selected,
        world.get::<&ae_core::ecs::Hidden>(entity).is_err(),
        action_bus,
    );
}

#[inline]
const fn row_text_color(is_selected: bool, is_visible: bool) -> Color {
    if is_selected {
        Color::rgba(0.0, 0.95, 1.0, 1.0)
    } else if !is_visible {
        Color::rgba(0.55, 0.60, 0.72, 0.65)
    } else {
        Color::rgba(0.88, 0.91, 0.98, 1.0)
    }
}

fn render_entity_icon(row_s: &mut UiScope<'_>, icon: EntityIcon, is_selected: bool) {
    match icon {
        EntityIcon::Texture(uv, tint) => row_s.icon(uv, tint, 16.0),
        EntityIcon::Text(glyph) => row_s.label_with_width(
            glyph.trim(),
            16.0,
            11.0,
            selection_accent(is_selected),
            TextAlign::Center,
        ),
    }
}

fn render_entity_label(
    row_s: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
    text_color: Color,
) {
    row_s.container(
        Style::new()
            .flex_row()
            .flex_grow(1.0)
            .align_items(AlignItems::Center),
        |nc| match world.get::<&ae_core::ecs::Name>(entity) {
            Ok(name_comp) => nc.colored_label(text_color, name_comp.0.as_str()),
            Err(_) => nc.text_colored_fmt::<32, _>(text_color, |buf| {
                _ = write!(buf, "Entity {entity:?}");
            }),
        },
    );
}

fn render_eye_button(
    row_s: &mut UiScope<'_>,
    entity: hecs::Entity,
    master_idx: usize,
    is_selected: bool,
    is_visible: bool,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    row_s
        .container_tagged_resp(
            "EyeVisibilityButton",
            Style::new()
                .width(22.0)
                .height(20.0)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center),
            WidgetRole::Button,
            make_eye_tag(master_idx),
            |eye_s| {
                eye_s.icon_hover(
                    if is_visible {
                        ICON_EYE_OPEN
                    } else {
                        ICON_EYE_CLOSED
                    },
                    if !is_visible {
                        Color::rgba(0.55, 0.60, 0.72, 0.65)
                    } else if is_selected {
                        Color::rgba(0.0, 0.95, 1.0, 1.0)
                    } else {
                        Color::rgba(0.88, 0.91, 0.98, 0.95)
                    },
                    Color::WHITE,
                    16.0,
                );
            },
        )
        .on_click(|| {
            action_bus.emit(EngineUiAction::ToggleVisibility(entity));
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_tree_depth_and_expansion_invariants() {
        let mut world = hecs::World::new();
        let parent = world.spawn((ae_core::ecs::Name("Root_Parent".into()),));
        let child_1 = world.spawn((
            ae_core::ecs::Name("Child_1".into()),
            ae_core::ecs::Parent(parent),
        ));
        let _grandchild = world.spawn((
            ae_core::ecs::Name("Grandchild".into()),
            ae_core::ecs::Parent(child_1),
        ));

        let src = SceneHierarchySource(&world);
        let mut model = TreeModel::new();

        // 1. Fully expanded
        let tree_state = TreeState::expanded_by_default();
        model.sync(&src, &tree_state, 1, world.len() as usize, "");
        assert_eq!(model.visible_count(), 3);
        assert_eq!(model.rows()[0].depth, 0);
        assert!(model.rows()[0].has_children);
        assert!(model.rows()[0].is_expanded);
        assert_eq!(model.rows()[1].depth, 1);
        assert!(model.rows()[1].has_children);
        assert_eq!(model.rows()[2].depth, 2);
        assert!(!model.rows()[2].has_children);

        // 2. Collapse child_1: only grandchild hidden
        let mut collapsed_state = TreeState::expanded_by_default();
        collapsed_state.set_expanded(child_1, false);
        model.sync(&src, &collapsed_state, 1, world.len() as usize, "");
        assert_eq!(model.visible_count(), 2);
        assert_eq!(model.rows()[0].depth, 0);
        assert_eq!(model.rows()[1].depth, 1);
        assert!(!model.rows()[1].is_expanded);
    }

    #[test]
    fn test_hierarchy_ui_element_and_3d_entity_icon_width_uniformity() {
        let mut world = hecs::World::new();
        let e_3d = world.spawn((
            ae_core::ecs::Name("Ground Plane".into()),
            ae_core::ecs::Shape::Cube,
        ));
        let e_ui_text = world.spawn((
            ae_core::ecs::Name("UI Text".into()),
            ae_core::ecs::UiElement::default(),
            ae_core::ecs::UiText::new("Sample Text", 16.0),
        ));
        let e_ui_btn = world.spawn((
            ae_core::ecs::Name("UI Button".into()),
            ae_core::ecs::UiElement::default(),
            ae_core::ecs::UiButton::default(),
        ));
        let e_ui_pnl = world.spawn((
            ae_core::ecs::Name("UI Panel".into()),
            ae_core::ecs::UiElement::default(),
            ae_core::ecs::UiPanel::default(),
        ));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");
        let mut bus = ActionBus::new();

        let total = {
            let mut scope = UiScope::new(&mut tree, root);
            build_hierarchy_rows(&mut scope, &world, 1, "", None, &mut bus)
        };
        assert_eq!(total, 4);

        let mut row_icon_widths = Vec::new();
        tree.traverse_depth_first(root, &mut |_id, node| {
            if node.name.as_deref() == Some("HierarchyEntityRow")
                && let Some(&first_child_id) = node.children.first()
                && let Some(child_node) = tree.get(first_child_id)
                && let Some(w) = child_node.style.width
            {
                row_icon_widths.push(w);
            }
        });

        assert_eq!(
            row_icon_widths.len(),
            4,
            "Every row must emit an icon node of exactly 16.0px width"
        );
        for w in row_icon_widths {
            assert_eq!(w, 16.0, "All entity icon widths must strictly be 16.0px");
        }

        let _ = (e_3d, e_ui_text, e_ui_btn, e_ui_pnl);
    }

    #[test]
    fn test_hierarchy_zebra_striping_row_backgrounds() {
        let mut world = hecs::World::new();
        let e0 = world.spawn((ae_core::ecs::Name("Entity 0".into()),));
        let e1 = world.spawn((ae_core::ecs::Name("Entity 1".into()),));
        let e2 = world.spawn((ae_core::ecs::Name("Entity 2".into()),));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");
        let mut bus = ActionBus::new();

        let total = {
            let mut scope = UiScope::new(&mut tree, root);
            build_hierarchy_rows(&mut scope, &world, 1, "", None, &mut bus)
        };
        assert_eq!(total, 3);

        let mut row_bgs = Vec::new();
        tree.traverse_depth_first(root, &mut |_id, node| {
            if node.name.as_deref() == Some("HierarchyEntityRow") {
                row_bgs.push(node.style.background_color);
            }
        });

        assert_eq!(row_bgs.len(), 3);
        let even_bg = Color::rgba(0.045, 0.050, 0.065, 0.70);
        let odd_bg = Color::rgba(0.080, 0.090, 0.120, 0.90);

        assert_eq!(row_bgs[0], even_bg);
        assert_eq!(row_bgs[1], odd_bg);
        assert_eq!(row_bgs[2], even_bg);

        let _ = (e0, e1, e2);
    }
}