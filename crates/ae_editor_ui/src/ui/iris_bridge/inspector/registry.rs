// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Extensible Component Inspector Registry
//!
//! Provides the centralized, type-safe registry storing and querying all registered
//! component inspector handlers in deterministic order.

pub use super::context::{
    ComponentRenderContext, PropComboMapConfig, PropComboRowLayout, PropF32Config, PropVec2Config,
    PropVec3Config,
};
pub use super::handler::ComponentInspectorHandler;
use super::types::ComponentCategory;
use std::sync::OnceLock;

/// Global registry storing all registered component inspector handlers in deterministic order.
pub struct InspectorRegistry {
    handlers: Vec<Box<dyn ComponentInspectorHandler>>,
}

static GLOBAL_REGISTRY: OnceLock<InspectorRegistry> = OnceLock::new();

impl InspectorRegistry {
    /// Initializes and returns a reference to the global `InspectorRegistry` singleton.
    #[must_use]
    pub fn global() -> &'static Self {
        GLOBAL_REGISTRY.get_or_init(Self::new)
    }

    /// Creates and populates the registry with all standard built-in component handlers.
    #[must_use]
    fn new() -> Self {
        let mut registry = Self {
            handlers: Vec::with_capacity(32),
        };
        super::components::register_all_components(&mut registry);
        registry
    }

    /// Registers a new custom component inspector handler.
    pub fn register<H: ComponentInspectorHandler + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }

    /// Returns an iterator over all registered component handlers.
    pub fn handlers(&self) -> &[Box<dyn ComponentInspectorHandler>] {
        &self.handlers
    }

    /// Finds a registered handler by component identifier name.
    pub fn find_by_name(&self, name: &str) -> Option<&dyn ComponentInspectorHandler> {
        self.handlers
            .iter()
            .find(|h| h.component_name() == name)
            .map(|h| &**h)
    }

    /// Returns all registered handlers belonging to a specific `ComponentCategory`.
    pub fn find_by_category(&self, cat: ComponentCategory) -> Vec<&dyn ComponentInspectorHandler> {
        self.handlers
            .iter()
            .filter(|h| h.category() == cat)
            .map(|h| &**h)
            .collect()
    }
}