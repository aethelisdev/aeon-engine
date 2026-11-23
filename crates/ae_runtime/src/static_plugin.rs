// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Static compile-time gameplay plugin bridge eliminating dynamic library loading in exported builds.

use ae_plugin_api::{PluginContext, PluginContextFFI, ScriptingBackend};
use ae_plugin_host::PluginManager;
use std::path::Path;

/// Safe Rust function signature implemented by statically linked gameplay logic crates.
pub type StaticUpdateFn = fn(&mut PluginContextFFI<'_>);

/// Compile-time [`ScriptingBackend`] invoking a statically linked gameplay update routine
/// without `unsafe` or runtime `.dll`/`.so` discovery.
pub struct StaticGameLogicBackend {
    backend_name: &'static str,
    update_fn: StaticUpdateFn,
}

impl StaticGameLogicBackend {
    /// Wraps a safe gameplay update entrypoint inside a [`ScriptingBackend`] adapter.
    #[must_use]
    pub const fn new(backend_name: &'static str, update_fn: StaticUpdateFn) -> Self {
        Self {
            backend_name,
            update_fn,
        }
    }
}

impl ScriptingBackend for StaticGameLogicBackend {
    fn name(&self) -> &str {
        self.backend_name
    }

    fn load(&mut self, _path: &Path) -> Result<(), String> {
        Ok(())
    }

    fn unload(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn reload(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn call_update(&self, ctx: &mut PluginContext) -> Result<(), String> {
        let mut ffi_ctx = PluginContextFFI {
            world: Some(ctx.world),
            resources: Some(ctx.resources),
            event_bus: Some(ctx.event_bus),
            delta_time: ctx.delta_time,
        };
        (self.update_fn)(&mut ffi_ctx);
        Ok(())
    }

    fn needs_reload(&self) -> bool {
        false
    }
}

/// Registers the workspace `game_logic` crate statically when compiled with
/// `--features static-game-logic`; returns `true` when static gameplay logic is attached.
pub fn attach_static_game_logic(manager: &mut PluginManager) -> bool {
    manager.hot_reload_enabled = false;
    #[cfg(feature = "static-game-logic")]
    {
        manager.add_backend(Box::new(StaticGameLogicBackend::new(
            "Static (game_logic)",
            game_logic::tick_static,
        )));
        true
    }
    #[cfg(not(feature = "static-game-logic"))]
    {
        let _ = manager;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_plugin_api::{DynamicEventBus, Resources};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static STATIC_CALLS: AtomicUsize = AtomicUsize::new(0);

    fn sample_static_logic(ctx: &mut PluginContextFFI<'_>) {
        if let Some(world) = ctx.get_world() {
            let _ = world.len();
            STATIC_CALLS.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn test_static_game_logic_backend_invokes_safely_without_dynamic_loading() {
        STATIC_CALLS.store(0, Ordering::Relaxed);
        let mut manager = PluginManager::new();
        manager.hot_reload_enabled = false;
        manager.add_backend(Box::new(StaticGameLogicBackend::new(
            "Static Test",
            sample_static_logic,
        )));

        let mut world = ae_ecs::World::new();
        let mut resources = Resources::new();
        let mut bus = DynamicEventBus::new();

        manager.tick(&mut world, &mut resources, &mut bus, 1.0 / 60.0);
        assert_eq!(STATIC_CALLS.load(Ordering::Relaxed), 1);
    }
}