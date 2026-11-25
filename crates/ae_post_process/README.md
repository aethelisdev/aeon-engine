# Aeon Post Process

WGPU post-processing passes, MSAA/color/depth render target management, bloom pipeline, and screen-space selection silhouette outlines.

## Scope
- **In scope**: `PostProcessSystem`, bloom extract/blur/composite passes, `SelectionOutlinePass`, MSAA target helpers, and VRAM estimation.
- **Out of scope**: 3D PBR scene passes (`ae_pbr`) and windowing.
