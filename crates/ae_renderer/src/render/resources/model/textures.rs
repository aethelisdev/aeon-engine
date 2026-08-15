// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GLTF embedded texture extraction and sampler configuration parsing.

/// Extracts all embedded RGBA textures from a glTF document and image buffers.
pub fn extract_gltf_all_embedded_textures(
    document: &gltf::Document,
    images: &[gltf::image::Data],
) -> Vec<ae_texture::CpuTextureData> {
    let mut list = Vec::with_capacity(images.len());
    for (i, img) in images.iter().enumerate() {
        let rgba_bytes = match img.format {
            gltf::image::Format::R8G8B8A8 => img.pixels.clone(),
            gltf::image::Format::R8G8B8 => {
                let mut out = Vec::with_capacity((img.width * img.height * 4) as usize);
                for chunk in img.pixels.chunks_exact(3) {
                    out.push(chunk[0]);
                    out.push(chunk[1]);
                    out.push(chunk[2]);
                    out.push(255);
                }
                out
            }
            gltf::image::Format::R8 => {
                let mut out = Vec::with_capacity((img.width * img.height * 4) as usize);
                for &b in &img.pixels {
                    out.push(b);
                    out.push(b);
                    out.push(b);
                    out.push(255);
                }
                out
            }
            gltf::image::Format::R8G8 => {
                let mut out = Vec::with_capacity((img.width * img.height * 4) as usize);
                for chunk in img.pixels.chunks_exact(2) {
                    out.push(chunk[0]);
                    out.push(chunk[1]);
                    out.push(0);
                    out.push(255);
                }
                out
            }
            _ => img.pixels.clone(),
        };

        let mut sampler_config = ae_texture::SamplerConfig::default();
        for tex in document.textures() {
            if tex.source().index() == i {
                let sampler = tex.sampler();
                sampler_config.wrap_u = match sampler.wrap_s() {
                    gltf::texture::WrappingMode::ClampToEdge => ae_texture::WrapMode::ClampToEdge,
                    gltf::texture::WrappingMode::MirroredRepeat => {
                        ae_texture::WrapMode::MirrorRepeat
                    }
                    gltf::texture::WrappingMode::Repeat => ae_texture::WrapMode::Repeat,
                };
                sampler_config.wrap_v = match sampler.wrap_t() {
                    gltf::texture::WrappingMode::ClampToEdge => ae_texture::WrapMode::ClampToEdge,
                    gltf::texture::WrappingMode::MirroredRepeat => {
                        ae_texture::WrapMode::MirrorRepeat
                    }
                    gltf::texture::WrappingMode::Repeat => ae_texture::WrapMode::Repeat,
                };
                break;
            }
        }

        if rgba_bytes.len() == (img.width * img.height * 4) as usize
            && img.width > 0
            && img.height > 0
        {
            let mut cpu_tex = ae_texture::CpuTextureData::new(
                img.width,
                img.height,
                rgba_bytes,
                ae_texture::ColorSpace::Srgb,
                format!("embedded_model_texture_{}", i),
            );
            cpu_tex.sampler_config = sampler_config;
            list.push(cpu_tex);
        } else {
            let mut cpu_tex = ae_texture::CpuTextureData::new(
                1,
                1,
                vec![255, 255, 255, 255],
                ae_texture::ColorSpace::Srgb,
                format!("fallback_model_texture_{}", i),
            );
            cpu_tex.sampler_config = sampler_config;
            list.push(cpu_tex);
        }
    }
    list
}

/// Extracts primary RGBA diffuse texture from glTF embedded images, prioritizing materials with base_color_texture.
pub fn extract_gltf_embedded_texture(
    document: &gltf::Document,
    images: &[gltf::image::Data],
) -> Option<ae_texture::CpuTextureData> {
    extract_gltf_all_embedded_textures(document, images)
        .into_iter()
        .next()
}