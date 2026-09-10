// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use glam::Vec2;
use rapier2d::prelude::*;

use crate::body::{BodyType2D, RigidBody2D};
use crate::collider::{Collider2D, ColliderShape2D};

/// Complete 2D rigid-body simulation world powered by Rapier2D.
/// Encapsulates collision pipelines, numerical integrators, contact solvers, and spatial queries.
pub struct Physics2DWorld {
    /// Global gravity acceleration vector (units/sec^2). Standard earth gravity is `(0.0, -9.81)`.
    pub gravity: Vec2,
    /// Rapier2D active rigid bodies pool.
    pub rigid_body_set: RigidBodySet,
    /// Rapier2D active colliders pool.
    pub collider_set: ColliderSet,
    /// Physics integration and solver parameters.
    pub integration_parameters: IntegrationParameters,
    /// Physics step orchestration pipeline.
    pub physics_pipeline: PhysicsPipeline,
    /// Spatial clustering island manager for sleep optimizations.
    pub island_manager: IslandManager,
    /// Broad-phase collision detection acceleration structure.
    pub broad_phase: BroadPhaseBvh,
    /// Narrow-phase precise geometry intersection solver.
    pub narrow_phase: NarrowPhase,
    /// Impulse joint constraint solver.
    pub impulse_joint_set: ImpulseJointSet,
    /// Multibody articulation joint constraint solver.
    pub multibody_joint_set: MultibodyJointSet,
    /// Continuous collision detection (CCD) solver.
    pub ccd_solver: CCDSolver,
}

impl Default for Physics2DWorld {
    fn default() -> Self {
        Self::new(Vec2::new(0.0, -9.81))
    }
}

impl Physics2DWorld {
    /// Creates an initialized 2D physics simulation world with specified gravity acceleration.
    pub fn new(gravity: Vec2) -> Self {
        Self {
            gravity,
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    /// Steps the physics simulation forward by `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        self.integration_parameters.dt = dt.min(0.1); // Clamp to prevent numerical explosion
        let physics_hooks = ();
        let event_handler = ();

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
            &physics_hooks,
            &event_handler,
        );
    }

    /// Registers a rigid body into the simulation world, returning its assigned `RigidBodyHandle`.
    pub fn spawn_rigid_body(
        &mut self,
        position: Vec2,
        rotation_radians: f32,
        body: &mut RigidBody2D,
    ) -> RigidBodyHandle {
        let rapier_type = match body.body_type {
            BodyType2D::Dynamic => RigidBodyType::Dynamic,
            BodyType2D::Fixed => RigidBodyType::Fixed,
            BodyType2D::KinematicVelocityBased => RigidBodyType::KinematicVelocityBased,
            BodyType2D::KinematicPositionBased => RigidBodyType::KinematicPositionBased,
        };

        let mut builder = RigidBodyBuilder::new(rapier_type)
            .translation(position)
            .rotation(rotation_radians)
            .linvel(body.linear_velocity)
            .angvel(body.angular_velocity)
            .linear_damping(body.linear_damping)
            .angular_damping(body.angular_damping)
            .gravity_scale(body.gravity_scale);

        if body.lock_rotations {
            builder = builder.lock_rotations();
        }

        let rb = builder.build();
        let handle = self.rigid_body_set.insert(rb);
        body.handle = Some(handle);
        handle
    }

    /// Attaches a collision geometry to an existing rigid body.
    pub fn attach_collider(
        &mut self,
        body_handle: RigidBodyHandle,
        collider: &mut Collider2D,
    ) -> ColliderHandle {
        let rapier_collider = match &collider.shape {
            ColliderShape2D::Cuboid { half_extents } => {
                ColliderBuilder::cuboid(half_extents.x, half_extents.y)
            }
            ColliderShape2D::Ball { radius } => ColliderBuilder::ball(*radius),
            ColliderShape2D::Capsule {
                half_height,
                radius,
            } => ColliderBuilder::capsule_y(*half_height, *radius),
        }
        .friction(collider.friction)
        .restitution(collider.restitution)
        .sensor(collider.is_sensor)
        .build();

        let handle = self.collider_set.insert_with_parent(
            rapier_collider,
            body_handle,
            &mut self.rigid_body_set,
        );
        collider.handle = Some(handle);
        handle
    }

    /// Performs a raycast in 2D space, returning the entity handle, hit point, normal, and time of impact.
    pub fn cast_ray(
        &self,
        origin: Vec2,
        direction: Vec2,
        max_distance: f32,
        solid: bool,
    ) -> Option<(ColliderHandle, Vec2, Vec2, f32)> {
        let dir_norm = direction.normalize_or_zero();
        if dir_norm == Vec2::ZERO {
            return None;
        }

        let ray = Ray::new(origin, dir_norm);
        let filter = QueryFilter::default();

        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_body_set,
            &self.collider_set,
            filter,
        );

        query_pipeline
            .cast_ray_and_get_normal(&ray, max_distance, solid)
            .map(|(handle, intersection)| {
                let hit_point = ray.point_at(intersection.time_of_impact);
                let hit_normal = intersection.normal;
                (handle, hit_point, hit_normal, intersection.time_of_impact)
            })
    }
}