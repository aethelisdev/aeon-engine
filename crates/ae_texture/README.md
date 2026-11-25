# Aeon Texture

CPU-side image decoding, mipmap generation, fallback textures, VFS/GUID texture loading, and file modification watching.

## Scope
- **In scope**: `CpuTextureData`, mipmap chains, `FallbackTextureGenerator`, `TextureFileWatcher`, `parse_texture_file`, and `load_texture_from_vfs`.
- **Out of scope**: `wgpu::Texture` creation (`ae_renderer`) and asset browser UI.
