// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Component Inspector Handler Trait
//!
//! Defines the common interface implemented by all component inspection cards
//! rendered within the Inspector panel.

use super::context::ComponentRenderContext;
use super::types::ComponentCategory;
use irisui::prelude::*;

/// Common trait implemented by all component inspection cards.
pub trait ComponentInspectorHandler: Send + Sync {
    /// Unique component identifier name (e.g. `"Collider"`, `"Light"`, `"RigidBody"`).
    fn component_name(&self) -> &'static str;

    /// Human-readable title displayed in the card header.
    fn display_title(&self) -> &'static str;

    /// Unicode icon prepended to the card title.
    fn icon(&self) -> &'static str;

    /// Optional GPU SDF atlas icon coordinates (`[u_min, v_min, u_max, layer]`).
    ///
    /// When `Some`, the component header and menu items render this hardware-accelerated vector icon quad
    /// instead of a plain unicode/emoji text icon.
    fn atlas_icon(&self) -> Option<[f32; 4]> {
        None
    }

    /// Color accent for the card icon and header title.
    fn header_color(&self) -> Color;

    /// Category under which this component is listed in the Add Component menu.
    fn category(&self) -> ComponentCategory;

    /// Whether this component currently exists on the target entity.
    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool;

    /// Renders the standardized component card header in a declarative [`UiScope`].
    ///
    /// Automatically populates atlas icon, unicode fallback icon, title label,
    /// header color accent, and delete action button from trait metadata.
    fn render_header(&self, scope: &mut UiScope<'_>) {
        scope.card_header_desc(
            CardHeaderDescriptor::new(self.display_title())
                .icon(self.icon())
                .maybe_atlas_icon(self.atlas_icon())
                .color(self.header_color())
                .action("×", UiScope::action_tag(self.component_name())),
        );
    }

    /// Renders the component card into the declarative [`UiScope`].
    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>);

    /// Instantiates and attaches this default component to the target entity in the ECS world.
    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity);
}