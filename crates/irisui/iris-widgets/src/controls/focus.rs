// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Focus & Keyboard Navigation Management
//!
//! Provides a centralized Single Source of Truth (SSOT) for UI keyboard focus,
//! outside-click clearing, Tab/Shift+Tab cycle navigation, and synchronization
//! with core layout focus targets.
//!

use iris_core::event::{FocusManager, KeyCode, MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;

/// Represents an interactive target capable of receiving keyboard input focus.
///
/// Supports either 64-bit semantic tags (persistent IDs used in declarative scopes)
/// or dynamic layout tree widget identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusTarget {
    /// 64-bit semantic tag assigned to a widget.
    Tag(u64),
    /// Discrete widget identifier within the UI tree.
    Widget(WidgetId),
}

impl From<u64> for FocusTarget {
    #[inline]
    fn from(tag: u64) -> Self {
        Self::Tag(tag)
    }
}

impl From<WidgetId> for FocusTarget {
    #[inline]
    fn from(id: WidgetId) -> Self {
        Self::Widget(id)
    }
}

/// Keyboard modifier keys active during a focus operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FocusModifiers {
    /// True if the Shift key is held down (reverses Tab cycling).
    pub shift: bool,
    /// True if the Control or Command key is held down.
    pub ctrl: bool,
    /// True if the Alt or Option key is held down.
    pub alt: bool,
}

impl FocusModifiers {
    /// Creates a new modifier record with all modifier flags inactive.
    #[inline]
    pub const fn new() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
        }
    }

    /// Sets the Shift modifier status.
    #[inline]
    pub const fn with_shift(mut self, shift: bool) -> Self {
        self.shift = shift;
        self
    }

    /// Sets the Control modifier status.
    #[inline]
    pub const fn with_ctrl(mut self, ctrl: bool) -> Self {
        self.ctrl = ctrl;
        self
    }

    /// Sets the Alt modifier status.
    #[inline]
    pub const fn with_alt(mut self, alt: bool) -> Self {
        self.alt = alt;
        self
    }
}

/// Bounding rectangle and navigation properties of a focusable interactive area.
///
/// # Examples
///
/// ```
/// use iris_widgets::controls::focus::{FocusableArea, FocusTarget};
/// use iris_core::geometry::Rect;
///
/// let area = FocusableArea::new(FocusTarget::Tag(42), Rect::new(0.0, 0.0, 100.0, 24.0));
/// assert!(area.bounds.contains_point(iris_core::geometry::Point::new(50.0, 12.0)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusableArea<T = FocusTarget> {
    /// Target identifier associated with this focusable area.
    pub id: T,
    /// Screen-space bounding rectangle for hit-testing pointer clicks.
    pub bounds: Rect,
    /// Explicit order for Tab key navigation. Lower values are navigated first.
    pub tab_index: Option<u32>,
}

impl<T> FocusableArea<T> {
    /// Creates a new focusable region with default tab ordering.
    pub fn new(id: T, bounds: Rect) -> Self {
        Self {
            id,
            bounds,
            tab_index: None,
        }
    }

    /// Creates a new focusable region with an explicit tab navigation index.
    pub fn with_tab_index(id: T, bounds: Rect, tab_index: u32) -> Self {
        Self {
            id,
            bounds,
            tab_index: Some(tab_index),
        }
    }
}

/// Operational policy rules governing focus acquisition, loss, and cycling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusPolicy {
    /// Whether clicking outside all registered focusable areas clears active focus.
    pub clear_on_outside_click: bool,
    /// Whether pressing the Escape key clears active focus.
    pub clear_on_escape: bool,
    /// Whether pressing Tab / Shift+Tab cycles focus among registered elements.
    pub cycle_tab: bool,
}

impl Default for FocusPolicy {
    fn default() -> Self {
        Self {
            clear_on_outside_click: true,
            clear_on_escape: true,
            cycle_tab: true,
        }
    }
}

