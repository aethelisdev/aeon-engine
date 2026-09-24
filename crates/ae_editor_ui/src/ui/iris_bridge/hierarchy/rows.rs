// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Rows and DFS Tree Flattener
//!
//! Flattens the ECS hierarchy tree into a deterministic DFS pre-order row list and
//! renders interactive tree rows with 1:1 selection styling, semantic hardware tagging,
//! and eye visibility toggles using pure declarative [`UiScope::virtual_scroll_area`]
//! with $O(1)$ UI frustum culling.
//!

use super::types::{
    HIERARCHY_TAG_PANEL_ROOT, HierarchyPanelParams, HierarchyRow, make_eye_tag, make_foldout_tag,
    make_row_tag,
};
use crate::ui::iris_bridge::icons::*;
use irisui::prelude::*;
use std::collections::{HashMap, HashSet};

/// Legacy aliases for Scene Hierarchy icons in `editor_atlas.png`.
pub use crate::ui::iris_bridge::icons::{
    ICON_CAMERA as HIERARCHY_ICON_CAMERA, ICON_CUBE as HIERARCHY_ICON_CUBE,
    ICON_EYE_CLOSED as HIERARCHY_ICON_EYE_CLOSED, ICON_EYE_OPEN as HIERARCHY_ICON_EYE_OPEN,
    ICON_FOLDER as HIERARCHY_ICON_FOLDER, ICON_LIGHT as HIERARCHY_ICON_LIGHT,
    ICON_PLUS as HIERARCHY_ICON_PLUS, ICON_SPHERE as HIERARCHY_ICON_SPHERE,
};

/// Physical height in pixels for an individual Scene Hierarchy tree row.
pub const HIERARCHY_ROW_HEIGHT: f32 = 24.0;

/// Vertical gap between consecutive tree rows in physical pixels.
pub const HIERARCHY_ROW_GAP: f32 = 2.0;

/// Total vertical stride for a single row including row gap (`26.0 px`).
pub const HIERARCHY_ROW_STRIDE: f32 = HIERARCHY_ROW_HEIGHT + HIERARCHY_ROW_GAP;

/// Flattens the ECS world hierarchy into a deterministic DFS pre-order row list,
/// respecting collapsed branch state.
pub fn sync_hierarchy_rows(
    world: &hecs::World,
    collapsed_entities: &HashSet<hecs::Entity>,
    out_rows: &mut Vec<HierarchyRow>,
) {
    out_rows.clear();
    let entity_count = world.len() as usize;
    if entity_count == 0 {
        return;
    }

    // 1. Fast sparse parent query: only iterates entities with Parent component
    let mut parent_map: HashMap<hecs::Entity, hecs::Entity> = HashMap::new();
    let mut children_map: HashMap<hecs::Entity, Vec<hecs::Entity>> = HashMap::new();

    for (ent, p) in world
        .query::<(hecs::Entity, &ae_core::ecs::Parent)>()
        .iter()
    {
        if world.contains(p.0) {
            parent_map.insert(ent, p.0);
            children_map.entry(p.0).or_default().push(ent);
        }
    }

    // 2. Fast flat path if no parenting exists in scene
    if parent_map.is_empty() {
        out_rows.reserve(entity_count);
        for ent_ref in world.iter() {
            if ent_ref.get::<&ae_core::ui::PauseMenuUiTag>().is_some() {
                continue;
            }
            out_rows.push(HierarchyRow {
                entity: ent_ref.entity(),
                depth: 0,
                has_children: false,
                is_expanded: true,
            });
        }
        out_rows.sort_by_key(|r| r.entity.id());
        return;
    }

    // 3. Hierarchical path: DFS traversal starting from root nodes
    for children in children_map.values_mut() {
        children.sort_by_key(|e| e.id());
    }

    out_rows.reserve(entity_count);
    let mut root_entities = Vec::with_capacity(entity_count);
    for ent_ref in world.iter() {
        let ent = ent_ref.entity();
        if ent_ref.get::<&ae_core::ui::PauseMenuUiTag>().is_some() {
            continue;
        }
        if !parent_map.contains_key(&ent) {
            root_entities.push(ent);
        }
    }
    root_entities.sort_by_key(|e| e.id());

    for root in root_entities {
        push_dfs_tree(root, 0, &children_map, collapsed_entities, out_rows);
    }
}

