// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # TreeCompositor & Layered Multi-Tree Render Pipeline
//!
//! Merges independent panel retained trees (`panel.tree`), floating window trees, and the
//! topmost [`OverlayTree`] into a unified [`DrawCommandList`] with exact back-to-front Z-order.
//!
//! ## Architectural Invariants
//!
//! 1. **(0, 0) Local Coordinates**: Panels construct and calculate layouts in their own `(0, 0)`
//!    local coordinate spaces (`Rect::new(0.0, 0.0, width, height)`).
//! 2. **Hardware Scissor Isolation**: Before drawing each panel, a hardware scissor rect
//!    is pushed to [`DrawCommandList`], strictly clamping rendering to `panel_rect`.
//! 3. **Local-to-Screen Translation**: All SDF quad, texture quad, and external texture quad
//!    bounds from `panel.tree` are offset by `+ panel_rect.min`.
//! 4. **Topmost Overlay Layer**: The [`OverlayTree`] is rendered at the absolute top Z-order
//!    without panel scissor clipping, ensuring dropdowns, modals, and popups are never cut off.
//! 5. **Zero Memory Allocation in Hot Loop**: Internal vector buffers are reused between frames.

use irisui::prelude::*;
use irisui::text::{TextCollectionOptions, TextSection, collect_text_sections_with_options};
use irisui::wgpu_backend::{DrawCommand, DrawCommandList};

/// Optional custom drawing callback used to inject specialized draw commands into panel trees.
pub type PanelCustomDrawer<'a> =
    Option<&'a mut dyn FnMut(&mut DrawCommandList, WidgetId, &WidgetNode, Option<Rect>)>;

/// Layered tree compositor combining independent panel trees, floating window trees,
/// and the top-level overlay tree into a unified WGPU draw command stream.
#[derive(Debug, Default)]
pub struct TreeCompositor {
    /// Combined draw command list compiled from all active layers in strict Z-order.
    pub command_list: DrawCommandList,
    /// Number of panels or sub-layers composited in the active frame.
    pub composited_layer_count: usize,
}

impl TreeCompositor {
    /// Creates a new, empty [`TreeCompositor`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears the internal command list and resets layer counters for reuse without heap reallocation.
    pub fn clear(&mut self) {
        self.command_list.clear();
        self.composited_layer_count = 0;
    }

    /// Appends the window chrome / shell commands (e.g. MenuBar, StatusBar, dock background, splitters)
    /// directly to the command stream.
    ///
    /// Shell commands are drawn at the base Z-layer without additional scissor restrictions.
    pub fn append_shell_commands(&mut self, shell_commands: DrawCommandList) {
        self.command_list.append(shell_commands);
    }