/// Centralized state machine tracking active and previous keyboard input focus.
///
/// Provides single-source-of-truth tracking for text fields, search bars, and
/// modal inputs across editor panels.
///
/// # Examples
///
/// ```
/// use iris_widgets::controls::focus::FocusState;
///
/// let mut state: FocusState<u64> = FocusState::new();
/// assert!(!state.has_focus());
///
/// state.set_focus(100);
/// assert!(state.is_focused(&100));
///
/// state.clear_focus();
/// assert!(!state.has_focus());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FocusState<T = FocusTarget> {
    /// Identifier of the target currently owning keyboard input focus.
    pub focused: Option<T>,
    /// Previous target that owned keyboard input focus before the last change.
    pub previous_focused: Option<T>,
}

impl<T: Eq + Clone> FocusState<T> {
    /// Creates a new empty focus state with no active target.
    pub fn new() -> Self {
        Self {
            focused: None,
            previous_focused: None,
        }
    }

    /// Creates a focus state initialized with a specific target.
    pub fn with_focus(target: T) -> Self {
        Self {
            focused: Some(target),
            previous_focused: None,
        }
    }

    /// Returns a reference to the currently focused target, if any.
    #[must_use]
    #[inline]
    pub fn focused(&self) -> Option<&T> {
        self.focused.as_ref()
    }

    /// Returns whether any target currently owns keyboard input focus.
    #[must_use]
    #[inline]
    pub fn has_focus(&self) -> bool {
        self.focused.is_some()
    }

    /// Checks if a specific target currently holds keyboard input focus.
    #[must_use]
    #[inline]
    pub fn is_focused(&self, target: &T) -> bool {
        self.focused.as_ref() == Some(target)
    }

    /// Assigns keyboard input focus to the specified target.
    ///
    /// Returns `true` if the focus state changed, or `false` if the target was already focused.
    pub fn set_focus(&mut self, target: T) -> bool {
        if self.focused.as_ref() == Some(&target) {
            return false;
        }
        self.previous_focused = self.focused.take();
        self.focused = Some(target);
        true
    }

    /// Clears any active keyboard input focus.
    ///
    /// Returns `true` if an active focus was cleared, or `false` if no target was focused.
    pub fn clear_focus(&mut self) -> bool {
        if self.focused.is_none() {
            return false;
        }
        self.previous_focused = self.focused.take();
        true
    }

    /// Cycles focus through an ordered slice of targets (Tab / Shift+Tab traversal).
    ///
    /// If no target is currently focused, selects the first (or last if reverse) target.
    pub fn cycle_focus(&mut self, targets: &[T], reverse: bool) -> Option<T> {
        if targets.is_empty() {
            self.clear_focus();
            return None;
        }

        let current_index = self
            .focused
            .as_ref()
            .and_then(|current| targets.iter().position(|t| t == current));

        let next_index = match current_index {
            Some(idx) => {
                if reverse {
                    if idx == 0 { targets.len() - 1 } else { idx - 1 }
                } else {
                    (idx + 1) % targets.len()
                }
            }
            None => {
                if reverse {
                    targets.len() - 1
                } else {
                    0
                }
            }
        };

        let next_target = targets[next_index].clone();
        self.set_focus(next_target.clone());
        Some(next_target)
    }
}

impl FocusState<FocusTarget> {
    /// Synchronizes internal focus from the core UI [`FocusManager`].
    pub fn sync_from_core(&mut self, core_manager: &FocusManager) {
        if let Some(tag) = core_manager.focused_tag {
            self.set_focus(FocusTarget::Tag(tag));
        } else if let Some(id) = core_manager.focused {
            self.set_focus(FocusTarget::Widget(id));
        } else {
            self.clear_focus();
        }
    }

    /// Propagates internal focus back to the core UI [`FocusManager`].
    pub fn sync_to_core(&self, core_manager: &mut FocusManager) {
        match self.focused {
            Some(FocusTarget::Tag(tag)) => core_manager.set_focus_tag(tag),
            Some(FocusTarget::Widget(id)) => core_manager.set_focus(id),
            None => core_manager.clear_focus(),
        }
    }
}

