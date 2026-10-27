// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Real-time Hover Reconciliation & Subtree Hover Styling
//!
//! Provides zero-latency recursive hover state propagation across interactive nodes
//! (such as modal buttons, links, compass widgets, and dropdown items) directly
//! following layout finalization, avoiding single-frame visual delay.

use crate::modal::{MODAL_TAG_CANCEL, MODAL_TAG_CLOSE, MODAL_TAG_CONFIRM, MODAL_TAG_DANGER};
use iris_core::{Color, Point, UiTree, WidgetId, WidgetRole};

/// Recursively updates visual hover styling for interactive widgets in a computed subtree.
///
/// Evaluates `computed_rect.contains_point(cursor_pos)` against known semantic tags and roles,
/// applying clean ~10% background brightness lifts without neon glows or shadow halos.
///
/// # Arguments
/// * `tree` - Mutable reference to the UI node hierarchy.
/// * `current` - Identifier of the subtree root node to evaluate.
/// * `cursor_pos` - Hardware cursor coordinates in physical screen space.
pub fn update_hover_styles_recursive(tree: &mut UiTree, current: WidgetId, cursor_pos: Point) {
    let (is_hovered, tag, role, interactive, children) = {
        let Some(node) = tree.get(current) else {
            return;
        };
        (
            node.computed_rect.contains_point(cursor_pos),
            node.tag,
            node.role,
            node.interactive,
            node.children.clone(),
        )
    };

    if let Some(node) = tree.get_mut(current) {
        if tag == MODAL_TAG_CLOSE {
            node.text_color = if is_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.60, 0.64, 0.72, 0.85)
            };
            node.style = node
                .style
                .background(if is_hovered {
                    Color::rgba(0.85, 0.22, 0.22, 0.28)
                } else {
                    Color::TRANSPARENT
                })
                .border(0.0, Color::TRANSPARENT);
            node.style.box_shadow = None;
        } else if tag == MODAL_TAG_CONFIRM {
            node.text_color = if is_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.92, 0.96, 1.0, 1.0)
            };
            node.style = node
                .style
                .background(if is_hovered {
                    Color::rgba(0.18, 0.48, 0.68, 1.0)
                } else {
                    Color::rgba(0.14, 0.40, 0.58, 1.0)
                })
                .border(
                    1.0,
                    if is_hovered {
                        Color::rgba(1.0, 1.0, 1.0, 0.25)
                    } else {
                        Color::rgba(1.0, 1.0, 1.0, 0.15)
                    },
                );
            node.style.box_shadow = None;
        } else if tag == MODAL_TAG_DANGER {
            node.text_color = Color::WHITE;
            node.style = node
                .style
                .background(if is_hovered {
                    Color::rgba(0.75, 0.20, 0.20, 1.0)
                } else {
                    Color::rgba(0.63, 0.14, 0.14, 1.0)
                })
                .border(
                    1.0,
                    if is_hovered {
                        Color::rgba(1.0, 1.0, 1.0, 0.25)
                    } else {
                        Color::rgba(1.0, 1.0, 1.0, 0.15)
                    },
                );
            node.style.box_shadow = None;
        } else if tag == MODAL_TAG_CANCEL {
            node.text_color = if is_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.75, 0.78, 0.84, 1.0)
            };
            node.style = node
                .style
                .background(if is_hovered {
                    Color::rgba(0.20, 0.22, 0.26, 1.0)
                } else {
                    Color::rgba(0.14, 0.15, 0.18, 1.0)
                })
                .border(
                    1.0,
                    if is_hovered {
                        Color::rgba(1.0, 1.0, 1.0, 0.18)
                    } else {
                        Color::rgba(1.0, 1.0, 1.0, 0.10)
                    },
                );
            node.style.box_shadow = None;
        } else if role == WidgetRole::MenuBarItem {
            node.text_color = if is_hovered {
                Color::WHITE
            } else {
                Color::hex("#dcdce2")
            };
            node.style = node.style.background(if is_hovered {
                Color::hex("#222634")
            } else {
                Color::TRANSPARENT
            });
        } else if role == WidgetRole::DropdownItem && interactive {
            node.style = node.style.background(if is_hovered {
                Color::hex("#222634")
            } else {
                Color::TRANSPARENT
            });
        } else if role == WidgetRole::Button && (2010..=2025).contains(&tag) {
            let is_active = node.style.background_color == Color::rgba(0.06, 0.46, 0.92, 1.0);
            if !is_active {
                let (bg, border) = if is_hovered {
                    (
                        Color::rgba(0.20, 0.23, 0.30, 0.95),
                        Color::rgba(0.35, 0.40, 0.50, 0.90),
                    )
                } else {
                    (
                        Color::rgba(0.12, 0.13, 0.16, 0.92),
                        Color::rgba(0.24, 0.26, 0.32, 0.85),
                    )
                };
                node.style = node.style.background(bg).border(1.0, border);
                for cid in &children {
                    if let Some(child) = tree.get_mut(*cid) {
                        child.set_texture_tint(if is_hovered {
                            Color::rgba(0.95, 0.98, 1.0, 1.0)
                        } else {
                            Color::rgba(0.80, 0.84, 0.90, 0.90)
                        });
                    }
                }
            }
        } else if role == WidgetRole::Button && (tag == 2001 || tag == 2002) {
            let is_open = node.style.background_color == Color::rgba(0.20, 0.23, 0.30, 0.90);
            if !is_open {
                node.style = node.style.background(if is_hovered {
                    Color::rgba(0.20, 0.23, 0.30, 0.90)
                } else {
                    Color::TRANSPARENT
                });
                let tc = if is_hovered {
                    Color::rgba(1.0, 1.0, 1.0, 1.0)
                } else {
                    Color::rgba(0.85, 0.88, 0.94, 1.0)
                };
                for cid in &children {
                    if let Some(child) = tree.get_mut(*cid) {
                        if child.text.is_some() {
                            child.text_color = tc;
                        } else {
                            child.set_texture_tint(tc);
                        }
                    }
                }
            }
        } else if role == WidgetRole::Button && (2030..=2035).contains(&tag) {
            let (bg, text_col) = super::compass::resolve_compass_hover_colors(tag, is_hovered);
            node.style = node.style.background(bg);
            if let Some(tc) = text_col {
                for cid in &children {
                    if let Some(child) = tree.get_mut(*cid) {
                        child.text_color = tc;
                    }
                }
            }
        }
    }

    if role == WidgetRole::DropdownItem && interactive {
        for cid in &children {
            if let Some(child) = tree.get_mut(*cid) {
                if child.role == WidgetRole::DropdownLabel || child.role == WidgetRole::DropdownIcon
                {
                    child.text_color = if is_hovered {
                        Color::WHITE
                    } else {
                        Color::hex("#dcdce2")
                    };
                } else if child.role == WidgetRole::DropdownShortcut
                    && child.text.as_deref() != Some("✓")
                {
                    child.text_color = if is_hovered {
                        Color::hex("#b0b0be")
                    } else {
                        Color::hex("#828292")
                    };
                }
            }
        }
    }

    for child_id in children {
        update_hover_styles_recursive(tree, child_id, cursor_pos);
    }
}