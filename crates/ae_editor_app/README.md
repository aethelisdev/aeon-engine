# Aeon Editor App

Desktop Studio IDE executable composing `ae_app`, `ae_editor`, `ae_editor_ui`, `ae_scene`, `ae_export`, and optional `ae_mcp`.

## Scope
- **In scope**: `winit` window lifecycle, CLI arguments, OS file dialogs (`rfd`), `EditorSession`, UI action routing, frame update/render loop, and optional MCP inspection bridge.
- **Out of scope**: Pure ECS component/picking/camera/asset algorithms (owned by `ae_scene`, `ae_editor`, `ae_camera`, `ae_assets`).

## Features
- `mcp`: Enables localhost live UI/world inspection (`ae_mcp`).
- `editor-dialogs`: Enables native OS file dialogs (`rfd`).
- `default`: Enables `mcp` and `editor-dialogs`.
