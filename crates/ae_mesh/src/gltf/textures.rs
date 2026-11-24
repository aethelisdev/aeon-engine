// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! glTF 2.0 embedded RGBA texture extraction and sampler wrap-mode conversion.

fn map_wrap_mode(mode: gltf::texture::WrappingMode) -> ae_texture::WrapMode {
    match mode {
        gltf::texture::WrappingMode::ClampToEdge => ae_texture::WrapMode::ClampToEdge,
        gltf::texture::WrappingMode::MirroredRepeat => ae_texture::WrapMode::MirrorRepeat,
        gltf::texture::WrappingMode::Repeat => ae_texture::WrapMode::Repeat,
    }
}

/// Extracts all embedded RGBA8 textures and sampler configurations from a glTF document.
///
/// Images in `R8`, `R8G8`, or `R8G8B8` are expanded to 4-channel RGBA8; invalid or
/// empty image payloads yield a 1x1 white fallback texture preserving sampler wrap state.
#[must_use]
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
                for chunk in img.pixels.as_chunks::<3>().0 {
                    out.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
                }
                out
            }
            gltf::image::Format::R8 => {
                let mut out = Vec::with_capacity((img.width * img.height * 4) as usize);
                for &b in &img.pixels {
                    out.extend_from_slice(&[b, b, b, 255]);
                }
                out
            }
            gltf::image::Format::R8G8 => {
                let mut out = Vec::with_capacity((img.width * img.height * 4) as usize);
                for chunk in img.pixels.as_chunks::<2>().0 {
                    out.extend_from_slice(&[chunk[0], chunk[1], 0, 255]);
                }
                out
            }
            _ => img.pixels.clone(),
        };

        let mut sampler_config = ae_texture::SamplerConfig::default();
        if let Some(tex) = document.textures().find(|tex| tex.source().index() == i) {
            let sampler = tex.sampler();
            sampler_config.wrap_u = map_wrap_mode(sampler.wrap_s());
            sampler_config.wrap_v = map_wrap_mode(sampler.wrap_t());
        }

        let mut cpu_tex = if rgba_bytes.len() == (img.width * img.height * 4) as usize
            && img.width > 0
            && img.height > 0
        {
            ae_texture::CpuTextureData::new(
                img.width,
                img.height,
                rgba_bytes,
                ae_texture::ColorSpace::Srgb,
                format!("embedded_model_texture_{i}"),
            )
        } else {
            ae_texture::CpuTextureData::new(
                1,
                1,
                vec![255, 255, 255, 255],
                ae_texture::ColorSpace::Srgb,
                format!("fallback_model_texture_{i}"),
            )
        };
        cpu_tex.sampler_config = sampler_config;
        list.push(cpu_tex);
    }
    list
}

/// Extracts the first embedded diffuse texture from glTF images, if present.
#[must_use]
pub fn extract_gltf_embedded_texture(
    document: &gltf::Document,
    images: &[gltf::image::Data],
) -> Option<ae_texture::CpuTextureData> {
    extract_gltf_all_embedded_textures(document, images)
        .into_iter()
        .next()
}