// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Floating Panel & Window Controller (`iris-widgets::floating_panel`)
//!
//! Provides a first-class declarative floating tool window and dialog controller.
//! Encapsulates titlebar drag detection, multi-axis screen boundary clamping,
//! autonomous scrollbar invalidation, click absorption, and event dispatching with
//! zero manual host orchestration.

use iris_core::dispatch::DispatchResult;
use iris_core::event::{MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect};
use iris_core::node::WidgetRole;
use iris_core::tree::UiTree;

/// Default persistent semantic tag for floating panel cards (`0xF101`).
pub const FLOATING_TAG_WINDOW: u64 = 0xF101;
/// Default persistent semantic tag for floating panel draggable titlebar strips (`0xF102`).
pub const FLOATING_TAG_TITLEBAR: u64 = 0xF102;
/// Default persistent semantic tag for floating panel header close buttons (`0xF103`).
pub const FLOATING_TAG_CLOSE: u64 = 0xF103;

/// Default titlebar height in logical pixels.
pub const DEFAULT_TITLEBAR_HEIGHT: f32 = 36.0;

/// Action resulting from user interaction within a floating panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatingPanelAction {
    /// The floating panel was repositioned to the specified coordinates via titlebar dragging.
    Move(Point),
    /// The floating panel close button was activated.
    Close,
    /// A semantic interactive control with the specified 64-bit tag was clicked inside the panel.
    Tag(u64),
}

/// Interaction outcome emitted when evaluating events against a [`FloatingPanelState`].
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FloatingPanelOutcome {
    /// Whether the incoming event was intercepted and consumed by the floating panel.
    pub consumed: bool,
    /// Whether the host application or shell requires a layout/render frame rebuild.
    pub needs_layout_rebuild: bool,
    /// Specific high-level action produced by the interaction, if any.
    pub action: Option<FloatingPanelAction>,
}

/// Persistent interactive state for a detached or elevated floating panel.
///
/// Tracks window positioning, continuous titlebar dragging offsets, bounding geometry,
/// and configured titlebar / close button semantic tags.
#[derive(Debug, Clone, PartialEq)]
pub struct FloatingPanelState {
    /// Current top-left screen position of the floating panel, or None for auto-centered default.
    pub pos: Option<Point>,
    /// Active drag offset from window top-left when dragging the title bar.
    pub drag_offset: Option<Point>,
    /// Bounding rectangle of the floating panel card on screen.
    pub rect: Option<Rect>,
    /// Minimum allowable Y coordinate (e.g. to prevent occluding top menubars).
    pub min_y: f32,
    /// Height of the top draggable titlebar strip in logical pixels.
    pub titlebar_height: f32,
    /// Configured semantic tag for the titlebar strip (0 if relying on geometry / role).
    pub titlebar_tag: u64,
    /// Configured semantic tag for the close button.
    pub close_tag: u64,
}

impl Default for FloatingPanelState {
    fn default() -> Self {
        Self {
            pos: None,
            drag_offset: None,
            rect: None,
            min_y: 0.0,
            titlebar_height: DEFAULT_TITLEBAR_HEIGHT,
            titlebar_tag: FLOATING_TAG_TITLEBAR,
            close_tag: FLOATING_TAG_CLOSE,
        }
    }
}

impl FloatingPanelState {
    /// Creates a new floating panel state with default configurations.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the minimum allowable Y screen coordinate.
    #[must_use]
    pub const fn with_min_y(mut self, min_y: f32) -> Self {
        self.min_y = min_y;
        self
    }

    /// Configures the height of the top draggable titlebar strip.
    #[must_use]
    pub const fn with_titlebar_height(mut self, height: f32) -> Self {
        self.titlebar_height = height;
        self
    }

    /// Configures custom semantic tags for titlebar dragging and close requests.
    #[must_use]
    pub const fn with_tags(mut self, titlebar_tag: u64, close_tag: u64) -> Self {
        self.titlebar_tag = titlebar_tag;
        self.close_tag = close_tag;
        self
    }

    /// Returns `true` if the titlebar is actively captured and being dragged by the pointer.
    #[must_use]
    pub const fn is_dragging(&self) -> bool {
        self.drag_offset.is_some()
    }

