// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Modal Dialogues & Floating Overlay Primitives
//!
//! Provides hardware-accelerated GPU SDF modal primitives on [`UiScope`]: full-screen
//! backdrop scrim blockers, elevated floating dialog cards, modern `✕` close buttons,
//! primary confirm buttons, and secondary cancel buttons.

use super::containers::InnerResponse;
use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use crate::modal::{
    MODAL_TAG_CANCEL, MODAL_TAG_CARD, MODAL_TAG_CLOSE, MODAL_TAG_CONFIRM, MODAL_TAG_DANGER,
    MODAL_TAG_SCRIM,
};
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, KeyCode, Rect, Style, TextAlign, UiLayer,
    WidgetCursor, WidgetRole,
};

/// Configuration descriptor for a declarative modal dialog window.
#[derive(Debug, Clone)]
pub struct ModalDescriptor {
    /// Window title caption displayed in the modal header bar.
    pub title: String,
    /// Width constraint in logical pixels.
    pub width: f32,
    /// Height constraint in logical pixels.
    pub height: f32,
    /// Whether clicking the semi-transparent backdrop scrim dismisses the modal.
    pub dismiss_on_scrim: bool,
    /// Whether pressing the Escape key dismisses the modal.
    pub dismiss_on_escape: bool,
    /// Whether to render the standard compact '✕' close button in the header.
    pub show_close_button: bool,
    /// Optional custom backdrop scrim fill color.
    pub scrim_color: Option<Color>,
}

impl ModalDescriptor {
    /// Constructs a new modal dialog descriptor with standard desktop IDE defaults.
    ///
    /// Defaults:
    /// - `dismiss_on_scrim`: `true`
    /// - `dismiss_on_escape`: `true`
    /// - `show_close_button`: `true`
    /// - `scrim_color`: `None` (uses default theme scrim color)
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            dismiss_on_scrim: true,
            dismiss_on_escape: true,
            show_close_button: true,
            scrim_color: None,
        }
    }

    /// Sets whether clicking the backdrop scrim dismisses the dialog.
    #[inline]
    #[must_use]
    pub fn with_dismiss_on_scrim(mut self, enabled: bool) -> Self {
        self.dismiss_on_scrim = enabled;
        self
    }

    /// Sets whether pressing the Escape key dismisses the dialog.
    #[inline]
    #[must_use]
    pub fn with_dismiss_on_escape(mut self, enabled: bool) -> Self {
        self.dismiss_on_escape = enabled;
        self
    }

    /// Sets whether the header '✕' close button is displayed.
    #[inline]
    #[must_use]
    pub fn with_close_button(mut self, enabled: bool) -> Self {
        self.show_close_button = enabled;
        self
    }

    /// Overrides the default semi-transparent backdrop scrim color.
    #[inline]
    #[must_use]
    pub fn with_scrim_color(mut self, color: Color) -> Self {
        self.scrim_color = Some(color);
        self
    }
}

impl From<(&str, f32, f32)> for ModalDescriptor {
    #[inline]
    fn from(val: (&str, f32, f32)) -> Self {
        Self::new(val.0, val.1, val.2)
    }
}

impl From<(String, f32, f32)> for ModalDescriptor {
    #[inline]
    fn from(val: (String, f32, f32)) -> Self {
        Self::new(val.0, val.1, val.2)
    }
}

