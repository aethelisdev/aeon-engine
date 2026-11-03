// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Strongly-Typed Centralized Action Bus
//!
//! Provides a lock-free, zero-reallocation FIFO action queue decoupling
//! UI control interactions (menu bars, toolbars, key bindings, modal dialogs, panels)
//! from backend application execution loops.

use std::collections::VecDeque;
use std::collections::vec_deque::Drain;

/// Strongly typed FIFO action bus queue decoupling UI triggers from execution logic.
///
/// Designed to eliminate ad-hoc action vectors (`pending_actions`, `menubar_actions`,
/// `inspector_actions`) and unify all event dispatches into a single deterministic stream.
///
/// # Type Parameters
/// * `A` - Strongly-typed action enum or struct implementing `'static + Send + Sync`.
pub struct ActionBus<A: 'static + Send + Sync> {
    queue: VecDeque<A>,
}

impl<A: 'static + Send + Sync> ActionBus<A> {
    /// Default pre-allocated capacity for the action bus queue to prevent reallocations.
    pub const DEFAULT_CAPACITY: usize = 64;

    /// Creates a new, empty action bus with default pre-allocated capacity.
    ///
    /// # Invariants
    /// * Initial queue length is 0.
    /// * Initial queue capacity is at least [`Self::DEFAULT_CAPACITY`].
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    /// Creates an empty action bus with a custom pre-allocated capacity.
    ///
    /// # Arguments
    /// * `capacity` - Initial number of elements the internal queue can hold without resizing.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
        }
    }

    /// Emits a single action to the back of the queue (FIFO ordering).
    ///
    /// # Arguments
    /// * `action` - Strongly-typed action payload to enqueue.
    pub fn emit(&mut self, action: A) {
        self.queue.push_back(action);
    }

    /// Emits a sequence of actions to the back of the queue in sequential order.
    ///
    /// # Arguments
    /// * `actions` - Iterator yielding action payloads.
    pub fn emit_batch<I>(&mut self, actions: I)
    where
        I: IntoIterator<Item = A>,
    {
        self.queue.extend(actions);
    }

    /// Drains all pending actions from the queue in FIFO order.
    ///
    /// Returns a [`Drain`] iterator that removes and yields elements.
    /// Retains the allocated capacity for subsequent frame dispatches without heap reallocations.
    pub fn drain(&mut self) -> Drain<'_, A> {
        self.queue.drain(..)
    }

    /// Returns the number of queued actions waiting to be drained.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns `true` if the queue contains no pending actions.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns the current total capacity of the internal queue buffer.
    #[must_use]
    #[inline]
    pub fn capacity(&self) -> usize {
        self.queue.capacity()
    }

    /// Returns a reference to the next action that will be popped by [`Self::drain`],
    /// or `None` if the queue is empty.
    #[must_use]
    #[inline]
    pub fn peek(&self) -> Option<&A> {
        self.queue.front()
    }

    /// Clears all queued actions without releasing the pre-allocated buffer memory.
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    /// Reserves capacity for at least `additional` more actions to be enqueued
    /// without reallocating the underlying storage.
    ///
    /// # Arguments
    /// * `additional` - Number of additional elements to allocate space for.
    pub fn reserve(&mut self, additional: usize) {
        self.queue.reserve(additional);
    }
}

impl<A: 'static + Send + Sync> Default for ActionBus<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: 'static + Send + Sync> std::fmt::Debug for ActionBus<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionBus")
            .field("len", &self.queue.len())
            .field("capacity", &self.queue.capacity())
            .finish()
    }
}

impl<A: 'static + Send + Sync + Clone> Clone for ActionBus<A> {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}

impl<A: 'static + Send + Sync + PartialEq> PartialEq for ActionBus<A> {
    fn eq(&self, other: &Self) -> bool {
        self.queue == other.queue
    }
}

impl<A: 'static + Send + Sync + Eq> Eq for ActionBus<A> {}

impl<A: 'static + Send + Sync> IntoIterator for ActionBus<A> {
    type Item = A;
    type IntoIter = std::collections::vec_deque::IntoIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        self.queue.into_iter()
    }
}
