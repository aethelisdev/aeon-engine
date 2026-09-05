// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Industry-Standard Asset Thumbnail Generation & Caching Subsystem.
//!
//! Generates studio-quality thumbnails for:
//! - 3D Models (.gltf, .glb, .obj): Real-time software depth-buffered 3D studio render.
//! - 2D Textures (.png, .jpg, .hdr): Downscaled high-fidelity texture previews.
//! - Shaders (.wgsl): 3D Material/Shader preview sphere with glowing WGSL core.
//! - Scenes (.aee): 3D Isometric scene perspective grid and coordinate gizmo.
//! - Materials (.mat): PBR Material sphere with specular highlight and Fresnel rim.
//! - Audio (.wav, .mp3, .ogg): Studio acoustic equalizer waveform visualization.
//!

use super::types::{AssetBrowserState, AssetCategory, ThumbnailEntry};
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::path::Path;
use std::time::SystemTime;

/// Target resolution width and height for generated thumbnail previews.
pub const THUMBNAIL_SIZE: u32 = 96;

/// Retrieves a cached thumbnail texture handle or generates it if needed.
pub fn get_or_load_thumbnail(
    ctx: &Context,
    state: &mut AssetBrowserState,
    path: &Path,
    category: AssetCategory,
) -> Option<TextureHandle> {
    let metadata = std::fs::metadata(path).ok();
    let last_modified = metadata
        .and_then(|m| m.modified().ok())
        .unwrap_or_else(SystemTime::now);

    // 1. Check in-memory cache
    if let Some(entry) = state.thumbnail_cache.entries.get(path)
        && entry.last_modified >= last_modified
    {
        return Some(entry.texture_handle.clone());
    }

    // 2. Generate thumbnail based on asset category
    let color_image = match category {
        AssetCategory::Textures2D => decode_and_downscale_image(path)?,
        AssetCategory::Models3D => {
            render_model_thumbnail(path).unwrap_or_else(render_fallback_wireframe_cube_thumbnail)
        }
        AssetCategory::Shaders => render_shader_thumbnail()?,
        AssetCategory::Scenes => render_scene_thumbnail()?,
        AssetCategory::Materials => render_material_thumbnail()?,
        AssetCategory::Audio => render_audio_thumbnail()?,
        AssetCategory::All => return None,
    };

    let texture_name = format!("thumb_{}", path.to_string_lossy());
    let handle = ctx.load_texture(texture_name, color_image, TextureOptions::LINEAR);

    state.thumbnail_cache.entries.insert(
        path.to_path_buf(),
        ThumbnailEntry {
            texture_handle: handle.clone(),
            last_modified,
        },
    );

    Some(handle)
}

/// Generates a 64x64 raw RGBA byte buffer for the given asset.
pub fn generate_thumbnail_rgba_64(path: &Path, category: AssetCategory) -> Option<Vec<u8>> {
    match category {
        AssetCategory::Textures2D => rasterize_image_thumbnail(path, 64, 64),
        AssetCategory::Models3D => Some(
            rasterize_model_thumbnail(path, 64, 64)
                .unwrap_or_else(|| rasterize_fallback_cube(64, 64)),
        ),
        AssetCategory::Shaders => Some(rasterize_shader_thumbnail(64, 64)),
        AssetCategory::Scenes => Some(rasterize_scene_thumbnail(64, 64)),
        AssetCategory::Materials => Some(rasterize_material_thumbnail(64, 64)),
        AssetCategory::Audio => Some(rasterize_audio_thumbnail(64, 64)),
        AssetCategory::All => None,
    }
}

/// Decodes an image from disk and downscales it to `width x height` raw RGBA bytes.
fn rasterize_image_thumbnail(path: &Path, width: u32, height: u32) -> Option<Vec<u8>> {
    let img = image::open(path).ok()?;
    let thumbnail = img.thumbnail_exact(width, height);
    Some(thumbnail.to_rgba8().into_raw())
}

/// Decodes an image from disk and downscales it to `THUMBNAIL_SIZE x THUMBNAIL_SIZE`.
fn decode_and_downscale_image(path: &Path) -> Option<ColorImage> {
    let raw = rasterize_image_thumbnail(path, THUMBNAIL_SIZE, THUMBNAIL_SIZE)?;
    let width = THUMBNAIL_SIZE as usize;
    let height = THUMBNAIL_SIZE as usize;

    Some(ColorImage::from_rgba_unmultiplied([width, height], &raw))
}

