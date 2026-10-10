// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Overlay lifecycle, initialization, and tree reconstruction subsystem for Iris UI editor overlays.

use super::compositor;
use super::menubar;
use super::preferences;
use super::status_bar;
use super::theme::*;
use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use crate::ui::panel_layout::PanelId;
use irisui::prelude::*;
use irisui::text::TextSystem;

impl IrisEditorOverlay {
    /// Height of the top menubar panel in physical pixels.
    pub const MENUBAR_HEIGHT: f32 = menubar::MENUBAR_HEIGHT;

    /// Height of the bottom status bar in physical pixels.
    pub const STATUS_BAR_HEIGHT: f32 = status_bar::STATUS_BAR_HEIGHT;

    /// Initializes a new Iris UI editor overlay pipeline for the specified surface format.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        Self {
            tree: UiTree::new(),
            layout_engine: LayoutEngine::new(),
            renderer: IrisRenderer::new(device, target_format),
            text_system: TextSystem::new(),
            text_renderer: None,
            command_list: DrawCommandList::new(),
            notifier: UiNotifier::new(),
            panels: super::update_panels::create_default_panel_registry(),
            screen_width: 1920.0,
            screen_height: 1080.0,
            is_visible: true,
            target_format,
            start_time: std::time::Instant::now(),
            tools_texture: None,
            chrome: super::types::IrisChromeState::default(),
            menubar: super::types::MenubarOverlayState::default(),
            modals: super::modals::types::ModalsOverlayState::default(),
            preferences: super::preferences::types::PreferencesDialogState::default(),
            viewport_hud: super::viewport_hud::types::ViewportHudState::default(),
            stats: super::stats::types::StatsPanelState::default(),
            hierarchy: super::hierarchy::types::HierarchyPanelState::default(),
            console: super::console::types::ConsolePanelState::default(),
            assets: super::assets::types::AssetsPanelState::default(),
            timeline: super::timeline::types::TimelinePanelState::default(),
            material: super::material::types::MaterialPanelState::default(),
            ui_designer: super::ui_designer::types::UiDesignerPanelState::default(),
            inspector: super::inspector::types::InspectorPanelState::default(),
            focus_manager: FocusManager::new(),
            overlay_tree: super::overlay_tree::OverlayTree::new(),
            compositor: super::compositor::TreeCompositor::new(),
        }
    }

    /// Reconstructs and resolves layout for the top menu bar, active dropdown, modals, and bottom status bar.
    pub fn update_overlays(&mut self, mut params: OverlayUpdateParams<'_>) {
        let (screen_width, screen_height) = params.context.dimensions;
        self.screen_width = screen_width;
        self.screen_height = screen_height;

        self.modals.sync_from_dialogs(&params.dialogs);

        let modal_active = self.modals.is_about_active
            || params.dialogs.show_preferences
            || self.modals.is_delete_active
            || self.modals.is_new_folder_active
            || self.modals.is_rename_active
            || self.modals.is_loading_active;

        let floating_count = params
            .context
            .layout_state
            .dock_state
            .floating_windows
            .len();

        let has_drag_payload = params.panel_data.asset_browser.drag_payload.is_some();
        let cursor = self.cursor_pos();
        let cursor_moved = (self.chrome.last_cursor_pos.x - cursor.x).abs() > 0.001
            || (self.chrome.last_cursor_pos.y - cursor.y).abs() > 0.001;

        // Multi-Tree Hit-Testing:
        // 1. First inspect the top-level OverlayTree (popups, dropdowns, modals)
        let mut hovered_target = None;
        let mut hovered_panel = None;

        if self.overlay_tree.is_open()
            && let Some(hit) = self.overlay_tree.tree().hit_test_target(cursor)
            && hit.tag != 0
        {
            hovered_target = Some(hit.tag);
        }

        // 2. Base shell tree (MenuBar, StatusBar, Viewport HUD)
        if hovered_target.is_none() {
            hovered_target = self
                .tree
                .hit_test_target(cursor)
                .and_then(|h| if h.tag != 0 { Some(h.tag) } else { None });
        }

        // 3. Active dock panel trees (only if not occluded by an open overlay)
        if hovered_target.is_none() && !self.overlay_tree.contains_point(cursor) {
            for &pid in crate::ui::panel_layout::PanelId::all_tool_panels() {
                if pid == crate::ui::panel_layout::PanelId::Viewport {
                    continue;
                }
                if let Some(panel) = self.panels.get(pid.id_str())
                    && let Some(bounds) = panel.bounds()
                    && compositor::is_point_in_panel(cursor, bounds)
                    && let Some(hit) = self.hit_test_panel(pid, cursor)
                    && hit.tag != 0
                {
                    hovered_target = Some(hit.tag);
                    hovered_panel = Some(pid);
                    break;
                }
            }
        }

        let hover_target_changed = cursor_moved && hovered_target != self.chrome.last_hovered_tag;
        self.chrome.hovered_tag = hovered_target;

        // When hover changes on a dock panel or overlay, tag for redraw so reactive UX styles activate instantly
        if hover_target_changed {
            if self.overlay_tree.is_open() {
                self.notifier.tag_all();
            }
            if let Some(pid) = hovered_panel {
                self.notifier.tag_redraw(pid);
            }
            if let Some(last_pid) = self.chrome.last_hovered_panel {
                self.notifier.tag_redraw(last_pid);
            }
        }
        self.chrome.hovered_panel = hovered_panel;

        // 1. Genuine global shell events (only these trigger global tag_all)
        if self.chrome.last_dimensions != params.context.dimensions
            || (self.chrome.last_zoom_factor - params.context.zoom_factor).abs() > 1e-4
            || self.chrome.last_floating_count != floating_count
            || self.modals.last_modal_active != modal_active
            || self.preferences.last_tab != self.preferences.tab
            || (self.preferences.scroll_y - self.preferences.last_scroll_y).abs() > 0.001
            || self.chrome.last_has_drag_payload != has_drag_payload
            || self.menubar.active_menu.is_some()
            || self.chrome.active_dock_overflow.is_some()
            || self.chrome.needs_layout_rebuild
            || self.chrome.needs_scroll_sync
            || has_drag_payload
        {
            self.notifier.tag_all();
        }

        // 2. Delegate panel dirty evaluations to the panel registry
        self.sync_panel_registry_dirty(&params);

        // 3. Poll panel registry: panel dirty decisions flow through the registry and tag redraws
        self.notifier
            .poll_registry(&self.panels, PanelId::from_id_str);

        if !self.is_visible {
            self.command_list.clear();
            return;
        }

        let blink_caret = (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2);
        let focus_target_changed = self.focus_manager.focused_tag != self.chrome.last_focused_tag;
        let blink_changed =
            self.focus_manager.has_focus() && (blink_caret != self.chrome.last_blink_caret);

        if !self.chrome.always_rebuild && !self.notifier.is_any_dirty() && !self.tree.is_empty() {
            if self.chrome.needs_scroll_sync || self.has_any_scroll_changed() {
                self.sync_tree_scroll_offsets(&params);
                self.command_list.clear();
                if let Some(root) = self.tree.root() {
                    self.populate_draw_commands(root, None, Some(params.telemetry.frame_pacing));
                }
                self.composite_active_layers();
            } else if (hover_target_changed || focus_target_changed || blink_changed)
                && let Some(root) = self.tree.root()
            {
                self.command_list.clear();
                self.populate_draw_commands(root, None, Some(params.telemetry.frame_pacing));
                self.composite_active_layers();
            }
            // UI is completely clean and sleeping; zero allocations, zero panel flicker or erasure
            self.chrome.last_cursor_pos = self.cursor_pos();
            self.chrome.last_hovered_tag = self.chrome.hovered_tag;
            self.chrome.last_hovered_panel = self.chrome.hovered_panel;
            self.chrome.last_focused_tag = self.focus_manager.focused_tag;
            self.chrome.last_blink_caret = blink_caret;
            return;
        }

        let cursor = self.cursor_pos();

        let floating_changed = floating_count != self.chrome.last_floating_count
            || params
                .context
                .layout_state
                .dock_state
                .floating_windows
                .iter()
                .zip(&self.chrome.floating_window_rects)
                .any(|(w, r)| w.rect != *r);

        let is_shell_dirty = self.chrome.always_rebuild
            || self.tree.root().is_none()
            || self.notifier.is_global_dirty()
            || self
                .notifier
                .is_dirty(crate::ui::panel_layout::PanelId::Viewport)
            || self.chrome.is_dirty(
                params.context.dimensions,
                params.context.zoom_factor,
                floating_count,
                has_drag_payload,
            )
            || floating_changed
            || params
                .context
                .layout_state
                .dock_state
                .active_splitter
                .is_some()
            || params.context.layout_state.dock_state.active_drag.is_some()
            || self.modals.last_modal_active != modal_active
            || modal_active
            || self.menubar.active_menu.is_some()
            || self.menubar.dropdown_rect.is_some()
            || self.inspector.is_add_menu_open
            || self.inspector.is_color_picker_open
            || self.inspector.active_dropdown.is_some()
            || params.dialogs.show_preferences != self.preferences.card_rect.is_some()
            || (params.dialogs.show_preferences
                && (self.preferences.last_tab != self.preferences.tab
                    || (self.preferences.scroll_y - self.preferences.last_scroll_y).abs() > 0.001))
            || self.chrome.needs_layout_rebuild
            || self.chrome.needs_scroll_sync;

        if self.menubar.active_menu.is_none() {
            self.menubar.actions.clear();
            self.menubar.dropdown_rect = None;
            if matches!(
                self.overlay_tree.active_overlay(),
                Some(super::overlay_tree::ActiveOverlay::MenubarDropdown(_))
            ) {
                self.overlay_tree.close();
            }
        }
        if !params.dialogs.show_preferences {
            self.preferences.card_rect = None;
            self.preferences.content_rect = None;
            if matches!(
                self.overlay_tree.active_overlay(),
                Some(super::overlay_tree::ActiveOverlay::Modal(
                    super::overlay_tree::ModalKind::Preferences
                ))
            ) {
                self.overlay_tree.close();
            }
        }
        if !params.dialogs.show_about
            && matches!(
                self.overlay_tree.active_overlay(),
                Some(super::overlay_tree::ActiveOverlay::Modal(
                    super::overlay_tree::ModalKind::About
                ))
            )
        {
            self.overlay_tree.close();
        }
        if params.dialogs.delete_target.is_none()
            && matches!(
                self.overlay_tree.active_overlay(),
                Some(super::overlay_tree::ActiveOverlay::Modal(
                    super::overlay_tree::ModalKind::DeleteConfirmation
                ))
            )
        {
            self.overlay_tree.close();
        }
        if params.dialogs.new_folder_parent.is_none()
            && matches!(
                self.overlay_tree.active_overlay(),
                Some(super::overlay_tree::ActiveOverlay::Modal(
                    super::overlay_tree::ModalKind::NewFolder
                ))
            )
        {
            self.overlay_tree.close();
        }
        if params.dialogs.rename_target.is_none()
            && matches!(
                self.overlay_tree.active_overlay(),
                Some(super::overlay_tree::ActiveOverlay::Modal(
                    super::overlay_tree::ModalKind::Rename
                ))
            )
        {
            self.overlay_tree.close();
        }
        if !self.inspector.is_add_menu_open
            && !self.inspector.is_color_picker_open
            && self.inspector.active_dropdown.is_none()
            && matches!(
                self.overlay_tree.active_overlay(),
                Some(
                    super::overlay_tree::ActiveOverlay::InspectorAddComponent
                        | super::overlay_tree::ActiveOverlay::InspectorColorPicker
                        | super::overlay_tree::ActiveOverlay::InspectorDropdown
                )
            )
        {
            self.overlay_tree.close();
        }

        if !self
            .focus_manager
            .is_tag_focused(super::assets::ASSETS_TAG_SEARCH_INPUT)
        {
            self.assets.search_query = params.panel_data.asset_browser.search_query.clone();
        }
        self.assets.current_folder = params.panel_data.asset_browser.current_folder.clone();
        self.timeline.selected_entity = params.scene.selected_entity;
        if self.material.selected_entity != params.scene.selected_entity {
            self.material.selected_entity = params.scene.selected_entity;
            self.material.scroll_y = 0.0;
        }

        // Safely commit pending inspector edits on selection change
        if let Some(session) = self.inspector.active_number_input.take() {
            if Some(session.entity) != params.scene.selected_entity {
                if let Ok(v) =
                    super::inspector::evaluate_inspector_math(&session.buffer, session.initial_val)
                {
                    self.inspector
                        .actions
                        .push(super::inspector::InspectorAction::SetNumberValue(
                            session.entity,
                            session.id,
                            v,
                        ));
                    self.inspector.actions.push(
                        super::inspector::InspectorAction::CommitNumberEdit(
                            session.entity,
                            session.id,
                        ),
                    );
                } else {
                    self.inspector.edit_start_snapshot = None;
                }
            } else {
                self.inspector.active_number_input = Some(session);
            }
        }
        if let Some((ent, id, buf)) = self.inspector.active_text_input.take() {
            if Some(ent) != params.scene.selected_entity {
                self.inspector
                    .actions
                    .push(super::inspector::InspectorAction::SetTextValue(
                        ent, id, buf,
                    ));
            } else {
                self.inspector.active_text_input = Some((ent, id, buf));
            }
        }
        if let Some((ent, buf)) = self.inspector.rename_buffer.take() {
            if Some(ent) != params.scene.selected_entity {
                if !buf.trim().is_empty() {
                    self.inspector
                        .actions
                        .push(super::inspector::InspectorAction::RenameEntity(ent, buf));
                }
            } else {
                self.inspector.rename_buffer = Some((ent, buf));
            }
        }

        if is_shell_dirty {
            self.tree.clear();
            self.layout_engine.clear();

            let Ok(root) = self.tree.create_root() else {
                return;
            };

            // Declarative configuration of root container spanning full viewport
            let mut root_scope = UiScope::new(&mut self.tree, root);
            root_scope.configure_container(
                "IrisEditorRoot",
                Style::new()
                    .flex_col()
                    .justify_content(JustifyContent::SpaceBetween)
                    .width(screen_width)
                    .height(screen_height),
                false,
            );

            // 1. Top MenuBar
            let menu_output = menubar::build_top_menu_bar(
                &mut self.tree,
                root,
                screen_width,
                self.menubar.active_menu,
                params.context.is_editing,
                cursor,
            );
            self.menubar.button_ids = menu_output.menu_button_ids.to_vec();

            // 2. Bottom Diagnostics & Status Bar
            let _status_bar_id = status_bar::build_bottom_status_bar(
                &mut self.tree,
                root,
                &status_bar::StatusBarParams {
                    screen_width,
                    screen_height,
                    status_spans: params.context.status_spans,
                },
            );

            // Pre-measure all text nodes to populate intrinsic content_size
            self.text_system.measure_subtree_text(&mut self.tree, root);

            // Compute Taffy layout for top menu bar and status bar (only 2 children for clean SpaceBetween pinning)
            let _ = self
                .layout_engine
                .compute_layout(&mut self.tree, Size::new(screen_width, screen_height));

            // 2b. Baseline dividers under MenuBar and above StatusBar (positioned absolutely with explicit styles)
            let mut scope = UiScope::new(&mut self.tree, root);
            scope.empty_box_passive_named(
                "IrisTopMenuBarDivider",
                Style::new()
                    .position_absolute()
                    .left(0.0)
                    .top(Self::MENUBAR_HEIGHT - 1.0)
                    .width(screen_width)
                    .height(1.0)
                    .background(BORDER_MICRON),
            );
            scope.empty_box_passive_named(
                "IrisBottomStatusBarDivider",
                Style::new()
                    .position_absolute()
                    .left(0.0)
                    .top(screen_height - Self::STATUS_BAR_HEIGHT)
                    .width(screen_width)
                    .height(1.0)
                    .background(BORDER_MICRON),
            );
            scope.finish_layout(Rect::new(0.0, 0.0, screen_width, screen_height));

            // 3. Build Native Iris UI Docking Frame (Splitters, Tab Strips, Compact Snug Tabs, Active Indicators)
            let workspace_rect = Rect::new(
                0.0,
                Self::MENUBAR_HEIGHT,
                screen_width,
                (screen_height - Self::MENUBAR_HEIGHT - Self::STATUS_BAR_HEIGHT).max(1.0),
            );
            let pref_rect = if params.dialogs.show_preferences {
                let (left, top) = if let Some(pos) = self.preferences.pos {
                    (pos.x, pos.y)
                } else {
                    (
                        ((screen_width - preferences::PREF_CARD_WIDTH) * 0.5).max(0.0),
                        ((screen_height - preferences::PREF_CARD_HEIGHT) * 0.5).max(28.0),
                    )
                };
                Some(Rect::new(
                    left,
                    top,
                    preferences::PREF_CARD_WIDTH,
                    preferences::PREF_CARD_HEIGHT,
                ))
            } else {
                None
            };

            let is_cursor_occluded = self.is_point_over_modal_or_dropdown(cursor)
                || pref_rect.is_some_and(|r| {
                    cursor.x >= r.x
                        && cursor.x <= r.right()
                        && cursor.y >= r.y
                        && cursor.y <= r.bottom()
                })
                || params
                    .context
                    .layout_state
                    .dock_state
                    .is_point_over_floating_window(cursor);

            // 3. Native Iris Docking Framework: construct full dock tree (splitters, container tabs, content rects)
            let dock_frame = super::native_dock::build_native_dock(
                &mut self.tree,
                root,
                params.context.layout_state,
                workspace_rect,
                cursor,
                is_cursor_occluded,
            );
            self.chrome.native_dock_frame = Some(dock_frame);

            let is_floating = |panel: crate::ui::panel_layout::PanelId| {
                super::floating_layer::is_panel_in_floating_window(
                    params.context.layout_state,
                    panel,
                )
            };

            // 4. If Viewport canvas is valid and docked, build Viewport content (3D scene texture + HUD)
            let is_viewport_active = !is_floating(crate::ui::panel_layout::PanelId::Viewport)
                && params.viewport.viewport_rect.width > 20.0
                && params.viewport.viewport_rect.height > 20.0;
            self.viewport_hud.is_active = is_viewport_active;

            if is_viewport_active {
                self.build_viewport_content(root, params.viewport.viewport_rect, &params);
            }

            // 5. Layer 1: Render all FLOATING Windows Frame Chrome (Z-Index: Foreground Frame Layer)
            let (floating_rects, _floating_containers) =
                super::floating_layer::build_floating_windows(
                    &mut self.tree,
                    root,
                    params.context.layout_state,
                    cursor,
                );
            self.chrome.floating_window_rects = floating_rects;

            // 6. FLOATING OVERLAYS (Rendered on top of docked and floating panels):
            self.build_shell_overlays(
                root,
                &mut params,
                super::update_overlays::ShellOverlayContext {
                    screen_width,
                    screen_height,
                    cursor,
                    workspace_rect,
                    is_cursor_occluded,
                },
            );
        }

        // Multi-Tree Architecture: Render all tool panels into their independent EditorDockPanel trees
        for &panel in crate::ui::panel_layout::PanelId::all_tool_panels() {
            if panel != crate::ui::panel_layout::PanelId::Viewport {
                self.render_dock_panel_into_tree(panel, &params);
            }
        }

        // Populate DrawCommandList from resolved layout nodes (with inline oscilloscope curves)
        self.command_list.clear();
        if let Some(root) = self.tree.root() {
            self.populate_draw_commands(root, None, Some(params.telemetry.frame_pacing));
        }

        // Composite active layers: Shell (Layer 0) + Dock Panels (Layer 1) + OverlayTree (Layer 2)
        self.composite_active_layers();

        self.chrome.last_dimensions = params.context.dimensions;
        self.chrome.last_zoom_factor = params.context.zoom_factor;
        self.inspector.last_selected_entity = params.scene.selected_entity;
        self.chrome.last_floating_count = floating_count;
        self.modals.last_modal_active = modal_active;
        self.preferences.last_tab = self.preferences.tab;
        self.preferences.last_scroll_y = self.preferences.scroll_y;
        self.material.last_scroll_y = self.material.scroll_y;
        self.stats.last_scroll_y = self.stats.scroll_y;
        self.hierarchy.last_scroll_y = self.hierarchy.scroll_y;
        self.console.interactions.last_scroll_y = self.console.interactions.scroll_y;
        self.inspector.interactions.last_scroll_y = self.inspector.scroll_y;
        self.assets.interactions.last_scroll_y = self.assets.scroll_y;
        self.assets.last_tree_scroll_y = self.assets.tree_scroll_y;
        self.chrome.last_has_viewport_texture = params.viewport.has_viewport_texture;
        self.chrome.last_has_drag_payload = has_drag_payload;
        self.chrome.last_cursor_pos = self.cursor_pos();
        self.chrome.last_hovered_tag = self.chrome.hovered_tag;
        self.chrome.last_hovered_panel = self.chrome.hovered_panel;
        self.chrome.last_focused_tag = self.focus_manager.focused_tag;
        self.chrome.last_blink_caret = blink_caret;
        self.chrome.needs_layout_rebuild = false;
        self.chrome.needs_scroll_sync = false;
        self.notifier.clear_all();
    }
}