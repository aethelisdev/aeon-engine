// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchy & Parenting Inspector Card
//!
//! Provides inspection and management for entity parent-child relationships.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::tags::TAG_INSPECTOR_UNPARENT;
use super::super::types::ComponentCategory;
use super::physics::helpers::{ComponentHeaderProps, build_declarative_card_header};
use irisui::prelude::*;

/// Inspector handler for entity hierarchy and parenting relationships.
pub struct ParentHandler;

impl ComponentInspectorHandler for ParentHandler {
    fn component_name(&self) -> &'static str {
        "Parenting"
    }

    fn display_title(&self) -> &'static str {
        "Parent / Hierarchy"
    }

    fn icon(&self) -> &'static str {
        "🔗"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.98, 0.80, 0.08, 1.0) // Gold / Warm Amber (#facc15)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Hierarchy
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Parent>(entity).is_ok()
            || world.get::<&ae_core::ecs::Children>(entity).is_ok()
    }

    fn can_remove(&self) -> bool {
        false
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let parent_entity = ctx
            .world
            .get::<&ae_core::ecs::Parent>(ctx.entity)
            .ok()
            .map(|p| p.0);

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("ParentingCard", card_style, |card| {
            build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: None,
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                true, // can_remove is false, so disable/hide delete button
            );

            if let Some(parent) = parent_entity {
                let parent_name = ctx
                    .world
                    .get::<&ae_core::ecs::Name>(parent)
                    .map(|n| n.0.clone())
                    .unwrap_or_else(|_| format!("Entity {:?}", parent));

                let row_style = Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::SpaceBetween)
                    .height(22.0);

                card.container_named("ParentRow", row_style, |row| {
                    row.label_styled_passive(
                        "ParentLbl",
                        format!("Parent: {}", parent_name),
                        11.0,
                        Color::rgba(0.886, 0.894, 0.918, 1.0),
                        TextAlign::Left,
                        Style::new().flex_grow(1.0).height(20.0),
                    );

                    let unparent_style = Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center)
                        .width(80.0)
                        .height(20.0)
                        .background(Color::rgba(0.157, 0.165, 0.188, 0.98))
                        .border(1.0, Color::rgba(0.212, 0.220, 0.259, 0.85))
                        .border_radius(4.0);

                    row.container_tagged(
                        "UnparentBtn",
                        unparent_style,
                        WidgetRole::Button,
                        TAG_INSPECTOR_UNPARENT,
                        |btn| {
                            btn.label_styled_passive(
                                "UnparentTxt",
                                "❌ Unparent",
                                10.0,
                                Color::rgba(0.82, 0.84, 0.88, 1.0),
                                TextAlign::Center,
                                Style::new().width(80.0).height(20.0),
                            );
                        },
                    );
                });
            } else {
                card.label_styled_passive(
                    "ParentRootLbl",
                    "Parent: None (Root Entity)",
                    11.0,
                    Color::rgba(0.620, 0.635, 0.678, 1.0),
                    TextAlign::Left,
                    Style::new().height(20.0),
                );
            }
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::Children::default());
    }
}