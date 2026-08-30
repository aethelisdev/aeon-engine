// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Extensible Component Inspector Registry & Handler Trait
//!
//! Replaces monolithic `if-else` cascades with an extensible, type-safe,
//! plugin-friendly component inspection and editing registry.

use super::types::{ComponentCategory, InspectorPanelParams, InspectorPanelTargets};
use irisui::prelude::*;
use std::sync::OnceLock;

/// Shared rendering context passed into each registered component inspector handler.
pub struct ComponentRenderContext<'a> {
    /// Selected entity being inspected.
    pub entity: hecs::Entity,
    /// The ECS world containing components.
    pub world: &'a hecs::World,
    /// Global parameters from the parent Inspector panel.
    pub params: &'a InspectorPanelParams<'a>,
    /// Global hit-test target buffers to record interactive widgets.
    pub targets: &'a mut InspectorPanelTargets,
    /// Absolute X start coordinate of the card.
    pub base_x: f32,
    /// Absolute Y start coordinate of the card.
    pub base_y: f32,
    /// Card width in pixels.
    pub card_w: f32,
}

/// Common trait implemented by all component inspection cards.
pub trait ComponentInspectorHandler: Send + Sync {
    /// Unique component identifier name (e.g. `"Collider"`, `"Light"`, `"KinematicCharacterController"`).
    fn component_name(&self) -> &'static str;

    /// Human-readable title displayed in the card header.
    fn display_title(&self) -> &'static str;

    /// Unicode icon prepended to the card title.
    fn icon(&self) -> &'static str;

    /// Color accent for the card icon and header title.
    fn header_color(&self) -> Color;

    /// Category under which this component is listed in the Add Component menu.
    fn category(&self) -> ComponentCategory;

    /// Whether this component currently exists on the target entity.
    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool;

    /// Whether this component can be removed via the trash `🗑` button.
    fn can_remove(&self) -> bool {
        true
    }

    /// Renders the component card into the `UiTree` and returns the total computed card height.
    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32;

    /// Instantiates and attaches this default component to the target entity in the ECS world.
    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity);
}

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