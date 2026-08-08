// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::ModelAsset;
use super::TextureAsset;
use crate::render::RenderState;
use crate::render::types::{DEPTH_FORMAT, SkinVertex, Vertex};
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
    /// Loads a texture from disk using `ae_texture` CPU parsing and uploads it to the GPU.
    /// Checks path security, deduplicates paths, parses RGBA pixels on CPU via `ae_texture`,
    /// and constructs the WGPU texture, sampler, and bind group.
    pub fn load_texture(
        &self,
        assets: &mut crate::asset::AssetManager,
        path: &str,
    ) -> crate::asset::AssetHandle {
        if !ae_texture::is_safe_path(path) {
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
        let is_loaded = assets.texture_path_map.get(&canonical_path).copied();
        if let Some(id) = is_loaded {
            // When load_texture is explicitly called on an existing path, re-read disk pixels and update GPU VRAM
            if let Ok(cpu_data) = ae_texture::parse_texture_file(path, ae_texture::ColorSpace::Srgb)
            {
                self.reload_cpu_texture_data(assets, &canonical_path, cpu_data);
            }
            return id;
        }

        let cpu_data = match ae_texture::parse_texture_file(path, ae_texture::ColorSpace::Srgb) {
            Ok(data) => data,
            Err(e) => {
                core::hint::cold_path();
                log::error!("{}", e);
                return crate::asset::AssetHandle::default();
            }
        };

        self.upload_cpu_texture_data(assets, canonical_path, cpu_data, path)
    }

    /// Constructs a WGPU GPU `TextureAsset` (Texture, Sampler, BindGroup) from CPU texture data.
    /// Writes all mipmap levels to VRAM and configures sampling according to `SamplerConfig`.
    pub fn build_gpu_texture_asset(
        &self,
        mut cpu_data: ae_texture::CpuTextureData,
        original_path: &str,
    ) -> TextureAsset {
        if cpu_data.mipmaps.is_empty() {
            cpu_data.generate_mipmaps();
        }

        let mip_level_count = cpu_data.mipmaps.len().max(1) as u32;

        let texture_size = wgpu::Extent3d {
            width: cpu_data.width,
            height: cpu_data.height,
            depth_or_array_layers: 1,
        };

        let format = match cpu_data.color_space {
            ae_texture::ColorSpace::Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            ae_texture::ColorSpace::Linear => wgpu::TextureFormat::Rgba8Unorm,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(original_path),
            view_formats: &[],
        });

        for (level_idx, mip_level) in cpu_data.mipmaps.iter().enumerate() {
            let mip_size = wgpu::Extent3d {
                width: mip_level.width,
                height: mip_level.height,
                depth_or_array_layers: 1,
            };

            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: level_idx as u32,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &mip_level.bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * mip_level.width),
                    rows_per_image: Some(mip_level.height),
                },
                mip_size,
            );
        }

        let wrap_u = match cpu_data.sampler_config.wrap_u {
            ae_texture::WrapMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            ae_texture::WrapMode::Repeat => wgpu::AddressMode::Repeat,
            ae_texture::WrapMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
        };
        let wrap_v = match cpu_data.sampler_config.wrap_v {
            ae_texture::WrapMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            ae_texture::WrapMode::Repeat => wgpu::AddressMode::Repeat,
            ae_texture::WrapMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
        };
        let mag_filter = match cpu_data.sampler_config.mag_filter {
            ae_texture::FilterMode::Nearest => wgpu::FilterMode::Nearest,
            ae_texture::FilterMode::Linear => wgpu::FilterMode::Linear,
        };
        let min_filter = match cpu_data.sampler_config.min_filter {
            ae_texture::FilterMode::Nearest => wgpu::FilterMode::Nearest,
            ae_texture::FilterMode::Linear => wgpu::FilterMode::Linear,
        };
        let mipmap_filter = match cpu_data.sampler_config.mipmap_filter {
            ae_texture::FilterMode::Nearest => wgpu::MipmapFilterMode::Nearest,
            ae_texture::FilterMode::Linear => wgpu::MipmapFilterMode::Linear,
        };

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wrap_u,
            address_mode_v: wrap_v,
            address_mode_w: wrap_u,
            mag_filter,
            min_filter,
            mipmap_filter,
            anisotropy_clamp: cpu_data.sampler_config.anisotropy_clamp.min(16).max(1),
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

        TextureAsset {
            bind_group,
            source_path: original_path.to_string(),
            width: cpu_data.width,
            height: cpu_data.height,
        }
    }

    /// Uploads CPU texture pixel data (`ae_texture::CpuTextureData`) into a newly created WGPU texture.
    /// Registers the texture asset in `AssetManager` with path deduplication.
    pub fn upload_cpu_texture_data(
        &self,
        assets: &mut crate::asset::AssetManager,
        canonical_path: std::path::PathBuf,
        cpu_data: ae_texture::CpuTextureData,
        original_path: &str,
    ) -> crate::asset::AssetHandle {
        if let Some(&id) = assets.texture_path_map.get(&canonical_path) {
            return id;
        }

        let asset = self.build_gpu_texture_asset(cpu_data, original_path);
        let handle = assets.textures.insert(asset);
        assets.texture_path_map.insert(canonical_path, handle);
        handle
    }

    /// Reloads a modified texture asset from disk live on VRAM.
    /// Re-parses CPU data via `ae_texture`, updates the GPU texture bind group in-place, and preserves the handle.
    pub fn reload_cpu_texture_data(
        &self,
        assets: &mut crate::asset::AssetManager,
        canonical_path: &std::path::Path,
        cpu_data: ae_texture::CpuTextureData,
    ) -> bool {
        if let Some(&handle) = assets.texture_path_map.get(canonical_path) {
            let path_str = canonical_path.to_string_lossy();
            let new_asset = self.build_gpu_texture_asset(cpu_data, &path_str);
            if let Some(existing) = assets.textures.get_mut(handle) {
                existing.bind_group = new_asset.bind_group;
                existing.width = new_asset.width;
                existing.height = new_asset.height;
                log::info!(
                    "[HOT-RELOAD] Live updated GPU VRAM for texture: {:?}",
                    canonical_path
                );
                return true;
            }
        }
        false
    }

    /// Uploads BC1/BC3/BC7 block-compressed texture pixel data (`ae_texture::CompressedTextureData`)
    /// directly into a WGPU compressed texture format.
    /// Achieves 75-80% VRAM bandwidth and memory savings for  material textures.
    pub fn upload_compressed_texture_data(
        &self,
        assets: &mut crate::asset::AssetManager,
        canonical_path: std::path::PathBuf,
        data: ae_texture::CompressedTextureData,
        original_path: &str,
    ) -> crate::asset::AssetHandle {
        if let Some(&id) = assets.texture_path_map.get(&canonical_path) {
            return id;
        }

        let format = match data.format {
            ae_texture::CompressedTextureFormat::Bc1Unorm => wgpu::TextureFormat::Bc1RgbaUnorm,
            ae_texture::CompressedTextureFormat::Bc1Srgb => wgpu::TextureFormat::Bc1RgbaUnormSrgb,
            ae_texture::CompressedTextureFormat::Bc3Unorm => wgpu::TextureFormat::Bc3RgbaUnorm,
            ae_texture::CompressedTextureFormat::Bc3Srgb => wgpu::TextureFormat::Bc3RgbaUnormSrgb,
            ae_texture::CompressedTextureFormat::Bc7Unorm => wgpu::TextureFormat::Bc7RgbaUnorm,
            ae_texture::CompressedTextureFormat::Bc7Srgb => wgpu::TextureFormat::Bc7RgbaUnormSrgb,
        };

        let texture_size = wgpu::Extent3d {
            width: data.width,
            height: data.height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(original_path),
            view_formats: &[],
        });

        let blocks_x = (data.width + 3) / 4;
        let blocks_y = (data.height + 3) / 4;
        let bytes_per_row = blocks_x * data.format.block_size();

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data.bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(blocks_y),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 16,
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
            width: data.width,
            height: data.height,
        });
        assets.texture_path_map.insert(canonical_path, handle);

        handle
    }

    /// Legacy wrapper for uploading raw `image::RgbaImage` data to the GPU.
    /// Delegates internally to `upload_cpu_texture_data`.
    pub fn upload_texture_data(
        &self,
        assets: &mut crate::asset::AssetManager,
        canonical_path: std::path::PathBuf,
        rgba: image::RgbaImage,
        original_path: &str,
    ) -> crate::asset::AssetHandle {
        let (w, h) = rgba.dimensions();
        let cpu_data = ae_texture::CpuTextureData::new(
            w,
            h,
            rgba.into_raw(),
            ae_texture::ColorSpace::Srgb,
            original_path,
        );
        self.upload_cpu_texture_data(assets, canonical_path, cpu_data, original_path)
    }
}