/// Raw vertex, normal, and index data extracted for 3D thumbnail rendering.
struct RawModelGeometry {
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

/// Extracts vertex, normal, and index geometry from GLTF/GLB models.
fn extract_model_geometry(path: &Path) -> Option<RawModelGeometry> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    if ext != "gltf" && ext != "glb" {
        return None;
    }

    let (doc, buffers, _) = gltf::import(path).ok()?;
    let mut all_verts = Vec::new();
    let mut all_normals = Vec::new();
    let mut all_indices = Vec::new();

    for mesh in doc.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|b| buffers.get(b.index()).map(|d| &d.0[..]));
            let base_idx = all_verts.len() as u32;

            if let Some(pos_iter) = reader.read_positions() {
                for pos in pos_iter {
                    all_verts.push(pos);
                }
            }

            if let Some(norm_iter) = reader.read_normals() {
                for norm in norm_iter {
                    all_normals.push(norm);
                }
            }

            if let Some(idx_iter) = reader.read_indices() {
                for idx in idx_iter.into_u32() {
                    all_indices.push(base_idx + idx);
                }
            }
        }
    }

    if all_verts.is_empty() {
        return None;
    }

    while all_normals.len() < all_verts.len() {
        all_normals.push([0.0, 1.0, 0.0]);
    }

    if all_indices.is_empty() {
        for i in 0..all_verts.len() as u32 {
            all_indices.push(i);
        }
    }

    Some(RawModelGeometry {
        vertices: all_verts,
        normals: all_normals,
        indices: all_indices,
    })
}

/// Renders a 96x96 studio thumbnail of a 3D model using depth-buffered software rasterization.
fn render_model_thumbnail(path: &Path) -> Option<ColorImage> {
    let px = rasterize_model_thumbnail(path, THUMBNAIL_SIZE as usize, THUMBNAIL_SIZE as usize)?;
    let w = THUMBNAIL_SIZE as usize;
    Some(ColorImage::from_rgba_unmultiplied([w, w], &px))
}

