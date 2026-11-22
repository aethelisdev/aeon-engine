// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Two-Way Bound Text Input Field (`TextInputField`)
//!
//! Eliminates consumer `auto_sync_text_input` boilerplate, manual clear-button tag
//! allocation, and `InputBoxProps` field unpacking by binding a [`TextInputState`]
//! directly inside a fluent [`Widget`] builder.

use super::types::{InputBoxProps, InputBoxStyle};
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::{ResponseFlags, WidgetResponse};
use crate::declarative::widget::Widget;
use crate::input::TextInputState;
use irisui_core::UiTag;

/// Two-way bound single-line text input widget syncing directly with [`TextInputState`].
///
/// Automatically resolves deterministic input and clear-button tags via [`UiScope`],
/// synchronizes active typing/selection buffers, clears text + focus when the clear
/// affordance is clicked, and inherits caret blink visibility from [`UiScope`].
#[must_use = "widgets do nothing unless passed to scope.add(...)"]
pub struct TextInputField<'a> {
    state: &'a mut TextInputState,
    placeholder: &'a str,
    width: f32,
    icon: Option<&'a str>,
    font_size: f32,
    clearable: bool,
    tag: Option<u64>,
    clear_tag: Option<u64>,
    blink_caret: Option<bool>,
    style: Option<InputBoxStyle>,
    clipboard_shortcuts: bool,
    undo_shortcuts: bool,
    external_text: Option<&'a str>,
}

impl<'a> TextInputField<'a> {
    /// Creates a new two-way bound text input field backed by `state`.
    #[inline]
    pub fn new(state: &'a mut TextInputState) -> Self {
        Self {
            state,
            placeholder: "",
            width: 0.0,
            icon: None,
            font_size: 11.0,
            clearable: true,
            tag: None,
            clear_tag: None,
            blink_caret: None,
            style: None,
            clipboard_shortcuts: true,
            undo_shortcuts: true,
            external_text: None,
        }
    }

    /// Synchronizes `state` from `external` whenever this field does not hold active keyboard focus.
    #[inline]
    pub const fn sync_external(mut self, external: &'a str) -> Self {
        self.external_text = Some(external);
        self
    }

    /// Sets the placeholder text displayed when the input buffer is empty.
    #[inline]
    pub const fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Sets an explicit logical width (`<= 0.0` uses `flex_grow(1.0)`).
    #[inline]
    pub const fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets a leading icon glyph inside the left edge of the text field.
    #[inline]
    pub const fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Applies the standard search magnifier icon (`"🔍"`).
    #[inline]
    pub const fn search_icon(mut self) -> Self {
        self.icon = Some("🔍");
        self
    }

    /// Overrides the text font size in logical points.
    #[inline]
    pub const fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Controls whether an inline `"✖"` clear button appears when non-empty.
    #[inline]
    pub const fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    /// Sets an explicit 64-bit semantic tag for focus and text edit tracking.
    #[inline]
    pub const fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Overrides the derived semantic tag used by the inline clear button.
    #[inline]
    pub const fn clear_tag(mut self, clear_tag: u64) -> Self {
        self.clear_tag = Some(clear_tag);
        self
    }

    /// Overrides the caret blink visibility flag inherited from [`UiScope`].
    #[inline]
    pub const fn blink_caret(mut self, visible: bool) -> Self {
        self.blink_caret = Some(visible);
        self
    }

    /// Applies custom [`InputBoxStyle`] tokens overriding active theme defaults.
    #[inline]
    pub const fn style(mut self, style: InputBoxStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Enables or disables clipboard keyboard shortcuts (`Ctrl+A/C/X/V`).
    #[inline]
    pub const fn clipboard_shortcuts(mut self, enabled: bool) -> Self {
        self.clipboard_shortcuts = enabled;
        self
    }

    /// Enables or disables text edit undo/redo keyboard shortcuts (`Ctrl+Z/Y`).
    #[inline]
    pub const fn undo_shortcuts(mut self, enabled: bool) -> Self {
        self.undo_shortcuts = enabled;
        self
    }
}

impl Widget for TextInputField<'_> {
    type Response = WidgetResponse;

