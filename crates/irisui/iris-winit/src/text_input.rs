// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Winit Keyboard & IME Text Input Handler (`TextInputState`)
//!
//! Provides unified typing, caret navigation, clipboard operations (Ctrl+A, Ctrl+C, Ctrl+V, Ctrl+X),
//! text selection, and focus management integration for interactive text inputs.

use iris_core::event::FocusManager;
use iris_core::platform::ClipboardProvider;
use iris_widgets::controls::focus::{FocusState, FocusTarget};
use iris_widgets::input::{TextInputState, copy_to_clipboard, get_clipboard_text};
use winit::event::{ElementState, Ime, KeyEvent, WindowEvent};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};

/// Result of evaluating a keyboard event against an active text input state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInputActionResult {
    /// The event was not applicable or ignored by the input.
    Ignored,
    /// The text buffer or cursor position was modified.
    Changed,
    /// Enter was pressed confirming the input session.
    Confirmed,
    /// Escape was pressed cancelling the input session.
    Cancelled,
}

impl TextInputActionResult {
    /// Returns `true` if the event was consumed by the text input.
    #[must_use]
    #[inline]
    pub const fn is_consumed(self) -> bool {
        !matches!(self, Self::Ignored)
    }

    /// Returns `true` if the text buffer content or cursor changed.
    #[must_use]
    #[inline]
    pub const fn is_changed(self) -> bool {
        matches!(self, Self::Changed)
    }
}

/// Evaluates a window event for a specific focused semantic tag.
///
/// If the tag holds focus in the provided `FocusState`, dispatches the keyboard event
/// through [`handle_text_input_event`]. On `Confirmed` or `Cancelled`, clears active focus
/// and synchronizes the core focus manager.
pub fn handle_focused_text_input(
    state: &mut TextInputState,
    focus_state: &mut FocusState<FocusTarget>,
    focus_manager: &mut FocusManager,
    target_tag: u64,
    event: &WindowEvent,
    ctrl_held: bool,
) -> TextInputActionResult {
    if !focus_state.is_tag_focused(target_tag) {
        return TextInputActionResult::Ignored;
    }

    let result = handle_text_input_event(state, event, ctrl_held);
    match result {
        TextInputActionResult::Confirmed | TextInputActionResult::Cancelled => {
            focus_state.clear_focus();
            focus_state.sync_to_core(focus_manager);
        }
        _ => {}
    }
    result
}

