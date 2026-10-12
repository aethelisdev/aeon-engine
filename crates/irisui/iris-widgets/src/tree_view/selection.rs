// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchical TreeView Selection, Chevron Toggles & Styling Engine
//!
//! Provides autonomous event handling for node expansion toggles, multi-selection gestures
//! (single, Ctrl-toggle, Shift-range, and Ctrl+Shift-range), and capsule row styling for tree views.
//!
//! ## Architectural Guarantees
//! - **Autonomous Expansion & Selection**: Evaluates user clicks and modifier keys without
//!   requiring repetitive boilerplate in host panels.
//! - **Zero Per-Frame Allocation**: Operates purely on references, ID handles, and fixed POD structures.
//! - **Standard Industrial UX**: Adheres strictly to desktop and game-engine tree-view conventions
//!   for hierarchical navigation and multi-node picking.
//!

use super::model::TreeState;
use iris_core::Color;
use iris_core::event::{MouseButton, UiEvent};
use iris_core::style::Style;
use std::hash::Hash;

/// Standard unicode glyph representing an expanded tree node folder/branch (`"▼"`).
pub const TREE_CHEVRON_EXPANDED: &str = "▼";

/// Standard unicode glyph representing a collapsed tree node folder/branch (`"▶"`).
pub const TREE_CHEVRON_COLLAPSED: &str = "▶";

/// Compact micro-arrow glyph representing an expanded tree branch (`"▾"`).
pub const TREE_CHEVRON_ARROW_EXPANDED: &str = "▾";

/// Compact micro-arrow glyph representing a collapsed tree branch (`"▸"`).
pub const TREE_CHEVRON_ARROW_COLLAPSED: &str = "▸";

/// Returns the standard unicode chevron glyph (`"▼"` or `"▶"`) corresponding to `is_expanded`.
#[inline]
pub const fn tree_chevron_glyph(is_expanded: bool) -> &'static str {
    if is_expanded {
        TREE_CHEVRON_EXPANDED
    } else {
        TREE_CHEVRON_COLLAPSED
    }
}

/// Returns the compact micro-arrow glyph (`"▾"` or `"▸"`) corresponding to `is_expanded`.
#[inline]
pub const fn tree_chevron_arrow_glyph(is_expanded: bool) -> &'static str {
    if is_expanded {
        TREE_CHEVRON_ARROW_EXPANDED
    } else {
        TREE_CHEVRON_ARROW_COLLAPSED
    }
}

/// Active keyboard modifier flags present during a tree selection interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TreeSelectionModifiers {
    /// True if the Control (or Command on macOS) key is held down.
    pub ctrl: bool,
    /// True if the Shift key is held down.
    pub shift: bool,
    /// True if the Alt (Option) key is held down.
    pub alt: bool,
}

impl TreeSelectionModifiers {
    /// Creates a new modifier record with all modifier keys inactive.
    #[inline]
    pub const fn new() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    /// Sets the Control modifier status.
    #[inline]
    pub const fn with_ctrl(mut self, ctrl: bool) -> Self {
        self.ctrl = ctrl;
        self
    }

    /// Sets the Shift modifier status.
    #[inline]
    pub const fn with_shift(mut self, shift: bool) -> Self {
        self.shift = shift;
        self
    }

    /// Sets the Alt modifier status.
    #[inline]
    pub const fn with_alt(mut self, alt: bool) -> Self {
        self.alt = alt;
        self
    }

    /// Creates a modifier record from explicit Ctrl and Shift states.
    #[inline]
    pub const fn from_ctrl_shift(ctrl: bool, shift: bool) -> Self {
        Self {
            ctrl,
            shift,
            alt: false,
        }
    }

    /// Returns `true` if no modifier keys are held.
    #[inline]
    pub const fn is_none(&self) -> bool {
        !self.ctrl && !self.shift && !self.alt
    }

    /// Returns `true` if either Ctrl or Shift is held down for multi-selection operations.
    #[inline]
    pub const fn is_multi_select(&self) -> bool {
        self.ctrl || self.shift
    }
}

