// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interaction events, hit-testing, focus management, and widget event dispatching.

use crate::geometry::Point;
use crate::id::WidgetId;
use crate::tree::UiTree;

/// Mouse button identifiers for pointer interaction events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MouseButton {
    /// Primary left mouse button.
    #[default]
    Left,
    /// Secondary right mouse button (context menus, navigation).
    Right,
    /// Middle mouse button / scroll wheel press.
    Middle,
    /// Auxiliary mouse button indexed by hardware code.
    Other(u16),
}

/// Virtual key codes for keyboard navigation and input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// Backspace key for deleting previous character.
    Backspace,
    /// Delete key for deleting next character.
    Delete,
    /// Enter / Return key for submitting or confirming input.
    Enter,
    /// Escape key for cancelling or unfocusing.
    Escape,
    /// Left arrow key.
    ArrowLeft,
    /// Right arrow key.
    ArrowRight,
    /// Up arrow key.
    ArrowUp,
    /// Down arrow key.
    ArrowDown,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Tab key for focus traversal.
    Tab,
    /// Spacebar key.
    Space,
    /// Printable text character.
    Character(char),
    /// Other unmapped hardware key.
    Other,
}

/// Interactive state of a widget node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WidgetState {
    /// Default idle state.
    #[default]
    Normal,
    /// Pointer is currently hovering over the widget.
    Hovered,
    /// Pointer button is currently pressed down on the widget.
    Pressed,
    /// Widget currently owns keyboard or input focus.
    Focused,
    /// Widget is disabled and ignores interaction events.
    Disabled,
}

/// Incoming raw interaction events from the windowing system.
#[derive(Debug, Clone, PartialEq)]
pub enum UiEvent {
    /// Cursor moved to a new screen-space coordinate.
    MouseMove {
        /// Absolute coordinate in screen pixels.
        point: Point,
    },
    /// Mouse button pressed down at a specific coordinate.
    MouseDown {
        /// Pressed mouse button.
        button: MouseButton,
        /// Screen coordinate where the press occurred.
        point: Point,
    },
    /// Mouse button released at a specific coordinate.
    MouseUp {
        /// Released mouse button.
        button: MouseButton,
        /// Screen coordinate where the release occurred.
        point: Point,
    },
    /// Mouse wheel or touchpad scroll delta.
    MouseScroll {
        /// Horizontal scroll amount.
        delta_x: f32,
        /// Vertical scroll amount.
        delta_y: f32,
        /// Screen coordinate of the cursor during scroll.
        point: Point,
    },
    /// Keyboard key press or release.
    KeyboardInput {
        /// Key code of the pressed/released key.
        key: KeyCode,
        /// Optional UTF-8 text representation produced by the key event.
        text: Option<String>,
        /// Whether the key was pressed (`true`) or released (`false`).
        is_pressed: bool,
    },
    /// Operating system IME input composition or commit.
    Ime(ImeEvent),
}

/// Operating system Input Method Editor (IME) composition and commit events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImeEvent {
    /// IME composition enabled by system.
    Enabled,
    /// In-progress uncommitted composition string and optional cursor range `(start, end)`.
    Preedit(String, Option<(usize, usize)>),
    /// Finalized committed text string.
    Commit(String),
    /// IME composition disabled by system.
    Disabled,
}

/// High-level interaction event emitted to widget callbacks.
#[derive(Debug, Clone, PartialEq)]
pub enum InteractionEvent {
    /// Cursor entered the widget bounds.
    HoverEnter,
    /// Cursor exited the widget bounds.
    HoverLeave,
    /// Mouse button was clicked (down + up) within the widget bounds.
    Click {
        /// Clicked button.
        button: MouseButton,
    },
    /// Dragging active with pointer displacement delta.
    Drag {
        /// Distance moved since the previous frame in pixels.
        delta: Point,
    },
    /// Text was inputted into the focused widget.
    TextInput {
        /// The new accumulated string value.
        text: String,
    },
    /// In-progress IME composition preedit string.
    ImePreedit {
        /// In-progress text.
        text: String,
        /// Selection or cursor range within preedit string.
        cursor: Option<(usize, usize)>,
    },
    /// Finalized committed IME text string.
    ImeCommit {
        /// Committed string value.
        text: String,
    },
    /// Key was pressed while this widget was focused.
    KeyDown {
        /// Pressed key.
        key: KeyCode,
    },
    /// Widget gained user input focus.
    FocusGained,
    /// Widget lost user input focus.
    FocusLost,
}

/// Global focus and active interaction state manager for the UI hierarchy.
#[derive(Debug, Clone, Default)]
pub struct FocusManager {
    /// Widget currently owning keyboard input focus.
    pub focused: Option<WidgetId>,
    /// Widget currently hovered by the pointer cursor.
    pub hovered: Option<WidgetId>,
    /// Widget currently pressed down by the pointer.
    pub pressed: Option<WidgetId>,
    /// Screen coordinate where mouse press initiated.
    pub press_origin: Option<Point>,
}