/// High-level response emitted after evaluating an interaction against the focus subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusEventResponse<T = FocusTarget> {
    /// Whether the interaction event was consumed by the focus management subsystem.
    pub consumed: bool,
    /// Whether the active focus target changed as a result of this event.
    pub focus_changed: bool,
    /// The currently focused target after handling the event, if any.
    pub focused: Option<T>,
    /// Whether focus was cleared due to a pointer click outside all registered focusable regions.
    pub cleared_on_outside_click: bool,
    /// Whether focus was cleared due to pressing the Escape key.
    pub cleared_on_escape: bool,
    /// Whether focus was cycled forward or backward using Tab / Shift+Tab.
    pub tab_navigated: bool,
}

impl<T> Default for FocusEventResponse<T> {
    fn default() -> Self {
        Self {
            consumed: false,
            focus_changed: false,
            focused: None,
            cleared_on_outside_click: false,
            cleared_on_escape: false,
            tab_navigated: false,
        }
    }
}

/// Evaluates a [`UiEvent`] against focusable areas and updates the central [`FocusState`].
///
/// Handles outside-click clearing, focus target selection, Escape cancellation,
/// and cyclic Tab navigation with default modifiers.
///
/// # Examples
///
/// ```
/// use iris_widgets::controls::focus::{
///     handle_focus_event, FocusPolicy, FocusState, FocusableArea, FocusTarget
/// };
/// use iris_core::event::{MouseButton, UiEvent};
/// use iris_core::geometry::{Point, Rect};
///
/// let mut state = FocusState::new();
/// let areas = [
///     FocusableArea::new(FocusTarget::Tag(1), Rect::new(10.0, 10.0, 100.0, 30.0)),
///     FocusableArea::new(FocusTarget::Tag(2), Rect::new(10.0, 50.0, 100.0, 30.0)),
/// ];
/// let policy = FocusPolicy::default();
///
/// // Click inside first area
/// let click_event = UiEvent::MouseDown {
///     point: Point::new(20.0, 20.0),
///     button: MouseButton::Left,
/// };
/// let res = handle_focus_event(&mut state, &click_event, &areas, &policy);
/// assert!(res.focus_changed);
/// assert_eq!(state.focused(), Some(&FocusTarget::Tag(1)));
/// ```
pub fn handle_focus_event<T: Eq + Clone>(
    state: &mut FocusState<T>,
    event: &UiEvent,
    focusables: &[FocusableArea<T>],
    policy: &FocusPolicy,
) -> FocusEventResponse<T> {
    handle_focus_event_with_modifiers(state, event, focusables, policy, FocusModifiers::default())
}

/// Evaluates a [`UiEvent`] with explicit keyboard modifiers against focusable areas.
pub fn handle_focus_event_with_modifiers<T: Eq + Clone>(
    state: &mut FocusState<T>,
    event: &UiEvent,
    focusables: &[FocusableArea<T>],
    policy: &FocusPolicy,
    modifiers: FocusModifiers,
) -> FocusEventResponse<T> {
    match event {
        UiEvent::MouseDown {
            button: MouseButton::Left,
            point,
        } => handle_pointer_click(state, *point, focusables, policy),
        UiEvent::KeyboardInput {
            key,
            is_pressed: true,
            ..
        } => match key {
            KeyCode::Tab if policy.cycle_tab && !focusables.is_empty() => {
                handle_tab_key(state, focusables, modifiers.shift)
            }
            KeyCode::Escape if policy.clear_on_escape && state.has_focus() => {
                let focus_changed = state.clear_focus();
                FocusEventResponse {
                    consumed: true,
                    focus_changed,
                    focused: None,
                    cleared_on_outside_click: false,
                    cleared_on_escape: true,
                    tab_navigated: false,
                }
            }
            _ => FocusEventResponse {
                consumed: false,
                focus_changed: false,
                focused: state.focused.clone(),
                cleared_on_outside_click: false,
                cleared_on_escape: false,
                tab_navigated: false,
            },
        },
        _ => FocusEventResponse {
            consumed: false,
            focus_changed: false,
            focused: state.focused.clone(),
            cleared_on_outside_click: false,
            cleared_on_escape: false,
            tab_navigated: false,
        },
    }
}

