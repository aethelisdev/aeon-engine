// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event routing logic for About, Delete, New Folder, and Rename modal dialogs.
//!
//! Uses semantic tag resolution and pure declarative responses instead of raw coordinate hit-testing.
//!

use super::super::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::{ModalDialogAction, evaluate_modal_tag};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles keyboard and click interactions for active modal dialogues.
    ///
    /// Evaluates semantic action tags (`MODAL_TAG_*`) and keyboard triggers without manual
    /// coordinate boundary hit-testing.
    pub(crate) fn handle_modal_events(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let any_modal_active = self.modals.is_about_active
            || self.modals.is_delete_active
            || self.modals.is_new_folder_active
            || self.modals.is_rename_active
            || self.modals.is_loading_active;

        if !any_modal_active {
            return None;
        }

        let mut result = IrisOverlayEventResult::default();

        // 1. Keyboard event handling
        match event {
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(key),
                        state: ElementState::Pressed,
                        text,
                        ..
                    },
                ..
            } => match *key {
                winit::keyboard::KeyCode::Escape => {
                    if self.modals.is_about_active {
                        result.close_about = true;
                    } else if self.modals.is_delete_active {
                        result.cancel_delete = true;
                    } else if self.modals.is_new_folder_active {
                        result.cancel_new_folder = true;
                    } else if self.modals.is_rename_active {
                        result.cancel_rename = true;
                    }
                    result.consumed = true;
                    return Some(result);
                }
                winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                    if self.modals.is_delete_active {
                        result.confirm_delete = true;
                    } else if self.modals.is_new_folder_active
                        && !self.modals.new_folder_buffer.trim().is_empty()
                    {
                        result.create_folder = Some(self.modals.new_folder_buffer.clone());
                    } else if self.modals.is_rename_active
                        && !self.modals.rename_buffer.trim().is_empty()
                    {
                        result.apply_rename = Some(self.modals.rename_buffer.clone());
                    }
                    result.consumed = true;
                    return Some(result);
                }
                winit::keyboard::KeyCode::Backspace => {
                    if self.modals.is_new_folder_active {
                        self.modals.new_folder_buffer.pop();
                        result.consumed = true;
                        return Some(result);
                    } else if self.modals.is_rename_active {
                        self.modals.rename_buffer.pop();
                        result.consumed = true;
                        return Some(result);
                    }
                }
                _ => {
                    if let Some(t) = text
                        && !t.chars().any(|c| c.is_control())
                    {
                        if self.modals.is_new_folder_active {
                            self.modals.new_folder_buffer.push_str(t);
                            result.consumed = true;
                            return Some(result);
                        } else if self.modals.is_rename_active {
                            self.modals.rename_buffer.push_str(t);
                            result.consumed = true;
                            return Some(result);
                        }
                    }
                }
            },
            WindowEvent::Ime(winit::event::Ime::Commit(t)) => {
                if self.modals.is_new_folder_active {
                    self.modals.new_folder_buffer.push_str(t);
                    result.consumed = true;
                    return Some(result);
                } else if self.modals.is_rename_active {
                    self.modals.rename_buffer.push_str(t);
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
                let hit_target = self.tree.hit_test_target(click_point);

                if let Some(ref h) = hit_target {
                    self.modals.pending_interaction_events.push((
                        h.id,
                        irisui::prelude::InteractionEvent::Click {
                            button: irisui::prelude::MouseButton::Left,
                        },
                    ));
                }

                let modal_action = hit_target.and_then(|h| evaluate_modal_tag(h.tag));

                if self.modals.is_about_active {
                    if let Some(action) = modal_action {
                        match action {
                            ModalDialogAction::Close
                            | ModalDialogAction::Confirm
                            | ModalDialogAction::Cancel
                            | ModalDialogAction::ScrimDismiss => {
                                result.close_about = true;
                            }
                        }
                    }
                    result.consumed = true;
                    return Some(result);
                }

                if self.modals.is_delete_active {
                    if let Some(action) = modal_action {
                        match action {
                            ModalDialogAction::Confirm => {
                                result.confirm_delete = true;
                            }
                            ModalDialogAction::Close
                            | ModalDialogAction::Cancel
                            | ModalDialogAction::ScrimDismiss => {
                                result.cancel_delete = true;
                            }
                        }
                    }
                    result.consumed = true;
                    return Some(result);
                }

                if self.modals.is_new_folder_active {
                    if let Some(action) = modal_action {
                        match action {
                            ModalDialogAction::Confirm => {
                                if !self.modals.new_folder_buffer.trim().is_empty() {
                                    result.create_folder =
                                        Some(self.modals.new_folder_buffer.clone());
                                }
                            }
                            ModalDialogAction::Close
                            | ModalDialogAction::Cancel
                            | ModalDialogAction::ScrimDismiss => {
                                result.cancel_new_folder = true;
                            }
                        }
                    }
                    result.consumed = true;
                    return Some(result);
                }

                if self.modals.is_rename_active {
                    if let Some(action) = modal_action {
                        match action {
                            ModalDialogAction::Confirm => {
                                if !self.modals.rename_buffer.trim().is_empty() {
                                    result.apply_rename = Some(self.modals.rename_buffer.clone());
                                }
                            }
                            ModalDialogAction::Close
                            | ModalDialogAction::Cancel
                            | ModalDialogAction::ScrimDismiss => {
                                result.cancel_rename = true;
                            }
                        }
                    }
                    result.consumed = true;
                    return Some(result);
                }

                if self.modals.is_loading_active {
                    result.consumed = true;
                    return Some(result);
                }
            }
            _ => {}
        }

        None
    }
}