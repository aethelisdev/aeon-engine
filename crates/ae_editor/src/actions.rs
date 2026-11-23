// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::editor_state::EditorState;
use crate::undo_redo::{Command, EntitySnapshot};
use ae_plugin_api::{
    BoundingBox, Children, Color, GlobalTransform, Light, ModelId, Name, Parent, PlayerTag,
    Position, Rotation, Scale, Shape, SpriteId, Velocity,
};

/// Deletes all currently selected entities from the ECS world.
///
/// Captures an `EntitySnapshot` for each entity before despawning to enable undo.
pub fn delete_selected(world: &mut ae_ecs::World, editor: &mut EditorState) {
    if editor.selected_entities.is_empty() {
        return;
    }

    let mut batch = Vec::new();
    let to_delete = std::mem::take(&mut editor.selected_entities);

    for ent in to_delete {
        if world.contains(ent) {
            // 1. Clean up parent's children list and prevent stale/stuck positions
            let parent_opt = world.get_copied::<Parent>(ent).ok().map(|p| p.0);

            if let Some(parent) = parent_opt {
                let mut remove_parent_children = false;

                if let Ok(children_ref) = world.get_mut::<Children>(parent) {
                    children_ref.0.retain(|&e| e != ent);
                    if children_ref.0.is_empty() {
                        remove_parent_children = true;
                    }
                }

                let remove_parent_gt = remove_parent_children && !world.has::<Parent>(parent);

                if remove_parent_children {
                    let _ = world.remove_one::<Children>(parent);
                }
                if remove_parent_gt {
                    let _ = world.remove_one::<GlobalTransform>(parent);
                }
            }

            // 2. Unparent any children first to prevent stale hierarchy components and stuck GlobalTransforms
            let children_list = world
                .inspect::<Children, _>(ent, |children_ref| children_ref.0.clone())
                .unwrap_or_default();

            for child in children_list {
                let _ = world.remove_one::<Parent>(child);
                let _ = world.remove_one::<GlobalTransform>(child);
            }

            let snap = EntitySnapshot::capture(world, ent);
            batch.push(Command::Delete(ent, snap));
            let _ = world.despawn(ent);
        }
    }

    if !batch.is_empty() {
        if batch.len() == 1 {
            editor.undo_stack.push(batch.remove(0));
        } else {
            editor.undo_stack.push(Command::Batch(batch));
        }
        editor.redo_stack.clear();
    }

    // Clean up all selection and interaction states to avoid "ghost" references
    editor.selected_entities.clear();
    editor.selected_entities_set.clear();
    editor.current_edit_snapshots.clear();
    editor.multi_start_positions.clear();
    editor.multi_start_rotations.clear();
    editor.multi_start_scales.clear();
    editor.gizmo_dragging = false;

    log::info!("Deleted selected entities and cleared selection state.");
}

/// Duplicates all currently selected entities with a position offset.
pub fn duplicate_selected(world: &mut ae_ecs::World, editor: &mut EditorState) {
    if editor.selected_entities.is_empty() {
        return;
    }

    let mut batch = Vec::new();
    let to_duplicate = std::mem::take(&mut editor.selected_entities);

    // Prepare new selection
    editor.selected_entities_set.clear();

    for ent in to_duplicate {
        if world.contains(ent) {
            let mut builder = ae_ecs::EntityBuilder::new();

            if let Ok(name_copy) = world.inspect::<Name, _>(ent, |n| format!("{} (Copy)", n.0)) {
                builder.add(Name(name_copy));
            } else {
                builder.add(Name("Entity (Copy)".to_string()));
            }

            if let Ok(pos) = world.get_copied::<Position>(ent) {
                builder.add(Position {
                    x: pos.x + 0.5,
                    y: pos.y + 0.5,
                    z: pos.z,
                });
            }
            if let Ok(rot) = world.get_copied::<Rotation>(ent) {
                builder.add(rot);
            }
            if let Ok(scl) = world.get_copied::<Scale>(ent) {
                builder.add(scl);
            }
            if let Ok(col) = world.get_copied::<Color>(ent) {
                builder.add(col);
            }
            if let Ok(vel) = world.get_copied::<Velocity>(ent) {
                builder.add(vel);
            }
            if let Ok(shape) = world.get_copied::<Shape>(ent) {
                builder.add(shape);
            }
            if let Ok(mid) = world.get_copied::<ModelId>(ent) {
                builder.add(mid);
            }
            if let Ok(sid) = world.get_copied::<SpriteId>(ent) {
                builder.add(sid);
            }
            if let Ok(l) = world.get_copied::<Light>(ent) {
                builder.add(l);
            }
            if let Ok(bbox) = world.get_copied::<BoundingBox>(ent) {
                builder.add(bbox);
            }
            if world.has::<PlayerTag>(ent) {
                builder.add(PlayerTag);
            }

            let new_ent = world.spawn(builder.build());
            let snap = EntitySnapshot::capture(world, new_ent);
            batch.push(Command::Spawn(new_ent, snap));

            editor.selected_entities.push(new_ent);
            editor.selected_entities_set.insert(new_ent);
        }
    }

    if !batch.is_empty() {
        if batch.len() == 1 {
            editor.undo_stack.push(batch.remove(0));
        } else {
            editor.undo_stack.push(Command::Batch(batch));
        }
        editor.redo_stack.clear();
    }

    log::info!("Duplicated {} entities.", editor.selected_entities.len());
}