// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::editor_state::EditorState;
use crate::undo_redo::{Command, EntitySnapshot, Property};

/// Pushes a new command to the undo stack, dropping the oldest if the limit is exceeded.
pub fn push_undo(editor: &mut EditorState, cmd: Command) {
    editor.undo_stack.push(cmd);
    let limit = editor.config.max_undo_history;
    if editor.undo_stack.len() > limit {
        editor.undo_stack.remove(0);
    }
}

/// Pops the most recent command from the undo stack and reverses it.
/// After undoing, the command is moved to the redo stack for potential re-application.
/// If the undo operation changes an Entity ID (e.g., re-spawning a deleted entity),
/// all remaining stack commands and the current selection are remapped to the new ID.
pub fn undo(editor: &mut EditorState, world: &mut hecs::World) {
    if let Some(mut cmd) = editor.undo_stack.pop() {
        let remap = cmd.undo(world);
        editor.redo_stack.push(cmd);

        // If an Entity ID change occurred, update all stacks and the selection state.
        if let Some((old_id, new_id)) = remap {
            for cmd in &mut editor.undo_stack {
                cmd.remap_entity(old_id, new_id);
            }
            for cmd in &mut editor.redo_stack {
                cmd.remap_entity(old_id, new_id);
            }
            for ent in &mut editor.selected_entities {
                if *ent == old_id {
                    *ent = new_id;
                }
            }
            log::debug!("Entity ID remapped in undo: {:?} → {:?}", old_id, new_id);
        }
    }
}

/// Pops the most recent command from the redo stack and re-applies it.
/// After redoing, the command is pushed back onto the undo stack via `push_undo`.
/// Entity ID remapping is performed identically to `undo()` to maintain
/// referential integrity across all stacks and selection state.
pub fn redo(editor: &mut EditorState, world: &mut hecs::World) {
    if let Some(mut cmd) = editor.redo_stack.pop() {
        let remap = cmd.redo(world);
        push_undo(editor, cmd);

        if let Some((old_id, new_id)) = remap {
            for cmd in &mut editor.undo_stack {
                cmd.remap_entity(old_id, new_id);
            }
            for cmd in &mut editor.redo_stack {
                cmd.remap_entity(old_id, new_id);
            }
            for ent in &mut editor.selected_entities {
                if *ent == old_id {
                    *ent = new_id;
                }
            }
            log::debug!("Entity ID remapped in redo: {:?} → {:?}", old_id, new_id);
        }
    }
}

/// Commits accumulated edit snapshots as undo history commands.
/// Called when a gizmo drag ends. Compares the pre-drag snapshots
/// (`current_edit_snapshots`) against current ECS state to produce
/// per-property `Command::Modify` entries. Multiple entity edits
/// within a single drag are batched into a `Command::Batch`.
/// The redo stack is cleared after committing new history.
pub fn commit_undo_history(
    editor: &mut EditorState,
    world: &hecs::World,
    _primary_entity: hecs::Entity,
) {
    if editor.current_edit_snapshots.is_empty() {
        return;
    }

    let mut batch = Vec::new();

    for (entity, old_snap) in editor.current_edit_snapshots.drain() {
        let new_snap = EntitySnapshot::capture(world, entity);

        if let (Some(os), Some(ns)) = (old_snap.pos, new_snap.pos) {
            if (os.x - ns.x).abs() > 0.001
                || (os.y - ns.y).abs() > 0.001
                || (os.z - ns.z).abs() > 0.001
            {
                batch.push(Command::Modify(entity, Property::Position(os, ns)));
            }
        }
        if let (Some(os), Some(ns)) = (old_snap.rot, new_snap.rot) {
            if (os.x - ns.x).abs() > 0.0001
                || (os.y - ns.y).abs() > 0.0001
                || (os.z - ns.z).abs() > 0.0001
                || (os.w - ns.w).abs() > 0.0001
            {
                batch.push(Command::Modify(entity, Property::Rotation(os, ns)));
            }
        }
        if let (Some(os), Some(ns)) = (old_snap.scale, new_snap.scale) {
            if (os.x - ns.x).abs() > 0.001
                || (os.y - ns.y).abs() > 0.001
                || (os.z - ns.z).abs() > 0.001
            {
                batch.push(Command::Modify(entity, Property::Scale(os, ns)));
            }
        }
    }

    if !batch.is_empty() {
        if batch.len() == 1 {
            push_undo(editor, batch.remove(0));
        } else {
            push_undo(editor, Command::Batch(batch));
        }
        editor.redo_stack.clear();
    }
}