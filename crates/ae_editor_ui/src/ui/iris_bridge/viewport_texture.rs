// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Iris registration for the renderer-owned resolved viewport image.

use super::IrisEditorOverlay;
use irisui::prelude::ExternalTextureId;

/// Stable identity reserved for the editor's resolved 3D viewport image.
pub const VIEWPORT_TEXTURE_ID: ExternalTextureId = ExternalTextureId(0);

impl IrisEditorOverlay {
    /// Updates the reusable D2 binding when the renderer replaces the viewport texture view.
    /// The source view is retained only while registered. Missing views remove the binding, making
    /// the corresponding draw command a safe no-op during resize and asset reload transitions.
    pub(crate) fn set_viewport_texture(
        &mut self,
        device: &wgpu::Device,
        view: Option<&wgpu::TextureView>,
    ) {
        if let Some(view) = view {
            self.renderer.external_textures.set(
                device,
                &self.renderer.external_pipeline,
                VIEWPORT_TEXTURE_ID,
                view,
            );
        } else {
            self.renderer.external_textures.remove(VIEWPORT_TEXTURE_ID);
        }
    }
}