// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Rows and DFS Tree Flattener
//!
//! Flattens the ECS hierarchy tree into a single-pass DFS list and renders
//! interactive tree rows with 1:1 selection pills, connector lines, and eye toggles.

use super::types::{HierarchyPanelParams, HierarchyPanelTargets, HierarchyRow};
use irisui::prelude::*;
use std::collections::HashMap;

/// Flattens the ECS world hierarchy into a deterministic DFS pre-order row list.
pub fn sync_hierarchy_rows(world: &hecs::World, out_rows: &mut Vec<HierarchyRow>) {
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

    // 2. Ultra-fast flat path if no parenting exists in scene (e.g. 100k benchmarks)
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
            });
        }
        out_rows.sort_by_key(|r| r.entity.id());
        return;
    }

    // 3. Hierarchical path: DFS traversal starting from root nodes (sorted deterministically)
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
        push_dfs_tree(root, 0, &children_map, out_rows);
    }
}

/// Helper function performing recursive DFS traversal into the hierarchy tree.
fn push_dfs_tree(
    ent: hecs::Entity,
    depth: u16,
    children_map: &HashMap<hecs::Entity, Vec<hecs::Entity>>,
    out_rows: &mut Vec<HierarchyRow>,
) {
    let has_children = children_map.get(&ent).is_some_and(|v| !v.is_empty());
    out_rows.push(HierarchyRow {
        entity: ent,
        depth,
        has_children,
    });
    if let Some(children) = children_map.get(&ent) {
        for &child in children {
            push_dfs_tree(child, depth + 1, children_map, out_rows);
        }
    }
}

/// UV coordinates for Scene Hierarchy icons in `editor_atlas.png`.
pub const HIERARCHY_ICON_EYE_OPEN: [f32; 4] = [0.00, 0.25, 0.25, 0.50];
pub const HIERARCHY_ICON_EYE_CLOSED: [f32; 4] = [0.25, 0.25, 0.50, 0.50];
pub const HIERARCHY_ICON_FOLDER: [f32; 4] = [0.50, 0.25, 0.75, 0.50];
pub const HIERARCHY_ICON_CUBE: [f32; 4] = [0.75, 0.25, 1.00, 0.50];

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
        return EntityIcon::Texture(HIERARCHY_ICON_FOLDER, folder_color);
    };

    if ent_ref.get::<&ae_core::ecs::Shape>().is_some()
        || ent_ref.get::<&ae_core::ecs::ModelId>().is_some()
        || ent_ref.get::<&ae_core::ecs::UiPanel>().is_some()
    {
        let cube_color = if is_selected {
            Color::rgba(0.0, 0.95, 1.0, 1.0)
        } else {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        };
        EntityIcon::Texture(HIERARCHY_ICON_CUBE, cube_color)
    } else if ent_ref.get::<&ae_core::ecs::PlayerHealthBarTag>().is_some() {
        EntityIcon::Text("❤️ ")
    } else if ent_ref.get::<&ae_core::ecs::ScoreDisplayTag>().is_some() {
        EntityIcon::Text("⭐ ")
    } else if ent_ref.get::<&ae_core::ecs::ReticleTag>().is_some() {
        EntityIcon::Text("🎯 ")
    } else if ent_ref.get::<&ae_core::ecs::UiProgressBar>().is_some() {
        EntityIcon::Text("📊 ")
    } else if ent_ref.get::<&ae_core::ecs::UiButton>().is_some() {
        EntityIcon::Text("🔘 ")
    } else if ent_ref.get::<&ae_core::ecs::UiText>().is_some() {
        EntityIcon::Text("🔤 ")
    } else if ent_ref.get::<&ae_core::ecs::UiImage>().is_some() {
        EntityIcon::Text("🖼️ ")
    } else if ent_ref.get::<&ae_core::ecs::UiSlider>().is_some() {
        EntityIcon::Text("🎚️ ")
    } else if ent_ref.get::<&ae_core::ecs::UiCheckbox>().is_some() {
        EntityIcon::Text("☑️ ")
    } else if ent_ref.get::<&ae_core::ecs::UiTextInput>().is_some() {
        EntityIcon::Text("📝 ")
    } else if ent_ref.get::<&ae_core::ecs::Light>().is_some() {
        EntityIcon::Text("💡 ")
    } else if ent_ref.get::<&ae_audio::AudioSource>().is_some() {
        EntityIcon::Text("🔊 ")
    } else if ent_ref.get::<&ae_core::ecs::PlayerTag>().is_some() {
        EntityIcon::Text("🎮 ")
    } else if ent_ref.get::<&ae_core::ecs::Rotator>().is_some() {
        EntityIcon::Text("🔄 ")
    } else if ent_ref.get::<&ae_core::ecs::MovingPlatform>().is_some() {
        EntityIcon::Text("🚡 ")
    } else if ent_ref.get::<&ae_core::ecs::TriggerZone>().is_some() {
        EntityIcon::Text("⚡ ")
    } else if ent_ref.get::<&ae_core::ecs::DestructibleTarget>().is_some() {
        EntityIcon::Text("🎯 ")
    } else if ent_ref.get::<&ae_core::ecs::CharacterAction>().is_some() {
        EntityIcon::Text("🔫 ")
    } else if ent_ref.get::<&ae_core::ecs::SpriteId>().is_some() {
        EntityIcon::Text("🖼 ")
    } else {
        let folder_color = if is_selected {
            Color::rgba(0.0, 0.95, 1.0, 1.0)
        } else {
            Color::rgba(0.96, 0.97, 1.0, 0.92)
        };
        EntityIcon::Texture(HIERARCHY_ICON_FOLDER, folder_color)
    }
}

