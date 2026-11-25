# Aeon Export

Standalone project packager providing dependency graph scanning, parallel `Intermediate/` asset cooking, `.aepak` packing, and runtime distribution bundling (`ae_export_cli`).

## Scope
- **In scope**: Asset dependency traversal, `CookedMesh`/`CookedTexture`/`CookedAudio` serialization, 16-byte aligned `.aepak` creation, and target platform launchers.
- **Out of scope**: `wgpu`, `winit`, and editor UI panels.
