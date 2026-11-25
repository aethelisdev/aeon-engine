# Aeon Scene

Unified scene lifecycle crate owning binary world serialization, async/VFS world loading, chunk streaming, prefabs, autosave, spawning, and ECS component mutation helpers.

## Scope
- **In scope**: `.aeworld`/`.aechunk` CRC32 diff persistence (`serialization`, `loader`), `RuntimeStreamingManager`, `.aeprefab` templates (`prefab`), Edit-mode `EditorAutosaveState`, `builtin_components`, `components`, `spawning`, `asset_spawning`, `defaults`, and `migrate_ae_scene`.
- **Out of scope**: Editor undo/redo (`ae_editor`), UI notifications (`ae_editor_ui`), OS file dialogs (`rfd`), and windowing (`winit`).