/// Uploads raw CPU texture data to GPU and returns a `TextureAsset` with bind group.
pub fn upload_raw_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_bgl: &wgpu::BindGroupLayout,
    cpu_data: &ae_texture::CpuTextureData,
) -> crate::render::TextureAsset {
    let size = wgpu::Extent3d {
        width: cpu_data.width,
        height: cpu_data.height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Fallback White Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &cpu_data.bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * cpu_data.width),
            rows_per_image: Some(cpu_data.height),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: texture_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("Fallback White Bind Group"),
    });

    crate::render::TextureAsset {
        bind_group,
        source_path: "internal://white_fallback.png".to_string(),
        width: cpu_data.width,
        height: cpu_data.height,
    }
}

/// Parses a texture file from disk using `ae_texture` CPU parser and returns an `image::RgbaImage`.
pub fn parse_texture_file(path: &str) -> Result<image::RgbaImage, String> {
    let cpu_data = ae_texture::parse_texture_file(path, ae_texture::ColorSpace::Srgb)?;
    image::RgbaImage::from_raw(cpu_data.width, cpu_data.height, cpu_data.bytes)
        .ok_or_else(|| format!("Failed to create RgbaImage from raw bytes for '{}'", path))
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
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
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
            raw_skin_vertices: data.raw_skin_vertices,
            gpu_vertices: data.all_vertices,
            skeleton: data.skeleton.clone(),
            animations: data.animations.clone(),
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

        let (skeleton, animations) = parse_gltf_skin_and_animations(&document, &buffers);

        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut raw_positions = Vec::new();
        let mut raw_skin_vertices = Vec::new();

        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                let mut pos_iter = match reader.read_positions() {
                    Some(iter) => iter,
                    None => continue,
                };
                let mut norm_iter = reader.read_normals();
                let mut col_iter = reader.read_colors(0).map(|c| c.into_rgb_f32());
                let mut tex_coord_iter = reader.read_tex_coords(0).map(|tc| tc.into_f32());

                let mut joint_iter = reader.read_joints(0).map(|j| j.into_u16());
                let mut weight_iter = reader.read_weights(0).map(|w| w.into_f32());

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
                    let uv = tex_coord_iter
                        .as_mut()
                        .and_then(|tc| tc.next())
                        .unwrap_or([0.0, 0.0]);

                    let (j_indices, j_weights, has_skin) = if let (Some(j_it), Some(w_it)) =
                        (joint_iter.as_mut(), weight_iter.as_mut())
                    {
                        let raw_j = j_it.next().unwrap_or([0, 0, 0, 0]);
                        let raw_w = w_it.next().unwrap_or([0.0, 0.0, 0.0, 0.0]);
                        (
                            [
                                raw_j[0] as u32,
                                raw_j[1] as u32,
                                raw_j[2] as u32,
                                raw_j[3] as u32,
                            ],
                            raw_w,
                            true,
                        )
                    } else {
                        ([0, 0, 0, 0], [0.0, 0.0, 0.0, 0.0], false)
                    };

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
                        uv,
                    });
                    raw_positions.push(pos);

                    if has_skin && skeleton.is_some() {
                        raw_skin_vertices.push(SkinVertex {
                            bind_position: pos,
                            bind_normal: normal,
                            joint_indices: j_indices,
                            joint_weights: j_weights,
                        });
                    }
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
            raw_skin_vertices,
            min,
            max,
            canonical_path,
            original_path: path.to_owned(),
            final_name: String::new(), // Not used here
            skeleton,
            animations,
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

    let (skeleton, animations) = parse_gltf_skin_and_animations(&document, &buffers);

    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();
    let mut raw_positions = Vec::new();
    let mut raw_skin_vertices = Vec::new();

    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let mut pos_iter = match reader.read_positions() {
                Some(iter) => iter,
                None => continue,
            };
            let mut norm_iter = reader.read_normals();
            let mut col_iter = reader.read_colors(0).map(|c| c.into_rgb_f32());
            let mut tex_coord_iter = reader.read_tex_coords(0).map(|tc| tc.into_f32());

            let mut joint_iter = reader.read_joints(0).map(|j| j.into_u16());
            let mut weight_iter = reader.read_weights(0).map(|w| w.into_f32());

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
                let uv = tex_coord_iter
                    .as_mut()
                    .and_then(|tc| tc.next())
                    .unwrap_or([0.0, 0.0]);

                let (j_indices, j_weights, has_skin) =
                    if let (Some(j_it), Some(w_it)) = (joint_iter.as_mut(), weight_iter.as_mut()) {
                        let raw_j = j_it.next().unwrap_or([0, 0, 0, 0]);
                        let raw_w = w_it.next().unwrap_or([0.0, 0.0, 0.0, 0.0]);
                        (
                            [
                                raw_j[0] as u32,
                                raw_j[1] as u32,
                                raw_j[2] as u32,
                                raw_j[3] as u32,
                            ],
                            raw_w,
                            true,
                        )
                    } else {
                        ([0, 0, 0, 0], [0.0, 0.0, 0.0, 0.0], false)
                    };

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
                    uv,
                });
                raw_positions.push(pos);

                if has_skin && skeleton.is_some() {
                    raw_skin_vertices.push(SkinVertex {
                        bind_position: pos,
                        bind_normal: normal,
                        joint_indices: j_indices,
                        joint_weights: j_weights,
                    });
                }
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
        raw_skin_vertices,
        min,
        max,
        canonical_path,
        original_path: path.to_owned(),
        final_name,
        skeleton,
        animations,
    })
}

