// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Host scene/prefab OS dialog orchestration and partitioned world persistence polling.

use crate::session::EditorSession;

fn resolve_pending_world_path(
    pending: &mut Option<std::path::PathBuf>,
    active_scene_path: Option<&std::path::PathBuf>,
) -> std::path::PathBuf {
    pending
        .take()
        .or_else(|| active_scene_path.cloned())
        .unwrap_or_else(|| {
            std::path::PathBuf::from(ae_world::ProjectWorkspaceLayout::DEFAULT_ENTRY_WORLD_REL)
        })
}

fn poll_pending_scene_save(engine: &mut EditorSession) {
    if !engine.editor.should_save_scene {
        return;
    }
    engine.editor.should_save_scene = false;
    let path = resolve_pending_world_path(
        &mut engine.editor.pending_save_path,
        engine.editor.active_scene_path.as_ref(),
    );
    let path_str = path.to_string_lossy().into_owned();
    match ae_scene::save_partitioned_world(
        &mut engine.core.world,
        &engine.render.asset_manager,
        &mut engine.core.active_world_guid,
        ae_scene::WorldSaveRequest {
            filepath: &path,
            is_2d: engine.render.dimension_mode.is_2d(),
            prev_saved_world_dir: engine.host_io.saved_world_dir.as_deref(),
            prev_cell_crcs: &engine.host_io.last_saved_cell_crcs,
        },
    ) {
        Ok(saved) => {
            engine.host_io.saved_world_dir = Some(saved.saved_world_dir);
            engine.host_io.last_saved_cell_crcs = saved.cell_crcs;
            engine.editor.active_scene_path = Some(saved.manifest_path);
            engine.host_io.autosave.reset_timer();
            engine.ui.set_status_message(
                format!("Scene saved to {path_str}"),
                irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
            );
        }
        Err(e) => {
            engine
                .ui
                .set_status_message(format!("Save Error: {e}"), irisui::prelude::Color::RED);
        }
    }
}

fn poll_pending_scene_load(engine: &mut EditorSession) {
    if !engine.editor.should_load_scene {
        return;
    }
    engine.editor.should_load_scene = false;
    let path = resolve_pending_world_path(
        &mut engine.editor.pending_load_path,
        engine.editor.active_scene_path.as_ref(),
    );
    match ae_scene::begin_async_scene_load(&path, engine.render.dimension_mode.is_2d()) {
        Ok(rx) => {
            engine.editor.pending_load_path = Some(path);
            engine.editor.is_loading_assets = true;
            engine.host_io.scene_rx = Some(rx);
            engine.ui.set_status_message(
                "Loading world: parsing assets in background...",
                irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
            );
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                log::error!("{e}");
                let ui_msg = if engine.render.dimension_mode.is_2d() {
                    "Cannot load 3D world in 2D mode! Switch to 3D mode."
                } else {
                    "Cannot load 2D world in 3D mode! Switch to 2D mode."
                };
                engine
                    .ui
                    .set_status_message(ui_msg, irisui::prelude::Color::RED);
            } else {
                engine
                    .ui
                    .set_status_message(format!("Load Error: {e}"), irisui::prelude::Color::RED);
            }
            engine.editor.is_loading_assets = false;
        }
    }
}

