// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::engine::AeEngine;
use std::path::PathBuf;

// is_safe_path and ParsedModelData are now imported from ae_renderer::asset

/// Process asynchronous asset imports, such as background FBX conversions and model parsing tasks.
/// Iterates ALL active receivers — multiple async operations can run concurrently.
/// Also drains `dialog_receivers` to process asynchronously selected files from the native dialog
/// without blocking the winit main thread / event loop.
pub fn process_async_imports(engine: &mut AeEngine) {
    // --- ASYNC DIALOG RECEIVERS ---
    // Drain asynchronously picked paths from native file dialogs and feed them to importer
    let mut dialog_paths = Vec::new();
    for rx in &engine.dialog_receivers {
        while let Ok(path) = rx.try_recv() {
            dialog_paths.push(path);
        }
    }
    // Clean up disconnected dialog receivers
    engine.dialog_receivers.retain(|rx| {
        !matches!(
            rx.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Disconnected)
        )
    });
    // Process each path identically to a drag-and-drop file import
    for path in dialog_paths {
        handle_dropped_file(engine, path);
    }

    let mut messages = Vec::new();

    // Drain all pending messages from ALL receivers
    for rx in &engine.asset_receivers {
        while let Ok(result) = rx.try_recv() {
            messages.push(result);
        }
    }

    // Remove disconnected receivers (sender dropped = thread finished)
    engine.asset_receivers.retain(|rx| {
        // A receiver is still alive if try_recv returns Ok or TryRecvError::Empty.
        // TryRecvError::Disconnected means the sender is gone — safe to drop.
        !matches!(
            rx.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Disconnected)
        )
    });

    for result in messages {
        match result {
            Ok(glb_path) => {
                let path_str = glb_path.to_string_lossy();
                if path_str == "DONE_DOWNLOAD" {
                    engine.ui.status_message = Some((
                        vec![(
                            "Tool installed successfully! You can now drag FBX files.".to_string(),
                            egui::Color32::LIGHT_BLUE,
                        )],
                        std::time::Instant::now(),
                    ));
                } else if path_str == "PYTHON_DONE" {
                    engine.ui.status_message = Some((vec![("Python installed successfully! You may need to restart the engine for changes to take effect.".to_string(), egui::Color32::LIGHT_BLUE)], std::time::Instant::now()));
                } else if glb_path.exists() {
                    if let Some(path_str) = glb_path.to_str() {
                        let (model_id, min, max) = engine
                            .render_state
                            .load_model(&mut engine.asset_manager, path_str);
                        let base_name = glb_path
                            .file_name()
                            .unwrap_or(std::ffi::OsStr::new("Model"))
                            .to_string_lossy()
                            .into_owned();

                        // Calculate Auto Scaling & Spawn
                        spawn_model(engine, base_name.clone(), model_id, min, max, path_str);
                        engine.ui.status_message = Some((
                            vec![(
                                "Asset loaded successfully!".to_string(),
                                egui::Color32::LIGHT_BLUE,
                            )],
                            std::time::Instant::now(),
                        ));
                        engine.ui.is_loading_assets = false;
                        log::info!("Asset loaded and spawned entity: {:?}", base_name);
                    }
                }
            }
            Err(e) => {
                log::error!("Async import failed: {}", e);
                engine.ui.is_loading_assets = false;
                engine.ui.status_message = Some((
                    vec![(format!("ERROR: {}", e), egui::Color32::RED)],
                    std::time::Instant::now(),
                ));
            }
        }
    }

    // --- ASYNC MODEL PARSING RECEIVERS ---
    let mut model_messages = Vec::new();
    for rx in &engine.model_receivers {
        while let Ok(result) = rx.try_recv() {
            model_messages.push(result);
        }
    }

    engine.model_receivers.retain(|rx| {
        !matches!(
            rx.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Disconnected)
        )
    });

    for result in model_messages {
        match result {
            Ok(parsed_data) => {
                let final_name = parsed_data.final_name.clone();
                let path_str = parsed_data.original_path.clone();
                let (model_id, min, max) = engine
                    .render_state
                    .upload_model_data(&mut engine.asset_manager, parsed_data);

                spawn_model(engine, final_name.clone(), model_id, min, max, &path_str);
                engine.ui.status_message = Some((
                    vec![(
                        format!("{} loaded successfully!", final_name),
                        egui::Color32::LIGHT_BLUE,
                    )],
                    std::time::Instant::now(),
                ));
                engine.ui.is_loading_assets = false;
                log::info!("Async GLTF loaded and spawned entity: {:?}", final_name);
            }
            Err(e) => {
                log::error!("Async model import failed: {}", e);
                engine.ui.is_loading_assets = false;
                engine.ui.status_message = Some((
                    vec![(format!("ERROR: {}", e), egui::Color32::RED)],
                    std::time::Instant::now(),
                ));
            }
        }
    }
}

