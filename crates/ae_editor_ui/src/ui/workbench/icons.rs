// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Canonical texture array layer coordinates for icons in the 64x64x16 Editor Texture Array (`editor_atlas.png`).
//!
//! The texture array contains 16 isolated 64x64 layers, each with its own independent mipmap chain.
//! This completely eliminates texture atlas seam bleeding and cross-tile bilinear/trilinear filtering artifacts.
//! Formatted as `[min_u, min_v, max_u, layer_index]`.

// ── Row 0: Viewport HUD & Gizmo Manipulation Tools (Layers 0..3) ──────────────

/// Viewport tool icon for entity selection (standard pointer arrow) - Layer 0.
pub const ICON_SELECT: [f32; 4] = [0.0, 0.0, 1.0, 0.0];

/// Viewport gizmo tool icon for 3D translation (4-way orthogonal axis arrows) - Layer 1.
pub const ICON_TRANSLATE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

/// Viewport gizmo tool icon for 3D rotation (dual circular orbital arrows) - Layer 2.
pub const ICON_ROTATE: [f32; 4] = [0.0, 0.0, 1.0, 2.0];

/// Viewport gizmo tool icon for 3D scaling (solid cube with opposing corner scale arrows) - Layer 3.
pub const ICON_SCALE: [f32; 4] = [0.0, 0.0, 1.0, 3.0];

// ── Row 1: Core Primitives & Visibility Controls (Layers 4..7) ────────────────

/// Eye icon indicating visible entity state in the scene hierarchy - Layer 4.
pub const ICON_EYE_OPEN: [f32; 4] = [0.0, 0.0, 1.0, 4.0];

/// Eye icon indicating hidden/invisible entity state in the scene hierarchy - Layer 5.
pub const ICON_EYE_CLOSED: [f32; 4] = [0.0, 0.0, 1.0, 5.0];

/// Folder icon representing folder entities, grouping nodes, and filesystem browse buttons - Layer 6.
pub const ICON_FOLDER: [f32; 4] = [0.0, 0.0, 1.0, 6.0];

/// 3D box icon representing mesh geometries, cube shapes, and perspective projection modes - Layer 7.
pub const ICON_CUBE: [f32; 4] = [0.0, 0.0, 1.0, 7.0];

// ── Row 2: Scene Entities & Generic Actions (Layers 8..11) ─────────────────────

/// Light bulb icon representing point, directional, or spot light sources - Layer 8.
pub const ICON_LIGHT: [f32; 4] = [0.0, 0.0, 1.0, 8.0];

/// Cinematic camera icon representing camera entities, audio listeners, and viewport views - Layer 9.
pub const ICON_CAMERA: [f32; 4] = [0.0, 0.0, 1.0, 9.0];

/// Wireframe sphere icon representing spherical meshes, colliders, and bounding spheres - Layer 10.
pub const ICON_SPHERE: [f32; 4] = [0.0, 0.0, 1.0, 10.0];

/// Plus icon representing creation, entity addition, and component addition actions - Layer 11.
pub const ICON_PLUS: [f32; 4] = [0.0, 0.0, 1.0, 11.0];

// ── Row 3: Viewport Modes & Audio Subsystems (Layers 12..15) ───────────────────

/// Wireframe cube icon representing unshaded mesh mode, geometry debugging, and topology rendering - Layer 12.
pub const ICON_WIREFRAME: [f32; 4] = [0.0, 0.0, 1.0, 12.0];

/// 3D Cartesian orthogonal coordinate axes icon representing world coordinate space - Layer 13.
pub const ICON_WORLD: [f32; 4] = [0.0, 0.0, 1.0, 13.0];

/// 3-way outward radiating tripod/axes icon representing local entity coordinate space - Layer 14.
pub const ICON_LOCAL: [f32; 4] = [0.0, 0.0, 1.0, 14.0];

/// Speaker audio source icon representing audio components, sound sources, and audio playback - Layer 15.
pub const ICON_AUDIO: [f32; 4] = [0.0, 0.0, 1.0, 15.0];