/// Identifies the sub-target within a tree node row that was clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeClickTarget<Id> {
    /// Click occurred directly on the expansion chevron / foldout button ("▸" / "▾").
    Foldout(Id),
    /// Click occurred on the node row item body (label, icon, or padding).
    Row(Id),
}

/// Evaluated outcome of a tree view interaction (click, chevron toggle, or selection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TreeSelectionResponse {
    /// True if the interaction was handled and should not propagate to lower layers.
    pub consumed: bool,
    /// True if an item's expanded state was toggled, requiring hierarchy re-flattening.
    pub layout_changed: bool,
    /// True if the selection set or primary selected node changed.
    pub selection_changed: bool,
}

impl TreeSelectionResponse {
    /// Returns an empty response indicating no action occurred.
    #[inline]
    pub const fn none() -> Self {
        Self {
            consumed: false,
            layout_changed: false,
            selection_changed: false,
        }
    }

    /// Returns an unhandled response that was ignored by the tree view.
    #[inline]
    pub const fn unhandled() -> Self {
        Self::none()
    }

    /// Constructs a handled response specifying whether layout or selection changed.
    #[inline]
    pub const fn handled(layout_changed: bool, selection_changed: bool) -> Self {
        Self {
            consumed: true,
            layout_changed,
            selection_changed,
        }
    }
}

/// Handles a click directly on the foldout chevron button of a tree node.
///
/// Toggles the node's expansion state in [`TreeState`], signaling that the hierarchy layout
/// has changed and requires re-flattening.
pub fn handle_tree_foldout_click<Id>(state: &mut TreeState<Id>, id: &Id) -> TreeSelectionResponse
where
    Id: Eq + Hash + Clone,
{
    state.toggle_expanded(id);
    TreeSelectionResponse::handled(true, false)
}

/// Handles a click on a tree row item, evaluating single, Ctrl-toggle, Shift-range,
/// or Ctrl+Shift-range multi-selection logic.
///
/// # Arguments
/// - `state`: The mutable tree state container.
/// - `id`: The identifier of the clicked node item.
/// - `modifiers`: Current state of keyboard modifiers (Ctrl, Shift, etc.).
/// - `visible_order`: Optional slice of visible/flattened item IDs used for contiguous Shift range calculations.
pub fn handle_tree_row_click<Id>(
    state: &mut TreeState<Id>,
    id: Id,
    modifiers: TreeSelectionModifiers,
    visible_order: Option<&[Id]>,
) -> TreeSelectionResponse
where
    Id: Eq + Hash + Clone,
{
    let selection_changed = if modifiers.ctrl && modifiers.shift {
        if let Some(order) = visible_order {
            state.select_range_additive(id, order)
        } else {
            state.select_toggle(id)
        }
    } else if modifiers.ctrl {
        state.select_toggle(id);
        true
    } else if modifiers.shift {
        if let Some(order) = visible_order {
            state.select_range(id, order)
        } else {
            state.select_single(id);
            true
        }
    } else {
        state.select_single(id);
        true
    };

    TreeSelectionResponse::handled(false, selection_changed)
}

/// Evaluates a high-level tree click target against the tree state.
///
/// Automatically routes foldout clicks to toggle expansion and row clicks to selection logic.
pub fn evaluate_tree_click<Id>(
    state: &mut TreeState<Id>,
    target: TreeClickTarget<Id>,
    modifiers: TreeSelectionModifiers,
    visible_order: Option<&[Id]>,
) -> TreeSelectionResponse
where
    Id: Eq + Hash + Clone,
{
    match target {
        TreeClickTarget::Foldout(id) => handle_tree_foldout_click(state, &id),
        TreeClickTarget::Row(id) => handle_tree_row_click(state, id, modifiers, visible_order),
    }
}