    /// Appends an independent panel's draw command list with hardware scissor clipping and
    /// local-to-screen coordinate offset.
    ///
    /// Translates every SDF quad, textured quad, and external quad by `+ panel_rect.min`
    /// (re-basing `(0, 0)` local coordinates to screen space) and clamps rendering to `panel_rect`.
    pub fn append_panel_commands(&mut self, mut panel_commands: DrawCommandList, panel_rect: Rect) {
        self.composited_layer_count += 1;

        let scissor_x = panel_rect.x.max(0.0).floor() as u32;
        let scissor_y = panel_rect.y.max(0.0).floor() as u32;
        let scissor_w = panel_rect.width.max(1.0).ceil() as u32;
        let scissor_h = panel_rect.height.max(1.0).ceil() as u32;

        // 1. Hardware scissor clamping to panel boundary
        self.command_list
            .push_scissor(scissor_x, scissor_y, scissor_w, scissor_h);

        let off_x = panel_rect.x;
        let off_y = panel_rect.y;

        // 2. Translate SDF quads by panel origin
        for quad in &mut panel_commands.quads {
            quad.rect[0] += off_x;
            quad.rect[1] += off_y;
            if quad.clip_rect != [0.0, 0.0, 0.0, 0.0] {
                quad.clip_rect[0] += off_x;
                quad.clip_rect[1] += off_y;
                quad.clip_rect[2] += off_x;
                quad.clip_rect[3] += off_y;
            }
        }

        // 3. Translate texture quads by panel origin
        for tex_quad in &mut panel_commands.texture_quads {
            tex_quad.rect[0] += off_x;
            tex_quad.rect[1] += off_y;
            if tex_quad.clip_rect != [0.0, 0.0, 0.0, 0.0] {
                tex_quad.clip_rect[0] += off_x;
                tex_quad.clip_rect[1] += off_y;
                tex_quad.clip_rect[2] += off_x;
                tex_quad.clip_rect[3] += off_y;
            }
        }

        // 4. Translate external texture quads by panel origin
        for ext_quad in &mut panel_commands.external_texture_quads {
            ext_quad.rect[0] += off_x;
            ext_quad.rect[1] += off_y;
            if ext_quad.clip_rect != [0.0, 0.0, 0.0, 0.0] {
                ext_quad.clip_rect[0] += off_x;
                ext_quad.clip_rect[1] += off_y;
                ext_quad.clip_rect[2] += off_x;
                ext_quad.clip_rect[3] += off_y;
            }
        }

        // 5. Offset internal scissors within panel bounds
        for cmd in &mut panel_commands.commands {
            match cmd {
                DrawCommand::SetScissor {
                    x,
                    y,
                    width,
                    height,
                } => {
                    *x = (*x + scissor_x).max(scissor_x);
                    *y = (*y + scissor_y).max(scissor_y);
                    *width = (*width).min(scissor_w);
                    *height = (*height).min(scissor_h);
                }
                DrawCommand::ResetScissor => {
                    *cmd = DrawCommand::SetScissor {
                        x: scissor_x,
                        y: scissor_y,
                        width: scissor_w,
                        height: scissor_h,
                    };
                }
                _ => {}
            }
        }

        // 6. Merge translated commands into main stream
        self.command_list.append(panel_commands);

        // 7. Reset scissor back to unconstrained for subsequent layers
        self.command_list.commands.push(DrawCommand::ResetScissor);
    }

    /// Compiles an independent panel's retained UI tree and appends it to the compositor stream.
    pub fn append_panel_tree(
        &mut self,
        tree: &UiTree,
        panel_rect: Rect,
        hovered_tag: Option<u64>,
        focused_tag: Option<u64>,
        blink_caret: bool,
    ) {
        self.append_panel_tree_with_drawer(
            tree,
            panel_rect,
            hovered_tag,
            focused_tag,
            blink_caret,
            None,
        );
    }

    /// Compiles an independent panel's retained UI tree with an optional custom node drawer.
    ///
    /// Allows custom rendering extensions (such as oscilloscope polylines or preview wireframes)
    /// to  inject draw commands into panel-local coordinates with hardware scissor support.
    pub fn append_panel_tree_with_drawer(
        &mut self,
        tree: &UiTree,
        panel_rect: Rect,
        hovered_tag: Option<u64>,
        focused_tag: Option<u64>,
        blink_caret: bool,
        mut custom_drawer: PanelCustomDrawer<'_>,
    ) {
        let Some(root) = tree.root() else {
            return;
        };

        let mut panel_commands = DrawCommandList::new();
        let mut options = TreeCompilerOptions::new()
            .with_hovered_tag(hovered_tag)
            .with_focused_tag(focused_tag)
            .with_blink_caret(blink_caret);

        if let Some(ref mut drawer) = custom_drawer {
            options = options.with_custom_drawer(&mut **drawer);
        }

        compile_tree_draw_commands_into(tree, root, &mut options, &mut panel_commands);
        self.append_panel_commands(panel_commands, panel_rect);
    }

    /// Appends the top-level overlay tree (dropdowns, context menus, modals, popups) at the absolute top Z-order.
    ///
    /// Overlay commands are rendered without scissor clipping, guaranteeing that popups and menus
    /// are never clipped by dock panel or window boundaries.
    pub fn append_overlay_commands(&mut self, overlay_commands: DrawCommandList) {
        self.command_list.commands.push(DrawCommand::ResetScissor);
        self.command_list.append(overlay_commands);
    }

