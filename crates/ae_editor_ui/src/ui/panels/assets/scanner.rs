// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Indexer & Directory Scanner.
//!
//! Scans physical workspace folders and synchronizes with in-memory `AssetManager`
//! storages to build unified asset models with metadata badges.
//!

use super::types::{AssetBrowserState, AssetCategory, AssetItem, AssetSource};
use ae_renderer::asset::{AssetStorage, ShaderAsset};
use ae_renderer::render::{ModelAsset, TextureAsset};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Scans the file system and active asset storages if the scan interval has elapsed.
pub fn rescan_assets_if_needed(
    state: &mut AssetBrowserState,
    models: &AssetStorage<ModelAsset>,
    textures: &AssetStorage<TextureAsset>,
    shaders: &AssetStorage<ShaderAsset>,
) {
    if state.last_scan_time.elapsed() < Duration::from_millis(1500)
        && !state.cached_items.is_empty()
    {
        return;
    }

    state.last_scan_time = Instant::now();

    let root_path = Path::new("assets");
    if !root_path.exists() {
        let _ = std::fs::create_dir_all("assets/models");
        let _ = std::fs::create_dir_all("assets/textures");
        let _ = std::fs::create_dir_all("assets/shaders");
        let _ = std::fs::create_dir_all("assets/scenes");
    }

    let mut discovered_items = Vec::new();
    let mut discovered_subfolders = Vec::new();

    // 1. Recursive File System Walk under `assets/`
    walk_directory(
        root_path,
        AssetSource::Project,
        &mut discovered_items,
        &mut discovered_subfolders,
    );

    // Also check `crates/ae_renderer/src/shaders` for internal shaders
    let internal_shaders_path = Path::new("crates/ae_renderer/src/shaders");
    if internal_shaders_path.exists() {
        walk_directory(
            internal_shaders_path,
            AssetSource::Engine,
            &mut discovered_items,
            &mut discovered_subfolders,
        );
    }

    // 2. Map loaded in-memory handles to discovered files
    for item in &mut discovered_items {
        let canonical_item_path =
            std::fs::canonicalize(&item.path).unwrap_or_else(|_| item.path.clone());

        // Check models
        for (handle, model) in models.iter() {
            if let Ok(model_cp) = std::fs::canonicalize(&model.source_path)
                && model_cp == canonical_item_path
            {
                item.is_loaded_in_memory = true;
                item.model_handle = Some(handle);
                item.metadata_badge = format!("{} Verts", model.raw_vertices.len());
                break;
            }
        }

        // Check textures
        for (handle, texture) in textures.iter() {
            if let Ok(tex_cp) = std::fs::canonicalize(&texture.source_path)
                && tex_cp == canonical_item_path
            {
                item.is_loaded_in_memory = true;
                item.texture_handle = Some(handle);
                item.metadata_badge = format!("{}x{}", texture.width, texture.height);
                break;
            }
        }

        // Check shaders
        for (handle, shader) in shaders.iter() {
            if let Ok(sh_cp) = std::fs::canonicalize(&shader.source_path)
                && (sh_cp == canonical_item_path || shader.name == item.name)
            {
                item.is_loaded_in_memory = true;
                item.shader_handle = Some(handle);
                item.metadata_badge = format!("{:.1} KB", shader.source_code.len() as f64 / 1024.0);
                break;
            }
        }
    }

    // 3. Inject any in-memory assets not found on disk
    for (handle, model) in models.iter() {
        if !discovered_items
            .iter()
            .any(|i| i.model_handle == Some(handle))
        {
            let name = Path::new(&model.source_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Memory Model")
                .to_string();
            discovered_items.push(AssetItem {
                name: name.clone(),
                path: PathBuf::from(&model.source_path),
                relative_path: name,
                category: AssetCategory::Models3D,
                source: AssetSource::Project,
                file_size_bytes: (model.raw_vertices.len() * std::mem::size_of::<[f32; 3]>())
                    as u64,
                metadata_badge: format!("{} Verts", model.raw_vertices.len()),
                is_loaded_in_memory: true,
                model_handle: Some(handle),
                texture_handle: None,
                shader_handle: None,
                is_3d: true,
            });
        }
    }

    for (handle, texture) in textures.iter() {
        if !discovered_items
            .iter()
            .any(|i| i.texture_handle == Some(handle))
        {
            let name = Path::new(&texture.source_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Memory Texture")
                .to_string();
            let source = if texture.source_path.contains("assets/icons") {
                AssetSource::Engine
            } else {
                AssetSource::Project
            };
            discovered_items.push(AssetItem {
                name: name.clone(),
                path: PathBuf::from(&texture.source_path),
                relative_path: name,
                category: AssetCategory::Textures2D,
                source,
                file_size_bytes: (texture.width * texture.height * 4) as u64,
                metadata_badge: format!("{}x{}", texture.width, texture.height),
                is_loaded_in_memory: true,
                model_handle: None,
                texture_handle: Some(handle),
                shader_handle: None,
                is_3d: false,
            });
        }
    }

    for (handle, shader) in shaders.iter() {
        if !discovered_items
            .iter()
            .any(|i| i.shader_handle == Some(handle))
        {
            discovered_items.push(AssetItem {
                name: shader.name.clone(),
                path: PathBuf::from(&shader.source_path),
                relative_path: shader.name.clone(),
                category: AssetCategory::Shaders,
                source: AssetSource::Engine,
                file_size_bytes: shader.source_code.len() as u64,
                metadata_badge: format!("{:.1} KB", shader.source_code.len() as f64 / 1024.0),
                is_loaded_in_memory: true,
                model_handle: None,
                texture_handle: None,
                shader_handle: Some(handle),
                is_3d: false,
            });
        }
    }

    state.cached_items = discovered_items;
    state.subfolders = discovered_subfolders;
}

fn walk_directory(
    dir: &Path,
    source: AssetSource,
    items: &mut Vec<AssetItem>,
    subfolders: &mut Vec<PathBuf>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let effective_source =
            if path == Path::new("assets/icons") || path.starts_with(Path::new("assets/icons")) {
                AssetSource::Engine
            } else {
                source
            };

        if path.is_dir() {
            subfolders.push(path.clone());
            walk_directory(&path, effective_source, items, subfolders);
        } else if path.is_file()
            && let Some(category) = classify_asset_category(&path)
        {
            let metadata = entry.metadata().ok();
            let file_size_bytes = metadata.map(|m| m.len()).unwrap_or(0);
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            let relative_path = path.to_string_lossy().to_string();
            let metadata_badge = AssetBrowserState::format_file_size(file_size_bytes);
            let is_3d = match category {
                AssetCategory::Models3D => true,
                AssetCategory::Scenes => {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if ext == "ae2d" {
                        false
                    } else if ext == "ae3d" {
                        true
                    } else {
                        is_scene_file_3d(&path)
                    }
                }
                _ => false,
            };

            items.push(AssetItem {
                name,
                path,
                relative_path,
                category,
                source: effective_source,
                file_size_bytes,
                metadata_badge,
                is_loaded_in_memory: false,
                model_handle: None,
                texture_handle: None,
                shader_handle: None,
                is_3d,
            });
        }
    }
}

/// Inspects a deserialized entity JSON object to determine if it defines 3D components or attributes.
pub fn is_entity_json_3d(entity: &serde_json::Value) -> bool {
    if let Some(dim) = entity.get("dimension").and_then(|v| v.as_str()) {
        if dim.eq_ignore_ascii_case("3d") {
            return true;
        }
        if dim.eq_ignore_ascii_case("2d") {
            return false;
        }
    }

    if let Some(mp) = entity.get("model_path")
        && !mp.is_null()
        && mp.as_str().is_some_and(|s| !s.is_empty())
    {
        return true;
    }

    if let Some(lod) = entity.get("lod_group")
        && !lod.is_null()
    {
        return true;
    }

    if let Some(shape) = entity.get("shape")
        && !shape.is_null()
        && shape.as_str().is_some_and(|s| !s.is_empty())
    {
        return true;
    }

    if let Some(kcc) = entity.get("character_controller")
        && !kcc.is_null()
    {
        return true;
    }

    if let Some(rb) = entity.get("rigid_body")
        && !rb.is_null()
    {
        return true;
    }

    if let Some(col) = entity.get("collider")
        && !col.is_null()
    {
        return true;
    }

    if let Some(light) = entity.get("light")
        && !light.is_null()
    {
        return true;
    }

    if let Some(mr) = entity.get("mesh_renderer")
        && !mr.is_null()
    {
        return true;
    }

    false
}

/// Inspects a parsed JSON structure to determine if it represents a 3D scene.
///
/// Supports standard arrays of entities (`[SavedEntity]`) as well as enveloped objects (`{"dimension": "3D", ...}`).
pub fn is_scene_json_3d(json: &serde_json::Value) -> bool {
    if let Some(dim) = json.get("dimension").and_then(|v| v.as_str()) {
        if dim.eq_ignore_ascii_case("3d") {
            return true;
        }
        if dim.eq_ignore_ascii_case("2d") {
            return false;
        }
    }

    if let Some(arr) = json.as_array() {
        return arr.iter().any(is_entity_json_3d);
    }

    if let Some(entities) = json.get("entities").and_then(|v| v.as_array()) {
        return entities.iter().any(is_entity_json_3d);
    }

    if json.is_object() {
        return is_entity_json_3d(json);
    }

    false
}

/// Inspects a scene file on disk to determine if it contains 3D entities or 3D scene metadata.
///
/// Reads and parses the file safely without modifying runtime state.
pub fn is_scene_file_3d(path: &Path) -> bool {
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    let reader = std::io::BufReader::new(file);
    let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(reader) else {
        return false;
    };
    is_scene_json_3d(&json)
}

/// Classifies a file extension into an `AssetCategory`.
pub fn classify_asset_category(path: &Path) -> Option<AssetCategory> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "gltf" | "glb" | "obj" | "fbx" => Some(AssetCategory::Models3D),
        "png" | "jpg" | "jpeg" | "tga" | "bmp" | "hdr" => Some(AssetCategory::Textures2D),
        "wgsl" => Some(AssetCategory::Shaders),
        "aee" | "ae3d" | "ae2d" => Some(AssetCategory::Scenes),
        "mat" => Some(AssetCategory::Materials),
        "wav" | "ogg" | "mp3" | "flac" => Some(AssetCategory::Audio),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_asset_category() {
        assert_eq!(
            classify_asset_category(Path::new("assets/models/character.glb")),
            Some(AssetCategory::Models3D)
        );
        assert_eq!(
            classify_asset_category(Path::new("assets/textures/sky.PNG")),
            Some(AssetCategory::Textures2D)
        );
        assert_eq!(
            classify_asset_category(Path::new("shaders/sky.wgsl")),
            Some(AssetCategory::Shaders)
        );
        assert_eq!(
            classify_asset_category(Path::new("scenes/main.aee")),
            Some(AssetCategory::Scenes)
        );
        assert_eq!(
            classify_asset_category(Path::new("scenes/main.ae3d")),
            Some(AssetCategory::Scenes)
        );
        assert_eq!(
            classify_asset_category(Path::new("scenes/level1.ae2d")),
            Some(AssetCategory::Scenes)
        );
        assert_eq!(
            classify_asset_category(Path::new("audio/bgm.ogg")),
            Some(AssetCategory::Audio)
        );
        assert_eq!(
            classify_asset_category(Path::new("random_file.unknown")),
            None
        );
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(AssetBrowserState::format_file_size(500), "500 B");
        assert_eq!(AssetBrowserState::format_file_size(2048), "2.0 KB");
        assert_eq!(
            AssetBrowserState::format_file_size(1024 * 1024 * 5),
            "5.00 MB"
        );
    }
}