/// Helper to extract skeleton joints and animation clips from glTF document.
pub fn parse_gltf_skin_and_animations(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> (
    Option<ae_animation::Skeleton>,
    Vec<ae_animation::AnimationClip>,
) {
    let mut skeleton = None;
    let mut animations = Vec::new();
    let mut node_indices = std::collections::HashMap::new();

    // 1. Parse Skins
    if let Some(skin) = document.skins().next() {
        let reader = skin.reader(|b| Some(&buffers[b.index()]));
        let ibms: Vec<glam::Mat4> = reader
            .read_inverse_bind_matrices()
            .map(|iter| iter.map(|m| glam::Mat4::from_cols_array_2d(&m)).collect())
            .unwrap_or_default();

        let joint_nodes: Vec<gltf::Node> = skin.joints().collect();
        node_indices = joint_nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| (node.index(), idx))
            .collect();

        let mut joints = Vec::with_capacity(joint_nodes.len());
        for (i, node) in joint_nodes.iter().enumerate() {
            let name = node.name().unwrap_or(&format!("Joint_{}", i)).to_string();
            let parent_index = node
                .children()
                .find_map(|child| node_indices.get(&child.index()).copied());
            let local_bind_pose = glam::Mat4::from_cols_array_2d(&node.transform().matrix());
            let ibm = ibms.get(i).copied().unwrap_or(glam::Mat4::IDENTITY);

            joints.push(ae_animation::Joint::new(
                name,
                parent_index,
                local_bind_pose,
                ibm,
            ));
        }

        if !joints.is_empty() {
            skeleton = Some(ae_animation::Skeleton::from_joints(joints));
        }
    }

    // 2. Parse Animations
    for (anim_idx, anim) in document.animations().enumerate() {
        let anim_name = anim
            .name()
            .unwrap_or(&format!("Animation_{}", anim_idx))
            .to_string();
        let mut max_duration = 0.0f32;
        let mut channels = Vec::new();

        for channel in anim.channels() {
            let target_node = channel.target().node().index();
            let joint_index = *node_indices.get(&target_node).unwrap_or(&target_node);
            let reader = channel.reader(|b| Some(&buffers[b.index()]));
            let timestamps: Vec<f32> = match reader.read_inputs() {
                Some(iter) => iter.collect(),
                None => continue,
            };

            if let Some(&last_t) = timestamps.last() {
                if last_t > max_duration {
                    max_duration = last_t;
                }
            }

            let interp = match channel.sampler().interpolation() {
                gltf::animation::Interpolation::Step => ae_animation::Interpolation::Step,
                gltf::animation::Interpolation::Linear => ae_animation::Interpolation::Linear,
                gltf::animation::Interpolation::CubicSpline => {
                    ae_animation::Interpolation::CubicSpline
                }
            };

            let property = match channel.target().property() {
                gltf::animation::Property::Translation => ae_animation::TargetProperty::Translation,
                gltf::animation::Property::Rotation => ae_animation::TargetProperty::Rotation,
                gltf::animation::Property::Scale => ae_animation::TargetProperty::Scale,
                _ => continue,
            };

            if let Some(outputs) = reader.read_outputs() {
                match outputs {
                    gltf::animation::util::ReadOutputs::Translations(iter) => {
                        let kfs: Vec<_> = timestamps
                            .iter()
                            .zip(iter)
                            .map(|(&time, val)| ae_animation::Keyframe {
                                time,
                                value: glam::Vec3::from_array(val),
                            })
                            .collect();
                        channels.push(ae_animation::Channel {
                            joint_index,
                            target_property: property,
                            vector_track: Some(ae_animation::VectorTrack {
                                keyframes: kfs,
                                interpolation: interp,
                            }),
                            rotation_track: None,
                        });
                    }
                    gltf::animation::util::ReadOutputs::Rotations(iter) => {
                        let kfs: Vec<_> = timestamps
                            .iter()
                            .zip(iter.into_f32())
                            .map(|(&time, val)| ae_animation::Keyframe {
                                time,
                                value: glam::Quat::from_array(val),
                            })
                            .collect();
                        channels.push(ae_animation::Channel {
                            joint_index,
                            target_property: property,
                            vector_track: None,
                            rotation_track: Some(ae_animation::RotationTrack {
                                keyframes: kfs,
                                interpolation: interp,
                            }),
                        });
                    }
                    gltf::animation::util::ReadOutputs::Scales(iter) => {
                        let kfs: Vec<_> = timestamps
                            .iter()
                            .zip(iter)
                            .map(|(&time, val)| ae_animation::Keyframe {
                                time,
                                value: glam::Vec3::from_array(val),
                            })
                            .collect();
                        channels.push(ae_animation::Channel {
                            joint_index,
                            target_property: property,
                            vector_track: Some(ae_animation::VectorTrack {
                                keyframes: kfs,
                                interpolation: interp,
                            }),
                            rotation_track: None,
                        });
                    }
                    _ => {}
                }
            }
        }

        if !channels.is_empty() {
            let mut clip = ae_animation::AnimationClip::new(anim_name, max_duration);
            clip.channels = channels;
            animations.push(clip);
        }
    }

    (skeleton, animations)
}