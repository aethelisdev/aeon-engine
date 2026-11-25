// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Execution phase ordering and deterministic system schedule storage.

use crate::core_world::CoreWorld;

/// Ordered execution stages within an Aeon application frame lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppPhase {
    /// One-time initialization before the main frame loop starts.
    Startup = 0,
    /// Beginning of frame housekeeping before fixed-step simulation.
    First = 1,
    /// Deterministic fixed-timestep physics and gameplay accumulation step.
    FixedUpdate = 2,
    /// Per-frame pre-update setup after fixed physics ticks.
    PreUpdate = 3,
    /// Primary variable-timestep gameplay, animation, and behavior step.
    Update = 4,
    /// Transform propagation, spatial grid, and post-gameplay housekeeping.
    PostUpdate = 5,
    /// Extracting ECS/camera data into GPU render snapshots.
    Extract = 6,
    /// Recording and submitting GPU render commands.
    Render = 7,
    /// End-of-frame input cleanup and telemetry finalization.
    Last = 8,
}

impl AppPhase {
    /// Total number of ordered execution phases in [`AppSchedule`].
    pub const COUNT: usize = 9;

    /// Canonical execution order executed each frame after [`AppPhase::Startup`].
    pub const FRAME_PHASES: [Self; 8] = [
        Self::First,
        Self::FixedUpdate,
        Self::PreUpdate,
        Self::Update,
        Self::PostUpdate,
        Self::Extract,
        Self::Render,
        Self::Last,
    ];

    /// Numeric table index corresponding to this phase.
    #[inline]
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Boxed deterministic system callback operating over [`CoreWorld`].
pub type AppSystemFn = Box<dyn FnMut(&mut CoreWorld) + Send + Sync>;

/// Registered system collections partitioned by [`AppPhase`].
#[derive(Default)]
pub struct AppSchedule {
    phases: [Vec<AppSystemFn>; AppPhase::COUNT],
}

impl AppSchedule {
    /// Creates an empty schedule with zero registered systems.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a callback into `phase`, preserving insertion order.
    pub fn add_system<F>(&mut self, phase: AppPhase, system: F)
    where
        F: FnMut(&mut CoreWorld) + Send + Sync + 'static,
    {
        self.phases[phase.index()].push(Box::new(system));
    }

    /// Returns the number of registered systems in `phase`.
    #[must_use]
    pub fn system_count(&self, phase: AppPhase) -> usize {
        self.phases[phase.index()].len()
    }

    /// Runs every system registered for `phase` in registration order.
    pub fn run_phase(&mut self, phase: AppPhase, core: &mut CoreWorld) {
        for system in &mut self.phases[phase.index()] {
            system(core);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_plugin_api::Name;

    #[test]
    fn test_app_schedule_executes_phases_in_registration_order() {
        let mut schedule = AppSchedule::new();
        let mut core = CoreWorld::new();

        schedule.add_system(AppPhase::Startup, |w| {
            w.world.spawn((Name("Boot".to_string()),));
        });
        schedule.add_system(AppPhase::Update, |w| {
            w.resources.insert(42u32);
        });
        schedule.add_system(AppPhase::Update, |w| {
            if let Some(val) = w.resources.get_mut::<u32>() {
                *val += 8;
            }
        });

        assert_eq!(schedule.system_count(AppPhase::Startup), 1);
        assert_eq!(schedule.system_count(AppPhase::Update), 2);
        assert_eq!(schedule.system_count(AppPhase::Render), 0);

        schedule.run_phase(AppPhase::Startup, &mut core);
        assert_eq!(core.world.len(), 1);

        schedule.run_phase(AppPhase::Update, &mut core);
        assert_eq!(core.resources.get::<u32>().copied(), Some(50));

        // Running an empty phase is a safe no-op
        schedule.run_phase(AppPhase::Render, &mut core);
    }
}