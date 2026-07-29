// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use glam::Vec3;
use hecs::{Entity, World};
use rapier3d::prelude::*;
/// AE Physics — Decoupled physics simulation world wrapper using Rapier3D.
use std::collections::HashMap;

use ae_core::ecs::{AssetHandle, RaycastHit};

pub mod character;
pub mod sync_ecs;
pub mod sync_physics;
pub mod tests;

/// Decoupled wrapper managing Rapier3D simulation datasets.
/// Handles mapping between ECS Entities and Rapier handles, keeps simulator state,
/// runs steps, and updates transforms back to the ECS.
pub struct PhysicsWorld {
    /// Gravitational acceleration vector.
    pub gravity: Vec3,
    /// Parameters of the physics simulation integration step.
    pub integration_parameters: IntegrationParameters,
    /// Pipeline managing the simulation steps.
    pub physics_pipeline: PhysicsPipeline,
    /// Manager handling islands of active/sleeping bodies.
    pub island_manager: IslandManager,
    /// Broad phase collision detection.
    pub broad_phase: BroadPhaseBvh,
    /// Narrow phase collision detection.
    pub narrow_phase: NarrowPhase,
    /// Set of all simulated rigid bodies.
    pub rigid_body_set: RigidBodySet,
    /// Set of all simulated colliders.
    pub collider_set: ColliderSet,
    /// Set of impulse joints.
    pub impulse_joint_set: ImpulseJointSet,
    /// Set of multibody joints.
    pub multibody_joint_set: MultibodyJointSet,
    /// Solver for continuous collision detection (CCD).
    pub ccd_solver: CCDSolver,
    /// Mapping of ECS entities to Rapier rigid body handles.
    pub entity_to_body: HashMap<Entity, RigidBodyHandle>,
    /// Mapping of Rapier rigid body handles to ECS entities.
    pub body_to_entity: HashMap<RigidBodyHandle, Entity>,
}

impl PhysicsWorld {
    /// Creates a new `PhysicsWorld` with standard gravity and simulation settings.
    pub fn new() -> Self {
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.num_solver_iterations = 12;

        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            integration_parameters,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            entity_to_body: HashMap::new(),
            body_to_entity: HashMap::new(),
        }
    }

    /// Resets and clears all physics simulation state, rigid bodies, colliders, and entity mappings.
    pub fn clear(&mut self) {
        self.rigid_body_set = RigidBodySet::new();
        self.collider_set = ColliderSet::new();
        self.island_manager = IslandManager::new();
        self.broad_phase = BroadPhaseBvh::new();
        self.narrow_phase = NarrowPhase::new();
        self.impulse_joint_set = ImpulseJointSet::new();
        self.multibody_joint_set = MultibodyJointSet::new();
        self.ccd_solver = CCDSolver::new();
        self.entity_to_body.clear();
        self.body_to_entity.clear();
    }

    /// Performs one physics simulation step, processes collision and sensor events,
    /// broadcasts events to the DynamicEventBus, and synchronizes positions/velocities with the ECS world.
    pub fn step<'a, F>(
        &mut self,
        world: &mut World,
        get_mesh_data: F,
        delta_time: f32,
        event_bus: &mut ae_core::events::DynamicEventBus,
    ) where
        F: Fn(AssetHandle) -> Option<(&'a [[f32; 3]], &'a [u32])>,
    {
        // Ensure simulation parameters match the requested time step
        self.integration_parameters.dt = delta_time;

        // Update hierarchy transforms so GlobalTransform is 100% current for all parent-child entities
        ae_core::ecs::update_hierarchy_transforms(world);

        // 1. Sync ECS state to simulator
        self.sync_ecs_to_physics(world, get_mesh_data);

        // 2. Setup event collector for collision and trigger events
        let (collision_send, collision_recv) = std::sync::mpsc::channel();
        let (contact_force_send, _contact_force_recv) = std::sync::mpsc::channel();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

        // 3. Step the simulator
        self.physics_pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &(),
            &event_handler,
        );

        // 4. Process physics events and broadcast to DynamicEventBus
        while let Ok(event) = collision_recv.try_recv() {
            match event {
                CollisionEvent::Started(h1, h2, flags) => {
                    let parent_1 = self.collider_set.get(h1).and_then(|c| c.parent());
                    let parent_2 = self.collider_set.get(h2).and_then(|c| c.parent());
                    let entity_a = parent_1.and_then(|p| self.body_to_entity.get(&p)).copied();
                    let entity_b = parent_2.and_then(|p| self.body_to_entity.get(&p)).copied();

                    if let (Some(ea), Some(eb)) = (entity_a, entity_b) {
                        if flags.contains(CollisionEventFlags::SENSOR) {
                            event_bus.send(ae_core::events::TriggerEnter {
                                entity_a: ea,
                                entity_b: eb,
                            });
                        } else {
                            event_bus.send(ae_core::events::CollisionEnter {
                                entity_a: ea,
                                entity_b: eb,
                            });
                        }
                    }
                }
                CollisionEvent::Stopped(h1, h2, flags) => {
                    let parent_1 = self.collider_set.get(h1).and_then(|c| c.parent());
                    let parent_2 = self.collider_set.get(h2).and_then(|c| c.parent());
                    let entity_a = parent_1.and_then(|p| self.body_to_entity.get(&p)).copied();
                    let entity_b = parent_2.and_then(|p| self.body_to_entity.get(&p)).copied();

                    if let (Some(ea), Some(eb)) = (entity_a, entity_b) {
                        if flags.contains(CollisionEventFlags::SENSOR) {
                            event_bus.send(ae_core::events::TriggerExit {
                                entity_a: ea,
                                entity_b: eb,
                            });
                        } else {
                            event_bus.send(ae_core::events::CollisionExit {
                                entity_a: ea,
                                entity_b: eb,
                            });
                        }
                    }
                }
            }
        }

        // 5. Sync simulator state back to ECS
        self.sync_physics_to_ecs(world);
    }

    /// Casts a 3D ray into the physics world and returns the closest hit collider information.
    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<RaycastHit> {
        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_body_set,
            &self.collider_set,
            QueryFilter::default(),
        );

        let ray = Ray::new(
            Vec3::new(origin.x, origin.y, origin.z),
            Vec3::new(direction.x, direction.y, direction.z),
        );

        if let Some((handle, intersection)) =
            query_pipeline.cast_ray_and_get_normal(&ray, max_distance, true)
        {
            let collider = self.collider_set.get(handle)?;
            let body_handle = collider.parent()?;
            let entity = *self.body_to_entity.get(&body_handle)?;

            let hit_point = ray.point_at(intersection.time_of_impact);
            let normal = intersection.normal;

            Some(RaycastHit {
                entity,
                point: [hit_point.x, hit_point.y, hit_point.z],
                normal: [normal.x, normal.y, normal.z],
                distance: intersection.time_of_impact,
            })
        } else {
            None
        }
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}