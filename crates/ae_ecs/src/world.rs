// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Primary archetype-backed ECS world container and iteration primitives.

use crate::component::{BatchBundle, Bundle, Component, ComponentBundle};
use crate::entity::{Entity, EntityRef};
use crate::error::{ComponentError, NoSuchEntity, QueryOneError};
use crate::query::{EcsQuery, QueryBorrow, QueryMut, QueryOne};

/// Archetype-backed Entity Component System database storing entities and heterogeneous components.
///
/// Exposes explicit `&T` callbacks (`inspect`), explicit `&mut` accessors (`get_mut`,
/// `query_one_mut`), and archetype queries without leaking runtime borrow-guard wrapper types.
#[derive(Default)]
pub struct World {
    inner: hecs::World,
}

impl World {
    /// Creates a new, empty [`World`].
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: hecs::World::new(),
        }
    }

    /// Spawns a new entity populated with `components` and returns its generational [`Entity`] handle.
    #[inline]
    pub fn spawn(&mut self, components: impl Bundle) -> Entity {
        Entity::from_raw(components.spawn_into(&mut self.inner))
    }

    /// Spawns all entities from an archetype-uniform iterator in a single batch.
    #[inline]
    pub fn spawn_batch<'a, I>(&'a mut self, iter: I) -> impl Iterator<Item = Entity> + 'a
    where
        I: IntoIterator,
        I::IntoIter: 'a,
        I::Item: BatchBundle + 'a,
    {
        self.inner
            .spawn_batch(iter.into_iter().map(BatchBundle::into_raw))
            .map(Entity::from_raw)
    }

    /// Allocates an empty [`Entity`] handle without immediately attaching components.
    #[must_use]
    #[inline]
    pub fn reserve_entity(&mut self) -> Entity {
        Entity::from_raw(self.inner.reserve_entity())
    }

    /// Removes `entity` and drops all of its attached components.
    ///
    /// Returns `Err(NoSuchEntity)` if `entity` is no longer live.
    #[inline]
    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.inner
            .despawn(entity.into_raw())
            .map_err(NoSuchEntity::from)
    }

    /// Despawns all entities from the world while retaining allocated archetype storage capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns the total count of live entities in the world.
    #[must_use]
    #[inline]
    pub fn len(&self) -> u32 {
        self.inner.len()
    }

    /// Returns `true` if no live entities exist in the world.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns `true` if `entity` refers to a currently live entity with matching generation.
    #[must_use]
    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.inner.contains(entity.into_raw())
    }

    /// Retrieves a read-capable [`EntityRef`] view over `entity`, or `Err(NoSuchEntity)` if invalid.
    #[inline]
    pub fn entity(&self, entity: Entity) -> Result<EntityRef<'_>, NoSuchEntity> {
        self.inner
            .entity(entity.into_raw())
            .map(EntityRef)
            .map_err(NoSuchEntity::from)
    }

    /// Iterates sequentially over [`EntityRef`] handles for every live entity in the world.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = EntityRef<'_>> + '_ {
        self.inner.iter().map(EntityRef)
    }

    /// Returns `true` when `entity` exists and holds component `T`.
    #[must_use]
    #[inline]
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        self.inner.get::<&T>(entity.into_raw()).is_ok()
    }

    /// Evaluates `visitor` with a plain shared `&T` reference to `entity`'s component `T`.
    ///
    /// Scopes storage borrows cleanly inside `visitor` with zero wrapper types exposed.
    #[inline]
    pub fn inspect<T: Component, R>(
        &self,
        entity: Entity,
        visitor: impl FnOnce(&T) -> R,
    ) -> Result<R, ComponentError> {
        let guard = self
            .inner
            .get::<&T>(entity.into_raw())
            .map_err(ComponentError::from)?;
        Ok(visitor(&guard))
    }

    /// Evaluates `visitor` with an exclusive `&mut T` reference to `entity`'s component `T`.
    ///
    /// Scopes storage borrows inside `visitor` so callers on `&World` never hold runtime borrow guards.
    /// Prefer [`Self::get_mut`] when holding `&mut World`.
    #[inline]
    pub fn mutate<T: Component, R>(
        &self,
        entity: Entity,
        visitor: impl FnOnce(&mut T) -> R,
    ) -> Result<R, ComponentError> {
        let mut guard = self
            .inner
            .get::<&mut T>(entity.into_raw())
            .map_err(ComponentError::from)?;
        Ok(visitor(&mut guard))
    }

    /// Returns a cloned copy of component `T` on `entity`.
    #[inline]
    pub fn get_cloned<T: Component + Clone>(&self, entity: Entity) -> Result<T, ComponentError> {
        self.inspect::<T, T>(entity, Clone::clone)
    }

    /// Returns a bitwise copy of `Copy` component `T` on `entity`.
    #[inline]
    pub fn get_copied<T: Component + Copy>(&self, entity: Entity) -> Result<T, ComponentError> {
        self.inspect::<T, T>(entity, |value| *value)
    }

    /// Obtains an exclusive `&mut T` reference directly via `&mut self` without runtime borrow guards.
    #[inline]
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Result<&mut T, QueryOneError> {
        self.inner
            .query_one_mut::<&mut T>(entity.into_raw())
            .map_err(QueryOneError::from)
    }

    /// Inserts all components from `components` onto an existing `entity`, replacing duplicates.
    #[inline]
    pub fn insert(&mut self, entity: Entity, components: impl Bundle) -> Result<(), NoSuchEntity> {
        components
            .insert_into(&mut self.inner, entity.into_raw())
            .map_err(NoSuchEntity::from)
    }

    /// Inserts or replaces a single `component` instance onto `entity`.
    #[inline]
    pub fn insert_one<T: Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), NoSuchEntity> {
        self.inner
            .insert_one(entity.into_raw(), component)
            .map_err(NoSuchEntity::from)
    }

    /// Removes and returns a bundle `T` of components from `entity`.
    #[inline]
    pub fn remove<T: ComponentBundle>(&mut self, entity: Entity) -> Result<T, ComponentError> {
        self.inner
            .remove::<T::Raw>(entity.into_raw())
            .map(T::from_raw)
            .map_err(ComponentError::from)
    }

    /// Removes and returns a single component of type `T` from `entity`.
    #[inline]
    pub fn remove_one<T: Component>(&mut self, entity: Entity) -> Result<T, ComponentError> {
        self.inner
            .remove_one::<T>(entity.into_raw())
            .map_err(ComponentError::from)
    }

    /// Prepares a dynamically borrow-checked archetype query over the world.
    #[inline]
    pub fn query<Q: EcsQuery>(&self) -> QueryBorrow<'_, Q> {
        QueryBorrow(self.inner.query::<Q::Raw>())
    }

    /// Executes an exclusive mutable query without runtime borrow-check overhead.
    #[inline]
    pub fn query_mut<Q: EcsQuery>(&mut self) -> QueryMut<'_, Q> {
        QueryMut(self.inner.query_mut::<Q::Raw>())
    }

    /// Queries multiple components on a single target `entity` with dynamic borrow checks.
    #[inline]
    pub fn query_one<Q: EcsQuery>(&self, entity: Entity) -> QueryOne<'_, Q> {
        QueryOne(self.inner.query_one::<Q::Raw>(entity.into_raw()))
    }

    /// Executes an exclusive mutable multi-component lookup on `entity`.
    #[inline]
    pub fn query_one_mut<Q: EcsQuery>(
        &mut self,
        entity: Entity,
    ) -> Result<Q::Item<'_>, QueryOneError> {
        self.inner
            .query_one_mut::<Q::Raw>(entity.into_raw())
            .map(Q::map_item)
            .map_err(QueryOneError::from)
    }
}