// ── Row 4: System, Shaders & Directional Controls (Layers 16..19) ──────────────

/// Cogwheel/gear icon representing engine content, system settings, and configuration - Layer 16.
pub const ICON_GEAR: [f32; 4] = [0.0, 0.0, 1.0, 16.0];

/// 4-pointed diamond sparkle star icon representing shaders, visual effects, and keyframes - Layer 17.
pub const ICON_SPARKLE: [f32; 4] = [0.0, 0.0, 1.0, 17.0];

/// Down-pointing chevron/triangle icon for collapsing sections, moving downward, or dropdown indicators - Layer 18.
pub const ICON_CHEVRON_DOWN: [f32; 4] = [0.0, 0.0, 1.0, 18.0];

/// Up-pointing chevron/triangle icon for expanding sections, moving upward, or increasing values - Layer 19.
pub const ICON_CHEVRON_UP: [f32; 4] = [0.0, 0.0, 1.0, 19.0];

// ── Row 5: Application Branding & Special Identifiers (Layers 20..23) ─────────

/// Aeon Engine official branding cube logo icon - Layer 20.
pub const ICON_AEON_LOGO: [f32; 4] = [0.0, 0.0, 1.0, 20.0];

/// Viewport grid transform snapping toggle icon - Layer 21.
pub const ICON_SNAP_GRID: [f32; 4] = [0.0, 0.0, 1.0, 21.0];

/// Viewport angle/rotation transform snapping toggle icon - Layer 22.
pub const ICON_SNAP_ROTATE: [f32; 4] = [0.0, 0.0, 1.0, 22.0];

const EDITOR_ATLAS_TILE_SIZE: u32 = 64;
const EDITOR_ATLAS_LAYERS: u32 = 256;
const EDITOR_LOGO_LAYER: u32 = 20;

/// Ensures the editor icon sheet (`editor_atlas.png`) and engine logo (`aeon_logo.png`) are
/// decoded and uploaded into `surface`'s managed 2D texture array atlas.
pub fn ensure_editor_icon_atlas(
    surface: &mut irisui::IrisSurfaceHost,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    if surface.has_texture_atlas() {
        return;
    }

    const TOOLS_ICON_BYTES: &[u8] = include_bytes!("../../../../../assets/icons/editor_atlas.png");
    let Ok(img) = image::load_from_memory(TOOLS_ICON_BYTES) else {
        log::warn!("Failed to decode editor_atlas.png texture atlas");
        return;
    };
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let atlas = surface.ensure_texture_atlas(
        device,
        EDITOR_ATLAS_TILE_SIZE,
        EDITOR_ATLAS_LAYERS,
        Some("Iris UI Editor Tools Texture Array"),
    );
    atlas.upload_sprite_sheet_tiles(queue, width, height, rgba.as_raw(), 0);

    const LOGO_ICON_BYTES: &[u8] = include_bytes!("../../../../../assets/icons/aeon_logo.png");
    if let Ok(logo_img) = image::load_from_memory(LOGO_ICON_BYTES) {
        let logo_bytes = logo_img
            .crop_imm(45, 65, 930, 900)
            .resize_exact(
                EDITOR_ATLAS_TILE_SIZE,
                EDITOR_ATLAS_TILE_SIZE,
                image::imageops::FilterType::Lanczos3,
            )
            .to_rgba8()
            .into_raw();
        atlas.upload_rgba_layer(queue, EDITOR_LOGO_LAYER, &logo_bytes);
    }
}

/// Open folder icon representing expandable asset collections and directories - Layer 23.
pub const ICON_FOLDER_OPEN: [f32; 4] = [0.0, 0.0, 1.0, 23.0];

/// Starting layer in the 2D Texture Array reserved for dynamically generated asset thumbnails.
///
/// Layers 0..31 are strictly reserved for canonical static system, editor, and tool icons.
pub const FIRST_THUMBNAIL_LAYER: u32 = 32;