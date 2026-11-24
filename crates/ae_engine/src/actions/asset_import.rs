// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unified host asset ingestion, OS dialog dispatch, drag-and-drop viewport spawning,
//! and typed asynchronous import polling.

use super::{scene_io, spawning};
use crate::engine::AeEngine;
use ae_assets::{AsyncImportResult, ImportAssetKind};
use ae_editor_ui::ui::types::DroppedFileTarget;
use irisui::prelude::Color;
use std::path::{Path, PathBuf};

/// Opens an OS model/texture import dialog when `editor-dialogs` is enabled.
pub fn handle_open_model_dialog(engine: &mut AeEngine) {
    #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        engine.import_receivers.push(rx);
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

/// Resolves whether an incoming file drop targets the 3D viewport and computes the world spawn position.
#[must_use]
pub fn resolve_viewport_drop_world_pos(
    target: DroppedFileTarget,
    viewport_rect: irisui::prelude::Rect,
    camera: &ae_camera::Camera,
) -> Option<[f32; 3]> {
    if !viewport_rect.width.is_finite()
        || !viewport_rect.height.is_finite()
        || !viewport_rect.x.is_finite()
        || !viewport_rect.y.is_finite()
        || viewport_rect.width <= 20.0
        || viewport_rect.height <= 20.0
    {
        return None;
    }

    let screen_pos = match target {
        DroppedFileTarget::AssetLibrary => return None,
        DroppedFileTarget::Workspace(Some(pos)) => {
            if !pos[0].is_finite()
                || !pos[1].is_finite()
                || !viewport_rect.contains_point(irisui::prelude::Point::new(pos[0], pos[1]))
            {
                return None;
            }
            pos
        }
        DroppedFileTarget::Workspace(None) => [
            viewport_rect.x + viewport_rect.width * 0.5,
            viewport_rect.y + viewport_rect.height * 0.5,
        ],
    };

    Some(
        ae_editor::picking::compute_ground_intersection(
            screen_pos,
            [
                viewport_rect.x,
                viewport_rect.y,
                viewport_rect.width,
                viewport_rect.height,
            ],
            &camera.build_view_projection_matrix(),
        )
        .unwrap_or_else(|| {
            ae_editor::picking::compute_camera_ground_fallback(
                [camera.position.x, camera.position.y, camera.position.z],
                [camera.target.x, camera.target.y, camera.target.z],
                3.0,
            )
        }),
    )
}

/// Ingests dropped external files into the asset library and routes viewport drops through
/// [`import_and_spawn_path`].
pub fn handle_import_dropped_files(
    engine: &mut AeEngine,
    paths: Vec<PathBuf>,
    target: DroppedFileTarget,
) {
    let viewport_rect = engine.ui.last_viewport_rect;
    let world_spawn_pos = resolve_viewport_drop_world_pos(target, viewport_rect, &engine.camera);
    let is_over_viewport = world_spawn_pos.is_some();

    let current_folder = engine.ui.asset_browser.current_folder.clone();
    let target_dir = if is_over_viewport && current_folder == Path::new("assets") {
        current_folder.join("models")
    } else {
        current_folder
    };

    if !target_dir.exists() {
        let _ = std::fs::create_dir_all(&target_dir);
    }

    let mut imported_paths = Vec::with_capacity(paths.len());
    let mut imported_count = 0;

    for path in paths {
        let import_path = ae_assets::import_asset_path(&path, &target_dir);
        if import_path != path {
            imported_count += 1;
        }
        imported_paths.push(import_path);
    }

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

/// Validates dimension safety and imports or spawns `path` via the single typed [`ImportAssetKind`] pipeline.
pub fn import_and_spawn_path(engine: &mut AeEngine, path: PathBuf, spawn_pos: [f32; 3]) {
    if let Err(reason) =
        ae_assets::validate_asset_drop_for_dimension(&path, engine.dimension_mode.is_2d())
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
            spawning::handle_spawn_sprite_path_at(engine, path, spawn_pos);
            engine.ui.set_status_message(
                format!(
                    "Spawned sprite at ({:.1}, {:.1}, {:.1})",
                    spawn_pos[0], spawn_pos[1], spawn_pos[2]
                ),
                Color::rgb(0.0, 0.898, 1.0),
            );
        }
        ImportAssetKind::GltfModel => import_gltf_model(engine, &path, &raw_label, spawn_pos),
        ImportAssetKind::FbxModel => {
            let final_name = ae_scene::resolve_unique_entity_name(&engine.world, &raw_label);
            engine.editor.is_loading_assets = true;
            engine.ui.set_status_message(
                "Converting FBX file... Please wait.",
                Color::rgb(0.0, 0.898, 1.0),
            );
            engine
                .import_receivers
                .push(ae_assets::spawn_fbx_import(&path, final_name, spawn_pos));
        }
        ImportAssetKind::WgslShader => load_wgsl_shader_asset(engine, &path, &raw_label),
        ImportAssetKind::Prefab => scene_io::handle_instantiate_prefab(engine, path),
        ImportAssetKind::WorldScene => scene_io::handle_load_scene_from_path(engine, path),
    }
}