    fn ui(self, scope: &mut UiScope<'_>) -> Self::Response {
        let input_tag = self.tag.unwrap_or_else(|| {
            if self.placeholder.is_empty() {
                scope.tag_for("TextInputField")
            } else {
                scope.tag_for(self.placeholder)
            }
        });
        let derived_clear = UiTag(input_tag).child("clear").raw();
        let clear_tag = self
            .clearable
            .then_some(self.clear_tag.unwrap_or(derived_clear));

        let cleared = clear_tag.is_some_and(|c_tag| scope.is_tag_clicked(c_tag));
        let mut text_changed = if cleared {
            let had_content = !self.state.text().is_empty()
                || scope
                    .tree
                    .event_state
                    .active_text_edit
                    .as_ref()
                    .is_some_and(|s| s.tag == input_tag);
            self.state.clear();
            scope.clear_focus();
            scope.clear_text_edit();
            had_content
        } else {
            false
        };

        let is_focused = scope.is_tag_focused(input_tag);
        if !cleared
            && !is_focused
            && let Some(ext) = self.external_text
            && self.state.text() != ext
        {
            self.state.set_text(ext);
            self.state.cursor_byte_idx = ext.len();
            self.state.selection = None;
        }

        if !cleared
            && let Some((next_text, cur, sel)) = scope
                .tree
                .event_state
                .active_text_edit
                .as_ref()
                .filter(|s| s.tag == input_tag)
                .map(|s| {
                    let changed = s.buffer.as_str() != self.state.text();
                    (
                        changed.then(|| s.buffer.clone()),
                        s.cursor_byte_idx,
                        s.selection,
                    )
                })
        {
            if let Some(buf) = next_text {
                self.state.set_text(buf);
                text_changed = true;
            }
            self.state.cursor_byte_idx = cur;
            self.state.selection = sel;
        }

        let blink_visible = self.blink_caret.unwrap_or(scope.active_text_input_blink);

        let mut props = InputBoxProps::new(
            self.state.text(),
            self.placeholder,
            self.width,
            is_focused,
            blink_visible,
        )
        .with_font_size(self.font_size)
        .with_cursor_byte_idx(self.state.cursor_byte_idx)
        .with_selection(self.state.selection)
        .with_clipboard_shortcuts(self.clipboard_shortcuts)
        .with_undo_shortcuts(self.undo_shortcuts);

        if let Some(icon) = self.icon {
            props = props.with_icon(icon);
        }
        if let Some(ct) = clear_tag {
            props = props.with_clear_tag(ct);
        }
        if let Some(style) = self.style {
            props = props.with_style(style);
        }

        let mut resp = scope.input_box_tagged(&props, input_tag);

        if !cleared
            && let Some((next_text, cur, sel)) = scope
                .tree
                .event_state
                .active_text_edit
                .as_ref()
                .filter(|s| s.tag == input_tag)
                .map(|s| {
                    let changed = s.buffer.as_str() != self.state.text();
                    (
                        changed.then(|| s.buffer.clone()),
                        s.cursor_byte_idx,
                        s.selection,
                    )
                })
        {
            if let Some(buf) = next_text {
                self.state.set_text(buf);
                text_changed = true;
            }
            self.state.cursor_byte_idx = cur;
            self.state.selection = sel;
        }

        if text_changed {
            resp.flags = resp.flags.union(ResponseFlags::CHANGED);
        }
        resp
    }
}

impl UiScope<'_> {
    /// Emits a two-way bound single-line text input field synced directly with `state`.
    ///
    /// Eliminates manual `auto_sync_text_input` and `InputBoxProps` boilerplate while
    /// preserving deterministic child tags via [`UiScope::next_tag`].
    #[inline]
    pub fn text_input_field(
        &mut self,
        state: &mut TextInputState,
        placeholder: &str,
        width: f32,
    ) -> WidgetResponse {
        self.add(
            TextInputField::new(state)
                .placeholder(placeholder)
                .width(width),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::UiTree;

    #[test]
    fn test_bound_text_input_field_syncs_typing_and_clear_click() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut state = TextInputState::new("hello");
        let input_tag = {
            let scope = UiScope::new(&mut tree, root);
            scope.tag_for("Search assets...")
        };
        let clear_tag = UiTag(input_tag).child("clear").raw();

        // Active text edit updates state and reports CHANGED
        tree.event_state
            .start_text_edit(None, input_tag, "world", 5, None);
        {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.add(
                TextInputField::new(&mut state)
                    .placeholder("Search assets...")
                    .width(200.0)
                    .search_icon(),
            );
            assert!(resp.changed());
            assert_eq!(state.text(), "world");
            assert_eq!(state.cursor_byte_idx, 5);
        }

        // Idle focused frame with unchanged buffer must NOT spuriously report CHANGED
        {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.add(
                TextInputField::new(&mut state)
                    .placeholder("Search assets...")
                    .width(200.0)
                    .search_icon(),
            );
            assert!(!resp.changed());
            assert_eq!(state.text(), "world");
        }

        // sync_external respects focus and synchronizes when unfocused
        {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.add(
                TextInputField::new(&mut state)
                    .placeholder("Search assets...")
                    .width(200.0)
                    .sync_external("from_model"),
            );
            assert!(!resp.changed());
            assert_eq!(state.text(), "world");
        }

        // Clicking derived clear button resets state and focus
        tree.event_state.frame_clicked_tag = Some(clear_tag);
        {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.add(
                TextInputField::new(&mut state)
                    .placeholder("Search assets...")
                    .width(200.0)
                    .search_icon(),
            );
            assert!(resp.changed());
            assert_eq!(state.text(), "");
            assert!(!scope.has_text_input_focus());
        }

        // Once unfocused, sync_external adopts model text without flagging CHANGED
        tree.event_state.frame_clicked_tag = None;
        {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.add(
                TextInputField::new(&mut state)
                    .placeholder("Search assets...")
                    .width(200.0)
                    .sync_external("from_model"),
            );
            assert!(!resp.changed());
            assert_eq!(state.text(), "from_model");
            assert_eq!(state.cursor_byte_idx, "from_model".len());
        }
    }
}