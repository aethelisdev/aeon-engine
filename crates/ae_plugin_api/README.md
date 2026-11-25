# Aeon Plugin API

Shared data types, POD ECS components, entity/asset GUIDs, dynamic resource map (`Resources`), `EngineModule` switches, and FFI ABI definitions.

## Scope
- **In scope**: `EntityGuid`, `AssetGuid`, `component_type_hash`, POD gameplay/physics/UI components, `engine_component_registry()`, and `PluginContextFFI`.
- **Out of scope**: Plugin loading (`ae_plugin_host`), rendering, and windowing.
