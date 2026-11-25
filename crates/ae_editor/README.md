# Aeon Editor

Editor selection, raycast/ground picking, grid snapping, shortcuts, viewport interaction handlers, and Undo/Redo command history.

## Scope
- **In scope**: `EditorState`, `picking` (`create_ray`, `resolve_ground_drop_world_pos`), `snapping`, `undo_redo`, `history`, `shortcuts`, and `actions` (`delete_selected`, `duplicate_selected`, `record_spawn`, `begin_play_session`/`end_play_session`).
- **Out of scope**: `wgpu`, `winit`, `ae_renderer`, `ae_scene`, `ae_assets`, and `irisui`.
