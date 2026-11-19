// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Periodic Panel Live-Refresh Scheduler
//!
//! Coordinates per-panel [`LiveRefreshInterval`] timers and scratch iteration state
//! for [`Workbench`](crate::workbench::Workbench) during `InPlaceRefresh`, automatically
//! clearing stale timers whenever the dock shell or individual leaf subtrees rebuild.

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

use irisui_dock::chrome::DockChromeFrame;
use irisui_widgets::UiScope;

use crate::descriptors::panel::LiveRefreshInterval;
use crate::descriptors::panel_catalog::PanelRegistryTarget;

/// Tracks per-panel [`Instant`] timers and a zero-allocation scratch buffer for live ticks.
pub(super) struct LivePanelScheduler<TabId: Eq + Hash + Copy> {
    timers: HashMap<TabId, Instant>,
    scratch: Vec<(TabId, LiveRefreshInterval)>,
}

impl<TabId: Eq + Hash + Copy> Default for LivePanelScheduler<TabId> {
    fn default() -> Self {
        Self::new()
    }
}

impl<TabId: Eq + Hash + Copy> LivePanelScheduler<TabId> {
    /// Creates an empty live-refresh scheduler.
    #[inline]
    #[must_use]
    pub(super) fn new() -> Self {
        Self {
            timers: HashMap::new(),
            scratch: Vec::new(),
        }
    }

    /// Clears all panel timers after a full shell rebuild.
    #[inline]
    pub(super) fn clear_all(&mut self) {
        self.timers.clear();
    }

    /// Clears timers for specific panels whose subtrees were rebuilt granularly.
    #[inline]
    pub(super) fn clear_tabs(&mut self, tabs: &[TabId]) {
        for tab in tabs {
            self.timers.remove(tab);
        }
    }

    /// Returns `true` if any visible docked/floating panel declares periodic live refresh behavior.
    pub(super) fn has_active_live_panel<Ctx, R>(
        &self,
        dock_frame: Option<&DockChromeFrame<TabId>>,
        registry: &R,
    ) -> bool
    where
        R: PanelRegistryTarget<TabId, Ctx> + ?Sized,
    {
        let Some(df) = dock_frame else {
            return false;
        };
        registry
            .catalog()
            .live_panels()
            .iter()
            .any(|(tab, _)| df.leaf_container(*tab).is_some())
    }

    /// Dispatches `refresh_live` for every active registered panel whose [`LiveRefreshInterval`] is due.
    pub(super) fn dispatch<Ctx, R>(
        &mut self,
        scope: &mut UiScope<'_>,
        dock_frame: Option<&DockChromeFrame<TabId>>,
        registry: &mut R,
        context: &mut Ctx,
    ) where
        R: PanelRegistryTarget<TabId, Ctx> + ?Sized,
    {
        self.scratch.clear();
        self.scratch
            .extend_from_slice(registry.catalog().live_panels());
        let now = Instant::now();
        for idx in 0..self.scratch.len() {
            let (tab, interval) = self.scratch[idx];
            if let Some(df) = dock_frame
                && df.leaf_container(tab).is_none()
            {
                continue;
            }
            match interval {
                LiveRefreshInterval::OnChange => {}
                LiveRefreshInterval::EveryFrame => {
                    registry.refresh_live(&tab, scope, context);
                }
                LiveRefreshInterval::ThrottledMs(ms) => {
                    let due = self.timers.get(&tab).is_none_or(|prev| {
                        now.saturating_duration_since(*prev) >= Duration::from_millis(u64::from(ms))
                    });
                    if due {
                        self.timers.insert(tab, now);
                        registry.refresh_live(&tab, scope, context);
                    }
                }
            }
        }
    }
}