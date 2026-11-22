// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector State
//!
//! Encapsulates the lightweight view-model entity name scratch buffer for the Scene Inspector panel.
//! Submenu, popup, Euler gimbal, hex string, HSV picker, saved palette swatches, rename sync,
//! and rename edit sessions are owned natively by `irisui::core::UiTree` and `UiScope::prop_text_synced`.

/// Lightweight persistent view-model state for the Scene Inspector panel.
#[derive(Debug, Default)]
pub struct InspectorPanelState {
    /// Current display name edit buffer of the inspected entity (synchronized automatically via [`irisui::prelude::UiScope::prop_text_synced`]).
    pub inspected_entity_name: String,
}

/// Semantic alias for [`InspectorPanelState`].
pub type InspectorState = InspectorPanelState;