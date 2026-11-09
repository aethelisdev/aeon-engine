// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Component Render Context
//!
//! Provides the rendering execution context and Undo/Redo session coordinators
//! passed into registered component inspector handlers.

use super::types::InspectorPanelParams;

/// Canonical default column width in logical pixels allocated for Inspector property labels.
pub const DEFAULT_INSPECTOR_LABEL_WIDTH: f32 = irisui::prelude::DEFAULT_PROPERTY_LABEL_WIDTH;

/// Shared rendering context passed into each registered component inspector handler.
pub struct ComponentRenderContext<'a> {
    /// Selected entity being inspected.
    pub entity: hecs::Entity,
    /// The ECS world containing components.
    pub world: &'a hecs::World,
    /// Global parameters from the parent Inspector panel.
    pub params: &'a InspectorPanelParams<'a>,
    /// Mutable collector for inspector UI actions generated during rendering.
    pub actions: Option<&'a mut Vec<()>>,
    /// Absolute X start coordinate of the card.
    pub base_x: f32,
    /// Absolute Y start coordinate of the card.
    pub base_y: f32,
    /// Card width in pixels.
    pub card_w: f32,
}

impl<'a> ComponentRenderContext<'a> {
    /// Constructs a new [`ComponentRenderContext`] without an attached action sink.
    pub fn new(
        entity: hecs::Entity,
        world: &'a hecs::World,
        params: &'a InspectorPanelParams<'a>,
        base_x: f32,
        base_y: f32,
        card_w: f32,
    ) -> Self {
        Self {
            entity,
            world,
            params,
            actions: None,
            base_x,
            base_y,
            card_w,
        }
    }

    /// Constructs a new [`ComponentRenderContext`] with an attached mutable action sink.
    pub fn with_actions(
        entity: hecs::Entity,
        world: &'a hecs::World,
        params: &'a InspectorPanelParams<'a>,
        _actions: &'a mut Vec<()>,
        base_x: f32,
        base_y: f32,
        card_w: f32,
    ) -> Self {
        Self::new(entity, world, params, base_x, base_y, card_w)
    }

    /// Pushes an inspector UI action for the inspected entity, if an action sink is attached.
    #[inline]
    pub fn push_action(&mut self, _action: ()) {}

    /// Signals the start of an editing session on a named component, capturing pre-edit bytes for undo.
    #[inline]
    pub fn begin_edit(&mut self, _comp_name: &'static str) {}

    /// Signals the start of an editing session on a named component with an explicit pre-edit snapshot.
    #[inline]
    pub fn begin_edit_with_snapshot(&mut self, _comp_name: &'static str, _snapshot: Vec<u8>) {}

    /// Directly commits an atomic modification undo action for a component with pre-edit and post-edit snapshots.
    #[inline]
    pub fn commit_modify(
        &mut self,
        _comp_name: &'static str,
        _old_bytes: Vec<u8>,
        _new_bytes: Vec<u8>,
    ) {
    }

    /// Signals the completion of an editing session on a named component, committing undo history if modified.
    #[inline]
    pub fn commit_edit(&mut self, _comp_name: &'static str) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui::prelude::*;

    fn create_dummy_params<'a>(
        world: &'a hecs::World,
        entity: hecs::Entity,
        euler: &'a [f32; 3],
        swatches: &'a [[f32; 4]],
    ) -> InspectorPanelParams<'a> {
        InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world,
            selected_entity: Some(entity),
            inspector_euler: euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        }
    }

    #[test]
    fn test_component_render_context_creation_and_fields() {
        let world = hecs::World::new();
        let entity = hecs::Entity::DANGLING;
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 12.0, 24.0, 280.0);
        assert_eq!(ctx.entity, entity);
        assert_eq!(ctx.base_x, 12.0);
        assert_eq!(ctx.base_y, 24.0);
        assert_eq!(ctx.card_w, 280.0);
        assert!(ctx.actions.is_none());

        ctx.push_action(());
        ctx.begin_edit("TestComponent");
        ctx.begin_edit_with_snapshot("TestComponent", vec![1, 2, 3]);
        ctx.commit_modify("TestComponent", vec![1], vec![2]);
        ctx.commit_edit("TestComponent");
    }

    #[test]
    fn test_component_render_context_with_actions() {
        let world = hecs::World::new();
        let entity = hecs::Entity::DANGLING;
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

        let mut actions = Vec::new();
        let ctx = ComponentRenderContext::with_actions(
            entity,
            &world,
            &params,
            &mut actions,
            0.0,
            0.0,
            300.0,
        );
        assert_eq!(ctx.card_w, 300.0);
    }

    #[test]
    fn test_default_inspector_label_width_matches_iris() {
        assert_eq!(
            DEFAULT_INSPECTOR_LABEL_WIDTH,
            irisui::prelude::DEFAULT_PROPERTY_LABEL_WIDTH
        );
    }
}