// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::render::RenderState;
use crate::render::types::TextureAsset;

impl RenderState {
    pub const MAX_TEXTURE_SIZE: u32 = 8192;

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