# Aeon World

ECS-agnostic binary world archives (`.aeworld`), spatial chunk streaming (`.aechunk`), block-compressed archives (`.aepak`), VFS providers, cooked asset formats, and atomic savegames (`.aesave`).

## Scope
- **In scope**: `WorldDocument`, `PartitionCell`, `CellStreamingPlanner`, `PakBuilder` / `PakReader`, `VfsProvider` implementations, `AlignedBuffer`, `WorldDeltaSnapshot`, and `CookedMesh`/`CookedTexture`/`CookedAudio`.
- **Out of scope**: ECS storage (`ae_ecs`), `wgpu` rendering (`ae_renderer`), and async editor tasks (`ae_scene`).