    /// Resets ongoing drag gestures and cached bounding geometry.
    pub fn reset(&mut self) {
        self.drag_offset = None;
        self.rect = None;
    }

    /// Evaluates incoming platform events and tree dispatch results in a single declarative call.
    ///
    /// Manages:
    /// 1. Titlebar drag inception on left [`UiEvent::MouseDown`].
    /// 2. Clamped window repositioning on [`UiEvent::MouseMove`].
    /// 3. Scrollbar thumb dragging invalidation.
    /// 4. Drag release on left [`UiEvent::MouseUp`].
    /// 5. Scroll wheel invalidation inside panel bounds.
    /// 6. Close button click and tag click routing.
    /// 7. Click absorption preventing pass-through to lower canvas layers.
    pub fn handle_event(
        &mut self,
        tree: &mut UiTree,
        event: &UiEvent,
        dispatch: &DispatchResult,
        screen_width: f32,
        screen_height: f32,
    ) -> FloatingPanelOutcome {
        let mut outcome = FloatingPanelOutcome::default();

        let Some(card_rect) = self.rect else {
            return outcome;
        };

        // 1. Titlebar Drag Inception on MouseDown
        if let UiEvent::MouseDown {
            button: MouseButton::Left,
            point,
        } = event
        {
            let is_in_titlebar_rect = point.x >= card_rect.x
                && point.x <= (card_rect.right() - 32.0)
                && point.y >= card_rect.y
                && point.y <= (card_rect.y + self.titlebar_height);

            let is_close_tag = (self.close_tag != 0 && dispatch.target_tag == self.close_tag)
                || dispatch.target_tag == FLOATING_TAG_CLOSE;

            let is_titlebar_tag =
                self.titlebar_tag != 0 && dispatch.target_tag == self.titlebar_tag;
            let is_titlebar_role = dispatch
                .target
                .and_then(|id| tree.get(id))
                .is_some_and(|n| n.role == WidgetRole::FloatingTitlebar);

            if !is_close_tag && (is_in_titlebar_rect || is_titlebar_tag || is_titlebar_role) {
                let cur_pos = self.pos.unwrap_or(Point::new(card_rect.x, card_rect.y));
                self.drag_offset = Some(Point::new(point.x - cur_pos.x, point.y - cur_pos.y));
                self.pos = Some(cur_pos);
                outcome.consumed = true;
            }
        }

        // 2. Titlebar Dragging or Scrollbar Thumb Dragging on MouseMove
        if let UiEvent::MouseMove { point } = event {
            if let Some(drag_offset) = self.drag_offset {
                let new_pos = calculate_floating_drag_pos(
                    *point,
                    drag_offset,
                    (card_rect.width, card_rect.height),
                    screen_width,
                    screen_height,
                    self.min_y,
                );
                self.pos = Some(new_pos);
                self.rect = Some(Rect::new(
                    new_pos.x,
                    new_pos.y,
                    card_rect.width,
                    card_rect.height,
                ));
                outcome.needs_layout_rebuild = true;
                outcome.consumed = true;
                outcome.action = Some(FloatingPanelAction::Move(new_pos));
            } else if tree.event_state.is_pointer_drag_active {
                let is_thumb_captured = tree.event_state.pointer_capture.is_some_and(|cid| {
                    tree.get(cid)
                        .is_some_and(|n| n.role == WidgetRole::ScrollBarThumb)
                }) || tree.event_state.pointer_capture_tag.is_some();
                if is_thumb_captured {
                    outcome.needs_layout_rebuild = true;
                    outcome.consumed = true;
                }
            }
        }

        // 3. Drag Release on MouseUp
        if let UiEvent::MouseUp {
            button: MouseButton::Left,
            ..
        } = event
            && self.drag_offset.take().is_some()
        {
            outcome.needs_layout_rebuild = true;
            outcome.consumed = true;
        }

        // 4. Mouse Scroll inside Floating Panel Bounds
        if let UiEvent::MouseScroll { point, .. } = event
            && card_rect.contains_point(*point)
        {
            outcome.needs_layout_rebuild = true;
            outcome.consumed = true;
        }

        // 5. Semantic Click Tag Handling
        if let Some(clicked_tag) = tree.event_state.frame_clicked_tag
            && clicked_tag != 0
        {
            if clicked_tag == self.close_tag || clicked_tag == FLOATING_TAG_CLOSE {
                outcome.consumed = true;
                outcome.needs_layout_rebuild = true;
                outcome.action = Some(FloatingPanelAction::Close);
                self.drag_offset = None;
            } else {
                outcome.action = Some(FloatingPanelAction::Tag(clicked_tag));
            }
        }

        // 6. Click Absorption: Prevent click pass-through to lower canvas or dock panels
        if let UiEvent::MouseDown { point, .. } | UiEvent::MouseUp { point, .. } = event
            && card_rect.contains_point(*point)
        {
            outcome.consumed = true;
        }

        if dispatch.is_consumed()
            && (self.titlebar_tag != 0 && dispatch.target_tag == self.titlebar_tag)
        {
            outcome.consumed = true;
        }

        outcome
    }
}

