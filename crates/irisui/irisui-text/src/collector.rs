// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Retained UI tree text collection pipeline with automatic layer-aware occlusion culling.
//!
//! Extracts renderable `TextSection` instances from an `irisui_core::UiTree`, computing
//! hierarchical scissor clipping and culling lower-layer typography (such as background docked panels)
//! hidden beneath opaque higher-layer containers (such as modal windows and popup dropdown menus).

use crate::section::TextSection;
use irisui_core::{
    CanvasPrimitive, Color, Rect, TextAlign, TextWrap, UiLayer, UiTree, WidgetId, WidgetRole,
};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy)]
struct Occluder {
    rect: Rect,
    layer: UiLayer,
    order: usize,
    subtree_max_order: usize,
}

/// Partitioning filter separating static panel typography from high-frequency volatile text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextVolatilityFilter {
    /// Collect all visible text sections in the tree.
    #[default]
    All,
    /// Collect only static retained widget labels (excluding live tags and canvas primitives).
    StaticOnly,
    /// Collect only volatile/live widget labels (`UiTree::is_live_tag`) and canvas `Text` primitives.
    VolatileOnly,
}

/// Options for configuring text section extraction and layer-based occlusion culling.
#[derive(Debug, Clone, Default)]
pub struct TextCollectionOptions {
    /// Optional additional screen-space opaque occlusion rectangles to cull background text against.
    pub extra_occluders: Vec<Rect>,
    /// Optional hovered semantic tag for highlighting text foreground colors.
    pub hovered_tag: Option<u64>,
    /// Optional hovered widget identifier for highlighting text foreground colors.
    pub hovered_id: Option<WidgetId>,
    /// Optional focused semantic tag for highlighting text foreground colors and controlling caret visibility.
    pub focused_tag: Option<u64>,
    /// Optional focused widget identifier for highlighting text foreground colors and controlling caret visibility.
    pub focused_id: Option<WidgetId>,
    /// Whether text input caret cursors are in the active visible blink phase.
    pub blink_caret: bool,
    /// Optional inclusive layer elevation range `(min, max)` to collect.
    pub layer_range: Option<(UiLayer, UiLayer)>,
    /// Volatility partition filter isolating static UI text from high-frequency live text.
    pub volatility_filter: TextVolatilityFilter,
}

impl TextCollectionOptions {
    /// Creates a new default configuration for text collection.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets an inclusive layer elevation range `(min, max)` to filter collected typography.
    #[must_use]
    #[inline]
    pub fn with_layer_range(mut self, min: UiLayer, max: UiLayer) -> Self {
        self.layer_range = Some((min, max));
        self
    }

    /// Sets the hovered semantic tag for dynamic hover text highlighting.
    #[must_use]
    #[inline]
    pub fn with_hovered_tag(mut self, tag: Option<u64>) -> Self {
        self.hovered_tag = tag;
        self
    }

    /// Sets the hovered widget identifier for dynamic hover text highlighting.
    #[must_use]
    #[inline]
    pub fn with_hovered_id(mut self, id: Option<WidgetId>) -> Self {
        self.hovered_id = id;
        self
    }

    /// Sets the focused semantic tag for dynamic focus text highlighting.
    #[must_use]
    #[inline]
    pub fn with_focused_tag(mut self, tag: Option<u64>) -> Self {
        self.focused_tag = tag;
        self
    }

    /// Sets the focused widget identifier for dynamic focus text highlighting.
    #[must_use]
    #[inline]
    pub fn with_focused_id(mut self, id: Option<WidgetId>) -> Self {
        self.focused_id = id;
        self
    }

    /// Sets whether text carets are currently visible during the blink cycle.
    #[must_use]
    #[inline]
    pub fn with_blink_caret(mut self, blink: bool) -> Self {
        self.blink_caret = blink;
        self
    }

    /// Sets the volatility filter isolating static UI text from high-frequency live text.
    #[must_use]
    #[inline]
    pub fn with_volatility(mut self, filter: TextVolatilityFilter) -> Self {
        self.volatility_filter = filter;
        self
    }
}

