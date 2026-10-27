// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Panel Registry
//!
//! Provides the generic [`PanelRegistry`] mapping abstract tab identifiers to
//! their declarative UI drawing closures, decoupling workbench chrome from domain state.

use crate::error::{WorkbenchError, WorkbenchResult};
use iris_widgets::UiScope;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

/// Type alias for a thread-safe panel drawing closure.
///
/// Accepts a mutable [`UiScope`] representing the panel's layout scope and a mutable
/// reference to the caller's custom `Context` containing domain/engine state.
pub type PanelDrawFn<Context> = Box<dyn Fn(&mut UiScope<'_>, &mut Context) + Send + Sync>;

/// Central panel registry mapping panel identifiers to their declarative UI drawing functions.
///
/// Designed to decouple the workbench application shell from domain-specific editor state.
/// The workbench calls the registered closure when constructing isolated dock leaf panels
/// or detached floating windows without knowing the internal ECS or engine architecture.
pub struct WorkbenchPanelRegistry<TabId, Context> {
    draw_callbacks: HashMap<TabId, PanelDrawFn<Context>>,
}

/// Type alias for [`WorkbenchPanelRegistry`] providing standard workbench ergonomics.
pub type PanelRegistry<TabId, Context> = WorkbenchPanelRegistry<TabId, Context>;

impl<TabId, Context> Default for WorkbenchPanelRegistry<TabId, Context> {
    fn default() -> Self {
        Self {
            draw_callbacks: HashMap::new(),
        }
    }
}

impl<TabId, Context> WorkbenchPanelRegistry<TabId, Context>
where
    TabId: Eq + Hash + Clone + Debug,
{
    /// Creates an empty panel registry.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a declarative drawing closure for the given panel tab identifier.
    ///
    /// If a panel with the same identifier was previously registered, its callback is replaced.
    ///
    /// # Arguments
    /// * `tab` - The panel tab identifier (e.g. enum variant `PanelId::Hierarchy`).
    /// * `draw_fn` - Thread-safe closure executing declarative UI calls within [`UiScope`].
    pub fn register_panel(
        &mut self,
        tab: TabId,
        draw_fn: impl Fn(&mut UiScope<'_>, &mut Context) + Send + Sync + 'static,
    ) {
        self.draw_callbacks.insert(tab, Box::new(draw_fn));
    }

    /// Unregisters the drawing closure for the specified panel tab identifier.
    ///
    /// Returns `true` if a panel callback was removed, or `false` if it was not present.
    pub fn unregister_panel(&mut self, tab: &TabId) -> bool {
        self.draw_callbacks.remove(tab).is_some()
    }

    /// Returns `true` if a drawing closure is registered for the given panel tab.
    #[inline]
    #[must_use]
    pub fn is_registered(&self, tab: &TabId) -> bool {
        self.draw_callbacks.contains_key(tab)
    }

    /// Returns the number of registered panels.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.draw_callbacks.len()
    }

    /// Returns `true` if no panels have been registered.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.draw_callbacks.is_empty()
    }

    /// Invokes the registered drawing closure for `tab` within the provided [`UiScope`].
    ///
    /// # Errors
    /// Returns [`WorkbenchError::PanelNotFound`] if no drawing callback is registered for `tab`.
    pub fn draw(
        &self,
        tab: &TabId,
        scope: &mut UiScope<'_>,
        context: &mut Context,
    ) -> WorkbenchResult<()> {
        match self.draw_callbacks.get(tab) {
            Some(callback) => {
                (callback)(scope, context);
                Ok(())
            }
            None => Err(WorkbenchError::PanelNotFound(format!("{tab:?}"))),
        }
    }
}
