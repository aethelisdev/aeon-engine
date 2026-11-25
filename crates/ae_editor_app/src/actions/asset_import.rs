// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Host asset import OS dialog dispatch, dropped-file routing, and async import polling.

use super::{scene_io, spawning};
use crate::session::EditorSession;
use ae_assets::{AsyncImportResult, ImportAssetKind};
use ae_editor_ui::ui::types::DroppedFileTarget;
use irisui::prelude::Color;
use std::path::{Path, PathBuf};

/// Opens an OS model/texture import dialog when `editor-dialogs` is enabled.
pub fn handle_open_model_dialog(engine: &mut EditorSession) {
    #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        engine.host_io.import_receivers.push(rx);
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(
                    "3D Models & Textures",
                    &["gltf", "glb", "obj", "fbx", "png", "jpg", "jpeg"],
                )
                .pick_file()
            {
                let _ = tx.send(AsyncImportResult::PickedPath {
                    path,
                    spawn_pos: [0.0, 0.0, 0.0],
                });
            }
        });
    }
    #[cfg(not(all(feature = "editor-dialogs", not(target_os = "android"))))]
    let _ = engine;
}

/// Ingests dropped external files into the asset library and routes viewport drops through
/// [`import_and_spawn_path`].
pub fn handle_import_dropped_files(
    engine: &mut EditorSession,
    paths: Vec<PathBuf>,
    target: DroppedFileTarget,
) {
    let vp = engine.render.render_state.last_viewport_rect;
    let world_spawn_pos = match target {
        DroppedFileTarget::AssetLibrary => None,
        DroppedFileTarget::Workspace(pos) => ae_editor::picking::resolve_ground_drop_world_pos(
            pos,
            [vp.min_x, vp.min_y, vp.max_x - vp.min_x, vp.max_y - vp.min_y],
            &engine.render.camera,
        ),
    };

    let current_folder = &engine.ui.asset_browser.current_folder;
    let target_dir = if world_spawn_pos.is_some() && current_folder == Path::new("assets") {
        current_folder.join("models")
    } else {
        current_folder.clone()
    };

    let (imported_paths, imported_count) = ae_assets::import_dropped_paths(&paths, &target_dir);
    if imported_count > 0 {
        engine.ui.refresh_asset_browser(None);
    }

    if let Some(world_pos) = world_spawn_pos {
        for path in imported_paths {
            import_and_spawn_path(engine, path, world_pos);
        }
    } else {
        engine.ui.set_status_message(
            format!(
                "Imported {} file(s) into project assets",
                imported_paths.len()
            ),
            Color::rgb(0.0, 0.898, 1.0),
        );
    }
}

/// Validates dimension safety and imports or spawns `path` via [`ImportAssetKind`].
pub fn import_and_spawn_path(engine: &mut EditorSession, path: PathBuf, spawn_pos: [f32; 3]) {
    if let Err(reason) =
        ae_assets::validate_asset_drop_for_dimension(&path, engine.render.dimension_mode.is_2d())
    {
        log::warn!("Rejected asset import {:?}: {}", path, reason);
        engine.ui.set_status_message(reason, Color::RED);
        return;
    }

    let Some(kind) = ImportAssetKind::from_path(&path) else {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("<none>");
        log::warn!("Unsupported asset format: {:?}", ext);
        engine.ui.set_status_message(
            format!("Unsupported file extension: .{ext}"),
            Color::rgb(1.0, 0.8, 0.1),
        );
        return;
    };

    let raw_label = ae_scene::resolve_asset_display_name(&path, "Asset");
    match kind {
        ImportAssetKind::Texture => {
            spawning::handle_spawn_sprite_path_at(engine, &path, spawn_pos);
            engine.ui.set_status_message(
                format!(
                    "Spawned sprite at ({:.1}, {:.1}, {:.1})",
                    spawn_pos[0], spawn_pos[1], spawn_pos[2]
                ),
                Color::rgb(0.0, 0.898, 1.0),
            );
        }
        ImportAssetKind::GltfModel => queue_gltf_import(engine, &path, &raw_label, spawn_pos),
        ImportAssetKind::FbxModel => {
            let final_name = ae_scene::resolve_unique_entity_name(&engine.core.world, &raw_label);
            engine.editor.is_loading_assets = true;
            engine.ui.set_status_message(
                "Converting FBX file... Please wait.",
                Color::rgb(0.0, 0.898, 1.0),
            );
            engine
                .host_io
                .import_receivers
                .push(ae_assets::spawn_fbx_import(&path, final_name, spawn_pos));
        }
        ImportAssetKind::WgslShader => load_wgsl_shader_asset(engine, &path, &raw_label),
        ImportAssetKind::Prefab => scene_io::handle_instantiate_prefab(engine, path),
        ImportAssetKind::WorldScene => scene_io::handle_load_scene_from_path(engine, path),
    }
}

