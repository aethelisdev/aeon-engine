// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Component Render Context & Declarative Property Binders
//!
//! Provides the rendering execution context and high-level two-way property
//! bindings (`prop_f32`, `prop_combobox_mapped`, `prop_checkbox`) with automatic
//! Undo/Redo command session snapshotting for inspected entities.

use super::types::{InspectorAction, InspectorPanelParams};
use irisui::widgets::declarative::scope::ComboRowLayout;

/// Canonical default column width in logical pixels allocated for Inspector property labels.
pub const DEFAULT_INSPECTOR_LABEL_WIDTH: f32 = irisui::prelude::DEFAULT_PROPERTY_LABEL_WIDTH;

/// Layout and boundary configuration for floating-point scalar properties.
#[derive(Debug, Clone)]
pub struct PropF32Config {
    /// Inclusive bounds constraint for valid scalar values.
    pub range: std::ops::RangeInclusive<f32>,
    /// Drag sensitivity multiplier applied per pointer displacement pixel.
    pub speed: f32,
    /// Column width in pixels for the left-aligned label.
    pub label_width: f32,
    /// Fixed width in pixels for the numeric pill box.
    pub box_width: f32,
}

impl PropF32Config {
    /// Constructs a standard property configuration with explicit label and box widths.
    #[inline]
    #[must_use]
    pub const fn new(
        range: std::ops::RangeInclusive<f32>,
        speed: f32,
        label_width: f32,
        box_width: f32,
    ) -> Self {
        Self {
            range,
            speed,
            label_width,
            box_width,
        }
    }

    /// Constructs a standard property configuration with default 105px label width and 48px box width.
    #[inline]
    #[must_use]
    pub const fn standard(range: std::ops::RangeInclusive<f32>, speed: f32) -> Self {
        Self {
            range,
            speed,
            label_width: DEFAULT_INSPECTOR_LABEL_WIDTH,
            box_width: 48.0,
        }
    }

    /// Constructs a property configuration with a custom label width and standard 48px box width.
    #[inline]
    #[must_use]
    pub const fn labeled(
        range: std::ops::RangeInclusive<f32>,
        speed: f32,
        label_width: f32,
    ) -> Self {
        Self {
            range,
            speed,
            label_width,
            box_width: 48.0,
        }
    }
}

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

/// Configuration descriptor for 3D coordinate vector property rows.
#[derive(Debug, Clone, Copy)]
pub struct PropVec3Config {
    /// Multiplier applied to continuous pointer drag displacement.
    pub speed: f32,
    /// Display precision decimals for formatted floating-point values.
    pub decimals: usize,
    /// Optional canonical reset vector restored when reset button `🔄` is triggered.
    pub reset_val: Option<[f32; 3]>,
    /// Response curve for pointer drag scrubbing.
    pub curve: irisui::prelude::DragCurve,
}

impl PropVec3Config {
    /// Constructs a standard 3D vector configuration descriptor.
    #[inline]
    #[must_use]
    pub const fn new(
        speed: f32,
        decimals: usize,
        reset_val: Option<[f32; 3]>,
        curve: irisui::prelude::DragCurve,
    ) -> Self {
        Self {
            speed,
            decimals,
            reset_val,
            curve,
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
    pub actions: Option<&'a mut Vec<InspectorAction>>,
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
        actions: &'a mut Vec<InspectorAction>,
        base_x: f32,
        base_y: f32,
        card_w: f32,
    ) -> Self {
        Self {
            entity,
            world,
            params,
            actions: Some(actions),
            base_x,
            base_y,
            card_w,
        }
    }

    /// Pushes an inspector UI action for the inspected entity, if an action sink is attached.
    #[inline]
    pub fn push_action(&mut self, action: InspectorAction) {
        if let Some(actions) = &mut self.actions {
            actions.push(action);
        }
    }

    /// Signals the start of an editing session on a named component, capturing pre-edit bytes for undo.
    #[inline]
    pub fn begin_edit(&mut self, comp_name: &'static str) {
        if let Some(actions) = &mut self.actions {
            actions.push(InspectorAction::StartComponentEdit(
                self.entity,
                comp_name,
                None,
            ));
        }
    }

    /// Signals the start of an editing session on a named component with an explicit pre-edit snapshot.
    #[inline]
    pub fn begin_edit_with_snapshot(&mut self, comp_name: &'static str, snapshot: Vec<u8>) {
        if let Some(actions) = &mut self.actions {
            actions.push(InspectorAction::StartComponentEdit(
                self.entity,
                comp_name,
                Some(snapshot),
            ));
        }
    }

    /// Directly commits an atomic modification undo action for a component with pre-edit and post-edit snapshots.
    #[inline]
    pub fn commit_modify(
        &mut self,
        comp_name: &'static str,
        old_bytes: Vec<u8>,
        new_bytes: Vec<u8>,
    ) {
        if let Some(actions) = &mut self.actions {
            actions.push(InspectorAction::CommitComponentModify(
                self.entity,
                comp_name,
                old_bytes,
                new_bytes,
            ));
        }
    }

    /// Signals the completion of an editing session on a named component, committing undo history if modified.
    #[inline]
    pub fn commit_edit(&mut self, comp_name: &'static str) {
        if let Some(actions) = &mut self.actions {
            actions.push(InspectorAction::CommitComponentEdit(self.entity, comp_name));
        }
    }

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
        self.prop_f32_styled(
            scope,
            comp_name,
            label,
            value,
            PropF32Config::standard(range, speed),
        );
    }

