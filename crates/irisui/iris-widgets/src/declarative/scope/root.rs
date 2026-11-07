// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Root-Level Declarative Scope Initializer
//!
//! Exposes factory methods for constructing a top-level [`UiScope`] directly from
//! a [`UiTree`] without requiring consumers to perform manual `WidgetId` or root allocation.

use super::core::UiScope;
use iris_core::UiTree;

impl<'a> UiScope<'a> {
    /// Obtains or initializes the root node of the given [`UiTree`] and constructs a top-level [`UiScope`].
    ///
    /// If the tree currently lacks a root node, one is created automatically.
    /// Returns `None` only if root node allocation fails.
    #[inline]
    pub fn for_root(tree: &'a mut UiTree) -> Option<Self> {
        tree.root()
            .or_else(|| tree.create_root().ok())
            .map(|root_id| Self::new(tree, root_id))
    }

    /// Returns the active parent [`iris_core::WidgetId`] and a mutable reference to the underlying [`UiTree`] simultaneously.
    ///
    /// Resolves borrow checker splitting limitations when bridging legacy tree subroutines without intermediate `let` bindings.
    #[inline]
    pub fn parent_and_tree_mut(&mut self) -> (iris_core::WidgetId, &mut UiTree) {
        (self.parent, self.tree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_scope_for_root_initializes_and_reuses_tree_root() {
        let mut tree = UiTree::new();
        assert!(tree.root().is_none());

        let scope = UiScope::for_root(&mut tree).expect("Must create root on empty tree");
        let root_id = scope.parent();
        assert_eq!(tree.root(), Some(root_id));

        let scope2 = UiScope::for_root(&mut tree).expect("Must reuse existing root");
        assert_eq!(scope2.parent(), root_id);
    }
}