// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Type-safe dynamic resource container for dependency injection.
//!

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Type-safe, dynamic resource container (dependency injection map).
pub struct Resources {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Resources {
    /// Creates a new empty `Resources` container.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Inserts a typed resource into the container.
    pub fn insert<T: Any + Send + Sync>(&mut self, resource: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(resource));
    }

    /// Retrieves an immutable reference to a typed resource if present.
    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|any| any.downcast_ref::<T>())
    }

    /// Retrieves a mutable reference to a typed resource if present.
    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|any| any.downcast_mut::<T>())
    }

    /// Removes and returns a typed resource from the container if present.
    pub fn remove<T: Any + Send + Sync>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|any| any.downcast::<T>().ok().map(|boxed| *boxed))
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

/// A resource containing the list of entity IDs that were visible in the previous frame.
#[derive(Debug, Clone, Default)]
pub struct VisibleEntities {
    /// Inner list of visible entity generational IDs.
    pub entities: Vec<hecs::Entity>,
}