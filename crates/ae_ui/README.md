# Aeon UI

In-game ECS UI layout resolution, gameplay event HUD synchronization, pause menu overlays, and isolated GPU UI pass rendering.

## Scope
- **In scope**: `UiLayoutResolver`, `InGameHudState`, `InGamePauseState`, shared pause/HUD builders (`iris_builder`), `IrisRenderPassParams` / `iris_render_pass`, and `RuntimeHudOverlay`.
- **Out of scope**: `winit` windowing, `ae_renderer`, `ae_editor`, and desktop IDE panels (`ae_editor_ui`).