/// Renders the scrollable tree list of ECS entity rows with O(1) virtualized viewport culling.
pub fn build_hierarchy_rows(
    tree: &mut UiTree,
    parent_id: WidgetId,
    rows: &[HierarchyRow],
    params: &HierarchyPanelParams<'_>,
    targets: &mut HierarchyPanelTargets,
) {
    targets.entity_rows.clear();

    let padding_x = 6.0;
    let list_x = params.panel_rect.x + padding_x;
    let list_y = params.panel_rect.y + 34.0;
    let footer_h = 24.0;
    let list_h = (params.panel_rect.height - 34.0 - footer_h).max(40.0);
    let list_w = params.panel_rect.width - padding_x * 2.0;

    let scroll_rect = Rect::new(list_x, list_y, list_w, list_h);
    targets.scroll_container_rect = scroll_rect;

    let container_id = tree.create_node();
    if let Some(node) = tree.get_mut(container_id) {
        node.set_name("HierarchyRowsContainer");
        node.computed_rect = scroll_rect;
        node.style = Style::new().clip_children(true);
    }
    let _ = tree.add_child(parent_id, container_id);

    let query_lower = params.search_query.trim().to_lowercase();
    let row_h = 24.0;
    let row_gap = 3.0;
    let item_stride = row_h + row_gap;

    let mut rendered_count = 0;

    if query_lower.is_empty() {
        rendered_count = rows.len();
        let total_rows = rows.len();
        let skip_count = if params.scroll_y > 0.0 {
            (params.scroll_y / item_stride).floor() as usize
        } else {
            0
        };
        let start_idx = skip_count.min(total_rows);
        let mut cur_y = list_y - params.scroll_y + (start_idx as f32) * item_stride;

        for row in &rows[start_idx..] {
            if cur_y > list_y + list_h {
                break;
            }

            let row_rect = Rect::new(list_x, cur_y, list_w, row_h);
            if cur_y + row_h >= list_y {
                render_single_row(
                    tree,
                    SingleRowParams {
                        container_id,
                        row,
                        row_rect,
                        cur_y,
                        list_x,
                        list_w,
                        row_h,
                        params,
                    },
                    targets,
                );
            }
            cur_y += item_stride;
        }
    } else {
        let mut cur_y = list_y - params.scroll_y;
        for row in rows {
            let matches = if let Ok(name_comp) = params.world.get::<&ae_core::ecs::Name>(row.entity)
            {
                name_comp.0.to_lowercase().contains(&query_lower)
            } else {
                format!("Entity {:?}", row.entity)
                    .to_lowercase()
                    .contains(&query_lower)
            };
            if !matches {
                continue;
            }

            rendered_count += 1;
            let row_rect = Rect::new(list_x, cur_y, list_w, row_h);

            if cur_y + row_h >= list_y && cur_y <= list_y + list_h {
                render_single_row(
                    tree,
                    SingleRowParams {
                        container_id,
                        row,
                        row_rect,
                        cur_y,
                        list_x,
                        list_w,
                        row_h,
                        params,
                    },
                    targets,
                );
            }
            cur_y += item_stride;
        }
    }

    // Empty State Message
    if rendered_count == 0 {
        let msg_id = tree.create_node();
        let msg_text = if !query_lower.is_empty() {
            "No matching entities found"
        } else {
            "Scene is empty"
        };
        if let Some(node) = tree.get_mut(msg_id) {
            node.set_name("EmptySceneText");
            node.set_text(msg_text);
            node.font_size = 11.0;
            node.line_height = 20.0;
            node.text_align = TextAlign::Center;
            node.text_color = Color::rgba(0.45, 0.48, 0.58, 1.0);
            node.computed_rect = Rect::new(list_x, list_y + 20.0, list_w, 20.0);
        }
        let _ = tree.add_child(container_id, msg_id);
    }
}

