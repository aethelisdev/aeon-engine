// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Collapsing Headers & Accordion Sections
//!
//! Provides egui-style autonomous collapsible containers on [`UiScope`].
//! Features persistent expanded/collapsed state tracking via [`irisui_core::UiTree::collapsing_state`],
//! animated/dynamic chevron arrows (`▶` / `▼`), hover/press styling, and immediate effective state
//! evaluation with zero per-frame allocation.

use super::core::UiScope;
use crate::declarative::types::{WidgetResponse, split_label_id};
use irisui_core::{
    AlignItems, Color, CornerRadii, Insets, JustifyContent, Style, TextAlign, UiTag, WidgetCursor,
    WidgetRole,
};

/// Response returned when emitting an autonomous collapsing section.
///
/// Encapsulates the interaction response of the header row alongside the optional
/// return value of the inner body closure, available only when the section is expanded.
#[derive(Debug, Clone)]
pub struct CollapsingResponse<R> {
    /// Interaction response corresponding to the collapsing header button/row.
    pub header_response: WidgetResponse,
    /// Return value of the inner body closure, populated when the section is expanded.
    pub body_response: Option<R>,
    /// Whether the collapsing section is currently open and expanded.
    pub is_open: bool,
}

impl<R> CollapsingResponse<R> {
    /// Returns `true` if the collapsing section is currently open and expanded.
    #[inline]
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns `true` if the collapsing header was clicked during the active frame.
    #[inline]
    #[must_use]
    pub fn clicked(&self) -> bool {
        self.header_response.clicked()
    }

    /// Returns `true` if the mouse cursor is currently hovering over the collapsing header.
    #[inline]
    #[must_use]
    pub fn hovered(&self) -> bool {
        self.header_response.hovered()
    }

    /// Returns a reference to the header widget response.
    #[inline]
    #[must_use]
    pub const fn header_response(&self) -> &WidgetResponse {
        &self.header_response
    }

    /// Returns a reference to the inner body closure result, if the section was expanded.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> Option<&R> {
        self.body_response.as_ref()
    }

    /// Consumes the response and extracts the inner body closure result, if present.
    #[inline]
    pub fn into_inner(self) -> Option<R> {
        self.body_response
    }
}

/// Fluent builder for assembling and customizing autonomous collapsing sections.
pub struct CollapsingBuilder<'a, 'b> {
    scope: &'a mut UiScope<'b>,
    title: String,
    tag: Option<u64>,
    default_open: bool,
    indent: f32,
}

impl<'a, 'b> CollapsingBuilder<'a, 'b> {
    /// Constructs a new collapsing section builder with the given header title.
    #[inline]
    pub fn new(scope: &'a mut UiScope<'b>, title: impl Into<String>) -> Self {
        Self {
            scope,
            title: title.into(),
            tag: None,
            default_open: false,
            indent: 14.0,
        }
    }

