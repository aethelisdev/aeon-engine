// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Dialog Focus Trap & Event Containment
//!
//! Enforces strict input isolation when one or more modal dialogs are presented.
//! Traps keyboard navigation and pointer clicks, preventing interaction leakage
//! into underlying workbench panels, docking splitters, and 3D scenes.

use crate::action_bus::ActionBus;
use crate::dialogs::host::active::{ActiveDialog, DialogKind};
use crate::dialogs::host::focus::DialogFocusItem;
use iris_core::event::KeyCode;
use iris_core::geometry::{Point, Rect};

/// Autonomous focus trap and input containment coordinator.
pub struct DialogFocusTrap;

impl DialogFocusTrap {
    /// Evaluates whether keyboard focus is currently trapped by an active dialog stack.
    ///
    /// Returns `true` if `depth > 0`, indicating that an elevated modal card is active.
    #[inline]
    #[must_use]
    pub fn is_trapped(depth: usize) -> bool {
        depth > 0
    }

    /// Evaluates whether a screen-space cursor coordinate falls within the modal occlusion zone.
    ///
    /// When modal dialogs are presented, the semi-transparent backdrop scrim spans the entire
    /// viewport, meaning all points within `viewport` are occluded and trapped.
    #[inline]
    #[must_use]
    pub fn occludes_point(is_active: bool, point: Point, viewport: Rect) -> bool {
        if !is_active {
            return false;
        }
        viewport.contains_point(point)
    }

    /// Processes keyboard events against the topmost active dialog, including modifier keys.
    ///
    /// Returns `true` if the event was captured and consumed by the modal dialog layer.
    ///
    /// # Keyboard Actions
    /// - `Escape`: If `dismiss_on_escape` is enabled, dismisses top dialog and emits cancel action;
    ///   otherwise triggers attention shake animation.
    /// - `Enter`:
    ///   - If current focus is `DialogFocusItem::Cancel`, cancels dialog.
    ///   - If current focus is affirmative or default, executes confirm/submit action.
    /// - `Tab`: Cycles keyboard focus forward (`shift_held == false`) or backward (`shift_held == true`).
    /// - All other keys: Consumed while modal stack is active to prevent shortcut leakage.
    pub fn handle_key_with_modifiers<Action: 'static + Send + Sync>(
        stack: &mut Vec<ActiveDialog<Action>>,
        key: &KeyCode,
        is_pressed: bool,
        shift_held: bool,
        action_bus: &mut ActionBus<Action>,
    ) -> bool {
        let Some(top) = stack.last() else {
            return false;
        };

        if !is_pressed {
            return true;
        }

        if *key == KeyCode::Tab {
            if let Some(top_mut) = stack.last_mut() {
                top_mut.focus_cycle.cycle(shift_held);
            }
            return true;
        }

        if *key == KeyCode::Escape {
            if !top.is_cancellable || !top.dismiss_on_escape {
                if let Some(top_mut) = stack.last_mut() {
                    top_mut.trigger_attention();
                }
                return true;
            }

            if let Some(mut dialog) = stack.pop()
                && let Some(action) = dialog.take_cancel_action()
            {
                action_bus.emit(action);
            }
            return true;
        }

        if *key == KeyCode::Enter {
            if top.focus_cycle.current() == Some(DialogFocusItem::Cancel) {
                if !top.is_cancellable {
                    if let Some(top_mut) = stack.last_mut() {
                        top_mut.trigger_attention();
                    }
                    return true;
                }
                if let Some(mut dialog) = stack.pop()
                    && let Some(action) = dialog.take_cancel_action()
                {
                    action_bus.emit(action);
                }
                return true;
            }

            return Self::handle_enter(stack, action_bus);
        }

        true
    }

    /// Processes keyboard events against the topmost active dialog without modifier state.
    pub fn handle_key<Action: 'static + Send + Sync>(
        stack: &mut Vec<ActiveDialog<Action>>,
        key: &KeyCode,
        is_pressed: bool,
        action_bus: &mut ActionBus<Action>,
    ) -> bool {
        Self::handle_key_with_modifiers(stack, key, is_pressed, false, action_bus)
    }

    /// Private helper handling Enter key affirmative submissions.
    fn handle_enter<Action: 'static + Send + Sync>(
        stack: &mut Vec<ActiveDialog<Action>>,
        action_bus: &mut ActionBus<Action>,
    ) -> bool {
        let Some(top) = stack.last() else {
            return false;
        };

        match &top.kind {
            DialogKind::Confirm { .. } => {
                if let Some(ActiveDialog {
                    kind: DialogKind::Confirm { on_confirm, .. },
                    ..
                }) = stack.pop()
                {
                    action_bus.emit(on_confirm);
                }
                true
            }
            DialogKind::Input { .. } => {
                if let Some(ActiveDialog {
                    kind:
                        DialogKind::Input {
                            text, on_submit, ..
                        },
                    ..
                }) = stack.pop()
                {
                    let action = on_submit(text);
                    action_bus.emit(action);
                }
                true
            }
            DialogKind::InfoCard { .. } => {
                if let Some(ActiveDialog {
                    kind: DialogKind::InfoCard { on_close, .. },
                    ..
                }) = stack.pop()
                    && let Some(action) = on_close
                {
                    action_bus.emit(action);
                }
                true
            }
            _ => true,
        }
    }
}