impl FocusManager {
    /// Creates a new, unpopulated focus manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets keyboard input focus to the specified widget.
    pub fn set_focus(&mut self, id: WidgetId) {
        self.focused = Some(id);
    }

    /// Clears any active keyboard input focus.
    pub fn clear_focus(&mut self) {
        self.focused = None;
    }

    /// Returns the recommended screen rectangle for the OS IME composition popup.
    pub fn get_ime_cursor_area(&self, tree: &UiTree) -> Option<crate::geometry::Rect> {
        let focused_id = self.focused?;
        let node = tree.get(focused_id)?;
        Some(node.computed_rect)
    }

    /// Advances focus to the next interactive widget in the tree (Tab traversal).
    pub fn advance_focus(&mut self, tree: &UiTree, reverse: bool) {
        let mut interactive_nodes = Vec::new();
        if let Some(root) = tree.root() {
            tree.traverse_depth_first(root, &mut |id, node| {
                if node.interactive && node.visible {
                    interactive_nodes.push(id);
                }
            });
        }

        if interactive_nodes.is_empty() {
            self.focused = None;
            return;
        }

        let current_idx = self
            .focused
            .and_then(|f| interactive_nodes.iter().position(|&id| id == f));

        let next_idx = match current_idx {
            Some(idx) => {
                if reverse {
                    if idx == 0 {
                        interactive_nodes.len() - 1
                    } else {
                        idx - 1
                    }
                } else {
                    (idx + 1) % interactive_nodes.len()
                }
            }
            None => {
                if reverse {
                    interactive_nodes.len() - 1
                } else {
                    0
                }
            }
        };

        self.focused = Some(interactive_nodes[next_idx]);
    }
}

/// Dispatches raw `UiEvent` inputs into target widgets and returns high-level responses.
pub struct EventDispatcher;

impl EventDispatcher {
    /// Dispatches a raw `UiEvent` into the UI tree, updating the `FocusManager` and returning emitted interaction events.
    pub fn dispatch(
        tree: &mut UiTree,
        focus: &mut FocusManager,
        event: UiEvent,
    ) -> Vec<(WidgetId, InteractionEvent)> {
        let mut emitted = Vec::new();

        match event {
            UiEvent::MouseMove { point } => {
                let hit = tree.hit_test(point);

                // Check for hover changes
                if hit != focus.hovered {
                    if let Some(old_hover) = focus.hovered {
                        emitted.push((old_hover, InteractionEvent::HoverLeave));
                    }
                    if let Some(new_hover) = hit {
                        emitted.push((new_hover, InteractionEvent::HoverEnter));
                    }
                    focus.hovered = hit;
                }

                // Check for active dragging
                if let Some(pressed_id) = focus.pressed
                    && let Some(origin) = focus.press_origin
                {
                    let delta = Point::new(point.x - origin.x, point.y - origin.y);
                    emitted.push((pressed_id, InteractionEvent::Drag { delta }));
                }
            }
            UiEvent::MouseDown { button: _, point } => {
                let hit = tree.hit_test(point);
                focus.pressed = hit;
                focus.press_origin = Some(point);

                // Focus transition
                if hit != focus.focused {
                    if let Some(old_focus) = focus.focused {
                        emitted.push((old_focus, InteractionEvent::FocusLost));
                    }
                    if let Some(new_focus) = hit {
                        emitted.push((new_focus, InteractionEvent::FocusGained));
                    }
                    focus.focused = hit;
                }
            }
            UiEvent::MouseUp { button, point } => {
                let hit = tree.hit_test(point);

                if let Some(pressed_id) = focus.pressed
                    && hit == Some(pressed_id)
                {
                    emitted.push((pressed_id, InteractionEvent::Click { button }));
                }

                focus.pressed = None;
                focus.press_origin = None;
            }
            UiEvent::KeyboardInput {
                key,
                text,
                is_pressed,
            } => {
                if is_pressed {
                    if key == KeyCode::Tab {
                        focus.advance_focus(tree, false);
                    }

                    if let Some(focused_id) = focus.focused {
                        emitted.push((focused_id, InteractionEvent::KeyDown { key }));
                        if let Some(txt) = text {
                            emitted.push((focused_id, InteractionEvent::TextInput { text: txt }));
                        }
                    }
                }
            }
            UiEvent::Ime(ime_event) => {
                if let Some(focused_id) = focus.focused {
                    match ime_event {
                        ImeEvent::Preedit(text, cursor) => {
                            emitted
                                .push((focused_id, InteractionEvent::ImePreedit { text, cursor }));
                        }
                        ImeEvent::Commit(text) => {
                            emitted.push((focused_id, InteractionEvent::ImeCommit { text }));
                        }
                        _ => {}
                    }
                }
            }
            UiEvent::MouseScroll { .. } => {}
        }

        emitted
    }
}

/// Result of a hit-test operation against the active UI hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitTestResult {
    /// Deepest interactive `WidgetId` under the cursor.
    pub target: WidgetId,
}