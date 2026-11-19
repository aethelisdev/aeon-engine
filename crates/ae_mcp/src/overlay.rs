// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU visual feedback overlay drawn on top of the host surface during live inspection.

use irisui::core::{Color, MouseButton, Point, Rect};
use irisui::wgpu_backend::{DrawCommandList, IrisRenderer, QuadInstance};
use std::time::Instant;

const CURSOR_IDLE_LIFETIME_SECS: f32 = 6.0;
const RIPPLE_DURATION_SECS: f32 = 0.5;
const DRAG_TRAIL_LINGER_SECS: f32 = 0.65;

/// Classifies the most recent MCP-injected action for visual feedback styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectActionKind {
    /// Cursor hover or motion update.
    Move,
    /// Primary left mouse button click.
    LeftClick,
    /// Secondary right mouse button click.
    RightClick,
    /// Middle mouse button click.
    MiddleClick,
    /// Double-click gesture.
    DoubleClick,
    /// Active or completed pointer drag gesture.
    Drag,
    /// Mouse wheel scroll injection.
    Scroll,
    /// Keyboard shortcut or key event injection.
    Key,
    /// Text typing injection.
    TypeText,
}

impl InspectActionKind {
    /// Returns the visual ring accent color associated with this action category.
    #[must_use]
    pub const fn accent_color(self) -> Color {
        match self {
            Self::Move => Color::rgba(0.15, 0.85, 1.0, 1.0),
            Self::LeftClick => Color::rgba(0.15, 0.95, 0.62, 1.0),
            Self::RightClick => Color::rgba(1.0, 0.68, 0.12, 1.0),
            Self::MiddleClick | Self::DoubleClick => Color::rgba(0.25, 0.92, 0.95, 1.0),
            Self::Drag => Color::rgba(0.76, 0.46, 1.0, 1.0),
            Self::Scroll => Color::rgba(0.35, 0.72, 1.0, 1.0),
            Self::Key | Self::TypeText => Color::rgba(0.98, 0.45, 0.68, 1.0),
        }
    }