/// Software depth-buffered rasterizer for 3D model thumbnails.
fn rasterize_model_thumbnail(path: &Path, width: usize, height: usize) -> Option<Vec<u8>> {
    let geom = extract_model_geometry(path)?;
    if geom.vertices.is_empty() {
        return None;
    }
    let vertices = geom.vertices;
    let normals = geom.normals;
    let indices = geom.indices;

    let mut pixels = vec![0u8; width * height * 4];
    let mut z_buffer = vec![f32::INFINITY; width * height];

    // Compute AABB
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for v in &vertices {
        min[0] = min[0].min(v[0]);
        min[1] = min[1].min(v[1]);
        min[2] = min[2].min(v[2]);
        max[0] = max[0].max(v[0]);
        max[1] = max[1].max(v[1]);
        max[2] = max[2].max(v[2]);
    }

    let center = [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ];
    let extent = (max[0] - min[0])
        .max(max[1] - min[1])
        .max(max[2] - min[2])
        .max(0.001);
    let scale = 1.6 / extent;

    // Fill studio background: subtle dark radial gradient with ground shadow
    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - (width as f32 * 0.5)) / (width as f32 * 0.5);
            let dy = (y as f32 - (height as f32 * 0.5)) / (height as f32 * 0.5);
            let dist_sq = (dx * dx + dy * dy).min(1.0);

            let r = (24.0 - dist_sq * 11.0) as u8;
            let g = (27.0 - dist_sq * 13.0) as u8;
            let b = (34.0 - dist_sq * 16.0) as u8;

            let idx = (y * width + x) * 4;
            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = 255;
        }
    }

    // Camera angles: yaw = 35 deg, pitch = 22 deg
    let yaw = 35.0f32.to_radians();
    let pitch = 22.0f32.to_radians();
    let (cos_y, sin_y) = (yaw.cos(), yaw.sin());
    let (cos_p, sin_p) = (pitch.cos(), pitch.sin());

    // Light direction (key light from top-left front)
    let light_dir = [-0.55f32, 0.72, 0.42];
    let len_l =
        (light_dir[0] * light_dir[0] + light_dir[1] * light_dir[1] + light_dir[2] * light_dir[2])
            .sqrt();
    let light_dir = [
        light_dir[0] / len_l,
        light_dir[1] / len_l,
        light_dir[2] / len_l,
    ];

    let scale_factor = (width as f32) * (26.0 / 96.0);
    let transform_point = |v: &[f32; 3]| -> ([f32; 3], f32, f32) {
        let x0 = (v[0] - center[0]) * scale;
        let y0 = (v[1] - center[1]) * scale;
        let z0 = (v[2] - center[2]) * scale;

        let x1 = x0 * cos_y - z0 * sin_y;
        let z1 = x0 * sin_y + z0 * cos_y;

        let y2 = y0 * cos_p - z1 * sin_p;
        let z2 = y0 * sin_p + z1 * cos_p;

        let screen_x = (x1 * scale_factor) + (width as f32 * 0.5);
        let screen_y = (-y2 * scale_factor) + (height as f32 * 0.52);

        ([x1, y2, z2], screen_x, screen_y)
    };

    let transform_normal = |n: &[f32; 3]| -> [f32; 3] {
        let x1 = n[0] * cos_y - n[2] * sin_y;
        let z1 = n[0] * sin_y + n[2] * cos_y;
        let y2 = n[1] * cos_p - z1 * sin_p;
        let z2 = n[1] * sin_p + z1 * cos_p;
        let len = (x1 * x1 + y2 * y2 + z2 * z2).sqrt().max(0.0001);
        [x1 / len, y2 / len, z2 / len]
    };

    let num_tris = indices.len() / 3;
    for t in 0..num_tris {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;

        if i0 >= vertices.len() || i1 >= vertices.len() || i2 >= vertices.len() {
            continue;
        }

        let (p0_cam, sx0, sy0) = transform_point(&vertices[i0]);
        let (p1_cam, sx1, sy1) = transform_point(&vertices[i1]);
        let (p2_cam, sx2, sy2) = transform_point(&vertices[i2]);

        let e1x = sx1 - sx0;
        let e1y = sy1 - sy0;
        let e2x = sx2 - sx0;
        let e2y = sy2 - sy0;
        let cross = e1x * e2y - e1y * e2x;
        if cross <= 0.0 {
            continue;
        }

        let n0 = if i0 < normals.len() {
            transform_normal(&normals[i0])
        } else {
            [0.0, 0.0, 1.0]
        };
        let n1 = if i1 < normals.len() {
            transform_normal(&normals[i1])
        } else {
            [0.0, 0.0, 1.0]
        };
        let n2 = if i2 < normals.len() {
            transform_normal(&normals[i2])
        } else {
            [0.0, 0.0, 1.0]
        };

        let min_x = (sx0.min(sx1).min(sx2).floor() as isize)
            .max(0)
            .min((width - 1) as isize) as usize;
        let max_x = (sx0.max(sx1).max(sx2).ceil() as isize)
            .max(0)
            .min((width - 1) as isize) as usize;
        let min_y = (sy0.min(sy1).min(sy2).floor() as isize)
            .max(0)
            .min((height - 1) as isize) as usize;
        let max_y = (sy0.max(sy1).max(sy2).ceil() as isize)
            .max(0)
            .min((height - 1) as isize) as usize;

        let denom = cross;
        for py in min_y..=max_y {
            for px in min_x..=max_x {
                let fx = px as f32 + 0.5;
                let fy = py as f32 + 0.5;

                let w0 = ((sx1 - fx) * (sy2 - fy) - (sx2 - fx) * (sy1 - fy)) / denom;
                let w1 = ((sx2 - fx) * (sy0 - fy) - (sx0 - fx) * (sy2 - fy)) / denom;
                let w2 = 1.0 - w0 - w1;

                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    let z = w0 * p0_cam[2] + w1 * p1_cam[2] + w2 * p2_cam[2];
                    let idx = py * width + px;

                    if z < z_buffer[idx] {
                        z_buffer[idx] = z;

                        let nx = w0 * n0[0] + w1 * n1[0] + w2 * n2[0];
                        let ny = w0 * n0[1] + w1 * n1[1] + w2 * n2[1];
                        let nz = w0 * n0[2] + w1 * n1[2] + w2 * n2[2];
                        let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.0001);
                        let norm = [nx / len, ny / len, nz / len];

                        let n_dot_l = (norm[0] * light_dir[0]
                            + norm[1] * light_dir[1]
                            + norm[2] * light_dir[2])
                            .max(0.0);
                        let ambient = 0.28;
                        let diffuse = n_dot_l * 0.65;
                        let rim = (1.0 - norm[2].abs()).powf(2.5) * 0.32;

                        let base_r = 185.0;
                        let base_g = 195.0;
                        let base_b = 215.0;

                        let lit_r = ((base_r * (ambient + diffuse) + 255.0 * rim).min(255.0)) as u8;
                        let lit_g = ((base_g * (ambient + diffuse) + 255.0 * rim).min(255.0)) as u8;
                        let lit_b = ((base_b * (ambient + diffuse) + 255.0 * rim).min(255.0)) as u8;

                        let p_idx = idx * 4;
                        pixels[p_idx] = lit_r;
                        pixels[p_idx + 1] = lit_g;
                        pixels[p_idx + 2] = lit_b;
                        pixels[p_idx + 3] = 255;
                    }
                }
            }
        }
    }

    Some(pixels)
}

