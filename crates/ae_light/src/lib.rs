// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU-free scene lighting components, cascaded shadow projection math, procedural
//! atmosphere/fog evaluation, and WebGPU-aligned uniform layouts for Aeon Engine.

pub mod atmosphere;
pub mod cascade;
pub mod components;
pub mod settings;
pub mod uniforms;

pub use atmosphere::{
    EnvironmentEvaluation, EnvironmentLightConfig, evaluate_environment, linear_clear_rgb,
    sun_direction_from_angles,
};
pub use cascade::{CascadeShadowConfig, compute_cascade_light_space};
pub use components::Light;
pub use settings::{PcfQuality, ShadowResolution, SkyQuality};
pub use uniforms::{LightSpaceUniform, LightUniform, SkyUniform};