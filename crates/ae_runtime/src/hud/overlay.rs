// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Shared Iris UI gameplay HUD and pause menu overlay builder and standalone WGPU renderer.
//!
//! Serves as the single source of truth for rendering active ECS HUD components
//! (`UiPanel`, `UiText`, `UiProgressBar`, `UiButton`, `UiImage`) and the centered
//! In-Game Pause Menu overlay in both the editor viewport and standalone `ae_runtime` exports.

use ae_renderer::render::{IrisRenderPassParams, iris_render_pass};
use irisui_core::{
    AlignItems, Color, Insets, JustifyContent, MouseButton, Point, Rect, Size, Style, TextAlign,
    UiEvent, UiTree,
};
use irisui_layout::LayoutEngine;
use irisui_text::{
    TextCollectionOptions, TextRenderer, TextSystem, collect_text_sections_with_options,
};
use irisui_wgpu::{
    DrawCommandList, IrisRenderer, TreeCompilerOptions, compile_tree_draw_commands_into,
};
use irisui_widgets::{Button, UiScope};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

/// Semantic tag assigned to the In-Game Pause Menu Resume button.
pub const TAG_PLAY_RESUME: u64 = 2040;

/// Semantic tag assigned to the In-Game Pause Menu Exit button.
pub const TAG_PLAY_EXIT: u64 = 2041;

/// Action returned by [`RuntimeHudOverlay::prepare_and_poll_action`] when a pause menu button is clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePauseAction {
    /// No pause menu button was clicked this frame.
    None,
    /// User clicked the Resume button.
    Resume,
    /// User clicked the Exit button.
    Exit,
}

/// Builds active in-game ECS UI components (health bars, score labels, panels) into declarative [`UiScope`] nodes.
pub fn build_ingame_ecs_ui(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    viewport_rect: Rect,
    cursor_pos: Point,
) {
    let screen_w = viewport_rect.width;
    let screen_h = viewport_rect.height;
    if screen_w <= 10.0 || screen_h <= 10.0 {
        return;
    }

    let mouse_pos = Some([
        cursor_pos.x - viewport_rect.x,
        cursor_pos.y - viewport_rect.y,
    ]);

    let draw_commands = ae_core::ui::UiLayoutResolver::resolve_draw_commands(
        world, screen_w, screen_h, mouse_pos, false,
    );

    if draw_commands.is_empty() {
        return;
    }

    for cmd in &draw_commands {
        match cmd {
            ae_core::ui::UiDrawCommand::Rect {
                rect,
                fill_color,
                border_color,
                border_width,
                border_radius,
                ..
            } => {
                let mut style = Style::new()
                    .position_absolute()
                    .left(rect.min_x)
                    .top(rect.min_y)
                    .width(rect.width())
                    .height(rect.height())
                    .background(Color::rgba(
                        fill_color[0],
                        fill_color[1],
                        fill_color[2],
                        fill_color[3],
                    ));
                if border_color[3] > 0.01 && *border_width > 0.0 {
                    style = style.border(
                        *border_width,
                        Color::rgba(
                            border_color[0],
                            border_color[1],
                            border_color[2],
                            border_color[3],
                        ),
                    );
                }
                if *border_radius > 0.0 {
                    style = style.border_radius(*border_radius);
                }
                scope.empty_box(style);
            }
            ae_core::ui::UiDrawCommand::Text {
                pos,
                text,
                font_size,
                color,
                alignment,
                ..
            } => {
                let approx_w = (text.len() as f32) * (*font_size * 0.65);
                let approx_h = *font_size * 1.3;
                let text_x = match alignment {
                    ae_core::ui::UiTextAlignment::Left => pos[0],
                    ae_core::ui::UiTextAlignment::Center => pos[0] - approx_w * 0.5,
                    ae_core::ui::UiTextAlignment::Right => pos[0] - approx_w,
                };
                let text_y = pos[1] - approx_h * 0.5;
                let iris_align = match alignment {
                    ae_core::ui::UiTextAlignment::Left => TextAlign::Left,
                    ae_core::ui::UiTextAlignment::Center => TextAlign::Center,
                    ae_core::ui::UiTextAlignment::Right => TextAlign::Right,
                };
                let style = Style::new()
                    .position_absolute()
                    .left(text_x)
                    .top(text_y)
                    .width(approx_w)
                    .height(approx_h);
                scope.container(style, |tc| {
                    tc.label_custom(
                        text,
                        *font_size,
                        Color::rgba(color[0], color[1], color[2], color[3]),
                        iris_align,
                    );
                });
            }
            ae_core::ui::UiDrawCommand::Image { rect, tint, .. } => {
                let style = Style::new()
                    .position_absolute()
                    .left(rect.min_x)
                    .top(rect.min_y)
                    .width(rect.width())
                    .height(rect.height())
                    .background(Color::rgba(tint[0], tint[1], tint[2], tint[3]));
                scope.empty_box(style);
            }
        }
    }
}

