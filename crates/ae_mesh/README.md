# Aeon Mesh

GPU-free 3D mesh layouts, procedural primitive generators, CPU linear-blend skinning, bounding spheres, and glTF 2.0 parsing.

## Scope
- **In scope**: `Vertex`, `SpriteVertex`, `SkinVertex`, `SubmeshAlphaMode`, `ParsedModelData`, `compute_aabb_bounding_radius`, `apply_skinning_palette`, primitives, and `gltf` parsing.
- **Out of scope**: `wgpu` buffers/textures (`ae_renderer`) and asset directory management (`ae_assets`).
