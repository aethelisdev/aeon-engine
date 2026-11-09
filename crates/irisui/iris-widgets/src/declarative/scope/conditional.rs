// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Conditional Scopes & Expression Bindings
//!
//! Provides functional, declarative control flow and value binding methods on [`UiScope`].
//! Enables consumer declarative UI trees to unpack values, handle optional data, and evaluate
//! conditional layout branches without intermediate imperative `let` variable statements or
//! nested `if let` pyramids, enforcing 100% pure declarative expressions.

use super::containers::cards::CardHeaderDescriptor;
use super::containers::generic::InnerResponse;
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

    /// Conditionally executes a declarative child UI scope if both results are `Ok`.
    ///
    /// Eliminates multi-variable unpacking pyramid nesting and imperative `if let (Ok(a), Ok(b)) = ...`
    /// statements in dual-component declarative inspectors.
    ///
    /// # Arguments
    /// * `r1` - First result to evaluate.
    /// * `r2` - Second result to evaluate.
    /// * `f` - Child scope closure evaluated with both unwrapped success values when both are `Ok`.
    #[inline]
    pub fn when_ok2<T1, E1, T2, E2, R, F>(
        &mut self,
        r1: Result<T1, E1>,
        r2: Result<T2, E2>,
        f: F,
    ) -> Option<R>
    where
        F: FnOnce(&mut UiScope<'_>, T1, T2) -> R,
    {
        match (r1, r2) {
            (Ok(v1), Ok(v2)) => Some(f(self, v1, v2)),
            _ => None,
        }
    }

    /// Conditionally executes a declarative child UI scope if all three results are `Ok`.
    ///
    /// Eliminates multi-variable unpacking pyramid nesting and imperative `if let (Ok(a), Ok(b), Ok(c)) = ...`
    /// statements in multi-component declarative inspectors (e.g. 3D spatial Position, Rotation, Scale).
    ///
    /// # Arguments
    /// * `r1` - First result to evaluate.
    /// * `r2` - Second result to evaluate.
    /// * `r3` - Third result to evaluate.
    /// * `f` - Child scope closure evaluated with all three unwrapped success values when all are `Ok`.
    #[inline]
    pub fn when_ok3<T1, E1, T2, E2, T3, E3, R, F>(
        &mut self,
        r1: Result<T1, E1>,
        r2: Result<T2, E2>,
        r3: Result<T3, E3>,
        f: F,
    ) -> Option<R>
    where
        F: FnOnce(&mut UiScope<'_>, T1, T2, T3) -> R,
    {
        match (r1, r2, r3) {
            (Ok(v1), Ok(v2), Ok(v3)) => Some(f(self, v1, v2, v3)),
            _ => None,
        }
    }

    /// Evaluates a boolean condition and branches declaratively to either `on_true` or `on_false`.
    ///
    /// Eliminates imperative `if / else` statements in declarative UI hierarchies, preserving fluent expression chains.
    ///
    /// # Arguments
    /// * `condition` - Boolean predicate determining which branch executes.
    /// * `on_true` - Scope closure evaluated when `condition` is `true`.
    /// * `on_false` - Scope closure evaluated when `condition` is `false`.
    #[inline]
    pub fn branch<R, FTrue, FFalse>(
        &mut self,
        condition: bool,
        on_true: FTrue,
        on_false: FFalse,
    ) -> R
    where
        FTrue: FnOnce(&mut UiScope<'_>) -> R,
        FFalse: FnOnce(&mut UiScope<'_>) -> R,
    {
        if condition {
            on_true(self)
        } else {
            on_false(self)
        }
    }

    /// Evaluates an [`Option`] and branches declaratively to either `on_some` or `on_none`.
    ///
    /// Eliminates imperative `if let Some(...) / else` statements and unpacking pyramids in declarative UI hierarchies.
    ///
    /// # Arguments
    /// * `option` - Optional value to unpack.
    /// * `on_some` - Scope closure evaluated with unwrapped value when `option` is `Some`.
    /// * `on_none` - Scope closure evaluated when `option` is `None`.
    #[inline]
    pub fn branch_some<T, R, FSome, FNone>(
        &mut self,
        option: Option<T>,
        on_some: FSome,
        on_none: FNone,
    ) -> R
    where
        FSome: FnOnce(&mut UiScope<'_>, T) -> R,
        FNone: FnOnce(&mut UiScope<'_>) -> R,
    {
        match option {
            Some(val) => on_some(self, val),
            None => on_none(self),
        }
    }

    /// Evaluates a [`Result`] and branches declaratively to either `on_ok` or `on_err`.
    ///
    /// Eliminates imperative `if let Ok(...) / else` or `match` statements and unpacking pyramids in declarative UI hierarchies.
    ///
    /// # Arguments
    /// * `result` - Result value to unpack.
    /// * `on_ok` - Scope closure evaluated with unwrapped value when `result` is `Ok`.
    /// * `on_err` - Scope closure evaluated with error value when `result` is `Err`.
    #[inline]
    pub fn branch_ok<T, E, R, FOk, FErr>(
        &mut self,
        result: Result<T, E>,
        on_ok: FOk,
        on_err: FErr,
    ) -> R
    where
        FOk: FnOnce(&mut UiScope<'_>, T) -> R,
        FErr: FnOnce(&mut UiScope<'_>, E) -> R,
    {
        match result {
            Ok(val) => on_ok(self, val),
            Err(err) => on_err(self, err),
        }
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

    /// Conditionally executes a declarative child closure inside a styled card panel if `result` is [`Ok`].
    ///
    /// Eliminates nested closure indentation pyramids (`scope.when_ok(..., |s, v| s.card_panel(..., |c| { ... })))`
    /// by combining conditional unwrapping and card panel emission into a single scope level.
    ///
    /// # Arguments
    /// * `name` - Node identifier for layout tracing and debugging.
    /// * `result` - Result value whose contained success item is forwarded to the card scope.
    /// * `f` - Child scope closure evaluated with the card container scope and unwrapped value when `result` is `Ok`.
    ///
    /// # Returns
    /// `Some(InnerResponse<R>)` containing the closure output if evaluated, or `None` if `result` was `Err`.
    #[inline]
    pub fn card_panel_when_ok<T, E, R, F>(
        &mut self,
        name: impl Into<String>,
        result: Result<T, E>,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>, T) -> R,
    {
        result
            .ok()
            .map(|val| self.card_panel(name, |card| f(card, val)))
    }

    /// Conditionally executes a declarative child closure inside a styled card panel if `option` is [`Some`].
    ///
    /// Eliminates nested closure indentation pyramids by combining optional unwrapping and card panel
    /// emission into a single scope level.
    ///
    /// # Arguments
    /// * `name` - Node identifier for layout tracing and debugging.
    /// * `option` - Optional value forwarded to the card scope if present.
    /// * `f` - Child scope closure evaluated with the card container scope and unwrapped value when `option` is `Some`.
    ///
    /// # Returns
    /// `Some(InnerResponse<R>)` containing the closure output if evaluated, or `None` if `option` was `None`.
    #[inline]
    pub fn card_panel_when_some<T, R, F>(
        &mut self,
        name: impl Into<String>,
        option: Option<T>,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>, T) -> R,
    {
        option.map(|val| self.card_panel(name, |card| f(card, val)))
    }

    /// Conditionally renders a styled card panel with a standardized header bar if `result` is [`Ok`].
    ///
    /// Automatically renders [`CardHeaderDescriptor`] inside the card panel and executes the child body
    /// closure with the unwrapped success value, collapsing header configuration, conditional evaluation,
    /// and container nesting into a single pure declarative expression.
    ///
    /// # Arguments
    /// * `name` - Node identifier for layout tracing and debugging.
    /// * `desc` - Card header descriptor configuring title, icon, color, and action button.
    /// * `result` - Result value whose contained success item is forwarded to the body closure.
    /// * `f` - Child scope closure receiving the card container scope and unwrapped success value.
    ///
    /// # Returns
    /// `Some(InnerResponse<R>)` containing the body closure output if evaluated, or `None` if `result` was `Err`.
    #[inline]
    pub fn card_panel_desc_when_ok<T, E, R, F>(
        &mut self,
        name: impl Into<String>,
        desc: CardHeaderDescriptor<'_>,
        result: Result<T, E>,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>, T) -> R,
    {
        result.ok().map(|val| {
            self.card_panel(name, |card| {
                card.card_header_desc(desc);
                f(card, val)
            })
        })
    }

    /// Conditionally renders a styled card panel with a standardized header bar if `option` is [`Some`].
    ///
    /// Automatically renders [`CardHeaderDescriptor`] inside the card panel and executes the child body
    /// closure with the unwrapped value, collapsing header configuration, conditional evaluation,
    /// and container nesting into a single pure declarative expression.
    ///
    /// # Arguments
    /// * `name` - Node identifier for layout tracing and debugging.
    /// * `desc` - Card header descriptor configuring title, icon, color, and action button.
    /// * `option` - Optional value forwarded to the body closure if present.
    /// * `f` - Child scope closure receiving the card container scope and unwrapped value.
    ///
    /// # Returns
    /// `Some(InnerResponse<R>)` containing the body closure output if evaluated, or `None` if `option` was `None`.
    #[inline]
    pub fn card_panel_desc_when_some<T, R, F>(
        &mut self,
        name: impl Into<String>,
        desc: CardHeaderDescriptor<'_>,
        option: Option<T>,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>, T) -> R,
    {
        option.map(|val| {
            self.card_panel(name, |card| {
                card.card_header_desc(desc);
                f(card, val)
            })
        })
    }

    /// Conditionally executes a declarative child closure inside a styled card panel if `result` is [`Ok`],
    /// dereferencing the contained smart pointer or reference into a direct `&mut T::Target`.
    ///
    /// Essential for smart pointer handles (e.g. `RefMut<U>`) to enable disjoint field borrowing
    /// in child controls (e.g. `prop_vec3_xyz`) without caller-side `let` statements or nested `bind` closures.
    ///
    /// # Arguments
    /// * `name` - Node identifier for layout tracing and debugging.
    /// * `result` - Result value whose contained item implements [`std::ops::DerefMut`].
    /// * `f` - Child scope closure evaluated with the card container scope and direct mutable target reference.
    #[inline]
    pub fn card_panel_when_ok_mut<T, E, R, F>(
        &mut self,
        name: impl Into<String>,
        mut result: Result<T, E>,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        T: std::ops::DerefMut,
        F: FnOnce(&mut UiScope<'_>, &mut T::Target) -> R,
    {
        result
            .as_mut()
            .ok()
            .map(|val| self.card_panel(name, |card| f(card, &mut **val)))
    }

    /// Conditionally renders a styled card panel with a standardized header bar if `result` is [`Ok`],
    /// dereferencing the contained smart pointer or reference into a direct `&mut T::Target`.
    ///
    /// Essential for smart pointer handles (e.g. `RefMut<U>`) to enable disjoint field borrowing
    /// in child controls (e.g. `prop_vec3_xyz`) without caller-side `let` statements or nested `bind` closures.
    ///
    /// # Arguments
    /// * `name` - Node identifier for layout tracing and debugging.
    /// * `desc` - Card header descriptor configuring title, icon, color, and action button.
    /// * `result` - Result value whose contained item implements [`std::ops::DerefMut`].
    /// * `f` - Child scope closure evaluated with the card container scope and direct mutable target reference.
    #[inline]
    pub fn card_panel_desc_when_ok_mut<T, E, R, F>(
        &mut self,
        name: impl Into<String>,
        desc: CardHeaderDescriptor<'_>,
        mut result: Result<T, E>,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        T: std::ops::DerefMut,
        F: FnOnce(&mut UiScope<'_>, &mut T::Target) -> R,
    {
        result.as_mut().ok().map(|val| {
            let target = &mut **val;
            self.card_panel(name, |card| {
                card.card_header_desc(desc);
                f(card, target)
            })
        })
    }

    /// Constructs a standard component card header descriptor with title, fallback text icon, accent color, and `"×"` action tag.
    ///
    /// Automatically derives the semantic action tag from `title` using [`UiScope::action_tag`].
    /// Eliminates multi-line builder method cascades across all component inspector views.
    ///
    /// # Arguments
    /// * `title` - Component display title and semantic action tag source.
    /// * `icon` - Fallback unicode text icon (e.g. `"🚶"`, `"📎"`).
    /// * `color` - Accent header color.
    #[inline]
    #[must_use]
    pub fn component_card_desc(
        title: &'static str,
        icon: &'static str,
        color: iris_core::Color,
    ) -> CardHeaderDescriptor<'static> {
        Self::comp_card_desc(title, title, icon, color)
    }

    /// Constructs a standard component card header descriptor with component identifier, display title, icon, and accent color.
    ///
    /// Automatically derives the semantic `"×"` action tag from `component_name` using [`UiScope::action_tag`].
    /// Eliminates multi-line builder cascades across all component inspector views.
    ///
    /// # Arguments
    /// * `component_name` - Canonical component identifier used for semantic action tagging.
    /// * `title` - Display heading title presented in the UI.
    /// * `icon` - Fallback unicode text icon.
    /// * `color` - Accent header color.
    #[inline]
    #[must_use]
    pub fn comp_card_desc(
        component_name: &'static str,
        title: &'static str,
        icon: &'static str,
        color: iris_core::Color,
    ) -> CardHeaderDescriptor<'static> {
        CardHeaderDescriptor::with_action_tag(
            title,
            icon,
            color,
            "×",
            Self::action_tag(component_name),
        )
    }
}

impl<'a> CardHeaderDescriptor<'a> {
    /// Constructs a card header descriptor with title, fallback text icon, accent color, and an action button with semantic tag.
    ///
    /// Eliminates multi-line builder method chaining for standard component inspector cards.
    #[inline]
    #[must_use]
    pub const fn with_action_tag(
        title: &'a str,
        icon: &'a str,
        color: iris_core::Color,
        action_label: &'a str,
        action_tag: u64,
    ) -> Self {
        Self {
            title,
            icon: Some(icon),
            atlas_icon: None,
            color: Some(color),
            action_label: Some(action_label),
            action_tag: Some(action_tag),
        }
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

    #[test]
    fn test_declarative_card_panel_when_ok() {
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        let ok_val: Result<i32, &'static str> = Ok(42);
        let resp_ok = scope.card_panel_when_ok("TestOkCard", ok_val, |_card, val| val * 2);
        assert!(resp_ok.is_some());
        assert_eq!(resp_ok.unwrap().inner, 84);

        let err_val: Result<i32, &'static str> = Err("error");
        let resp_err = scope.card_panel_when_ok("TestErrCard", err_val, |_card, val| val * 2);
        assert!(resp_err.is_none());
    }

    #[test]
    fn test_declarative_card_panel_desc_when_ok() {
        use super::CardHeaderDescriptor;
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        let desc = CardHeaderDescriptor::new("Inspector");
        let ok_val: Result<&str, ()> = Ok("Payload");
        let resp = scope.card_panel_desc_when_ok("DescCard", desc, ok_val, |_card, val| {
            assert_eq!(val, "Payload");
            val.len()
        });
        assert_eq!(resp.unwrap().inner, 7);
    }

    #[test]
    fn test_component_card_desc_construction() {
        use iris_core::Color;
        let desc = UiScope::component_card_desc("TestComp", "🧪", Color::rgba(1.0, 0.0, 0.0, 1.0));
        assert_eq!(desc.title, "TestComp");
        assert_eq!(desc.icon, Some("🧪"));
        assert_eq!(desc.action_label, Some("×"));
        assert_eq!(desc.action_tag, Some(UiScope::action_tag("TestComp")));
    }
}