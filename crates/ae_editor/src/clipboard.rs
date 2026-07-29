// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::editor_state::EditorState;
use crate::undo_redo::{Command, EntitySnapshot};
use ae_core::ecs::{Name, Position};

/// Copies all currently selected entities into the editor clipboard buffer (`editor.clipboard`).
pub fn copy_selected(world: &hecs::World, editor: &mut EditorState) {
    if editor.selected_entities.is_empty() {
        return;
    }

    editor.clipboard.clear();
    for &ent in &editor.selected_entities {
        if world.contains(ent) {
            let snap = EntitySnapshot::capture(world, ent);
            editor.clipboard.push(snap);
        }
    }
    log::info!(
        "Copied {} entity snapshot(s) to clipboard.",
        editor.clipboard.len()
    );
}

/// Pastes all entity snapshots from `editor.clipboard` into the ECS world.
/// New entities receive a slight position offset (+0.5, +0.5) and a " (Copy)" suffix.
/// Pushes an undoable `Command::Spawn` or `Command::Batch` onto `editor.undo_stack`
/// and updates the selection to the newly pasted entities.
pub fn paste_clipboard(
    world: &mut hecs::World,
    editor: &mut EditorState,
    ui_selected_entity: &mut Option<hecs::Entity>,
) {
    if editor.clipboard.is_empty() {
        return;
    }

    let mut batch = Vec::new();
    let clipboard_snapshots = editor.clipboard.clone();

    // Clear existing selection and select the newly spawned copies
    editor.selected_entities.clear();
    editor.selected_entities_set.clear();

    for snap in clipboard_snapshots {
        let new_ent = snap.spawn(world);

        // Apply slight offset to avoid exact position overlap
        if let Ok(mut pos) = world.get::<&mut Position>(new_ent) {
            pos.x += 0.5;
            pos.y += 0.5;
        }

        // Append (Copy) suffix to name
        if let Ok(mut name) = world.get::<&mut Name>(new_ent) {
            if !name.0.ends_with("(Copy)") {
                name.0 = format!("{} (Copy)", name.0);
            }
        }

        let new_snap = EntitySnapshot::capture(world, new_ent);
        batch.push(Command::Spawn(new_ent, new_snap));

        editor.selected_entities.push(new_ent);
        editor.selected_entities_set.insert(new_ent);
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

    log::info!(
        "Pasted {} entity(ies) from clipboard.",
        editor.selected_entities.len()
    );
}

/// Triggers inline renaming mode (F2) for the currently selected entity.
pub fn rename_selected(editor: &mut EditorState) {
    if !editor.selected_entities.is_empty() {
        editor.focus_rename = true;
        log::info!("Triggered inline rename (F2) for selected entity.");
    }
}