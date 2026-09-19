// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event routing logic for the native dock tab overflow dropdown menu.
//!
//! Evaluates interaction states via [`iris_dock::evaluate_dock_overflow_click`].
//!

use super::super::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::dock::{
    DockOverflowClickAction, DockOverflowItemTarget, DockOverflowMenuFrame,
    evaluate_dock_overflow_click,
};
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles mouse hovering, clicks, and dismissal for the active dock tab overflow popup menu.
    ///
    /// Evaluates clicks via [`iris_dock::evaluate_dock_overflow_click`] to route tab activation,
    /// chevron toggling, or outside click dismissal.
    pub(crate) fn handle_dock_overflow_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let (active_leaf, chevron_rect) = self.chrome.active_dock_overflow?;
        let frame = self.chrome.native_dock_frame.as_ref()?;
        let mut result = IrisOverlayEventResult::default();

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.chrome.cursor_pos = Point::new(position.x as f32, position.y as f32);
                if let Some(ref overflow_rect) = frame.active_overflow_rect
                    && overflow_rect.contains_point(self.chrome.cursor_pos)
                {
                    result.consumed = true;
                    return Some(result);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } => {
                let click_point = self.cursor_pos();

                let dock_menu_frame =
                    frame
                        .active_overflow_rect
                        .map(|menu_rect| DockOverflowMenuFrame {
                            menu_node_id: WidgetId::default(),
                            menu_rect,
                            items: frame
                                .overflow_item_targets
                                .iter()
                                .map(|t| DockOverflowItemTarget {
                                    leaf: t.leaf,
                                    tab_index: t.tab_index,
                                    rect: t.rect,
                                })
                                .collect(),
                        });

                let action = evaluate_dock_overflow_click(
                    click_point,
                    active_leaf,
                    chevron_rect,
                    dock_menu_frame.as_ref(),
                );

                match action {
                    DockOverflowClickAction::SelectTab { leaf, tab_index } => {
                        result.activate_dock_tab = Some((leaf, tab_index));
                        self.chrome.active_dock_overflow = None;
                    }
                    DockOverflowClickAction::ToggleChevron { .. } => {
                        // Check if clicking same chevron or another leaf's chevron
                        if let Some(target) = frame
                            .chevron_targets
                            .iter()
                            .find(|t| t.rect.contains_point(click_point))
                        {
                            if target.leaf == active_leaf {
                                self.chrome.active_dock_overflow = None;
                            } else {
                                self.chrome.active_dock_overflow = Some((target.leaf, target.rect));
                            }
                        } else {
                            self.chrome.active_dock_overflow = None;
                        }
                    }
                    DockOverflowClickAction::Dismiss => {
                        self.chrome.active_dock_overflow = None;
                    }
                }

                self.chrome.needs_layout_rebuild = true;
                result.consumed = true;
                return Some(result);
            }
            _ => {}
        }

        None
    }
}