/// Handle a file drag-and-dropped into the application window.
/// Dispatches based on file extension:
/// - `.png`/`.jpg`/`.jpeg` → synchronous texture load + sprite spawn
/// - `.gltf`/`.glb` → async thread-based model parsing
/// - `.fbx` → Native FBX2glTF direct tool converter pipeline (no Python required)
/// Auto-generates unique entity names to avoid collisions.
pub fn handle_dropped_file(engine: &mut AeEngine, path: PathBuf) {
    let path_str = path.to_string_lossy();
    if !ae_renderer::asset::is_safe_path(&path_str) {
        log::error!("Dropped file has an unsafe path, ignoring: {:?}", path_str);
        engine.ui.status_message = Some((
            vec![(
                "Security Error: Unsafe path blocked!".to_string(),
                egui::Color32::RED,
            )],
            std::time::Instant::now(),
        ));
        return;
    }

    log::info!("File dropped for import: {:?}", path);
    println!("File dropped: {:?}", path);
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_str().unwrap_or("").to_lowercase();
        let mut final_name = path
            .file_name()
            .unwrap_or(std::ffi::OsStr::new("Asset"))
            .to_string_lossy()
            .into_owned();
        let base_name = final_name.clone();
        let mut count = 0;
        loop {
            let mut exists = false;
            for ent_ref in engine.ecs.world.iter() {
                if let Ok(name) = engine
                    .ecs
                    .world
                    .get::<&ae_core::ecs::Name>(ent_ref.entity())
                {
                    if name.0 == final_name {
                        exists = true;
                        break;
                    }
                }
            }
            if !exists {
                break;
            }
            count += 1;
            final_name = format!("{} {}", base_name, count);
        }

        if ext_str == "png" || ext_str == "jpg" || ext_str == "jpeg" {
            engine.ui.is_loading_assets = true;
            if let Some(path_str) = path.to_str() {
                let texture_id = engine
                    .render_state
                    .load_texture(&mut engine.asset_manager, path_str);
                spawn_sprite(engine, final_name.clone(), texture_id);
                engine.ui.is_loading_assets = false;
                log::info!("Image loaded and spawned entity: {:?}", final_name);
            }
        } else if ext_str == "gltf" || ext_str == "glb" {
            engine.ui.is_loading_assets = true;
            if let Some(path_str) = path.to_str() {
                engine.ui.status_message = Some((
                    vec![(
                        format!("Loading {} in background...", final_name),
                        egui::Color32::LIGHT_BLUE,
                    )],
                    std::time::Instant::now(),
                ));
                let (tx, rx) = std::sync::mpsc::channel();
                engine.model_receivers.push(rx);

                let path_str_clone = path_str.to_string();
                let name_clone = final_name.clone();

                rayon::spawn(move || {
                    let result = ae_renderer::render::resources::parse_gltf_file(
                        &path_str_clone,
                        name_clone,
                    );
                    let _ = tx.send(result);
                });
            }
        } else if ext_str == "fbx" {
            engine.ui.is_loading_assets = true;
            if let Some(path_str) = path.to_str() {
                engine.ui.status_message = Some((
                    vec![(
                        "Converting FBX file... Please wait.".to_string(),
                        egui::Color32::LIGHT_BLUE,
                    )],
                    std::time::Instant::now(),
                ));

                let (tx, rx) = std::sync::mpsc::channel();
                engine.asset_receivers.push(rx);

                let path_clone = path.to_path_buf();
                let path_str_clone = path_str.to_string();

                rayon::spawn(move || {
                    let tool_path = if cfg!(target_os = "windows") {
                        "tools/windows/FBX2glTF.exe"
                    } else if cfg!(target_os = "macos") {
                        "tools/macos/FBX2glTF_macos"
                    } else {
                        "tools/linux/FBX2glTF_linux"
                    };

                    let p = std::path::Path::new(tool_path);
                    if !p.exists() {
                        let _ = tx.send(Err(format!("FBX2glTF tool not found at '{}'. Please make sure the tools/ folder is intact.", tool_path)));
                        return;
                    }

                    let output_path = path_clone.with_extension("glb");
                    let output_path_str = output_path.to_string_lossy().to_string();

                    // Direct invocation of the local precompiled tool without Python
                    let output = std::process::Command::new(tool_path)
                        .arg("-i")
                        .arg(&path_str_clone)
                        .arg("-o")
                        .arg(&output_path_str)
                        .output();

                    match output {
                        Ok(out) if out.status.success() => {
                            if output_path.exists() {
                                let _ = tx.send(Ok(output_path));
                            } else {
                                let _ = tx.send(Err(
                                    "Conversion reported success but .glb file was not found."
                                        .to_string(),
                                ));
                            }
                        }
                        Ok(out) => {
                            let err_msg = String::from_utf8_lossy(&out.stderr).to_string();
                            let out_msg = String::from_utf8_lossy(&out.stdout).to_string();
                            let combined_err = if err_msg.is_empty() { out_msg } else { err_msg };
                            let _ = tx.send(Err(format!("Conversion failed: {}", combined_err)));
                        }
                        Err(e) => {
                            log::error!("Failed to start FBX2glTF executable: {}", e);
                            let _ =
                                tx.send(Err(format!("Failed to start FBX2glTF executable: {}", e)));
                        }
                    }
                });
            }
        }
    }
}

