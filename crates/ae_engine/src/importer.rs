// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular Asset Importer & Loader Pipeline.
//!
//! Provides a decoupled `AssetLoader` and `AssetLoaderRegistry` architecture
//! for drag-and-drop file ingestion, background async parsing, and entity spawning.
//!

use crate::engine::AeEngine;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Interface for custom asset format loaders and background converters.
pub trait AssetLoader: Send + Sync {
    /// List of lower-case file extensions handled by this loader (e.g. `["png", "jpg"]`, `["gltf", "glb"]`).
    fn supported_extensions(&self) -> &'static [&'static str];

    /// Executes the import/loading logic for this asset file format.
    fn load(&self, engine: &mut AeEngine, path: &Path, final_name: String);
}

/// 2D Sprite / Texture Asset Loader for image formats.
pub struct TextureAssetLoader;

impl AssetLoader for TextureAssetLoader {
    fn supported_extensions(&self) -> &'static [&'static str] {
        &["png", "jpg", "jpeg", "tga", "bmp"]
    }

    fn load(&self, engine: &mut AeEngine, path: &Path, final_name: String) {
        engine.ui.is_loading_assets = true;
        if let Some(path_str) = path.to_str() {
            let texture_id = engine
                .render_state
                .load_texture(&mut engine.asset_manager, path_str);
            engine.ui.is_loading_assets = false;
            log::info!("Image loaded and spawned entity: {:?}", final_name);
            spawn_sprite(engine, final_name, texture_id);
        }
    }
}

/// 3D glTF / GLB Model Asset Loader with async parsing via Rayon.
pub struct GltfAssetLoader;

impl AssetLoader for GltfAssetLoader {
    fn supported_extensions(&self) -> &'static [&'static str] {
        &["gltf", "glb"]
    }

    fn load(&self, engine: &mut AeEngine, path: &Path, final_name: String) {
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

            rayon::spawn(move || {
                let result =
                    ae_renderer::render::resources::parse_gltf_file(&path_str_clone, final_name);
                let _ = tx.send(result);
            });
        }
    }
}

/// 3D FBX Model Asset Loader using direct native FBX2glTF conversion pipeline.
pub struct FbxAssetLoader;

impl AssetLoader for FbxAssetLoader {
    fn supported_extensions(&self) -> &'static [&'static str] {
        &["fbx"]
    }

    fn load(&self, engine: &mut AeEngine, path: &Path, _final_name: String) {
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
                    let _ = tx.send(Err(format!(
                        "FBX2glTF tool not found at '{}'. Please make sure the tools/ folder is intact.",
                        tool_path
                    )));
                    return;
                }

                let output_path = path_clone.with_extension("glb");
                let output_stem = path_clone.with_extension("");
                let output_stem_str = output_stem.to_string_lossy().to_string();

                let output = std::process::Command::new(tool_path)
                    .arg("-i")
                    .arg(&path_str_clone)
                    .arg("-o")
                    .arg(&output_stem_str)
                    .arg("-b")
                    .arg("--pbr-metallic-roughness")
                    .arg("--normalize-weights")
                    .arg("1")
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
                        let _ = tx.send(Err(format!("Failed to start FBX2glTF executable: {}", e)));
                    }
                }
            });
        }
    }
}

/// Central registry managing all file format asset loaders.
#[derive(Default)]
pub struct AssetLoaderRegistry {
    loaders: Vec<Box<dyn AssetLoader>>,
    extension_map: HashMap<&'static str, usize>,
}

impl AssetLoaderRegistry {
    /// Creates a new empty asset loader registry.
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
            extension_map: HashMap::new(),
        }
    }

    /// Registers an asset loader for its supported extensions.
    pub fn register<L: AssetLoader + 'static>(&mut self, loader: L) {
        let idx = self.loaders.len();
        for &ext in loader.supported_extensions() {
            self.extension_map.insert(ext, idx);
        }
        self.loaders.push(Box::new(loader));
    }

    /// Finds a registered loader matching the given file extension.
    pub fn find_loader(&self, extension: &str) -> Option<&dyn AssetLoader> {
        let ext_lower = extension.to_ascii_lowercase();
        self.extension_map
            .get(ext_lower.as_str())
            .map(|&idx| &*self.loaders[idx])
    }

    /// Builds the default engine registry with all built-in loaders.
    pub fn default_registry() -> Self {
        let mut registry = Self::new();
        registry.register(TextureAssetLoader);
        registry.register(GltfAssetLoader);
        registry.register(FbxAssetLoader);
        registry
    }

    /// Returns a reference to the global engine asset loader registry singleton.
    pub fn global() -> &'static AssetLoaderRegistry {
        static REGISTRY: OnceLock<AssetLoaderRegistry> = OnceLock::new();
        REGISTRY.get_or_init(Self::default_registry)
    }
}

