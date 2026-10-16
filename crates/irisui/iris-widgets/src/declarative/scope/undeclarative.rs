// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Imperative Escape Hatch Scope (`UndeclarativeScope`)
//!
//! Provides an explicit, deliberate escape hatch for scenarios where raw identifier extraction,
//! imperative tree mutations, or legacy manual interaction tracking are temporarily required.
//!

use super::core::UiScope;
use iris_core::{UiTree, WidgetId, WidgetResponse};

/// Explicit escape hatch for imperative mutations and manual [`WidgetId`] extraction.
///
/// Declarative UI architecture encourages purely hierarchical component definitions and semantic
/// tag routing. However, when interfacing with lower-level systems or executing manual node
/// inspections, this scope allows explicit opt-in access to raw identifiers and tree mutations.
pub struct UndeclarativeScope<'a, 's> {
    pub(crate) scope: &'s mut UiScope<'a>,
}

impl<'a, 's> UndeclarativeScope<'a, 's> {
    /// Creates a new undeclarative scope wrapping an existing [`UiScope`].
    ///
    /// # Arguments
    /// * `scope` - Mutable reference to the parent declarative scope.
    pub fn new(scope: &'s mut UiScope<'a>) -> Self {
        Self { scope }
    }

    /// Returns the raw parent [`WidgetId`] of the current scope container.
    pub fn parent_id(&self) -> WidgetId {
        self.scope.parent
    }

    /// Provides immutable access to the underlying [`UiTree`] arena.
    pub fn tree(&self) -> &UiTree {
        self.scope.tree
    }

    /// Provides mutable access to the underlying [`UiTree`] arena.
    pub fn tree_mut(&mut self) -> &mut UiTree {
        self.scope.tree
    }

    /// Returns a mutable reference to the wrapped [`UiScope`].
    pub fn scope_mut(&mut self) -> &mut UiScope<'a> {
        self.scope
    }

    /// Emits a standard push button and explicitly returns both its [`WidgetId`] and [`WidgetResponse`].
    ///
    /// # Arguments
    /// * `label` - Text label displayed on the button.
    pub fn raw_button(&mut self, label: impl Into<String>) -> (WidgetId, WidgetResponse) {
        let resp = self.scope.button(label);
        (resp.id, resp)
    }

    /// Emits a tagged push button and explicitly returns both its [`WidgetId`] and [`WidgetResponse`].
    ///
    /// # Arguments
    /// * `label` - Text label displayed on the button.
    /// * `tag` - 64-bit semantic tag for interaction identification.
    pub fn raw_button_tagged(
        &mut self,
        label: impl Into<String>,
        tag: u64,
    ) -> (WidgetId, WidgetResponse) {
        let resp = self
            .scope
            .add(crate::declarative::widget::Button::new(label).tag(tag));
        (resp.id, resp)
    }

    /// Explicitly extracts the raw [`WidgetId`] from an emitted [`WidgetResponse`].
    ///
    /// # Arguments
    /// * `response` - Widget response containing the identifier.
    pub fn leak_id(&self, response: &WidgetResponse) -> WidgetId {
        response.id
    }
}

#[cfg(test)]
mod tests {
    use crate::declarative::scope::UiScope;
    use iris_core::UiTree;

    #[test]
    fn test_undeclarative_escape_hatch_extraction() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        let (extracted_id, response) = scope.undeclarative(|imp| {
            assert_eq!(imp.parent_id(), root);
            imp.raw_button("Imperative Button")
        });

        assert_eq!(extracted_id, response.id);
        assert_ne!(extracted_id, root);
    }

    #[test]
    fn test_undeclarative_leak_id_and_tree_access() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        let leaked = scope.undeclarative(|imp| {
            let (btn_id, resp) = imp.raw_button_tagged("Tagged Raw", 999);
            assert_eq!(imp.leak_id(&resp), btn_id);
            assert!(imp.tree().get(btn_id).is_some());
            btn_id
        });

        assert!(tree.get(leaked).is_some());
    }
}