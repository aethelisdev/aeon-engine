# irisui-wgpu

WGPU rendering backend for Iris UI (`#![forbid(unsafe_code)]`):

- Analytical SDF quad, rounded corner, border, and soft-shadow WGSL shader pipeline (`IrisRenderer`).
- Texture array atlas management, external RTT texture blitting, hardware scissor stacking, and offscreen boundary layer caching.

Licensed under `MPL-2.0`.
