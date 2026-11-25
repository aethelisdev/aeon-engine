# Aeon Transform

Unified transform matrix composition/decomposition, `cgmath` ↔ `glam` conversions, hierarchy propagation, keep-world reparenting, and spatial grid partitioning.

## Scope
- **In scope**: `compose_local_matrix`, `extract_trs_columns`, `update_hierarchy_transforms`, `parent_entity_keep_world_transform`, `unparent_entity_keep_world_transform`, and `SpatialGrid`.
- **Out of scope**: Rendering (`ae_renderer`), picking (`ae_editor`), and editor undo history.
