# Aeon Renderer

WGPU rendering engine owning device/surface configuration, GPU resource storage (`AssetManager`), frustum-culled `RenderScene` extraction, and frame passes.

## Scope
- **In scope**: `RenderState` initialized from any `wgpu::SurfaceTarget<'static>`, `RenderSurfaceSize`, GPU mesh/texture/shader caches, forward PBR + shadow + post-process orchestration, and `FloatingOrigin` camera offset helpers.
- **Out of scope**: Windowing (`winit`), UI passes (`irisui` / `ae_ui`), and `gltf` file parsing (`ae_mesh`).
