# Aeon App

Application lifecycle, phase schedules, plugin groups, and shared render/simulation hosts for headless and windowed Aeon Engine executables.

## Scope
- **In scope**: `AeApp`, `AppSchedule` / `AppPhase`, `AppPlugin`, `PluginGroup` / `PluginGroupBuilder`, `CoreWorld` (`Resources` + `NonSendResources`), plus optional `RenderHost` and `SimulationHost`.
- **Out of scope**: `winit` windowing, `irisui` widgets, OS file dialogs (`rfd`), inspection (`ae_mcp`), and project export (`ae_export`).

## Features
- `--no-default-features`: Headless core (`AeApp`, schedules, plugins, `CoreWorld`) with zero GPU, physics, or audio dependencies.
- `state`, `physics`, `audio`, `ui`, `plugins`, `rendering`, `2d`, `animation`, `scene`, `gameplay`: Composable subsystem feature gates.
- `default`: Enables `gameplay` and `2d`.
