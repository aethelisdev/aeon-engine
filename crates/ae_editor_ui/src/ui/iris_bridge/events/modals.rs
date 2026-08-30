// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event routing logic for About, Delete, New Folder, and Rename modal dialogs.

use super::super::about;
use super::super::types::{IrisEditorOverlay, IrisOverlayEventResult};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles keyboard and click interactions for active modal dialogues.
    pub(crate) fn handle_modal_events(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let mut result = IrisOverlayEventResult::default();

        // 1. If About modal is active
        if let Some(ref targets) = self.about_targets {
            match event {
                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    if *key == winit::keyboard::KeyCode::Escape {
                        result.close_about = true;
                        result.consumed = true;
                        return Some(result);
                    }
                }
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: WinitMouseButton::Left,
                    ..
                } => {
                    let click_point = self.cursor_pos;
                    if targets.header_close_rect.contains_point(click_point)
                        || targets.bottom_close_rect.contains_point(click_point)
                    {
                        result.close_about = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    if targets.link_rect.contains_point(click_point) {
                        about::open_url("https://mozilla.org/MPL/2.0/");
                        result.consumed = true;
                        return Some(result);
                    }
                    if !targets.dialog_rect.contains_point(click_point) {
                        result.close_about = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    result.consumed = true;
                    return Some(result);
                }
                _ => {}
            }
        }

        // 2. If Delete Confirmation modal is active
        if let Some(ref targets) = self.delete_targets {
            match event {
                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match *key {
                    winit::keyboard::KeyCode::Escape => {
                        result.cancel_delete = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                        result.confirm_delete = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    _ => {}
                },
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: WinitMouseButton::Left,
                    ..
                } => {
                    let click_point = self.cursor_pos;
                    if targets.header_close_rect.contains_point(click_point)
                        || targets.cancel_btn_rect.contains_point(click_point)
                    {
                        result.cancel_delete = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    if targets.confirm_btn_rect.contains_point(click_point) {
                        result.confirm_delete = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    if !targets.dialog_rect.contains_point(click_point) {
                        result.cancel_delete = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    result.consumed = true;
                    return Some(result);
                }
                _ => {}
            }
        }

        // 3. If New Folder modal is active
        if let Some(ref targets) = self.new_folder_targets {
            match event {
                WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                    self.new_folder_buffer.push_str(text);
                    result.consumed = true;
                    return Some(result);
                }
                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key),
                            text,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match *key {
                    winit::keyboard::KeyCode::Escape => {
                        result.cancel_new_folder = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                        if !self.new_folder_buffer.trim().is_empty() {
                            result.create_folder = Some(self.new_folder_buffer.clone());
                        }
                        result.consumed = true;
                        return Some(result);
                    }
                    winit::keyboard::KeyCode::Backspace => {
                        self.new_folder_buffer.pop();
                        result.consumed = true;
                        return Some(result);
                    }
                    _ => {
                        if let Some(t) = text
                            && !t.chars().any(|c| c.is_control())
                        {
                            self.new_folder_buffer.push_str(t);
                            result.consumed = true;
                            return Some(result);
                        }
                    }
                },
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: WinitMouseButton::Left,
                    ..
                } => {
                    let click_point = self.cursor_pos;
                    if targets.header_close_rect.contains_point(click_point)
                        || targets.cancel_btn_rect.contains_point(click_point)
                    {
                        result.cancel_new_folder = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    if targets.confirm_btn_rect.contains_point(click_point) {
                        if !self.new_folder_buffer.trim().is_empty() {
                            result.create_folder = Some(self.new_folder_buffer.clone());
                        }
                        result.consumed = true;
                        return Some(result);
                    }
                    if !targets.dialog_rect.contains_point(click_point) {
                        result.cancel_new_folder = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    result.consumed = true;
                    return Some(result);
                }
                _ => {}
            }
        }

        // 4. If Rename modal is active
        if let Some(ref targets) = self.rename_targets {
            match event {
                WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                    self.rename_buffer.push_str(text);
                    result.consumed = true;
                    return Some(result);
                }
                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key),
                            text,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match *key {
                    winit::keyboard::KeyCode::Escape => {
                        result.cancel_rename = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                        if !self.rename_buffer.trim().is_empty() {
                            result.apply_rename = Some(self.rename_buffer.clone());
                        }
                        result.consumed = true;
                        return Some(result);
                    }
                    winit::keyboard::KeyCode::Backspace => {
                        self.rename_buffer.pop();
                        result.consumed = true;
                        return Some(result);
                    }
                    _ => {
                        if let Some(t) = text
                            && !t.chars().any(|c| c.is_control())
                        {
                            self.rename_buffer.push_str(t);
                            result.consumed = true;
                            return Some(result);
                        }
                    }
                },
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: WinitMouseButton::Left,
                    ..
                } => {
                    let click_point = self.cursor_pos;
                    if targets.header_close_rect.contains_point(click_point)
                        || targets.cancel_btn_rect.contains_point(click_point)
                    {
                        result.cancel_rename = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    if targets.confirm_btn_rect.contains_point(click_point) {
                        if !self.rename_buffer.trim().is_empty() {
                            result.apply_rename = Some(self.rename_buffer.clone());
                        }
                        result.consumed = true;
                        return Some(result);
                    }
                    if !targets.dialog_rect.contains_point(click_point) {
                        result.cancel_rename = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    result.consumed = true;
                    return Some(result);
                }
                _ => {}
            }
        }

        None
    }
}