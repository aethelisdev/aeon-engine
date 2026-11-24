// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor UI transform hierarchy action adapters delegating math and ECS reparenting to `ae_scene`.

use super::UiContext;

/// Handles parenting an entity while preserving its exact world-space position and scale ("Keep World Transform").
pub fn handle_parent_entity(ctx: &mut UiContext, child: ae_ecs::Entity, parent: ae_ecs::Entity) {
    let _ = ae_scene::parent_entity_keep_world_transform(ctx.world, child, parent);
}

/// Handles unparenting an entity while preserving its exact world-space position and scale ("Keep World Transform").
pub fn handle_unparent_entity(ctx: &mut UiContext, child: ae_ecs::Entity) {
    let _ = ae_scene::unparent_entity_keep_world_transform(ctx.world, child);
}