// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Two-Way Mapped ComboBox Change Callback Combinators
//!
//! Extends [`UiScope`] with mapped dropdown rows that mutate a target value in-place
//! and immediately invoke an `on_change(&T)` handler only when the selection changes,
//! eliminating imperative `let resp = ...; if resp.changed()` boilerplate in UI views.

use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::input::ComboRowLayout;
use crate::declarative::widget::ComboBoxResponse;

/// Configuration descriptor for custom-layout mapped dropdown selection rows.
///
/// Groups the label, active option index, option captions, and [`ComboRowLayout`]
/// column widths cleanly without parameter explosion.
#[derive(Debug, Clone, Copy)]
pub struct ComboBoxMapConfig<'a> {
    /// Human-readable property description displayed on the left (supports `##` separator).
    pub label: &'a str,
    /// Zero-based index of the currently active option.
    pub current_idx: usize,
    /// Selectable option labels displayed inside the dropdown popup menu.
    pub options: &'a [&'a str],
    /// Label column width and dropdown trigger pill width ([`ComboRowLayout`]).
    pub layout: ComboRowLayout,
}

impl<'a> ComboBoxMapConfig<'a> {
    /// Constructs a mapped dropdown configuration using [`ComboRowLayout::default`].
    #[inline]
    #[must_use]
    pub fn new(label: &'a str, current_idx: usize, options: &'a [&'a str]) -> Self {
        Self {
            label,
            current_idx,
            options,
            layout: ComboRowLayout::default(),
        }
    }

    /// Overrides the label and button column widths for form alignment.
    #[inline]
    #[must_use]
    pub const fn with_layout(mut self, layout: ComboRowLayout) -> Self {
        self.layout = layout;
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a mapped two-way dropdown row and invokes `on_change` with the newly selected value.
    ///
    /// Mutates `*target = map_fn(idx)` in-place and runs `on_change(target)` only when the user
    /// selects an option index differing from `current_idx`.
    pub fn prop_combobox_mapped_on_change<T>(
        &mut self,
        label: &str,
        current_idx: usize,
        options: &[&str],
        target: &mut T,
        map_fn: impl FnOnce(usize) -> T,
        on_change: impl FnOnce(&T),
    ) -> ComboBoxResponse {
        self.prop_combobox_mapped_styled_on_change(
            ComboBoxMapConfig::new(label, current_idx, options),
            target,
            map_fn,
            on_change,
        )
    }

    /// Emits a custom-width mapped two-way dropdown row and invokes `on_change` upon selection change.
    ///
    /// Mutates `*target = map_fn(idx)` in-place and runs `on_change(target)` only when the user
    /// selects an option index differing from `config.current_idx`.
    pub fn prop_combobox_mapped_styled_on_change<T>(
        &mut self,
        config: ComboBoxMapConfig<'_>,
        target: &mut T,
        map_fn: impl FnOnce(usize) -> T,
        on_change: impl FnOnce(&T),
    ) -> ComboBoxResponse {
        let mut idx = config.current_idx;
        let resp = self.prop_combobox_styled(
            config.label,
            &mut idx,
            config.options,
            Some(config.layout.label_width),
            Some(config.layout.button_width),
        );
        if resp.changed() && idx != config.current_idx {
            *target = map_fn(idx);
            on_change(target);
        }
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarative::types::combine_seeds;
    use irisui_core::{InteractionEvent, MouseButton, UiTree};

    #[test]
    fn test_prop_combobox_mapped_on_change_updates_target_and_triggers_callback() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let trigger_tag = {
            let scope = UiScope::new(&mut tree, root);
            scope.tag_for("Theme Preset")
        };
        let option1_tag = combine_seeds(trigger_tag, 2);
        tree.active_combobox_tag = Some(trigger_tag);

        let events = [(
            option1_tag,
            InteractionEvent::Click {
                button: MouseButton::Left,
            },
        )];
        let mut theme_id = 0usize;
        let mut observed = None;

        {
            let mut scope = UiScope::with_tagged_interactions(&mut tree, root, &events, None);
            let resp = scope.prop_combobox_mapped_styled_on_change(
                ComboBoxMapConfig::new("Theme Preset", theme_id, &["Obsidian", "Cold Slate"])
                    .with_layout(ComboRowLayout::new(170.0, 180.0)),
                &mut theme_id,
                |idx| idx * 10,
                |&val| observed = Some(val),
            );
            assert!(resp.changed());
        }

        assert_eq!(theme_id, 10);
        assert_eq!(observed, Some(10));
    }
}