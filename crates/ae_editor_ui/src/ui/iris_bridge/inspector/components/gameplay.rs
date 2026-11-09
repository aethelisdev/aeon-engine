// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Gameplay and Motion Component Inspector Cards
//!
//! Re-exports modular inspector handlers for gameplay, motion, and interaction components:
//! - `CharacterAction`: Action speed, range, and cooldown settings.
//! - `DestructibleTarget`: Numeric health readout and visual health status bar.
//! - `MovingPlatform`: Waypoint navigation and linear translation speed.
//! - `Rotator`: Continuous rotation around an arbitrary 3D axis.
//! - `TriggerZone`: Proximity detection and elevation activation trigger.
//! - `Velocity`: 3D linear velocity components.

pub mod action;
pub mod destructible;
pub mod platform;
pub mod rotator;
pub mod trigger;
pub mod velocity;

pub use action::{CharacterActionHandler, draw_character_action_card};
pub use destructible::{DestructibleTargetHandler, draw_destructible_target_card};
pub use platform::{MovingPlatformHandler, draw_moving_platform_card};
pub use rotator::{RotatorHandler, draw_rotator_card};
pub use trigger::{TriggerZoneHandler, draw_trigger_zone_card};
pub use velocity::{VelocityHandler, draw_velocity_card};