    /// Explicitly assigns a 64-bit persistent semantic tag instead of hashing `title`.
    #[inline]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Sets whether this section defaults to open/expanded if it has never been toggled.
    #[inline]
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    /// Sets the left indentation width (in logical pixels) for the inner body scope.
    /// Defaults to `14.0px`.
    #[inline]
    pub fn indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    /// Displays the collapsing header and evaluates `add_contents` if expanded.
    pub fn show<R>(
        self,
        add_contents: impl FnOnce(&mut UiScope<'_>) -> R,
    ) -> CollapsingResponse<R> {
        let (visible_title, _) = split_label_id(&self.title);
        let tag = self.tag.unwrap_or_else(|| self.scope.tag_for(&self.title));

        let is_open = self.scope.tree.is_collapsing_open(tag, self.default_open);

        // 1. Construct interactive header row node
        let header_id = self.scope.tree.create_node();
        let _ = self.scope.tree.add_child(self.scope.parent, header_id);
        if let Some(node) = self.scope.tree.get_mut(header_id) {
            node.set_name("CollapsingHeader");
            node.tag = tag;
            node.role = WidgetRole::Button;
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Pointer);
        }

        let (clicked, hovered, _) = self.scope.check_interaction(header_id);
        let pressed = self.scope.is_pressed(header_id);

        let effective_open = is_open ^ clicked;
        if clicked {
            self.scope.tree.set_collapsing_open(tag, effective_open);
        }

        if let Some(node) = self.scope.tree.get_mut(header_id) {
            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .gap(6.0)
                .padding_insets(Insets::new(3.0, 6.0, 3.0, 6.0))
                .border_radius(4.0);
            style.hover_background = Some(Color::rgba(0.24, 0.27, 0.35, 0.35));
            if pressed {
                style.background_color = Color::rgba(0.18, 0.20, 0.26, 0.60);
            } else if hovered && let Some(hbg) = style.hover_background {
                style.background_color = hbg;
            }
            node.set_style(style);
        }

        // Child 1: Foldout chevron arrow indicator (▶ collapsed, ▼ expanded)
        let chevron_id = self.scope.tree.create_node();
        let _ = self.scope.tree.add_child(header_id, chevron_id);
        if let Some(ch_node) = self.scope.tree.get_mut(chevron_id) {
            ch_node.set_name("CollapsingChevron");
            ch_node.interactive = false;
            ch_node.role = WidgetRole::Default;
            ch_node.font_size = 9.5;
            ch_node.line_height = 14.0;
            ch_node.text_align = TextAlign::Center;
            if effective_open {
                ch_node.set_text("▼");
                ch_node.text_color = Color::rgba(0.90, 0.92, 0.98, 1.0);
            } else {
                ch_node.set_text("▶");
                ch_node.text_color = if hovered || pressed {
                    Color::WHITE
                } else {
                    Color::rgba(0.60, 0.64, 0.72, 1.0)
                };
            }
            ch_node.set_style(
                Style::new()
                    .width(14.0)
                    .height(14.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center),
            );
        }

        // Child 2: Header title label
        let title_id = self.scope.tree.create_node();
        let _ = self.scope.tree.add_child(header_id, title_id);
        if let Some(t_node) = self.scope.tree.get_mut(title_id) {
            t_node.set_name("CollapsingTitle");
            t_node.interactive = false;
            t_node.role = WidgetRole::Default;
            t_node.font_size = 11.5;
            t_node.line_height = 14.0;
            t_node.text_align = TextAlign::Left;
            t_node.text_color = if effective_open || hovered || pressed {
                Color::WHITE
            } else {
                Color::rgba(0.82, 0.85, 0.90, 1.0)
            };
            t_node.set_text(visible_title);
            t_node.set_style(Style::new());
        }

        let mut header_response = WidgetResponse::new(header_id, clicked, hovered, clicked);
        if pressed {
            header_response = header_response.with_pressed(true);
        }

        // 2. If open, construct indented body container and execute child closure
        let body_response = if effective_open {
            let body_id = self.scope.tree.create_node();
            let _ = self.scope.tree.add_child(self.scope.parent, body_id);
            if let Some(b_node) = self.scope.tree.get_mut(body_id) {
                b_node.set_name("CollapsingBody");
                b_node.role = WidgetRole::Default;
                b_node.set_style(Style::new().flex_col().gap(4.0).padding_insets(Insets::new(
                    2.0,
                    4.0,
                    4.0,
                    self.indent,
                )));
            }
            let body_seed = crate::declarative::types::combine_seeds(self.scope.seed, tag);
            let mut body_scope = self.scope.child_scope_seeded(body_id, body_seed);
            Some(add_contents(&mut body_scope))
        } else {
            None
        };

        CollapsingResponse {
            header_response,
            body_response,
            is_open: effective_open,
        }
    }
}

/// Fluent builder for assembling and customizing autonomous collapsible card sections.
///
/// Renders a themed card container with a `34px` interactive header bar, dynamic chevron
/// indicator (`▶` / `▼`), crisp divider line, and collapsible content body. Persistent
/// expansion state is stored inside [`irisui_core::UiTree::collapsing_state`].
pub struct CollapsingCardBuilder<'a, 'b> {
    scope: &'a mut UiScope<'b>,
    title: String,
    tag: Option<u64>,
    default_open: bool,
    bottom_margin: f32,
}

