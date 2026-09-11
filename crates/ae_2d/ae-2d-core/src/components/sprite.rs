// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 2D Sprite component definitions and re-exports.
//!
//! Re-exports the unified [`SpriteRenderer`] component defined in core engine reflection
//! to ensure  reflection, undo/redo serialization, and cross-subsystem integration.
//!

pub use ae_core::ecs::SpriteRenderer;