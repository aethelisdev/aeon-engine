// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Number Input Value Handlers
//!
//! Subsystem coordinator and re-exports for reading and mutating numeric ECS component properties.
//!

pub mod getter;
pub mod setter;

pub use getter::read_inspector_number_value;
pub(crate) use setter::handle_set_number_value;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::InspectorNumberInputId;

    #[test]
    fn test_read_and_set_inspector_number_value_consistency() {
        let mut world = hecs::World::new();
        let entity = world.spawn((
            ae_core::ecs::Position::new(10.5, 20.25, -30.75),
            ae_core::ecs::Scale::new(2.0, 3.0, 4.0),
        ));

        let x = read_inspector_number_value(&world, entity, InspectorNumberInputId::PosX);
        let y = read_inspector_number_value(&world, entity, InspectorNumberInputId::PosY);
        let z = read_inspector_number_value(&world, entity, InspectorNumberInputId::PosZ);
        assert!((x - 10.5).abs() < 1e-4);
        assert!((y - 20.25).abs() < 1e-4);
        assert!((z - (-30.75)).abs() < 1e-4);

        let mut euler = [0.0; 3];
        handle_set_number_value(
            &world,
            entity,
            InspectorNumberInputId::PosX,
            42.0,
            &mut euler,
        );
        let new_x = read_inspector_number_value(&world, entity, InspectorNumberInputId::PosX);
        assert!((new_x - 42.0).abs() < 1e-4);
    }
}