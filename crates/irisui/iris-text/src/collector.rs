// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Retained UI tree text collection pipeline with automatic layer-aware occlusion culling.
//!
//! Extracts renderable `TextSection` instances from an `iris_core::UiTree`, computing
//! hierarchical scissor clipping and culling lower-layer typography (such as background docked panels)
//! hidden beneath opaque higher-layer containers (such as modal windows and popup dropdown menus).

use crate::section::TextSection;
use iris_core::{Rect, TextAlign, UiLayer, UiTree, WidgetId, WidgetRole};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy)]
struct Occluder {
    rect: Rect,
    layer: UiLayer,
    order: usize,
    subtree_max_order: usize,
}

/// Options for configuring text section extraction and layer-based occlusion culling.
#[derive(Debug, Clone, Default)]
pub struct TextCollectionOptions {
    /// Optional additional screen-space opaque occlusion rectangles to cull background text against.
    pub extra_occluders: Vec<Rect>,
}

/// Helper context passed during recursive tree traversal.
struct CollectionContext<'a> {
    clip_rect: Option<Rect>,
    inherited_layer: UiLayer,
    occluders: &'a [Occluder],
    extra_occluders: &'a [Rect],
}

/// Collects shaped text sections from the UI tree with automatic layer-based occlusion culling.
///
/// Text elements on lower layers (e.g. `UiLayer::Content`) positioned behind opaque
/// higher-layer containers (e.g. `UiLayer::Popup`, `UiLayer::Modal`) are automatically
/// scissored or culled to prevent visual bleeding.
pub fn collect_text_sections<'a>(tree: &'a UiTree) -> Vec<TextSection<'a>> {
    collect_text_sections_with_options(tree, &TextCollectionOptions::default())
}

/// Collects shaped text sections from the UI tree using explicit configuration options.
pub fn collect_text_sections_with_options<'a>(
    tree: &'a UiTree,
    options: &TextCollectionOptions,
) -> Vec<TextSection<'a>> {
    let Some(root) = tree.root() else {
        return Vec::new();
    };

    // Step 1: Discover all opaque occluders with tree traversal order
    let mut occluders = Vec::with_capacity(32);
    let mut counter = 0usize;
    discover_layer_occluders(tree, root, UiLayer::Content, &mut counter, &mut occluders);

    // Step 2: Recursively extract text sections with occlusion clipping and culling
    let mut sections = Vec::with_capacity(64);
    let mut text_counter = 0usize;
    let ctx = CollectionContext {
        clip_rect: None,
        inherited_layer: UiLayer::Content,
        occluders: &occluders,
        extra_occluders: &options.extra_occluders,
    };
    collect_node_text(tree, root, &ctx, &mut text_counter, &mut sections);

    sections
}

/// Discovers opaque rectangular containers capable of occluding text beneath their layer.
fn discover_layer_occluders(
    tree: &UiTree,
    current: WidgetId,
    inherited_layer: UiLayer,
    counter: &mut usize,
    occluders: &mut Vec<Occluder>,
) {
    let Some(node) = tree.get(current) else {
        return;
    };
    if !node.visible {
        return;
    }

    let node_order = *counter;
    *counter += 1;

    let effective_layer = if node.layer > inherited_layer {
        node.layer
    } else {
        inherited_layer
    };

    let is_layer_container = matches!(
        node.role,
        WidgetRole::DropdownPopup | WidgetRole::ModalWindow | WidgetRole::FloatingWindow
    ) || effective_layer >= UiLayer::Floating;

    // An occluder MUST be a recognized top-level container AND strictly opaque (alpha >= 0.90).
    // Semi-transparent elements like modal scrims (alpha ~ 0.55) or sub-cards must NEVER occlude background text.
    let is_opaque = node.style.background_color.a >= 0.90;

    let occluder_idx = if is_layer_container
        && is_opaque
        && node.computed_rect.width > 2.0
        && node.computed_rect.height > 2.0
    {
        let idx = occluders.len();
        occluders.push(Occluder {
            rect: node.computed_rect,
            layer: effective_layer,
            order: node_order,
            subtree_max_order: node_order,
        });
        Some(idx)
    } else {
        None
    };

    for &child_id in &node.children {
        discover_layer_occluders(tree, child_id, effective_layer, counter, occluders);
    }

    if let Some(idx) = occluder_idx {
        occluders[idx].subtree_max_order = *counter;
    }
}