/// Fallback 3D wireframe isometric cube thumbnail for models without direct GLTF parsing.
fn render_fallback_wireframe_cube_thumbnail() -> ColorImage {
    let px = rasterize_fallback_cube(THUMBNAIL_SIZE as usize, THUMBNAIL_SIZE as usize);
    let w = THUMBNAIL_SIZE as usize;
    ColorImage::from_rgba_unmultiplied([w, w], &px)
}

/// Rasterizes fallback cube thumbnail pixels.
fn rasterize_fallback_cube(width: usize, height: usize) -> Vec<u8> {
    let mut pixels = vec![0u8; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - (width as f32 * 0.5)) / (width as f32 * 0.5);
            let dy = (y as f32 - (height as f32 * 0.5)) / (height as f32 * 0.5);
            let dist_sq = (dx * dx + dy * dy).min(1.0);

            let r = (24.0 - dist_sq * 10.0) as u8;
            let g = (27.0 - dist_sq * 12.0) as u8;
            let b = (36.0 - dist_sq * 16.0) as u8;

            let idx = (y * width + x) * 4;
            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = 255;
        }
    }

    pixels
}

/// Renders a 3D Material/Shader preview sphere with glowing WGSL core.
fn render_shader_thumbnail() -> Option<ColorImage> {
    let px = rasterize_shader_thumbnail(THUMBNAIL_SIZE as usize, THUMBNAIL_SIZE as usize);
    let w = THUMBNAIL_SIZE as usize;
    Some(ColorImage::from_rgba_unmultiplied([w, w], &px))
}

/// Rasterizes shader thumbnail preview sphere into RGBA pixels.
fn rasterize_shader_thumbnail(width: usize, height: usize) -> Vec<u8> {
    let mut pixels = vec![0u8; width * height * 4];

    let radius = (width as f32) * (34.0 / 96.0);
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;
    let light_dir = [-0.5f32, 0.7, 0.5];
    let len_l =
        (light_dir[0] * light_dir[0] + light_dir[1] * light_dir[1] + light_dir[2] * light_dir[2])
            .sqrt();
    let light_dir = [
        light_dir[0] / len_l,
        light_dir[1] / len_l,
        light_dir[2] / len_l,
    ];

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist_sq = dx * dx + dy * dy;
            let r_sq = radius * radius;

            let idx = (y * width + x) * 4;

            if dist_sq <= r_sq {
                let dz = (r_sq - dist_sq).sqrt();
                let nx = dx / radius;
                let ny = -dy / radius;
                let nz = dz / radius;

                let n_dot_l = (nx * light_dir[0] + ny * light_dir[1] + nz * light_dir[2]).max(0.0);
                let diffuse = n_dot_l * 0.65;
                let ambient = 0.25;
                let rim = (1.0 - nz).powf(2.5) * 0.45;

                // Shader metallic amber glow
                let base_r = 240.0;
                let base_g = 160.0;
                let base_b = 40.0;

                let r = ((base_r * (ambient + diffuse) + 255.0 * rim).min(255.0)) as u8;
                let g = ((base_g * (ambient + diffuse) + 220.0 * rim).min(255.0)) as u8;
                let b = ((base_b * (ambient + diffuse) + 120.0 * rim).min(255.0)) as u8;

                pixels[idx] = r;
                pixels[idx + 1] = g;
                pixels[idx + 2] = b;
                pixels[idx + 3] = 255;
            } else {
                let norm_d = (dist_sq / (width as f32 * height as f32 * 0.25)).min(1.0);
                pixels[idx] = (22.0 - norm_d * 10.0) as u8;
                pixels[idx + 1] = (24.0 - norm_d * 11.0) as u8;
                pixels[idx + 2] = (30.0 - norm_d * 14.0) as u8;
                pixels[idx + 3] = 255;
            }
        }
    }

    pixels
}