/// Evaluates a left mouse press against focusable areas for target acquisition or outside-click loss.
fn handle_pointer_click<T: Eq + Clone>(
    state: &mut FocusState<T>,
    pos: Point,
    focusables: &[FocusableArea<T>],
    policy: &FocusPolicy,
) -> FocusEventResponse<T> {
    // Check if clicked inside any focusable area
    for area in focusables {
        if area.bounds.contains_point(pos) {
            let focus_changed = state.set_focus(area.id.clone());
            return FocusEventResponse {
                consumed: true,
                focus_changed,
                focused: state.focused.clone(),
                cleared_on_outside_click: false,
                cleared_on_escape: false,
                tab_navigated: false,
            };
        }
    }

    // Click was outside all registered focusables
    if policy.clear_on_outside_click && state.has_focus() {
        let focus_changed = state.clear_focus();
        FocusEventResponse {
            consumed: false, // Allow outside clicks to pass through to host layout
            focus_changed,
            focused: None,
            cleared_on_outside_click: true,
            cleared_on_escape: false,
            tab_navigated: false,
        }
    } else {
        FocusEventResponse {
            consumed: false,
            focus_changed: false,
            focused: state.focused.clone(),
            cleared_on_outside_click: false,
            cleared_on_escape: false,
            tab_navigated: false,
        }
    }
}

