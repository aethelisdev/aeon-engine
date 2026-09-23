// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Subsystem
//!
//! Exposes 100% native Iris UI GPU SDF panel rendering for PBR texture inspection,
//! submesh alpha blending modes, 2D sprite surface settings, and hardware texture array icons.
//!

pub mod empty_state;
pub mod events;
pub mod header;
pub mod panel;
pub mod sprite_view;
pub mod submesh_view;
#[cfg(test)]
mod tests;
pub mod types;

pub use events::{handle_material_click, handle_material_scroll};
pub use header::{MATERIAL_HEADER_HEIGHT, build_material_header};
pub use panel::build_material_panel;
pub use types::{
    MATERIAL_TAG_ADD_COLOR, MATERIAL_TAG_ADD_TEXTURE, MATERIAL_TAG_SPRITE_CHANGE,
    MATERIAL_TAG_SPRITE_REMOVE, MATERIAL_TAG_SUBMESH_ALPHA_BASE, MATERIAL_TAG_SUBMESH_TEXTURE_BASE,
    MaterialAction, MaterialPanelParams, MaterialPanelState, MaterialPanelTargets,
    decode_submesh_alpha_tag, decode_submesh_texture_tag, make_submesh_alpha_tag,
    make_submesh_texture_tag,
};