/// Evaluates an incoming [`UiEvent`] against a target tree item.
///
/// Consumes left mouse button press events and performs the corresponding expansion
/// or selection state transition.
pub fn handle_tree_selection_event<Id>(
    state: &mut TreeState<Id>,
    event: &UiEvent,
    target: TreeClickTarget<Id>,
    modifiers: TreeSelectionModifiers,
    visible_order: Option<&[Id]>,
) -> TreeSelectionResponse
where
    Id: Eq + Hash + Clone,
{
    match event {
        UiEvent::MouseDown {
            button: MouseButton::Left,
            ..
        } => evaluate_tree_click(state, target, modifiers, visible_order),
        _ => TreeSelectionResponse::unhandled(),
    }
}

/// Visual styling configuration for tree view row items and active selection capsules.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeRowStyle {
    /// Background color of an idle, unselected row item.
    pub normal_bg: Color,
    /// Border stroke color of an idle, unselected row item.
    pub normal_border: Color,
    /// Background color when hovered and unselected.
    pub hover_bg: Color,
    /// Border stroke color when hovered and unselected.
    pub hover_border: Color,
    /// Background color when selected (active pill capsule background).
    pub selected_bg: Color,
    /// Border stroke color when selected (active highlight border).
    pub selected_border: Color,
    /// Corner radius applied to the selection capsule pill in physical pixels.
    pub border_radius: f32,
    /// Stroke width of the row border in physical pixels.
    pub border_width: f32,
    /// Default text and icon tint color when unselected.
    pub text_color: Color,
    /// Text and icon tint color when hovered.
    pub hover_text_color: Color,
    /// Text and icon tint color when actively selected.
    pub selected_text_color: Color,
    /// Color of the foldout chevron glyph when unselected.
    pub chevron_color: Color,
    /// Color of the foldout chevron glyph when hovered.
    pub chevron_hover_color: Color,
    /// Color of the foldout chevron glyph when selected.
    pub chevron_selected_color: Color,
}

impl Default for TreeRowStyle {
    #[inline]
    fn default() -> Self {
        Self::dark_default()
    }
}

impl TreeRowStyle {
    /// Creates a tree row style configured with the engine's standard dark theme palette.
    ///
    /// Matches the cyan selection pill and dark charcoal hover aesthetics used across engine panels.
    #[inline]
    pub fn dark_default() -> Self {
        Self {
            normal_bg: Color::TRANSPARENT,
            normal_border: Color::TRANSPARENT,
            hover_bg: Color::rgba(0.14, 0.18, 0.26, 0.65),
            hover_border: Color::rgba(0.24, 0.32, 0.45, 0.50),
            selected_bg: Color::rgba(0.0, 0.35, 0.50, 0.65),
            selected_border: Color::rgba(0.0, 0.85, 1.0, 0.80),
            border_radius: 4.0,
            border_width: 1.0,
            text_color: Color::WHITE,
            hover_text_color: Color::rgba(0.95, 0.96, 1.0, 1.0),
            selected_text_color: Color::rgba(0.0, 0.95, 1.0, 1.0),
            chevron_color: Color::rgba(0.65, 0.68, 0.78, 1.0),
            chevron_hover_color: Color::WHITE,
            chevron_selected_color: Color::WHITE,
        }
    }

    /// Overrides the selected pill capsule background color.
    #[inline]
    pub fn with_selected_bg(mut self, color: Color) -> Self {
        self.selected_bg = color;
        self
    }

    /// Overrides the selected pill capsule border stroke color.
    #[inline]
    pub fn with_selected_border(mut self, color: Color) -> Self {
        self.selected_border = color;
        self
    }

    /// Overrides the hover background color for unselected items.
    #[inline]
    pub fn with_hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    /// Overrides the hover border color for unselected items.
    #[inline]
    pub fn with_hover_border(mut self, color: Color) -> Self {
        self.hover_border = color;
        self
    }