/// Advances or recedes active focus across focusable elements via Tab navigation.
fn handle_tab_key<T: Eq + Clone>(
    state: &mut FocusState<T>,
    focusables: &[FocusableArea<T>],
    reverse: bool,
) -> FocusEventResponse<T> {
    // Sort targets by explicit tab_index if provided, preserving original order as secondary key
    let mut indexed_areas: Vec<(usize, &FocusableArea<T>)> =
        focusables.iter().enumerate().collect();
    indexed_areas.sort_by(|(idx_a, a), (idx_b, b)| match (a.tab_index, b.tab_index) {
        (Some(ta), Some(tb)) => ta.cmp(&tb).then(idx_a.cmp(idx_b)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => idx_a.cmp(idx_b),
    });

    let ordered_targets: Vec<T> = indexed_areas
        .into_iter()
        .map(|(_, area)| area.id.clone())
        .collect();

    let new_target = state.cycle_focus(&ordered_targets, reverse);

    FocusEventResponse {
        consumed: true,
        focus_changed: true,
        focused: new_target,
        cleared_on_outside_click: false,
        cleared_on_escape: false,
        tab_navigated: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_manager_switch_and_clear_on_outside_click() {
        let mut state: FocusState<u64> = FocusState::new();
        let areas = [
            FocusableArea::new(101, Rect::new(0.0, 0.0, 100.0, 30.0)),
            FocusableArea::new(102, Rect::new(0.0, 40.0, 100.0, 30.0)),
        ];
        let policy = FocusPolicy::default();

        // 1. Initial click on item 101 -> acquires focus
        let click_1 = UiEvent::MouseDown {
            point: Point::new(50.0, 15.0),
            button: MouseButton::Left,
        };
        let res_1 = handle_focus_event(&mut state, &click_1, &areas, &policy);
        assert!(res_1.consumed);
        assert!(res_1.focus_changed);
        assert_eq!(res_1.focused, Some(101));
        assert_eq!(state.focused(), Some(&101));

        // 2. Click on item 102 -> switches focus directly
        let click_2 = UiEvent::MouseDown {
            point: Point::new(50.0, 55.0),
            button: MouseButton::Left,
        };
        let res_2 = handle_focus_event(&mut state, &click_2, &areas, &policy);
        assert!(res_2.consumed);
        assert!(res_2.focus_changed);
        assert_eq!(res_2.focused, Some(102));
        assert_eq!(state.focused(), Some(&102));

        // 3. Click outside all items -> clears focus
        let click_outside = UiEvent::MouseDown {
            point: Point::new(200.0, 200.0),
            button: MouseButton::Left,
        };
        let res_outside = handle_focus_event(&mut state, &click_outside, &areas, &policy);
        assert!(!res_outside.consumed); // Non-blocking for host containers
        assert!(res_outside.focus_changed);
        assert!(res_outside.cleared_on_outside_click);
        assert_eq!(res_outside.focused, None);
        assert!(!state.has_focus());
    }

    #[test]
    fn test_focus_tab_cycling_forward_and_reverse() {
        let mut state: FocusState<u64> = FocusState::new();
        let areas = [
            FocusableArea::new(1, Rect::new(0.0, 0.0, 50.0, 20.0)),
            FocusableArea::new(2, Rect::new(0.0, 25.0, 50.0, 20.0)),
            FocusableArea::new(3, Rect::new(0.0, 50.0, 50.0, 20.0)),
        ];
        let policy = FocusPolicy::default();

        // Tab without active focus -> selects first item (1)
        let tab_forward = UiEvent::KeyboardInput {
            key: KeyCode::Tab,
            text: None,
            is_pressed: true,
        };
        let res = handle_focus_event(&mut state, &tab_forward, &areas, &policy);
        assert!(res.consumed);
        assert!(res.tab_navigated);
        assert_eq!(state.focused(), Some(&1));

        // Tab forward -> 2
        let res = handle_focus_event(&mut state, &tab_forward, &areas, &policy);
        assert!(res.consumed);
        assert!(res.tab_navigated);
        assert_eq!(state.focused(), Some(&2));

        // Tab forward -> 3
        let res = handle_focus_event(&mut state, &tab_forward, &areas, &policy);
        assert!(res.consumed);
        assert!(res.tab_navigated);
        assert_eq!(state.focused(), Some(&3));

        // Tab forward -> wraps to 1
        let res = handle_focus_event(&mut state, &tab_forward, &areas, &policy);
        assert!(res.consumed);
        assert!(res.tab_navigated);
        assert_eq!(state.focused(), Some(&1));

        // Shift+Tab backward -> wraps to 3
        let res = handle_focus_event_with_modifiers(
            &mut state,
            &tab_forward,
            &areas,
            &policy,
            FocusModifiers::new().with_shift(true),
        );
        assert!(res.consumed);
        assert!(res.tab_navigated);
        assert_eq!(state.focused(), Some(&3));
    }

    #[test]
    fn test_focus_tab_order_by_explicit_index() {
        let mut state: FocusState<&'static str> = FocusState::new();
        let areas = [
            FocusableArea::with_tab_index("second", Rect::new(0.0, 0.0, 50.0, 20.0), 20),
            FocusableArea::with_tab_index("first", Rect::new(0.0, 25.0, 50.0, 20.0), 10),
            FocusableArea::with_tab_index("third", Rect::new(0.0, 50.0, 50.0, 20.0), 30),
        ];
        let policy = FocusPolicy::default();

        let tab = UiEvent::KeyboardInput {
            key: KeyCode::Tab,
            text: None,
            is_pressed: true,
        };

        handle_focus_event(&mut state, &tab, &areas, &policy);
        assert_eq!(state.focused(), Some(&"first"));

        handle_focus_event(&mut state, &tab, &areas, &policy);
        assert_eq!(state.focused(), Some(&"second"));

        handle_focus_event(&mut state, &tab, &areas, &policy);
        assert_eq!(state.focused(), Some(&"third"));
    }

    #[test]
    fn test_focus_escape_clears_focus() {
        let mut state = FocusState::with_focus(FocusTarget::Tag(42));
        let areas = [FocusableArea::new(
            FocusTarget::Tag(42),
            Rect::new(0.0, 0.0, 100.0, 30.0),
        )];
        let policy = FocusPolicy::default();

        let escape_event = UiEvent::KeyboardInput {
            key: KeyCode::Escape,
            text: None,
            is_pressed: true,
        };

        let res = handle_focus_event(&mut state, &escape_event, &areas, &policy);
        assert!(res.consumed);
        assert!(res.cleared_on_escape);
        assert!(res.focus_changed);
        assert_eq!(state.focused(), None);
    }

    #[test]
    fn test_focus_state_sync_with_core_focus_manager() {
        let mut core_manager = FocusManager::new();
        core_manager.set_focus_tag(999);

        let mut widget_focus = FocusState::new();
        widget_focus.sync_from_core(&core_manager);
        assert_eq!(widget_focus.focused(), Some(&FocusTarget::Tag(999)));

        widget_focus.set_focus(FocusTarget::Tag(888));
        widget_focus.sync_to_core(&mut core_manager);
        assert_eq!(core_manager.focused_tag(), Some(888));
    }
}