/// Helper context passed during recursive tree traversal.
struct CollectionContext<'a> {
    clip_rect: Option<Rect>,
    inherited_layer: UiLayer,
    inherited_hovered: bool,
    inherited_focused: bool,
    hovered_tag: Option<u64>,
    hovered_id: Option<WidgetId>,
    focused_tag: Option<u64>,
    focused_id: Option<WidgetId>,
    blink_caret: bool,
    occluders: &'a [Occluder],
    extra_occluders: &'a [Rect],
}

struct CollectionTargets<'r, 'a> {
    filter: TextVolatilityFilter,
    primary_layers: &'r mut [Vec<TextSection<'a>>; 6],
    volatile_layers: Option<&'r mut [Vec<TextSection<'a>>; 6]>,
}

impl<'r, 'a> CollectionTargets<'r, 'a> {
    #[inline]
    fn accepts(&self, is_volatile: bool) -> bool {
        if self.volatile_layers.is_some() {
            return true;
        }
        match self.filter {
            TextVolatilityFilter::All => true,
            TextVolatilityFilter::StaticOnly => !is_volatile,
            TextVolatilityFilter::VolatileOnly => is_volatile,
        }
    }

    #[inline]
    fn push(&mut self, layer_idx: usize, is_volatile: bool, section: TextSection<'a>) {
        if let Some(volatile_layers) = &mut self.volatile_layers {
            if is_volatile {
                volatile_layers[layer_idx].push(section);
            } else {
                self.primary_layers[layer_idx].push(section);
            }
            return;
        }
        if self.accepts(is_volatile) {
            self.primary_layers[layer_idx].push(section);
        }
    }
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
    let mut single_layers: [Vec<TextSection<'a>>; 6] = Default::default();
    collect_partitioned_layers(
        tree,
        options,
        CollectionTargets {
            filter: options.volatility_filter,
            primary_layers: &mut single_layers,
            volatile_layers: None,
        },
    );
    flatten_layer_buckets(single_layers, options.layer_range)
}

/// Collects both static and volatile text sections in a single tree traversal with shared
/// occlusion discovery, returning `(static_sections, volatile_sections)`.
pub fn collect_split_text_sections_with_options<'a>(
    tree: &'a UiTree,
    options: &TextCollectionOptions,
) -> (Vec<TextSection<'a>>, Vec<TextSection<'a>>) {
    let mut static_layers: [Vec<TextSection<'a>>; 6] = Default::default();
    let mut volatile_layers: [Vec<TextSection<'a>>; 6] = Default::default();
    collect_partitioned_layers(
        tree,
        options,
        CollectionTargets {
            filter: TextVolatilityFilter::All,
            primary_layers: &mut static_layers,
            volatile_layers: Some(&mut volatile_layers),
        },
    );
    (
        flatten_layer_buckets(static_layers, options.layer_range),
        flatten_layer_buckets(volatile_layers, options.layer_range),
    )
}

fn collect_partitioned_layers<'a>(
    tree: &'a UiTree,
    options: &TextCollectionOptions,
    mut targets: CollectionTargets<'_, 'a>,
) {
    let Some(root) = tree.root() else {
        return;
    };

    let mut occluders = Vec::with_capacity(32);
    let mut counter = 0usize;
    discover_layer_occluders(
        tree,
        root,
        UiLayer::Background,
        &mut counter,
        &mut occluders,
    );

    let mut text_counter = 0usize;
    let ctx = CollectionContext {
        clip_rect: None,
        inherited_layer: UiLayer::Background,
        inherited_hovered: false,
        inherited_focused: false,
        hovered_tag: options.hovered_tag,
        hovered_id: options.hovered_id,
        focused_tag: options.focused_tag,
        focused_id: options.focused_id,
        blink_caret: options.blink_caret,
        occluders: &occluders,
        extra_occluders: &options.extra_occluders,
    };
    collect_node_text(tree, root, &ctx, &mut text_counter, &mut targets);
}

fn flatten_layer_buckets<'a>(
    buckets: [Vec<TextSection<'a>>; 6],
    layer_range: Option<(UiLayer, UiLayer)>,
) -> Vec<TextSection<'a>> {
    let total_count: usize = buckets.iter().map(|s| s.len()).sum();
    let mut sections = Vec::with_capacity(total_count);
    for (layer_idx, layer) in buckets.into_iter().enumerate() {
        if let Some((min_layer, max_layer)) = layer_range
            && (layer_idx < min_layer.index() || layer_idx > max_layer.index())
        {
            continue;
        }
        sections.extend(layer);
    }
    sections
}

