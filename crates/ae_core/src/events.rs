// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! AE Core - Dynamic Event Bus Infrastructure & Standard Gameplay Event Channels.
//!
//! Provides type-safe, decoupled pub/sub event dispatching for game systems,
//! combat actions, health tracking, trigger activations, and score state in 100% Safe Rust.
//!

pub use ae_plugin_api::{
    AudioMute, CollisionEnter, CollisionExit, DynamicEventBus, EntityDestroyed, EntitySpawned,
    Event, MaterialChanged, PhysicsTick, PhysicsUpdate, PlaySound, RaycastHitEvent, RenderFrame,
    StopSound, TargetDestroyedEvent, TriggerEnter, TriggerExit, ViewportResize,
};

/// Event broadcast when an entity inflicts damage upon a target entity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageEvent {
    /// Optional source entity (e.g., player, enemy, hazard trap) inflicting damage.
    pub source: Option<hecs::Entity>,
    /// Target entity receiving the damage.
    pub target: hecs::Entity,
    /// Floating-point damage amount deducted from target health.
    pub amount: f32,
    /// 3D world space coordinate where the impact occurred.
    pub hit_point: Option<[f32; 3]>,
    /// Surface normal vector at the point of impact.
    pub hit_normal: Option<[f32; 3]>,
}
impl Event for DamageEvent {}

/// Event broadcast when an entity receives healing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealEvent {
    /// Optional source entity (e.g., medkit, healing zone, spellcaster).
    pub source: Option<hecs::Entity>,
    /// Target entity receiving the health restoration.
    pub target: hecs::Entity,
    /// Amount of health points restored.
    pub amount: f32,
}
impl Event for HealEvent {}

/// Event broadcast when an entity enters or exits a trigger volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerEvent {
    /// The trigger sensor entity.
    pub trigger: hecs::Entity,
    /// The activator entity (e.g., player or actor) that triggered the zone.
    pub activator: hecs::Entity,
    /// `true` if entering the trigger zone, `false` if leaving/exiting.
    pub is_enter: bool,
}
impl Event for TriggerEvent {}

/// Event broadcast when an actor or destructible target is destroyed or killed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorKilledEvent {
    /// Optional entity responsible for the kill.
    pub killer: Option<hecs::Entity>,
    /// Victim entity that was eliminated.
    pub victim: hecs::Entity,
}
impl Event for ActorKilledEvent {}

/// Event broadcast when gameplay score or points are awarded or modified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreEvent {
    /// Relative point delta (positive for award, negative for penalty).
    pub delta: i32,
    /// New cumulative total score.
    pub new_total: i32,
}
impl Event for ScoreEvent {}

/// Generic custom gameplay event for game-specific scripts and plugins.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomGameplayEvent {
    /// Unique identifier tag for the custom game event (e.g., "QuestObjectiveMet", "WaveStarted").
    pub name: String,
    /// Arbitrary numeric payload associated with the event.
    pub value: f32,
}
impl Event for CustomGameplayEvent {}

/// Event broadcast when physical impact or projectile collision strikes a material surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceImpactEvent {
    /// Entity that was impacted.
    pub entity: hecs::Entity,
    /// Surface material type of the struck surface.
    pub surface_type: ae_plugin_api::SurfaceType,
    /// 3D world space coordinate where the impact occurred.
    pub hit_point: [f32; 3],
    /// Surface normal vector pointing outward from the impact plane.
    pub hit_normal: [f32; 3],
    /// Impact energy or impulse magnitude.
    pub energy: f32,
}
impl Event for SurfaceImpactEvent {}