    /// Binds an autonomous floating-point property with explicit label and box widths.
    ///
    /// Modifies `*value` in-place, triggering `begin_edit` when interaction starts and `commit_edit` on release.
    pub fn prop_f32_styled(
        &mut self,
        scope: &mut irisui::prelude::UiScope<'_>,
        comp_name: &'static str,
        label: &str,
        value: &mut f32,
        cfg: PropF32Config,
    ) {
        let old_val = *value;
        let resp = scope.drag_f32_styled(
            label,
            value,
            cfg.range,
            cfg.speed,
            cfg.label_width,
            cfg.box_width,
        );
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

    /// Binds an autonomous 3D coordinate vector property with dedicated axis inputs and automatic undo snapshotting.
    ///
    /// Modifies `values` (`[x, y, z]`) in-place, triggering `begin_edit` when interaction begins and `commit_edit` on release.
    /// Supports optional canonical reset button and response curves.
    pub fn prop_vec3(
        &mut self,
        scope: &mut irisui::prelude::UiScope<'_>,
        comp_name: &'static str,
        label: &str,
        values: &mut [f32; 3],
        cfg: PropVec3Config,
    ) {
        let old_vals = *values;
        let mut builder = irisui::prelude::DragVec3::new(label, values)
            .speed(cfg.speed)
            .decimals(cfg.decimals)
            .curve(cfg.curve);
        if let Some(reset_val) = cfg.reset_val {
            builder = builder.reset(reset_val);
        }
        let resp = scope.add(builder);
        let changed = (values[0] - old_vals[0]).abs() > f32::EPSILON
            || (values[1] - old_vals[1]).abs() > f32::EPSILON
            || (values[2] - old_vals[2]).abs() > f32::EPSILON;

        if resp.clicked() || resp.response.pressed() {
            self.begin_edit(comp_name);
        }

        if resp.reset_clicked() || (changed && resp.committed()) {
            self.begin_edit(comp_name);
            self.commit_edit(comp_name);
        } else if changed {
            self.begin_edit(comp_name);
        } else if resp.committed() {
            self.commit_edit(comp_name);
        }
    }

    /// Binds the 3D linear velocity vector with dedicated X, Y, Z axis scrubbers and automatic undo snapshotting.
    ///
    /// Modifies `velocity` in-place, recording atomic undo/redo operations when confirmed via Enter, blur, or drag release.
    pub fn prop_velocity(
        &mut self,
        scope: &mut irisui::prelude::UiScope<'_>,
        velocity: &mut ae_core::ecs::Velocity,
    ) {
        let mut vel = [velocity.x, velocity.y, velocity.z];
        let old_vel = vel;
        let resp = scope.add(
            irisui::prelude::DragVec3::new("Linear", &mut vel)
                .speed(0.05)
                .decimals(2)
                .curve(irisui::prelude::DragCurve::quadratic())
                .reset([0.0, 0.0, 0.0]),
        );
        let changed = (vel[0] - old_vel[0]).abs() > f32::EPSILON
            || (vel[1] - old_vel[1]).abs() > f32::EPSILON
            || (vel[2] - old_vel[2]).abs() > f32::EPSILON;

        let old_bytes = serde_json::to_vec(&ae_core::ecs::Velocity {
            x: old_vel[0],
            y: old_vel[1],
            z: old_vel[2],
        })
        .unwrap_or_default();

        if resp.clicked() || resp.response.pressed() {
            self.begin_edit_with_snapshot("Velocity", old_bytes.clone());
        }

        if resp.reset_clicked() {
            *velocity = ae_core::ecs::Velocity::zero();
            let new_bytes = serde_json::to_vec(velocity).unwrap_or_default();
            self.commit_modify("Velocity", old_bytes, new_bytes);
        } else if changed {
            velocity.x = vel[0];
            velocity.y = vel[1];
            velocity.z = vel[2];
            if resp.committed() {
                let new_bytes = serde_json::to_vec(velocity).unwrap_or_default();
                self.commit_modify("Velocity", old_bytes, new_bytes);
            } else {
                self.begin_edit_with_snapshot("Velocity", old_bytes);
            }
        } else if resp.committed() {
            self.commit_edit("Velocity");
        }
    }

    /// Binds the complete RigidBody physics parameters (body type, mass, gravity scale)
    /// with declarative dropdown, numeric drag scrubbers, and atomic undo snapshotting.
    pub fn prop_rigidbody(
        &mut self,
        scope: &mut irisui::prelude::UiScope<'_>,
        rb: &mut ae_core::ecs::RigidBody,
    ) {
        let old_rb = *rb;
        let mut body_type_idx = match rb.body_type {
            ae_core::ecs::RigidBodyType::Dynamic => 0,
            ae_core::ecs::RigidBodyType::Kinematic => 1,
            ae_core::ecs::RigidBodyType::Static => 2,
        };

        let type_resp = scope.combobox_styled(
            None,
            &mut body_type_idx,
            &["Dynamic", "Kinematic", "Static"],
            96.0,
        );
        rb.body_type = match body_type_idx {
            0 => ae_core::ecs::RigidBodyType::Dynamic,
            1 => ae_core::ecs::RigidBodyType::Kinematic,
            _ => ae_core::ecs::RigidBodyType::Static,
        };

        let mass_resp = scope.drag_f32_styled(
            "Mass:",
            &mut rb.mass,
            0.001..=100_000.0,
            0.1,
            irisui::prelude::DEFAULT_PROPERTY_LABEL_WIDTH,
            48.0,
        );

        let grav_resp = scope.drag_f32_styled(
            "Gravity:",
            &mut rb.gravity_scale,
            -100.0..=100.0,
            0.1,
            irisui::prelude::DEFAULT_PROPERTY_LABEL_WIDTH,
            48.0,
        );

        let changed = *rb != old_rb;
        let committed = type_resp.changed() || mass_resp.committed() || grav_resp.committed();
        let pressed = mass_resp.pressed() || grav_resp.pressed();

        let old_bytes = serde_json::to_vec(&old_rb).unwrap_or_default();

        if pressed {
            self.begin_edit_with_snapshot("RigidBody", old_bytes.clone());
        }

        if changed {
            if committed {
                let new_bytes = serde_json::to_vec(rb).unwrap_or_default();
                self.commit_modify("RigidBody", old_bytes, new_bytes);
            } else {
                self.begin_edit_with_snapshot("RigidBody", old_bytes);
            }
        } else if committed {
            self.commit_edit("RigidBody");
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

    /// Binds an autonomous dropdown combobox with automatic undo snapshotting.
    ///
    /// Modifies `*selected` index in-place, triggering `begin_edit` and `commit_edit` automatically upon option selection.
    pub fn prop_combobox(
        &mut self,
        scope: &mut irisui::prelude::UiScope<'_>,
        comp_name: &'static str,
        label: Option<&str>,
        selected: &mut usize,
        options: &[&str],
        width: f32,
    ) {
        let resp = scope.combobox_styled(label, selected, options, width);
        if resp.changed() {
            self.begin_edit(comp_name);
            self.commit_edit(comp_name);
        }
    }

    /// Binds an autonomous dropdown combobox row with explicit label and pill widths.
    ///
    /// Modifies `*selected` index in-place, triggering `begin_edit` and `commit_edit` automatically upon option selection.
    pub fn prop_combobox_row(
        &mut self,
        scope: &mut irisui::prelude::UiScope<'_>,
        comp_name: &'static str,
        label: &str,
        selected: &mut usize,
        options: &[&str],
        layout: ComboRowLayout,
    ) {
        let resp = scope.combobox_row(
            label,
            layout.label_width,
            selected,
            options,
            layout.button_width,
        );
        if resp.changed() {
            self.begin_edit(comp_name);
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