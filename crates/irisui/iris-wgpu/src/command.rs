// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Draw command stream for interleaved Z-ordered UI compositing and hardware scissor clipping.

use crate::quad::QuadInstance;
use crate::texture_pipeline::TextureQuadInstance;

/// Individual render command in an interleaved, Z-ordered UI draw list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawCommand {
    /// Draw a range of instanced SDF quads.
    DrawSdfQuads {
        /// Start index in the instanced quad buffer.
        start: u32,
        /// Number of quads to draw in this batch.
        count: u32,
    },
    /// Draw a textured quad (e.g. 3D engine viewport, image, or thumbnail).
    DrawTexture {
        /// Index of the texture instance in the texture quad buffer.
        instance_index: u32,
    },
    /// Set hardware scissor clipping rectangle.
    SetScissor {
        /// X offset in physical pixels.
        x: u32,
        /// Y offset in physical pixels.
        y: u32,
        /// Width in physical pixels.
        width: u32,
        /// Height in physical pixels.
        height: u32,
    },
    /// Reset hardware scissor rectangle to full screen dimensions.
    ResetScissor,
}

/// Ordered list of draw commands and instanced batches for pixel-perfect Z-order compositing.
#[derive(Debug, Default)]
pub struct DrawCommandList {
    /// Sequential draw and scissor commands in exact front-to-back/back-to-front Z-order.
    pub commands: Vec<DrawCommand>,
    /// Flattened SDF quad instances.
    pub quads: Vec<QuadInstance>,
    /// Flattened texture quad instances.
    pub texture_quads: Vec<TextureQuadInstance>,
}

impl DrawCommandList {
    /// Creates a new, empty draw command list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all commands and instance buffers for reuse without reallocating heap capacity.
    pub fn clear(&mut self) {
        self.commands.clear();
        self.quads.clear();
        self.texture_quads.clear();
    }

    /// Appends an SDF quad to the active batch or begins a new batch.
    pub fn push_quad(&mut self, quad: QuadInstance) {
        let current_len = self.quads.len() as u32;
        self.quads.push(quad);

        if let Some(DrawCommand::DrawSdfQuads { count, .. }) = self.commands.last_mut() {
            *count += 1;
        } else {
            self.commands.push(DrawCommand::DrawSdfQuads {
                start: current_len,
                count: 1,
            });
        }
    }

    /// Appends a textured quad drawing command.
    pub fn push_texture_quad(&mut self, tex_quad: TextureQuadInstance) {
        let instance_index = self.texture_quads.len() as u32;
        self.texture_quads.push(tex_quad);
        self.commands
            .push(DrawCommand::DrawTexture { instance_index });
    }

    /// Sets the active hardware scissor rectangle for subsequent draw calls.
    pub fn push_scissor(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.commands.push(DrawCommand::SetScissor {
            x,
            y,
            width,
            height,
        });
    }

    /// Resets the hardware scissor rectangle to full screen.
    pub fn push_reset_scissor(&mut self) {
        self.commands.push(DrawCommand::ResetScissor);
    }
}