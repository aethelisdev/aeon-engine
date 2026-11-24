// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 3D Model GPU buffer upload and asset deduplication logic.

use crate::render::RenderState;
use crate::render::types::ModelAsset;
use ae_mesh::{
    ModelSubmesh, ParsedModelData, SubmeshAlphaMode, Vertex, is_safe_path, parse_gltf_bytes,
    parse_gltf_file,
};
use wgpu::util::DeviceExt;

impl RenderState {
    /// Uploads pre-parsed model data (vertices, indices) to GPU buffers and registers
    /// the asset in the manager with deduplication.
    pub fn upload_model_data(
        &self,
        assets: &mut crate::asset::AssetManager,
        data: ParsedModelData,
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

    /// Uploads a pre-cooked [`ae_world::CookedMesh`] directly to GPU vertex/index buffers
    /// using safe zero-parse `bytemuck::try_cast_slice` without invoking `gltf`.
    ///
    /// # Errors
    /// Returns an error string if the cooked vertex stride mismatches [`Vertex`] or casts fail.
    pub fn upload_cooked_mesh(
        &self,
        assets: &mut crate::asset::AssetManager,
        canonical_key: std::path::PathBuf,
        virtual_path: &str,
        cooked: &ae_world::CookedMesh,
    ) -> Result<(crate::asset::AssetHandle, [f32; 3], [f32; 3]), String> {
        if let Some(&id) = assets.model_path_map.get(&canonical_key) {
            return Ok((id, cooked.min, cooked.max));
        }
        let vertices = cooked
            .vertices_as::<Vertex>()
            .map_err(|e| format!("CookedMesh vertex layout mismatch: {e}"))?;
        let indices = cooked
            .indices_as_u32()
            .map_err(|e| format!("CookedMesh index layout mismatch: {e}"))?;

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(virtual_path),
                contents: cooked.vertices.as_bytes(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(virtual_path),
                contents: cooked.indices.as_bytes(),
                usage: wgpu::BufferUsages::INDEX,
            });

        let raw_vertices: Vec<[f32; 3]> = vertices.iter().map(|v| v.position).collect();
        let submeshes = cooked
            .submeshes
            .iter()
            .map(|sub| ModelSubmesh {
                start_index: sub.start_index,
                index_count: sub.index_count,
                texture_index: sub.texture_index.map(|i| i as usize),
                base_color: sub.base_color,
                alpha_mode: match sub.alpha_mode {
                    ae_world::CookedAlphaMode::Opaque => SubmeshAlphaMode::Opaque,
                    ae_world::CookedAlphaMode::Mask => SubmeshAlphaMode::Mask,
                    ae_world::CookedAlphaMode::Blend => SubmeshAlphaMode::Blend,
                },
                alpha_cutoff: sub.alpha_cutoff,
                metallic_factor: sub.metallic_factor,
                roughness_factor: sub.roughness_factor,
                uv_scale: sub.uv_scale,
            })
            .collect();

        let handle = assets.models.insert(ModelAsset {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            source_path: virtual_path.to_string(),
            min: cooked.min,
            max: cooked.max,
            raw_vertices,
            raw_indices: indices.to_vec(),
            raw_skin_vertices: Vec::new(),
            gpu_vertices: vertices.to_vec(),
            skeleton: None,
            animations: Vec::new(),
            default_texture: None,
            embedded_textures: Vec::new(),
            submeshes,
        });
        assets.model_path_map.insert(canonical_key, handle);
        Ok((handle, cooked.min, cooked.max))
    }

    /// Loads a model from a [`ae_world::VfsProvider`], accepting either pre-cooked [`CookedMesh`]
    /// (`AECKM001`, zero GLTF parsing) or raw `.glb` binary streams.
    pub fn load_model_from_vfs(
        &self,
        assets: &mut crate::asset::AssetManager,
        vfs: &dyn ae_world::VfsProvider,
        virtual_path: &str,
    ) -> (crate::asset::AssetHandle, [f32; 3], [f32; 3]) {
        let Ok(clean_key) = ae_world::sanitize_virtual_path(virtual_path) else {
            return (crate::asset::AssetHandle::default(), [0.0; 3], [0.0; 3]);
        };
        if let Some(&id) = assets.model_path_map.get(&clean_key) {
            let (min, max) = assets
                .models
                .get(id)
                .map(|m| (m.min, m.max))
                .unwrap_or(([0.0; 3], [0.0; 3]));
            return (id, min, max);
        }
        let Ok(bytes) = vfs.read_bytes(virtual_path) else {
            return (crate::asset::AssetHandle::default(), [0.0; 3], [0.0; 3]);
        };
        if bytes.starts_with(&ae_world::COOKED_MESH_MAGIC)
            && let Ok(cooked) = ae_world::CookedMesh::decode(&bytes)
            && let Ok(res) =
                self.upload_cooked_mesh(assets, clean_key.clone(), virtual_path, &cooked)
        {
            return res;
        }
        match parse_gltf_bytes(&bytes, clean_key, virtual_path, String::new()) {
            Ok(parsed) => self.upload_model_data(assets, parsed),
            Err(e) => {
                log::error!("Failed to load VFS model '{virtual_path}': {e}");
                (crate::asset::AssetHandle::default(), [0.0; 3], [0.0; 3])
            }
        }
    }

    /// Synchronous model loader: parses GLTF file, extracts mesh data with AABB bounds,
    /// and uploads to GPU. Includes path deduplication.
    pub fn load_model(
        &self,
        assets: &mut crate::asset::AssetManager,
        path: &str,
    ) -> (crate::asset::AssetHandle, [f32; 3], [f32; 3]) {
        if !is_safe_path(path) {
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

        match parse_gltf_file(path, String::new()) {
            Ok(data) => self.upload_model_data(assets, data),
            Err(e) => {
                core::hint::cold_path();
                log::error!("{}", e);
                (crate::asset::AssetHandle::default(), [0.0; 3], [0.0; 3])
            }
        }
    }
}