/// Evaluates a key stroke and optional text payload directly against an interactive [`TextInputState`],
/// routing clipboard operations (`Ctrl+C`, `Ctrl+X`, `Ctrl+V`) through an explicit [`ClipboardProvider`].
pub fn handle_key_stroke_with_clipboard(
    state: &mut TextInputState,
    logical_key: &Key,
    physical_key: Option<KeyCode>,
    text: Option<&str>,
    ctrl_held: bool,
    clipboard: &impl ClipboardProvider,
) -> TextInputActionResult {
    // 1. Control key shortcuts
    if ctrl_held {
        let is_key_a = physical_key == Some(KeyCode::KeyA)
            || matches!(logical_key, Key::Character(c) if c.eq_ignore_ascii_case("a"));
        let is_key_c = physical_key == Some(KeyCode::KeyC)
            || matches!(logical_key, Key::Character(c) if c.eq_ignore_ascii_case("c"));
        let is_key_x = physical_key == Some(KeyCode::KeyX)
            || matches!(logical_key, Key::Character(c) if c.eq_ignore_ascii_case("x"));
        let is_key_v = physical_key == Some(KeyCode::KeyV)
            || matches!(logical_key, Key::Character(c) if c.eq_ignore_ascii_case("v"));

        if is_key_a {
            state.select_all();
            return TextInputActionResult::Changed;
        }
        if is_key_c {
            let _ = state.copy_selection_with_provider(clipboard);
            return TextInputActionResult::Changed;
        }
        if is_key_x {
            if state.cut_selection_with_provider(clipboard).is_some() {
                return TextInputActionResult::Changed;
            }
            return TextInputActionResult::Changed;
        }
        if is_key_v {
            if let Some(content) = clipboard.get_text() {
                state.paste(&content);
                return TextInputActionResult::Changed;
            }
            return TextInputActionResult::Changed;
        }
    }

    // 2. Named structural keys
    match logical_key {
        Key::Named(NamedKey::Escape) => TextInputActionResult::Cancelled,
        Key::Named(NamedKey::Enter) => TextInputActionResult::Confirmed,
        Key::Named(NamedKey::Backspace) => {
            state.backspace();
            TextInputActionResult::Changed
        }
        Key::Named(NamedKey::Delete) => {
            state.delete();
            TextInputActionResult::Changed
        }
        Key::Named(NamedKey::ArrowLeft) => {
            state.move_left();
            TextInputActionResult::Changed
        }
        Key::Named(NamedKey::ArrowRight) => {
            state.move_right();
            TextInputActionResult::Changed
        }
        Key::Named(NamedKey::Home) => {
            state.move_home();
            TextInputActionResult::Changed
        }
        Key::Named(NamedKey::End) => {
            state.move_end();
            TextInputActionResult::Changed
        }
        _ => {
            // Check physical KeyCode for NumpadEnter fallback
            if physical_key == Some(KeyCode::NumpadEnter) {
                return TextInputActionResult::Confirmed;
            }

            // 3. Printable character input
            if let Some(t) = text
                && !t.chars().any(|c| c.is_control())
            {
                state.insert_str(t);
                TextInputActionResult::Changed
            } else {
                TextInputActionResult::Ignored
            }
        }
    }
}

/// Evaluates a key stroke and optional text payload directly against an interactive [`TextInputState`].
pub fn handle_key_stroke(
    state: &mut TextInputState,
    logical_key: &Key,
    physical_key: Option<KeyCode>,
    text: Option<&str>,
    ctrl_held: bool,
) -> TextInputActionResult {
    struct SystemFallback;
    impl ClipboardProvider for SystemFallback {
        fn get_text(&self) -> Option<String> {
            get_clipboard_text()
        }
        fn set_text(&self, text: &str) -> bool {
            copy_to_clipboard(text);
            true
        }
    }

    handle_key_stroke_with_clipboard(
        state,
        logical_key,
        physical_key,
        text,
        ctrl_held,
        &SystemFallback,
    )
}

