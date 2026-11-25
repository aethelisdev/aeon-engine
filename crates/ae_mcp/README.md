# Aeon MCP

Live UI inspection server, visual feedback overlay, and host trait (`McpHost`, `McpSession`) for headless inspection and UI diagnostics.

## Scope
- **In scope**: Local JSON-RPC inspection server (`ae_mcp_server`), node tree queries, synthetic pointer/key event injection, and screenshot readback.
- **Out of scope**: Direct ECS/renderer ownership (accessed strictly through the `McpHost` contract).
