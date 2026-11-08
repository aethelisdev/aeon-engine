// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Conditional Scopes & Expression Bindings
//!
//! Provides functional, declarative control flow and value binding methods on [`UiScope`].
//! Enables consumer declarative UI trees to unpack values, handle optional data, and evaluate
//! conditional layout branches without intermediate imperative `let` variable statements or
//! nested `if let` pyramids, enforcing 100% pure declarative expressions.

use super::core::UiScope;

impl<'a> UiScope<'a> {
    /// Conditionally executes a declarative child closure if `condition` is `true`.
    ///
    /// Eliminates intermediate imperative branching (`if condition { ... }`) in consumer
    /// declarative UI hierarchies, preserving fluent expression chains.
    ///
    /// # Arguments
    /// * `condition` - Boolean predicate determining whether the child scope executes.
    /// * `f` - Child scope closure evaluated when `condition` is `true`.
    ///
    /// # Returns
    /// `Some(R)` with the closure output if evaluated, or `None` if skipped.
    #[inline]
    pub fn when<R, F>(&mut self, condition: bool, f: F) -> Option<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if condition { Some(f(self)) } else { None }
    }

    /// Conditionally executes a declarative child closure if `option` is [`Some`].
    ///
    /// Eliminates intermediate imperative `let Some(x) = ...` unpacking statements and guard clauses
    /// in consumer declarative UI trees, preserving pure fluent closure hierarchy.
    ///
    /// # Arguments
    /// * `option` - Optional value whose contained item is forwarded to the child scope closure.
    /// * `f` - Child scope closure evaluated with the unwrapped value when `option` is `Some`.
    ///
    /// # Returns
    /// `Some(R)` with the closure output if evaluated, or `None` if `option` was `None`.
    #[inline]
    pub fn when_some<T, R, F>(&mut self, option: Option<T>, f: F) -> Option<R>
    where
        F: FnOnce(&mut UiScope<'_>, T) -> R,
    {
        option.map(|val| f(self, val))
    }

    /// Conditionally executes a declarative child closure if `result` is [`Ok`].
    ///
    /// Eliminates intermediate imperative `let Ok(x) = ... else { return; }` unpacking statements
    /// and guard clauses in consumer declarative UI trees, preserving pure fluent closure hierarchy.
    ///
    /// # Arguments
    /// * `result` - Result value whose contained success item is forwarded to the child scope closure.
    /// * `f` - Child scope closure evaluated with the unwrapped success value when `result` is `Ok`.
    ///
    /// # Returns
    /// `Some(R)` with the closure output if evaluated, or `None` if `result` was `Err`.
    #[inline]
    pub fn when_ok<T, E, R, F>(&mut self, result: Result<T, E>, f: F) -> Option<R>
    where
        F: FnOnce(&mut UiScope<'_>, T) -> R,
    {
        result.ok().map(|val| f(self, val))
    }

    /// Declaratively binds an evaluated expression value and passes it into a child scope closure.
    ///
    /// Eliminates intermediate imperative `let x = ...;` statements by transforming variable definitions
    /// into scoped closure parameters within fluent UI builder expressions.
    ///
    /// # Arguments
    /// * `value` - Evaluated value to bind into the child scope.
    /// * `f` - Child scope closure receiving the bound value.
    ///
    /// # Returns
    /// The return value of the closure `R`.
    #[inline]
    pub fn bind<T, R, F>(&mut self, value: T, f: F) -> R
    where
        F: FnOnce(&mut UiScope<'_>, T) -> R,
    {
        f(self, value)
    }
}

#[cfg(test)]
mod tests {
    use crate::declarative::scope::UiScope;
    use iris_core::{UiTree, WidgetId};

    #[test]
    fn test_declarative_when_executes_only_on_true() {
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        let mut ran_true = false;
        let res_true = scope.when(true, |_| {
            ran_true = true;
            42
        });
        assert_eq!(res_true, Some(42));
        assert!(ran_true);

        let mut ran_false = false;
        let res_false = scope.when(false, |_| {
            ran_false = true;
            100
        });
        assert_eq!(res_false, None);
        assert!(!ran_false);
    }

    #[test]
    fn test_declarative_when_some_unpacks_correctly() {
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        let some_val: Option<&str> = Some("Aeon");
        let res_some = scope.when_some(some_val, |_s, val| {
            assert_eq!(val, "Aeon");
            val.len()
        });
        assert_eq!(res_some, Some(4));

        let none_val: Option<&str> = None;
        let res_none = scope.when_some(none_val, |_s, val| val.len());
        assert_eq!(res_none, None);
    }

    #[test]
    fn test_declarative_when_ok_unpacks_correctly() {
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        let ok_val: Result<i32, &'static str> = Ok(128);
        let res_ok = scope.when_ok(ok_val, |_s, val| {
            assert_eq!(val, 128);
            val * 2
        });
        assert_eq!(res_ok, Some(256));

        let err_val: Result<i32, &'static str> = Err("failed");
        let res_err = scope.when_ok(err_val, |_s, val| val * 2);
        assert_eq!(res_err, None);
    }

    #[test]
    fn test_declarative_bind_passes_value_to_closure() {
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        let result = scope.bind(10 + 20, |_s, val| {
            assert_eq!(val, 30);
            val + 5
        });
        assert_eq!(result, 35);
    }
}