/// Recursively processes a widget node, testing text against higher-layer occluders.
fn collect_node_text<'a>(
    tree: &'a UiTree,
    current: WidgetId,
    ctx: &CollectionContext<'_>,
    counter: &mut usize,
    sections: &mut Vec<TextSection<'a>>,
) {
    let Some(node) = tree.get(current) else {
        return;
    };
    if !node.visible {
        return;
    }

    let node_order = *counter;
    *counter += 1;

    let effective_layer = if node.layer > ctx.inherited_layer {
        node.layer
    } else {
        ctx.inherited_layer
    };

    // Calculate hierarchical scissor clipping
    let child_clip = if node.style.clip_children {
        match ctx.clip_rect {
            Some(existing) => Some(existing.intersect(node.computed_rect)),
            None => Some(node.computed_rect),
        }
    } else {
        ctx.clip_rect
    };

    // If node has text and valid dimensions, test occlusion and collect
    if let Some(text) = &node.text
        && !text.is_empty()
        && node.computed_rect.width > 0.0
        && node.computed_rect.height > 0.0
    {
        let mut effective_clip = child_clip;

        // Approximate visual horizontal footprint of the text
        let text_char_count = text.chars().count() as f32;
        let approx_text_width = text_char_count * (node.font_size * 0.62);
        let (text_min_x, text_max_x) = match node.text_align {
            TextAlign::Left => (
                node.computed_rect.x,
                (node.computed_rect.x + approx_text_width).min(node.computed_rect.right()),
            ),
            TextAlign::Right => (
                (node.computed_rect.right() - approx_text_width).max(node.computed_rect.x),
                node.computed_rect.right(),
            ),
            TextAlign::Center => {
                let cx = node.computed_rect.x + node.computed_rect.width * 0.5;
                (
                    (cx - approx_text_width * 0.5).max(node.computed_rect.x),
                    (cx + approx_text_width * 0.5).min(node.computed_rect.right()),
                )
            }
        };

        let text_center_y = node.computed_rect.y + node.computed_rect.height * 0.5;
        let mut is_fully_occluded = false;

        // Test against occluders that are on a higher layer OR drawn after this node on the same layer
        for occluder in ctx.occluders {
            // A container never occludes its own subtree descendants
            if node_order >= occluder.order && node_order <= occluder.subtree_max_order {
                continue;
            }

            let is_higher_z = occluder.layer > effective_layer
                || (occluder.layer == effective_layer && occluder.order > node_order);

            if !is_higher_z {
                continue;
            }

            let vert_overlap = node.computed_rect.bottom() > occluder.rect.y
                && node.computed_rect.y < occluder.rect.bottom();
            if !vert_overlap {
                continue;
            }
            let horiz_overlap = text_max_x > occluder.rect.x && text_min_x < occluder.rect.right();
            if !horiz_overlap {
                continue;
            }

            // If the entire text bounding footprint falls within the opaque occluder, cull it completely
            if text_min_x >= occluder.rect.x
                && text_max_x <= occluder.rect.right()
                && text_center_y >= occluder.rect.y
                && text_center_y <= occluder.rect.bottom()
            {
                is_fully_occluded = true;
                break;
            }

            // If text is vertically within the occluder and partially covered horizontally, scissor-clip it
            let text_vertically_covered = node.computed_rect.y >= occluder.rect.y
                && node.computed_rect.bottom() <= occluder.rect.bottom();
            if text_vertically_covered {
                if text_min_x < occluder.rect.x && text_max_x > occluder.rect.x {
                    let clip_sub = Rect::new(
                        0.0,
                        node.computed_rect.y,
                        occluder.rect.x,
                        node.computed_rect.height,
                    );
                    effective_clip = match effective_clip {
                        Some(c) => Some(c.intersect(clip_sub)),
                        None => Some(clip_sub),
                    };
                } else if text_min_x < occluder.rect.right() && text_max_x > occluder.rect.right() {
                    let clip_sub = Rect::new(
                        occluder.rect.right(),
                        node.computed_rect.y,
                        100_000.0,
                        node.computed_rect.height,
                    );
                    effective_clip = match effective_clip {
                        Some(c) => Some(c.intersect(clip_sub)),
                        None => Some(clip_sub),
                    };
                }
            }
        }

        // Also test against any extra occluders provided by host application (only for content/background layers)
        if effective_layer <= UiLayer::Content {
            for occluder in ctx.extra_occluders {
                if is_fully_occluded {
                    break;
                }
                let vert_overlap = node.computed_rect.bottom() > occluder.y
                    && node.computed_rect.y < occluder.bottom();
                if !vert_overlap {
                    continue;
                }
                let horiz_overlap = text_max_x > occluder.x && text_min_x < occluder.right();
                if !horiz_overlap {
                    continue;
                }

                if text_min_x >= occluder.x
                    && text_max_x <= occluder.right()
                    && text_center_y >= occluder.y
                    && text_center_y <= occluder.bottom()
                {
                    is_fully_occluded = true;
                    break;
                }
            }
        }

        if !is_fully_occluded {
            let section = TextSection {
                text: Cow::Borrowed(text.as_str()),
                font_size: node.font_size,
                line_height: node.line_height,
                color: node.text_color,
                align: node.text_align,
                wrap: node.text_wrap,
                bounds: node.computed_rect,
                clip_bounds: effective_clip,
            };
            sections.push(section);
        }
    }

    let child_ctx = CollectionContext {
        clip_rect: child_clip,
        inherited_layer: effective_layer,
        occluders: ctx.occluders,
        extra_occluders: ctx.extra_occluders,
    };

    for &child_id in &node.children {
        collect_node_text(tree, child_id, &child_ctx, counter, sections);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::{Color, Rect, UiTree};

    #[test]
    fn test_collect_text_sections_basic() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
        }

        let label = tree.create_node();
        if let Some(node) = tree.get_mut(label) {
            node.computed_rect = Rect::new(10.0, 10.0, 100.0, 24.0);
            node.text = Some("Hello World".to_string());
        }
        tree.add_child(root, label).unwrap();

        let sections = collect_text_sections(&tree);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].text, "Hello World");
    }

    #[test]
    fn test_collect_text_occlusion_culling() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
        }

        // Background label at (50, 50, 100, 30) on Content layer
        let bg_label = tree.create_node();
        if let Some(node) = tree.get_mut(bg_label) {
            node.computed_rect = Rect::new(50.0, 50.0, 100.0, 30.0);
            node.text = Some("Occluded Text".to_string());
            node.layer = UiLayer::Content;
        }
        tree.add_child(root, bg_label).unwrap();

        // Popup covering (40, 40, 150, 100) on Popup layer
        let popup = tree.create_node();
        if let Some(node) = tree.get_mut(popup) {
            node.computed_rect = Rect::new(40.0, 40.0, 150.0, 100.0);
            node.layer = UiLayer::Popup;
            node.style.background_color = Color::BLACK; // Opaque
        }
        tree.add_child(root, popup).unwrap();

        // Popup label inside the popup
        let popup_label = tree.create_node();
        if let Some(node) = tree.get_mut(popup_label) {
            node.computed_rect = Rect::new(50.0, 60.0, 100.0, 20.0);
            node.text = Some("Popup Text".to_string());
            node.layer = UiLayer::Popup;
        }
        tree.add_child(popup, popup_label).unwrap();

        let sections = collect_text_sections(&tree);
        // The background label MUST be culled completely, only popup_label should survive!
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].text, "Popup Text");
    }
}