    /// Overrides the corner radius of the capsule pill in physical pixels.
    #[inline]
    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius.max(0.0);
        self
    }

    /// Constructs an Iris UI [`Style`] pre-configured with background, border, radius,
    /// and interactive hover states based on whether the row is currently selected.
    ///
    /// # Behavior
    /// - If `is_selected` is `true`, applies `selected_bg` and `selected_border`.
    /// - If `is_selected` is `false`, applies `normal_bg` and attaches `hover_bg` / `hover_border`.
    pub fn build_style(&self, is_selected: bool) -> Style {
        let (bg, border) = if is_selected {
            (self.selected_bg, self.selected_border)
        } else {
            (self.normal_bg, self.normal_border)
        };

        let mut style = Style::new()
            .background(bg)
            .border(self.border_width, border)
            .border_radius(self.border_radius);

        if !is_selected {
            style = style
                .hover_background(self.hover_bg)
                .hover_border(self.border_width, self.hover_border);
        }

        style
    }

    /// Resolves the background and border colors for the given state.
    #[inline]
    pub fn resolve_colors(&self, is_selected: bool, is_hovered: bool) -> (Color, Color) {
        if is_selected {
            (self.selected_bg, self.selected_border)
        } else if is_hovered {
            (self.hover_bg, self.hover_border)
        } else {
            (self.normal_bg, self.normal_border)
        }
    }

    /// Resolves the text and icon tint color for the given state.
    #[inline]
    pub fn resolve_text_color(&self, is_selected: bool, is_hovered: bool) -> Color {
        if is_selected {
            self.selected_text_color
        } else if is_hovered {
            self.hover_text_color
        } else {
            self.text_color
        }
    }

    /// Resolves the foldout chevron glyph color for the given state.
    #[inline]
    pub fn resolve_chevron_color(&self, is_selected: bool, is_hovered: bool) -> Color {
        if is_selected {
            self.chevron_selected_color
        } else if is_hovered {
            self.chevron_hover_color
        } else {
            self.chevron_color
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::geometry::Point;

    #[test]
    fn test_tree_toggle_expanded() {
        let mut state = TreeState::<u32>::new();

        // 1. Initial state: not expanded
        assert!(!state.is_expanded(&1));

        // 2. Click foldout: should expand and signal layout_changed
        let resp = handle_tree_foldout_click(&mut state, &1);
        assert!(resp.consumed);
        assert!(resp.layout_changed);
        assert!(!resp.selection_changed);
        assert!(state.is_expanded(&1));

        // 3. Click again: should collapse
        let resp2 = handle_tree_foldout_click(&mut state, &1);
        assert!(resp2.consumed);
        assert!(resp2.layout_changed);
        assert!(!state.is_expanded(&1));

        // 4. Multiple nodes expansion
        state.set_expanded(10, true);
        state.set_expanded(20, true);
        assert!(state.is_expanded(&10));
        assert!(state.is_expanded(&20));

        // 5. Glyph checks
        assert_eq!(tree_chevron_glyph(true), "▼");
        assert_eq!(tree_chevron_glyph(false), "▶");
        assert_eq!(tree_chevron_arrow_glyph(true), "▾");
        assert_eq!(tree_chevron_arrow_glyph(false), "▸");
    }

    #[test]
    fn test_tree_multi_selection_ctrl_shift() {
        let mut state = TreeState::<u32>::new();
        let visible_items = vec![100, 101, 102, 103, 104, 105];

        // 1. Single click on 101
        let no_mods = TreeSelectionModifiers::new();
        let resp = handle_tree_row_click(&mut state, 101, no_mods, Some(&visible_items));
        assert!(resp.consumed);
        assert!(!resp.layout_changed);
        assert!(resp.selection_changed);
        assert_eq!(state.selected_count(), 1);
        assert!(state.is_selected(&101));
        assert_eq!(state.primary_selected(), Some(&101));

        // 2. Ctrl-click on 103 -> should add 103 to selection
        let ctrl_mods = TreeSelectionModifiers::new().with_ctrl(true);
        let resp = handle_tree_row_click(&mut state, 103, ctrl_mods, Some(&visible_items));
        assert!(resp.consumed);
        assert_eq!(state.selected_count(), 2);
        assert!(state.is_selected(&101));
        assert!(state.is_selected(&103));

        // 3. Ctrl-click on 101 again -> should remove 101 from selection
        let resp = handle_tree_row_click(&mut state, 101, ctrl_mods, Some(&visible_items));
        assert!(resp.consumed);
        assert_eq!(state.selected_count(), 1);
        assert!(!state.is_selected(&101));
        assert!(state.is_selected(&103));

        // 4. Shift-click from 103 down to 105 -> should select 103..=105
        let shift_mods = TreeSelectionModifiers::new().with_shift(true);
        let resp = handle_tree_row_click(&mut state, 105, shift_mods, Some(&visible_items));
        assert!(resp.consumed);
        assert_eq!(state.selected_count(), 3);
        assert!(state.is_selected(&103));
        assert!(state.is_selected(&104));
        assert!(state.is_selected(&105));
        assert!(!state.is_selected(&102));

        // 5. Shift-click backward from 103 up to 100 -> should select 100..=103
        state.select_single(103);
        let resp = handle_tree_row_click(&mut state, 100, shift_mods, Some(&visible_items));
        assert!(resp.consumed);
        assert_eq!(state.selected_count(), 4);
        assert!(state.is_selected(&100));
        assert!(state.is_selected(&101));
        assert!(state.is_selected(&102));
        assert!(state.is_selected(&103));
        assert!(!state.is_selected(&104));

        // 6. Normal click resets multi-selection to single item
        let resp = handle_tree_row_click(&mut state, 102, no_mods, Some(&visible_items));
        assert!(resp.consumed);
        assert_eq!(state.selected_count(), 1);
        assert!(state.is_selected(&102));

        // 7. Ctrl+Shift additive selection
        let ctrl_shift_mods = TreeSelectionModifiers::from_ctrl_shift(true, true);
        let resp = handle_tree_row_click(&mut state, 104, ctrl_shift_mods, Some(&visible_items));
        assert!(resp.consumed);
        assert_eq!(state.selected_count(), 3);
        assert!(state.is_selected(&102));
        assert!(state.is_selected(&103));
        assert!(state.is_selected(&104));
    }

    #[test]
    fn test_tree_row_style_capsule_and_hover() {
        let style = TreeRowStyle::dark_default();
        assert_eq!(style.border_radius, 4.0);

        let sel_ui_style = style.build_style(true);
        assert_eq!(sel_ui_style.background_color, style.selected_bg);
        assert_eq!(
            sel_ui_style.corner_radii,
            iris_core::geometry::CornerRadii::all(4.0)
        );

        let unsel_ui_style = style.build_style(false);
        assert_eq!(unsel_ui_style.background_color, Color::TRANSPARENT);
        assert_eq!(unsel_ui_style.hover_background, Some(style.hover_bg));

        let (bg, border) = style.resolve_colors(true, false);
        assert_eq!(bg, style.selected_bg);
        assert_eq!(border, style.selected_border);

        let (h_bg, h_border) = style.resolve_colors(false, true);
        assert_eq!(h_bg, style.hover_bg);
        assert_eq!(h_border, style.hover_border);
    }

    #[test]
    fn test_tree_selection_event_mouse_down() {
        let mut state = TreeState::<String>::new();
        let target = TreeClickTarget::Row("node_a".to_string());
        let mods = TreeSelectionModifiers::new();

        // Left click down consumes and selects
        let left_down = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(10.0, 20.0),
        };
        let resp = handle_tree_selection_event(&mut state, &left_down, target.clone(), mods, None);
        assert!(resp.consumed);
        assert!(resp.selection_changed);
        assert!(state.is_selected(&"node_a".to_string()));

        // Right click down is unhandled by selection handler (reserved for context menus)
        let right_down = UiEvent::MouseDown {
            button: MouseButton::Right,
            point: Point::new(10.0, 20.0),
        };
        let resp_right = handle_tree_selection_event(&mut state, &right_down, target, mods, None);
        assert!(!resp_right.consumed);
    }
}