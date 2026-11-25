// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Layout-Anchored Select Pill (`UiScope::anchored_select_pill`)
//!
//! Couples a toolbar trigger button with an elevated anchored dropdown menu card whose
//! position resolves directly from [`irisui_core::UiTree::resolve_anchor_rect`] without consumer pixel arithmetic.

use super::core::UiScope;
use super::menu::PopupPlacement;
use super::scoped_popup::AnchoredPopupConfig;
use crate::declarative::types::WidgetResponse;
use irisui_core::{CornerRadii, Rect};

/// Single selectable option rendered inside [`UiScope::anchored_select_pill`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnchoredSelectOption<'a> {
    /// Semantic interaction tag assigned to this menu row.
    pub tag: u64,
    /// Leading glyph or icon caption displayed before the label.
    pub icon: &'a str,
    /// User-visible option label.
    pub label: &'a str,
}

impl<'a> AnchoredSelectOption<'a> {
    /// Constructs a new option entry for an anchored select dropdown.
    #[inline]
    #[must_use]
    pub const fn new(tag: u64, icon: &'a str, label: &'a str) -> Self {
        Self { tag, icon, label }
    }
}

/// Visual presentation mode for [`AnchoredSelectPillSpec`] trigger buttons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectPillTriggerStyle {
    /// Segmented toolbar mode button with optional atlas icon, caption, width, and corner radii.
    Mode {
        /// Optional GPU texture atlas UV bounds `[u0, v0, u1, v1]`.
        icon_uv: Option<[f32; 4]>,
        /// Button width in logical pixels.
        width: f32,
        /// Per-corner rounding radii.
        corner_radii: CornerRadii,
    },
    /// Compact standalone dropdown trigger button with label and chevron (default 32px height).
    Dropdown {
        /// Button width in logical pixels.
        width: f32,
    },
    /// Compact standalone dropdown trigger button with explicit width and height.
    DropdownSized {
        /// Button width in logical pixels.
        width: f32,
        /// Button height in logical pixels.
        height: f32,
    },
}

/// Configuration descriptor for [`UiScope::anchored_select_pill`].
#[derive(Debug, Clone, Copy)]
pub struct AnchoredSelectPillSpec<'a> {
    /// Debug name assigned to the elevated dropdown popup node.
    pub popup_name: &'static str,
    /// Unique semantic tag identifying both the trigger button and active popup slot.
    pub tag: u64,
    /// Caption displayed on the trigger button.
    pub label: &'a str,
    /// Visual presentation variant of the trigger button.
    pub trigger_style: SelectPillTriggerStyle,
    /// Fixed width of the dropdown menu card in logical pixels.
    pub popup_width: f32,
    /// Vertical placement relative to the trigger button (`Above` or `Below`).
    pub placement: PopupPlacement,
    /// Ordered selectable options listed when opened.
    pub options: &'a [AnchoredSelectOption<'a>],
    /// Optional currently active option index marked with a trailing checkmark (`"✓"`).
    pub selected_index: Option<usize>,
}