/// Builds the centered, interactive In-Game Pause Menu overlay when gameplay is paused.
///
/// Button click events are dispatched via semantic tags ([`TAG_PLAY_RESUME`], [`TAG_PLAY_EXIT`]).
pub fn build_in_game_pause_menu(scope: &mut UiScope<'_>, viewport_rect: Rect, exit_label: &str) {
    let card_w = 320.0;
    let card_h = 220.0;

    let backdrop_style = Style::new()
        .position_absolute()
        .left(0.0)
        .top(0.0)
        .width(viewport_rect.width)
        .height(viewport_rect.height)
        .background(Color::rgba(0.04, 0.05, 0.08, 0.70));

    let card_rect = Rect {
        x: (viewport_rect.width - card_w) * 0.5,
        y: (viewport_rect.height - card_h) * 0.5,
        width: card_w,
        height: card_h,
    };

    scope.container_named("InGamePauseBackdrop", backdrop_style, |card_scope| {
        build_pause_card(card_scope, card_rect, exit_label);
    });
}

fn build_pause_card(scope: &mut UiScope<'_>, card_rect: Rect, exit_label: &str) {
    let card_style = Style::new()
        .position_absolute()
        .left(card_rect.x)
        .top(card_rect.y)
        .flex_col()
        .align_items(AlignItems::Center)
        .padding_insets(Insets::new(24.0, 20.0, 20.0, 20.0))
        .gap(10.0)
        .width(card_rect.width)
        .height(card_rect.height)
        .background(Color::rgba(0.09, 0.11, 0.15, 0.96))
        .border(1.5, Color::rgba(0.0, 0.80, 1.0, 0.60))
        .border_radius(8.0)
        .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.80));

    scope.container_named("InGamePauseCard", card_style, |card| {
        card.container_named("InGamePauseTitle", Style::new(), |title| {
            title.label_custom(
                "GAME PAUSED",
                20.0,
                Color::rgba(1.0, 1.0, 1.0, 1.0),
                TextAlign::Center,
            );
        });

        card.label_custom(
            "Press P or choose an option below",
            11.0,
            Color::rgba(0.60, 0.68, 0.78, 1.0),
            TextAlign::Center,
        );

        let btn_group_style = Style::new()
            .flex_col()
            .align_items(AlignItems::Center)
            .gap(12.0)
            .width(230.0);

        card.container(btn_group_style, |btns| {
            let btn_style = Style::new()
                .width(180.0)
                .height(26.0)
                .padding_insets(Insets::new(5.0, 14.0, 5.0, 14.0))
                .background(Color::rgba(0.0, 0.30, 0.42, 0.60))
                .border(1.0, Color::rgba(0.0, 0.75, 0.95, 0.60))
                .border_radius(4.0)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);

            btns.add(
                Button::new("▶  Resume Game")
                    .name("InGamePauseResumeButton")
                    .style(btn_style)
                    .tag(TAG_PLAY_RESUME),
            );
            btns.add(
                Button::new(exit_label)
                    .name("InGamePauseExitButton")
                    .style(btn_style)
                    .tag(TAG_PLAY_EXIT),
            );
        });
    });
}