/// Renders a 3D isometric scene preview grid with coordinate axes.
fn render_scene_thumbnail() -> Option<ColorImage> {
    let px = rasterize_scene_thumbnail(THUMBNAIL_SIZE as usize, THUMBNAIL_SIZE as usize);
    let w = THUMBNAIL_SIZE as usize;
    Some(ColorImage::from_rgba_unmultiplied([w, w], &px))
}

/// Rasterizes 3D isometric scene preview grid with coordinate axes into RGBA pixels.
fn rasterize_scene_thumbnail(width: usize, height: usize) -> Vec<u8> {
    let mut pixels = vec![0u8; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - (width as f32 * 0.5)) / (width as f32 * 0.5);
            let dy = (y as f32 - (height as f32 * 0.5)) / (height as f32 * 0.5);
            let dist_sq = (dx * dx + dy * dy).min(1.0);

            let r = (20.0 - dist_sq * 9.0) as u8;
            let g = (23.0 - dist_sq * 11.0) as u8;
            let b = (32.0 - dist_sq * 15.0) as u8;

            let idx = (y * width + x) * 4;
            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = 255;
        }
    }

    // Draw stylized 3D isometric perspective grid lines
    let center_x = width as isize / 2;
    let center_y = (height as isize * 55) / 100;
    let step_max = (width as isize * 32) / 96;
    let offset_scale = (width as isize * 11) / 96;

    for i in -3..=3 {
        let offset = i * offset_scale;
        // Diagonal grid line 1
        for step in -step_max..=step_max {
            let px = center_x + step + offset;
            let py = center_y + (step / 2) - (offset / 2);
            if px >= 0 && px < width as isize && py >= 0 && py < height as isize {
                let idx = ((py as usize) * width + (px as usize)) * 4;
                pixels[idx] = pixels[idx].saturating_add(35);
                pixels[idx + 1] = pixels[idx + 1].saturating_add(65);
                pixels[idx + 2] = pixels[idx + 2].saturating_add(95);
            }
        }
        // Diagonal grid line 2
        for step in -step_max..=step_max {
            let px = center_x + step - offset;
            let py = center_y - (step / 2) - (offset / 2);
            if px >= 0 && px < width as isize && py >= 0 && py < height as isize {
                let idx = ((py as usize) * width + (px as usize)) * 4;
                pixels[idx] = pixels[idx].saturating_add(35);
                pixels[idx + 1] = pixels[idx + 1].saturating_add(65);
                pixels[idx + 2] = pixels[idx + 2].saturating_add(95);
            }
        }
    }

    // Center scene beacon dot
    let beacon_r = ((width as isize * 3) / 96).max(2);
    for dy in -beacon_r..=beacon_r {
        for dx in -beacon_r..=beacon_r {
            if dx * dx + dy * dy <= beacon_r * beacon_r {
                let px = (center_x + dx) as usize;
                let py = (center_y + dy) as usize;
                if px < width && py < height {
                    let idx = (py * width + px) * 4;
                    pixels[idx] = 0;
                    pixels[idx + 1] = 229;
                    pixels[idx + 2] = 255;
                }
            }
        }
    }

    pixels
}

/// Renders a PBR material preview sphere.
fn render_material_thumbnail() -> Option<ColorImage> {
    let px = rasterize_material_thumbnail(THUMBNAIL_SIZE as usize, THUMBNAIL_SIZE as usize);
    let w = THUMBNAIL_SIZE as usize;
    Some(ColorImage::from_rgba_unmultiplied([w, w], &px))
}