/// Calculates the clamped position for a floating window during titlebar dragging.
///
/// Guarantees that the panel stays fully within the visible screen area and does not
/// escape above `min_y`.
#[must_use]
pub fn calculate_floating_drag_pos(
    cursor: Point,
    drag_offset: Point,
    panel_size: (f32, f32),
    screen_width: f32,
    screen_height: f32,
    min_y: f32,
) -> Point {
    let (panel_w, panel_h) = panel_size;
    let max_x = (screen_width - panel_w).max(0.0);
    let max_y = (screen_height - panel_h).max(min_y);
    Point::new(
        (cursor.x - drag_offset.x).clamp(0.0, max_x).round(),
        (cursor.y - drag_offset.y).clamp(min_y, max_y).round(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_floating_drag_pos_clamping() {
        let size = (760.0, 540.0);
        let offset = Point::new(50.0, 15.0);

        // Center
        let pos = calculate_floating_drag_pos(
            Point::new(500.0, 400.0),
            offset,
            size,
            1920.0,
            1080.0,
            30.0,
        );
        assert_eq!(pos.x, 450.0);
        assert_eq!(pos.y, 385.0);

        // Clamped top-left
        let pos_tl =
            calculate_floating_drag_pos(Point::new(10.0, 20.0), offset, size, 1920.0, 1080.0, 30.0);
        assert_eq!(pos_tl.x, 0.0);
        assert_eq!(pos_tl.y, 30.0);

        // Clamped bottom-right
        let pos_br = calculate_floating_drag_pos(
            Point::new(3000.0, 3000.0),
            offset,
            size,
            1920.0,
            1080.0,
            30.0,
        );
        assert_eq!(pos_br.x, 1920.0 - 760.0);
        assert_eq!(pos_br.y, 1080.0 - 540.0);
    }

    #[test]
    fn test_floating_panel_state_drag_lifecycle() {
        let mut state = FloatingPanelState::new()
            .with_min_y(30.0)
            .with_titlebar_height(36.0);
        let mut tree = UiTree::new();
        state.rect = Some(Rect::new(200.0, 100.0, 760.0, 540.0));
        state.pos = Some(Point::new(200.0, 100.0));

        let down_event = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(250.0, 115.0),
        };
        let down_outcome = state.handle_event(
            &mut tree,
            &down_event,
            &DispatchResult::ignored(),
            1920.0,
            1080.0,
        );
        assert!(down_outcome.consumed);
        assert!(state.is_dragging());
        assert_eq!(state.drag_offset, Some(Point::new(50.0, 15.0)));

        let move_event = UiEvent::MouseMove {
            point: Point::new(300.0, 150.0),
        };
        let move_outcome = state.handle_event(
            &mut tree,
            &move_event,
            &DispatchResult::ignored(),
            1920.0,
            1080.0,
        );
        assert!(move_outcome.consumed);
        assert!(move_outcome.needs_layout_rebuild);
        assert_eq!(state.pos, Some(Point::new(250.0, 135.0)));
        assert_eq!(
            move_outcome.action,
            Some(FloatingPanelAction::Move(Point::new(250.0, 135.0)))
        );

        let up_event = UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(300.0, 150.0),
        };
        let up_outcome = state.handle_event(
            &mut tree,
            &up_event,
            &DispatchResult::ignored(),
            1920.0,
            1080.0,
        );
        assert!(up_outcome.consumed);
        assert!(!state.is_dragging());
    }
}