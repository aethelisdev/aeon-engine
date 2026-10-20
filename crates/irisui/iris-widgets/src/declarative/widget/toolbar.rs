// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Toolbar & ToolbarGroup Container Builders
//!
//! Provides the configurable [`Toolbar`] floating pill container builder and [`ToolbarGroup`]
//! modular section builders for assembling desktop application tool strips, segmented mode
//! selectors, and interactive utility toolbars without engine-specific coupling.
//!

use crate::declarative::scope::containers::generic::InnerResponse;
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use crate::declarative::widget::Widget;
use iris_core::{AlignItems, Color, CornerRadii, Insets, Style, WidgetRole};
use std::ops::{Deref, DerefMut};

/// Configurable builder for horizontal toolbar strips and floating pill toolbars.
///
/// Supports absolute screen anchoring, pill glass styling, customizable heights and gaps,
/// and nested modular [`ToolbarGroup`] sections.
#[derive(Debug, Clone)]
pub struct Toolbar {
    name: &'static str,
    position: Option<(f32, f32)>,
    width: Option<f32>,
    height: f32,
    gap: f32,
    tag: u64,
    background: Option<Color>,
    border_color: Option<Color>,
    corner_radius: Option<f32>,
    is_pill: bool,
    is_transparent: bool,
}

impl Toolbar {
    /// Creates a new inline toolbar container builder with a descriptive debug name.
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            position: None,
            width: None,
            height: 32.0,
            gap: 8.0,
            tag: 0,
            background: None,
            border_color: None,
            corner_radius: None,
            is_pill: false,
            is_transparent: false,
        }
    }

    /// Creates a floating toolbar container absolutely positioned at `(x, y)` in logical points.
    #[inline]
    pub fn floating(name: &'static str, x: f32, y: f32) -> Self {
        Self {
            name,
            position: Some((x, y)),
            width: None,
            height: 32.0,
            gap: 8.0,
            tag: 0,
            background: None,
            border_color: None,
            corner_radius: None,
            is_pill: false,
            is_transparent: false,
        }
    }

    /// Configures an explicit width in logical points for the toolbar container.
    #[inline]
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Configures the toolbar container height in logical points.
    #[inline]
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Configures the gap spacing between nested toolbar groups and standalone tool buttons.
    #[inline]
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Sets a persistent 64-bit semantic tag for interaction tracking.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Applies rounded pill glass styling with subtle translucent background, border, and elevation shadow.
    #[inline]
    #[must_use]
    pub fn pill(mut self) -> Self {
        self.is_pill = true;
        self
    }

    /// Configures the toolbar to have a transparent background and no outer border (ideal for floating strips).
    #[inline]
    #[must_use]
    pub fn transparent(mut self) -> Self {
        self.is_transparent = true;
        self
    }

    /// Sets an explicit background fill color for the toolbar container.
    #[inline]
    #[must_use]
    pub fn background(mut self, bg: Color) -> Self {
        self.background = Some(bg);
        self
    }

    /// Sets an explicit border outline color for the toolbar container.
    #[inline]
    #[must_use]
    pub fn border_color(mut self, border: Color) -> Self {
        self.border_color = Some(border);
        self
    }

    /// Sets explicit corner radii for the toolbar container.
    #[inline]
    #[must_use]
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Renders the toolbar container and provides a specialized [`ToolbarScope`] to child closures.
    pub fn show<R>(
        self,
        scope: &mut UiScope<'_>,
        f: impl FnOnce(&mut ToolbarScope<'_, '_>) -> R,
    ) -> InnerResponse<R> {
        let theme = scope.theme();
        let mut style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(self.height)
            .gap(self.gap);

        if let Some((x, y)) = self.position {
            style = style.position_absolute().left(x).top(y);
        }

        if let Some(w) = self.width {
            style = style.width(w);
        }

        if self.is_transparent {
            // Transparent floating strip: no background or border
        } else if self.is_pill {
            let bg = self
                .background
                .unwrap_or_else(|| Color::rgba(0.12, 0.13, 0.16, 0.92));
            let border = self
                .border_color
                .unwrap_or_else(|| Color::rgba(0.24, 0.26, 0.32, 0.85));
            let radius = self.corner_radius.unwrap_or(4.0);
            style = style
                .background(bg)
                .border(1.0, border)
                .corner_radii(CornerRadii::all(radius))
                .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35));
        } else if let Some(bg) = self.background {
            style = style.background(bg);
            if let Some(border) = self.border_color {
                style = style.border(1.0, border);
            }
            if let Some(radius) = self.corner_radius {
                style = style.corner_radii(CornerRadii::all(radius));
            }
        } else {
            style = style
                .background(theme.toolbar_bg)
                .border(1.0, theme.toolbar_border)
                .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0));
        }

        let mut result = None;
        let response = scope.container_tagged(
            self.name,
            style,
            WidgetRole::Default,
            self.tag,
            |child_scope| {
                let mut tb_scope = ToolbarScope {
                    scope: child_scope,
                    height: self.height,
                };
                result = Some(f(&mut tb_scope));
            },
        );

        InnerResponse::new(
            response.response,
            result.expect("Toolbar content closure must execute synchronously"),
        )
    }
}