    /// Maps a [`MouseButton`] and `double_click` flag to the corresponding visual category.
    #[must_use]
    pub const fn from_click(button: MouseButton, double_click: bool) -> Self {
        if double_click {
            return Self::DoubleClick;
        }
        match button {
            MouseButton::Left => Self::LeftClick,
            MouseButton::Right => Self::RightClick,
            MouseButton::Middle | MouseButton::Other(_) => Self::MiddleClick,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RippleEvent {
    point: Point,
    kind: InspectActionKind,
    started_at: Instant,
}

/// Renders non-intrusive visual feedback (agent cursor crosshair, click ripples, drag trail)
/// directly onto the host framebuffer surface while live inspection is active.
///
/// Reuses internal SDF command buffers with zero allocations in steady-state frames.
pub struct InspectVisualOverlay {
    renderer: IrisRenderer,
    cmd_list: DrawCommandList,
    cursor_pos: Option<Point>,
    cursor_kind: InspectActionKind,
    last_activity_at: Option<Instant>,
    ripple: Option<RippleEvent>,
    drag_segment: Option<(Point, Point, Instant, bool)>,
}

impl InspectVisualOverlay {
    /// Creates a visual inspection overlay GPU renderer targeting `target_format`.
    #[must_use]
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        Self {
            renderer: IrisRenderer::new(device, target_format),
            cmd_list: DrawCommandList::new(),
            cursor_pos: None,
            cursor_kind: InspectActionKind::Move,
            last_activity_at: None,
            ripple: None,
            drag_segment: None,
        }
    }

    /// Records a pointer move or hover target position.
    pub fn record_move(&mut self, point: Point) {
        self.cursor_pos = Some(point);
        self.cursor_kind = InspectActionKind::Move;
        self.last_activity_at = Some(Instant::now());
    }

    /// Records a mouse click and starts an expanding ripple animation at `point`.
    pub fn record_click(&mut self, point: Point, button: MouseButton, double_click: bool) {
        let now = Instant::now();
        let kind = InspectActionKind::from_click(button, double_click);
        self.cursor_pos = Some(point);
        self.cursor_kind = kind;
        self.last_activity_at = Some(now);
        self.ripple = Some(RippleEvent {
            point,
            kind,
            started_at: now,
        });
    }

    /// Records a wheel scroll injection and emits a ripple indicator at `point`.
    pub fn record_scroll(&mut self, point: Point) {
        let now = Instant::now();
        self.cursor_pos = Some(point);
        self.cursor_kind = InspectActionKind::Scroll;
        self.last_activity_at = Some(now);
        self.ripple = Some(RippleEvent {
            point,
            kind: InspectActionKind::Scroll,
            started_at: now,
        });
    }

    /// Records an active frame of a pointer drag gesture from `from` to `current`.
    pub fn record_drag_step(&mut self, from: Point, current: Point) {
        let now = Instant::now();
        self.cursor_pos = Some(current);
        self.cursor_kind = InspectActionKind::Drag;
        self.last_activity_at = Some(now);
        self.drag_segment = Some((from, current, now, true));
    }

    /// Marks completion of a pointer drag gesture from `from` to `to` with an arrival ripple.
    pub fn record_drag_end(&mut self, from: Point, to: Point) {
        let now = Instant::now();
        self.cursor_pos = Some(to);
        self.cursor_kind = InspectActionKind::Drag;
        self.last_activity_at = Some(now);
        self.drag_segment = Some((from, to, now, false));
        self.ripple = Some(RippleEvent {
            point: to,
            kind: InspectActionKind::Drag,
            started_at: now,
        });
    }

    /// Records a keyboard or typed-text action without displacing the cursor marker.
    pub fn record_key_activity(&mut self, kind: InspectActionKind) {
        self.cursor_kind = kind;
        self.last_activity_at = Some(Instant::now());
    }

    /// Returns `true` while a click ripple or drag trajectory trail is still animating.
    #[must_use]
    pub fn has_active_animation(&self) -> bool {
        let now = Instant::now();
        if let Some(r) = self.ripple
            && now.duration_since(r.started_at).as_secs_f32() < RIPPLE_DURATION_SECS
        {
            return true;
        }
        if let Some((_, _, at, active)) = self.drag_segment
            && (active || now.duration_since(at).as_secs_f32() < DRAG_TRAIL_LINGER_SECS)
        {
            return true;
        }
        false
    }

    /// Renders active agent cursor markers, click ripples, and drag trails onto `target_view`.
    pub fn render_overlay(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        physical_size: (u32, u32),
        zoom: f32,
    ) {
        let now = Instant::now();
        let safe_zoom = if zoom.is_finite() && zoom > 0.1 {
            zoom
        } else {
            1.0
        };
        let logical_size = [
            (physical_size.0 as f32 / safe_zoom).round().max(1.0),
            (physical_size.1 as f32 / safe_zoom).round().max(1.0),
        ];

        self.cmd_list.clear();
        self.build_drag_trail_quads(now);
        self.build_ripple_quads(now);
        self.build_cursor_quads(now);
        self.build_status_beacon_quad(now, logical_size);

        if self.cmd_list.quads.is_empty() {
            return;
        }

        self.renderer
            .prepare_command_list(device, queue, logical_size, &self.cmd_list);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Aeon MCP Visual Overlay Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        self.renderer.render_command_list(
            &mut pass,
            &self.cmd_list,
            (logical_size[0] as u32, logical_size[1] as u32),
        );
    }

    fn build_drag_trail_quads(&mut self, now: Instant) {
        let Some((from, to, at, active)) = self.drag_segment else {
            return;
        };
        let alpha = if active {
            0.9
        } else {
            let elapsed = now.duration_since(at).as_secs_f32();
            if elapsed >= DRAG_TRAIL_LINGER_SECS {
                return;
            }
            1.0 - (elapsed / DRAG_TRAIL_LINGER_SECS)
        };

        let accent = InspectActionKind::Drag.accent_color();
        push_circle_ring(
            &mut self.cmd_list,
            from,
            7.0,
            Color::rgba(accent.r, accent.g, accent.b, 0.25 * alpha),
            Color::rgba(accent.r, accent.g, accent.b, 0.85 * alpha),
            2.0,
        );

        const SAMPLES: usize = 8;
        for idx in 1..SAMPLES {
            let t = idx as f32 / SAMPLES as f32;
            let p = Point::new(from.x + (to.x - from.x) * t, from.y + (to.y - from.y) * t);
            push_circle_ring(
                &mut self.cmd_list,
                p,
                3.0,
                Color::rgba(accent.r, accent.g, accent.b, (0.35 + 0.55 * t) * alpha),
                Color::TRANSPARENT,
                0.0,
            );
        }
    }

    fn build_ripple_quads(&mut self, now: Instant) {
        let Some(r) = self.ripple else {
            return;
        };
        let elapsed = now.duration_since(r.started_at).as_secs_f32();
        if elapsed >= RIPPLE_DURATION_SECS {
            return;
        }
        let t = (elapsed / RIPPLE_DURATION_SECS).clamp(0.0, 1.0);
        let ease = 1.0 - (1.0 - t) * (1.0 - t);
        let radius = 8.0 + ease * 26.0;
        let alpha = (1.0 - t).powf(1.3);
        let accent = r.kind.accent_color();

        push_circle_ring(
            &mut self.cmd_list,
            r.point,
            radius,
            Color::rgba(accent.r, accent.g, accent.b, 0.14 * alpha),
            Color::rgba(accent.r, accent.g, accent.b, 0.95 * alpha),
            2.5,
        );
    }

    fn build_cursor_quads(&mut self, now: Instant) {
        let Some(pos) = self.cursor_pos else {
            return;
        };
        let Some(last) = self.last_activity_at else {
            return;
        };
        let elapsed = now.duration_since(last).as_secs_f32();
        if elapsed >= CURSOR_IDLE_LIFETIME_SECS {
            return;
        }
        let fade = if elapsed > CURSOR_IDLE_LIFETIME_SECS - 1.0 {
            (CURSOR_IDLE_LIFETIME_SECS - elapsed).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let accent = self.cursor_kind.accent_color();
        push_circle_ring(
            &mut self.cmd_list,
            pos,
            10.5,
            Color::rgba(0.02, 0.03, 0.05, 0.45 * fade),
            Color::rgba(0.02, 0.03, 0.05, 0.85 * fade),
            2.0,
        );
        push_circle_ring(
            &mut self.cmd_list,
            pos,
            8.5,
            Color::TRANSPARENT,
            Color::rgba(accent.r, accent.g, accent.b, 0.95 * fade),
            2.0,
        );
        push_circle_ring(
            &mut self.cmd_list,
            pos,
            3.0,
            Color::rgba(1.0, 1.0, 1.0, fade),
            Color::TRANSPARENT,
            0.0,
        );

        let bar_col = Color::rgba(accent.r, accent.g, accent.b, 0.85 * fade);
        push_rect_quad(
            &mut self.cmd_list,
            Rect::new(pos.x - 16.0, pos.y - 1.0, 5.0, 2.0),
            bar_col,
        );
        push_rect_quad(
            &mut self.cmd_list,
            Rect::new(pos.x + 11.0, pos.y - 1.0, 5.0, 2.0),
            bar_col,
        );
        push_rect_quad(
            &mut self.cmd_list,
            Rect::new(pos.x - 1.0, pos.y - 16.0, 2.0, 5.0),
            bar_col,
        );
        push_rect_quad(
            &mut self.cmd_list,
            Rect::new(pos.x - 1.0, pos.y + 11.0, 2.0, 5.0),
            bar_col,
        );
    }

    fn build_status_beacon_quad(&mut self, now: Instant, logical_size: [f32; 2]) {
        let is_recent = self
            .last_activity_at
            .is_some_and(|at| now.duration_since(at).as_secs_f32() < 2.0);
        let accent = self.cursor_kind.accent_color();
        let beacon_pos = Point::new(logical_size[0] - 12.0, 44.0);
        let alpha = if is_recent { 0.95 } else { 0.35 };
        push_circle_ring(
            &mut self.cmd_list,
            beacon_pos,
            4.5,
            Color::rgba(accent.r, accent.g, accent.b, alpha),
            Color::rgba(0.05, 0.08, 0.12, alpha),
            1.5,
        );
    }
}

fn push_circle_ring(
    list: &mut DrawCommandList,
    center: Point,
    radius: f32,
    fill: Color,
    border: Color,
    border_width: f32,
) {
    let d = radius * 2.0;
    let rect = Rect::new(center.x - radius, center.y - radius, d, d);
    list.push_quad(QuadInstance {
        rect: rect.to_array(),
        color: fill.to_linear().to_array(),
        border_color: border.to_linear().to_array(),
        border_width: [border_width; 4],
        corner_radii: [radius; 4],
        ..QuadInstance::default()
    });
}

fn push_rect_quad(list: &mut DrawCommandList, rect: Rect, color: Color) {
    list.push_quad(QuadInstance {
        rect: rect.to_array(),
        color: color.to_linear().to_array(),
        corner_radii: [1.0; 4],
        ..QuadInstance::default()
    });
}