/// Dispatches raw keyboard and IME window events into an interactive [`TextInputState`].
///
/// Handles:
/// - Navigation: ArrowLeft, ArrowRight, Home, End
/// - Deletion: Backspace, Delete
/// - Shortcuts: Ctrl+A (Select All), Ctrl+C (Copy), Ctrl+X (Cut), Ctrl+V (Paste)
/// - Confirmation: Enter, NumpadEnter
/// - Cancellation: Escape
/// - Typing: Printable character text insertion and IME commit
pub fn handle_text_input_event(
    state: &mut TextInputState,
    event: &WindowEvent,
    ctrl_held: bool,
) -> TextInputActionResult {
    match event {
        WindowEvent::Ime(Ime::Commit(text)) => {
            state.insert_str(text);
            TextInputActionResult::Changed
        }
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key,
                    logical_key,
                    text,
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } => {
            let code_opt = match physical_key {
                PhysicalKey::Code(code) => Some(*code),
                _ => None,
            };
            handle_key_stroke(state, logical_key, code_opt, text.as_deref(), ctrl_held)
        }
        _ => TextInputActionResult::Ignored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::{Key, KeyCode, NamedKey};

    #[test]
    fn test_text_input_dispatch_typing() {
        let mut state = TextInputState::new("");
        let res = handle_key_stroke(
            &mut state,
            &Key::Character("h".into()),
            Some(KeyCode::KeyH),
            Some("h"),
            false,
        );
        assert_eq!(res, TextInputActionResult::Changed);
        assert_eq!(state.buffer, "h");

        let res_enter = handle_key_stroke(
            &mut state,
            &Key::Named(NamedKey::Enter),
            Some(KeyCode::Enter),
            None,
            false,
        );
        assert_eq!(res_enter, TextInputActionResult::Confirmed);
    }

    #[test]
    fn test_text_input_dispatch_navigation_and_deletion() {
        let mut state = TextInputState::new("test");
        assert_eq!(state.cursor_byte_idx, 4);

        handle_key_stroke(
            &mut state,
            &Key::Named(NamedKey::Home),
            Some(KeyCode::Home),
            None,
            false,
        );
        assert_eq!(state.cursor_byte_idx, 0);

        handle_key_stroke(
            &mut state,
            &Key::Named(NamedKey::ArrowRight),
            Some(KeyCode::ArrowRight),
            None,
            false,
        );
        assert_eq!(state.cursor_byte_idx, 1);

        handle_key_stroke(
            &mut state,
            &Key::Named(NamedKey::Delete),
            Some(KeyCode::Delete),
            None,
            false,
        );
        assert_eq!(state.buffer, "tst");
    }

    #[test]
    fn test_text_input_dispatch_ctrl_shortcuts() {
        let mut state = TextInputState::new("sample");
        handle_key_stroke(
            &mut state,
            &Key::Character("a".into()),
            Some(KeyCode::KeyA),
            None,
            true,
        );
        assert!(state.has_selection());
        assert_eq!(state.selected_text(), Some("sample"));

        handle_key_stroke(
            &mut state,
            &Key::Character("c".into()),
            Some(KeyCode::KeyC),
            None,
            true,
        );
        assert_eq!(get_clipboard_text(), Some("sample".to_string()));

        state.clear();
        handle_key_stroke(
            &mut state,
            &Key::Character("v".into()),
            Some(KeyCode::KeyV),
            None,
            true,
        );
        assert_eq!(state.buffer, "sample");
    }

    #[test]
    fn test_text_input_dispatch_ctrl_cut_with_provider() {
        use iris_core::platform::InMemoryClipboard;

        let clipboard = InMemoryClipboard::new();
        let mut state = TextInputState::new("CutTest");
        state.select_all();

        let res_cut = handle_key_stroke_with_clipboard(
            &mut state,
            &Key::Character("x".into()),
            Some(KeyCode::KeyX),
            None,
            true,
            &clipboard,
        );
        assert_eq!(res_cut, TextInputActionResult::Changed);
        assert_eq!(state.buffer, "");
        assert_eq!(clipboard.get_text(), Some("CutTest".to_owned()));

        let res_paste = handle_key_stroke_with_clipboard(
            &mut state,
            &Key::Character("v".into()),
            Some(KeyCode::KeyV),
            None,
            true,
            &clipboard,
        );
        assert_eq!(res_paste, TextInputActionResult::Changed);
        assert_eq!(state.buffer, "CutTest");
    }

    #[test]
    fn test_focused_text_input_confirmation_clears_focus() {
        let mut state = TextInputState::new("query");
        let mut focus_state = FocusState::new();
        let mut focus_manager = FocusManager::new();
        let target_tag = 0x1234;
        focus_state.set_focus_tag(target_tag);
        focus_state.sync_to_core(&mut focus_manager);

        let event = WindowEvent::Ime(Ime::Commit("!".into()));
        let res = handle_focused_text_input(
            &mut state,
            &mut focus_state,
            &mut focus_manager,
            target_tag,
            &event,
            false,
        );
        assert_eq!(res, TextInputActionResult::Changed);
        assert_eq!(state.buffer, "query!");
        assert!(focus_state.is_tag_focused(target_tag));
        assert!(focus_manager.is_tag_focused(target_tag));
    }
}