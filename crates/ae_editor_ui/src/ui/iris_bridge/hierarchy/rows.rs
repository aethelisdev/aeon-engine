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

    let max_cap = entity_count.min(25_000);
    let mut name_map: HashMap<hecs::Entity, String> = HashMap::with_capacity(max_cap);
    let mut parent_map: HashMap<hecs::Entity, hecs::Entity> = HashMap::with_capacity(max_cap);
    let mut children_map: HashMap<hecs::Entity, Vec<hecs::Entity>> =
        HashMap::with_capacity(max_cap);
    let mut icon_map: HashMap<hecs::Entity, &'static str> = HashMap::with_capacity(max_cap);
    let mut visibility_map: HashMap<hecs::Entity, bool> = HashMap::with_capacity(max_cap);

    for (processed, ent_ref) in world.iter().enumerate() {
        if processed >= 25_000 {
            break;
        }
        let ent = ent_ref.entity();

        // Skip internal pause menu UI entities from Scene Hierarchy tree
        if ent_ref.get::<&ae_core::ui::PauseMenuUiTag>().is_some() {
            continue;
        }

        // Entity Display Name
        let name = ent_ref
            .get::<&ae_core::ecs::Name>()
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("Entity {:?}", ent));
        name_map.insert(ent, name);

        // Parent Link
        if let Some(p) = ent_ref.get::<&ae_core::ecs::Parent>()
            && world.contains(p.0)
        {
            parent_map.insert(ent, p.0);
        }

        // Children Links
        if let Some(c) = ent_ref.get::<&ae_core::ecs::Children>() {
            let valid_children: Vec<hecs::Entity> =
                c.0.iter()
                    .copied()
                    .filter(|&ch| world.contains(ch))
                    .collect();
            if !valid_children.is_empty() {
                children_map.insert(ent, valid_children);
            }
        }

        // 100% Data-Driven Component Icon Assignment
        let icon = if ent_ref.get::<&ae_core::ecs::PlayerHealthBarTag>().is_some() {
            "❤️ "
        } else if ent_ref.get::<&ae_core::ecs::ScoreDisplayTag>().is_some() {
            "⭐ "
        } else if ent_ref.get::<&ae_core::ecs::ReticleTag>().is_some() {
            "🎯 "
        } else if ent_ref.get::<&ae_core::ecs::UiProgressBar>().is_some() {
            "📊 "
        } else if ent_ref.get::<&ae_core::ecs::UiButton>().is_some() {
            "🔘 "
        } else if ent_ref.get::<&ae_core::ecs::UiText>().is_some() {
            "🔤 "
        } else if ent_ref.get::<&ae_core::ecs::UiImage>().is_some() {
            "🖼️ "
        } else if ent_ref.get::<&ae_core::ecs::UiSlider>().is_some() {
            "🎚️ "
        } else if ent_ref.get::<&ae_core::ecs::UiCheckbox>().is_some() {
            "☑️ "
        } else if ent_ref.get::<&ae_core::ecs::UiTextInput>().is_some() {
            "📝 "
        } else if ent_ref.get::<&ae_core::ecs::UiPanel>().is_some() {
            "🟩 "
        } else if ent_ref.get::<&ae_core::ecs::Light>().is_some() {
            "💡 "
        } else if ent_ref.get::<&ae_audio::AudioSource>().is_some() {
            "🔊 "
        } else if ent_ref.get::<&ae_core::ecs::PlayerTag>().is_some() {
            "🎮 "
        } else if ent_ref.get::<&ae_core::ecs::Rotator>().is_some() {
            "🔄 "
        } else if ent_ref.get::<&ae_core::ecs::MovingPlatform>().is_some() {
            "🚡 "
        } else if ent_ref.get::<&ae_core::ecs::TriggerZone>().is_some() {
            "⚡ "
        } else if ent_ref.get::<&ae_core::ecs::DestructibleTarget>().is_some() {
            "🎯 "
        } else if ent_ref.get::<&ae_core::ecs::CharacterAction>().is_some() {
            "🔫 "
        } else if ent_ref.get::<&ae_core::ecs::ModelId>().is_some() {
            "📦 "
        } else if ent_ref.get::<&ae_core::ecs::Shape>().is_some() {
            "🎲 "
        } else if ent_ref.get::<&ae_core::ecs::SpriteId>().is_some() {
            "🖼 "
        } else {
            "📁 "
        };
        icon_map.insert(ent, icon);

        // Visibility
        let is_visible = ent_ref.get::<&ae_core::ecs::Hidden>().is_none();
        visibility_map.insert(ent, is_visible);
    }

    // Two-way synchronization: ensure parent_map links exist in children_map
    for (&child, &parent) in &parent_map {
        let list = children_map.entry(parent).or_default();
        if !list.contains(&child) {
            list.push(child);
        }
    }

    // Collect roots
    let mut roots: Vec<hecs::Entity> = name_map
        .keys()
        .copied()
        .filter(|ent| !parent_map.contains_key(ent))
        .collect();
    roots.sort_by_key(|e| e.id());

    // DFS Traversal
    let mut stack: Vec<(hecs::Entity, usize)> = Vec::with_capacity(roots.len() * 2);
    for &root in roots.iter().rev() {
        stack.push((root, 0));
    }

    while let Some((ent, depth)) = stack.pop() {
        let name = name_map.remove(&ent).unwrap_or_default();
        let has_children = children_map
            .get(&ent)
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        let icon = icon_map.get(&ent).copied().unwrap_or("📁 ");
        let is_visible = visibility_map.get(&ent).copied().unwrap_or(true);

        out_rows.push(HierarchyRow {
            entity: ent,
            name,
            depth,
            has_children,
            icon,
            is_visible,
        });

        if let Some(children) = children_map.get(&ent) {
            for &child in children.iter().rev() {
                stack.push((child, depth + 1));
            }
        }
    }
}

