// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dock tab overflow popup dropdown menu builder and interaction evaluator.
//!
//! Provides hardware-accelerated GPU SDF tab overflow popup menus displaying
//! overflowing tab lists when a leaf pane cannot fit all tabs horizontally.
//! Supports texture atlas icons, active tab checkmarks, hovered/active color states,
//! retained-mode tagging (`node.tag = tab_index as u64`), and deterministic click evaluation.
//!

use crate::tab_viewer::TabViewer;
use crate::tree::DockNodeId;
use iris_core::color::Color;
use iris_core::geometry::{CornerRadii, Point, Rect};
use iris_core::id::WidgetId;
use iris_core::node::{UiLayer, WidgetRole};
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Visual styling configuration for a dock tab overflow dropdown popup menu.
#[derive(Debug, Clone, PartialEq)]
pub struct DockOverflowMenuStyle {
    /// Background color of the popup menu container card.
    pub background: Color,
    /// Border stroke color of the menu container.
    pub border_color: Color,
    /// Border width in logical pixels.
    pub border_width: f32,
    /// Corner radius in logical pixels.
    pub corner_radius: f32,
    /// Height of each individual tab option row in logical pixels.
    pub item_height: f32,
    /// Padding around the menu items inside the container in logical pixels.
    pub menu_padding: f32,
    /// Total width of the popup menu card in logical pixels.
    pub menu_width: f32,
    /// Background color for a tab option that is both active and hovered.
    pub item_active_hover_bg: Color,
    /// Border color for a tab option that is both active and hovered.
    pub item_active_hover_border: Color,
    /// Background color for an unselected hovered tab option.
    pub item_hover_bg: Color,
    /// Border color for an unselected hovered tab option.
    pub item_hover_border: Color,
    /// Background color for an active (selected) tab option in idle state.
    pub item_active_bg: Color,
    /// Border color for an active (selected) tab option in idle state.
    pub item_active_border: Color,
    /// Accent color used for active text, icons, and checkmarks.
    pub accent_color: Color,
    /// Text color for the active tab option.
    pub text_active_color: Color,
    /// Text color for a hovered tab option.
    pub text_hover_color: Color,
    /// Text color for an unselected, idle tab option.
    pub text_normal_color: Color,
    /// Font size in logical pixels for tab titles.
    pub font_size: f32,
}

impl Default for DockOverflowMenuStyle {
    fn default() -> Self {
        Self {
            background: Color::from_u8(22, 26, 36, 255),
            border_color: Color::from_u8(50, 60, 80, 200),
            border_width: 1.0,
            corner_radius: 4.0,
            item_height: 26.0,
            menu_padding: 4.0,
            menu_width: 180.0,
            item_active_hover_bg: Color::from_u8(38, 64, 96, 255),
            item_active_hover_border: Color::from_u8(64, 160, 220, 180),
            item_hover_bg: Color::from_u8(36, 48, 68, 255),
            item_hover_border: Color::from_u8(70, 110, 160, 160),
            item_active_bg: Color::from_u8(28, 42, 60, 220),
            item_active_border: Color::from_u8(40, 75, 110, 140),
            accent_color: Color::rgba(0.0, 0.90, 1.0, 1.0),
            text_active_color: Color::rgba(0.0, 0.90, 1.0, 1.0),
            text_hover_color: Color::WHITE,
            text_normal_color: Color::rgba(0.85, 0.88, 0.94, 1.0),
            font_size: 11.5,
        }
    }
}

/// Hit target descriptor for an individual tab option inside an active overflow dropdown menu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockOverflowItemTarget {
    /// Leaf node owning the overflowing tab bar.
    pub leaf: DockNodeId,
    /// Zero-based index of the tab within the owning leaf.
    pub tab_index: usize,
    /// Absolute computed screen-space rectangle of the dropdown option row.
    pub rect: Rect,
}

