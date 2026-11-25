# Aeon Runtime

Standalone, editor-free exported game player executable layered directly on `ae_app` and `ae_ui`.

## Scope
- **In scope**: Runtime window loop, `.aepak` / VFS world discovery (`world_loader`), embedded pak execution, and static `game_logic` linkage.
- **Out of scope**: Editor state (`ae_editor`), IDE UI (`ae_editor_ui`), OS file dialogs (`rfd`), MCP inspection, and duplicated gameplay loops.

## Features
- `static-game-logic`: Statically links `game_logic` instead of loading a dynamic library at runtime.