/// Helper function performing recursive DFS traversal into the hierarchy tree.
fn push_dfs_tree(
    ent: hecs::Entity,
    depth: u16,
    children_map: &HashMap<hecs::Entity, Vec<hecs::Entity>>,
    collapsed_entities: &HashSet<hecs::Entity>,
    out_rows: &mut Vec<HierarchyRow>,
) {
    let has_children = children_map.get(&ent).is_some_and(|v| !v.is_empty());
    let is_expanded = !collapsed_entities.contains(&ent);
    out_rows.push(HierarchyRow {
        entity: ent,
        depth,
        has_children,
        is_expanded,
    });
    if is_expanded {
        if let Some(children) = children_map.get(&ent) {
            for &child in children {
                push_dfs_tree(child, depth + 1, children_map, collapsed_entities, out_rows);
            }
        }
    }
}

/// Represents an entity icon either from the hardware texture atlas or text glyph fallback.
#[derive(Clone, Copy, Debug)]
enum EntityIcon {
    Texture([f32; 4], Color),
    Text(&'static str),
}

/// Resolves the type icon for a visible entity based on its components.
fn resolve_entity_icon(world: &hecs::World, entity: hecs::Entity, is_selected: bool) -> EntityIcon {
    let Ok(ent_ref) = world.entity(entity) else {
        let folder_color = if is_selected {
            Color::rgba(0.0, 0.95, 1.0, 1.0)
        } else {
            Color::rgba(0.96, 0.97, 1.0, 0.92)
        };
        return EntityIcon::Texture(ICON_FOLDER, folder_color);
    };

    let base_tint = if is_selected {
        Color::rgba(0.0, 0.95, 1.0, 1.0)
    } else {
        Color::rgba(1.0, 1.0, 1.0, 1.0)
    };

    if let Some(light) = ent_ref.get::<&ae_core::ecs::Light>() {
        let tint = if is_selected {
            Color::rgba(0.0, 0.95, 1.0, 1.0)
        } else {
            let is_default_white = (light.color[0] - 1.0).abs() < 0.05
                && (light.color[1] - 1.0).abs() < 0.05
                && (light.color[2] - 1.0).abs() < 0.05;
            if is_default_white {
                Color::rgba(1.0, 0.88, 0.35, 1.0)
            } else {
                let max_c = light.color[0].max(light.color[1]).max(light.color[2]);
                if max_c > 0.01 {
                    Color::rgba(
                        (light.color[0] / max_c).max(0.3),
                        (light.color[1] / max_c).max(0.3),
                        (light.color[2] / max_c).max(0.3),
                        1.0,
                    )
                } else {
                    Color::rgba(1.0, 0.88, 0.35, 1.0)
                }
            }
        };
        EntityIcon::Texture(ICON_LIGHT, tint)
    } else if ent_ref.get::<&ae_audio::AudioListener>().is_some()
        || ent_ref.get::<&ae_core::camera::Camera>().is_some()
    {
        let cam_tint = if is_selected {
            Color::rgba(0.0, 0.95, 1.0, 1.0)
        } else {
            Color::rgba(0.40, 0.75, 1.0, 1.0)
        };
        EntityIcon::Texture(ICON_CAMERA, cam_tint)
    } else if let Some(shape) = ent_ref.get::<&ae_core::ecs::Shape>() {
        match *shape {
            ae_core::ecs::Shape::Sphere => EntityIcon::Texture(ICON_SPHERE, base_tint),
            _ => EntityIcon::Texture(ICON_CUBE, base_tint),
        }
    } else if ent_ref.get::<&ae_core::ecs::ModelId>().is_some() {
        EntityIcon::Texture(ICON_CUBE, base_tint)
    } else if ent_ref.get::<&ae_core::ecs::UiPanel>().is_some() {
        EntityIcon::Text("🔲")
    } else if ent_ref.get::<&ae_core::ecs::PlayerHealthBarTag>().is_some() {
        EntityIcon::Text("❤️")
    } else if ent_ref.get::<&ae_core::ecs::ScoreDisplayTag>().is_some() {
        EntityIcon::Text("⭐")
    } else if ent_ref.get::<&ae_core::ecs::ReticleTag>().is_some() {
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
    } else if ent_ref.get::<&ae_audio::AudioSource>().is_some() {
        let audio_tint = if is_selected {
            Color::rgba(0.0, 0.95, 1.0, 1.0)
        } else {
            Color::rgba(0.40, 0.75, 1.0, 1.0)
        };
        EntityIcon::Texture(ICON_AUDIO, audio_tint)
    } else if ent_ref.get::<&ae_core::ecs::PlayerTag>().is_some() {
        EntityIcon::Text("🎮")
    } else if ent_ref.get::<&ae_core::ecs::Rotator>().is_some() {
        EntityIcon::Text("🔄")
    } else if ent_ref.get::<&ae_core::ecs::MovingPlatform>().is_some() {
        EntityIcon::Text("🚡")
    } else if ent_ref.get::<&ae_core::ecs::TriggerZone>().is_some() {
        EntityIcon::Text("⚡")
    } else if ent_ref.get::<&ae_core::ecs::DestructibleTarget>().is_some() {
        EntityIcon::Text("🎯")
    } else if ent_ref.get::<&ae_core::ecs::CharacterAction>().is_some() {
        EntityIcon::Text("🔫")
    } else if ent_ref.get::<&ae_core::ecs::SpriteId>().is_some() {
        EntityIcon::Text("🖼")
    } else {
        let folder_color = if is_selected {
            Color::rgba(0.0, 0.95, 1.0, 1.0)
        } else {
            Color::rgba(0.96, 0.97, 1.0, 0.92)
        };
        EntityIcon::Texture(ICON_FOLDER, folder_color)
    }
}

/// Renders the virtualized, frustum-culled tree list of ECS entity rows using pure declarative [`UiScope::virtual_scroll_area`].
///
/// Automatically prunes off-screen entity rows from the UI tree, keeping node generation and layout cost
/// strictly $O(1)$ regardless of whether the scene contains 10 or 100,000 entities.
///
/// Returns the computed maximum scroll limit `max_scroll_y` in physical pixels.
pub fn build_hierarchy_rows(
    scope: &mut UiScope<'_>,
    rows: &[HierarchyRow],
    params: &HierarchyPanelParams<'_>,
) -> f32 {
    let query_lower = params.search_query.trim().to_lowercase();

    // 1. Collect indices of matching entities
    let mut matching_indices: Vec<usize> = Vec::with_capacity(rows.len());
    if query_lower.is_empty() {
        matching_indices.extend(0..rows.len());
    } else {
        for (master_idx, row) in rows.iter().enumerate() {
            let matches = if let Ok(name_comp) = params.world.get::<&ae_core::ecs::Name>(row.entity)
            {
                name_comp.0.to_lowercase().contains(&query_lower)
            } else {
                format!("Entity {:?}", row.entity)
                    .to_lowercase()
                    .contains(&query_lower)
            };
            if matches {
                matching_indices.push(master_idx);
            }
        }
    }

    // 2. Empty state when no entities exist or filter produces 0 results
    if matching_indices.is_empty() {
        let empty_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .flex_grow(1.0)
            .height(40.0);

        let msg_text = if !query_lower.is_empty() {
            "No matching entities found"
        } else {
            "Scene is empty"
        };

        scope.container(empty_style, |e| {
            e.label(
                msg_text,
                11.0,
                Color::rgba(0.45, 0.48, 0.58, 1.0),
                TextAlign::Center,
            );
        });
        return 0.0;
    }

    // 3. Delegate frustum culling, overscan buffers, and sub-pixel scrolling to virtual_scroll_area
    let total_filtered = matching_indices.len();
    let vp_height = (params.panel_rect.height - 63.0).max(40.0);

    let config = VirtualScrollConfig::fixed(total_filtered, HIERARCHY_ROW_STRIDE)
        .with_overscan(2)
        .with_bg(Color::TRANSPARENT);

    scope.virtual_scroll_area(
        "HierarchyRowsViewport",
        HIERARCHY_TAG_PANEL_ROOT,
        vp_height,
        params.scroll_y,
        config,
        |row_scope, filtered_idx| {
            let master_idx = matching_indices[filtered_idx];
            let row = &rows[master_idx];

            let is_selected = params.selected_entity == Some(row.entity);
            let row_tag = make_row_tag(master_idx);
            let eye_tag = make_eye_tag(master_idx);
            let foldout_tag = make_foldout_tag(master_idx);
            let is_row_hovered = params.hovered_tag == Some(row_tag);
            let is_eye_hovered = params.hovered_tag == Some(eye_tag);
            let is_foldout_hovered = params.hovered_tag == Some(foldout_tag);

            let (bg_color, border_color) = if is_selected {
                (
                    Color::rgba(0.0, 0.35, 0.50, 0.65), // Active cyan pill background
                    Color::rgba(0.0, 0.85, 1.0, 0.80),  // Cyan border highlight
                )
            } else if is_row_hovered {
                (
                    Color::rgba(0.14, 0.18, 0.26, 0.65), // Subtle sleek hover highlight
                    Color::rgba(0.24, 0.32, 0.45, 0.50), // Subtle hover border
                )
            } else {
                (Color::TRANSPARENT, Color::TRANSPARENT)
            };

            let (_line_tip, padding_left) = compute_tree_connector_geometry(row.depth);
            let row_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .height(HIERARCHY_ROW_HEIGHT)
                .padding_insets(Insets::new(0.0, 6.0, 0.0, padding_left))
                .gap(4.0)
                .background(bg_color)
                .border(1.0, border_color)
                .border_radius(4.0);

            row_scope.container_tagged(
                "HierarchyEntityRow",
                row_style,
                WidgetRole::Button,
                row_tag,
                |row_s| {
                    // 0. Hierarchy Tree Connector Lines (L-shape connector when depth > 0)
                    if row.depth > 0 {
                        let line_color = Color::rgba(0.20, 0.55, 0.90, 0.85);
                        for d in 0..row.depth {
                            let stem_x = 12.0 + (d as f32 * 18.0);
                            let is_last_level = d == row.depth - 1;

                            // Iris UI absolute positioning adds `inner.x` (which equals `padding_left`).
                            // To place the stem line precisely at absolute `stem_x`, subtract `padding_left`.
                            let rel_x = stem_x - padding_left;

                            // Vertical branch line
                            let v_h = if is_last_level {
                                HIERARCHY_ROW_HEIGHT * 0.5
                            } else {
                                HIERARCHY_ROW_HEIGHT
                            };
                            let v_style = Style::new()
                                .position_absolute()
                                .left(rel_x)
                                .top(0.0)
                                .width(1.2)
                                .height(v_h)
                                .background(line_color);
                            row_s.empty_box(v_style);

                            // Horizontal branch arm ending precisely 4.0px before the child icon
                            if is_last_level {
                                let h_style = Style::new()
                                    .position_absolute()
                                    .left(rel_x)
                                    .top((HIERARCHY_ROW_HEIGHT * 0.5 - 0.6).round())
                                    .width(8.0)
                                    .height(1.2)
                                    .background(line_color);
                                row_s.empty_box(h_style);
                            }
                        }
                    }

                    // 1. Foldout Chevron Button (if entity has children)
                    if row.has_children {
                        let foldout_glyph = if row.is_expanded { "▼" } else { "▶" };
                        let foldout_col = if is_selected || is_foldout_hovered {
                            Color::WHITE
                        } else {
                            Color::rgba(0.65, 0.68, 0.78, 1.0)
                        };
                        let fold_style = Style::new()
                            .flex_row()
                            .width(12.0)
                            .height(HIERARCHY_ROW_HEIGHT)
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::Center);

                        row_s.container_tagged(
                            "HierarchyFoldoutButton",
                            fold_style,
                            WidgetRole::Button,
                            make_foldout_tag(master_idx),
                            |fold_s| {
                                fold_s.label_with_width(
                                    foldout_glyph,
                                    12.0,
                                    9.0,
                                    foldout_col,
                                    TextAlign::Center,
                                );
                            },
                        );
                    }

                    // 2. Entity Type Icon
                    let comp_icon = resolve_entity_icon(params.world, row.entity, is_selected);
                    match comp_icon {
                        EntityIcon::Texture(uv, tint) => {
                            row_s.icon(uv, tint, 16.0);
                        }
                        EntityIcon::Text(glyph) => {
                            let text_col = if is_selected {
                                Color::rgba(0.0, 0.95, 1.0, 1.0)
                            } else {
                                Color::WHITE
                            };
                            row_s.label_with_width(
                                glyph.trim(),
                                16.0,
                                11.0,
                                text_col,
                                TextAlign::Center,
                            );
                        }
                    }

                    // 2. Entity Name Label
                    let is_visible = params
                        .world
                        .get::<&ae_core::ecs::Hidden>(row.entity)
                        .is_err();

                    let text_color = if is_selected {
                        Color::rgba(0.0, 0.95, 1.0, 1.0) // Bright cyan #00e5ff
                    } else if !is_visible {
                        Color::rgba(0.55, 0.60, 0.72, 0.65) // Muted slate when entity is hidden
                    } else {
                        Color::rgba(0.88, 0.91, 0.98, 1.0) // Crisp slate white
                    };

                    let entity_name = if let Ok(name_comp) =
                        params.world.get::<&ae_core::ecs::Name>(row.entity)
                    {
                        name_comp.0.clone()
                    } else {
                        format!("Entity {:?}", row.entity)
                    };

                    let name_container_style = Style::new()
                        .flex_row()
                        .flex_grow(1.0)
                        .align_items(AlignItems::Center);

                    row_s.container(name_container_style, |nc| {
                        nc.label(entity_name, 11.5, text_color, TextAlign::Left);
                    });

                    // 3. Eye Visibility Toggle Button
                    let (eye_uv, eye_col) = if is_eye_hovered {
                        (
                            if is_visible {
                                ICON_EYE_OPEN
                            } else {
                                ICON_EYE_CLOSED
                            },
                            Color::WHITE,
                        )
                    } else if !is_visible {
                        (ICON_EYE_CLOSED, Color::rgba(0.55, 0.60, 0.72, 0.65))
                    } else if is_selected {
                        (ICON_EYE_OPEN, Color::rgba(0.0, 0.95, 1.0, 1.0))
                    } else {
                        (ICON_EYE_OPEN, Color::rgba(0.88, 0.91, 0.98, 0.95))
                    };

                    let eye_style = Style::new()
                        .width(22.0)
                        .height(20.0)
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center);

                    row_s.container_tagged(
                        "EyeVisibilityButton",
                        eye_style,
                        WidgetRole::Button,
                        eye_tag,
                        |eye_s| {
                            eye_s.icon(eye_uv, eye_col, 16.0);
                        },
                    );
                },
            );
        },
    )
}