/// Process asynchronous asset imports, such as background FBX conversions and model parsing tasks.
pub fn process_async_imports(engine: &mut AeEngine) {
    // 1. Drain asynchronously picked paths from native file dialogs
    let mut dialog_paths = Vec::new();
    for rx in &engine.dialog_receivers {
        while let Ok(path) = rx.try_recv() {
            dialog_paths.push(path);
        }
    }
    engine.dialog_receivers.retain(|rx| {
        !matches!(
            rx.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Disconnected)
        )
    });
    for path in dialog_paths {
        handle_dropped_file(engine, path);
    }

    // 2. Drain FBX converter results
    let mut messages = Vec::new();
    for rx in &engine.asset_receivers {
        while let Ok(result) = rx.try_recv() {
            messages.push(result);
        }
    }
    engine.asset_receivers.retain(|rx| {
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
                    engine.ui.status_message = Some((
                        vec![(
                            "Python installed successfully! You may need to restart the engine for changes to take effect.".to_string(),
                            egui::Color32::LIGHT_BLUE,
                        )],
                        std::time::Instant::now(),
                    ));
                } else if glb_path.exists()
                    && let Some(path_str) = glb_path.to_str()
                {
                    let (model_id, min, max) = engine
                        .render_state
                        .load_model(&mut engine.asset_manager, path_str);
                    let base_name = glb_path
                        .file_name()
                        .unwrap_or(std::ffi::OsStr::new("Model"))
                        .to_string_lossy()
                        .into_owned();

                    log::info!("Asset loaded and spawned entity: {:?}", base_name);
                    spawn_model(engine, base_name, model_id, min, max, path_str);
                    engine.ui.status_message = Some((
                        vec![(
                            "Asset loaded successfully!".to_string(),
                            egui::Color32::LIGHT_BLUE,
                        )],
                        std::time::Instant::now(),
                    ));
                    engine.ui.is_loading_assets = false;
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

    // 3. Drain async model parsing receivers
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

                engine.ui.status_message = Some((
                    vec![(
                        format!("{} loaded successfully!", final_name),
                        egui::Color32::LIGHT_BLUE,
                    )],
                    std::time::Instant::now(),
                ));
                engine.ui.is_loading_assets = false;
                log::info!("Async GLTF loaded and spawned entity: {:?}", final_name);
                spawn_model(engine, final_name, model_id, min, max, &path_str);
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
/// Automatically resolves matching `AssetLoader` via `AssetLoaderRegistry`
/// using `Path::extension()` without magic string searches.
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
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let registry = AssetLoaderRegistry::global();
        if let Some(loader) = registry.find_loader(ext) {
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
                        && name.0 == final_name
                    {
                        exists = true;
                        break;
                    }
                }
                if !exists {
                    break;
                }
                count += 1;
                final_name = format!("{} {}", base_name, count);
            }

            loader.load(engine, &path, final_name);
        } else {
            log::warn!("Unsupported file format dropped: {:?}", ext);
            engine.ui.status_message = Some((
                vec![(
                    format!("Unsupported file extension: .{}", ext),
                    egui::Color32::YELLOW,
                )],
                std::time::Instant::now(),
            ));
        }
    }
}

/// Spawns a model entity with auto-scaling based on AABB dimensions.
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
    if max_dim < 0.05 {
        auto_scale = 100.0;
        if max_dim < 0.005 {
            auto_scale = 1000.0;
        }
    }

    let mut rotation = ae_core::ecs::Rotation::identity();
    let is_fbx = std::path::Path::new(path_str)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("fbx"));
    if is_fbx {
        rotation = ae_core::ecs::Rotation {
            x: -std::f32::consts::FRAC_1_SQRT_2,
            y: 0.0,
            z: 0.0,
            w: std::f32::consts::FRAC_1_SQRT_2,
        };
    }

    let (bbox_min, bbox_max, default_tex) =
        if let Some(model_asset) = engine.asset_manager.models.get(model_id) {
            (
                model_asset.min,
                model_asset.max,
                model_asset.default_texture,
            )
        } else {
            ([-0.5; 3], [0.5; 3], None)
        };

    let new_entity = engine.ecs.world.spawn((
        ae_core::ecs::Name(final_name),
        ae_core::ecs::ModelId(model_id),
        ae_core::ecs::BoundingBox {
            min: bbox_min,
            max: bbox_max,
        },
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

    if let Some(tex) = default_tex {
        let _ = engine
            .ecs
            .world
            .insert_one(new_entity, ae_core::ecs::SpriteId(tex));
    }

    if let Some(model_asset) = engine.asset_manager.models.get(model_id)
        && let Some(ref skel) = model_asset.skeleton
    {
        let _ = engine.ecs.world.insert_one(new_entity, skel.clone());
        if !model_asset.animations.is_empty() {
            let mut player = ae_animation::AnimationPlayer::default();
            player.play(model_asset.animations[0].clone());
            let _ = engine.ecs.world.insert_one(new_entity, player);
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_loader_registry_routing() {
        let registry = AssetLoaderRegistry::global();
        assert!(registry.find_loader("png").is_some());
        assert!(registry.find_loader("PNG").is_some());
        assert!(registry.find_loader("gltf").is_some());
        assert!(registry.find_loader("GLB").is_some());
        assert!(registry.find_loader("fbx").is_some());
        assert!(registry.find_loader("unsupported_xyz").is_none());
    }
}