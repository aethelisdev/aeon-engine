// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor icon atlas manifest and array layer UV constants.
//!
//! Defines the engine editor's static 64x64 texture-array layer mapping and stages
//! embedded PNG icon sheets into [`irisui::IrisSurfaceHost`] once at startup without
//! storing imperative state inside per-frame UI code.

use irisui::core::UiIcon;

/// Total square resolution of each layer in the editor 2D texture array atlas.
const EDITOR_ATLAS_TILE_SIZE: u32 = 64;
/// Total static + dynamic array capacity reserved on the GPU texture array.
const EDITOR_ATLAS_LAYERS: u32 = 256;
/// Array layer index carrying the cropped application branding mark.
const EDITOR_LOGO_LAYER: u32 = 20;

/// Starting layer index above static editor icon slots (`0..32`) for dynamic thumbnails.
pub const FIRST_THUMBNAIL_LAYER: u32 = 32;

// ── Row 0: Viewport HUD & Gizmo Manipulation Tools (Layers 0..3) ──────────────

/// Viewport tool icon for entity selection (pointer arrow) - Layer 0.
pub const ICON_SELECT: [f32; 4] = UiIcon::layer_uv(0);
/// Viewport gizmo tool icon for 3D translation (orthogonal axis arrows) - Layer 1.
pub const ICON_TRANSLATE: [f32; 4] = UiIcon::layer_uv(1);
/// Viewport gizmo tool icon for 3D rotation (orbital arrows) - Layer 2.
pub const ICON_ROTATE: [f32; 4] = UiIcon::layer_uv(2);
/// Viewport gizmo tool icon for 3D scaling (cube with scale arrows) - Layer 3.
pub const ICON_SCALE: [f32; 4] = UiIcon::layer_uv(3);

// ── Row 1: Core Primitives & Visibility Controls (Layers 4..7) ────────────────

/// Eye icon indicating visible entity state in the scene hierarchy - Layer 4.
pub const ICON_EYE_OPEN: [f32; 4] = UiIcon::layer_uv(4);
/// Eye icon indicating hidden entity state in the scene hierarchy - Layer 5.
pub const ICON_EYE_CLOSED: [f32; 4] = UiIcon::layer_uv(5);
/// Folder icon representing folder entities, grouping nodes, and browse buttons - Layer 6.
pub const ICON_FOLDER: [f32; 4] = UiIcon::layer_uv(6);
/// 3D box icon representing mesh geometries, cube shapes, and perspective projection - Layer 7.
pub const ICON_CUBE: [f32; 4] = UiIcon::layer_uv(7);

// ── Row 2: Scene Entities & Generic Actions (Layers 8..11) ─────────────────────

/// Light bulb icon representing point, directional, or spot light sources - Layer 8.
pub const ICON_LIGHT: [f32; 4] = UiIcon::layer_uv(8);
/// Cinematic camera icon representing camera entities and viewport views - Layer 9.
pub const ICON_CAMERA: [f32; 4] = UiIcon::layer_uv(9);
/// Wireframe sphere icon representing spherical meshes and colliders - Layer 10.
pub const ICON_SPHERE: [f32; 4] = UiIcon::layer_uv(10);
/// Plus icon representing entity and component creation actions - Layer 11.
pub const ICON_PLUS: [f32; 4] = UiIcon::layer_uv(11);

// ── Row 3: Viewport Modes & Audio Subsystems (Layers 12..15) ───────────────────

/// Wireframe cube icon representing unshaded wireframe viewport rendering - Layer 12.
pub const ICON_WIREFRAME: [f32; 4] = UiIcon::layer_uv(12);
/// Cartesian coordinate axes icon representing world coordinate space - Layer 13.
pub const ICON_WORLD: [f32; 4] = UiIcon::layer_uv(13);
/// Outward radiating axes icon representing local entity coordinate space - Layer 14.
pub const ICON_LOCAL: [f32; 4] = UiIcon::layer_uv(14);
/// Speaker icon representing audio sources and listeners - Layer 15.
pub const ICON_AUDIO: [f32; 4] = UiIcon::layer_uv(15);

// ── Row 4: System, Shaders & Directional Controls (Layers 16..19) ──────────────

/// Gear icon representing engine content and system preferences - Layer 16.
pub const ICON_GEAR: [f32; 4] = UiIcon::layer_uv(16);
/// Sparkle star icon representing shaders and keyframe accents - Layer 17.
pub const ICON_SPARKLE: [f32; 4] = UiIcon::layer_uv(17);
/// Down-pointing chevron icon for dropdowns and expanded sections - Layer 18.
pub const ICON_CHEVRON_DOWN: [f32; 4] = UiIcon::layer_uv(18);
/// Up-pointing chevron icon for collapsed sections - Layer 19.
pub const ICON_CHEVRON_UP: [f32; 4] = UiIcon::layer_uv(19);

// ── Row 5: Application Branding & Snap Toggles (Layers 20..23) ─────────────────

/// Engine branding cube logo icon - Layer 20.
pub const ICON_AEON_LOGO: [f32; 4] = UiIcon::layer_uv(EDITOR_LOGO_LAYER);
/// Grid transform snapping toggle icon - Layer 21.
pub const ICON_SNAP_GRID: [f32; 4] = UiIcon::layer_uv(21);
/// Angle/rotation transform snapping toggle icon - Layer 22.
pub const ICON_SNAP_ROTATE: [f32; 4] = UiIcon::layer_uv(22);
/// Open folder icon representing expanded directory nodes - Layer 23.
pub const ICON_FOLDER_OPEN: [f32; 4] = UiIcon::layer_uv(23);

/// Initializes the editor's managed GPU texture array atlas on `surface` once at startup
/// and stages embedded PNG icon sheets for automatic GPU upload on the first render pass.
pub fn register_editor_icon_atlas(surface: &mut irisui::IrisSurfaceHost, device: &wgpu::Device) {
    const TOOLS_ICON_BYTES: &[u8] = include_bytes!("../../../../assets/icons/editor_atlas.png");
    const LOGO_ICON_BYTES: &[u8] = include_bytes!("../../../../assets/icons/aeon_logo.png");

    surface.init_texture_atlas(
        device,
        EDITOR_ATLAS_TILE_SIZE,
        EDITOR_ATLAS_LAYERS,
        FIRST_THUMBNAIL_LAYER,
        Some("Iris UI Editor Tools Texture Array"),
    );

    match image::load_from_memory(TOOLS_ICON_BYTES) {
        Ok(img) => stage_sprite_sheet_rgba(surface, img.to_rgba8()),
        Err(_) => log::warn!("Failed to decode editor_atlas.png texture atlas"),
    }

    image::load_from_memory(LOGO_ICON_BYTES)
        .into_iter()
        .for_each(|logo_img| stage_logo_rgba(surface, logo_img.to_rgba8()));
}

fn stage_sprite_sheet_rgba(surface: &mut irisui::IrisSurfaceHost, rgba: image::RgbaImage) {
    surface.stage_atlas_sprite_sheet(0, rgba.width(), rgba.height(), rgba.into_raw());
}

fn stage_logo_rgba(surface: &mut irisui::IrisSurfaceHost, rgba: image::RgbaImage) {
    surface.stage_atlas_cropped_rgba_layer(
        EDITOR_LOGO_LAYER,
        rgba.width(),
        rgba.height(),
        [45, 65, 930, 900],
        rgba.into_raw(),
    );
}