/// Renders the scrollable tree list of ECS entity rows.
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
    let mut cur_y = list_y - params.scroll_y;

    let mut rendered_count = 0;

    for row in rows {
        if !query_lower.is_empty() && !row.name.to_lowercase().contains(&query_lower) {
            continue;
        }

        rendered_count += 1;
        let row_rect = Rect::new(list_x, cur_y, list_w, row_h);

        // Cull rows outside vertical viewport
        if cur_y + row_h >= list_y && cur_y <= list_y + list_h {
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
                            node.computed_rect =
                                Rect::new(stem_x, cur_y + row_h * 0.5 - 0.6, 9.0, 1.2);
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
            let icon_id = tree.create_node();
            let icon_color = if is_selected {
                Color::rgba(0.0, 0.95, 1.0, 1.0) // Cyan highlight
            } else {
                Color::WHITE
            };

            if let Some(node) = tree.get_mut(icon_id) {
                node.set_name("ComponentIcon");
                node.set_text(row.icon);
                node.font_size = 12.0;
                node.line_height = row_h;
                node.text_color = icon_color;
                node.computed_rect = Rect::new(icon_x, cur_y, 16.0, row_h);
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
                node.set_text(&row.name);
                node.font_size = 11.5;
                node.line_height = row_h;
                node.text_align = TextAlign::Center;
                node.text_color = text_color;
                node.computed_rect = Rect::new(list_x + 28.0, cur_y, list_w - 56.0, row_h);
            }
            let _ = tree.add_child(row_id, name_id);

            // 5. Eye Visibility Button (Right-aligned edge column)
            let eye_w = 20.0;
            let eye_rect = Rect::new(list_x + list_w - eye_w - 2.0, cur_y, eye_w, row_h);
            let (eye_icon, eye_col) = if is_selected {
                ("👁", Color::rgba(0.0, 0.95, 1.0, 1.0))
            } else if row.is_visible {
                ("👁", Color::rgba(0.75, 0.78, 0.88, 0.90))
            } else {
                ("🚫", Color::rgba(0.92, 0.28, 0.28, 0.95))
            };

            let eye_id = tree.create_node();
            if let Some(node) = tree.get_mut(eye_id) {
                node.set_name("EyeVisibilityButton");
                node.set_text(eye_icon);
                node.font_size = 11.0;
                node.line_height = row_h;
                node.text_align = TextAlign::Center;
                node.text_color = eye_col;
                node.computed_rect = eye_rect;
            }
            let _ = tree.add_child(row_id, eye_id);

            targets
                .entity_rows
                .push((row.entity, row_rect, eye_rect, None));
        }

        cur_y += row_h + row_gap;
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