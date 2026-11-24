// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Host scene/prefab OS dialog orchestration and partitioned world persistence polling.

use crate::engine::AeEngine;

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

fn poll_pending_scene_save(engine: &mut AeEngine) {
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
        &mut engine.world,
        &engine.asset_manager,
        &mut engine.active_world_guid,
        ae_scene::WorldSaveRequest {
            filepath: &path,
            is_2d: engine.dimension_mode.is_2d(),
            prev_saved_world_dir: engine.saved_world_dir.as_deref(),
            prev_cell_crcs: &engine.last_saved_cell_crcs,
        },
    ) {
        Ok(saved) => {
            engine.saved_world_dir = Some(saved.saved_world_dir);
            engine.last_saved_cell_crcs = saved.cell_crcs;
            engine.editor.active_scene_path = Some(saved.manifest_path);
            engine.autosave.reset_timer();
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

fn poll_pending_scene_load(engine: &mut AeEngine) {
    if !engine.editor.should_load_scene {
        return;
    }
    engine.editor.should_load_scene = false;
    let path = resolve_pending_world_path(
        &mut engine.editor.pending_load_path,
        engine.editor.active_scene_path.as_ref(),
    );
    match ae_scene::begin_async_scene_load(&path, engine.dimension_mode.is_2d()) {
        Ok(rx) => {
            engine.editor.is_loading_assets = true;
            engine.scene_rx = Some(rx);
            engine.ui.set_status_message(
                "Loading world: parsing assets in background...",
                irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
            );
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                log::error!("{e}");
                let ui_msg = if engine.dimension_mode.is_2d() {
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

fn poll_async_scene_load_result(engine: &mut AeEngine) {
    let Some(rx) = engine.scene_rx.as_ref() else {
        return;
    };
    let Ok(result) = rx.try_recv() else {
        return;
    };
    engine.scene_rx = None;
    match result {
        Ok(scene_data) => {
            engine.physics_world.clear();
            engine.editor.selected_entities.clear();
            engine.editor.undo_stack.clear();
            engine.editor.redo_stack.clear();
            engine.autosave.reset_timer();

            let applied = ae_scene::apply_loaded_scene_data(
                scene_data,
                &mut engine.world,
                &engine.render_state,
                &mut engine.asset_manager,
                engine.active_vfs.as_deref(),
            );
            engine.saved_world_dir = applied.saved_world_dir;
            engine.active_world_guid = applied.world_guid;
            engine.last_saved_cell_crcs = applied.cell_crcs;
            engine.streaming = applied.streaming;

            engine
                .physics_world
                .sync_ecs_to_physics(&mut engine.world, |handle| {
                    engine
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
pub fn process_pending_scene_io(engine: &mut AeEngine) {
    poll_pending_scene_save(engine);
    poll_pending_scene_load(engine);
    poll_async_scene_load_result(engine);
}

/// Opens an OS file dialog to instantiate an `.aeprefab` archive.
pub fn handle_open_load_prefab_dialog(engine: &mut AeEngine) {
    #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        engine.import_receivers.push(rx);
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
pub fn handle_open_save_scene_dialog(engine: &mut AeEngine) {
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
pub fn handle_open_load_scene_dialog(engine: &mut AeEngine) {
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
pub fn handle_save_scene_to_path(engine: &mut AeEngine, path: std::path::PathBuf) {
    engine.editor.pending_save_path = Some(path.clone());
    engine.editor.should_save_scene = true;
    engine.editor.active_scene_path = Some(path.clone());
    log::info!("💾 Scene save request queued for path {:?}", path);
}

/// Queues an asynchronous scene load request targeting `path`.
pub fn handle_load_scene_from_path(engine: &mut AeEngine, path: std::path::PathBuf) {
    engine.editor.pending_load_path = Some(path.clone());
    engine.editor.should_load_scene = true;
    engine.editor.is_loading_assets = true;
    engine.editor.active_scene_path = Some(path.clone());
    log::info!("📂 Scene load request queued for path {:?}", path);
}

/// Captures `entity` into an `.aeprefab` archive via [`ae_scene::Prefab`] and writes it to `path`.
pub fn handle_save_entity_as_prefab(
    engine: &mut AeEngine,
    entity: ae_ecs::Entity,
    path: std::path::PathBuf,
) {
    let prefab = ae_scene::Prefab::create_from_entity(&engine.world, entity);
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
pub fn handle_instantiate_prefab(engine: &mut AeEngine, path: std::path::PathBuf) {
    match ae_scene::Prefab::load_from_file(&path) {
        Ok(prefab) => {
            let new_entity = prefab.instantiate(&mut engine.world, None);
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