// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Keymap Preferences Tab
//!
//! Renders categorized hotkey cheat-sheets and keyboard shortcuts for viewport navigation,
//! transform gizmos, and scene operations declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::pref_heading;
use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;

/// Categorized hotkey group descriptor.
struct KeymapGroup {
    title: &'static str,
    shortcuts: &'static [(&'static str, &'static str, &'static str)],
}

const KEYMAP_GROUPS: [KeymapGroup; 4] = [
    KeymapGroup {
        title: "🎮  Viewport Navigation",
        shortcuts: &[
            (
                "Fly Navigation",
                "RMB + WASD",
                "Fly through 3D viewport (Q/E for vertical descent/ascent).",
            ),
            (
                "Camera Boost",
                "Shift",
                "Accelerate fly camera translation speed.",
            ),
            (
                "Orbit View",
                "Alt + LMB",
                "Orbit around cursor target or selected entity.",
            ),
            (
                "Pan View",
                "MMB",
                "Translate camera horizontally/vertically in viewport plane.",
            ),
            (
                "Focus Selected",
                "F",
                "Frame and center camera view on selected entity.",
            ),
        ],
    },
    KeymapGroup {
        title: "📐  Transform Gizmos",
        shortcuts: &[
            ("Translate Mode", "W", "Activate 3D axis translation gizmo."),
            ("Rotate Mode", "E", "Activate 3D rotation ring gizmo."),
            (
                "Scale Mode",
                "R",
                "Activate 3D uniform and per-axis scale gizmo.",
            ),
            (
                "Coordinate Space",
                "Q",
                "Toggle between World and Local manipulation frames.",
            ),
        ],
    },
    KeymapGroup {
        title: "✨  Scene Editing & History",
        shortcuts: &[
            ("Undo", "Ctrl + Z", "Revert last scene or transform action."),
            (
                "Redo",
                "Ctrl + Y / Ctrl + Shift + Z",
                "Re-apply previously reverted action.",
            ),
            (
                "Save Scene",
                "Ctrl + S",
                "Save active scene hierarchy to disk.",
            ),
            (
                "Delete Entity",
                "Delete",
                "Remove selected entity from active ECS world.",
            ),
            (
                "Rename Entity",
                "F2",
                "Focus inline name text edit in Hierarchy panel.",
            ),
            (
                "Duplicate Entity",
                "Ctrl + D",
                "Clone selected entity with its components and transforms.",
            ),
        ],
    },
    KeymapGroup {
        title: "🪟  Workspace & View Toggles",
        shortcuts: &[
            (
                "Toggle Preferences",
                "Ctrl + ,",
                "Open or dismiss the Preferences dialog.",
            ),
            (
                "2D / 3D Mode",
                "2 / 3",
                "Switch viewport between 2D ortho and 3D perspective.",
            ),
            (
                "Toggle Wireframe",
                "Shift + W",
                "Overlay geometric wireframe lines on 3D meshes.",
            ),
            (
                "Toggle Grid",
                "Shift + G",
                "Show or hide viewport coordinate ground plane grid.",
            ),
        ],
    },
];

/// Builds the Keymap preferences tab content declaratively using [`UiScope`].
pub fn build_keymap_tab(scope: &mut UiScope<'_>, _params: &PreferencesParams<'_>) {
    pref_heading(
        scope,
        "⌨  Keymap & Shortcuts",
        "Essential keyboard shortcuts, transform hotkeys, and viewport navigation gestures.",
    );

    for group in &KEYMAP_GROUPS {
        scope.container_named(
            "KeymapGroupCard",
            Style::new()
                .flex_col()
                .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
                .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
                .border_radius(6.0)
                .padding_insets(Insets::new(12.0, 14.0, 10.0, 14.0))
                .margin_insets(Insets::new(0.0, 0.0, 12.0, 0.0)),
            |card| {
                card.label_styled_passive(
                    "KeymapGroupTitle",
                    group.title,
                    12.5,
                    Color::rgba(1.0, 1.0, 1.0, 1.0),
                    TextAlign::Left,
                    Style::new().margin_insets(Insets::new(0.0, 0.0, 8.0, 0.0)),
                );

                for &(name, hotkey, note) in group.shortcuts {
                    card.container_named(
                        "KeymapItemRow",
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::SpaceBetween)
                            .padding_insets(Insets::new(4.0, 0.0, 4.0, 0.0)),
                        |row| {
                            row.container_named(
                                "KeymapLeftCol",
                                Style::new().flex_col().flex_grow(1.0),
                                |left| {
                                    left.label_styled_passive(
                                        "KeymapItemName",
                                        name,
                                        11.5,
                                        Color::rgba(0.90, 0.92, 0.96, 1.0),
                                        TextAlign::Left,
                                        Style::new(),
                                    );
                                    left.label_styled_passive(
                                        "KeymapItemNote",
                                        note,
                                        10.5,
                                        Color::rgba(0.60, 0.64, 0.72, 1.0),
                                        TextAlign::Left,
                                        Style::new(),
                                    );
                                },
                            );

                            row.container_named(
                                "KeymapBadge",
                                Style::new()
                                    .background(Color::rgba(0.0, 0.65, 0.85, 0.15))
                                    .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.40))
                                    .border_radius(4.0)
                                    .padding_insets(Insets::new(3.0, 8.0, 3.0, 8.0)),
                                |badge| {
                                    badge.label_styled_passive(
                                        "KeymapBadgeText",
                                        hotkey,
                                        11.0,
                                        Color::rgba(0.0, 0.90, 1.0, 1.0),
                                        TextAlign::Center,
                                        Style::new(),
                                    );
                                },
                            );
                        },
                    );
                }
            },
        );
    }
}