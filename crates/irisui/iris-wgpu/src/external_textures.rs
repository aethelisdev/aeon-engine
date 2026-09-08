// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Persistent D2 texture registrations for host-owned render targets and images.

use crate::ExternalTexturePipeline;
use iris_core::node::ExternalTextureId;
use std::collections::HashMap;

struct RegisteredTexture {
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

/// Resource table keeping external images separate from texture-array atlas bindings.
/// Register only single-sampled, filterable D2 views. Reusing an identity with an unchanged view
/// performs no resource creation; replacing or removing it releases the old registration.
#[derive(Default)]
pub struct ExternalTextures {
    textures: HashMap<ExternalTextureId, RegisteredTexture>,
}

impl ExternalTextures {
    /// Registers or replaces a source view, returning whether the GPU binding changed.
    /// A reference-counted view handle is retained only on replacement to compare resource
    /// identity across frames. This does not copy image data or allocate per-frame image buffers.
    pub fn set(
        &mut self,
        device: &wgpu::Device,
        pipeline: &ExternalTexturePipeline,
        id: ExternalTextureId,
        view: &wgpu::TextureView,
    ) -> bool {
        if self
            .textures
            .get(&id)
            .is_some_and(|entry| entry.view == *view)
        {
            return false;
        }
        self.textures.insert(
            id,
            RegisteredTexture {
                // Retain the resource identity beyond this borrowed call, not a copy of its pixels.
                view: view.clone(),
                bind_group: pipeline.create_texture_bind_group(device, view),
            },
        );
        true
    }

    /// Releases a host registration; subsequent commands using its identity draw nothing.
    pub fn remove(&mut self, id: ExternalTextureId) {
        self.textures.remove(&id);
    }

    pub(crate) fn get(&self, id: ExternalTextureId) -> Option<&wgpu::BindGroup> {
        self.textures.get(&id).map(|entry| &entry.bind_group)
    }
}