/// Output frame produced when building a dock tab overflow dropdown menu into a [`UiTree`].
#[derive(Debug, Clone, PartialEq)]
pub struct DockOverflowMenuFrame {
    /// Generational node identifier for the popup container quad.
    pub menu_node_id: WidgetId,
    /// Absolute computed screen-space rectangle of the entire popup menu card.
    pub menu_rect: Rect,
    /// Geometric hit targets for each tab option row in the menu.
    pub items: Vec<DockOverflowItemTarget>,
}

/// Action resulting from evaluating a mouse click against an active dock overflow menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockOverflowClickAction {
    /// User clicked a tab option, requesting activation of that tab.
    SelectTab {
        /// Leaf node hosting the tab.
        leaf: DockNodeId,
        /// Tab index to activate.
        tab_index: usize,
    },
    /// User clicked the chevron button, toggling or switching the active overflow menu.
    ToggleChevron {
        /// Leaf node whose chevron was clicked.
        leaf: DockNodeId,
    },
    /// User clicked outside the overflow popup and chevron, requesting dismissal.
    Dismiss,
}

/// Parameters describing the active leaf, tabs, and layout geometry for building an overflow popup menu.
#[derive(Debug, Clone)]
pub struct DockOverflowMenuParams<'a, Tab> {
    /// Leaf node owning the overflowing tab bar.
    pub leaf_id: DockNodeId,
    /// Slice of tabs hosted within the leaf.
    pub tabs: &'a [Tab],
    /// Currently active tab index in the leaf.
    pub active_tab: usize,
    /// Bounding rectangle of the trigger chevron button.
    pub anchor_rect: Rect,
    /// Current logical mouse cursor coordinates.
    pub cursor_pos: Point,
}

