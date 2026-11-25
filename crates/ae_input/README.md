# Aeon Input

Three-state physical keyboard/mouse input tracking and JSON-serializable virtual action/axis bindings (`InputManager`, `ActionMap`, `AxisBinding`).

## Scope
- **In scope**: Pressed/just-pressed/just-released tracking, pointer deltas, scroll deltas, and `Config/input.json` serialization.
- **Out of scope**: UI hit-testing (`irisui`), pointer capture arbitration, and window event loops.
