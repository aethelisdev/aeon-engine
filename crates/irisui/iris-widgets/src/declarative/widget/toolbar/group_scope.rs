// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # ToolbarGroup Scope
//!
//! Provides the specialized [`ToolbarGroupScope`] builder context for nested tool clusters.
//!

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use crate::declarative::widget::Widget;
use crate::declarative::widget::toolbar_button::ToolbarButton;
use iris_core::{Color, CornerRadii};
use std::ops::{Deref, DerefMut};

/// Scope provided inside a [`ToolbarGroup`](super::group::ToolbarGroup) container.
pub struct ToolbarGroupScope<'a, 's> {
    scope: &'a mut UiScope<'s>,
    height: f32,
}

impl<'a, 's> Deref for ToolbarGroupScope<'a, 's> {
    type Target = UiScope<'s>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.scope
    }
}

impl<'a, 's> DerefMut for ToolbarGroupScope<'a, 's> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.scope
    }
}

impl<'a, 's> ToolbarGroupScope<'a, 's> {
    /// Constructs a new [`ToolbarGroupScope`] wrapping the given parent [`UiScope`] and container height.
    #[inline]
    pub fn new(scope: &'a mut UiScope<'s>, height: f32) -> Self {
        Self { scope, height }
    }

    /// Emits a square 32×32 pixel icon tool button with hover, active, and pointer styling.
    #[inline]
    pub fn icon_button(&mut self, icon_uv: [f32; 4], is_active: bool) -> WidgetResponse {
        self.icon_button_tagged(icon_uv, 0, is_active)
    }

    /// Emits a square 32×32 pixel icon tool button with an explicit semantic tag.
    #[inline]
    pub fn icon_button_tagged(
        &mut self,
        icon_uv: [f32; 4],
        tag: u64,
        is_active: bool,
    ) -> WidgetResponse {
        self.scope.toolbar_icon_button(icon_uv, tag, is_active)
    }

    /// Emits a mode button displaying an optional icon and text caption with auto-sizing padding.
    #[inline]
    pub fn mode_button(
        &mut self,
        icon_uv: Option<[f32; 4]>,
        label: impl Into<String>,
        is_open: bool,
        radii: CornerRadii,
    ) -> WidgetResponse {
        self.mode_button_tagged(icon_uv, label, 0, is_open, 0.0, radii)
    }

    /// Emits a mode button displaying an optional icon and text caption with an explicit width.
    #[inline]
    pub fn mode_button_with_width(
        &mut self,
        icon_uv: Option<[f32; 4]>,
        label: impl Into<String>,
        is_open: bool,
        width: f32,
        radii: CornerRadii,
    ) -> WidgetResponse {
        self.mode_button_tagged(icon_uv, label, 0, is_open, width, radii)
    }

    /// Emits a mode button displaying an optional icon and text caption with a semantic tag and explicit width.
    #[inline]
    pub fn mode_button_tagged(
        &mut self,
        icon_uv: Option<[f32; 4]>,
        label: impl Into<String>,
        tag: u64,
        is_open: bool,
        width: f32,
        radii: CornerRadii,
    ) -> WidgetResponse {
        self.scope
            .toolbar_mode_button(icon_uv, label, tag, is_open, width, radii)
    }

    /// Emits a vertical divider line inside this group.
    #[inline]
    pub fn divider(&mut self) {
        self.scope
            .vertical_divider(self.height - 10.0, Color::rgba(1.0, 1.0, 1.0, 0.12));
    }

    /// Emits a declarative radio tool button returning a [`WidgetResponse`] with fluent `.on_click` chaining.
    #[inline]
    pub fn selectable_tool(&mut self, is_selected: bool, icon_uv: [f32; 4]) -> WidgetResponse {
        self.selectable_tool_tagged(is_selected, icon_uv, 0)
    }

    /// Emits a declarative radio tool button with a semantic tag returning a [`WidgetResponse`].
    #[inline]
    pub fn selectable_tool_tagged(
        &mut self,
        is_selected: bool,
        icon_uv: [f32; 4],
        tag: u64,
    ) -> WidgetResponse {
        self.icon_button_tagged(icon_uv, tag, is_selected)
    }