    /// Compiles the top-level overlay tree and appends it to the compositor stream at topmost Z-order.
    pub fn append_overlay_tree(
        &mut self,
        overlay_tree: &UiTree,
        hovered_tag: Option<u64>,
        focused_tag: Option<u64>,
        blink_caret: bool,
    ) {
        let Some(root) = overlay_tree.root() else {
            return;
        };

        let mut overlay_commands = DrawCommandList::new();
        let mut options = TreeCompilerOptions::new()
            .with_hovered_tag(hovered_tag)
            .with_focused_tag(focused_tag)
            .with_blink_caret(blink_caret);

        compile_tree_draw_commands_into(overlay_tree, root, &mut options, &mut overlay_commands);
        self.append_overlay_commands(overlay_commands);
    }

    /// Filters out or scissoring-clips text sections that intersect with active overlay occluders (such as open menus or modal dialogs).
    ///
    /// If a text section begins horizontally inside the occluder or is completely enveloped vertically, it is discarded.
    /// If it begins to the left of the occluder and protrudes into it, its clip boundary is clamped at the occluder's left edge.
    pub fn occlude_text_sections<'a>(sections: &mut Vec<TextSection<'a>>, occluders: &[Rect]) {
        if occluders.is_empty() {
            return;
        }

        sections.retain_mut(|sec| {
            for occ in occluders {
                let intersects = sec.bounds.x < occ.right()
                    && sec.bounds.right() > occ.x
                    && sec.bounds.y < occ.bottom()
                    && sec.bounds.bottom() > occ.y;

                if !intersects {
                    continue;
                }

                if sec.bounds.x >= occ.x {
                    return false;
                }

                if let Some(ref mut clip) = sec.clip_bounds {
                    let max_width = (occ.x - clip.x).max(0.0);
                    clip.width = clip.width.min(max_width);
                    if clip.width <= 0.0 {
                        return false;
                    }
                } else {
                    let w = (occ.x - sec.bounds.x).max(0.0);
                    if w <= 0.0 {
                        return false;
                    }
                    sec.clip_bounds =
                        Some(Rect::new(sec.bounds.x, sec.bounds.y, w, sec.bounds.height));
                }
            }
            true
        });
    }

    /// Collects and translates text sections from an independent panel tree into screen space coordinates.
    pub fn collect_panel_text_sections<'a>(
        tree: &'a UiTree,
        panel_rect: Rect,
        options: &TextCollectionOptions,
    ) -> Vec<TextSection<'a>> {
        // Collect text in panel-local coordinates without screen-space occluders to prevent false occlusion
        let mut local_options = options.clone();
        local_options.extra_occluders.clear();
        let mut sections = collect_text_sections_with_options(tree, &local_options);
        for sec in &mut sections {
            sec.bounds.x += panel_rect.x;
            sec.bounds.y += panel_rect.y;
            let panel_clip = panel_rect;
            sec.clip_bounds = Some(match sec.clip_bounds {
                Some(clip) => {
                    let shifted = Rect::new(
                        clip.x + panel_rect.x,
                        clip.y + panel_rect.y,
                        clip.width,
                        clip.height,
                    );
                    shifted.intersect(panel_clip)
                }
                None => panel_clip,
            });
        }

        // Apply screen-space extra occluders only after text bounds have been translated to screen space
        if !options.extra_occluders.is_empty() {
            Self::occlude_text_sections(&mut sections, &options.extra_occluders);
        }

        sections
    }

    /// Collects composited text sections across all active UI trees: window chrome, docked/floating panels,
    /// and top-level overlay in strict Z-order.
    pub fn collect_composited_text_sections<'a>(
        shell_tree: &'a UiTree,
        docked_panels: &[(&'a UiTree, Rect)],
        overlay_tree: &'a UiTree,
        options: &TextCollectionOptions,
    ) -> Vec<TextSection<'a>> {
        let mut all_sections = Vec::new();

        // 1. Shell typography (MenuBar, StatusBar, dock chrome)
        if shell_tree.root().is_some() {
            let mut shell_sections = collect_text_sections_with_options(shell_tree, options);
            if !options.extra_occluders.is_empty() {
                Self::occlude_text_sections(&mut shell_sections, &options.extra_occluders);
            }
            all_sections.extend(shell_sections);
        }

        // 2. Docked & Floating panel typography with local-to-screen shift and scissor clamp
        for &(panel_tree, panel_rect) in docked_panels {
            if panel_tree.root().is_some() {
                let panel_sections =
                    Self::collect_panel_text_sections(panel_tree, panel_rect, options);
                all_sections.extend(panel_sections);
            }
        }

        // 3. Top-level overlay typography (dropdowns, popups, modals)
        // Overlays reside at topmost Z-order; clear extra_occluders so their own bounds never occlude their content
        if overlay_tree.root().is_some() {
            let mut overlay_options = options.clone();
            overlay_options.extra_occluders.clear();
            let overlay_sections =
                collect_text_sections_with_options(overlay_tree, &overlay_options);
            all_sections.extend(overlay_sections);
        }

        all_sections
    }
}

