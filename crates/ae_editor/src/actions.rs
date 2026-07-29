// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::editor_state::EditorState;
use crate::undo_redo::{Command, EntitySnapshot};
use ae_core::ecs::{
    BoundingBox, Children, Color, GlobalTransform, Light, ModelId, Name, Parent, PlayerTag,
    Position, Rotation, Scale, Shape, SpriteId, Velocity,
};

/// Deletes all currently selected entities from the ECS world.
/// Captures an `EntitySnapshot` for each entity before despawning to enable undo.
pub fn delete_selected(
    world: &mut hecs::World,
    editor: &mut EditorState,
    ui_selected_entity: &mut Option<hecs::Entity>,
) {
    if editor.selected_entities.is_empty() {
        return;
    }

    let mut batch = Vec::new();
    let to_delete = editor.selected_entities.clone();

    for ent in to_delete {
        if world.contains(ent) {
            // 1. Clean up parent's children list and prevent stale/stuck positions
            let parent_opt = if let Ok(parent_ref) = world.get::<&Parent>(ent) {
                Some(parent_ref.0)
            } else {
                None
            };

            if let Some(parent) = parent_opt {
                let mut remove_parent_children = false;
                let mut remove_parent_gt = false;

                if let Ok(mut children_ref) = world.get::<&mut Children>(parent) {
                    children_ref.0.retain(|&e| e != ent);
                    if children_ref.0.is_empty() {
                        remove_parent_children = true;
                        if world.get::<&Parent>(parent).is_err() {
                            remove_parent_gt = true;
                        }
                    }
                }

                if remove_parent_children {
                    let _ = world.remove_one::<Children>(parent);
                }
                if remove_parent_gt {
                    let _ = world.remove_one::<GlobalTransform>(parent);
                }
            }

            // 2. Unparent any children first to prevent stale hierarchy components and stuck GlobalTransforms
            let children_list = if let Ok(children_ref) = world.get::<&Children>(ent) {
                children_ref.0.clone()
            } else {
                Vec::new()
            };

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

    // Sync UI selection state
    *ui_selected_entity = None;

    log::info!("Deleted selected entities and cleared selection state.");
}

/// Duplicates all currently selected entities with a position offset.
pub fn duplicate_selected(
    world: &mut hecs::World,
    editor: &mut EditorState,
    ui_selected_entity: &mut Option<hecs::Entity>,
) {
    if editor.selected_entities.is_empty() {
        return;
    }

    let mut batch = Vec::new();
    let to_duplicate = editor.selected_entities.clone();

    // Prepare new selection
    editor.selected_entities.clear();
    editor.selected_entities_set.clear();

    for ent in to_duplicate {
        if world.contains(ent) {
            let mut builder = hecs::EntityBuilder::new();

            if let Ok(name) = world.get::<&Name>(ent) {
                builder.add(Name(format!("{} (Copy)", name.0)));
            } else {
                builder.add(Name("Entity (Copy)".to_string()));
            }

            if let Ok(pos) = world.get::<&Position>(ent) {
                builder.add(Position {
                    x: pos.x + 0.5,
                    y: pos.y + 0.5,
                    z: pos.z,
                });
            }
            if let Ok(rot) = world.get::<&Rotation>(ent) {
                builder.add(*rot);
            }
            if let Ok(scl) = world.get::<&Scale>(ent) {
                builder.add(*scl);
            }
            if let Ok(col) = world.get::<&Color>(ent) {
                builder.add(*col);
            }
            if let Ok(vel) = world.get::<&Velocity>(ent) {
                builder.add(*vel);
            }
            if let Ok(shape) = world.get::<&Shape>(ent) {
                builder.add(*shape);
            }
            if let Ok(mid) = world.get::<&ModelId>(ent) {
                builder.add(*mid);
            }
            if let Ok(sid) = world.get::<&SpriteId>(ent) {
                builder.add(*sid);
            }
            if let Ok(l) = world.get::<&Light>(ent) {
                builder.add(*l);
            }
            if let Ok(bbox) = world.get::<&BoundingBox>(ent) {
                builder.add(*bbox);
            }
            if world.get::<&PlayerTag>(ent).is_ok() {
                builder.add(PlayerTag);
            }

            let new_ent = world.spawn(builder.build());
            let snap = EntitySnapshot::capture(world, new_ent);
            batch.push(Command::Spawn(new_ent, snap));

            editor.selected_entities.push(new_ent);
            editor.selected_entities_set.insert(new_ent);
        }
    }

    *ui_selected_entity = editor.selected_entities.first().copied();

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