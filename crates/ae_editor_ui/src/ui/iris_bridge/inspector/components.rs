// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Built-in Component Handlers Registration
//!
//! Registers all standard ECS component cards into the `InspectorRegistry`.
//!

pub mod animation;
pub mod audio;
pub mod character;
pub mod gameplay;
pub mod hierarchy;
pub mod lod;
pub mod physics;
pub mod registration;
pub mod rendering;
pub mod ui_canvas;

pub use registration::register_all_components;