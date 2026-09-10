// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! `ae-2d-animation` - Frame-Based Flipbook and Sprite Sheet Animation for Aeon Engine.
//!
//! Provides timeline sequencing, UV-coordinate slicing, and looping state controls for 2D sprites.
//! - [`SpriteAnimation`]: Core animation controller driving sprite sheet frame progression.
//!

pub mod flipbook;

#[cfg(test)]
mod tests;

pub use flipbook::SpriteAnimation;