    /// Emits a declarative radio tool button that automatically binds to mutable state.
    ///
    /// When clicked, mutates `selected` to `value` with zero manual interaction boilerplate.
    pub fn selectable_tool_mut<T: PartialEq + Copy>(
        &mut self,
        selected: &mut T,
        value: T,
        icon_uv: [f32; 4],
        tag: u64,
    ) -> WidgetResponse {
        let is_active = *selected == value;
        let resp = self.icon_button_tagged(icon_uv, tag, is_active);
        if resp.clicked() {
            *selected = value;
        }
        resp
    }

    /// Emits a declarative toggle tool button returning a [`WidgetResponse`] with fluent `.on_click` chaining.
    #[inline]
    pub fn toggle_tool(&mut self, is_active: bool, icon_uv: [f32; 4]) -> WidgetResponse {
        self.toggle_tool_tagged(is_active, icon_uv, 0)
    }

    /// Emits a declarative toggle tool button with a semantic tag returning a [`WidgetResponse`].
    #[inline]
    pub fn toggle_tool_tagged(
        &mut self,
        is_active: bool,
        icon_uv: [f32; 4],
        tag: u64,
    ) -> WidgetResponse {
        self.icon_button_tagged(icon_uv, tag, is_active)
    }

    /// Emits a declarative toggle tool button that automatically inverts a boolean state when clicked.
    pub fn toggle_tool_mut(
        &mut self,
        target: &mut bool,
        icon_active: [f32; 4],
        icon_inactive: [f32; 4],
        tag: u64,
    ) -> WidgetResponse {
        let is_active = *target;
        let uv = if is_active {
            icon_active
        } else {
            icon_inactive
        };
        let resp = self.icon_button_tagged(uv, tag, is_active);
        if resp.clicked() {
            *target = !*target;
        }
        resp
    }

    /// Emits a declarative text toolbar button returning a [`WidgetResponse`].
    ///
    /// Enables fluent `.on_click(...)` chaining without manual event boilerplate.
    #[inline]
    pub fn button(&mut self, label: impl Into<String>) -> WidgetResponse {
        self.scope.add(ToolbarButton::new(label))
    }

    /// Emits a declarative text toolbar button with a semantic tag returning a [`WidgetResponse`].
    #[inline]
    pub fn button_tagged(&mut self, label: impl Into<String>, tag: u64) -> WidgetResponse {
        self.scope.add(ToolbarButton::new(label).tag(tag))
    }

    /// Emits a declarative text toolbar button with an explicit width constraint.
    #[inline]
    pub fn button_with_width(&mut self, label: impl Into<String>, width: f32) -> WidgetResponse {
        self.scope.add(ToolbarButton::new(label).width(width))
    }

    /// Emits a declarative text toolbar button with an explicit width and semantic tag.
    #[inline]
    pub fn button_tagged_with_width(
        &mut self,
        label: impl Into<String>,
        tag: u64,
        width: f32,
    ) -> WidgetResponse {
        self.scope
            .add(ToolbarButton::new(label).tag(tag).width(width))
    }

    /// Emits a standalone toolbar dropdown button displaying a text caption with full framing.
    #[inline]
    pub fn dropdown_button_tagged(
        &mut self,
        label: impl Into<String>,
        tag: u64,
        is_open: bool,
        width: f32,
    ) -> WidgetResponse {
        self.scope
            .toolbar_dropdown_button(label, tag, is_open, width)
    }

    /// Emits a declarative toggle toolbar button reflecting active state.
    #[inline]
    pub fn toggle_button(&mut self, label: impl Into<String>, is_active: bool) -> WidgetResponse {
        self.scope.add(ToolbarButton::new(label).active(is_active))
    }

    /// Emits a declarative toggle toolbar button with a semantic tag reflecting active state.
    #[inline]
    pub fn toggle_button_tagged(
        &mut self,
        label: impl Into<String>,
        tag: u64,
        is_active: bool,
    ) -> WidgetResponse {
        self.scope
            .add(ToolbarButton::new(label).tag(tag).active(is_active))
    }

    /// Emits a declarative toggle toolbar button that automatically inverts a boolean state when clicked.
    pub fn toggle_button_mut(
        &mut self,
        label: impl Into<String>,
        target: &mut bool,
    ) -> WidgetResponse {
        let is_active = *target;
        let resp = self.toggle_button(label, is_active);
        if resp.clicked() {
            *target = !*target;
        }
        resp
    }

    /// Adds any generic widget implementing [`Widget`] to this group.
    #[inline]
    pub fn add<W: Widget>(&mut self, widget: W) -> W::Response {
        self.scope.add(widget)
    }
}