fn poll_async_scene_load_result(engine: &mut EditorSession) {
    let Some(rx) = engine.host_io.scene_rx.as_ref() else {
        return;
    };
    let Ok(result) = rx.try_recv() else {
        return;
    };
    engine.host_io.scene_rx = None;
    let requested_path = engine.editor.pending_load_path.take();
    match result {
        Ok(scene_data) => {
            engine.sim.physics_world.clear();
            engine.editor.select_single(None);
            engine.editor.current_edit_snapshots.clear();
            engine.editor.multi_start_positions.clear();
            engine.editor.multi_start_rotations.clear();
            engine.editor.multi_start_scales.clear();
            engine.editor.gizmo_dragging = false;
            engine.editor.undo_stack.clear();
            engine.editor.redo_stack.clear();
            engine.ui.clear_interaction_history();
            engine.host_io.autosave.reset_timer();

            let applied = ae_scene::apply_loaded_scene_data(
                scene_data,
                &mut engine.core.world,
                &engine.render.render_state,
                &mut engine.render.asset_manager,
                engine.host_io.active_vfs.as_deref(),
            );
            engine.editor.active_scene_path = applied
                .saved_world_dir
                .as_ref()
                .map(|dir| dir.join("world.aeworld"))
                .or(requested_path);
            engine.host_io.saved_world_dir = applied.saved_world_dir;
            engine.core.active_world_guid = applied.world_guid;
            engine.host_io.last_saved_cell_crcs = applied.cell_crcs;
            engine.sim.streaming = applied.streaming;

            engine
                .sim
                .physics_world
                .sync_ecs_to_physics(&mut engine.core.world, |handle| {
                    engine
                        .render
                        .asset_manager
                        .get_physics_mesh_data(handle)
                        .map(|(v, i)| (v.as_slice(), i.as_slice()))
                });
            engine.editor.is_loading_assets = false;
            engine.ui.set_status_message(
                "World loaded successfully!",
                irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
            );
            log::info!("Async binary world load processed cleanly.");
        }
        Err(e) => {
            engine.editor.is_loading_assets = false;
            engine.ui.set_status_message(
                format!("Async world load failed: {e}"),
                irisui::prelude::Color::RED,
            );
        }
    }
}

/// Processes queued editor world save/load requests and polls async world loading completion.
pub fn process_pending_scene_io(engine: &mut EditorSession) {
    poll_pending_scene_save(engine);
    poll_pending_scene_load(engine);
    poll_async_scene_load_result(engine);
}

/// Opens an OS file dialog to instantiate an `.aeprefab` archive.
pub fn handle_open_load_prefab_dialog(engine: &mut EditorSession) {
    #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        engine.host_io.import_receivers.push(rx);
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Aeon Prefab (*.aeprefab)", &["aeprefab"])
                .pick_file()
            {
                let _ = tx.send(ae_assets::AsyncImportResult::PickedPath {
                    path,
                    spawn_pos: [0.0, 0.0, 0.0],
                });
            }
        });
    }
    #[cfg(not(all(feature = "editor-dialogs", not(target_os = "android"))))]
    let _ = engine;
}

/// Opens an OS file dialog to save the active `.aeworld` archive.
pub fn handle_open_save_scene_dialog(engine: &mut EditorSession) {
    #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        engine.editor.scene_dialog_receivers.push(rx);
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Aeon Binary World (*.aeworld)", &["aeworld"])
                .set_file_name("world.aeworld")
                .save_file()
            {
                let _ = tx.send(ae_editor::editor_state::SceneDialogAction::SaveTo(path));
            }
        });
    }
    #[cfg(not(all(feature = "editor-dialogs", not(target_os = "android"))))]
    let _ = engine;
}

/// Opens an OS file dialog to load an `.aeworld` archive.
pub fn handle_open_load_scene_dialog(engine: &mut EditorSession) {
    #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        engine.editor.scene_dialog_receivers.push(rx);
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Aeon Binary World (*.aeworld)", &["aeworld"])
                .pick_file()
            {
                let _ = tx.send(ae_editor::editor_state::SceneDialogAction::LoadFrom(path));
            }
        });
    }
    #[cfg(not(all(feature = "editor-dialogs", not(target_os = "android"))))]
    let _ = engine;
}

/// Queues a synchronous save trigger on editor state targeting `path`.
pub fn handle_save_scene_to_path(engine: &mut EditorSession, path: std::path::PathBuf) {
    log::info!("💾 Scene save request queued for path {:?}", path);
    engine.editor.pending_save_path = Some(path);
    engine.editor.should_save_scene = true;
}

