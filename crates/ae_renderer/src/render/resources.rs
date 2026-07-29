// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::ModelAsset;
use super::TextureAsset;
use crate::render::RenderState;
use crate::render::types::{DEPTH_FORMAT, Vertex};
use wgpu::util::DeviceExt;

/// Creates a depth/stencil texture view for the given surface configuration and sample count.
pub fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    sample_count: u32,
    label: &str,
) -> wgpu::TextureView {
    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };
    let desc = wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    let texture = device.create_texture(&desc);
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

/// Creates a color render-target texture view (RENDER_ATTACHMENT + TEXTURE_BINDING).
pub fn create_target_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    sample_count: u32,
    label: &str,
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

/// Creates both a color render-target texture and its associated view.
/// This provides access to the raw texture to query the actual driver-selected format
/// at runtime, helping prevent pipeline validation errors on backends like Vulkan.
pub fn create_target_texture_and_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    sample_count: u32,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

impl RenderState {
    pub const MAX_TEXTURE_SIZE: u32 = 8192;

    /// Loads a texture from disk with path deduplication (O(1) lookup),
    /// size guards (max 8192×8192), and GPU upload.
    /// Loads a texture from disk with path deduplication, size guards, and GPU upload.
    /// Internally uses `parse_texture_file` for CPU parsing and `upload_texture_data` for GPU upload.
    pub fn load_texture(
        &self,
        assets: &mut crate::asset::AssetManager,
        path: &str,
    ) -> crate::asset::AssetHandle {
        if !crate::asset::is_safe_path(path) {
            core::hint::cold_path();
            log::error!(
                "[SECURITY ERROR] Blocked unsafe texture load path: {}",
                path
            );
            return crate::asset::AssetHandle::default();
        }

        // --- DEDUPLICATION LOGIC ---
        let canonical_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(e) => {
                core::hint::cold_path();
                log::error!(
                    "[ERROR] Failed to canonicalize texture path '{}': {}",
                    path,
                    e
                );
                return crate::asset::AssetHandle::default();
            }
        };

        if let Some(&id) = assets.texture_path_map.get(&canonical_path) {
            log::info!(
                "Texture already loaded, returning existing ID: {:?}",
                canonical_path
            );
            return id;
        }

        let rgba = match parse_texture_file(path) {
            Ok(img) => img,
            Err(e) => {
                core::hint::cold_path();
                log::error!("{}", e);
                return crate::asset::AssetHandle::default();
            }
        };

        self.upload_texture_data(assets, canonical_path, rgba, path)
    }

    /// Uploads raw texture pixel data (RGBA) directly into a newly created WGPU texture on the GPU.
    /// Registers the texture asset in the asset manager with path deduplication.
    /// Must be called from the main thread or a thread with valid GPU context access.
    pub fn upload_texture_data(
        &self,
        assets: &mut crate::asset::AssetManager,
        canonical_path: std::path::PathBuf,
        rgba: image::RgbaImage,
        original_path: &str,
    ) -> crate::asset::AssetHandle {
        if let Some(&id) = assets.texture_path_map.get(&canonical_path) {
            return id;
        }

        let dimensions = rgba.dimensions();
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(original_path),
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.uniforms.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some(original_path),
        });

        let source_path_str = canonical_path.to_string_lossy().to_string();
        let handle = assets.textures.insert(TextureAsset {
            bind_group,
            source_path: source_path_str,
            width: dimensions.0,
            height: dimensions.1,
        });
        assets.texture_path_map.insert(canonical_path, handle);

        handle
    }
}

