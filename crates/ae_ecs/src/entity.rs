// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Entity identifier handles, incremental entity builder, and scoped entity inspection views.

use std::any::TypeId;
use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use crate::component::{Bundle, Component};

/// Opaque generational handle uniquely identifying an entity in [`crate::World`].
///
/// Wraps the underlying storage handle without exposing third-party ECS types, allowing
/// the storage backend to be replaced independently of gameplay, physics, rendering, and UI code.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Entity(pub(crate) hecs::Entity);

impl Entity {
    /// Sentinel invalid handle that never matches a live entity in any world.
    pub const DANGLING: Self = Self(hecs::Entity::DANGLING);

    /// Wraps a backend entity handle.
    #[inline]
    pub(crate) const fn from_raw(raw: hecs::Entity) -> Self {
        Self(raw)
    }

    /// Unwraps the backend entity handle.
    #[inline]
    pub(crate) const fn into_raw(self) -> hecs::Entity {
        self.0
    }

    /// Returns the packed non-zero 64-bit representation `(generation << 32) | index`.
    #[must_use]
    #[inline]
    pub const fn to_bits(self) -> NonZeroU64 {
        self.0.to_bits()
    }

    /// Reconstructs an [`Entity`] handle from packed 64-bit bits, returning `None` for zero bits.
    #[must_use]
    #[inline]
    pub const fn from_bits(bits: u64) -> Option<Self> {
        match hecs::Entity::from_bits(bits) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    /// Returns the raw slot index of this entity handle.
    #[must_use]
    #[inline]
    pub const fn id(self) -> u32 {
        self.0.id()
    }
}

/// Incremental builder for assembling an entity's initial heterogeneous component set dynamically.
#[derive(Default)]
pub struct EntityBuilder(hecs::EntityBuilder);

impl EntityBuilder {
    /// Creates an empty component builder.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(hecs::EntityBuilder::new())
    }

    /// Appends a single component instance to the entity being constructed.
    #[inline]
    pub fn add<T: Component>(&mut self, component: T) -> &mut Self {
        self.0.add(component);
        self
    }

    /// Appends all components from a bundle to the entity being constructed.
    #[inline]
    pub fn add_bundle(&mut self, bundle: impl Bundle) -> &mut Self {
        bundle.add_to_builder(&mut self.0);
        self
    }

    /// Returns `true` if a component of type `T` has already been staged in this builder.
    #[must_use]
    #[inline]
    pub fn has<T: Component>(&self) -> bool {
        self.0.has::<T>()
    }

    /// Clears all staged components without freeing the builder's allocation.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Finalizes the staged components into a spawnable [`BuiltEntity`] payload.
    #[must_use]
    #[inline]
    pub fn build(&mut self) -> BuiltEntity<'_> {
        BuiltEntity(self.0.build())
    }
}

/// Finalized dynamic component bundle produced by [`EntityBuilder::build`].
///
/// Can be passed directly to [`crate::World::spawn`] or [`crate::World::insert`].
pub struct BuiltEntity<'a>(pub(crate) hecs::BuiltEntity<'a>);

impl Bundle for BuiltEntity<'_> {
    #[inline]
    fn spawn_into(self, world: &mut hecs::World) -> hecs::Entity {
        world.spawn(self.0)
    }

    #[inline]
    fn insert_into(
        self,
        world: &mut hecs::World,
        entity: hecs::Entity,
    ) -> Result<(), hecs::NoSuchEntity> {
        world.insert(entity, self.0)
    }

    #[inline]
    fn add_to_builder(self, builder: &mut hecs::EntityBuilder) {
        builder.add_bundle(self.0);
    }
}

/// Borrowed inspection view of all components attached to a single live entity in [`crate::World`].
pub struct EntityRef<'w>(pub(crate) hecs::EntityRef<'w>);

impl<'w> EntityRef<'w> {
    /// Returns the [`Entity`] handle referenced by this view.
    #[must_use]
    #[inline]
    pub fn entity(&self) -> Entity {
        Entity::from_raw(self.0.entity())
    }

    /// Evaluates `visitor` with a plain shared `&T` reference when component `T` is present.
    ///
    /// Keeps internal storage borrow guards completely scoped inside the callback so no
    /// runtime borrow wrapper leaks into caller code.
    #[inline]
    pub fn inspect<T: Component, R>(&self, visitor: impl FnOnce(&T) -> R) -> Option<R> {
        self.0.get::<&T>().map(|guard| visitor(&guard))
    }

    /// Returns a cloned copy of component `T` when attached to this entity.
    #[must_use]
    #[inline]
    pub fn get_cloned<T: Component + Clone>(&self) -> Option<T> {
        self.0.get::<&T>().map(|guard| (*guard).clone())
    }

    /// Returns a copy of component `T` when attached to this entity.
    #[must_use]
    #[inline]
    pub fn get_copied<T: Component + Copy>(&self) -> Option<T> {
        self.0.get::<&T>().map(|guard| *guard)
    }

    /// Returns `true` if the entity contains a component of type `T`.
    #[must_use]
    #[inline]
    pub fn has<T: Component>(&self) -> bool {
        self.0.has::<T>()
    }

    /// Returns the number of components currently attached to this entity.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if this entity currently owns zero components.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    /// Iterates over the `TypeId` of every component type attached to this entity.
    #[inline]
    pub fn component_types(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.0.component_types()
    }
}