impl<'a, 'b> CollapsingCardBuilder<'a, 'b> {
    /// Constructs a new collapsible card builder with the given header title.
    #[inline]
    pub fn new(scope: &'a mut UiScope<'b>, title: impl Into<String>) -> Self {
        Self {
            scope,
            title: title.into(),
            tag: None,
            default_open: true,
            bottom_margin: 14.0,
        }
    }

    /// Explicitly assigns a 64-bit persistent semantic tag instead of hashing `title`.
    #[inline]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Sets whether this card defaults to expanded if it has never been toggled.
    #[inline]
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    /// Sets the bottom spacing margin below the card container in logical pixels (default `14.0`).
    #[inline]
    pub fn bottom_margin(mut self, bottom_margin: f32) -> Self {
        self.bottom_margin = bottom_margin;
        self
    }

    /// Displays the collapsible card and evaluates `add_contents` inside the padded card body when open.
    pub fn show<R>(
        self,
        add_contents: impl FnOnce(&mut UiScope<'_>) -> R,
    ) -> CollapsingResponse<R> {
        let (visible_title, _) = split_label_id(&self.title);
        let tag = self.tag.unwrap_or_else(|| self.scope.tag_for(&self.title));
        let is_open = self.scope.tree.is_collapsing_open(tag, self.default_open);

        let card_bg = self.scope.theme().card_bg;
        let card_border = self.scope.theme().card_border;
        let header_bg = self.scope.theme().card_header_bg;
        let hover_bg = header_bg.lerp(Color::WHITE, 0.08);

        let card_id = self.scope.tree.create_node();
        let _ = self.scope.tree.add_child(self.scope.parent, card_id);
        if let Some(c_node) = self.scope.tree.get_mut(card_id) {
            c_node.set_name("CollapsingCard");
            c_node.role = WidgetRole::Default;
            c_node.set_style(
                Style::new()
                    .flex_col()
                    .background(card_bg)
                    .border(1.0, card_border)
                    .border_radius(6.0)
                    .margin_insets(Insets::new(0.0, 0.0, self.bottom_margin, 0.0)),
            );
        }

        let header_id = self.scope.tree.create_node();
        let _ = self.scope.tree.add_child(card_id, header_id);
        if let Some(h_node) = self.scope.tree.get_mut(header_id) {
            h_node.set_name("CollapsingCardHeader");
            h_node.tag = tag;
            h_node.role = WidgetRole::Button;
            h_node.interactive = true;
            h_node.cursor = Some(WidgetCursor::Pointer);
        }

        let (clicked, hovered, _) = self.scope.check_interaction(header_id);
        let pressed = self.scope.is_pressed(header_id);

        let effective_open = is_open ^ clicked;
        if clicked {
            self.scope.tree.set_collapsing_open(tag, effective_open);
        }

        if let Some(h_node) = self.scope.tree.get_mut(header_id) {
            let radii = if effective_open {
                CornerRadii::new(5.0, 5.0, 0.0, 0.0)
            } else {
                CornerRadii::all(5.0)
            };
            h_node.set_style(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .height(34.0)
                    .padding_insets(Insets::new(0.0, 12.0, 0.0, 12.0))
                    .background(if hovered || pressed {
                        hover_bg
                    } else {
                        header_bg
                    })
                    .hover_background(hover_bg)
                    .corner_radii(radii),
            );
        }

        let chevron_id = self.scope.tree.create_node();
        let _ = self.scope.tree.add_child(header_id, chevron_id);
        if let Some(ch_node) = self.scope.tree.get_mut(chevron_id) {
            ch_node.set_name("CollapsingCardChevron");
            ch_node.interactive = false;
            ch_node.role = WidgetRole::Default;
            ch_node.font_size = 10.0;
            ch_node.line_height = 14.0;
            ch_node.text_align = TextAlign::Left;
            ch_node.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
            ch_node.set_text(if effective_open { "▼ " } else { "▶ " });
            ch_node.set_style(Style::new().margin_insets(Insets::new(0.0, 8.0, 0.0, 0.0)));
        }

        let title_id = self.scope.tree.create_node();
        let _ = self.scope.tree.add_child(header_id, title_id);
        if let Some(t_node) = self.scope.tree.get_mut(title_id) {
            t_node.set_name("CollapsingCardTitle");
            t_node.interactive = false;
            t_node.role = WidgetRole::Default;
            t_node.font_size = 12.5;
            t_node.line_height = 16.0;
            t_node.text_align = TextAlign::Left;
            t_node.text_color = if hovered || pressed {
                Color::WHITE
            } else {
                Color::rgba(0.92, 0.94, 0.98, 1.0)
            };
            t_node.hover_text_color = Some(Color::WHITE);
            t_node.set_text(visible_title);
            t_node.set_style(Style::new().flex_grow(1.0));
        }

        let mut header_response = WidgetResponse::new(header_id, clicked, hovered, clicked);
        if pressed {
            header_response = header_response.with_pressed(true);
        }

        let body_response = if effective_open {
            let mut card_scope = self.scope.child_scope(card_id);
            card_scope.divider(card_border);

            let body_id = self.scope.tree.create_node();
            let _ = self.scope.tree.add_child(card_id, body_id);
            if let Some(b_node) = self.scope.tree.get_mut(body_id) {
                b_node.set_name("CollapsingCardBody");
                b_node.role = WidgetRole::Default;
                b_node.set_style(
                    Style::new()
                        .flex_col()
                        .gap(2.0)
                        .padding_insets(Insets::new(8.0, 12.0, 10.0, 12.0)),
                );
            }
            let mut body_scope = self.scope.child_scope(body_id);
            Some(add_contents(&mut body_scope))
        } else {
            None
        };

        CollapsingResponse {
            header_response,
            body_response,
            is_open: effective_open,
        }
    }
}

impl<'a> UiScope<'a> {
    /// Emits an autonomous collapsible accordion section using persistent state tracking.
    ///
    /// Automatically manages open/closed states across frames using a persistent 64-bit tag derived from `title`.
    /// Features foldout chevron arrows (`▶` / `▼`), hover highlighting, and zero-allocation immediate state toggling.
    ///
    /// # Arguments
    /// * `title` - Descriptive caption displayed on the collapsing header.
    /// * `add_contents` - Closure invoked to populate children within the indented body scope when open.
    ///
    /// # Example
    /// ```rust,no_run
    /// use irisui_widgets::UiScope;
    ///
    /// fn render(scope: &mut UiScope<'_>) {
    ///     scope.collapsing("Advanced Settings", |body| {
    ///         body.label("Internal debug options");
    ///     });
    /// }
    /// ```
    pub fn collapsing<R>(
        &mut self,
        title: impl Into<String>,
        add_contents: impl FnOnce(&mut UiScope<'_>) -> R,
    ) -> CollapsingResponse<R> {
        CollapsingBuilder::new(self, title).show(add_contents)
    }

    /// Emits an autonomous collapsing section that defaults to open/expanded on initial view.
    pub fn collapsing_default_open<R>(
        &mut self,
        title: impl Into<String>,
        add_contents: impl FnOnce(&mut UiScope<'_>) -> R,
    ) -> CollapsingResponse<R> {
        CollapsingBuilder::new(self, title)
            .default_open(true)
            .show(add_contents)
    }

    /// Returns a fluent [`CollapsingBuilder`] to configure custom indentation, tags, or default expansion.
    #[inline]
    pub fn collapsing_builder(&mut self, title: impl Into<String>) -> CollapsingBuilder<'_, 'a> {
        CollapsingBuilder::new(self, title)
    }

    /// Emits an autonomous collapsible card section with themed card chrome, header bar, divider, and body.
    pub fn collapsing_card<R>(
        &mut self,
        title: impl Into<String>,
        add_contents: impl FnOnce(&mut UiScope<'_>) -> R,
    ) -> CollapsingResponse<R> {
        CollapsingCardBuilder::new(self, title).show(add_contents)
    }

    /// Returns a fluent [`CollapsingCardBuilder`] to configure semantic tag, default expansion, or bottom margin.
    #[inline]
    pub fn collapsing_card_builder(
        &mut self,
        title: impl Into<String>,
    ) -> CollapsingCardBuilder<'_, 'a> {
        CollapsingCardBuilder::new(self, title)
    }

    /// Emits an autonomous collapsible card section keyed directly by a persistent [`UiTag`] or `&str`.
    ///
    /// Stores and evaluates open/collapsed state internally within [`irisui_core::UiTree`], eliminating
    /// external host-side `HashSet` state bags and manual bit-encoding shims.
    ///
    /// # Arguments
    /// * `key` - Persistent tag or static string key identifying the section state.
    /// * `title` - Visible header caption displayed in the collapsible card titlebar.
    /// * `default_open` - Initial expanded state when no prior interaction has been recorded.
    /// * `add_contents` - Body scope closure evaluated only when the section is expanded.
    pub fn collapsing_section<R>(
        &mut self,
        key: impl Into<UiTag>,
        title: impl Into<String>,
        default_open: bool,
        add_contents: impl FnOnce(&mut UiScope<'_>) -> R,
    ) -> CollapsingResponse<R> {
        self.collapsing_card_builder(title)
            .tag(key.into().raw())
            .default_open(default_open)
            .show(add_contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::UiTree;

    #[test]
    fn test_collapsing_card_toggles_body_and_persists_state() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root creation must succeed");
        let card_tag = 0xCAFE_1234_5678_9ABCu64;

        // Pass 1: Default collapsed -> body closure does not execute
        let resp_collapsed = {
            let mut scope = UiScope::new(&mut tree, root);
            scope
                .collapsing_card_builder("Section A")
                .tag(card_tag)
                .default_open(false)
                .show(|body| {
                    body.label("Inner");
                    42
                })
        };
        assert!(!resp_collapsed.is_open());
        assert_eq!(resp_collapsed.body(), None);

        // Pass 2: Simulated click on header toggles immediately to open
        tree.event_state.frame_clicked_id = Some(resp_collapsed.header_response().id);
        tree.event_state.frame_clicked_tag = Some(card_tag);
        let root2 = tree.create_node();
        let _ = tree.add_child(root, root2);
        let resp_opened = {
            let mut scope = UiScope::new(&mut tree, root2);
            scope
                .collapsing_card_builder("Section A")
                .tag(card_tag)
                .default_open(false)
                .show(|body| {
                    body.label("Inner");
                    42
                })
        };
        assert!(resp_opened.is_open());
        assert_eq!(resp_opened.body(), Some(&42));
        assert!(tree.is_collapsing_open(card_tag, false));
    }

    #[test]
    fn test_collapsing_section_uses_semantic_key_without_external_hashset() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root creation must succeed");
        let section_key = "graphics_shadows";

        let first = {
            let mut scope = UiScope::new(&mut tree, root);
            scope.collapsing_section(section_key, "Shadows", true, |body| {
                body.label("Bias");
                7
            })
        };
        assert!(first.is_open());
        assert_eq!(first.body(), Some(&7));

        tree.event_state.frame_clicked_id = Some(first.header_response().id);
        tree.event_state.frame_clicked_tag = Some(UiTag::from(section_key).raw());
        let root2 = tree.create_node();
        let _ = tree.add_child(root, root2);
        let second = {
            let mut scope = UiScope::new(&mut tree, root2);
            scope.collapsing_section(section_key, "Shadows", true, |body| {
                body.label("Bias");
                7
            })
        };
        assert!(!second.is_open());
        assert_eq!(second.body(), None);
        assert!(!tree.is_collapsing_open(UiTag::from(section_key).raw(), true));
    }
}