/// Queues an asynchronous scene load request targeting `path`.
pub fn handle_load_scene_from_path(engine: &mut EditorSession, path: std::path::PathBuf) {
    log::info!("📂 Scene load request queued for path {:?}", path);
    engine.editor.pending_load_path = Some(path);
    engine.editor.should_load_scene = true;
    engine.editor.is_loading_assets = true;
}

/// Captures `entity` into an `.aeprefab` archive via [`ae_scene::Prefab`] and writes it to `path`.
pub fn handle_save_entity_as_prefab(
    engine: &mut EditorSession,
    entity: ae_ecs::Entity,
    path: std::path::PathBuf,
) {
    let prefab = ae_scene::Prefab::create_from_entity(&engine.core.world, entity);
    if let Err(e) = prefab.save_to_file(&path) {
        log::error!("Failed to save prefab to {:?}: {}", path, e);
        engine.ui.set_status_message(
            format!("Failed to save prefab: {}", e),
            irisui::prelude::Color::rgba(0.95, 0.35, 0.35, 1.0),
        );
    } else {
        log::info!("📦 Prefab successfully saved to {:?}", path);
        engine.ui.set_status_message(
            format!("Saved prefab to {}", path.display()),
            irisui::prelude::Color::rgba(0.25, 0.85, 0.45, 1.0),
        );
    }
}

/// Loads an `.aeprefab` archive from disk, instantiates it into `world`, and selects the root.
pub fn handle_instantiate_prefab(engine: &mut EditorSession, path: std::path::PathBuf) {
    match ae_scene::Prefab::load_from_file(&path) {
        Ok(prefab) => {
            let new_entity = prefab.instantiate(&mut engine.core.world, None);
            engine.editor.select_single(Some(new_entity));
            log::info!("✨ Instantiated prefab from {:?}", path);
            let label = ae_scene::resolve_asset_display_name(&path, "Prefab");
            engine.ui.set_status_message(
                format!("Prefab '{label}' loaded successfully!"),
                irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
            );
        }
        Err(e) => {
            log::error!("Failed to instantiate prefab from {:?}: {}", path, e);
            engine.ui.set_status_message(
                format!("Error loading prefab: {e}"),
                irisui::prelude::Color::RED,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_editor::editor_state::{EditorState, SceneDialogAction};
    use std::path::PathBuf;

    #[test]
    fn test_resolve_pending_world_path_priority_and_default_fallback() {
        let mut pending = Some(PathBuf::from("Worlds/Pending/world.aeworld"));
        let active = PathBuf::from("Worlds/Active/world.aeworld");

        assert_eq!(
            resolve_pending_world_path(&mut pending, Some(&active)),
            PathBuf::from("Worlds/Pending/world.aeworld")
        );
        assert!(pending.is_none());

        assert_eq!(
            resolve_pending_world_path(&mut pending, Some(&active)),
            active
        );

        assert_eq!(
            resolve_pending_world_path(&mut pending, None),
            PathBuf::from(ae_world::ProjectWorkspaceLayout::DEFAULT_ENTRY_WORLD_REL)
        );
    }

    #[test]
    fn test_poll_scene_dialogs_preserves_active_scene_path_until_io_completes() {
        let mut editor = EditorState {
            active_scene_path: Some(PathBuf::from("Worlds/Working/world.aeworld")),
            ..EditorState::default()
        };
        let (tx, rx) = std::sync::mpsc::channel();
        editor.scene_dialog_receivers.push(rx);
        tx.send(SceneDialogAction::LoadFrom(PathBuf::from(
            "Worlds/UnloadedOrInvalid/world.aeworld",
        )))
        .expect("channel send must succeed");

        editor.poll_scene_dialogs();

        assert!(editor.should_load_scene);
        assert_eq!(
            editor.pending_load_path,
            Some(PathBuf::from("Worlds/UnloadedOrInvalid/world.aeworld"))
        );
        // Active scene path must remain unchanged until async load actually succeeds.
        assert_eq!(
            editor.active_scene_path,
            Some(PathBuf::from("Worlds/Working/world.aeworld"))
        );
    }
}