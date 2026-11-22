// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Types
//!
//! Defines parameter descriptors, semantic viewport tags, and state containers
//! for the declarative Iris UI GPU SDF Material & Submesh Editor panel.

use irisui::prelude::*;

/// Semantic tag for the scrollable viewport container of the Material Studio.
pub const MATERIAL_TAG_VIEWPORT: u64 = UiTag::from_str("material.viewport").raw();

/// Semantic tag for the vertical scrollbar track in the Material Studio.
pub const MATERIAL_TAG_SCROLLBAR_TRACK: u64 = UiTag::from_str("material.scrollbar_track").raw();

/// Semantic tag for the vertical scrollbar thumb draggable pill in the Material Studio.
pub const MATERIAL_TAG_SCROLLBAR_THUMB: u64 = UiTag::from_str("material.scrollbar_thumb").raw();

/// Persistent interactive state for the Material & Surface Studio panel.
///
/// Actions are emitted directly through [`irisui::workbench::ActionBus`] without
/// intermediate panel action queues.
#[derive(Debug, Default, Clone)]
pub struct MaterialPanelState {
    /// Selected entity handle cached for scroll offset resets across entity switches.
    pub selected_entity: Option<hecs::Entity>,
    /// Active model asset handle inspected, if any.
    pub active_model: Option<ae_renderer::asset::AssetHandle>,
}

/// Semantic alias for [`MaterialPanelState`].
pub type MaterialState = MaterialPanelState;