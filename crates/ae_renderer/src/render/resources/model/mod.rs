// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 3D Model Loading and GPU Asset Management Sub-module.
//!
//! Handles glTF/GLB import, scene graph traversal, skeletal armatures,
//! animations, and GPU buffer uploads.
//!

pub mod animation;
pub mod geometry;
pub mod textures;

pub use animation::*;
pub use geometry::*;
pub use textures::*;

use crate::render::RenderState;
use crate::render::types::ModelAsset;
use wgpu::util::DeviceExt;

impl RenderState {
    /// Uploads pre-parsed model data (vertices, indices) to GPU buffers and registers
    /// the asset in the manager with deduplication.
    pub fn upload_model_data(
        &self,
        assets: &mut crate::asset::AssetManager,
        data: crate::asset::ParsedModelData,
    ) -> (crate::asset::AssetHandle, [f32; 3], [f32; 3]) {
        // If it was loaded while we were parsing
        if let Some(&id) = assets.model_path_map.get(&data.canonical_path) {
            return (id, data.min, data.max);
        }

        let v_label = format!("{} Vertex Buffer", data.original_path);
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&v_label),
                contents: bytemuck::cast_slice(&data.all_vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let i_label = format!("{} Index Buffer", data.original_path);
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&i_label),
                contents: bytemuck::cast_slice(&data.all_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let mut embedded_textures = Vec::new();
        for (i, cpu_tex) in data.embedded_textures.into_iter().enumerate() {
            let tex_path = data
                .canonical_path
                .with_extension(format!("embedded_tex_{}", i));
            let tex_label = format!("{}_tex_{}", data.original_path, i);
            embedded_textures
                .push(self.upload_cpu_texture_data(assets, tex_path, cpu_tex, &tex_label));
        }

        let default_texture = embedded_textures.first().copied();

        let source_path_str = data.canonical_path.to_string_lossy().to_string();
        let handle = assets.models.insert(ModelAsset {
            vertex_buffer,
            index_buffer,
            num_indices: data.all_indices.len() as u32,
            source_path: source_path_str,
            min: data.min,
            max: data.max,
            raw_vertices: data.raw_positions,
            raw_indices: data.all_indices,
            raw_skin_vertices: data.raw_skin_vertices,
            gpu_vertices: data.all_vertices,
            skeleton: data.skeleton,
            animations: data.animations,
            default_texture,
            embedded_textures,
            submeshes: data.submeshes,
        });

        assets.model_path_map.insert(data.canonical_path, handle);

        (handle, data.min, data.max)
    }

    /// Synchronous model loader: parses GLTF file, extracts mesh data with AABB bounds,
    /// and uploads to GPU. Includes path deduplication.
    pub fn load_model(
        &self,
        assets: &mut crate::asset::AssetManager,
        path: &str,
    ) -> (crate::asset::AssetHandle, [f32; 3], [f32; 3]) {
        if !crate::asset::is_safe_path(path) {
            core::hint::cold_path();
            log::error!("[SECURITY ERROR] Blocked unsafe model load path: {}", path);
            return (crate::asset::AssetHandle::default(), [0.0; 3], [0.0; 3]);
        }

        // --- DEDUPLICATION LOGIC ---
        let canonical_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(e) => {
                core::hint::cold_path();
                log::error!(
                    "[ERROR] Failed to canonicalize model path '{}': {}",
                    path,
                    e
                );
                return (crate::asset::AssetHandle::default(), [0.0; 3], [0.0; 3]);
            }
        };

        if let Some(&id) = assets.model_path_map.get(&canonical_path) {
            log::info!(
                "Model already loaded, returning existing ID: {:?}",
                canonical_path
            );
            let (min, max) = assets
                .models
                .get(id)
                .map(|m| (m.min, m.max))
                .unwrap_or(([0.0; 3], [0.0; 3]));
            return (id, min, max);
        }

        let import_result = gltf::import(path);
        let (document, buffers, images) = match import_result {
            Ok(res) => res,
            Err(e) => {
                core::hint::cold_path();
                log::error!("Failed to load GLTF Model file {}: {:?}", path, e);
                return (crate::asset::AssetHandle::default(), [0.0; 3], [0.0; 3]);
            }
        };

        let (skeleton, animations) = parse_gltf_skin_and_animations(&document, &buffers);
        let embedded_textures = extract_gltf_all_embedded_textures(&document, &images);
        let default_texture = embedded_textures.first().cloned();

        let (all_vertices, all_indices, raw_positions, raw_skin_vertices, submeshes, min, max) =
            parse_gltf_scene_geometry(&document, &buffers, &images, skeleton.is_some());

        let data = crate::asset::ParsedModelData {
            all_vertices,
            all_indices,
            raw_positions,
            raw_skin_vertices,
            min,
            max,
            canonical_path,
            original_path: path.to_owned(),
            final_name: String::new(),
            skeleton,
            animations,
            default_texture,
            embedded_textures,
            submeshes,
        };

        self.upload_model_data(assets, data)
    }
}

/// Thread-safe GLTF parser for async import pipeline. Extracts vertices, indices,
/// normals, colors, and computes AABB bounds with full scene graph node hierarchy transforms.
pub fn parse_gltf_file(
    path: &str,
    final_name: String,
) -> Result<crate::asset::ParsedModelData, String> {
    if !crate::asset::is_safe_path(path) {
        core::hint::cold_path();
        return Err(format!(
            "Security Error: Blocked unsafe GLTF path: {}",
            path
        ));
    }

    let canonical_path = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(e) => {
            core::hint::cold_path();
            return Err(format!(
                "Failed to canonicalize model path '{}': {}",
                path, e
            ));
        }
    };

    let import_result = gltf::import(path);
    let (document, buffers, images) = match import_result {
        Ok(res) => res,
        Err(e) => {
            core::hint::cold_path();
            return Err(format!("Failed to load GLTF Model file {}: {:?}", path, e));
        }
    };

    let (skeleton, animations) = parse_gltf_skin_and_animations(&document, &buffers);
    let embedded_textures = extract_gltf_all_embedded_textures(&document, &images);
    let default_texture = embedded_textures.first().cloned();

    let (all_vertices, all_indices, raw_positions, raw_skin_vertices, submeshes, min, max) =
        parse_gltf_scene_geometry(&document, &buffers, &images, skeleton.is_some());

    Ok(crate::asset::ParsedModelData {
        all_vertices,
        all_indices,
        raw_positions,
        raw_skin_vertices,
        min,
        max,
        canonical_path,
        original_path: path.to_owned(),
        final_name,
        skeleton,
        animations,
        default_texture,
        embedded_textures,
        submeshes,
    })
}