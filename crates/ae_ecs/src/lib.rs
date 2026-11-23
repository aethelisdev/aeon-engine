// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Swappable Entity Component System (`ae_ecs`)
//!
//! Isolates the underlying ECS backend behind engine-owned types (`World`, `Entity`,
//! `EntityBuilder`, `EntityCommandBuffer`, and typed query traits) so upper subsystems
//! never depend directly on external ECS crate APIs.

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![deny(clippy::allow_attributes)]

pub mod commands;
pub mod component;
pub mod entity;
pub mod error;
pub mod query;
pub mod registry;
pub mod world;

#[cfg(test)]
mod tests;

pub use commands::{CommandFn, EntityCommandBuffer};
pub use component::{BatchBundle, Bundle, Component, ComponentBundle};
pub use entity::{BuiltEntity, Entity, EntityBuilder, EntityRef};
pub use error::{ComponentError, NoSuchEntity, QueryOneError};
pub use query::{EcsQuery, QueryBorrow, QueryIter, QueryMut, QueryOne};
pub use registry::{
    ComponentHandler, ComponentRegistry, TypedComponentHandler, component_type_hash,
};
pub use world::World;