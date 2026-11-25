# Aeon PBR

WGPU 3D PBR pipelines, cascaded shadow map passes, procedural sky, infinite viewport grid, and 3D sprite pipelines.

## Scope
- **In scope**: `PipelineManager`, `ShadowSystem` / `ShadowCascadePass`, `SceneUniforms`, `Instance` / `RenderInstanceItem`, and WGSL forward shaders.
- **Out of scope**: Post-processing bloom/outline (`ae_post_process`), scene culling (`ae_renderer`), and windowing.
