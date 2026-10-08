// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Overlay lifecycle, initialization, and tree reconstruction subsystem for Iris UI editor overlays.

use super::about::build_about_dialog;
use super::hierarchy;
use super::menubar;
use super::modals::{
    self, build_delete_modal, build_loading_overlay, build_new_folder_modal, build_rename_modal,
};
use super::preferences::{self, build_preferences_dialog};
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
        }
    }

    /// Reconstructs and resolves layout for the top menu bar, active dropdown, modals, and bottom status bar.
    pub fn update_overlays(&mut self, params: OverlayUpdateParams<'_>) {
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
        let cursor_moved = (self.chrome.last_cursor_pos.x - self.cursor_pos().x).abs() > 0.001
            || (self.chrome.last_cursor_pos.y - self.cursor_pos().y).abs() > 0.001;
        let hovered_target = self
            .tree
            .hit_test_target(self.cursor_pos())
            .and_then(|h| if h.tag != 0 { Some(h.tag) } else { None });
        let hover_target_changed = cursor_moved && hovered_target != self.chrome.last_hovered_tag;
        self.chrome.hovered_tag = hovered_target;

        // 1. Genuine global shell events (only these trigger global tag_all)
        if self.chrome.last_dimensions != params.context.dimensions
            || (self.chrome.last_zoom_factor - params.context.zoom_factor).abs() > 1e-4
            || self.chrome.last_floating_count != floating_count
            || self.modals.last_modal_active != modal_active
            || self.preferences.last_tab != self.preferences.tab
            || self.chrome.last_has_drag_payload != has_drag_payload
            || self.menubar.active_menu.is_some()
            || self.chrome.active_dock_overflow.is_some()
            || self.chrome.needs_layout_rebuild
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
            } else if (hover_target_changed || focus_target_changed || blink_changed)
                && let Some(root) = self.tree.root()
            {
                self.populate_draw_commands(root, None, Some(params.telemetry.frame_pacing));
            }
            // UI is completely clean and sleeping; zero allocations, zero panel flicker or erasure
            self.chrome.last_cursor_pos = self.cursor_pos();
            self.chrome.last_hovered_tag = self.chrome.hovered_tag;
            self.chrome.last_focused_tag = self.focus_manager.focused_tag;
            self.chrome.last_blink_caret = blink_caret;
            return;
        }

        let cursor = self.cursor_pos();

        self.tree.clear();
        self.layout_engine.clear();
        self.command_list.clear();

        if self.menubar.active_menu.is_none() {
            self.menubar.actions.clear();
            self.menubar.dropdown_rect = None;
        }
        if !params.dialogs.show_preferences {
            self.preferences.card_rect = None;
            self.preferences.content_rect = None;
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
            super::floating_layer::is_panel_in_floating_window(params.context.layout_state, panel)
        };

        // 4. If Viewport canvas is valid and docked, build Viewport content (3D scene texture + HUD)
        let is_viewport_active = !is_floating(crate::ui::panel_layout::PanelId::Viewport)
            && params.viewport.viewport_rect.width > 20.0
            && params.viewport.viewport_rect.height > 20.0;
        self.viewport_hud.is_active = is_viewport_active;

        if is_viewport_active {
            self.build_viewport_content(root, params.viewport.viewport_rect, &params);
        }

        // 4. Layer 0: Render all DOCKED panels first (Z-Index: Background Workspace Layer)
        for &panel in crate::ui::panel_layout::PanelId::all_tool_panels() {
            if !is_floating(panel) {
                self.render_panel_by_id(panel, root, &params);
            }
        }

        // 5. Layer 1: Render all FLOATING Windows and their Active Panels (Z-Index: Foreground Layer)
        let (floating_rects, floating_containers) = super::floating_layer::build_floating_windows(
            &mut self.tree,
            root,
            params.context.layout_state,
            cursor,
        );
        self.chrome.floating_window_rects = floating_rects;

        for (panel_id, container_id) in floating_containers {
            self.render_panel_by_id(panel_id, container_id, &params);
        }

        // 6. FLOATING OVERLAYS (Rendered on top of docked and floating panels):

        // 6b. If Preferences dialogue is active, build its floating card
        if params.dialogs.show_preferences {
            let blink_caret = (self.start_time.elapsed().as_millis() % 1060) < 530;
            let hovered_tag = self.tree.hit_test_target(cursor).map(|h| h.tag);
            let caret_buf;
            let active_number_input =
                if let Some((t, ref s, is_all_selected)) = self.preferences.active_number_input {
                    let text_with_caret = if !is_all_selected && blink_caret {
                        caret_buf = format!("{}|", s);
                        caret_buf.as_str()
                    } else {
                        s.as_str()
                    };
                    Some((t, text_with_caret, is_all_selected))
                } else {
                    None
                };
            let (_pref_id, card_rect, content_rect, max_scroll_y) = build_preferences_dialog(
                &mut self.tree,
                preferences::PreferencesParams {
                    screen_width,
                    screen_height,
                    window_pos: self.preferences.pos,
                    active_tab: self.preferences.tab,
                    scroll_offset_y: self.preferences.scroll_y,
                    is_scrollbar_dragging: self.preferences.active_scrollbar_drag.is_some(),
                    active_dropdown: self.preferences.dropdown,
                    dropdown_trigger_rect: self.preferences.dropdown_trigger_rect,
                    collapsed_sections: &self.preferences.collapsed_sections,
                    active_number_input,
                    blink_caret,
                    cursor_pos: cursor,
                    hovered_tag,
                    zoom_factor: params.context.zoom_factor,
                    graphics_settings: params.preferences.graphics_settings,
                    snapping_settings: params.preferences.snapping_settings,
                    editor_config: params.preferences.editor_config,
                    enable_live_updates: params.context.enable_live_updates,
                    enabled_modules: params.preferences.enabled_modules,
                    events: &self.preferences.pending_interaction_events,
                },
            );
            self.preferences.pending_interaction_events.clear();
            self.preferences.card_rect = Some(card_rect);
            self.preferences.content_rect = Some(content_rect);
            self.preferences.max_scroll_y = max_scroll_y;
        }

        // 6c. If About Aeon Engine modal dialogue is active, build its centered card
        if params.dialogs.show_about {
            let pending_events = std::mem::take(&mut self.modals.pending_interaction_events);
            let hovered_id = self.tree.hit_test_layered(cursor);
            let _about_id = build_about_dialog(
                &mut self.tree,
                screen_width,
                screen_height,
                cursor,
                &pending_events,
                hovered_id,
            );
        }

        // 6d. If Delete Confirmation modal is active, build its card
        if let Some(target_path) = params.dialogs.delete_target {
            let _del_id = build_delete_modal(
                &mut self.tree,
                target_path,
                screen_width,
                screen_height,
                cursor,
            );
        }

        let elapsed_secs = self.start_time.elapsed().as_secs_f32();
        let cursor_blink_visible = (self.start_time.elapsed().as_millis() / 530).is_multiple_of(2);

        // 6e. If New Folder modal is active, build its card
        if let Some(parent_path) = params.dialogs.new_folder_parent {
            let input_name = self.modals.new_folder_buffer.as_str();
            let text_width = self
                .text_system
                .measure_text(input_name, 12.0, 28.0, None)
                .width;
            let _folder_id = build_new_folder_modal(
                &mut self.tree,
                modals::FolderModalParams {
                    parent_path,
                    input_text: input_name,
                    text_width,
                    cursor_blink_visible,
                    screen_width,
                    screen_height,
                    cursor_pos: cursor,
                },
            );
        }

        // 6f. If Rename modal is active, build its card
        if let Some((target_path, is_folder)) = params.dialogs.rename_target {
            let input_name = self.modals.rename_buffer.as_str();
            let text_width = self
                .text_system
                .measure_text(input_name, 12.0, 28.0, None)
                .width;
            let _rename_id = build_rename_modal(
                &mut self.tree,
                modals::RenameModalParams {
                    target_path,
                    input_text: input_name,
                    text_width,
                    is_folder,
                    cursor_blink_visible,
                    screen_width,
                    screen_height,
                    cursor_pos: cursor,
                },
            );
        }

        // 6g. If Asset Loading overlay is active, build its splash screen
        if params.dialogs.is_loading_assets {
            let _loading_id = build_loading_overlay(
                &mut self.tree,
                modals::LoadingOverlayParams {
                    screen_width,
                    screen_height,
                    time_secs: elapsed_secs,
                },
            );
        }

        // 6h. Hierarchy Add Menu and Context Menu (Rendered as topmost floating overlays)
        if let Some(hier_rect) = params.panel_rects.hierarchy {
            let hier_params = hierarchy::HierarchyPanelParams {
                panel_rect: hier_rect,
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                search_query: &self.hierarchy.search_query,
                is_editing: params.context.is_editing,
                is_2d: params.context.is_2d_mode,
                scroll_y: self.hierarchy.scroll_y,
                active_submenu: self.hierarchy.active_submenu,
                active_sub_submenu: self.hierarchy.active_sub_submenu,
                is_add_menu_open: self.hierarchy.is_add_menu_open,
                active_context_menu: self.hierarchy.active_context_menu,
                cursor_pos: cursor,
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
                collapsed_entities: &self.hierarchy.collapsed_entities,
            };
            hierarchy::build_hierarchy_overlays(&mut self.tree, root, &hier_params);
        }

        // 6i. Native Dock Drag Overlays (5-way compass navigator, drop zone preview, floating tab badge)
        // Rendered as topmost floating overlays so they are drawn above the 3D Viewport texture,
        // docked panels, and floating windows.
        super::native_dock::build_native_dock_drag_overlays(
            &mut self.tree,
            root,
            params.context.layout_state,
            workspace_rect,
        );

        // 6j. Asset Drag & Viewport Drop Overlays (Landing ring on ground plane & cursor tooltip badge)
        if let Some(payload) = &params.panel_data.asset_browser.drag_payload {
            super::assets::build_asset_drag_overlays(
                &mut self.tree,
                root,
                payload,
                cursor,
                params.viewport.viewport_rect,
                params.viewport.camera,
                params.context.is_2d_mode,
            );
        }

        // 6k. Top Menubar Dropdown Popup (Rendered as topmost overlay above all docked panels,
        // floating windows, and modal dialogs so it always has absolute top visual hierarchy)
        if let Some(active) = self.menubar.active_menu {
            let anchor_x = self
                .menubar
                .button_ids
                .iter()
                .find(|(m, _)| *m == active)
                .and_then(|(_, id)| self.tree.get(*id))
                .map(|n| n.computed_rect.x)
                .unwrap_or(6.0);

            let (_dropdown_id, actions, dd_rect) = menubar::build_floating_dropdown(
                &mut self.tree,
                root,
                menubar::DropdownMenuParams {
                    active,
                    anchor_x,
                    layout_state: params.context.layout_state,
                    can_undo: params.context.can_undo,
                    can_redo: params.context.can_redo,
                    cursor_pos: cursor,
                },
            );

            self.menubar.actions = actions;
            self.menubar.dropdown_rect = Some(dd_rect);
        }

        // 6l. Native Dock Tab Overflow Dropdown Menu (Rendered at top overlay layer)
        if let Some((leaf_id, anchor_rect)) = self.chrome.active_dock_overflow
            && let Some(ref mut frame) = self.chrome.native_dock_frame
        {
            super::native_dock::build_native_dock_overflow_menu(
                &mut self.tree,
                root,
                super::native_dock::NativeDockOverflowMenuParams {
                    leaf_id,
                    anchor_rect,
                    layout_state: params.context.layout_state,
                    cursor_pos: cursor,
                    is_cursor_occluded,
                },
                frame,
            );
        }

        // Populate DrawCommandList from resolved layout nodes (with inline oscilloscope curves)
        self.populate_draw_commands(root, None, Some(params.telemetry.frame_pacing));

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
        self.chrome.last_focused_tag = self.focus_manager.focused_tag;
        self.chrome.last_blink_caret = blink_caret;
        self.chrome.needs_layout_rebuild = false;
        self.chrome.needs_scroll_sync = false;
        self.notifier.clear_all();
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::iris_bridge::types::IrisChromeState;

    #[test]
    fn test_always_rebuild_default_and_setter() {
        let mut chrome = IrisChromeState::default();
        assert!(
            chrome.always_rebuild,
            "always_rebuild must default to true for immediate-mode frame updates"
        );
        chrome.always_rebuild = false;
        assert!(!chrome.always_rebuild);
    }

    #[test]
    fn test_collect_text_sections_with_hover_state() {
        use crate::ui::iris_bridge::IrisEditorOverlay;
        use irisui::prelude::{Color, Rect, UiScope, UiTree};

        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        let btn_tag = 888u64;

        {
            let mut scope = UiScope::new(&mut tree, root);
            let _ = scope.button_tagged("Export", btn_tag);
            scope.finish_layout(Rect::new(0.0, 0.0, 800.0, 600.0));
        }

        // Without hover
        let normal = IrisEditorOverlay::collect_text_sections_from_tree(&tree, &[], &[], &[]);
        assert_eq!(normal.len(), 1);
        assert_eq!(normal[0].color, Color::rgba(0.0, 0.88, 1.0, 1.0));

        // With hover on btn_tag
        let hovered = IrisEditorOverlay::collect_text_sections_from_tree_with_hover(
            &tree,
            &[],
            &[],
            &[],
            Some(btn_tag),
        );
        assert_eq!(hovered.len(), 1);
        assert_eq!(hovered[0].color, Color::WHITE);
    }

    #[test]
    fn test_focus_manager_text_input_focus() {
        use crate::ui::iris_bridge::hierarchy::HIERARCHY_TAG_SEARCH_INPUT;
        use irisui::prelude::FocusManager;

        let mut focus_mgr = FocusManager::new();
        assert!(!focus_mgr.has_focus());
        assert_eq!(focus_mgr.focused_tag(), None);

        focus_mgr.set_focus_tag(HIERARCHY_TAG_SEARCH_INPUT);
        assert!(focus_mgr.has_focus());
        assert_eq!(focus_mgr.focused_tag(), Some(HIERARCHY_TAG_SEARCH_INPUT));
        assert!(focus_mgr.is_tag_focused(HIERARCHY_TAG_SEARCH_INPUT));
        assert!(!focus_mgr.is_tag_focused(99999));

        focus_mgr.clear_focus();
        assert!(!focus_mgr.has_focus());
        assert_eq!(focus_mgr.focused_tag(), None);
    }
}