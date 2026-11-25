# Aeon ECS

Swappable Entity Component System abstraction isolating the workspace from external ECS crate APIs.

## Scope
- **In scope**: `World`, `Entity`, `EntityBuilder`, `EntityCommandBuffer`, `EcsQuery`, scoped `inspect`/`mutate` access, `ComponentRegistry`, and `component_type_hash`.
- **Out of scope**: Engine-specific components (`ae_plugin_api`), rendering, and editor UI.
