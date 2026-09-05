// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Types
//!
//! Defines parameter descriptors, hit-testing target registries, and dispatchable actions
//! for the 100% Iris UI GPU SDF Material & Submesh Editor panel.
//!

use irisui::prelude::*;

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
}

/// Hit-testing bounding box cache for interactive elements in the Material Studio.
#[derive(Debug, Clone, Default)]
pub struct MaterialPanelTargets {
    /// Total bounding rectangle of the docked panel.
    pub panel_rect: Rect,
    /// Hit target for picking and replacing the active 2D sprite texture.
    pub btn_change_texture: Option<Rect>,
    /// Hit target for removing the active 2D sprite texture.
    pub btn_remove_texture: Option<Rect>,
    /// Hit target for assigning a texture when none is present.
    pub btn_add_texture: Option<Rect>,
    /// Hit target for adding a Color tint component to an entity.
    pub btn_add_color: Option<Rect>,
    /// Hit targets for submesh alpha mode pill selectors: `(model_handle, submesh_index, target_alpha_mode, button_rect)`.
    pub submesh_alpha_buttons: Vec<(
        ae_renderer::asset::AssetHandle,
        usize,
        ae_renderer::render::types::SubmeshAlphaMode,
        Rect,
    )>,
    /// Hit targets for submesh texture change buttons: `(model_handle, submesh_index, button_rect)`.
    pub submesh_texture_buttons: Vec<(ae_renderer::asset::AssetHandle, usize, Rect)>,
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