#[inline]
fn resolve_effective_layer(node: &irisui_core::WidgetNode, inherited_layer: UiLayer) -> UiLayer {
    let explicit_or_default = if node.layer != UiLayer::Content {
        node.layer
    } else {
        node.role.default_layer()
    };
    if explicit_or_default > inherited_layer {
        explicit_or_default
    } else {
        inherited_layer
    }
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

    let effective_layer = resolve_effective_layer(node, inherited_layer);

    let is_layer_container = matches!(
        node.role,
        WidgetRole::DropdownPopup
            | WidgetRole::ModalWindow
            | WidgetRole::FloatingWindow
            | WidgetRole::Tooltip
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
    targets: &mut CollectionTargets<'_, 'a>,
) {
    let Some(node) = tree.get(current) else {
        return;
    };
    if !node.visible {
        return;
    }

    let node_order = *counter;
    *counter += 1;

    let effective_layer = resolve_effective_layer(node, ctx.inherited_layer);

    // A node is an elevated overlay root if its explicit or role-inferred layer exceeds the inherited layer,
    // or if its semantic role designates an independent floating/overlay container
    // (DropdownPopup, ModalWindow, FloatingWindow, Tooltip).
    let is_elevated_root = effective_layer > ctx.inherited_layer
        || matches!(
            node.role,
            WidgetRole::DropdownPopup
                | WidgetRole::ModalWindow
                | WidgetRole::FloatingWindow
                | WidgetRole::Tooltip
        );

    // Decouple scissor clip when transitioning into an elevated overlay layer (e.g. Popup or Modal)
    // so that child popups or modal cards are not clipped by parent panel boundaries.
    let incoming_clip = if is_elevated_root {
        None
    } else {
        ctx.clip_rect
    };

    // Calculate hierarchical scissor clipping
    let child_clip = if node.style.clip_children {
        match incoming_clip {
            Some(existing) => Some(existing.intersect(node.computed_rect)),
            None => Some(node.computed_rect),
        }
    } else {
        incoming_clip
    };

    let node_matches_hover = (ctx.hovered_id.is_some() && ctx.hovered_id == Some(current))
        || (node.tag != 0 && ctx.hovered_tag == Some(node.tag))
        || (node.role == WidgetRole::DockTab
            && ctx
                .hovered_id
                .is_some_and(|hid| tree.is_descendant_of(hid, current)));
    let is_hovered = node_matches_hover || (ctx.inherited_hovered && node.tag == 0);

    let node_matches_focus = (ctx.focused_id.is_some() && ctx.focused_id == Some(current))
        || (node.tag != 0 && ctx.focused_tag == Some(node.tag));
    let is_focused = node_matches_focus || (ctx.inherited_focused && node.tag == 0);

    // Skip collecting text for caret indicator if un-focused or in blink-off phase
    if node.role == WidgetRole::TextInputCaret && (!ctx.inherited_focused || !ctx.blink_caret) {
        return;
    }

    let is_node_volatile = node.tag != 0 && tree.is_live_tag(node.tag);

    // If node has text and valid dimensions, test occlusion and collect
    if targets.accepts(is_node_volatile)
        && let Some(text) = &node.text
        && !text.is_empty()
        && node.computed_rect.width > 0.0
        && node.computed_rect.height > 0.0
    {
        // Hierarchical scissor culling: If this text node lies entirely outside the active
        // scissor clipping boundary (e.g. from parent panel with clip_children: true),
        // skip collection immediately to prevent bleeding into adjacent UI panels or status bar.
        let is_clipped_out = if let Some(clip) = child_clip {
            node.computed_rect.bottom() <= clip.y
                || node.computed_rect.y >= clip.bottom()
                || node.computed_rect.right() <= clip.x
                || node.computed_rect.x >= clip.right()
        } else {
            false
        };

        if !is_clipped_out {
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
                let horiz_overlap =
                    text_max_x > occluder.rect.x && text_min_x < occluder.rect.right();
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
                    } else if text_min_x < occluder.rect.right()
                        && text_max_x > occluder.rect.right()
                    {
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
                let color = if is_focused {
                    if let Some(fc) = node.focus_text_color {
                        fc
                    } else if is_hovered {
                        if let Some(hc) = node.hover_text_color {
                            hc
                        } else if matches!(
                            node.role,
                            WidgetRole::Button | WidgetRole::MenuBarItem | WidgetRole::DropdownItem
                        ) {
                            Color::WHITE
                        } else {
                            node.text_color
                        }
                    } else {
                        node.text_color
                    }
                } else if is_hovered {
                    if let Some(hc) = node.hover_text_color {
                        hc
                    } else if matches!(
                        node.role,
                        WidgetRole::Button | WidgetRole::MenuBarItem | WidgetRole::DropdownItem
                    ) {
                        Color::WHITE
                    } else {
                        node.text_color
                    }
                } else {
                    node.text_color
                };

                let section = TextSection {
                    text: Cow::Borrowed(text.as_str()),
                    font_size: node.font_size,
                    line_height: node.line_height,
                    color,
                    align: node.text_align,
                    wrap: node.text_wrap,
                    bounds: node.computed_rect,
                    clip_bounds: effective_clip,
                    layer: effective_layer,
                };
                targets.push(effective_layer.index(), is_node_volatile, section);
            }
        }
    }

    // Extract volatile immediate-mode canvas text primitives if registered on this node
    if targets.accepts(true)
        && let Some(primitives) = tree.canvas_primitives(current)
    {
        let origin_offset = node.canvas_origin.resolve_offset(node.computed_rect);
        for prim in primitives {
            if let CanvasPrimitive::Text {
                pos,
                text,
                font_size,
                color,
            } = prim
                && !text.is_empty()
            {
                let abs_x = pos.x + origin_offset.x;
                let abs_y = pos.y + origin_offset.y;
                let text_bounds = Rect::new(abs_x, abs_y, 10_000.0, *font_size + 4.0);
                let is_clipped_out = if let Some(clip) = child_clip {
                    abs_y + *font_size + 4.0 <= clip.y
                        || abs_y >= clip.bottom()
                        || abs_x >= clip.right()
                } else {
                    false
                };
                if !is_clipped_out {
                    let section = TextSection {
                        text: Cow::Borrowed(text.as_str()),
                        font_size: *font_size,
                        line_height: *font_size * 1.2,
                        color: *color,
                        align: TextAlign::Left,
                        wrap: TextWrap::None,
                        bounds: text_bounds,
                        clip_bounds: child_clip,
                        layer: effective_layer,
                    };
                    targets.push(effective_layer.index(), true, section);
                }
            }
        }
    }

    let child_ctx = CollectionContext {
        clip_rect: child_clip,
        inherited_layer: effective_layer,
        inherited_hovered: is_hovered,
        inherited_focused: is_focused,
        hovered_tag: ctx.hovered_tag,
        hovered_id: ctx.hovered_id,
        focused_tag: ctx.focused_tag,
        focused_id: ctx.focused_id,
        blink_caret: ctx.blink_caret,
        occluders: ctx.occluders,
        extra_occluders: ctx.extra_occluders,
    };

    for &child_id in &node.children {
        collect_node_text(tree, child_id, &child_ctx, counter, targets);
    }
}

#[cfg(test)]
#[path = "collector_tests.rs"]
mod tests;