/// Parameter descriptor for rendering an individual entity row in the Scene Hierarchy.
struct SingleRowParams<'a, 'b> {
    container_id: WidgetId,
    row: &'a HierarchyRow,
    row_rect: Rect,
    cur_y: f32,
    list_x: f32,
    list_w: f32,
    row_h: f32,
    params: &'a HierarchyPanelParams<'b>,
}

/// Helper function rendering a single interactive entity row in the Scene Hierarchy.
fn render_single_row(
    tree: &mut UiTree,
    ctx: SingleRowParams<'_, '_>,
    targets: &mut HierarchyPanelTargets,
) {
    let container_id = ctx.container_id;
    let row = ctx.row;
    let params = ctx.params;
    let row_rect = ctx.row_rect;
    let cur_y = ctx.cur_y;
    let list_x = ctx.list_x;
    let list_w = ctx.list_w;
    let row_h = ctx.row_h;

    let is_selected = params.selected_entity == Some(row.entity);
    let is_hovered = row_rect.contains_point(params.cursor_pos);

    let row_id = tree.create_node();
    if let Some(node) = tree.get_mut(row_id) {
        node.set_name(format!("EntityRow_{:?}", row.entity));
        node.computed_rect = row_rect;

        let (bg, border, border_w) = if is_selected {
            (
                Color::rgba(0.02, 0.22, 0.32, 0.95), // Dark petrol blue capsule
                Color::rgba(0.0, 0.88, 1.0, 0.95),   // Vibrant Cyan ring #00e5ff
                1.5,
            )
        } else if is_hovered {
            (
                Color::rgba(0.10, 0.14, 0.20, 0.60),
                Color::rgba(0.18, 0.24, 0.35, 0.50),
                1.0,
            )
        } else {
            (Color::TRANSPARENT, Color::TRANSPARENT, 0.0)
        };

        node.style = Style::new()
            .background(bg)
            .border(border_w, border)
            .border_radius(6.0);
    }
    let _ = tree.add_child(container_id, row_id);

    // 1. Hierarchy Tree Connector Lines (Aeon Engine Blue GPU SDF lines for children)
    if row.depth > 0 {
        let tree_line_color = Color::rgba(0.20, 0.55, 0.90, 0.85); // Aeon Engine Blue
        for d in 0..row.depth {
            let stem_x = list_x + 8.5 + d as f32 * 14.0;
            let is_last_level = d == row.depth - 1;

            // Vertical branch line
            let v_h = if is_last_level { row_h * 0.5 } else { row_h };
            let v_id = tree.create_node();
            if let Some(node) = tree.get_mut(v_id) {
                node.set_name("TreeLineVertical");
                node.computed_rect = Rect::new(stem_x, cur_y, 1.2, v_h);
                node.style = Style::new().background(tree_line_color);
            }
            let _ = tree.add_child(row_id, v_id);

            // Horizontal branch arm into icon
            if is_last_level {
                let h_id = tree.create_node();
                if let Some(node) = tree.get_mut(h_id) {
                    node.set_name("TreeLineHorizontal");
                    node.computed_rect = Rect::new(stem_x, cur_y + row_h * 0.5 - 0.6, 9.0, 1.2);
                    node.style = Style::new().background(tree_line_color);
                }
                let _ = tree.add_child(row_id, h_id);
            }
        }
    }

    // 2. Foldout Arrow (for parent nodes)
    let indent = row.depth as f32 * 14.0;
    let prefix_x = list_x + 6.0 + indent;

    if row.has_children {
        let fold_id = tree.create_node();
        if let Some(node) = tree.get_mut(fold_id) {
            node.set_name("FoldoutArrow");
            node.set_text("▼");
            node.font_size = 9.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.65, 0.68, 0.78, 1.0);
            node.computed_rect = Rect::new(prefix_x, cur_y, 10.0, row_h);
        }
        let _ = tree.add_child(row_id, fold_id);
    }

    // 3. Component Icon (Left-aligned column)
    let icon_x = if row.has_children {
        prefix_x + 12.0
    } else {
        prefix_x + 2.0
    };
    let comp_icon = resolve_entity_icon(params.world, row.entity, is_selected);
    let comp_icon_size = 16.0;
    let comp_icon_y = cur_y + (row_h - comp_icon_size) * 0.5;
    let icon_id = tree.create_node();

    if let Some(node) = tree.get_mut(icon_id) {
        node.set_name("ComponentIcon");
        match comp_icon {
            EntityIcon::Texture(uv, tint) => {
                node.computed_rect = Rect::new(icon_x, comp_icon_y, comp_icon_size, comp_icon_size);
                node.set_texture_uv(uv);
                node.set_texture_tint(tint);
            }
            EntityIcon::Text(icon_str) => {
                node.computed_rect = Rect::new(icon_x, cur_y, 16.0, row_h);
                node.set_text(icon_str);
                node.font_size = 12.0;
                node.line_height = row_h;
                let icon_color = if is_selected {
                    Color::rgba(0.0, 0.95, 1.0, 1.0)
                } else {
                    Color::WHITE
                };
                node.text_color = icon_color;
            }
        }
    }
    let _ = tree.add_child(row_id, icon_id);

    // 4. Entity Name Text (Centered horizontally across the row)
    let name_id = tree.create_node();
    let text_color = if is_selected {
        Color::rgba(0.0, 0.95, 1.0, 1.0) // Bright cyan #00e5ff
    } else {
        Color::rgba(0.88, 0.91, 0.98, 1.0) // Crisp slate white
    };

    if let Some(node) = tree.get_mut(name_id) {
        node.set_name("EntityName");
        if let Ok(name_comp) = params.world.get::<&ae_core::ecs::Name>(row.entity) {
            node.set_text(&name_comp.0);
        } else {
            node.set_text(format!("Entity {:?}", row.entity));
        }
        node.font_size = 11.5;
        node.line_height = row_h;
        node.text_align = TextAlign::Center;
        node.text_color = text_color;
        node.computed_rect = Rect::new(list_x + 28.0, cur_y, list_w - 56.0, row_h);
    }
    let _ = tree.add_child(row_id, name_id);

    // 5. Eye Visibility Button (Right-aligned edge column)
    let eye_w = 22.0;
    let eye_rect = Rect::new(list_x + list_w - eye_w - 2.0, cur_y, eye_w, row_h);
    let is_visible = params
        .world
        .get::<&ae_core::ecs::Hidden>(row.entity)
        .is_err();

    let (eye_uv, eye_col) = if !is_visible {
        (HIERARCHY_ICON_EYE_CLOSED, Color::rgba(1.0, 1.0, 1.0, 1.0))
    } else if is_selected {
        (HIERARCHY_ICON_EYE_OPEN, Color::rgba(0.0, 0.95, 1.0, 1.0))
    } else {
        (HIERARCHY_ICON_EYE_OPEN, Color::rgba(0.88, 0.91, 0.98, 0.95))
    };

    let eye_id = tree.create_node();
    let eye_icon_size = 18.0;
    let eye_icon_x = eye_rect.x + (eye_w - eye_icon_size) * 0.5;
    let eye_icon_y = eye_rect.y + (row_h - eye_icon_size) * 0.5;
    if let Some(node) = tree.get_mut(eye_id) {
        node.set_name("EyeVisibilityButton");
        node.computed_rect = Rect::new(eye_icon_x, eye_icon_y, eye_icon_size, eye_icon_size);
        node.set_texture_uv(eye_uv);
        node.set_texture_tint(eye_col);
    }
    let _ = tree.add_child(row_id, eye_id);

    targets
        .entity_rows
        .push((row.entity, row_rect, eye_rect, None));
}