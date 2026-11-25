# Aeon State

Stack-based pushdown automaton game state machine (`StateManager`, `GameState`, `StateContext`, `StateTransition`).

## Scope
- **In scope**: State stack transitions (`Push`, `Pop`, `Switch`), `is_pause_state()` hooks, and default playing/paused state types.
- **Out of scope**: Rendering, physics stepping, and editor UI.
