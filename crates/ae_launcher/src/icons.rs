// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Canonical hardware texture array coordinates for launcher icons.
//!
//! Maps to the 64×64×16 hardware texture array (`editor_atlas.png`), avoiding emojis
//! and rendering crisp GPU SDF vector quads.
//! Formatted as `[min_u, min_v, max_u, layer_index]`.
//!

/// Pointer/Selection arrow icon for buttons and actions - Layer 0.
pub const ICON_SELECT: [f32; 4] = [0.0, 0.0, 1.0, 0.0];

/// Folder icon for Recent Projects and directory browsing - Layer 6.
pub const ICON_FOLDER: [f32; 4] = [0.0, 0.0, 1.0, 6.0];

/// 3D Cube icon representing 3D spatial project mode - Layer 7.
pub const ICON_CUBE: [f32; 4] = [0.0, 0.0, 1.0, 7.0];

/// Plus icon for New Project creation - Layer 11.
pub const ICON_PLUS: [f32; 4] = [0.0, 0.0, 1.0, 11.0];

/// Wireframe icon for Engine Version and technical info - Layer 12.
pub const ICON_WIREFRAME: [f32; 4] = [0.0, 0.0, 1.0, 12.0];

/// 3D World Cartesian axes icon for general project coordinates - Layer 13.
pub const ICON_WORLD: [f32; 4] = [0.0, 0.0, 1.0, 13.0];