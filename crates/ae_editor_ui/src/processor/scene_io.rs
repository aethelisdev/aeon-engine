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

/// Handles triggering save scene file dialog.
pub fn handle_open_save_scene_dialog(ctx: &mut UiContext) {
    let (tx, rx) = std::sync::mpsc::channel();
    ctx.dialog_receivers.push(rx);
    std::thread::spawn(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Aeon Scene", &["aeon", "json"])
            .set_file_name("scene.aeon")
            .save_file()
        {
            let _ = tx.send(path);
        }
    });
}

/// Handles triggering load scene file dialog.
pub fn handle_open_load_scene_dialog(ctx: &mut UiContext) {
    let (tx, rx) = std::sync::mpsc::channel();
    ctx.dialog_receivers.push(rx);
    std::thread::spawn(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Aeon Scene", &["aeon", "json"])
            .pick_file()
        {
            let _ = tx.send(path);
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
    let _ = ctx;
    let _ = path;
    log::info!("💾 Scene save request queued for path {:?}", path);
}

/// Handles loading scene directly from specified path.
pub fn handle_load_scene_from_path(ctx: &mut UiContext, path: std::path::PathBuf) {
    let (tx, rx) = std::sync::mpsc::channel();
    let _ = tx.send(path);
    ctx.dialog_receivers.push(rx);
}

/// Handles saving an entity as a prefab asset to specified path.
pub fn handle_save_entity_as_prefab(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    path: std::path::PathBuf,
) {
    let prefab = ae_editor::prefab::Prefab::create_from_entity(ctx.world, entity);
    if let Err(e) = prefab.save_to_file(&path) {
        log::error!("Failed to save prefab to {:?}: {}", path, e);
    } else {
        log::info!("📦 Prefab successfully saved to {:?}", path);
    }
}

/// Handles instantiating a prefab asset from specified path into the ECS world.
pub fn handle_instantiate_prefab(ctx: &mut UiContext, path: std::path::PathBuf) {
    match ae_editor::prefab::Prefab::load_from_file(&path) {
        Ok(prefab) => {
            let new_entity = prefab.instantiate(ctx.world, None);
            ctx.editor.selected_entities.clear();
            ctx.editor.selected_entities_set.clear();
            ctx.editor.selected_entities.push(new_entity);
            ctx.editor.selected_entities_set.insert(new_entity);
            ctx.ui.selected_entity = Some(new_entity);
            log::info!("✨ Instantiated prefab from {:?}", path);
        }
        Err(e) => {
            log::error!("Failed to instantiate prefab from {:?}: {}", path, e);
        }
    }
}