/// Specialized declarative scope providing toolbar item builders, modular sub-groups, and tool buttons.
pub struct ToolbarScope<'a, 's> {
    scope: &'a mut UiScope<'s>,
    height: f32,
}

impl<'a, 's> Deref for ToolbarScope<'a, 's> {
    type Target = UiScope<'s>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.scope
    }
}

impl<'a, 's> DerefMut for ToolbarScope<'a, 's> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.scope
    }
}

impl<'a, 's> ToolbarScope<'a, 's> {
    /// Returns a mutable reference to the underlying [`UiScope`].
    #[inline]
    pub fn scope_mut(&mut self) -> &mut UiScope<'s> {
        self.scope
    }

    /// Emits a modular sub-group container (such as a segmented pill group or tool cluster).
    ///
    /// Configured with subtle framing, rounded corners, and centered flex row alignment.
    pub fn group<R>(
        &mut self,
        f: impl FnOnce(&mut ToolbarGroupScope<'_, '_>) -> R,
    ) -> InnerResponse<R> {
        ToolbarGroup::new().show(self.scope, self.height, f)
    }

    /// Emits a modular sub-group container with an explicit gap between nested items.
    pub fn group_with_gap<R>(
        &mut self,
        gap: f32,
        f: impl FnOnce(&mut ToolbarGroupScope<'_, '_>) -> R,
    ) -> InnerResponse<R> {
        ToolbarGroup::new()
            .gap(gap)
            .show(self.scope, self.height, f)
    }

    /// Emits a customized sub-group container using an explicit [`ToolbarGroup`] builder.
    pub fn group_styled<R>(
        &mut self,
        group: ToolbarGroup,
        f: impl FnOnce(&mut ToolbarGroupScope<'_, '_>) -> R,
    ) -> InnerResponse<R> {
        group.show(self.scope, self.height, f)
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

    /// Emits a vertical divider line between toolbar sections.
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

    /// Adds any generic widget implementing [`Widget`] to the toolbar.
    #[inline]
    pub fn add<W: Widget>(&mut self, widget: W) -> W::Response {
        self.scope.add(widget)
    }
}

/// Configurable builder for modular toolbar sub-groups.
#[derive(Debug, Clone)]
pub struct ToolbarGroup {
    width: Option<f32>,
    gap: f32,
    background: Option<Color>,
    border_color: Option<Color>,
    corner_radius: f32,
    is_transparent: bool,
}

impl Default for ToolbarGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolbarGroup {
    /// Creates a new modular toolbar sub-group with default dark pill styling.
    #[inline]
    pub fn new() -> Self {
        Self {
            width: None,
            gap: 4.0,
            background: None,
            border_color: None,
            corner_radius: 4.0,
            is_transparent: false,
        }
    }

    /// Configures an explicit width in logical points for this toolbar group.
    #[inline]
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Configures the group to have a transparent background and no border.
    #[inline]
    #[must_use]
    pub fn transparent(mut self) -> Self {
        self.is_transparent = true;
        self
    }

    /// Configures the gap spacing between items in this group.
    #[inline]
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Configures the background color of this group.
    #[inline]
    #[must_use]
    pub fn background(mut self, bg: Color) -> Self {
        self.background = Some(bg);
        self
    }

    /// Configures the border color of this group.
    #[inline]
    #[must_use]
    pub fn border_color(mut self, border: Color) -> Self {
        self.border_color = Some(border);
        self
    }

    /// Configures the corner radius of this group container.
    #[inline]
    #[must_use]
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Renders this modular sub-group inside the parent scope.
    pub fn show<R>(
        self,
        scope: &mut UiScope<'_>,
        parent_height: f32,
        f: impl FnOnce(&mut ToolbarGroupScope<'_, '_>) -> R,
    ) -> InnerResponse<R> {
        let mut style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(parent_height)
            .gap(self.gap);

        if let Some(w) = self.width {
            style = style.width(w);
        }

        if self.is_transparent {
            // Transparent group: no background, border, or shadow
        } else {
            let bg = self
                .background
                .unwrap_or_else(|| Color::rgba(0.12, 0.13, 0.16, 0.92));
            let border = self
                .border_color
                .unwrap_or_else(|| Color::rgba(0.24, 0.26, 0.32, 0.85));
            style = style
                .background(bg)
                .border(1.0, border)
                .corner_radii(CornerRadii::all(self.corner_radius))
                .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35));
        }

        let mut result = None;
        let response = scope.container_named("ToolbarGroup", style, |group_scope| {
            let mut g_scope = ToolbarGroupScope {
                scope: group_scope,
                height: parent_height,
            };
            result = Some(f(&mut g_scope));
        });

        InnerResponse::new(
            response.response,
            result.expect("ToolbarGroup content closure must execute synchronously"),
        )
    }
}

/// Scope provided inside a [`ToolbarGroup`] container.
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

    /// Adds any generic widget implementing [`Widget`] to this group.
    #[inline]
    pub fn add<W: Widget>(&mut self, widget: W) -> W::Response {
        self.scope.add(widget)
    }
}