# Aeon Light

GPU-free lighting components, cascaded shadow projection math, procedural atmosphere/fog evaluation, and aligned GPU uniform layouts.

## Scope
- **In scope**: `Light` component, `ShadowResolution` / `PcfQuality` / `SkyQuality`, 4-cascade CSM matrices, and 16-byte aligned uniforms (`LightUniform`, `LightSpaceUniform`, `SkyUniform`).
- **Out of scope**: `wgpu` device handles, render pass execution (`ae_pbr`), and ECS world management.
