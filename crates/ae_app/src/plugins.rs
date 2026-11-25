// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular feature-gated subsystem plugins and [`DefaultCorePlugins`] group.

pub mod subsystems;

#[cfg(feature = "audio")]
pub use subsystems::AudioPlugin;
pub use subsystems::DefaultCorePlugins;
#[cfg(feature = "ui")]
pub use subsystems::HudPlugin;
#[cfg(feature = "physics")]
pub use subsystems::PhysicsPlugin;
#[cfg(feature = "plugins")]
pub use subsystems::ScriptingPlugin;
#[cfg(feature = "state")]
pub use subsystems::StatePlugin;