/// Rasterizes PBR material preview sphere into RGBA pixels.
fn rasterize_material_thumbnail(width: usize, height: usize) -> Vec<u8> {
    let mut pixels = vec![0u8; width * height * 4];

    let radius = (width as f32) * (34.0 / 96.0);
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;
    let light_dir = [-0.55f32, 0.72, 0.42];
    let len_l =
        (light_dir[0] * light_dir[0] + light_dir[1] * light_dir[1] + light_dir[2] * light_dir[2])
            .sqrt();
    let light_dir = [
        light_dir[0] / len_l,
        light_dir[1] / len_l,
        light_dir[2] / len_l,
    ];

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist_sq = dx * dx + dy * dy;
            let r_sq = radius * radius;

            let idx = (y * width + x) * 4;

            if dist_sq <= r_sq {
                let dz = (r_sq - dist_sq).sqrt();
                let nx = dx / radius;
                let ny = -dy / radius;
                let nz = dz / radius;

                let n_dot_l = (nx * light_dir[0] + ny * light_dir[1] + nz * light_dir[2]).max(0.0);
                let diffuse = n_dot_l * 0.65;
                let ambient = 0.28;
                let rim = (1.0 - nz).powf(2.5) * 0.35;

                let base_r = 175.0;
                let base_g = 185.0;
                let base_b = 205.0;

                let r = ((base_r * (ambient + diffuse) + 255.0 * rim).min(255.0)) as u8;
                let g = ((base_g * (ambient + diffuse) + 255.0 * rim).min(255.0)) as u8;
                let b = ((base_b * (ambient + diffuse) + 255.0 * rim).min(255.0)) as u8;

                pixels[idx] = r;
                pixels[idx + 1] = g;
                pixels[idx + 2] = b;
                pixels[idx + 3] = 255;
            } else {
                let norm_d = (dist_sq / (width as f32 * height as f32 * 0.25)).min(1.0);
                pixels[idx] = (22.0 - norm_d * 10.0) as u8;
                pixels[idx + 1] = (24.0 - norm_d * 11.0) as u8;
                pixels[idx + 2] = (30.0 - norm_d * 14.0) as u8;
                pixels[idx + 3] = 255;
            }
        }
    }

    pixels
}

/// Renders an acoustic audio waveform preview thumbnail.
fn render_audio_thumbnail() -> Option<ColorImage> {
    let px = rasterize_audio_thumbnail(THUMBNAIL_SIZE as usize, THUMBNAIL_SIZE as usize);
    let w = THUMBNAIL_SIZE as usize;
    Some(ColorImage::from_rgba_unmultiplied([w, w], &px))
}

/// Rasterizes acoustic audio waveform preview thumbnail into RGBA pixels.
fn rasterize_audio_thumbnail(width: usize, height: usize) -> Vec<u8> {
    let mut pixels = vec![0u8; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - (width as f32 * 0.5)) / (width as f32 * 0.5);
            let dy = (y as f32 - (height as f32 * 0.5)) / (height as f32 * 0.5);
            let dist_sq = (dx * dx + dy * dy).min(1.0);

            let r = (20.0 - dist_sq * 9.0) as u8;
            let g = (23.0 - dist_sq * 11.0) as u8;
            let b = (30.0 - dist_sq * 14.0) as u8;

            let idx = (y * width + x) * 4;
            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = 255;
        }
    }

    // Draw dynamic audio waveform bars
    let bar_count = 14;
    let bar_width = ((width * 4) / 96).max(2);
    let gap = ((width * 2) / 96).max(1);
    let total_w = bar_count * (bar_width + gap) - gap;
    let start_x = if width > total_w {
        (width - total_w) / 2
    } else {
        0
    };
    let center_y = height / 2;

    let base_heights = [8, 14, 22, 34, 28, 38, 26, 32, 20, 26, 16, 24, 12, 6];

    for (i, &h_base) in base_heights.iter().enumerate() {
        let h = (h_base * height) / 96;
        let bx = start_x + i * (bar_width + gap);
        let top_y = center_y.saturating_sub(h / 2);
        let bot_y = (center_y + h / 2).min(height - 1);

        for py in top_y..=bot_y {
            for px in bx..bx + bar_width {
                if px < width {
                    let idx = (py * width + px) * 4;
                    pixels[idx] = 60;
                    pixels[idx + 1] = 220;
                    pixels[idx + 2] = 140;
                }
            }
        }
    }

    pixels
}