pub fn parse_texture_file(path: &str) -> Result<image::RgbaImage, String> {
    if !crate::asset::is_safe_path(path) {
        return Err(format!(
            "Security Error: Blocked unsafe texture path: {}",
            path
        ));
    }

    let img = image::open(path).map_err(|e| format!("Failed to open image '{}': {:?}", path, e))?;
    let rgba = img.to_rgba8();
    let dimensions = rgba.dimensions();

    if dimensions.0 == 0 || dimensions.1 == 0 {
        return Err(format!("Image size of '{}' cannot be 0!", path));
    }
    if dimensions.0 > RenderState::MAX_TEXTURE_SIZE || dimensions.1 > RenderState::MAX_TEXTURE_SIZE
    {
        return Err(format!(
            "Image '{}' is too large! Max: {}",
            path,
            RenderState::MAX_TEXTURE_SIZE
        ));
    }
    Ok(rgba)
}

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

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&(data.original_path.clone() + " Vertex Buffer")),
                contents: bytemuck::cast_slice(&data.all_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&(data.original_path.clone() + " Index Buffer")),
                contents: bytemuck::cast_slice(&data.all_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

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
        // 1. Normalize path
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

        // 2. O(1) Lookup — return stored AABB from the already-loaded asset
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
        let (document, buffers, _) = match import_result {
            Ok(res) => res,
            Err(e) => {
                core::hint::cold_path();
                log::error!("Failed to load GLTF Model file {}: {:?}", path, e);
                return (crate::asset::AssetHandle::default(), [0.0; 3], [0.0; 3]);
            }
        };

        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut raw_positions = Vec::new();

        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                let mut pos_iter = match reader.read_positions() {
                    Some(iter) => iter,
                    None => {
                        log::warn!("GLTF mesh primitive missing positions, skipping.");
                        continue;
                    }
                };
                let mut norm_iter = reader.read_normals();
                let mut col_iter = reader.read_colors(0).map(|c| c.into_rgb_f32());

                let start_vertex = all_vertices.len() as u32;

                while let Some(pos) = pos_iter.next() {
                    let normal = norm_iter
                        .as_mut()
                        .and_then(|n| n.next())
                        .unwrap_or([0.0, 1.0, 0.0]);
                    let color = col_iter
                        .as_mut()
                        .and_then(|c| c.next())
                        .unwrap_or([1.0, 1.0, 1.0]);

                    for i in 0..3 {
                        if pos[i] < min[i] {
                            min[i] = pos[i];
                        }
                        if pos[i] > max[i] {
                            max[i] = pos[i];
                        }
                    }

                    all_vertices.push(Vertex {
                        position: pos,
                        color,
                        normal,
                    });
                    raw_positions.push(pos);
                }

                if let Some(indices) = reader.read_indices() {
                    for idx in indices.into_u32() {
                        all_indices.push(start_vertex + idx);
                    }
                }
            }
        }

        let data = crate::asset::ParsedModelData {
            all_vertices,
            all_indices: all_indices.clone(),
            raw_positions,
            min,
            max,
            canonical_path,
            original_path: path.to_owned(),
            final_name: String::new(), // Not used here
        };

        self.upload_model_data(assets, data)
    }
}

/// Thread-safe GLTF parser for async import pipeline. Extracts vertices, indices,
/// normals, colors, and computes AABB bounds without GPU access.
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
    let (document, buffers, _) = match import_result {
        Ok(res) => res,
        Err(e) => {
            core::hint::cold_path();
            return Err(format!("Failed to load GLTF Model file {}: {:?}", path, e));
        }
    };

    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();
    let mut raw_positions = Vec::new();

    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let mut pos_iter = match reader.read_positions() {
                Some(iter) => iter,
                None => {
                    log::warn!("GLTF mesh primitive missing positions, skipping.");
                    continue;
                }
            };
            let mut norm_iter = reader.read_normals();
            let mut col_iter = reader.read_colors(0).map(|c| c.into_rgb_f32());

            let start_vertex = all_vertices.len() as u32;

            while let Some(pos) = pos_iter.next() {
                let normal = norm_iter
                    .as_mut()
                    .and_then(|n| n.next())
                    .unwrap_or([0.0, 1.0, 0.0]);
                let color = col_iter
                    .as_mut()
                    .and_then(|c| c.next())
                    .unwrap_or([1.0, 1.0, 1.0]);

                for i in 0..3 {
                    if pos[i] < min[i] {
                        min[i] = pos[i];
                    }
                    if pos[i] > max[i] {
                        max[i] = pos[i];
                    }
                }

                all_vertices.push(Vertex {
                    position: pos,
                    color,
                    normal,
                });
                raw_positions.push(pos);
            }

            if let Some(indices) = reader.read_indices() {
                for idx in indices.into_u32() {
                    all_indices.push(start_vertex + idx);
                }
            }
        }
    }

    Ok(crate::asset::ParsedModelData {
        all_vertices,
        all_indices,
        raw_positions,
        min,
        max,
        canonical_path,
        original_path: path.to_owned(),
        final_name,
    })
}