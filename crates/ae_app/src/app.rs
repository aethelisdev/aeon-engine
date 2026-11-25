// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular application coordinator orchestrating [`CoreWorld`], [`AppSchedule`], and [`PluginRegistry`].

use crate::core_world::CoreWorld;
use crate::plugin::{AppPlugin, PluginRegistry};
use crate::schedule::{AppPhase, AppSchedule};

/// Thin application coordinator uniting core ECS state, plugin registration, and frame phases.
///
/// Mirrors the clean app/schedule pattern without holding GPU pipelines, editor panels, or
/// platform window handles directly inside the core app struct.
pub struct AeApp {
    /// Core ECS world, resources, events, timing, and input state.
    pub core: CoreWorld,
    /// Scheduled system pipelines partitioned by [`AppPhase`].
    pub schedules: AppSchedule,
    /// Deduplicated registry of attached [`AppPlugin`] names.
    pub plugins: PluginRegistry,
    /// Application shutdown flag queried by platform event loops.
    pub should_exit: bool,
    startup_ran: bool,
}

impl AeApp {
    /// Creates an empty [`AeApp`] ready for plugin attachment and schedule execution.
    #[must_use]
    pub fn new() -> Self {
        Self {
            core: CoreWorld::new(),
            schedules: AppSchedule::new(),
            plugins: PluginRegistry::new(),
            should_exit: false,
            startup_ran: false,
        }
    }

    /// Creates an [`AeApp`] wrapping an initialized [`CoreWorld`].
    #[must_use]
    pub fn from_core(core: CoreWorld) -> Self {
        Self {
            core,
            schedules: AppSchedule::new(),
            plugins: PluginRegistry::new(),
            should_exit: false,
            startup_ran: false,
        }
    }

    /// Registers `plugin` if its name has not already been attached.
    pub fn add_plugin<P: AppPlugin>(&mut self, plugin: &P) -> bool {
        if !self.plugins.mark_registered(plugin.name()) {
            return false;
        }
        plugin.build(self);
        true
    }

    /// Registers a system closure directly onto `phase`.
    pub fn add_system<F>(&mut self, phase: AppPhase, system: F)
    where
        F: FnMut(&mut CoreWorld) + Send + Sync + 'static,
    {
        self.schedules.add_system(phase, system);
    }

    /// Executes [`AppPhase::Startup`] once (subsequent calls are safe no-ops).
    pub fn run_startup_once(&mut self) {
        if !self.startup_ran {
            self.startup_ran = true;
            self.schedules.run_phase(AppPhase::Startup, &mut self.core);
        }
    }

    /// Executes [`AppPhase::Startup`] if needed, followed by every frame phase in order.
    pub fn run_frame(&mut self) {
        self.run_startup_once();
        for phase in AppPhase::FRAME_PHASES {
            self.schedules.run_phase(phase, &mut self.core);
        }
    }
}

impl Default for AeApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ae_app_runs_startup_once_and_all_frame_phases_in_order() {
        let mut app = AeApp::new();
        app.core.resources.insert(Vec::<&'static str>::new());

        app.add_system(AppPhase::Startup, |c| {
            c.resources
                .get_mut::<Vec<&'static str>>()
                .expect("log vec")
                .push("startup");
        });
        app.add_system(AppPhase::First, |c| {
            c.resources
                .get_mut::<Vec<&'static str>>()
                .expect("log vec")
                .push("first");
        });
        app.add_system(AppPhase::Update, |c| {
            c.resources
                .get_mut::<Vec<&'static str>>()
                .expect("log vec")
                .push("update");
        });
        app.add_system(AppPhase::Last, |c| {
            c.resources
                .get_mut::<Vec<&'static str>>()
                .expect("log vec")
                .push("last");
        });

        app.run_frame();
        app.run_frame();

        let log = app
            .core
            .resources
            .get::<Vec<&'static str>>()
            .expect("log vec");
        assert_eq!(
            log.as_slice(),
            &[
                "startup", "first", "update", "last", "first", "update", "last"
            ]
        );
    }
}