// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Modal Dialogues & Floating Overlay Primitives
//!
//! Provides hardware-accelerated GPU SDF modal primitives on [`UiScope`]: full-screen
//! backdrop scrim blockers, elevated floating dialog cards, modern `✕` close buttons,
//! primary confirm buttons, and secondary cancel buttons.
//!

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use crate::modal::{
    MODAL_TAG_CANCEL, MODAL_TAG_CLOSE, MODAL_TAG_CONFIRM, MODAL_TAG_DANGER, MODAL_TAG_SCRIM,
};
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, UiLayer, WidgetCursor, WidgetId,
    WidgetRole,
};

impl<'a> UiScope<'a> {
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
    pub fn modal_scrim<F>(&mut self, color: Color, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
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
                    .flex_col()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .background(color),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = UiScope {
            tree: self.tree,
            parent: node_id,
            events: self.events,
            hovered_id: self.hovered_id,
            tagged_events: self.tagged_events,
            hovered_tag: self.hovered_tag,
        };
        f(&mut child_scope);
        node_id
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
    pub fn modal_card<F>(&mut self, width: f32, height: f32, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("ModalDialogCard");
            node.role = WidgetRole::ModalWindow;
            node.layer = UiLayer::Modal;
            node.set_style(
                Style::new()
                    .flex_col()
                    .justify_content(JustifyContent::SpaceBetween)
                    .align_items(AlignItems::Stretch)
                    .width(width)
                    .height(height)
                    .background(Color::rgba(0.08, 0.08, 0.10, 0.98))
                    .border(1.0, Color::rgba(0.20, 0.22, 0.28, 1.0))
                    .border_radius(8.0)
                    .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.70))
                    .padding_insets(Insets::new(10.0, 16.0, 14.0, 16.0)),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = UiScope {
            tree: self.tree,
            parent: node_id,
            events: self.events,
            hovered_id: self.hovered_id,
            tagged_events: self.tagged_events,
            hovered_tag: self.hovered_tag,
        };
        f(&mut child_scope);
        node_id
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
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text("✕");
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                Color::WHITE
            } else {
                Color::rgba(0.60, 0.64, 0.72, 0.85)
            };
            node.set_style(
                Style::new()
                    .width(22.0)
                    .height(22.0)
                    .border_radius(5.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .background(if hovered {
                        Color::rgba(0.85, 0.22, 0.22, 0.28)
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
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.5;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                Color::WHITE
            } else {
                Color::rgba(0.92, 0.96, 1.0, 1.0)
            };
            node.set_style(
                Style::new()
                    .width(width)
                    .height(26.0)
                    .border_radius(4.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .background(if hovered {
                        Color::rgba(0.18, 0.48, 0.68, 1.0)
                    } else {
                        Color::rgba(0.14, 0.40, 0.58, 1.0)
                    })
                    .border(
                        1.0,
                        if hovered {
                            Color::rgba(1.0, 1.0, 1.0, 0.25)
                        } else {
                            Color::rgba(1.0, 1.0, 1.0, 0.15)
                        },
                    ),
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
        self.modal_cancel_button_tagged(label, width, MODAL_TAG_CANCEL)
    }

    /// Emits a secondary dismiss action push button with a custom semantic tag.
    ///
    /// Applies neutral dark gray button styling with a gentle 10% background highlight.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `width` - Button width constraint in physical pixels.
    /// * `tag` - Semantic tag constant for event routing.
    pub fn modal_cancel_button_tagged(
        &mut self,
        label: impl Into<String>,
        width: f32,
        tag: u64,
    ) -> WidgetResponse {
        let label_str = label.into();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("ModalCancelBtn");
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.5;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                Color::WHITE
            } else {
                Color::rgba(0.75, 0.78, 0.84, 1.0)
            };
            node.set_style(
                Style::new()
                    .width(width)
                    .height(26.0)
                    .border_radius(4.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .background(if hovered {
                        Color::rgba(0.20, 0.22, 0.26, 1.0)
                    } else {
                        Color::rgba(0.14, 0.15, 0.18, 1.0)
                    })
                    .border(
                        1.0,
                        if hovered {
                            Color::rgba(1.0, 1.0, 1.0, 0.18)
                        } else {
                            Color::rgba(1.0, 1.0, 1.0, 0.10)
                        },
                    ),
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
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.5;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = Color::WHITE;
            node.set_style(
                Style::new()
                    .width(width)
                    .height(26.0)
                    .border_radius(4.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .background(if hovered {
                        Color::rgba(0.75, 0.20, 0.20, 1.0)
                    } else {
                        Color::rgba(0.63, 0.14, 0.14, 1.0)
                    })
                    .border(
                        1.0,
                        if hovered {
                            Color::rgba(1.0, 1.0, 1.0, 0.25)
                        } else {
                            Color::rgba(1.0, 1.0, 1.0, 0.15)
                        },
                    ),
            );
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }
}