/// Standalone Iris UI overlay renderer for [`crate::RuntimeSession`].
///
/// Renders ECS HUD components and the interactive Pause Menu directly on top of the
/// standalone swapchain surface without depending on `ae_editor` or `ae_editor_ui`.
pub struct RuntimeHudOverlay {
    tree: UiTree,
    layout_engine: LayoutEngine,
    renderer: IrisRenderer,
    text_system: TextSystem,
    text_renderer: Option<TextRenderer>,
    command_list: DrawCommandList,
    cursor_pos: Point,
    target_format: wgpu::TextureFormat,
    physical_size: (u32, u32),
    scale_factor: f32,
}

impl RuntimeHudOverlay {
    /// Creates a new standalone HUD overlay renderer bound to the swapchain surface format.
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        physical_size: (u32, u32),
        scale_factor: f32,
    ) -> Self {
        Self {
            tree: UiTree::new(),
            layout_engine: LayoutEngine::new(),
            renderer: IrisRenderer::new(device, target_format),
            text_system: TextSystem::new(),
            text_renderer: None,
            command_list: DrawCommandList::default(),
            cursor_pos: Point::new(0.0, 0.0),
            target_format,
            physical_size,
            scale_factor: if scale_factor.is_finite() && scale_factor > 0.1 {
                scale_factor
            } else {
                1.0
            },
        }
    }

    /// Updates target surface dimensions and DPI scale factor after window resize.
    pub fn resize(&mut self, physical_size: (u32, u32), scale_factor: f32) {
        self.physical_size = physical_size;
        if scale_factor.is_finite() && scale_factor > 0.1 {
            self.scale_factor = scale_factor;
        }
    }

    /// Forwards pointer motion and left-click events to the runtime HUD [`UiTree`].
    pub fn handle_window_event(&mut self, event: &WindowEvent, is_paused: bool) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = Point::new(
                    position.x as f32 / self.scale_factor,
                    position.y as f32 / self.scale_factor,
                );
                if is_paused {
                    let _ = self.tree.dispatch_event(&UiEvent::MouseMove {
                        point: self.cursor_pos,
                    });
                }
            }
            WindowEvent::MouseInput {
                state,
                button: WinitMouseButton::Left,
                ..
            } if is_paused => {
                let ui_event = match state {
                    ElementState::Pressed => UiEvent::MouseDown {
                        button: MouseButton::Left,
                        point: self.cursor_pos,
                    },
                    ElementState::Released => UiEvent::MouseUp {
                        button: MouseButton::Left,
                        point: self.cursor_pos,
                    },
                };
                let _ = self.tree.dispatch_event(&ui_event);
            }
            _ => {}
        }
    }

    /// Polls any Pause Menu button click from the last event dispatch, rebuilds the HUD tree,
    /// computes Taffy layout, and uploads text/quad GPU buffers.
    pub fn prepare_and_poll_action(
        &mut self,
        world: &hecs::World,
        is_paused: bool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> RuntimePauseAction {
        let action = if is_paused {
            if self.tree.event_state.frame_clicked_tag == Some(TAG_PLAY_RESUME) {
                RuntimePauseAction::Resume
            } else if self.tree.event_state.frame_clicked_tag == Some(TAG_PLAY_EXIT) {
                RuntimePauseAction::Exit
            } else {
                RuntimePauseAction::None
            }
        } else {
            RuntimePauseAction::None
        };

        self.tree.clear_frame_clicks();
        self.tree.clear();
        self.command_list.clear();

        let logical_w = (self.physical_size.0 as f32 / self.scale_factor)
            .round()
            .max(1.0);
        let logical_h = (self.physical_size.1 as f32 / self.scale_factor)
            .round()
            .max(1.0);

        if let Ok(root) = self.tree.create_root() {
            let viewport_rect = Rect::new(0.0, 0.0, logical_w, logical_h);
            let mut scope = UiScope::new(&mut self.tree, root);
            build_ingame_ecs_ui(&mut scope, world, viewport_rect, self.cursor_pos);
            if is_paused {
                build_in_game_pause_menu(&mut scope, viewport_rect, "⏹  Exit Game");
            }

            let _ = self
                .layout_engine
                .compute_layout(&mut self.tree, Size::new(logical_w, logical_h));

            let mut options = TreeCompilerOptions::new()
                .with_clip(Some(viewport_rect))
                .with_hovered_id(self.tree.event_state.hovered);
            compile_tree_draw_commands_into(&self.tree, root, &mut options, &mut self.command_list);

            if self.text_renderer.is_none() {
                self.text_renderer = Some(TextRenderer::new(device, queue, self.target_format));
            }
            let text_options = TextCollectionOptions {
                hovered_id: self.tree.event_state.hovered,
                ..Default::default()
            };
            let sections = collect_text_sections_with_options(&self.tree, &text_options);
            if let Some(txt_renderer) = &mut self.text_renderer {
                txt_renderer.prepare(
                    device,
                    queue,
                    &mut self.text_system,
                    self.physical_size,
                    self.scale_factor,
                    &sections,
                );
            }
        }

        action
    }

    /// Encodes the Iris UI HUD and Pause Menu pass onto `target_view`.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
    ) {
        if self.command_list.quads.is_empty() && self.command_list.texture_quads.is_empty() {
            return;
        }
        let logical_screen_size = (
            (self.physical_size.0 as f32 / self.scale_factor)
                .round()
                .max(1.0) as u32,
            (self.physical_size.1 as f32 / self.scale_factor)
                .round()
                .max(1.0) as u32,
        );
        iris_render_pass(IrisRenderPassParams {
            device,
            queue,
            encoder,
            target_view,
            renderer: &mut self.renderer,
            command_list: &self.command_list,
            text_renderer: self.text_renderer.as_ref(),
            volatile_text_renderer: None,
            screen_size: logical_screen_size,
            dirty_boundaries: None,
        });
        self.tree.clear_dirty_boundaries();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::ui::{UiAnchor, UiElement, UiPanel, UiText, UiTextAlignment};
    use irisui_core::WidgetId;

    fn collect_text_values(tree: &UiTree, current: WidgetId, out: &mut Vec<String>) {
        if let Some(node) = tree.get(current) {
            if let Some(t) = &node.text {
                out.push(t.clone());
            }
            for &child in &node.children {
                collect_text_values(tree, child, out);
            }
        }
    }

    #[test]
    fn test_shared_ingame_ecs_ui_and_pause_menu_build_nodes() {
        let mut world = hecs::World::new();
        world.spawn((
            UiElement::new(UiAnchor::TopLeft, [16.0, 16.0], [200.0, 48.0]),
            UiPanel::default(),
            UiText {
                text: "HP: 100 | Score: 250".to_string(),
                font_size: 16.0,
                color: [1.0, 1.0, 1.0, 1.0],
                alignment: UiTextAlignment::Left,
                ..Default::default()
            },
        ));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let vp = Rect::new(0.0, 0.0, 1280.0, 720.0);
        {
            let mut scope = UiScope::new(&mut tree, root);
            build_ingame_ecs_ui(&mut scope, &world, vp, Point::new(0.0, 0.0));
            build_in_game_pause_menu(&mut scope, vp, "⏹  Exit Game");
        }

        let mut texts = Vec::new();
        collect_text_values(&tree, root, &mut texts);
        assert!(texts.iter().any(|t| t == "HP: 100 | Score: 250"));
        assert!(texts.iter().any(|t| t == "GAME PAUSED"));
        assert!(tree.find_node_by_tag(TAG_PLAY_RESUME).is_some());
        assert!(tree.find_node_by_tag(TAG_PLAY_EXIT).is_some());
    }
}