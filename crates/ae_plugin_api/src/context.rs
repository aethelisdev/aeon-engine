// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Plugin and gameplay execution context wrappers.
//!

use crate::events::DynamicEventBus;
use crate::resources::Resources;
use std::path::Path;

/// Context passed from the engine to plugins on each update call.
pub struct PluginContext<'a> {
    /// Mutable reference to the `hecs::World`.
    pub world: &'a mut hecs::World,
    /// Mutable reference to the `Resources` map.
    pub resources: &'a mut Resources,
    /// Mutable reference to the `DynamicEventBus`.
    pub event_bus: &'a mut DynamicEventBus,
    /// Time elapsed since the last frame, in seconds.
    pub delta_time: f32,
}

impl<'a> PluginContext<'a> {
    /// Creates a new PluginContext.
    pub fn new(
        world: &'a mut hecs::World,
        resources: &'a mut Resources,
        event_bus: &'a mut DynamicEventBus,
        delta_time: f32,
    ) -> Self {
        Self {
            world,
            resources,
            event_bus,
            delta_time,
        }
    }

    /// Returns a mutable reference to the ECS World.
    pub fn world_mut(&mut self) -> &mut hecs::World {
        self.world
    }

    /// Returns an immutable reference to the ECS World.
    pub fn world(&self) -> &hecs::World {
        self.world
    }
}

/// FFI-safe context struct that crosses the `extern "C"` boundary.
#[repr(C)]
pub struct PluginContextFFI<'a> {
    /// Reference to the ECS World.
    pub world: Option<&'a mut hecs::World>,
    /// Reference to the Resources map.
    pub resources: Option<&'a mut Resources>,
    /// Reference to the DynamicEventBus.
    pub event_bus: Option<&'a mut DynamicEventBus>,
    /// Delta time since last frame.
    pub delta_time: f32,
}

impl<'a> PluginContextFFI<'a> {
    /// Provides safe access to the World for plugin code.
    pub fn get_world(&mut self) -> Option<&mut hecs::World> {
        match &mut self.world {
            Some(w) => Some(*w),
            None => None,
        }
    }

    /// Provides safe access to the Resources map for plugin code.
    pub fn get_resources(&mut self) -> Option<&mut Resources> {
        match &mut self.resources {
            Some(r) => Some(*r),
            None => None,
        }
    }

    /// Provides safe access to the DynamicEventBus for plugin code.
    pub fn get_event_bus(&mut self) -> Option<&mut DynamicEventBus> {
        match &mut self.event_bus {
            Some(e) => Some(*e),
            None => None,
        }
    }
}

/// Type signature for the plugin update function exported via `extern "C"`.
pub type PluginUpdateFn = unsafe extern "C" fn(&mut PluginContextFFI<'_>);

/// Trait abstraction for scripting/plugin backends.
pub trait ScriptingBackend {
    /// Returns the human-readable name of this backend.
    fn name(&self) -> &str;

    /// Loads a plugin/script from the given path.
    fn load(&mut self, path: &Path) -> Result<(), String>;

    /// Unloads the currently loaded plugin/script.
    fn unload(&mut self) -> Result<(), String>;

    /// Calls the plugin's update function with the given context.
    fn call_update(&self, ctx: &mut PluginContext) -> Result<(), String>;

    /// Checks if the source file has changed and a reload is needed.
    fn needs_reload(&self) -> bool;

    /// Performs a hot reload cycle: unload old → load new.
    fn reload(&mut self) -> Result<(), String>;
}