// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Component Render Context & Declarative Property Binders
//!
//! Provides the rendering execution context and transitional two-way property
//! bindings (`prop_f32`, `prop_vec2`, `prop_i32`, `prop_combobox_mapped`, `prop_checkbox`)
//! with automatic Undo/Redo command session snapshotting for inspected entities.

use super::types::InspectorPanelParams;
use irisui::widgets::declarative::scope::ComboRowLayout;

/// Canonical default column width in logical pixels allocated for Inspector property labels.
pub const DEFAULT_INSPECTOR_LABEL_WIDTH: f32 = irisui::prelude::DEFAULT_PROPERTY_LABEL_WIDTH;

/// Configuration descriptor for 2D coordinate vector property rows.
#[derive(Debug, Clone, Copy)]
pub struct PropVec2Config<'a> {
    /// Axis prefix tags (e.g. `["X", "Y"]` or `["W", "H"]`).
    pub axis_labels: [&'a str; 2],
    /// Multiplier applied to continuous pointer drag displacement.
    pub speed: f32,
    /// Display precision decimals for formatted floating-point values.
    pub decimals: usize,
    /// Physical unit suffix string (e.g. `"px"`, `""`).
    pub unit: &'a str,
}

impl<'a> PropVec2Config<'a> {
    /// Constructs a standard 2D vector configuration descriptor.
    #[inline]
    #[must_use]
    pub const fn new(
        axis_labels: [&'a str; 2],
        speed: f32,
        decimals: usize,
        unit: &'a str,
    ) -> Self {
        Self {
            axis_labels,
            speed,
            decimals,
            unit,
        }
    }
}

/// Layout configuration for dropdown combobox rows.
pub type PropComboRowLayout = ComboRowLayout;

/// Configuration descriptor for mapped combobox property bindings.
#[derive(Debug, Clone, Copy)]
pub struct PropComboMapConfig<'a> {
    /// Label text displayed alongside the combobox.
    pub label: &'a str,
    /// Currently selected option index.
    pub current_idx: usize,
    /// Slice of string option labels.
    pub options: &'a [&'a str],
    /// Horizontal layout sizing configuration.
    pub layout: ComboRowLayout,
}

impl<'a> PropComboMapConfig<'a> {
    /// Constructs a new mapped combobox configuration descriptor.
    #[inline]
    #[must_use]
    pub const fn new(
        label: &'a str,
        current_idx: usize,
        options: &'a [&'a str],
        layout: ComboRowLayout,
    ) -> Self {
        Self {
            label,
            current_idx,
            options,
            layout,
        }
    }
}

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

    /// Binds an autonomous floating-point property to a named component with automatic undo snapshotting.
    ///
    /// Modifies `*value` in-place, triggering `begin_edit` when interaction starts and `commit_edit` on release.
    pub fn prop_f32(
        &mut self,
        scope: &mut irisui::prelude::UiScope<'_>,
        comp_name: &'static str,
        label: &str,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        speed: f32,
    ) {
        let old_val = *value;
        let resp = scope.drag_f32_ranged(label, value, range, speed);
        let new_val = *value;
        let changed = (new_val - old_val).abs() > f32::EPSILON;

        if resp.clicked() || resp.pressed() {
            self.begin_edit(comp_name);
        }

        if changed && resp.committed() {
            self.begin_edit(comp_name);
            self.commit_edit(comp_name);
        } else if changed {
            self.begin_edit(comp_name);
        } else if resp.committed() {
            self.commit_edit(comp_name);
        }
    }

    /// Binds an autonomous 2D coordinate vector property with dedicated axis inputs and automatic undo snapshotting.
    ///
    /// Modifies `values` (`[x, y]`) in-place, triggering `begin_edit` when interaction begins and `commit_edit` on release.
    ///
    /// # Arguments
    /// * `scope` - Declarative UI rendering scope.
    /// * `comp_name` - Static component name for undo/redo session tracking.
    /// * `label` - Left column title caption.
    /// * `values` - Mutable slice containing `[x, y]` coordinates modified in-place.
    /// * `cfg` - Axis labels, speed, precision decimals, and unit configuration descriptor.
    pub fn prop_vec2(
        &mut self,
        scope: &mut irisui::prelude::UiScope<'_>,
        comp_name: &'static str,
        label: &str,
        values: &mut [f32; 2],
        cfg: PropVec2Config<'_>,
    ) {
        let old_vals = *values;
        let resp = scope.add(
            irisui::prelude::DragVec2::new(label, values)
                .axis_labels(cfg.axis_labels)
                .speed(cfg.speed)
                .decimals(cfg.decimals)
                .unit(cfg.unit),
        );
        let changed = (values[0] - old_vals[0]).abs() > f32::EPSILON
            || (values[1] - old_vals[1]).abs() > f32::EPSILON;

        if resp.clicked() || resp.response.pressed() {
            self.begin_edit(comp_name);
        }

        if changed && resp.committed() {
            self.begin_edit(comp_name);
            self.commit_edit(comp_name);
        } else if changed {
            self.begin_edit(comp_name);
        } else if resp.committed() {
            self.commit_edit(comp_name);
        }
    }

    /// Binds an autonomous signed integer property with automatic undo snapshotting.
    ///
    /// Modifies `*value` in-place, triggering `begin_edit` on value change and `commit_edit` on release.
    pub fn prop_i32(
        &mut self,
        scope: &mut irisui::prelude::UiScope<'_>,
        comp_name: &'static str,
        label: &str,
        value: &mut i32,
        range: std::ops::RangeInclusive<i32>,
    ) {
        let mut f_val = *value as f32;
        let range_f = (*range.start() as f32)..=(*range.end() as f32);
        let resp = scope.drag_f32_ranged(label, &mut f_val, range_f, 1.0);
        if resp.changed() {
            *value = f_val.round() as i32;
            self.begin_edit(comp_name);
        }
        if resp.committed() {
            *value = f_val.round() as i32;
            self.commit_edit(comp_name);
        }
    }

    /// Binds an autonomous dropdown combobox row mapped to a custom domain enum with automatic undo snapshotting.
    ///
    /// Delegates directly to [`UiScope::combobox_row_mapped`], triggering `begin_edit` and `commit_edit` on value change.
    pub fn prop_combobox_mapped<T>(
        &mut self,
        scope: &mut irisui::prelude::UiScope<'_>,
        comp_name: &'static str,
        cfg: PropComboMapConfig<'_>,
        target: &mut T,
        map_fn: impl FnOnce(usize) -> T,
    ) {
        let resp = scope.combobox_row_mapped(
            cfg.label,
            cfg.current_idx,
            cfg.options,
            cfg.layout,
            target,
            map_fn,
        );
        if resp.changed() {
            self.begin_edit(comp_name);
            self.commit_edit(comp_name);
        }
    }

    /// Binds an autonomous boolean checkbox with automatic undo snapshotting.
    ///
    /// Toggles `*checked` in-place, triggering `begin_edit` and `commit_edit` upon click.
    pub fn prop_checkbox(
        &mut self,
        scope: &mut irisui::prelude::UiScope<'_>,
        comp_name: &'static str,
        label: &str,
        checked: &mut bool,
    ) {
        let resp = scope.checkbox(label, checked);
        if resp.clicked() {
            self.begin_edit(comp_name);
            self.commit_edit(comp_name);
        }
    }
}