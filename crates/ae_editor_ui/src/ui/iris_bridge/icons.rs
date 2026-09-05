// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Canonical UV coordinates for icons in the 256x256 Master Editor Texture Atlas (`editor_atlas.png`).
//!
//! The texture atlas is laid out in a 4x4 grid of 64x64 pixel square slots.
//! UV coordinates are specified in normalized floating-point range `[u_min, v_min, u_max, v_max]`,
//! matching GPU instanced quad samplers across all Iris UI widgets, Viewport HUD overlays,
//! Hierarchy tree rows, and Inspector cards.

// ── Row 0: Viewport HUD & Gizmo Manipulation Tools (v in [0.00..0.25]) ──────────

/// Viewport tool icon for entity selection (standard pointer arrow).
pub const ICON_SELECT: [f32; 4] = [0.00, 0.00, 0.25, 0.25];

/// Viewport gizmo tool icon for 3D translation (4-way orthogonal axis arrows).
pub const ICON_TRANSLATE: [f32; 4] = [0.25, 0.00, 0.50, 0.25];

/// Viewport gizmo tool icon for 3D rotation (dual circular orbital arrows).
pub const ICON_ROTATE: [f32; 4] = [0.50, 0.00, 0.75, 0.25];

/// Viewport gizmo tool icon for 3D scaling (solid cube with opposing corner scale arrows).
pub const ICON_SCALE: [f32; 4] = [0.75, 0.00, 1.00, 0.25];

// ── Row 1: Core Primitives & Visibility Controls (v in [0.25..0.50]) ───────────

/// Eye icon indicating visible entity state in the scene hierarchy.
pub const ICON_EYE_OPEN: [f32; 4] = [0.00, 0.25, 0.25, 0.50];

/// Eye icon indicating hidden/invisible entity state in the scene hierarchy.
pub const ICON_EYE_CLOSED: [f32; 4] = [0.25, 0.25, 0.50, 0.50];

/// Folder icon representing folder entities, grouping nodes, and filesystem browse buttons.
pub const ICON_FOLDER: [f32; 4] = [0.50, 0.25, 0.75, 0.50];

/// 3D box icon representing mesh geometries, cube shapes, and perspective projection modes.
pub const ICON_CUBE: [f32; 4] = [0.75, 0.25, 1.00, 0.50];

// ── Row 2: Scene Entities & Generic Actions (v in [0.50..0.75]) ─────────────────

/// Light bulb icon representing point, directional, or spot light sources.
pub const ICON_LIGHT: [f32; 4] = [0.00, 0.50, 0.25, 0.75];

/// Cinematic camera icon representing camera entities, audio listeners, and viewport views.
pub const ICON_CAMERA: [f32; 4] = [0.25, 0.50, 0.50, 0.75];

/// Wireframe sphere icon representing spherical meshes, colliders, and bounding spheres.
pub const ICON_SPHERE: [f32; 4] = [0.50, 0.50, 0.75, 0.75];

/// Plus icon representing creation, entity addition, and component addition actions.
pub const ICON_PLUS: [f32; 4] = [0.75, 0.50, 1.00, 0.75];

// ── Row 3: Viewport Modes & Audio Subsystems (v in [0.75..1.00]) ────────────────

/// Wireframe cube icon representing unshaded mesh mode, geometry debugging, and topology rendering.
pub const ICON_WIREFRAME: [f32; 4] = [0.00, 0.75, 0.25, 1.00];

/// 3D Cartesian orthogonal coordinate axes icon representing world coordinate space.
pub const ICON_WORLD: [f32; 4] = [0.25, 0.75, 0.50, 1.00];

/// 3-way outward radiating tripod/axes icon representing local entity coordinate space.
pub const ICON_LOCAL: [f32; 4] = [0.50, 0.75, 0.75, 1.00];

/// Speaker audio source icon representing audio components, sound sources, and audio playback.
pub const ICON_AUDIO: [f32; 4] = [0.75, 0.75, 1.00, 1.00];