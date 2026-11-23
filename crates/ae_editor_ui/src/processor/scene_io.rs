// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::UiContext;

/// Handles triggering model import file dialog.
pub fn handle_open_model_dialog(ctx: &mut UiContext) {
    let (tx, rx) = std::sync::mpsc::channel();
    ctx.dialog_receivers.push(rx);
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

/// Handles ingesting dropped external files into the asset pipeline.
///
/// Follows industry-standard DCC / game engine conventions:
/// - If dropped onto the 3D Viewport: copies the asset (and glTF external dependencies)
///   to the project directory, computes ground plane raycast intersection from the cursor,
///   and spawns the entity directly at the pointed world position.
/// - If dropped onto the Asset Browser: imports the asset into the active folder
///   and refreshes the content browser without dirtying the 3D scene.
pub fn handle_import_dropped_files(
    ctx: &mut UiContext,
    paths: Vec<std::path::PathBuf>,
    screen_pos: Option<[f32; 2]>,
) {
    let viewport_rect = ctx.ui.last_viewport_rect;
    let is_over_viewport = screen_pos.is_some_and(|pos| {
        viewport_rect.contains_point(irisui::prelude::Point::new(pos[0], pos[1]))
    });

    let current_folder = ctx.ui.asset_browser.current_folder.clone();
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
        let before_path = path.clone();
        let import_path = ae_assets::import_asset_path(&path, &target_dir);
        if import_path != before_path {
            imported_count += 1;
        }
        imported_paths.push(import_path);
    }

    if imported_count > 0 {
        ctx.ui.asset_browser.cached_items.clear();
        ctx.ui
            .host
            .tag_redraw(crate::ui::panel_layout::PanelId::Assets);
    }

    if is_over_viewport {
        // Industry Standard Viewport Drop: raycast to ground plane at mouse cursor
        let world_pos = if let Some(pos) = screen_pos {
            ae_editor::picking::compute_ground_intersection(
                pos,
                [
                    viewport_rect.x,
                    viewport_rect.y,
                    viewport_rect.width,
                    viewport_rect.height,
                ],
                &ctx.camera.build_view_projection_matrix(),
            )
            .unwrap_or([0.0, 0.0, 0.0])
        } else {
            [0.0, 0.0, 0.0]
        };

        for path in imported_paths {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            match ext.as_str() {
                "gltf" | "glb" => {
                    super::spawning::handle_spawn_model_path_at(ctx, path.clone(), world_pos);
                    ctx.ui.set_status_message(
                        format!(
                            "Spawned 3D model at ({:.1}, {:.1}, {:.1})",
                            world_pos[0], world_pos[1], world_pos[2]
                        ),
                        irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
                    );
                }
                "png" | "jpg" | "jpeg" => {
                    super::spawning::handle_spawn_sprite_path_at(ctx, path.clone(), world_pos);
                    ctx.ui.set_status_message(
                        format!(
                            "Spawned sprite at ({:.1}, {:.1}, {:.1})",
                            world_pos[0], world_pos[1], world_pos[2]
                        ),
                        irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
                    );
                }
                _ => {
                    // Non-immediate formats (FBX, scenes, etc.) route to engine async importer
                    let (tx, rx) = std::sync::mpsc::channel();
                    let _ = tx.send(path);
                    ctx.dialog_receivers.push(rx);
                }
            }
        }
    } else {
        // Dropped onto Asset Browser or other panels: import to library only
        ctx.ui.set_status_message(
            format!(
                "Imported {} file(s) into project assets",
                imported_paths.len()
            ),
            irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
        );
    }
}

/// Handles triggering prefab load file dialog.
pub fn handle_open_load_prefab_dialog(ctx: &mut UiContext) {
    let (tx, rx) = std::sync::mpsc::channel();
    ctx.dialog_receivers.push(rx);
    std::thread::spawn(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Aeon Prefab (*.aeprefab)", &["aeprefab"])
            .pick_file()
        {
            let _ = tx.send(path);
        }
    });
}

/// Handles triggering save scene file dialog.
pub fn handle_open_save_scene_dialog(ctx: &mut UiContext) {
    let (tx, rx) = std::sync::mpsc::channel();
    ctx.editor.scene_dialog_receivers.push(rx);
    let _is_2d = ctx.editor.is_2d;
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

/// Handles triggering load scene file dialog.
pub fn handle_open_load_scene_dialog(ctx: &mut UiContext) {
    let (tx, rx) = std::sync::mpsc::channel();
    ctx.editor.scene_dialog_receivers.push(rx);
    std::thread::spawn(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Aeon Binary World (*.aeworld)", &["aeworld"])
            .pick_file()
        {
            let _ = tx.send(ae_editor::editor_state::SceneDialogAction::LoadFrom(path));
        }
    });
}

/// Handles saving scene via file dialog.
pub fn handle_save_scene(ctx: &mut UiContext) {
    handle_open_save_scene_dialog(ctx);
}

/// Handles loading scene via file dialog.
pub fn handle_load_scene(ctx: &mut UiContext) {
    handle_open_load_scene_dialog(ctx);
}

/// Handles saving scene directly to specified path.
pub fn handle_save_scene_to_path(ctx: &mut UiContext, path: std::path::PathBuf) {
    ctx.editor.pending_save_path = Some(path.clone());
    ctx.editor.should_save_scene = true;
    ctx.editor.active_scene_path = Some(path.clone());
    log::info!("💾 Scene save request queued for path {:?}", path);
}

/// Handles loading scene directly from specified path.
pub fn handle_load_scene_from_path(ctx: &mut UiContext, path: std::path::PathBuf) {
    ctx.editor.pending_load_path = Some(path.clone());
    ctx.editor.should_load_scene = true;
    ctx.editor.is_loading_assets = true;
    ctx.editor.active_scene_path = Some(path.clone());
    log::info!("📂 Scene load request queued for path {:?}", path);
}

/// Handles saving an entity as a prefab asset to specified path.
pub fn handle_save_entity_as_prefab(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    path: std::path::PathBuf,
) {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        log::error!("Failed to create prefab directory {:?}: {}", parent, e);
        ctx.ui.set_status_message(
            format!("Failed to create directory: {}", e),
            irisui::prelude::Color::rgba(0.95, 0.35, 0.35, 1.0),
        );
        return;
    }
    let prefab = ae_editor::prefab::Prefab::create_from_entity(ctx.world, entity);
    if let Err(e) = prefab.save_to_file(&path) {
        log::error!("Failed to save prefab to {:?}: {}", path, e);
        ctx.ui.set_status_message(
            format!("Failed to save prefab: {}", e),
            irisui::prelude::Color::rgba(0.95, 0.35, 0.35, 1.0),
        );
    } else {
        log::info!("📦 Prefab successfully saved to {:?}", path);
        ctx.ui.set_status_message(
            format!("Saved prefab to {}", path.display()),
            irisui::prelude::Color::rgba(0.25, 0.85, 0.45, 1.0),
        );
    }
}

/// Handles instantiating a prefab asset from specified path into the ECS world.
pub fn handle_instantiate_prefab(ctx: &mut UiContext, path: std::path::PathBuf) {
    match ae_editor::prefab::Prefab::load_from_file(&path) {
        Ok(prefab) => {
            let new_entity = prefab.instantiate(ctx.world, None);
            ctx.editor.select_single(Some(new_entity));
            log::info!("✨ Instantiated prefab from {:?}", path);
        }
        Err(e) => {
            log::error!("Failed to instantiate prefab from {:?}: {}", path, e);
        }
    }
}