/// Builds the dock tab overflow dropdown popup menu into the target [`UiTree`].
///
/// Anchors the menu immediately below the trigger chevron button aligned to its right edge.
/// Generates nodes tagged with `node.tag = tab_index as u64` and role [`WidgetRole::DropdownItem`]
/// on [`UiLayer::Popup`], ensuring proper stacking and zero-allocation hit testing.
pub fn build_dock_overflow_menu<Tab, V: TabViewer<Tab>>(
    tree: &mut UiTree,
    parent: WidgetId,
    params: DockOverflowMenuParams<'_, Tab>,
    viewer: &V,
    style: &DockOverflowMenuStyle,
) -> Option<DockOverflowMenuFrame> {
    if params.tabs.is_empty() {
        return None;
    }

    let menu_height = (params.tabs.len() as f32 * style.item_height) + (style.menu_padding * 2.0);
    // Anchor below the chevron button, aligned to its right edge
    let menu_x = (params.anchor_rect.right() - style.menu_width).max(0.0);
    let menu_y = params.anchor_rect.bottom() + 2.0;
    let menu_rect = Rect::new(menu_x, menu_y, style.menu_width, menu_height);

    // 1. Menu container card
    let menu_card_id = tree.create_node();
    if let Some(node) = tree.get_mut(menu_card_id) {
        node.set_name("IrisDockOverflowMenu");
        node.set_role(WidgetRole::DropdownPopup);
        node.set_layer(UiLayer::Popup);
        node.computed_rect = menu_rect;
        node.style = Style::new()
            .background(style.background)
            .border(style.border_width, style.border_color)
            .corner_radii(CornerRadii::all(style.corner_radius))
            .clip_children(true);
    }
    let _ = tree.add_child(parent, menu_card_id);

    let mut items = Vec::with_capacity(params.tabs.len());

    // 2. Populate tab option rows
    for (index, tab) in params.tabs.iter().enumerate() {
        let is_active = index == params.active_tab;
        let item_y = menu_y + style.menu_padding + (index as f32 * style.item_height);
        let item_rect = Rect::new(
            menu_x + style.menu_padding,
            item_y,
            style.menu_width - (style.menu_padding * 2.0),
            style.item_height,
        );
        let is_hovered = item_rect.contains_point(params.cursor_pos);

        let (item_bg, item_border_col) = if is_hovered && is_active {
            (style.item_active_hover_bg, style.item_active_hover_border)
        } else if is_hovered {
            (style.item_hover_bg, style.item_hover_border)
        } else if is_active {
            (style.item_active_bg, style.item_active_border)
        } else {
            (Color::TRANSPARENT, Color::TRANSPARENT)
        };

        let item_id = tree.create_node();
        if let Some(node) = tree.get_mut(item_id) {
            node.set_name("IrisDockOverflowItem");
            node.set_role(WidgetRole::DropdownItem);
            node.set_layer(UiLayer::Popup);
            node.set_tag(index as u64);
            node.computed_rect = item_rect;
            node.style = Style::new()
                .background(item_bg)
                .border(1.0, item_border_col)
                .corner_radii(CornerRadii::all(3.0));
        }
        let _ = tree.add_child(menu_card_id, item_id);

        let atlas_icon = viewer.atlas_icon(tab);
        let title = viewer.raw_title(tab);

        // Optional texture atlas icon
        if let Some(uv) = atlas_icon {
            let icon_id = tree.create_node();
            if let Some(node) = tree.get_mut(icon_id) {
                node.set_name("IrisDockOverflowItemIcon");
                node.interactive = false;
                node.computed_rect = Rect::new(item_rect.x + 6.0, item_rect.y + 5.0, 16.0, 16.0);
                node.set_texture_uv(uv);
                let tint = if is_active {
                    style.accent_color
                } else if is_hovered {
                    Color::WHITE
                } else {
                    Color::rgba(0.70, 0.75, 0.85, 1.0)
                };
                node.set_texture_tint(tint);
            }
            let _ = tree.add_child(item_id, icon_id);
        }

        let label_offset_x = if atlas_icon.is_some() { 26.0 } else { 8.0 };
        let label_width =
            (item_rect.width - label_offset_x - if is_active { 20.0 } else { 4.0 }).max(10.0);
        let label_rect = Rect::new(
            item_rect.x + label_offset_x,
            item_rect.y + 4.0,
            label_width,
            18.0,
        );
        let text_col = if is_active {
            style.text_active_color
        } else if is_hovered {
            style.text_hover_color
        } else {
            style.text_normal_color
        };

        let label_id = tree.create_node();
        if let Some(node) = tree.get_mut(label_id) {
            node.set_name("IrisDockOverflowItemLabel");
            node.interactive = false;
            node.computed_rect = label_rect;
            node.set_text(&title);
            node.font_size = style.font_size;
            node.text_color = text_col;
            node.text_align = TextAlign::Left;
        }
        let _ = tree.add_child(item_id, label_id);

        // Active tab checkmark indicator
        if is_active {
            let check_rect = Rect::new(item_rect.right() - 20.0, item_rect.y + 4.0, 16.0, 18.0);
            let check_id = tree.create_node();
            if let Some(node) = tree.get_mut(check_id) {
                node.set_name("IrisDockOverflowItemCheck");
                node.interactive = false;
                node.computed_rect = check_rect;
                node.set_text("✓");
                node.font_size = style.font_size;
                node.text_color = style.accent_color;
                node.text_align = TextAlign::Center;
            }
            let _ = tree.add_child(item_id, check_id);
        }

        items.push(DockOverflowItemTarget {
            leaf: params.leaf_id,
            tab_index: index,
            rect: item_rect,
        });
    }

    Some(DockOverflowMenuFrame {
        menu_node_id: menu_card_id,
        menu_rect,
        items,
    })
}