/// Computes the horizontal branch line tip X position and row content start X position.
///
/// Ensures the architectural invariant that child entity icons sit strictly 4.0px beyond
/// the tip of horizontal connector lines, guaranteeing that connector lines never penetrate
/// or occlude icon glyphs or textures.
///
/// Returns `(line_tip_x, content_start_x)` in physical pixels.
#[inline]
pub fn compute_tree_connector_geometry(depth: u16) -> (f32, f32) {
    if depth == 0 {
        (0.0, 6.0)
    } else {
        let last_d = (depth - 1) as f32;
        let stem_x = 12.0 + last_d * 18.0;
        let line_tip = stem_x + 8.0;
        let content_start = line_tip + 4.0;
        (line_tip, content_start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_connector_geometry_spacing_invariants() {
        // Root entities have 6px padding and no connector arm
        let (root_tip, root_start) = compute_tree_connector_geometry(0);
        assert_eq!(root_tip, 0.0);
        assert_eq!(root_start, 6.0);

        // Child entities (depth 1..=5) must have content starting precisely 4.0px beyond line tip
        for depth in 1..=5 {
            let (line_tip, content_start) = compute_tree_connector_geometry(depth);
            let gap = content_start - line_tip;
            assert!(
                (gap - 4.0).abs() < f32::EPSILON,
                "Content start must be precisely 4.0px beyond connector line tip at depth {depth}"
            );
            assert!(
                content_start > line_tip,
                "Line must never penetrate or occlude icon at depth {depth}"
            );
        }
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

        let collapsed = HashSet::new();
        let mut rows = Vec::new();
        sync_hierarchy_rows(&world, &collapsed, &mut rows);

        assert_eq!(rows.len(), 4);
        for row in &rows {
            assert_eq!(row.depth, 0);
        }

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");
        let mut scope = UiScope::new(&mut tree, root);
        let params = HierarchyPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: None,
            search_query: "",
            is_editing: true,
            is_2d: true,
            scroll_y: 0.0,
            active_submenu: None,
            active_sub_submenu: None,
            is_add_menu_open: false,
            active_context_menu: None,
            cursor_pos: Point::new(-1.0, -1.0),
            is_search_focused: false,
            blink_caret: false,
            collapsed_entities: &collapsed,
            hovered_tag: None,
        };

        build_hierarchy_rows(&mut scope, &rows, &params);

        // Verify that every HierarchyEntityRow container has an icon node of exactly 16.0px width
        let mut row_icon_widths = Vec::new();
        tree.traverse_depth_first(root, &mut |_id, node| {
            if node.name.as_deref() == Some("HierarchyEntityRow") {
                // The first child of a depth-0 non-branch row is the entity icon
                if let Some(&first_child_id) = node.children.first() {
                    if let Some(child_node) = tree.get(first_child_id) {
                        if let Some(w) = child_node.style.width {
                            row_icon_widths.push(w);
                        }
                    }
                }
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
}