impl<'a> UiScope<'a> {
    /// Declaratively renders an autonomous modal dialog window when `is_open` is true.
    ///
    /// Coordinates full-screen backdrop scrim blocker on [`UiLayer::Modal`], hardware-accelerated
    /// elevated card presentation, title bar with optional '✕' close button, and automatic dismissal
    /// via backdrop click or Escape keystroke.
    ///
    /// If `*is_open` is `false`, this is a zero-cost no-op that returns `None`.
    ///
    /// # Arguments
    /// * `descriptor` - Modal configuration or `(title, width, height)` tuple.
    /// * `is_open` - Mutable boolean reference controlling modal visibility.
    /// * `f` - Child builder closure receiving the scoped context of the modal card body.
    ///
    /// # Returns
    /// `Some(InnerResponse<R>)` if the modal was rendered, or `None` if `*is_open == false`.
    pub fn modal<M, R, F>(
        &mut self,
        descriptor: M,
        is_open: &mut bool,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        M: Into<ModalDescriptor>,
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if !*is_open {
            return None;
        }

        let desc: ModalDescriptor = descriptor.into();

        // 1. Autonomous Escape key dismissal
        if desc.dismiss_on_escape
            && self.tree.event_state.frame_key == Some(KeyCode::Escape)
            && self.tree.event_state.frame_key_pressed
        {
            *is_open = false;
            return None;
        }

        // 2. Resolve screen/window bounds from root computed_rect
        let screen_bounds = self
            .tree
            .root()
            .and_then(|r| self.tree.get(r))
            .map_or(Rect::new(0.0, 0.0, 1920.0, 1080.0), |n| n.computed_rect);

        let scrim_col = desc
            .scrim_color
            .unwrap_or_else(|| Color::rgba(0.0, 0.0, 0.0, 0.65));

        let mut close_requested = false;

        let scrim_res = self.modal_scrim_sized(screen_bounds, scrim_col, |scrim| {
            scrim.modal_card(desc.width, desc.height, |card| {
                // Header row
                card.container(
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::SpaceBetween)
                        .height(32.0)
                        .padding_insets(Insets::new(0.0, 4.0, 0.0, 4.0)),
                    |header| {
                        header.label_custom(
                            &desc.title,
                            13.0,
                            Color::rgba(0.90, 0.92, 0.96, 1.0),
                            TextAlign::Left,
                        );

                        if desc.show_close_button && header.modal_close_button().clicked() {
                            close_requested = true;
                        }
                    },
                );

                // Divider line below header
                card.divider(Color::rgba(0.18, 0.20, 0.26, 1.0));

                // Card body content closure
                card.container(
                    Style::new()
                        .flex_col()
                        .flex_grow(1.0)
                        .padding_insets(Insets::new(8.0, 4.0, 4.0, 4.0)),
                    f,
                )
            })
        });

        // 3. Autonomous backdrop scrim click dismissal
        if desc.dismiss_on_scrim {
            let card_id = scrim_res.inner.response.id;
            let fallback_card_rect = Rect::new(
                screen_bounds.x + (screen_bounds.width - desc.width) * 0.5,
                screen_bounds.y + (screen_bounds.height - desc.height) * 0.5,
                desc.width,
                desc.height,
            );
            let card_rect = self
                .tree
                .get(card_id)
                .map(|n| n.computed_rect)
                .filter(|r| r.width > 0.0 && r.height > 0.0)
                .unwrap_or(fallback_card_rect);
            let is_inside_card = self
                .tree
                .event_state
                .frame_clicked_pos
                .or(self.tree.event_state.last_pointer_pos)
                .is_some_and(|pos| card_rect.contains_point(pos));

            if !is_inside_card
                && (scrim_res.response.clicked()
                    || self.tree.event_state.frame_clicked_tag == Some(MODAL_TAG_SCRIM))
            {
                *is_open = false;
            }
        }

        if close_requested {
            *is_open = false;
        }

