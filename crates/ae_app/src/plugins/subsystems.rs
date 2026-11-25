// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Feature-isolated [`AppPlugin`] implementations wiring individual subsystems into
//! [`CoreWorld::resources`](crate::CoreWorld::resources) without cross-module coupling.

#[cfg(any(
    feature = "physics",
    feature = "audio",
    feature = "state",
    feature = "plugins",
    feature = "ui"
))]
use crate::app::AeApp;
#[cfg(any(
    feature = "physics",
    feature = "audio",
    feature = "state",
    feature = "plugins",
    feature = "ui"
))]
use crate::plugin::AppPlugin;
use crate::plugin_group::{PluginGroup, PluginGroupBuilder};

/// Attaches Rapier3D physics world storage into [`CoreWorld::resources`](crate::CoreWorld::resources).
#[cfg(feature = "physics")]
pub struct PhysicsPlugin;

#[cfg(feature = "physics")]
impl AppPlugin for PhysicsPlugin {
    fn name(&self) -> &'static str {
        "ae_physics"
    }

    fn build(&self, app: &mut AeApp) {
        if app
            .core
            .resources
            .get::<ae_physics::PhysicsWorld>()
            .is_none()
        {
            app.core.resources.insert(ae_physics::PhysicsWorld::new());
        }
    }
}

/// Attaches 3D spatial audio playback into [`CoreWorld::resources`](crate::CoreWorld::resources).
#[cfg(feature = "audio")]
pub struct AudioPlugin;

#[cfg(feature = "audio")]
impl AppPlugin for AudioPlugin {
    fn name(&self) -> &'static str {
        "ae_audio"
    }

    fn build(&self, app: &mut AeApp) {
        if app
            .core
            .local_resources
            .get::<ae_audio::AudioManager>()
            .is_none()
        {
            app.core
                .local_resources
                .insert(ae_audio::AudioManager::new());
        }
    }
}

/// Attaches the pushdown gameplay/pause state machine into [`CoreWorld::resources`](crate::CoreWorld::resources).
#[cfg(feature = "state")]
pub struct StatePlugin;

#[cfg(feature = "state")]
impl AppPlugin for StatePlugin {
    fn name(&self) -> &'static str {
        "ae_state"
    }

    fn build(&self, app: &mut AeApp) {
        if app.core.resources.get::<ae_state::StateManager>().is_none() {
            app.core.resources.insert(ae_state::StateManager::new());
        }
    }
}

/// Attaches native/static script plugin host into [`CoreWorld::resources`](crate::CoreWorld::resources).
#[cfg(feature = "plugins")]
pub struct ScriptingPlugin;

#[cfg(feature = "plugins")]
impl AppPlugin for ScriptingPlugin {
    fn name(&self) -> &'static str {
        "ae_plugin_host"
    }

    fn build(&self, app: &mut AeApp) {
        if app
            .core
            .local_resources
            .get::<ae_plugin_host::PluginManager>()
            .is_none()
        {
            app.core
                .local_resources
                .insert(ae_plugin_host::PluginManager::new());
        }
    }
}

/// Attaches gameplay HUD event state synchronization into [`CoreWorld::resources`](crate::CoreWorld::resources).
#[cfg(feature = "ui")]
pub struct HudPlugin;

#[cfg(feature = "ui")]
impl AppPlugin for HudPlugin {
    fn name(&self) -> &'static str {
        "ae_ui"
    }

    fn build(&self, app: &mut AeApp) {
        if app.core.resources.get::<ae_ui::InGameHudState>().is_none() {
            app.core.resources.insert(ae_ui::InGameHudState::new());
        }
    }
}

/// Standard pluggable bundle attaching every compiled simulation/runtime subsystem plugin.
///
/// Individual plugins can be disabled cleanly via `.build().disable::<P>()`.
pub struct DefaultCorePlugins;

impl PluginGroup for DefaultCorePlugins {
    fn name(&self) -> &'static str {
        "default_core_plugins"
    }

    fn build(self) -> PluginGroupBuilder {
        let builder = PluginGroupBuilder::new();
        #[cfg(feature = "state")]
        let builder = builder.with_plugin(StatePlugin);
        #[cfg(feature = "physics")]
        let builder = builder.with_plugin(PhysicsPlugin);
        #[cfg(feature = "audio")]
        let builder = builder.with_plugin(AudioPlugin);
        #[cfg(feature = "plugins")]
        let builder = builder.with_plugin(ScriptingPlugin);
        #[cfg(feature = "ui")]
        let builder = builder.with_plugin(HudPlugin);
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_core_plugins_registers_available_features_cleanly() {
        let mut app = AeApp::new();
        app.add_plugins(DefaultCorePlugins);
        #[cfg(feature = "state")]
        assert!(app.plugins.contains("ae_state"));
        #[cfg(feature = "physics")]
        assert!(app.plugins.contains("ae_physics"));
    }
}