fn queue_gltf_import(
    engine: &mut EditorSession,
    path: &Path,
    raw_label: &str,
    spawn_pos: [f32; 3],
) {
    if let Ok(canonical) = std::fs::canonicalize(path)
        && let Some(&handle) = engine.render.asset_manager.model_path_map.get(&canonical)
    {
        spawning::handle_spawn_model_at(engine, handle, spawn_pos);
        engine.ui.set_status_message(
            format!(
                "Spawned 3D model at ({:.1}, {:.1}, {:.1})",
                spawn_pos[0], spawn_pos[1], spawn_pos[2]
            ),
            Color::rgb(0.0, 0.898, 1.0),
        );
        return;
    }

    let final_name = ae_scene::resolve_unique_entity_name(&engine.core.world, raw_label);
    engine.editor.is_loading_assets = true;
    engine.ui.set_status_message(
        format!("Loading {final_name} in background..."),
        Color::rgb(0.0, 0.898, 1.0),
    );
    engine
        .host_io
        .import_receivers
        .push(ae_assets::spawn_gltf_import(path, final_name, spawn_pos));
}

fn load_wgsl_shader_asset(engine: &mut EditorSession, path: &Path, display_name: &str) {
    let Some(path_str) = path.to_str() else {
        return;
    };
    engine.editor.is_loading_assets = true;
    match engine
        .render
        .render_state
        .load_shader(&mut engine.render.asset_manager, path_str)
    {
        Ok(_handle) => {
            log::info!("⚡ Loaded shader asset: {:?}", display_name);
            engine.ui.set_status_message(
                format!("Shader '{display_name}' successfully loaded."),
                Color::rgb(0.1, 0.9, 0.3),
            );
        }
        Err(err) => {
            log::error!("❌ Failed to load shader '{}': {}", display_name, err);
            engine.ui.set_status_message(
                format!("Failed to load shader '{display_name}': {err}"),
                Color::RED,
            );
        }
    }
    engine.editor.is_loading_assets = false;
}

/// Drains completed dialog selections and `rayon` background import jobs into the session.
pub fn poll_async_imports(engine: &mut EditorSession) {
    if engine.host_io.import_receivers.is_empty() {
        return;
    }
    let mut completed = Vec::new();
    engine.host_io.import_receivers.retain(|rx| {
        loop {
            match rx.try_recv() {
                Ok(msg) => completed.push(msg),
                Err(std::sync::mpsc::TryRecvError::Empty) => return true,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => return false,
            }
        }
    });

    for msg in completed {
        match msg {
            AsyncImportResult::PickedPath { path, spawn_pos } => {
                import_and_spawn_path(engine, path, spawn_pos);
            }
            AsyncImportResult::ModelParsed {
                data,
                spawn_pos,
                apply_fbx_axis_fix,
            } => {
                let model_name = data.final_name.clone();
                let entity = ae_scene::spawn_parsed_model_at(
                    &mut engine.core.world,
                    &engine.render.render_state,
                    &mut engine.render.asset_manager,
                    *data,
                    spawn_pos,
                    apply_fbx_axis_fix,
                );
                ae_editor::actions::record_spawn(&engine.core.world, &mut engine.editor, entity);
                engine.editor.is_loading_assets = false;
                engine.ui.set_status_message(
                    format!("{model_name} loaded successfully!"),
                    Color::rgb(0.0, 0.898, 1.0),
                );
                log::info!("Async 3D model loaded and spawned entity: {:?}", model_name);
            }
            AsyncImportResult::Error(err) => {
                log::error!("Async asset import failed: {err}");
                engine.editor.is_loading_assets = false;
                engine
                    .ui
                    .set_status_message(format!("ERROR: {err}"), Color::RED);
            }
        }
    }
}