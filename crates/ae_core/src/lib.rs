// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub mod camera;
/// AE Core - Core engine runtime and orchestration infrastructure.
/// Provides a dynamic downcasting-based `Resources` store to prevent
/// borrow checker conflicts between modular subsystems.
pub mod ecs;
pub mod events;
pub mod math;
pub mod modules;
pub mod spatial;
pub mod time;

pub use ae_plugin_api::Resources;
pub use cgmath;