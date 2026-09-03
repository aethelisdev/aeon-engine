// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// AE Editor UI — Egui-based panel rendering and user interface event handlers.
pub mod processor;
pub mod ui;

pub use processor as ui_processor;
pub use processor::UiContext;

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::ecs::*;
    use cgmath::SquareMatrix;

    /// Tests that ParentEntity preserves exact world-space position and scale ("Keep World Transform").
    #[test]
    fn test_keep_world_transform_parenting() {
        let mut world = hecs::World::new();

        // Spawn scaled parent (Zemin)
        let parent = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 50.0,
                y: 1.0,
                z: 50.0,
            },
            GlobalTransform(cgmath::Matrix4::identity()),
        ));

        // Spawn child (Küp)
        let child = world.spawn((
            Position {
                x: 10.0,
                y: 5.0,
                z: 10.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            GlobalTransform(cgmath::Matrix4::identity()),
        ));

        // Sync initial transforms
        update_hierarchy_transforms(&mut world);

        // Process ParentEntity action logic
        let child_world = processor::transform::get_world_matrix(&world, child);
        let parent_world = processor::transform::get_world_matrix(&world, parent);
        let parent_inv = parent_world.invert().unwrap();

        let new_local = parent_inv * child_world;
        let (new_pos, new_rot, new_scale) = processor::transform::decompose_matrix(new_local);

        world.insert_one(child, new_pos).unwrap();
        world.insert_one(child, new_rot).unwrap();
        world.insert_one(child, new_scale).unwrap();
        world.insert_one(child, Parent(parent)).unwrap();

        // Sync post-parenting hierarchy transforms
        update_hierarchy_transforms(&mut world);

        // Verify child's new local scale compensated for parent's 50x scale
        let local_scale = world.get::<&Scale>(child).unwrap();
        assert!((local_scale.x - 0.02).abs() < 0.001);

        // Verify child's world position remains EXACTLY at (10, 5, 10)
        let child_gt = world.get::<&GlobalTransform>(child).unwrap();
        assert!((child_gt.0.w.x - 10.0).abs() < 0.001);
        assert!((child_gt.0.w.y - 5.0).abs() < 0.001);
        assert!((child_gt.0.w.z - 10.0).abs() < 0.001);
    }

    /// Tests that CommitComponentModify pushes an undo command and undoes/redoes correctly.
    #[test]
    fn test_commit_component_modify_undo_redo() {
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Position::new(0.0, 0.0, 0.0),
            Velocity {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        ));

        let mut editor = ae_editor::editor_state::EditorState::default();
        let old_vel = Velocity {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let new_vel = Velocity {
            x: 10.0,
            y: 20.0,
            z: 30.0,
        };

        let old_data = serde_json::to_vec(&old_vel).unwrap();
        let new_data = serde_json::to_vec(&new_vel).unwrap();

        // Simulate CommitComponentModify processing
        let _ = world.insert_one(entity, new_vel);
        ae_editor::history::push_undo(
            &mut editor,
            ae_editor::undo_redo::Command::Modify(
                entity,
                ae_editor::undo_redo::Property::Component {
                    type_name: "Velocity".to_string(),
                    old_data,
                    new_data,
                },
            ),
        );
        assert_eq!(editor.undo_stack.len(), 1);

        // Verify current value
        assert_eq!(world.get::<&Velocity>(entity).unwrap().x, 10.0);

        // Undo
        ae_editor::history::undo(&mut editor, &mut world);
        assert_eq!(world.get::<&Velocity>(entity).unwrap().x, 1.0);
        assert_eq!(world.get::<&Velocity>(entity).unwrap().y, 2.0);
        assert_eq!(world.get::<&Velocity>(entity).unwrap().z, 3.0);

        // Redo
        ae_editor::history::redo(&mut editor, &mut world);
        assert_eq!(world.get::<&Velocity>(entity).unwrap().x, 10.0);
        assert_eq!(world.get::<&Velocity>(entity).unwrap().y, 20.0);
        assert_eq!(world.get::<&Velocity>(entity).unwrap().z, 30.0);
    }
}