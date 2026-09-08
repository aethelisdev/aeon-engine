// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Window event normalization and platform state for the native Iris editor.

use irisui::core::pointer::PointerState;
use irisui::prelude::Point;
use std::time::Instant;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

/// Window-owned input state; all UI positions use logical pixels, independent of DPI and zoom.
pub(crate) struct PlatformState {
    /// Pointer edges and capture origins shared by docking and viewport UI.
    pub pointer: PointerState,
    /// Current platform modifier state used by UI shortcuts.
    pub modifiers: ModifiersState,
    /// Native physical-pixel scale supplied by the windowing backend.
    pub native_scale: f32,
    physical_cursor: Option<Point>,
    start: Instant,
}

impl PlatformState {
    /// Initializes the platform adapter without creating another UI context or GPU renderer.
    pub fn new(native_scale: f32) -> Self {
        Self { pointer: Default::default(), modifiers: Default::default(), native_scale, physical_cursor: None, start: Instant::now() }
    }

    /// Returns a finite positive combined scale for rendering, hit testing, and scene picking.
    pub fn scale(&self, zoom: f32) -> f32 {
        let scale = self.native_scale * zoom;
        if scale.is_finite() && scale > 0.0 { scale } else { 1.0 }
    }

    /// Records physical events before UI consumption and applies editor zoom shortcuts.
    /// Returns true only for a handled zoom shortcut. Losing focus cancels pointer state rather
    /// than generating a drop, while button releases remain observable after widget consumption.
    pub fn record(&mut self, event: &WindowEvent, zoom: &mut f32) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.physical_cursor = Some(Point::new(position.x as f32, position.y as f32));
                self.refresh_cursor(*zoom);
            }
            WindowEvent::CursorLeft { .. } if !self.pointer.primary_down => { self.pointer.position = None; }
            WindowEvent::MouseInput { state, button, .. } => {
                let down = *state == ElementState::Pressed;
                match button {
                    MouseButton::Left => self.pointer.primary_button(down, self.start.elapsed()),
                    MouseButton::Right => self.pointer.secondary_button(down),
                    _ => {}
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.native_scale = *scale_factor as f32;
                self.refresh_cursor(*zoom);
            }
            WindowEvent::Focused(false) => { self.pointer.cancel(); self.modifiers = ModifiersState::empty(); }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed && (self.modifiers.control_key() || self.modifiers.super_key()) => {
                let changed = match event.physical_key {
                    PhysicalKey::Code(KeyCode::Equal | KeyCode::NumpadAdd) => { *zoom = (*zoom + 0.1).min(2.0); true }
                    PhysicalKey::Code(KeyCode::Minus | KeyCode::NumpadSubtract) => { *zoom = (*zoom - 0.1).max(0.5); true }
                    PhysicalKey::Code(KeyCode::Digit0 | KeyCode::Numpad0) => { *zoom = 1.0; true }
                    _ => false,
                };
                if changed { self.refresh_cursor(*zoom); return true; }
            }
            _ => {}
        }
        false
    }

    fn refresh_cursor(&mut self, zoom: f32) {
        if let Some(p) = self.physical_cursor {
            let scale = self.scale(zoom);
            self.pointer.move_to(Point::new(p.x / scale, p.y / scale));
        }
    }

    /// Normalizes only coordinate-bearing events, borrowing every keyboard and IME event intact.
    pub fn logical_event(&self, event: &WindowEvent, zoom: f32) -> Option<WindowEvent> {
        let scale = self.scale(zoom) as f64;
        match event {
            WindowEvent::CursorMoved { device_id, position } => Some(WindowEvent::CursorMoved {
                device_id: *device_id,
                position: winit::dpi::PhysicalPosition::new(position.x / scale, position.y / scale),
            }),
            WindowEvent::MouseWheel { device_id, delta: MouseScrollDelta::PixelDelta(position), phase } => Some(WindowEvent::MouseWheel {
                device_id: *device_id,
                delta: MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(position.x / scale, position.y / scale)),
                phase: *phase,
            }),
            _ => None,
        }
    }
}