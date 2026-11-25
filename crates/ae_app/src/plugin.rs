// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular application plugin contract and registration registry.

use crate::app::AeApp;
use std::collections::HashSet;

/// Modular unit that wires resources, subsystems, and scheduled systems into [`AeApp`].
pub trait AppPlugin: Send + Sync {
    /// Unique static identifier used to prevent duplicate registration.
    fn name(&self) -> &'static str;

    /// Configures `app` resources and schedules.
    fn build(&self, app: &mut AeApp);
}

/// Collection of distinct [`AppPlugin`] modules attached to an application instance.
#[derive(Default)]
pub struct PluginRegistry {
    registered_names: HashSet<&'static str>,
    ordered_names: Vec<&'static str>,
}

impl PluginRegistry {
    /// Creates an empty plugin registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks `name` as registered and returns `false` if it was already registered.
    pub fn mark_registered(&mut self, name: &'static str) -> bool {
        if !self.registered_names.insert(name) {
            return false;
        }
        self.ordered_names.push(name);
        true
    }

    /// Returns `true` if `name` has already been attached.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.registered_names.contains(name)
    }

    /// Returns registered plugin names in attachment order.
    #[must_use]
    pub fn names(&self) -> &[&'static str] {
        &self.ordered_names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SamplePlugin;
    impl AppPlugin for SamplePlugin {
        fn name(&self) -> &'static str {
            "sample_plugin"
        }

        fn build(&self, app: &mut AeApp) {
            app.core.resources.insert(99i32);
        }
    }

    #[test]
    fn test_plugin_registration_prevents_duplicates() {
        let mut app = AeApp::new();
        assert!(app.add_plugin(&SamplePlugin));
        assert!(!app.add_plugin(&SamplePlugin));
        assert!(app.plugins.contains("sample_plugin"));
        assert_eq!(app.plugins.names(), &["sample_plugin"]);
        assert_eq!(app.core.resources.get::<i32>().copied(), Some(99));
    }
}