// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Number Input Value Handlers
//!
//! Subsystem coordinator and re-exports for reading and mutating numeric ECS component properties.

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

    #[test]
    fn test_rotation_y_drag_preserves_x_and_z() {
        let mut world = hecs::World::new();
        let entity = world.spawn((
            ae_core::ecs::Position::new(0.0, 5.0, 0.0),
            crate::ui::iris_bridge::inspector::euler_deg_to_quaternion(-34.0, 0.0, 0.0),
            ae_core::ecs::Scale::new(1.0, 1.0, 1.0),
        ));

        let mut euler = [-34.0, 0.0, 0.0];
        handle_set_number_value(
            &world,
            entity,
            InspectorNumberInputId::RotY,
            41.0,
            &mut euler,
        );

        let rx = read_inspector_number_value(&world, entity, InspectorNumberInputId::RotX);
        let ry = read_inspector_number_value(&world, entity, InspectorNumberInputId::RotY);
        let rz = read_inspector_number_value(&world, entity, InspectorNumberInputId::RotZ);

        assert!((rx - (-34.0)).abs() < 1e-2, "RotX drifted to {}", rx);
        assert!((ry - 41.0).abs() < 1e-2, "RotY drifted to {}", ry);
        assert!((rz - 0.0).abs() < 1e-2, "RotZ drifted to {}", rz);
        assert_eq!(euler, [-34.0, 41.0, 0.0]);
    }

    #[test]
    fn test_rotation_z_drag_negative_and_positive() {
        let mut world = hecs::World::new();
        let entity = world.spawn((
            ae_core::ecs::Position::new(0.0, 5.0, 0.0),
            ae_core::ecs::Rotation::default(),
            ae_core::ecs::Scale::new(1.0, 1.0, 1.0),
        ));

        let mut euler = [0.0; 3];

        // Drag Z to negative value (user's bug: dragging left dropped to 0 and got stuck)
        handle_set_number_value(
            &world,
            entity,
            InspectorNumberInputId::RotZ,
            -25.0,
            &mut euler,
        );
        let rz_neg = read_inspector_number_value(&world, entity, InspectorNumberInputId::RotZ);
        assert!(
            (rz_neg - (-25.0)).abs() < 1e-2,
            "RotZ neg failed: {}",
            rz_neg
        );

        // Drag Z to positive value
        handle_set_number_value(
            &world,
            entity,
            InspectorNumberInputId::RotZ,
            50.0,
            &mut euler,
        );
        let rz_pos = read_inspector_number_value(&world, entity, InspectorNumberInputId::RotZ);
        assert!((rz_pos - 50.0).abs() < 1e-2, "RotZ pos failed: {}", rz_pos);
    }

    #[test]
    fn test_rotation_y_drag_past_90_degrees_does_not_flip_x_or_z() {
        let mut world = hecs::World::new();
        let entity = world.spawn((
            ae_core::ecs::Position::new(0.0, 5.0, 0.0),
            crate::ui::iris_bridge::inspector::euler_deg_to_quaternion(-34.0, 0.0, 0.0),
            ae_core::ecs::Scale::new(1.0, 1.0, 1.0),
        ));

        let mut euler = [-34.0, 0.0, 0.0];

        // Drag Y past 90 degrees to 120 degrees
        handle_set_number_value(
            &world,
            entity,
            InspectorNumberInputId::RotY,
            120.0,
            &mut euler,
        );

        assert_eq!(euler[0], -34.0, "RotX flipped in euler!");
        assert_eq!(euler[1], 120.0, "RotY incorrect!");
        assert_eq!(euler[2], 0.0, "RotZ flipped in euler!");

        // Also test negative past -90 degrees (-135 degrees)
        handle_set_number_value(
            &world,
            entity,
            InspectorNumberInputId::RotY,
            -135.0,
            &mut euler,
        );

        assert_eq!(euler[0], -34.0, "RotX flipped in euler on negative!");
        assert_eq!(euler[1], -135.0, "RotY incorrect on negative!");
        assert_eq!(euler[2], 0.0, "RotZ flipped in euler on negative!");
    }

    #[test]
    fn test_commit_number_edit_action_lifecycle() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Position::new(1.0, 2.0, 3.0),));

        let registry = ae_core::registry::ComponentRegistry::global();
        let comp_name = "Position";
        let handler = registry
            .get_by_name(comp_name)
            .expect("Position component must be registered in ComponentRegistry");

        let old_bytes = handler
            .capture(&world, entity)
            .expect("Position should be captured");

        // Mutate position
        let mut euler = [0.0; 3];
        handle_set_number_value(
            &world,
            entity,
            InspectorNumberInputId::PosX,
            10.0,
            &mut euler,
        );

        let new_bytes = handler
            .capture(&world, entity)
            .expect("New position should be captured");
        assert_ne!(old_bytes, new_bytes);

        // Verify that applying old_bytes restores original value (Undo)
        handler
            .apply(&mut world, entity, &old_bytes)
            .expect("Apply must succeed");
        let restored_x = read_inspector_number_value(&world, entity, InspectorNumberInputId::PosX);
        assert!((restored_x - 1.0).abs() < 1e-4);

        // Verify that applying new_bytes re-applies mutated value (Redo)
        handler
            .apply(&mut world, entity, &new_bytes)
            .expect("Apply must succeed");
        let redone_x = read_inspector_number_value(&world, entity, InspectorNumberInputId::PosX);
        assert!((redone_x - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_checkbox_toggle_action_lifecycle() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.7,
            restitution: 0.0,
            is_sensor: false,
        },));

        let registry = ae_core::registry::ComponentRegistry::global();
        let comp_name = "Collider";
        let handler = registry
            .get_by_name(comp_name)
            .expect("Collider component must be registered in ComponentRegistry");

        let old_bytes = handler
            .capture(&world, entity)
            .expect("Collider should be captured");

        // Toggle is_sensor
        if let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity) {
            c.is_sensor = !c.is_sensor;
        }

        let new_bytes = handler
            .capture(&world, entity)
            .expect("Toggled collider should be captured");
        assert_ne!(old_bytes, new_bytes);

        // Verify that is_sensor is true
        assert!(
            world
                .get::<&ae_core::ecs::Collider>(entity)
                .unwrap()
                .is_sensor
        );

        // Verify Undo: restore old bytes restores is_sensor = false
        handler
            .apply(&mut world, entity, &old_bytes)
            .expect("Apply must succeed");
        assert!(
            !world
                .get::<&ae_core::ecs::Collider>(entity)
                .unwrap()
                .is_sensor
        );

        // Verify Redo: restore new bytes restores is_sensor = true
        handler
            .apply(&mut world, entity, &new_bytes)
            .expect("Apply must succeed");
        assert!(
            world
                .get::<&ae_core::ecs::Collider>(entity)
                .unwrap()
                .is_sensor
        );
    }
}