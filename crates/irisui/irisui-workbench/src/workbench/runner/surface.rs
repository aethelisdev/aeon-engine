// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench GPU Surface Coordination (`irisui-workbench::workbench::runner::surface`)
//!
//! Unites [`Workbench`] event handling, resize scaling, and [`IrisSurfaceHost`] hover/render
//! synchronization directly inside `irisui-workbench`, eliminating boilerplate surface glue
//! in consumer applications without exposing internal tree or text-system state.

use std::fmt::Debug;
use std::hash::Hash;

use irisui_core::UiEvent;
use irisui_wgpu::{IrisSurfaceHost, IrisSurfaceRenderParams};
use irisui_winit::frameless::WindowInterface;

use super::Workbench;
use crate::workbench::WorkbenchOutcome;

/// Descriptor specifying WGPU target surface and scaling parameters for
/// [`Workbench::render_to_surface`].
pub struct WorkbenchSurfaceRender<'a> {
    /// Active WGPU device handle.
    pub device: &'a wgpu::Device,
    /// Active WGPU submission queue.
    pub queue: &'a wgpu::Queue,
    /// Active WGPU command encoder recording the render pass.
    pub encoder: &'a mut wgpu::CommandEncoder,
    /// Destination texture view into which the retained UI tree is composited.
    pub target_view: &'a wgpu::TextureView,
    /// Target surface dimensions in physical pixels `(width, height)`.
    pub physical_screen_size: (u32, u32),
    /// Active UI scale factor applied to logical UI geometry.
    pub zoom_factor: f32,
}

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
    Action: Send + Sync + 'static,
{
    /// Dispatches a translated [`UiEvent`] through this workbench, forwards hover flags onto
    /// `surface`, updates logical window bounds when zoom changes, and propagates shell/panel
    /// invalidation flags into [`Self::notifier`].
    pub fn handle_event_with_surface<W: WindowInterface>(
        &mut self,
        surface: &mut IrisSurfaceHost,
        window: &mut W,
        physical_size: (u32, u32),
        event: &UiEvent,
    ) -> WorkbenchOutcome<Action, TabId> {
        let outcome = self.handle_event(window, event);
        surface.apply_hover_flags(
            outcome.is_hover_target_changed(),
            outcome.hover_affects_text,
            outcome.hover_requires_full_upload,
        );
        if outcome.zoom_changed.is_some() {
            self.sync_physical_window_size(physical_size.0, physical_size.1);
        }
        self.sync_outcome_invalidation(&outcome);
        outcome
    }

    /// Renders the retained UI tree directly into `params.target_view` using `surface` without
    /// requiring callers to access internal tree, text-system, or hover state fields.
    pub fn render_to_surface(
        &mut self,
        surface: &mut IrisSurfaceHost,
        params: WorkbenchSurfaceRender<'_>,
    ) {
        let hovered_tag = self.hovered_tag();
        surface.render_to_view(
            &mut self.tree,
            &mut self.text_system,
            hovered_tag,
            IrisSurfaceRenderParams {
                device: params.device,
                queue: params.queue,
                encoder: params.encoder,
                target_view: params.target_view,
                physical_screen_size: params.physical_screen_size,
                zoom_factor: params.zoom_factor,
                extra_text_occluders: &[],
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptors::WorkbenchDescriptor;
    use irisui_core::geometry::{Point, Size};
    use irisui_dock::{DockState, DockTree};
    use irisui_winit::CursorIcon;
    use irisui_winit::frameless::{ResizeDirection, WindowOpError};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Tab {
        Viewport,
    }

    struct DummyWindow;
    impl WindowInterface for DummyWindow {
        fn set_cursor(&mut self, _icon: CursorIcon) {}
        fn drag_resize_window(&mut self, _dir: ResizeDirection) -> Result<(), WindowOpError> {
            Ok(())
        }
        fn drag_window(&mut self) -> Result<(), WindowOpError> {
            Ok(())
        }
        fn is_maximized(&self) -> bool {
            false
        }
        fn toggle_maximize(&mut self) {}
        fn minimize(&mut self) {}
        fn close(&mut self) {}
        fn logical_size(&self) -> Size {
            Size::new(1920.0, 1080.0)
        }
        fn scale_factor(&self) -> f64 {
            1.0
        }
    }

    #[test]
    fn test_sync_outcome_invalidation_and_event_dispatch() {
        let mut tree = DockTree::new();
        let root = tree.create_leaf(vec![Tab::Viewport]);
        tree.set_root(root);
        let mut wb: Workbench<Tab, (), ()> =
            Workbench::new(WorkbenchDescriptor::new("Test", DockState::new(tree)));

        wb.notifier.clear_all();
        wb.sync_outcome_invalidation(&WorkbenchOutcome::<(), Tab>::empty().with_shell_dirty(true));
        assert!(wb.notifier.is_dirty(Tab::Viewport));

        wb.notifier.clear_all();
        wb.sync_outcome_invalidation(
            &WorkbenchOutcome::<(), Tab>::empty().with_invalidated_panel(Some(Tab::Viewport)),
        );
        assert!(wb.notifier.dirty_panels().contains(&Tab::Viewport));

        let mut win = DummyWindow;
        let outcome = wb.handle_event(
            &mut win,
            &UiEvent::MouseMove {
                point: Point::new(10.0, 10.0),
            },
        );
        wb.sync_outcome_invalidation(&outcome);
    }
}