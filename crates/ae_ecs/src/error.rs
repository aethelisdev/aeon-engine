// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! ECS error types decoupled from backend storage implementation details.

use std::fmt;

/// Error returned when an operation targets an [`crate::Entity`] handle that does not exist
/// in the [`crate::World`] (either despawned, stale generation, or [`crate::Entity::DANGLING`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoSuchEntity;

impl fmt::Display for NoSuchEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("entity does not exist in world")
    }
}

impl std::error::Error for NoSuchEntity {}

impl From<hecs::NoSuchEntity> for NoSuchEntity {
    #[inline]
    fn from(_: hecs::NoSuchEntity) -> Self {
        Self
    }
}

/// Error returned when fetching or removing a component from an entity fails.
///
/// Distinguishes missing entities from entities lacking the requested component type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentError {
    /// Target entity handle is not live in the world.
    NoSuchEntity,
    /// Target entity exists, but does not contain the requested component type.
    MissingComponent(String),
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchEntity => f.write_str("entity does not exist in world"),
            Self::MissingComponent(name) => write!(f, "{name}"),
        }
    }
}

impl std::error::Error for ComponentError {}

impl From<hecs::ComponentError> for ComponentError {
    #[inline]
    fn from(err: hecs::ComponentError) -> Self {
        match err {
            hecs::ComponentError::NoSuchEntity => Self::NoSuchEntity,
            hecs::ComponentError::MissingComponent(missing) => {
                Self::MissingComponent(missing.to_string())
            }
        }
    }
}

/// Error returned when evaluating a single-entity query fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryOneError {
    /// Target entity does not exist or its generation is stale.
    NoSuchEntity,
    /// Target entity exists, but does not satisfy every required component in the query.
    Unsatisfied,
}

impl fmt::Display for QueryOneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchEntity => f.write_str("entity does not exist in world"),
            Self::Unsatisfied => f.write_str("entity does not satisfy query"),
        }
    }
}

impl std::error::Error for QueryOneError {}

impl From<hecs::QueryOneError> for QueryOneError {
    #[inline]
    fn from(err: hecs::QueryOneError) -> Self {
        match err {
            hecs::QueryOneError::NoSuchEntity => Self::NoSuchEntity,
            hecs::QueryOneError::Unsatisfied => Self::Unsatisfied,
        }
    }
}