/// Spawns a model entity with auto-scaling based on AABB dimensions.
/// Applies 100x/1000x scale for micro-models, FBX rotation fix (-90° X),
/// records undo `Command::Spawn`, and selects the new entity.
pub fn spawn_model(
    engine: &mut crate::engine::AeEngine,
    final_name: String,
    model_id: ae_renderer::asset::AssetHandle,
    min: [f32; 3],
    max: [f32; 3],
    path_str: &str,
) {
    let size_x = max[0] - min[0];
    let size_y = max[1] - min[1];
    let size_z = max[2] - min[2];
    let max_dim = size_x.max(size_y).max(size_z);
    let mut auto_scale = 1.0;
    if max_dim < 0.1 {
        auto_scale = 100.0;
        if max_dim < 0.01 {
            auto_scale = 1000.0;
        }
    }

    let mut rotation = ae_core::ecs::Rotation::identity();
    if path_str.contains(".fbx") || path_str.contains("temp_fbx") {
        rotation = ae_core::ecs::Rotation {
            x: -0.7071068,
            y: 0.0,
            z: 0.0,
            w: 0.7071068,
        };
    }

    let new_entity = engine.ecs.world.spawn((
        ae_core::ecs::Name(final_name),
        ae_core::ecs::ModelId(model_id),
        ae_core::ecs::Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        rotation,
        ae_core::ecs::Scale {
            x: auto_scale,
            y: auto_scale,
            z: auto_scale,
        },
        ae_core::ecs::Velocity {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    ));
    let snap = ae_editor::undo_redo::EntitySnapshot::capture(&engine.ecs.world, new_entity);
    engine
        .editor
        .undo_stack
        .push(ae_editor::undo_redo::Command::Spawn(new_entity, snap));
    engine.editor.redo_stack.clear();
    engine.editor.selected_entities.clear();
    engine.editor.selected_entities.push(new_entity);
}

/// Spawns a sprite entity (textured quad) with default transform.
/// Records undo `Command::Spawn` and selects the new entity.
pub fn spawn_sprite(
    engine: &mut crate::engine::AeEngine,
    final_name: String,
    texture_id: ae_renderer::asset::AssetHandle,
) {
    let new_entity = engine.ecs.world.spawn((
        ae_core::ecs::Name(final_name),
        ae_core::ecs::SpriteId(texture_id),
        ae_core::ecs::Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        ae_core::ecs::Rotation::identity(),
        ae_core::ecs::Scale {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        ae_core::ecs::Velocity {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    ));
    let snap = ae_editor::undo_redo::EntitySnapshot::capture(&engine.ecs.world, new_entity);
    engine
        .editor
        .undo_stack
        .push(ae_editor::undo_redo::Command::Spawn(new_entity, snap));
    engine.editor.redo_stack.clear();
    engine.editor.selected_entities.clear();
    engine.editor.selected_entities.push(new_entity);
}