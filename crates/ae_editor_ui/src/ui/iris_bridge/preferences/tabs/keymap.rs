// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Keymap Preferences Tab
//!
//! Renders categorized hotkey cheat-sheets and keyboard shortcuts for viewport navigation,
//! transform gizmos, and scene operations declaratively using [`UiScope`].

use irisui::prelude::*;

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
pub fn build_keymap_tab(scope: &mut UiScope<'_>) {
    scope.section_heading(
        "⌨  Keymap & Shortcuts",
        "Essential keyboard shortcuts, transform hotkeys, and viewport navigation gestures.",
    );

    for group in &KEYMAP_GROUPS {
        render_keymap_group(scope, group);
    }
}

fn render_keymap_group(scope: &mut UiScope<'_>, group: &KeymapGroup) {
    scope.titled_surface_card_named("KeymapGroupCard", "KeymapGroupTitle", group.title, |card| {
        for &(name, hotkey, note) in group.shortcuts {
            card.titled_badge_row(name, note, hotkey, Color::rgba(0.0, 0.90, 1.0, 1.0));
        }
    });
}