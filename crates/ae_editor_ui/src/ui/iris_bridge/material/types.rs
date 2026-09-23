// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Types
//!
//! Defines parameter descriptors, hit-testing target registries, and dispatchable actions
//! for the 100% Iris UI GPU SDF Material & Submesh Editor panel.
//!

use irisui::prelude::*;

/// Semantic tag for picking and replacing the active 2D sprite texture.
pub const MATERIAL_TAG_SPRITE_CHANGE: u64 = 0xBB01;

/// Semantic tag for removing the active 2D sprite texture.
pub const MATERIAL_TAG_SPRITE_REMOVE: u64 = 0xBB02;

/// Semantic tag for assigning a texture when no renderable geometry is present.
pub const MATERIAL_TAG_ADD_TEXTURE: u64 = 0xBB03;

/// Semantic tag for adding a Color tint component to an entity.
pub const MATERIAL_TAG_ADD_COLOR: u64 = 0xBB04;

/// Base semantic tag for submesh alpha mode buttons: `MATERIAL_TAG_SUBMESH_ALPHA_BASE + (submesh_idx * 4) + mode_idx`.
pub const MATERIAL_TAG_SUBMESH_ALPHA_BASE: u64 = 0xBB10_0000;

/// Base semantic tag for submesh texture picker buttons: `MATERIAL_TAG_SUBMESH_TEXTURE_BASE + submesh_idx`.
pub const MATERIAL_TAG_SUBMESH_TEXTURE_BASE: u64 = 0xBB20_0000;

/// Encodes a submesh alpha mode pill button tag.
#[inline]
pub fn make_submesh_alpha_tag(
    submesh_idx: usize,
    mode: ae_renderer::render::types::SubmeshAlphaMode,
) -> u64 {
    let mode_idx = match mode {
        ae_renderer::render::types::SubmeshAlphaMode::Opaque => 0,
        ae_renderer::render::types::SubmeshAlphaMode::Mask => 1,
        ae_renderer::render::types::SubmeshAlphaMode::Blend => 2,
    };
    MATERIAL_TAG_SUBMESH_ALPHA_BASE + (submesh_idx as u64 * 4) + mode_idx
}

/// Decodes a submesh alpha mode pill button tag into submesh index and target alpha mode.
#[inline]
pub fn decode_submesh_alpha_tag(
    tag: u64,
) -> Option<(usize, ae_renderer::render::types::SubmeshAlphaMode)> {
    if (MATERIAL_TAG_SUBMESH_ALPHA_BASE..MATERIAL_TAG_SUBMESH_TEXTURE_BASE).contains(&tag) {
        let offset = tag - MATERIAL_TAG_SUBMESH_ALPHA_BASE;
        let submesh_idx = (offset / 4) as usize;
        let mode = match offset % 4 {
            0 => ae_renderer::render::types::SubmeshAlphaMode::Opaque,
            1 => ae_renderer::render::types::SubmeshAlphaMode::Mask,
            2 => ae_renderer::render::types::SubmeshAlphaMode::Blend,
            _ => return None,
        };
        Some((submesh_idx, mode))
    } else {
        None
    }
}

/// Encodes a submesh texture change button tag.
#[inline]
pub fn make_submesh_texture_tag(submesh_idx: usize) -> u64 {
    MATERIAL_TAG_SUBMESH_TEXTURE_BASE + (submesh_idx as u64)
}

/// Decodes a submesh texture change button tag into a submesh index.
#[inline]
pub fn decode_submesh_texture_tag(tag: u64) -> Option<usize> {
    if (MATERIAL_TAG_SUBMESH_TEXTURE_BASE..(MATERIAL_TAG_SUBMESH_TEXTURE_BASE + 0x10_0000))
        .contains(&tag)
    {
        Some((tag - MATERIAL_TAG_SUBMESH_TEXTURE_BASE) as usize)
    } else {
        None
    }
}

/// Parameters required to construct and lay out the Material & Surface Studio panel.
pub struct MaterialPanelParams<'a> {
    /// Absolute bounding rectangle allocated for the material panel in the docking tree.
    pub panel_rect: Rect,
    /// Currently selected entity in the scene hierarchy or viewport, if any.
    pub entity: Option<hecs::Entity>,
    /// Active ECS world reference for component querying.
    pub world: &'a hecs::World,
    /// GPU texture asset repository for metadata resolution.
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    /// GPU 3D model asset repository for submesh slot inspection.
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    /// Current mouse cursor position in window coordinates.
    pub cursor_pos: Point,
    /// Current vertical scroll offset of the scrollable content view.
    pub scroll_y: f32,
    /// Tagged interaction events emitted during this frame for declarative widgets.
    pub events: &'a [(u64, InteractionEvent)],
    /// Currently hovered widget tag, if any.
    pub hovered_tag: Option<u64>,
}

/// Hit-testing bounding box cache for interactive elements in the Material Studio.
#[derive(Debug, Clone, Default)]
pub struct MaterialPanelTargets {
    /// Total bounding rectangle of the docked panel.
    pub panel_rect: Rect,
    /// Model asset handle currently inspected in the material panel, if any.
    pub active_model: Option<ae_renderer::asset::AssetHandle>,
    /// Total computed height of all items in the scrollable content container.
    pub content_height: f32,
}

/// Interactive user actions dispatched from the Material & Surface Studio panel.
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialAction {
    /// Assigns a texture file from local disk to a 2D sprite entity.
    AssignTextureToEntity(hecs::Entity, String),
    /// Removes the texture component from a 2D sprite entity.
    RemoveTextureFromEntity(hecs::Entity),
    /// Sets the alpha blending/testing mode for a specific submesh of a 3D model.
    SetModelSubmeshAlphaMode(
        ae_renderer::asset::AssetHandle,
        usize,
        ae_renderer::render::types::SubmeshAlphaMode,
    ),
    /// Assigns a custom texture path to a specific submesh slot of a 3D model.
    SetModelSubmeshTexture(ae_renderer::asset::AssetHandle, usize, String),
    /// Opens the native OS file picker and assigns the chosen image to the entity.
    PickAndAssignEntityTexture(hecs::Entity),
    /// Opens the native OS file picker and sets the chosen image for a model submesh.
    PickAndSetSubmeshTexture(ae_renderer::asset::AssetHandle, usize),
    /// Adds a default Color component to the selected entity.
    AddColorComponent(hecs::Entity),
    /// Scrolls the panel content container vertically by delta pixels.
    Scroll(f32),
}

/// Persistent interactive state for the Material & Surface Studio panel overlay.
#[derive(Debug, Default, Clone)]
pub struct MaterialPanelState {
    /// Common panel interaction state (targets, scroll_y, search, actions).
    pub interactions:
        crate::ui::iris_bridge::types::PanelInteractionState<MaterialPanelTargets, MaterialAction>,
    /// Selected entity handle cached for material panel interactions.
    pub selected_entity: Option<hecs::Entity>,
    /// Previously baked selected entity handle used for retained dirty-checking.
    pub last_selected_entity: Option<hecs::Entity>,
    /// Previously baked vertical scroll offset used for retained dirty-checking.
    pub last_scroll_y: f32,
    /// Pending tagged interaction events collected during window event routing.
    pub pending_interaction_events: Vec<(u64, InteractionEvent)>,
}

impl std::ops::Deref for MaterialPanelState {
    type Target =
        crate::ui::iris_bridge::types::PanelInteractionState<MaterialPanelTargets, MaterialAction>;
    fn deref(&self) -> &Self::Target {
        &self.interactions
    }
}

impl std::ops::DerefMut for MaterialPanelState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.interactions
    }
}