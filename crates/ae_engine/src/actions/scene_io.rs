// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Host scene/prefab dialog orchestration and viewport drag-and-drop asset ingestion.

use crate::engine::AeEngine;
use ae_editor_ui::ui::PanelId;
use ae_editor_ui::ui::types::DroppedFileTarget;

/// Opens an OS model/texture import dialog when `editor-dialogs` is enabled.
pub fn handle_open_model_dialog(engine: &mut AeEngine) {
    #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        engine.dialog_receivers.push(rx);
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(
                    "3D Models & Textures",
                    &["gltf", "glb", "obj", "fbx", "png", "jpg", "jpeg"],
                )
                .pick_file()
            {
                let _ = tx.send(path);
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

/// Ingests dropped external files into the asset library and spawns dropped 3D/sprite assets
/// directly when dropped onto the 3D viewport.
pub fn handle_import_dropped_files(
    engine: &mut AeEngine,
    paths: Vec<std::path::PathBuf>,
    target: DroppedFileTarget,
) {
    let viewport_rect = engine.ui.last_viewport_rect;
    let world_spawn_pos = resolve_viewport_drop_world_pos(target, viewport_rect, &engine.camera);
    let is_over_viewport = world_spawn_pos.is_some();

    let current_folder = engine.ui.asset_browser.current_folder.clone();
    let target_dir = if is_over_viewport && current_folder == std::path::Path::new("assets") {
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
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            match ext.as_str() {
                "gltf" | "glb" => {
                    match super::spawning::handle_spawn_model_path_at(
                        engine,
                        path.clone(),
                        world_pos,
                    ) {
                        Ok(()) => engine.ui.set_status_message(
                            format!(
                                "Spawned 3D model at ({:.1}, {:.1}, {:.1})",
                                world_pos[0], world_pos[1], world_pos[2]
                            ),
                            irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
                        ),
                        Err(err) => engine.ui.set_status_message(
                            format!("Failed to load 3D model: {err}"),
                            irisui::prelude::Color::RED,
                        ),
                    }
                }
                "png" | "jpg" | "jpeg" => {
                    super::spawning::handle_spawn_sprite_path_at(engine, path.clone(), world_pos);
                    engine.ui.set_status_message(
                        format!(
                            "Spawned sprite at ({:.1}, {:.1}, {:.1})",
                            world_pos[0], world_pos[1], world_pos[2]
                        ),
                        irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
                    );
                }
                _ => {
                    let (tx, rx) = std::sync::mpsc::channel();
                    let _ = tx.send(path);
                    engine.dialog_receivers.push(rx);
                }
            }
        }
    } else {
        engine.ui.set_status_message(
            format!(
                "Imported {} file(s) into project assets",
                imported_paths.len()
            ),
            irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
        );
    }
    let _ = PanelId::Assets;
}

/// Opens an OS file dialog to instantiate an `.aeprefab` archive.
pub fn handle_open_load_prefab_dialog(engine: &mut AeEngine) {
    #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        engine.dialog_receivers.push(rx);
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Aeon Prefab (*.aeprefab)", &["aeprefab"])
                .pick_file()
            {
                let _ = tx.send(path);
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
        }
        Err(e) => {
            log::error!("Failed to instantiate prefab from {:?}: {}", path, e);
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
    fn test_resolve_viewport_drop_world_pos_rejects_library_and_malformed_viewports() {
        let camera = test_camera();
        let viewport = Rect::new(300.0, 60.0, 900.0, 600.0);

        assert_eq!(
            resolve_viewport_drop_world_pos(DroppedFileTarget::AssetLibrary, viewport, &camera),
            None
        );
        assert_eq!(
            resolve_viewport_drop_world_pos(
                DroppedFileTarget::Workspace(Some([50.0, 50.0])),
                viewport,
                &camera,
            ),
            None
        );
        assert_eq!(
            resolve_viewport_drop_world_pos(
                DroppedFileTarget::Workspace(None),
                Rect::ZERO,
                &camera
            ),
            None
        );
        assert_eq!(
            resolve_viewport_drop_world_pos(
                DroppedFileTarget::Workspace(Some([f32::NAN, 360.0])),
                viewport,
                &camera,
            ),
            None
        );
    }
}