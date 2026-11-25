# Aeon Core

Core engine event bus, gameplay behavior trait contracts, module switches, and thread-safe resource storage re-exports.

## Scope
- **In scope**: `DynamicEventBus`, `EngineEvent`, `Behavior` / `BehaviorContext` / `NativeBehavior`, `EngineMode`, `EngineModule`, and `Resources`.
- **Out of scope**: `winit`, `wgpu`, math crates (`cgmath`/`glam`), physics, rendering, scene storage, and UI.