fn import_gltf_model(engine: &mut AeEngine, path: &Path, raw_label: &str, spawn_pos: [f32; 3]) {
    if let Ok(canonical) = std::fs::canonicalize(path)
        && let Some(&handle) = engine.asset_manager.model_path_map.get(&canonical)
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

    let final_name = ae_scene::resolve_unique_entity_name(&engine.world, raw_label);
    engine.editor.is_loading_assets = true;
    engine.ui.set_status_message(
        format!("Loading {final_name} in background..."),
        Color::rgb(0.0, 0.898, 1.0),
    );
    engine
        .import_receivers
        .push(ae_assets::spawn_gltf_import(path, final_name, spawn_pos));
}

fn load_wgsl_shader_asset(engine: &mut AeEngine, path: &Path, display_name: &str) {
    let Some(path_str) = path.to_str() else {
        return;
    };
    engine.editor.is_loading_assets = true;
    match engine
        .render_state
        .load_shader(&mut engine.asset_manager, path_str)
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

/// Drains completed dialog selections and `rayon` background import jobs into the engine.
pub fn poll_async_imports(engine: &mut AeEngine) {
    if engine.import_receivers.is_empty() {
        return;
    }
    let mut completed = Vec::new();
    engine.import_receivers.retain(|rx| {
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
                spawning::spawn_parsed_model_at(engine, *data, spawn_pos, apply_fbx_axis_fix);
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

#[cfg(test)]
mod tests {
    use super::resolve_viewport_drop_world_pos;
    use ae_camera::{Camera, ProjectionMode};
    use ae_editor_ui::ui::types::DroppedFileTarget;
    use cgmath::{Deg, Point3, Rad};
    use irisui::prelude::Rect;

    fn test_camera() -> Camera {
        Camera {
            position: Point3::new(0.0, 5.0, 5.0),
            yaw: Rad::from(Deg(-90.0)),
            pitch: Rad::from(Deg(-45.0)),
            aspect: 900.0 / 600.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 10.0,
            target: Point3::new(0.0, 0.0, 0.0),
        }
    }

    #[test]
    fn test_resolve_viewport_drop_world_pos_accepts_positioned_and_unpositioned_workspace_drops() {
        let camera = test_camera();
        let viewport = Rect::new(300.0, 60.0, 900.0, 600.0);

        let center_pos = resolve_viewport_drop_world_pos(
            DroppedFileTarget::Workspace(Some([750.0, 360.0])),
            viewport,
            &camera,
        )
        .expect("Workspace drop inside viewport must resolve a spawn position");
        assert!(center_pos[0].abs() < 1e-3);
        assert!(center_pos[1].abs() < 1e-3);
        assert!(center_pos[2].abs() < 1e-3);

        let unpositioned_pos =
            resolve_viewport_drop_world_pos(DroppedFileTarget::Workspace(None), viewport, &camera)
                .expect("Unpositioned workspace drop must fall back to viewport center");
        assert!((unpositioned_pos[0] - center_pos[0]).abs() < 1e-3);
        assert!((unpositioned_pos[1] - center_pos[1]).abs() < 1e-3);
        assert!((unpositioned_pos[2] - center_pos[2]).abs() < 1e-3);
    }

    #[test]
    fn test_resolve_viewport_drop_world_pos_rejects_asset_library_and_outside_points() {
        let camera = test_camera();
        let viewport = Rect::new(300.0, 60.0, 900.0, 600.0);

        assert!(
            resolve_viewport_drop_world_pos(DroppedFileTarget::AssetLibrary, viewport, &camera)
                .is_none()
        );
        assert!(
            resolve_viewport_drop_world_pos(
                DroppedFileTarget::Workspace(Some([10.0, 10.0])),
                viewport,
                &camera,
            )
            .is_none()
        );
        assert!(
            resolve_viewport_drop_world_pos(
                DroppedFileTarget::Workspace(None),
                Rect::ZERO,
                &camera
            )
            .is_none()
        );
        assert!(
            resolve_viewport_drop_world_pos(
                DroppedFileTarget::Workspace(Some([f32::NAN, 300.0])),
                viewport,
                &camera,
            )
            .is_none()
        );
    }
}