        Some(InnerResponse::new(
            scrim_res.response,
            scrim_res.inner.inner.inner,
        ))
    }

    /// Returns whether the modal dialog identified by semantic `tag` is currently opened.
    #[inline]
    #[must_use]
    pub fn is_modal_open(&self, tag: u64) -> bool {
        self.tree.is_modal_open(tag)
    }

    /// Sets the persistent visibility state of a modal dialog identified by semantic `tag`.
    #[inline]
    pub fn set_modal_open(&mut self, tag: u64, open: bool) {
        self.tree.set_modal_open(tag, open);
    }

    /// Opens the modal dialog identified by semantic `tag`.
    #[inline]
    pub fn open_modal(&mut self, tag: u64) {
        self.tree.open_modal(tag);
    }

    /// Closes the modal dialog identified by semantic `tag`.
    #[inline]
    pub fn close_modal(&mut self, tag: u64) {
        self.tree.close_modal(tag);
    }

    /// Closes all active modal dialogs in the UI tree.
    #[inline]
    pub fn close_all_modals(&mut self) {
        self.tree.close_all_modals();
    }

    /// Returns whether any modal dialog is currently opened in the UI tree.
    #[inline]
    #[must_use]
    pub fn is_any_modal_open(&self) -> bool {
        self.tree.is_any_modal_open()
    }

    /// Declaratively renders an autonomous modal dialog window when the modal with `tag` is open.
    ///
    /// Automatically manages persistent modal open/closed state on [`UiTree`]. When open, renders
    /// the full-screen backdrop blocker on [`UiLayer::Modal`] and the centered modal card with title,
    /// and invokes the child builder closure `f`. If the user presses Escape, clicks the backdrop scrim,
    /// or clicks the '✕' close button, closes the modal and updates persistent state.
    pub fn modal_tagged<M, R, F>(
        &mut self,
        tag: u64,
        descriptor: M,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        M: Into<ModalDescriptor>,
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let mut is_open = self.is_modal_open(tag);
        if !is_open {
            return None;
        }
        let res = self.modal(descriptor, &mut is_open, f);
        if !is_open {
            self.set_modal_open(tag, false);
        }
        res
    }
    /// Emits a full-screen backdrop modal scrim container elevating its subtree to [`UiLayer::Modal`].
    ///
    /// The generated container is tagged with [`MODAL_TAG_SCRIM`] for dismissive hit-testing,
    /// given the [`WidgetRole::ModalWindow`] accessibility and interaction semantics, and configured
    /// with centered flexbox alignment (`AlignItems::Center`, `JustifyContent::Center`) to automatically
    /// center child modal dialog cards.
    ///
    /// # Arguments
    /// * `color` - RGBA fill color of the semi-transparent backdrop blocker.
    /// * `f` - Child builder closure receiving the scoped context of the scrim container.
    pub fn modal_scrim<R, F>(&mut self, color: Color, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("ModalScrim");
            node.tag = MODAL_TAG_SCRIM;
            node.interactive = true;
            node.role = WidgetRole::ModalWindow;
            node.layer = UiLayer::Modal;
            node.set_style(
                Style::new()
                    .flex_grow(1.0)
                    .flex_shrink(0.0)
                    .flex_col()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .background(color),
            );
        }
        let attach_parent = self.tree.root().unwrap_or(self.parent);
        let _ = self.tree.add_child(attach_parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        let inner = f(&mut child_scope);
        let _ = self.tree.set_layer_recursive(node_id, UiLayer::Modal);
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }

    /// Emits a full-screen backdrop modal scrim container elevating its subtree to [`UiLayer::Modal`]
    /// and autonomously computing subtree layout across `bounds`.
    ///
    /// Eliminates manual calls to `scrim.finish_layout(bounds)` in consumer dialog builders.
    ///
    /// # Arguments
    /// * `bounds` - Physical screen bounding rectangle allocated for the modal backdrop.
    /// * `color` - RGBA fill color of the semi-transparent backdrop blocker.
    /// * `f` - Child builder closure receiving the scoped context of the scrim container.
    pub fn modal_scrim_sized<R, F>(&mut self, bounds: Rect, color: Color, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let effective_bounds = if bounds.width > 0.0 && bounds.height > 0.0 {
            bounds
        } else {
            self.screen_bounds()
        };
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("ModalScrim");
            node.tag = MODAL_TAG_SCRIM;
            node.interactive = true;
            node.role = WidgetRole::ModalWindow;
            node.layer = UiLayer::Modal;
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(effective_bounds.x)
                    .top(effective_bounds.y)
                    .width(effective_bounds.width)
                    .height(effective_bounds.height)
                    .flex_col()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .background(color),
            );
            node.computed_rect = effective_bounds;
        }
        let attach_parent = self.tree.root().unwrap_or(self.parent);
        let _ = self.tree.add_child(attach_parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        let inner = f(&mut child_scope);
        let _ = self.tree.set_layer_recursive(node_id, UiLayer::Modal);
        crate::declarative::layout_subtree(self.tree, node_id, effective_bounds);
        if let Some(cursor) = self.tree.cursor_position() {
            super::hover::update_hover_styles_recursive(self.tree, node_id, cursor);
        }
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }

    /// Emits an elevated modal dialog card container styled with standard dark themes, borders, and drop shadows.
    ///
    /// Automatically applies [`WidgetRole::ModalWindow`] and [`UiLayer::Modal`] properties,
    /// configuring vertical flex column distribution (`JustifyContent::SpaceBetween`, `AlignItems::Stretch`)
    /// with standardized inner padding (`10.0` top, `16.0` horizontal, `14.0` bottom).
    ///
    /// # Arguments
    /// * `width` - Card width constraint in physical pixels.
    /// * `height` - Card height constraint in physical pixels.
    /// * `f` - Child builder closure receiving the scoped context of the card container.
    pub fn modal_card<R, F>(&mut self, width: f32, height: f32, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let theme = self.tree.theme();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("ModalDialogCard");
            node.tag = MODAL_TAG_CARD;
            node.interactive = true;
            node.role = WidgetRole::ModalWindow;
            node.layer = UiLayer::Modal;
            node.set_style(
                Style::new()
                    .flex_col()
                    .justify_content(JustifyContent::SpaceBetween)
                    .align_items(AlignItems::Stretch)
                    .width(width)
                    .height(height)
                    .background(theme.modal_bg)
                    .border(1.0, theme.modal_border)
                    .border_radius(8.0)
                    .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.70))
                    .padding_insets(Insets::new(10.0, 16.0, 14.0, 16.0)),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        let inner = f(&mut child_scope);
        let _ = self.tree.set_layer_recursive(node_id, UiLayer::Modal);
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }

    /// Emits a standardized modal header close button glyph ("✕") tagged with [`MODAL_TAG_CLOSE`].
    ///
    /// Configured with a modern, frameless compact pill (`22.0×22.0px`) that sits transparently in the
    /// header bar during idle state and smoothly highlights with a soft, borderless danger tint upon hover.
    pub fn modal_close_button(&mut self) -> WidgetResponse {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("ModalCloseBtn");
            node.tag = MODAL_TAG_CLOSE;
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.layer = UiLayer::Modal;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        let theme = self.tree.theme();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text("✕");
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                theme.text_primary
            } else {
                theme.text_secondary
            };
            let mut danger_hover = theme.danger;
            danger_hover.a = 0.28;
            node.set_style(
                Style::new()
                    .width(22.0)
                    .height(22.0)
                    .border_radius(5.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .background(if hovered {
                        danger_hover
                    } else {
                        Color::TRANSPARENT
                    })
                    .border(0.0, Color::TRANSPARENT),
            );
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits a primary confirm action push button tagged with [`MODAL_TAG_CONFIRM`].
    ///
    /// Applies clean blue desktop action button styling with a subtle ~10% background
    /// brightness lift upon mouse intersection, without distracting neon glow.
    ///
    /// # Arguments
    /// * `label` - Button caption text (e.g., "Close", "Confirm", "Save").
    /// * `width` - Button width constraint in physical pixels.
    pub fn modal_confirm_button(&mut self, label: impl Into<String>, width: f32) -> WidgetResponse {
        self.modal_confirm_button_tagged(label, width, MODAL_TAG_CONFIRM)
    }

    /// Emits a primary confirm action push button with a custom semantic tag.
    ///
    /// Applies clean blue desktop action button styling with a subtle ~10% background
    /// brightness lift upon mouse intersection, without distracting neon glow.
    ///
    /// # Arguments
    /// * `label` - Button caption text (e.g., "Close", "Confirm", "Save").
    /// * `width` - Button width constraint in physical pixels.
    /// * `tag` - Custom semantic tag constant for event routing.
    pub fn modal_confirm_button_tagged(
        &mut self,
        label: impl Into<String>,
        width: f32,
        tag: u64,
    ) -> WidgetResponse {
        let label_str = label.into();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("ModalConfirmBtn");
            node.tag = tag;
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.layer = UiLayer::Modal;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        let theme = self.tree.theme();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.5;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = theme.text_primary;
            let (bg, border) = if hovered {
                (theme.button_active_bg, theme.button_hover_border)
            } else {
                (
                    Color::rgba(
                        theme.button_active_bg.r * 0.88,
                        theme.button_active_bg.g * 0.88,
                        theme.button_active_bg.b * 0.88,
                        theme.button_active_bg.a,
                    ),
                    theme.button_border,
                )
            };
            node.set_style(
                Style::new()
                    .width(width)
                    .height(26.0)
                    .border_radius(4.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .background(bg)
                    .border(1.0, border),
            );
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits a secondary dismiss action push button tagged with [`MODAL_TAG_CANCEL`].
    ///
    /// Applies neutral dark gray button styling with a gentle 10% background highlight.
    ///
    /// # Arguments
    /// * `label` - Button caption text (e.g., "Cancel", "Dismiss").
    /// * `width` - Button width constraint in physical pixels.
    pub fn modal_cancel_button(&mut self, label: impl Into<String>, width: f32) -> WidgetResponse {
        let label_str = label.into();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("ModalCancelBtn");
            node.tag = MODAL_TAG_CANCEL;
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.layer = UiLayer::Modal;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        let theme = self.tree.theme();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.5;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                theme.text_primary
            } else {
                theme.text_secondary
            };
            let (bg, border) = if hovered {
                (theme.button_hover_bg, theme.button_hover_border)
            } else {
                (theme.button_idle_bg, theme.button_border)
            };
            node.set_style(
                Style::new()
                    .width(width)
                    .height(26.0)
                    .border_radius(4.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .background(bg)
                    .border(1.0, border),
            );
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits a destructive confirmation action push button tagged with [`MODAL_TAG_DANGER`].
    ///
    /// Applies high-visibility danger red button styling with a subtle ~10% background
    /// brightness lift upon mouse intersection, without distracting neon glow.
    ///
    /// # Arguments
    /// * `label` - Button caption text (e.g., "Delete Permanently", "Discard Changes").
    /// * `width` - Button width constraint in physical pixels.
    pub fn modal_danger_button(&mut self, label: impl Into<String>, width: f32) -> WidgetResponse {
        self.modal_danger_button_tagged(label, width, MODAL_TAG_DANGER)
    }

    /// Emits a destructive confirmation action push button with a custom semantic tag.
    ///
    /// Applies high-visibility danger red button styling with a subtle ~10% background
    /// brightness lift upon mouse intersection, without distracting neon glow.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `width` - Button width constraint in physical pixels.
    /// * `tag` - Custom semantic tag constant for event routing.
    pub fn modal_danger_button_tagged(
        &mut self,
        label: impl Into<String>,
        width: f32,
        tag: u64,
    ) -> WidgetResponse {
        let label_str = label.into();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("ModalDangerBtn");
            node.tag = tag;
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.layer = UiLayer::Modal;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        let theme = self.tree.theme();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.5;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = theme.text_primary;
            let (bg, border) = if hovered {
                (theme.danger, theme.button_hover_border)
            } else {
                (
                    Color::rgba(
                        theme.danger.r * 0.85,
                        theme.danger.g * 0.85,
                        theme.danger.b * 0.85,
                        theme.danger.a,
                    ),
                    theme.button_border,
                )
            };
            node.set_style(
                Style::new()
                    .width(width)
                    .height(26.0)
                    .border_radius(4.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .background(bg)
                    .border(1.0, border),
            );
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }
}