/// Evaluates a mouse click event against an active dock tab overflow menu.
///
/// Determines whether the user clicked an item in the menu, clicked the chevron to toggle,
/// or clicked elsewhere outside to dismiss.
pub fn evaluate_dock_overflow_click(
    click_point: Point,
    active_leaf: DockNodeId,
    chevron_rect: Rect,
    frame: Option<&DockOverflowMenuFrame>,
) -> DockOverflowClickAction {
    // 1. Check if clicking on any item inside the active overflow dropdown menu
    if let Some(frame) = frame {
        for target in &frame.items {
            if target.rect.contains_point(click_point) {
                return DockOverflowClickAction::SelectTab {
                    leaf: target.leaf,
                    tab_index: target.tab_index,
                };
            }
        }
    }

    // 2. Check if clicking on the chevron button itself
    if chevron_rect.contains_point(click_point) {
        return DockOverflowClickAction::ToggleChevron { leaf: active_leaf };
    }

    // 3. Clicked anywhere else outside
    DockOverflowClickAction::Dismiss
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tab_viewer::SimpleTabViewer;
    use crate::tree::DockTree;

    #[test]
    fn test_dock_overflow_menu_build_and_tagging() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        }
        let _ = tree.set_root(root);

        let mut dock_tree = DockTree::<&'static str>::new();
        let leaf_id = dock_tree.create_leaf(vec!["Console", "Assets", "Hierarchy", "Inspector"]);
        dock_tree.set_root(leaf_id);

        let tabs = vec!["Console", "Assets", "Hierarchy", "Inspector"];
        let viewer = SimpleTabViewer;
        let anchor = Rect::new(300.0, 10.0, 24.0, 24.0);
        let cursor = Point::new(250.0, 40.0);
        let style = DockOverflowMenuStyle::default();

        let frame = build_dock_overflow_menu(
            &mut tree,
            root,
            DockOverflowMenuParams {
                leaf_id,
                tabs: &tabs,
                active_tab: 1, // "Assets" is active
                anchor_rect: anchor,
                cursor_pos: cursor,
            },
            &viewer,
            &style,
        )
        .expect("Dock overflow menu frame must be constructed");

        assert_eq!(frame.items.len(), 4);
        assert!(frame.menu_rect.width > 100.0);

        // Verify that items are properly tagged with their indices and roles in UiTree
        for (expected_idx, item_target) in frame.items.iter().enumerate() {
            assert_eq!(item_target.leaf, leaf_id);
            assert_eq!(item_target.tab_index, expected_idx);

            let hit = tree
                .hit_test_target(Point::new(
                    item_target.rect.x + 10.0,
                    item_target.rect.y + 10.0,
                ))
                .expect("Item point must be hit in tree");

            assert_eq!(hit.layer, UiLayer::Popup);
            assert_eq!(hit.role, WidgetRole::DropdownItem);
            assert_eq!(hit.tag, expected_idx as u64);
        }
    }

    #[test]
    fn test_evaluate_dock_overflow_click() {
        let mut dock_tree = DockTree::<&'static str>::new();
        let leaf_id = dock_tree.create_leaf(vec!["Tab A", "Tab B"]);
        dock_tree.set_root(leaf_id);

        let chevron_rect = Rect::new(400.0, 10.0, 24.0, 24.0);

        let item0_rect = Rect::new(250.0, 40.0, 172.0, 26.0);
        let item1_rect = Rect::new(250.0, 68.0, 172.0, 26.0);

        let mut tree = UiTree::new();
        let dummy_node_id = tree.create_node();

        let frame = DockOverflowMenuFrame {
            menu_node_id: dummy_node_id,
            menu_rect: Rect::new(246.0, 36.0, 180.0, 60.0),
            items: vec![
                DockOverflowItemTarget {
                    leaf: leaf_id,
                    tab_index: 0,
                    rect: item0_rect,
                },
                DockOverflowItemTarget {
                    leaf: leaf_id,
                    tab_index: 1,
                    rect: item1_rect,
                },
            ],
        };

        // 1. Click on item 1
        let action1 = evaluate_dock_overflow_click(
            Point::new(260.0, 75.0),
            leaf_id,
            chevron_rect,
            Some(&frame),
        );
        assert_eq!(
            action1,
            DockOverflowClickAction::SelectTab {
                leaf: leaf_id,
                tab_index: 1
            }
        );

        // 2. Click on chevron button
        let action_chev = evaluate_dock_overflow_click(
            Point::new(410.0, 15.0),
            leaf_id,
            chevron_rect,
            Some(&frame),
        );
        assert_eq!(
            action_chev,
            DockOverflowClickAction::ToggleChevron { leaf: leaf_id }
        );

        // 3. Click outside
        let action_outside = evaluate_dock_overflow_click(
            Point::new(10.0, 10.0),
            leaf_id,
            chevron_rect,
            Some(&frame),
        );
        assert_eq!(action_outside, DockOverflowClickAction::Dismiss);
    }
}