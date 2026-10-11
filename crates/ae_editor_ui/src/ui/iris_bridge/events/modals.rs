// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event routing logic for About, Delete, New Folder, and Rename modal dialogs.
//!
//! Uses semantic tag resolution and pure declarative responses instead of raw coordinate hit-testing.
//!

use super::super::types::{IrisEditorOverlay, IrisOverlayEventResult};
use crate::ui::iris_bridge::events::text_input::TextInputActionResult;
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

        // 1. Text Input modal keyboard and IME handling (New Folder)
        if self.modals.is_new_folder_active {
            let action = crate::ui::iris_bridge::events::text_input::handle_text_input_event(
                &mut self.modals.new_folder_input,
                event,
                self.chrome.ctrl_held,
            );
            match action {
                TextInputActionResult::Confirmed => {
                    let folder = self.modals.new_folder_input.text().trim().to_string();
                    if !folder.is_empty() {
                        result.create_folder = Some(folder);
                        self.modals.is_new_folder_active = false;
                        self.modals.clear_new_folder();
                        self.overlay_tree.close();
                    }
                    result.consumed = true;
                    return Some(result);
                }
                TextInputActionResult::Cancelled => {
                    result.cancel_new_folder = true;
                    self.modals.is_new_folder_active = false;
                    self.modals.clear_new_folder();
                    self.overlay_tree.close();
                    result.consumed = true;
                    return Some(result);
                }
                TextInputActionResult::Changed => {
                    result.consumed = true;
                    return Some(result);
                }
                TextInputActionResult::Ignored => {}
            }
        }

        // 2. Text Input modal keyboard and IME handling (Rename Asset / Folder)
        if self.modals.is_rename_active {
            let action = crate::ui::iris_bridge::events::text_input::handle_text_input_event(
                &mut self.modals.rename_input,
                event,
                self.chrome.ctrl_held,
            );
            match action {
                TextInputActionResult::Confirmed => {
                    let rename = self.modals.rename_input.text().trim().to_string();
                    if !rename.is_empty() {
                        result.apply_rename = Some(rename);
                        self.modals.is_rename_active = false;
                        self.modals.clear_rename();
                        self.overlay_tree.close();
                    }
                    result.consumed = true;
                    return Some(result);
                }
                TextInputActionResult::Cancelled => {
                    result.cancel_rename = true;
                    self.modals.is_rename_active = false;
                    self.modals.clear_rename();
                    self.overlay_tree.close();
                    result.consumed = true;
                    return Some(result);
                }
                TextInputActionResult::Changed => {
                    result.consumed = true;
                    return Some(result);
                }
                TextInputActionResult::Ignored => {}
            }
        }

        // 3. Non-text modal dialogues (About / Delete Confirmation)
        if let WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    physical_key: winit::keyboard::PhysicalKey::Code(key),
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } = event
        {
            match *key {
                winit::keyboard::KeyCode::Escape => {
                    if self.modals.is_about_active {
                        result.close_about = true;
                        self.modals.is_about_active = false;
                    } else if self.modals.is_delete_active {
                        result.cancel_delete = true;
                        self.modals.is_delete_active = false;
                    }
                    self.overlay_tree.close();
                    result.consumed = true;
                    return Some(result);
                }
                winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter
                    if self.modals.is_delete_active =>
                {
                    result.confirm_delete = true;
                    self.modals.is_delete_active = false;
                    self.overlay_tree.close();
                    result.consumed = true;
                    return Some(result);
                }
                _ => {}
            }
        }

        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos();
            let hit_target = self
                .overlay_tree
                .tree()
                .hit_test_target(click_point)
                .or_else(|| self.tree.hit_test_target(click_point));

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
                            self.modals.is_about_active = false;
                            self.overlay_tree.close();
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
                            self.modals.is_delete_active = false;
                            self.overlay_tree.close();
                        }
                        ModalDialogAction::Close
                        | ModalDialogAction::Cancel
                        | ModalDialogAction::ScrimDismiss => {
                            result.cancel_delete = true;
                            self.modals.is_delete_active = false;
                            self.overlay_tree.close();
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
                            let folder = self.modals.new_folder_input.text().trim().to_string();
                            if !folder.is_empty() {
                                result.create_folder = Some(folder);
                                self.modals.is_new_folder_active = false;
                                self.modals.clear_new_folder();
                                self.overlay_tree.close();
                            }
                        }
                        ModalDialogAction::Close
                        | ModalDialogAction::Cancel
                        | ModalDialogAction::ScrimDismiss => {
                            result.cancel_new_folder = true;
                            self.modals.is_new_folder_active = false;
                            self.modals.clear_new_folder();
                            self.overlay_tree.close();
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
                            let rename = self.modals.rename_input.text().trim().to_string();
                            if !rename.is_empty() {
                                result.apply_rename = Some(rename);
                                self.modals.is_rename_active = false;
                                self.modals.clear_rename();
                                self.overlay_tree.close();
                            }
                        }
                        ModalDialogAction::Close
                        | ModalDialogAction::Cancel
                        | ModalDialogAction::ScrimDismiss => {
                            result.cancel_rename = true;
                            self.modals.is_rename_active = false;
                            self.modals.clear_rename();
                            self.overlay_tree.close();
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

        None
    }
}