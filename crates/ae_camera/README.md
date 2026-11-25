# Aeon Camera

GPU-independent camera transform, perspective/orthographic projections, frustum planes, viewport bounds, and uniform payloads.

## Scope
- **In scope**: `Camera`, `ProjectionMode`, `Frustum`, `CameraUniform`, `ViewportRect`, and `OPENGL_TO_WGPU_MATRIX`.
- **Out of scope**: `wgpu`, `ae_ecs`, `ae_core`, `ae_world`, and input handling (`ae_camera_controller`).
