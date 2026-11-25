// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Composable plugin group abstractions enabling selective enable/disable of subsystem plugins.

use crate::app::AeApp;
use crate::plugin::AppPlugin;
use std::any::TypeId;

type BoxedPlugin = Box<dyn AppPlugin>;

/// Collection of related [`AppPlugin`] modules that can be customized before building into [`AeApp`].
pub trait PluginGroup: Sized {
    /// Human-readable group name.
    fn name(&self) -> &'static str;

    /// Populates a [`PluginGroupBuilder`] with this group's default ordered plugins.
    fn build(self) -> PluginGroupBuilder;
}

struct PluginGroupEntry {
    type_id: TypeId,
    plugin: BoxedPlugin,
    enabled: bool,
}

/// Ordered builder allowing individual plugins in a [`PluginGroup`] to be toggled off before attachment.
pub struct PluginGroupBuilder {
    entries: Vec<PluginGroupEntry>,
}

impl Default for PluginGroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginGroupBuilder {
    /// Creates an empty [`PluginGroupBuilder`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Appends or replaces a plugin entry of concrete type `P` in registration order.
    #[must_use]
    pub fn with_plugin<P: AppPlugin + 'static>(mut self, plugin: P) -> Self {
        let type_id = TypeId::of::<P>();
        if let Some(existing) = self.entries.iter_mut().find(|e| e.type_id == type_id) {
            existing.plugin = Box::new(plugin);
            existing.enabled = true;
        } else {
            self.entries.push(PluginGroupEntry {
                type_id,
                plugin: Box::new(plugin),
                enabled: true,
            });
        }
        self
    }

    /// Disables plugin `P` so it will not be built or registered when applied to [`AeApp`].
    #[must_use]
    pub fn disable<P: AppPlugin + 'static>(mut self) -> Self {
        let type_id = TypeId::of::<P>();
        if let Some(entry) = self.entries.iter_mut().find(|e| e.type_id == type_id) {
            entry.enabled = false;
        }
        self
    }

    /// Returns `true` if plugin `P` is present and currently enabled in this builder.
    #[must_use]
    pub fn is_enabled<P: AppPlugin + 'static>(&self) -> bool {
        let type_id = TypeId::of::<P>();
        self.entries
            .iter()
            .any(|e| e.type_id == type_id && e.enabled)
    }

    /// Registers all enabled plugins in insertion order onto `app`.
    pub fn finish(self, app: &mut AeApp) {
        for entry in self.entries {
            if entry.enabled && app.plugins.mark_registered(entry.plugin.name()) {
                entry.plugin.build(app);
            }
        }
    }
}

impl PluginGroup for PluginGroupBuilder {
    fn name(&self) -> &'static str {
        "PluginGroupBuilder"
    }

    fn build(self) -> PluginGroupBuilder {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlphaPlugin;
    impl AppPlugin for AlphaPlugin {
        fn name(&self) -> &'static str {
            "alpha"
        }

        fn build(&self, app: &mut AeApp) {
            app.core.resources.insert(10u32);
        }
    }

    struct BetaPlugin;
    impl AppPlugin for BetaPlugin {
        fn name(&self) -> &'static str {
            "beta"
        }

        fn build(&self, app: &mut AeApp) {
            app.core.resources.insert("beta_active".to_string());
        }
    }

    struct TwoPluginGroup;
    impl PluginGroup for TwoPluginGroup {
        fn name(&self) -> &'static str {
            "two_plugins"
        }

        fn build(self) -> PluginGroupBuilder {
            PluginGroupBuilder::new()
                .with_plugin(AlphaPlugin)
                .with_plugin(BetaPlugin)
        }
    }

    #[test]
    fn test_plugin_group_supports_selective_disable_and_attachment() {
        let builder = TwoPluginGroup.build().disable::<BetaPlugin>();
        assert!(builder.is_enabled::<AlphaPlugin>());
        assert!(!builder.is_enabled::<BetaPlugin>());

        let mut app = AeApp::new();
        app.add_plugins(builder);

        assert!(app.plugins.contains("alpha"));
        assert!(!app.plugins.contains("beta"));
        assert_eq!(app.core.resources.get::<u32>().copied(), Some(10));
        assert!(app.core.resources.get::<String>().is_none());
    }
}