/// Translates a screen-space coordinate into a panel's local `(0, 0)` coordinate space.
#[inline]
pub fn screen_to_panel_local(screen_point: Point, panel_rect: Rect) -> Point {
    Point::new(screen_point.x - panel_rect.x, screen_point.y - panel_rect.y)
}

/// Checks if a screen-space coordinate falls within a panel's bounding rectangle.
#[inline]
pub fn is_point_in_panel(screen_point: Point, panel_rect: Rect) -> bool {
    screen_point.x >= panel_rect.x
        && screen_point.x <= panel_rect.right()
        && screen_point.y >= panel_rect.y
        && screen_point.y <= panel_rect.bottom()
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui::wgpu_backend::QuadInstance;

    #[test]
    fn test_tree_compositor_initial_state() {
        let compositor = TreeCompositor::new();
        assert_eq!(compositor.composited_layer_count, 0);
        assert!(compositor.command_list.commands.is_empty());
        assert!(compositor.command_list.quads.is_empty());
    }

    #[test]
    fn test_tree_compositor_append_panel_commands_translates_quads() {
        let mut compositor = TreeCompositor::new();
        let mut panel_commands = DrawCommandList::new();

        // Local quad at (10, 20, 100, 50)
        let quad = QuadInstance {
            rect: [10.0, 20.0, 100.0, 50.0],
            ..Default::default()
        };
        panel_commands.push_quad(quad);

        let panel_rect = Rect::new(200.0, 300.0, 400.0, 500.0);
        compositor.append_panel_commands(panel_commands, panel_rect);

        assert_eq!(compositor.composited_layer_count, 1);
        assert_eq!(compositor.command_list.quads.len(), 1);

        // Quad rect must be translated by +200.0, +300.0
        let translated = compositor.command_list.quads[0].rect;
        assert_eq!(translated[0], 210.0);
        assert_eq!(translated[1], 320.0);
        assert_eq!(translated[2], 100.0);
        assert_eq!(translated[3], 50.0);

        // Commands must contain: SetScissor (panel bounds), DrawSdfQuads, ResetScissor
        assert!(compositor.command_list.commands.len() >= 3);
        assert_eq!(
            compositor.command_list.commands[0],
            DrawCommand::SetScissor {
                x: 200,
                y: 300,
                width: 400,
                height: 500
            }
        );
        assert_eq!(
            compositor.command_list.commands.last(),
            Some(&DrawCommand::ResetScissor)
        );
    }

    #[test]
    fn test_tree_compositor_append_overlay_commands_at_top() {
        let mut compositor = TreeCompositor::new();

        let mut panel_commands = DrawCommandList::new();
        let quad_a = QuadInstance {
            rect: [0.0, 0.0, 50.0, 50.0],
            ..Default::default()
        };
        panel_commands.push_quad(quad_a);
        compositor.append_panel_commands(panel_commands, Rect::new(50.0, 50.0, 100.0, 100.0));

        let mut overlay_commands = DrawCommandList::new();
        let quad_b = QuadInstance {
            rect: [10.0, 10.0, 200.0, 150.0],
            ..Default::default()
        };
        overlay_commands.push_quad(quad_b);
        compositor.append_overlay_commands(overlay_commands);

        assert_eq!(compositor.command_list.quads.len(), 2);
        // Overlay quad must preserve its screen coordinates without scissor clamping
        assert_eq!(compositor.command_list.quads[1].rect[0], 10.0);
        assert_eq!(compositor.command_list.quads[1].rect[1], 10.0);
    }

    #[test]
    fn test_coordinate_transformation_helpers() {
        let panel_rect = Rect::new(150.0, 250.0, 300.0, 400.0);
        let screen_point = Point::new(200.0, 300.0);

        assert!(is_point_in_panel(screen_point, panel_rect));
        let local = screen_to_panel_local(screen_point, panel_rect);
        assert_eq!(local.x, 50.0);
        assert_eq!(local.y, 50.0);

        let outside_point = Point::new(100.0, 100.0);
        assert!(!is_point_in_panel(outside_point, panel_rect));
    }

    #[test]
    fn test_tree_compositor_three_layer_z_order_hierarchy() {
        let mut compositor = TreeCompositor::new();

        // Layer 0: Shell / 3D Viewport canvas command
        let mut shell_commands = DrawCommandList::new();
        shell_commands.push_quad(QuadInstance {
            rect: [0.0, 0.0, 1920.0, 1080.0],
            ..Default::default()
        });
        compositor.append_shell_commands(shell_commands);

        // Layer 1: Dock Panel with scissor isolation & translation
        let mut panel_commands = DrawCommandList::new();
        panel_commands.push_quad(QuadInstance {
            rect: [0.0, 0.0, 300.0, 400.0],
            ..Default::default()
        });
        compositor.append_panel_commands(panel_commands, Rect::new(100.0, 50.0, 300.0, 400.0));

        // Layer 2: OverlayTree (Modal / Dropdown)
        let mut overlay_commands = DrawCommandList::new();
        overlay_commands.push_quad(QuadInstance {
            rect: [200.0, 150.0, 500.0, 300.0],
            ..Default::default()
        });
        compositor.append_overlay_commands(overlay_commands);

        // Assert Z-order: 3 quads in exact Layer 0 -> Layer 1 -> Layer 2 order
        assert_eq!(compositor.command_list.quads.len(), 3);
        assert_eq!(
            compositor.command_list.quads[0].rect,
            [0.0, 0.0, 1920.0, 1080.0]
        );
        assert_eq!(
            compositor.command_list.quads[1].rect,
            [100.0, 50.0, 300.0, 400.0]
        );
        assert_eq!(
            compositor.command_list.quads[2].rect,
            [200.0, 150.0, 500.0, 300.0]
        );
    }

    #[test]
    fn test_occlude_text_sections_filters_intersecting_bounds() {
        let occluder = Rect::new(100.0, 50.0, 150.0, 200.0);

        // Section A: completely inside occluder -> must be discarded
        let sec_a = TextSection::new("Inside", Rect::new(120.0, 80.0, 80.0, 20.0));

        // Section B: completely outside occluder -> must be preserved
        let sec_b = TextSection::new("Outside", Rect::new(10.0, 10.0, 50.0, 20.0));

        // Section C: begins to the left of occluder and overlaps into it -> must be clipped at occ.x
        let sec_c = TextSection::new("Overlapping Left", Rect::new(60.0, 100.0, 100.0, 20.0));

        let mut sections = vec![sec_a, sec_b, sec_c];
        TreeCompositor::occlude_text_sections(&mut sections, &[occluder]);

        assert_eq!(sections.len(), 2, "Inside section must be discarded");
        assert_eq!(sections[0].text, "Outside");
        assert_eq!(sections[1].text, "Overlapping Left");
        assert_eq!(
            sections[1].clip_bounds,
            Some(Rect::new(60.0, 100.0, 40.0, 20.0)),
            "Overlapping section must be scissored at occluder.x"
        );
    }
}