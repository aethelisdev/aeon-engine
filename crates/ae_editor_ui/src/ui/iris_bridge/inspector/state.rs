// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector State
//!
//! Encapsulates the lightweight view-model entity name scratch buffer for the Scene Inspector panel.
//! Submenu, popup, Euler gimbal, hex string, HSV picker, saved palette swatches, and rename edit
//! sessions are owned natively by `irisui::core::UiTree`.

/// Lightweight persistent view-model state for the Scene Inspector panel.
#[derive(Debug, Default)]
pub struct InspectorPanelState {
    /// Current display name edit buffer of the inspected entity.
    pub inspected_entity_name: String,
}

/// Semantic alias for [`InspectorPanelState`].
pub type InspectorState = InspectorPanelState;

impl InspectorPanelState {
    /// Synchronizes the cached entity name scratch buffer after building the inspector view.
    pub(crate) fn sync_entity_name(
        &mut self,
        selected_entity: Option<hecs::Entity>,
        world: &hecs::World,
    ) {
        self.inspected_entity_name.clear();
        selected_entity
            .and_then(|entity| world.get::<&ae_core::ecs::Name>(entity).ok())
            .into_iter()
            .for_each(|name| self.inspected_entity_name.push_str(name.0.as_str()));
    }
}