impl<'a> UiScope<'a> {
    /// Emits a declarative trigger button bound to an auto-anchored dropdown popup in a single call.
    ///
    /// Features:
    /// - Resolves open/active visuals directly from [`irisui_core::UiTree::active_popup_tags`].
    /// - Toggles exclusive popup visibility on trigger click.
    /// - Positions the floating card beneath the trigger's real layout [`Rect`] resolved by
    ///   [`irisui_core::UiTree::resolve_anchor_rect`] with zero consumer coordinate math.
    /// - Autoclose option rows invoke `on_select(index)` immediately upon selection.
    pub fn anchored_select_pill<F>(
        &mut self,
        spec: AnchoredSelectPillSpec<'_>,
        mut on_select: F,
    ) -> WidgetResponse
    where
        F: FnMut(usize),
    {
        let is_open = self.is_popup_tag_active(spec.tag);
        let resp = match spec.trigger_style {
            SelectPillTriggerStyle::Mode {
                icon_uv,
                width,
                corner_radii,
            } => self.toolbar_mode_button(
                icon_uv,
                spec.label,
                spec.tag,
                is_open,
                width,
                corner_radii,
            ),
            SelectPillTriggerStyle::Dropdown { width } => {
                self.toolbar_dropdown_button(spec.label, spec.tag, is_open, width)
            }
            SelectPillTriggerStyle::DropdownSized { width, height } => self.add(
                crate::declarative::widget::ToolbarButton::new(spec.label)
                    .tag(spec.tag)
                    .width(width)
                    .height(height)
                    .open(is_open),
            ),
        };

        if resp.clicked() {
            self.toggle_exclusive_popup(spec.tag);
        }

        if self.tree.is_popup_tag_active(spec.tag) {
            let parent_bounds = self.parent_rect();
            let trigger_width = match spec.trigger_style {
                SelectPillTriggerStyle::Mode { width, .. }
                | SelectPillTriggerStyle::Dropdown { width }
                | SelectPillTriggerStyle::DropdownSized { width, .. } => width,
            };
            let trigger_fallback = Rect::new(
                parent_bounds.x,
                parent_bounds.y,
                trigger_width.max(40.0),
                32.0,
            );
            let anchor_rect = self
                .tree
                .resolve_anchor_rect(spec.tag)
                .unwrap_or(trigger_fallback);

            self.popup_anchored_styled(
                spec.popup_name,
                AnchoredPopupConfig::new(spec.tag, anchor_rect, spec.popup_width)
                    .with_placement(spec.placement),
                |menu| {
                    for (idx, opt) in spec.options.iter().enumerate() {
                        let shortcut = (spec.selected_index == Some(idx)).then_some("✓");
                        if menu
                            .dropdown_item_autoclose(opt.tag, opt.icon, opt.label, shortcut, true)
                            .clicked()
                        {
                            on_select(idx);
                        }
                    }
                },
            );
        }

        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::{MouseButton, Point, UiEvent, UiLayer, UiTree, WidgetRole};

    const PILL_TAG: u64 = 0xD101;
    const OPT_A_TAG: u64 = 0xD102;
    const OPT_B_TAG: u64 = 0xD103;
    const OPTIONS: &[AnchoredSelectOption<'_>] = &[
        AnchoredSelectOption::new(OPT_A_TAG, "🎥", "Perspective"),
        AnchoredSelectOption::new(OPT_B_TAG, "📐", "Orthographic"),
    ];

    #[test]
    fn test_anchored_select_pill_docked_layout_and_selection() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let panel_rect = Rect::new(260.0, 56.0, 800.0, 600.0);

        // 1. Frame 1: build pill inside docked panel and resolve layout
        let mut selected = 0usize;
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.panel("Viewport").bounds(panel_rect).show(|p| {
                p.anchored_select_pill(
                    AnchoredSelectPillSpec {
                        popup_name: "CameraModePopup",
                        tag: PILL_TAG,
                        label: "Perspective",
                        trigger_style: SelectPillTriggerStyle::Mode {
                            icon_uv: None,
                            width: 116.0,
                            corner_radii: CornerRadii::all(4.0),
                        },
                        popup_width: 150.0,
                        placement: PopupPlacement::Below,
                        options: OPTIONS,
                        selected_index: Some(selected),
                    },
                    |idx| selected = idx,
                );
            });
        }

        let trigger_rect = tree
            .resolve_anchor_rect(PILL_TAG)
            .expect("trigger rect cached");
        assert!(trigger_rect.x >= panel_rect.x);

        // 2. Frame 2: open popup and verify popup anchors beneath real trigger layout rect
        tree.open_popup(PILL_TAG);
        let _ = tree.clear_children(root);
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.panel("Viewport").bounds(panel_rect).show(|p| {
                p.anchored_select_pill(
                    AnchoredSelectPillSpec {
                        popup_name: "CameraModePopup",
                        tag: PILL_TAG,
                        label: "Perspective",
                        trigger_style: SelectPillTriggerStyle::Mode {
                            icon_uv: None,
                            width: 116.0,
                            corner_radii: CornerRadii::all(4.0),
                        },
                        popup_width: 150.0,
                        placement: PopupPlacement::Below,
                        options: OPTIONS,
                        selected_index: Some(selected),
                    },
                    |idx| selected = idx,
                );
            });
        }

        let popup = tree
            .iter()
            .map(|(_, n)| n)
            .find(|n| n.name.as_deref() == Some("CameraModePopup"))
            .expect("popup node must exist");
        assert_eq!(popup.layer, UiLayer::Popup);
        assert_eq!(popup.role, WidgetRole::DropdownPopup);
        assert!((popup.computed_rect.x - trigger_rect.x).abs() < 1.0);
        assert!((popup.computed_rect.y - (trigger_rect.bottom() + 2.0)).abs() < 1.0);

        // 3. Simulate click on second option (OPT_B_TAG) and verify autoclose + callback
        let opt_b_rect = tree
            .node_rect_tagged(OPT_B_TAG, Point::ZERO)
            .expect("opt B");
        let opt_b_click = Point::new(opt_b_rect.x + 8.0, opt_b_rect.y + 8.0);
        tree.dispatch_event(&UiEvent::MouseDown {
            point: opt_b_click,
            button: MouseButton::Left,
        });
        tree.dispatch_event(&UiEvent::MouseUp {
            point: opt_b_click,
            button: MouseButton::Left,
        });
        let _ = tree.clear_children(root);
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.panel("Viewport").bounds(panel_rect).show(|p| {
                p.anchored_select_pill(
                    AnchoredSelectPillSpec {
                        popup_name: "CameraModePopup",
                        tag: PILL_TAG,
                        label: "Perspective",
                        trigger_style: SelectPillTriggerStyle::Mode {
                            icon_uv: None,
                            width: 116.0,
                            corner_radii: CornerRadii::all(4.0),
                        },
                        popup_width: 150.0,
                        placement: PopupPlacement::Below,
                        options: OPTIONS,
                        selected_index: Some(selected),
                    },
                    |idx| selected = idx,
                );
            });
        }

        assert_eq!(selected, 1);
        assert!(!tree.is_popup_tag_active(PILL_TAG));
    }
}